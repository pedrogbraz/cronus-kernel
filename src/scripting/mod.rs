//! .scriptcronus — imperative scripting layer for CRONUS
//!
//! Bridges external apps/languages to the CRONUS kernel.
//! Event-driven, sandboxed per user, with db/http/sse builtins.

pub mod ast;
pub mod parser;
pub mod vm;

use ast::{ScriptBlock, ScriptFile};
use vm::{EventData, ScriptContext};
use std::collections::HashMap;
use std::sync::Arc;

/// Registry of loaded .scriptcronus files
#[derive(Debug, Clone)]
pub struct ScriptRegistry {
    pub scripts: Vec<ScriptFile>,
}

impl ScriptRegistry {
    pub fn new() -> Self {
        Self { scripts: Vec::new() }
    }

    /// Load and parse all .scriptcronus files from a directory
    pub fn load_from_directory(dir: &str) -> Self {
        let mut registry = Self::new();
        let path = std::path::Path::new(dir);
        if !path.is_dir() { return registry; }

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.extension().and_then(|e| e.to_str()) == Some("scriptcronus") {
                    match std::fs::read_to_string(&file_path) {
                        Ok(source) => {
                            match parser::ScriptParser::parse(&source) {
                                Ok(script) => {
                                    eprintln!("  \x1b[32m✓\x1b[0m Script: {} ({})", script.name, file_path.display());
                                    registry.scripts.push(script);
                                }
                                Err(e) => {
                                    eprintln!("  \x1b[31m✗\x1b[0m Script parse error in {}: {}", file_path.display(), e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("  \x1b[31m✗\x1b[0m Could not read {}: {}", file_path.display(), e);
                        }
                    }
                }
            }
        }
        registry
    }

    /// Get all OnEvent blocks matching an entity+event pair
    pub fn get_event_handlers(&self, entity: &str, event: &str) -> Vec<(&ScriptFile, &ast::OnEventBlock)> {
        let mut handlers = Vec::new();
        for script in &self.scripts {
            for block in &script.blocks {
                if let ScriptBlock::OnEvent(on) = block {
                    if on.entity == entity && on.event == event {
                        handlers.push((script, on));
                    }
                }
            }
        }
        handlers
    }

    /// Get all webhook handlers for a path
    pub fn get_webhook_handlers(&self, path: &str) -> Vec<(&ScriptFile, &ast::OnWebhookBlock)> {
        let mut handlers = Vec::new();
        for script in &self.scripts {
            for block in &script.blocks {
                if let ScriptBlock::OnWebhook(wh) = block {
                    if wh.path == path {
                        handlers.push((script, wh));
                    }
                }
            }
        }
        handlers
    }

    /// Get all custom endpoints
    pub fn get_endpoints(&self) -> Vec<(&ScriptFile, &ast::EndpointBlock)> {
        let mut endpoints = Vec::new();
        for script in &self.scripts {
            for block in &script.blocks {
                if let ScriptBlock::Endpoint(ep) = block {
                    endpoints.push((script, ep));
                }
            }
        }
        endpoints
    }

    /// Get all schedules
    pub fn get_schedules(&self) -> Vec<(&ScriptFile, &ast::ScheduleBlock)> {
        let mut schedules = Vec::new();
        for script in &self.scripts {
            for block in &script.blocks {
                if let ScriptBlock::Schedule(sched) = block {
                    schedules.push((script, sched));
                }
            }
        }
        schedules
    }

    /// Count total blocks
    pub fn block_count(&self) -> usize {
        self.scripts.iter().map(|s| s.blocks.len()).sum()
    }
}

/// Fire all matching script handlers for an entity event.
/// Called from the CRUD pipeline after fire_effects() and fire_webhooks().
pub fn fire_scripts(
    registry: &ScriptRegistry,
    entity: &str,
    event_type: &str,
    record: &serde_json::Value,
    record_id: &str,
    prev_record: Option<&serde_json::Value>,
    db: &crate::database::CronusDB,
    user_id: &str,
    role: &str,
    env_vars: &HashMap<String, String>,
) {
    let handlers = registry.get_event_handlers(entity, event_type);
    if handlers.is_empty() { return; }

    let event_data = EventData {
        entity: entity.to_string(),
        event: event_type.to_string(),
        record: record.clone(),
        record_id: record_id.to_string(),
        prev_record: prev_record.cloned(),
    };

    for (script, on_block) in handlers {
        let mut ctx = ScriptContext::new(user_id, role, env_vars.clone());
        let block_id = format!("{}.{}.{}", script.name, entity, event_type);
        eprintln!("  \x1b[36m[script]\x1b[0m firing {}.{} from \"{}\"", entity, event_type, script.name);
        let start = std::time::Instant::now();
        let result = vm::execute_statements(&on_block.body, &mut ctx, db, Some(&event_data));
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        match result {
            Ok(()) => {
                // Track successful execution for trust engine
                crate::trust::track_execution(&block_id, elapsed_ms, None);
            }
            Err(ref e) => {
                crate::trust::track_execution(&block_id, elapsed_ms, Some(e));
                eprintln!("  \x1b[31m[script]\x1b[0m error in {}.{}: {}", entity, event_type, e);
            }
        }
    }
}

/// Execute a webhook handler and return the script context.
/// SECURITY: webhooks run as "webhook" role with limited DB access, NOT as admin.
/// The webhook can only create records (insert), not update/delete existing ones
/// unless the script explicitly handles it with proper validation.
pub fn execute_webhook(
    registry: &ScriptRegistry,
    path: &str,
    body: &serde_json::Value,
    db: &crate::database::CronusDB,
    env_vars: &HashMap<String, String>,
) -> Option<ScriptContext> {
    let handlers = registry.get_webhook_handlers(path);
    if handlers.is_empty() { return None; }

    // SECURITY: limit webhook body size to prevent memory exhaustion
    let body_str = body.to_string();
    if body_str.len() > 1_048_576 { // 1MB max
        eprintln!("  \x1b[31m[script]\x1b[0m webhook {} REJECTED — body too large ({} bytes)", path, body_str.len());
        return None;
    }

    let event_data = EventData {
        entity: "webhook".to_string(),
        event: "receive".to_string(),
        record: body.clone(),
        record_id: String::new(),
        prev_record: None,
    };

    let (script, wh_block) = handlers[0]; // first match
    // SECURITY: webhooks run as "webhook" role, NOT admin — owner isolation applies
    let mut ctx = ScriptContext::new("webhook", "webhook", env_vars.clone());
    let block_id = format!("{}.webhook.{}", script.name, path);
    eprintln!("  \x1b[36m[script]\x1b[0m webhook {} from \"{}\"", path, script.name);
    let start = std::time::Instant::now();
    let result = vm::execute_statements(&wh_block.body, &mut ctx, db, Some(&event_data));
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    match result {
        Ok(()) => {
            crate::trust::track_execution(&block_id, elapsed, None);
            Some(ctx)
        }
        Err(e) => {
            crate::trust::track_execution(&block_id, elapsed, Some(&e));
            eprintln!("  \x1b[31m[script]\x1b[0m webhook error: {}", e);
            None
        }
    }
}

/// Execute a custom endpoint handler
pub fn execute_endpoint(
    endpoint: &ast::EndpointBlock,
    script_name: &str,
    db: &crate::database::CronusDB,
    user_id: &str,
    role: &str,
    body: Option<&serde_json::Value>,
    env_vars: &HashMap<String, String>,
) -> ScriptContext {
    let event_data = body.map(|b| EventData {
        entity: "endpoint".to_string(),
        event: "request".to_string(),
        record: b.clone(),
        record_id: String::new(),
        prev_record: None,
    });

    let mut ctx = ScriptContext::new(user_id, role, env_vars.clone());
    let block_id = format!("{}.endpoint.{}.{}", script_name, endpoint.method, endpoint.path);
    eprintln!("  \x1b[36m[script]\x1b[0m endpoint {} {} from \"{}\"", endpoint.method, endpoint.path, script_name);
    let start = std::time::Instant::now();
    match vm::execute_statements(&endpoint.body, &mut ctx, db, event_data.as_ref()) {
        Ok(()) => {
            crate::trust::track_execution(&block_id, start.elapsed().as_secs_f64() * 1000.0, None);
            ctx
        }
        Err(e) => {
            crate::trust::track_execution(&block_id, start.elapsed().as_secs_f64() * 1000.0, Some(&e));
            eprintln!("  \x1b[31m[script]\x1b[0m endpoint error: {}", e);
            ctx.response = Some(vm::ScriptResponse {
                status: 500,
                body: format!("{{\"error\":\"{}\"}}", e),
                headers: HashMap::new(),
            });
            ctx
        }
    }
}

/// Parse interval string to Duration
/// Supports: "30s", "5m", "1h", "1d"
pub fn parse_interval(s: &str) -> std::time::Duration {
    let s = s.trim();
    if let Some(num) = s.strip_suffix('s') {
        std::time::Duration::from_secs(num.parse().unwrap_or(60))
    } else if let Some(num) = s.strip_suffix('m') {
        std::time::Duration::from_secs(num.parse::<u64>().unwrap_or(1) * 60)
    } else if let Some(num) = s.strip_suffix('h') {
        std::time::Duration::from_secs(num.parse::<u64>().unwrap_or(1) * 3600)
    } else if let Some(num) = s.strip_suffix('d') {
        std::time::Duration::from_secs(num.parse::<u64>().unwrap_or(1) * 86400)
    } else {
        std::time::Duration::from_secs(3600) // default 1h
    }
}
