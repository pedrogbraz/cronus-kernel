#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS HMR Engine — Hot Module Replacement without Vite
//!
//! File watcher + version polling for instant browser reload.
//! No WebSocket needed — simple HTTP polling every 500ms.

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

/// HMR client script — polls /__cronus/version every 500ms
/// Pauses during SPA navigation to avoid false reloads.
pub const HMR_CLIENT_JS: &str = r#"
(function(){
  var v=0;
  window.__hmrPaused=false;
  setInterval(async function(){
    if(window.__hmrPaused)return;
    try{
      var res=await fetch('/__cronus/version');
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
                    if !name.ends_with(".cronus") { continue; }

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

/// Start file watcher — polls file metadata every 500ms
/// Calls `on_change` when the file's modified time changes.
pub fn start_watcher(path: &str, on_change: impl Fn() + Send + 'static) {
    let path = path.to_string();
    std::thread::spawn(move || {
        let mut last_modified = std::fs::metadata(&path)
            .and_then(|m| m.modified())
            .ok();

        loop {
            std::thread::sleep(Duration::from_millis(500));

            let current = std::fs::metadata(&path)
                .and_then(|m| m.modified())
                .ok();

            if current != last_modified {
                last_modified = current;
                on_change();
            }
        }
    });
}
