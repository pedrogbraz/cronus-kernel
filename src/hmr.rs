//! CRONUS HMR Engine — Hot Module Replacement without Vite
//!
//! File watcher reloads the spec into the live `AppState`, then bumps
//! `BUILD_VERSION` so the polling client (`HMR_CLIENT_JS`, every 500ms)
//! refreshes and sees the new AST. No WebSocket.

use crate::parser::EntityNode;
use crate::server::state::AppState;
use std::sync::Arc;

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Build version — incremented on every .cronus file change
static BUILD_VERSION: AtomicU64 = AtomicU64::new(0);

/// Bump version after a file change, returns new version
pub fn bump_version() -> u64 {
    BUILD_VERSION.fetch_add(1, Ordering::SeqCst) + 1
}

/// Get current build version
pub fn current_version() -> u64 {
    BUILD_VERSION.load(Ordering::SeqCst)
}

/// HMR client script — polls /.cronus/version every 500ms
/// Pauses during SPA navigation to avoid false reloads.
pub const HMR_CLIENT_JS: &str = r#"
(function(){
  var v=0;
  window.__hmrPaused=false;
  setInterval(async function(){
    if(window.__hmrPaused)return;
    try{
      var res=await fetch('/.cronus/version');
      var data=await res.json();
      if(v>0&&data.version!==v){
        console.log('[CRONUS HMR] File changed, reloading...');
        window.location.reload();
      }
      v=data.version;
    }catch(e){}
  },500);
})();
"#;

/// Watch all .cronus files in a directory for changes.
/// Calls `on_change` when any file's modified time changes.
pub fn start_directory_watcher(dir: &str, on_change: impl Fn() + Send + 'static) {
    let dir = dir.to_string();
    std::thread::spawn(move || {
        let mut last_state: std::collections::HashMap<String, std::time::SystemTime> =
            std::collections::HashMap::new();

        // Initial scan
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".cronus") {
                    if let Ok(meta) = entry.metadata() {
                        if let Ok(modified) = meta.modified() {
                            last_state.insert(name, modified);
                        }
                    }
                }
            }
        }

        loop {
            std::thread::sleep(Duration::from_millis(500));

            let mut changed = false;
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.ends_with(".cronus") {
                        continue;
                    }

                    if let Ok(meta) = entry.metadata() {
                        if let Ok(modified) = meta.modified() {
                            match last_state.get(&name) {
                                Some(prev) if *prev != modified => {
                                    changed = true;
                                    last_state.insert(name, modified);
                                }
                                None => {
                                    // New file added
                                    changed = true;
                                    last_state.insert(name, modified);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            if changed {
                on_change();
            }
        }
    });
}

/// Rebuild `AppState` after a spec change. Shares the previous process's
/// database, SSE hub, rate limiters, audit trail, traces, Zeus buffer and
/// brain. Migrates the new entity list (indexes included) onto the live DB.
pub(crate) fn reload_state(prev: &AppState, entities: Vec<EntityNode>) -> Result<AppState, String> {
    prev.db.migrate(&entities)?;
    Ok(AppState {
        app: prev.app.clone(),
        entities,
        pages: prev.pages.clone(),
        components: prev.components.clone(),
        style: prev.style.clone(),
        apis: prev.apis.clone(),
        db_path: prev.db_path.clone(),
        db: prev.db.clone(),
        brain: prev.brain.clone(),
        auth_entity: prev.auth_entity.clone(),
        auth_roles: prev.auth_roles.clone(),
        auth_required_pages: prev.auth_required_pages.clone(),
        auth_redirect: prev.auth_redirect.clone(),
        session_policy: crate::auth::SessionPolicy {
            ttl_secs: prev.session_policy.ttl_secs,
        },
        layout: prev.layout.clone(),
        webhooks: prev.webhooks.clone(),
        rate_limiter: prev.rate_limiter.clone(),
        auth_rate_limiter: prev.auth_rate_limiter.clone(),
        sse_hub: Arc::clone(&prev.sse_hub),
        audit_trail: prev.audit_trail.clone(),
        trace_buffer: Arc::clone(&prev.trace_buffer),
        script_registry: prev.script_registry.clone(),
        zeus: Arc::clone(&prev.zeus),
    })
}

/// Apply a fully rebuilt spec onto the previous runtime (shared resources
/// taken from `prev`; `next` supplies pages, entities, scripts, …).
pub(crate) fn adopt_spec(prev: &AppState, mut next: AppState) -> Result<AppState, String> {
    next.db_path = prev.db_path.clone();
    next.db = prev.db.clone();
    next.brain = prev.brain.clone();
    next.rate_limiter = prev.rate_limiter.clone();
    next.auth_rate_limiter = prev.auth_rate_limiter.clone();
    next.sse_hub = Arc::clone(&prev.sse_hub);
    next.audit_trail = prev.audit_trail.clone();
    next.trace_buffer = Arc::clone(&prev.trace_buffer);
    next.zeus = Arc::clone(&prev.zeus);
    next.db.migrate(&next.entities)?;
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{parse, AstNode};
    use serde_json::json;

    #[test]
    fn bump_version_increments() {
        let a = current_version();
        let b = bump_version();
        assert_eq!(b, a + 1);
        assert_eq!(current_version(), b);
    }

    #[test]
    fn reload_state_swaps_entities_on_shared_db() {
        let prev = crate::api_security_tests::state_from(
            r#"app "H" { port 5175 }
entity Note { title string! }
"#,
        );
        prev.db
            .insert("Note", &json!({"title": "kept"}))
            .expect("insert");
        let entities: Vec<EntityNode> = parse(
            r#"app "H" { port 5175 }
entity Note { title string! }
entity Task { title string! index }
"#,
        )
        .expect("parse")
        .into_iter()
        .filter_map(|n| match n {
            AstNode::Entity(e) => Some(e),
            _ => None,
        })
        .collect();
        let next = reload_state(&prev, entities).expect("reload");
        assert!(next.entities.iter().any(|e| e.name == "Task"));
        assert_eq!(next.entities.len(), 2);
        assert_eq!(next.db.count("Note").unwrap(), 1, "db connection is shared");
        assert!(next.db.count("Task").is_ok());
        let names = next
            .db
            .query_raw("SELECT name FROM sqlite_master WHERE type='index'")
            .unwrap()
            .iter()
            .filter_map(|r| r.get("name").and_then(|v| v.as_str()).map(str::to_string))
            .collect::<Vec<_>>();
        assert!(
            names.iter().any(|n| n == "idx_Task_title"),
            "new field index missing: {names:?}"
        );
        assert!(names.iter().any(|n| n == "idx_Task__owner_id"));
    }
}

/// Start file watcher — polls file metadata every 500ms
/// Calls `on_change` when the file's modified time changes.
pub fn start_watcher(path: &str, on_change: impl Fn() + Send + 'static) {
    let path = path.to_string();
    std::thread::spawn(move || {
        let mut last_modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok();

        loop {
            std::thread::sleep(Duration::from_millis(500));

            let current = std::fs::metadata(&path).and_then(|m| m.modified()).ok();

            if current != last_modified {
                last_modified = current;
                on_change();
            }
        }
    });
}
