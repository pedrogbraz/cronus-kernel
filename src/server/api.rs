#![allow(dead_code, unused_imports)]
//! Entity CRUD handler and related helpers extracted from main.rs.

use std::sync::Arc;

use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, Response, StatusCode};
use serde_json::{json, Value};

use crate::parser::{self, EntityNode};
use crate::brain;
use crate::sse;

use super::response::json_response;
use super::state::AppState;

pub(crate) fn handle_api(method: &Method, path: &str, body: Option<&serde_json::Value>, state: &AppState, owner_id: &str) -> Response<Full<Bytes>> {
    // Split path and query string
    let full_api = &path[4..]; // strip /api
    let (api_path, query_string) = match full_api.split_once('?') {
        Some((p, q)) => (p, q),
        None => (full_api, ""),
    };

    // Parse query params
    let params: Vec<(&str, &str)> = query_string.split('&')
        .filter(|p| !p.is_empty())
        .filter_map(|p| p.split_once('='))
        .collect();
    let get_param = |name: &str| -> Option<&str> {
        params.iter().find(|(k, _)| *k == name).map(|(_, v)| *v)
    };

    let limit: usize = get_param("limit").and_then(|v| v.parse().ok()).unwrap_or(100);
    let offset: usize = get_param("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    // Find matching entity by pluralized name in path
    let entity = state.entities.iter().find(|e| {
        let lower = e.name.to_lowercase();
        api_path.starts_with(&format!("/{}", lower))
            || api_path.starts_with(&format!("/{}s", lower))
    });

    if let Some(entity) = entity {
        let table = &entity.name;
        let segments: Vec<&str> = api_path.split('/').filter(|s| !s.is_empty()).collect();

        // SECURITY: Build owner filter for data isolation
        // Skip User entity and shared entities (visible to all authenticated users)
        let is_user_entity = table.to_lowercase() == "user" || table.to_lowercase() == "users";
        let is_shared = entity.shared;
        let owner_filter: Vec<(String, String, String)> = if !owner_id.is_empty() && !is_user_entity && !is_shared {
            vec![("_owner_id".to_string(), "=".to_string(), owner_id.to_string())]
        } else {
            vec![]
        };

        match *method {
            Method::GET => {
                if segments.len() >= 2 {
                    // GET /api/entity/:id -- verify ownership
                    match state.db.find_by_id(table, segments[1]) {
                        Ok(Some(val)) => {
                            // SECURITY: Check owner match (skip for shared entities)
                            if !owner_id.is_empty() && !is_user_entity && !is_shared {
                                let row_owner = val.get("_owner_id").and_then(|v| v.as_str()).unwrap_or("");
                                if !row_owner.is_empty() && row_owner != owner_id {
                                    return json_response(StatusCode::NOT_FOUND, json!({"error": "not found"}));
                                }
                            }
                            json_response(StatusCode::OK, val)
                        }
                        Ok(None) => json_response(StatusCode::NOT_FOUND, json!({"error": "not found"})),
                        Err(e) => json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e})),
                    }
                } else {
                    // GET /api/entity?search=term&limit=N&offset=M
                    let search_query = get_param("search").or(get_param("q"));

                    if let Some(q) = search_query {
                        // Search mode -- filtered by owner
                        match state.db.find_many(table, &owner_filter, None, None, Some(limit), None) {
                            Ok(Value::Array(rows)) => {
                                let filtered: Vec<Value> = rows.into_iter().filter(|r| {
                                    let txt = r.to_string().to_lowercase();
                                    txt.contains(&q.to_lowercase())
                                }).collect();
                                return json_response(StatusCode::OK, Value::Array(filtered));
                            }
                            _ => return json_response(StatusCode::OK, json!([])),
                        }
                    }

                    // Paginated list -- SECURITY: filtered by owner_id
                    match state.db.find_many(table, &owner_filter, None, None, Some(limit), Some(offset)) {
                        Ok(rows) => {
                            let count = if let Value::Array(ref arr) = rows { arr.len() } else { 0 };
                            Response::builder()
                                .status(StatusCode::OK)
                                .header("Content-Type", "application/json")
                                .header("Access-Control-Expose-Headers", "X-Total-Count, X-Limit, X-Offset")
                                .header("X-Total-Count", count.to_string())
                                .header("X-Limit", limit.to_string())
                                .header("X-Offset", offset.to_string())
                                .body(Full::new(Bytes::from(rows.to_string())))
                                .unwrap()
                        }
                        Err(_) => json_response(StatusCode::OK, json!([])),
                    }
                }
            }
            Method::POST => {
                match body {
                    Some(data) => {
                        // SECURITY: Inject _owner_id automatically
                        let mut owned_data = data.clone();
                        if !owner_id.is_empty() && !is_user_entity {
                            if let Some(obj) = owned_data.as_object_mut() {
                                obj.insert("_owner_id".to_string(), json!(owner_id));
                            }
                        }
                        let data = &owned_data;

                        // Find entity definition for validation
                        let entity_def = state.entities.iter().find(|e| e.name.to_lowercase() == table.to_lowercase());
                        match entity_def {
                            Some(entity) => match state.db.validated_insert(entity, data) {
                                Ok(row) => {
                                    fire_webhooks(&state.webhooks, table, "create", &row);
                                    fire_effects(entity, "create", &row, None, &state.brain, &state.sse_hub);
                                    let row_id_for_script = row.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    crate::scripting::fire_scripts(&state.script_registry, &entity.name, "create", &row, &row_id_for_script, None, &state.db, owner_id, "user", &std::collections::HashMap::new());
                                    let row_id = row.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    if let Err(e) = state.audit_trail.log("INSERT", table, &row_id, owner_id, &row, None) {
                                        eprintln!("  \x1b[33m⚠\x1b[0m Audit log failed (INSERT {}:{}): {}", table, row_id, e);
                                    }
                                    state.sse_hub.broadcast(sse::DataChangeEvent {
                                        entity: table.to_string(),
                                        action: "created".to_string(),
                                        id: row_id,
                                    });
                                    json_response(StatusCode::CREATED, row)
                                }
                                Err(e) => {
                                    let status = if e.contains("required") {
                                        StatusCode::BAD_REQUEST // 400
                                    } else if e.contains("already exists") {
                                        StatusCode::CONFLICT // 409
                                    } else if e.contains("must be") {
                                        StatusCode::UNPROCESSABLE_ENTITY // 422
                                    } else {
                                        StatusCode::BAD_REQUEST
                                    };
                                    json_response(status, json!({"error": e}))
                                }
                            },
                            None => match state.db.insert(table, data) {
                                Ok(row) => {
                                    fire_webhooks(&state.webhooks, table, "create", &row);
                                    fire_effects(entity, "create", &row, None, &state.brain, &state.sse_hub);
                                    let row_id = row.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    if let Err(e) = state.audit_trail.log("INSERT", table, &row_id, owner_id, &row, None) {
                                        eprintln!("  \x1b[33m⚠\x1b[0m Audit log failed (INSERT {}:{}): {}", table, row_id, e);
                                    }
                                    state.sse_hub.broadcast(sse::DataChangeEvent {
                                        entity: table.to_string(),
                                        action: "created".to_string(),
                                        id: row_id,
                                    });
                                    json_response(StatusCode::CREATED, row)
                                }
                                Err(e) => json_response(StatusCode::BAD_REQUEST, json!({"error": e})),
                            }
                        }
                    },
                    None => json_response(StatusCode::BAD_REQUEST, json!({"error": "expected JSON body"})),
                }
            }
            Method::PATCH | Method::PUT => {
                if segments.len() >= 2 {
                    match body {
                        Some(data) => {
                            // Fetch current record before update for audit diff tracking
                            let prev_record = state.db.find_by_id(table, segments[1]).ok().flatten();

                            // Validate state transitions if entity has transition rules
                            if !entity.transitions.is_empty() {
                                if let Some(ref current) = prev_record {
                                    if let Err(err_body) = validate_transitions(entity, data, current) {
                                        return json_response(StatusCode::CONFLICT, err_body);
                                    }
                                }
                            }

                            match state.db.update(table, segments[1], data) {
                            Ok(row) => {
                                fire_webhooks(&state.webhooks, table, "update", &row);
                                fire_effects(entity, "update", &row, prev_record.as_ref(), &state.brain, &state.sse_hub);
                                crate::scripting::fire_scripts(&state.script_registry, &entity.name, "update", &row, segments[1], prev_record.as_ref(), &state.db, owner_id, "user", &std::collections::HashMap::new());
                                if let Err(e) = state.audit_trail.log("UPDATE", table, segments[1], owner_id, &row, prev_record.as_ref()) {
                                    eprintln!("  \x1b[33m⚠\x1b[0m Audit log failed (UPDATE {}:{}): {}", table, segments[1], e);
                                }
                                state.sse_hub.broadcast(sse::DataChangeEvent {
                                    entity: table.to_string(),
                                    action: "updated".to_string(),
                                    id: segments[1].to_string(),
                                });
                                json_response(StatusCode::OK, row)
                            }
                            Err(e) => json_response(StatusCode::BAD_REQUEST, json!({"error": e})),
                        }},
                        None => json_response(StatusCode::BAD_REQUEST, json!({"error": "expected JSON body"})),
                    }
                } else {
                    json_response(StatusCode::BAD_REQUEST, json!({"error": "id required for PATCH"}))
                }
            }
            Method::DELETE => {
                if segments.len() >= 2 {
                    // Fetch current record before delete for audit trail
                    let prev_record = state.db.find_by_id(table, segments[1]).ok().flatten();
                    match state.db.delete(table, segments[1]) {
                        Ok(true) => {
                            let delete_payload = json!({"id": segments[1], "entity": table});
                            fire_webhooks(&state.webhooks, table, "delete", &delete_payload);
                            // For effects, use prev_record if available (has field values for interpolation)
                            let effect_record = prev_record.as_ref().unwrap_or(&delete_payload);
                            fire_effects(entity, "delete", effect_record, None, &state.brain, &state.sse_hub);
                            let entity_name = entity.map(|e| e.name.as_str()).unwrap_or(table);
                            crate::scripting::fire_scripts(&state.script_registry, entity_name, "delete", effect_record, segments[1], None, &state.db, owner_id, "user", &std::collections::HashMap::new());
                            if let Err(e) = state.audit_trail.log("DELETE", table, segments[1], owner_id, &json!({"id": segments[1]}), prev_record.as_ref()) {
                                eprintln!("  \x1b[33m⚠\x1b[0m Audit log failed (DELETE {}:{}): {}", table, segments[1], e);
                            }
                            state.sse_hub.broadcast(sse::DataChangeEvent {
                                entity: table.to_string(),
                                action: "deleted".to_string(),
                                id: segments[1].to_string(),
                            });
                            json_response(StatusCode::OK, json!({"deleted": segments[1]}))
                        }
                        Ok(false) => json_response(StatusCode::NOT_FOUND, json!({"error": "not found"})),
                        Err(e) => json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e})),
                    }
                } else {
                    json_response(StatusCode::BAD_REQUEST, json!({"error": "id required"}))
                }
            }
            _ => json_response(StatusCode::METHOD_NOT_ALLOWED, json!({"error": "method not allowed"})),
        }
    } else {
        json_response(StatusCode::NOT_FOUND, json!({"error": "unknown endpoint", "path": path}))
    }
}

/// Validate that a field transition is allowed by the entity's transition rules.
/// Returns Ok(()) if no transition rules apply or if the transition is valid.
/// Returns Err with a JSON value containing the error details if the transition is invalid.
pub(crate) fn validate_transitions(
    entity: &EntityNode,
    update_data: &Value,
    current_record: &Value,
) -> Result<(), Value> {
    for transition in &entity.transitions {
        let field = &transition.field;

        // Check if the update data includes this transition field
        let new_value = match update_data.get(field).and_then(|v| v.as_str()) {
            Some(v) => v,
            None => continue, // Field not being updated, skip
        };

        // Get the current value from the existing record
        let old_value = current_record
            .get(field)
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // If old == new, no transition needed
        if old_value == new_value {
            continue;
        }

        // Find the rule for this old_value
        let allowed: Vec<&str> = transition.rules.iter()
            .filter(|r| r.from == old_value)
            .flat_map(|r| r.to.iter().map(|s| s.as_str()))
            .collect();

        // If no rules found for the current state, check if it's a wildcard "*" rule
        let allowed = if allowed.is_empty() {
            transition.rules.iter()
                .filter(|r| r.from == "*")
                .flat_map(|r| r.to.iter().map(|s| s.as_str()))
                .collect()
        } else {
            allowed
        };

        // If there are rules but the new value is not in the allowed targets, reject
        if !allowed.is_empty() && !allowed.contains(&new_value) {
            return Err(json!({
                "error": format!(
                    "Invalid transition: {} '{}' -> '{}' is not allowed",
                    field, old_value, new_value
                ),
                "allowed": allowed,
                "field": field,
                "current": old_value,
                "requested": new_value,
            }));
        }

        // If no rules match the current state at all (not even wildcard), the transition is unconstrained
        // (no rule = no restriction for that source state)
    }
    Ok(())
}

/// Fire webhooks in background for a given entity + event.
/// Sends the payload as JSON body to each matching webhook URL.
pub(crate) fn fire_webhooks(webhooks: &[parser::WebhookNode], entity: &str, event: &str, payload: &serde_json::Value) {
    let entity_lower = entity.to_lowercase();
    for wh in webhooks {
        let wh_entity = wh.entity.to_lowercase();
        // Match entity name (with or without trailing 's')
        if wh_entity != entity_lower
            && format!("{}s", wh_entity) != entity_lower
            && wh_entity != format!("{}s", entity_lower) {
            continue;
        }
        for hook in &wh.hooks {
            if hook.event != event { continue; }
            let url = hook.url.clone();
            let method = hook.method.clone();
            let payload = payload.clone();
            let headers: Vec<(String, String)> = hook.headers.clone();
            // Spawn background task -- fire and forget
            tokio::spawn(async move {
                let client_result = tokio::net::TcpStream::connect(
                    url.trim_start_matches("http://")
                       .trim_start_matches("https://")
                       .split('/')
                       .next()
                       .unwrap_or("")
                ).await;
                // Use a simple HTTP request via hyper or raw TCP
                // For robustness, just use the process's own fetch
                let body_str = payload.to_string();
                let req_body = format!(
                    "{method} {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {len}\r\n{extra_headers}\r\n{body}",
                    method = method,
                    path = url.find('/').map(|_| {
                        let after_scheme = url.trim_start_matches("http://").trim_start_matches("https://");
                        after_scheme.find('/').map(|i| &after_scheme[i..]).unwrap_or("/")
                    }).unwrap_or("/"),
                    host = url.trim_start_matches("http://").trim_start_matches("https://").split('/').next().unwrap_or(""),
                    len = body_str.len(),
                    extra_headers = headers.iter().map(|(k,v)| format!("{}: {}\r\n", k, v)).collect::<String>(),
                    body = body_str,
                );
                if let Ok(mut stream) = client_result {
                    use tokio::io::AsyncWriteExt;
                    let _ = stream.write_all(req_body.as_bytes()).await;
                } else {
                    eprintln!("  \x1b[33m⚠\x1b[0m Webhook failed: {}", url);
                }
            });
        }
    }
}

/// Execute entity effect blocks after create/update/delete.
/// Interpolates `{{field}}` placeholders with record values.
/// For "on update <field>" effects, checks if the field changed and matches `when` conditions.
pub(crate) fn fire_effects(
    entity: &parser::EntityNode,
    event: &str,
    record: &serde_json::Value,
    prev_record: Option<&serde_json::Value>,
    brain: &Option<brain::CronusBrain>,
    sse_hub: &Arc<sse::SseHub>,
) {
    for effect in &entity.effects {
        if effect.event != event {
            continue;
        }

        // For "on update <field>" -- check if the specific field changed
        if event == "update" {
            if let Some(ref watched_field) = effect.field {
                let new_val = record.get(watched_field).and_then(|v| v.as_str()).unwrap_or("");
                let old_val = prev_record
                    .and_then(|p| p.get(watched_field))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if new_val == old_val {
                    continue; // field didn't change, skip this effect block
                }

                // Process actions, respecting `when` conditions
                for action in &effect.actions {
                    if let Some(ref condition) = action.condition {
                        if condition != new_val {
                            continue; // condition doesn't match current value
                        }
                    }
                    execute_effect_action(action, entity_name_str(&entity.name), record, brain, sse_hub);
                }
                continue;
            }
        }

        // For "on create" / "on delete" / "on update" (no specific field) -- run all actions
        for action in &effect.actions {
            execute_effect_action(action, entity_name_str(&entity.name), record, brain, sse_hub);
        }
    }
}

fn entity_name_str(name: &str) -> &str {
    name
}

/// Interpolate `{{field_name}}` placeholders in a message with actual record values.
pub(crate) fn interpolate_effect_message(template: &str, record: &serde_json::Value) -> String {
    let mut result = template.to_string();
    // Find all {{field}} patterns and replace with record values
    while let Some(start) = result.find("{{") {
        if let Some(end) = result[start..].find("}}") {
            let field_name = &result[start + 2..start + end];
            let value = record
                .get(field_name)
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .unwrap_or_default();
            result = format!("{}{}{}", &result[..start], value, &result[start + end + 2..]);
        } else {
            break;
        }
    }
    result
}

/// Execute a single effect action (log or notify).
pub(crate) fn execute_effect_action(
    action: &parser::EffectAction,
    entity_name: &str,
    record: &serde_json::Value,
    brain: &Option<brain::CronusBrain>,
    sse_hub: &Arc<sse::SseHub>,
) {
    match action.action_type.as_str() {
        "log" => {
            if let Some(msg_template) = action.args.first() {
                let message = interpolate_effect_message(msg_template, record);
                eprintln!("  \x1b[36m[effect]\x1b[0m {} -> {}", entity_name, message);
                // Write to brain events
                if let Some(ref brain) = brain {
                    brain.track(&format!("effect:{}", entity_name), &json!({
                        "type": "log",
                        "entity": entity_name,
                        "message": message,
                        "record_id": record.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                    }));
                }
            }
        }
        "notify" => {
            // args: [provider, channel/severity, message]
            let provider = action.args.get(0).cloned().unwrap_or_default();
            let channel = action.args.get(1).cloned().unwrap_or_default();
            let msg_template = action.args.get(2).cloned().unwrap_or_default();
            let message = interpolate_effect_message(&msg_template, record);

            eprintln!("  \x1b[35m[notify]\x1b[0m {} -> {}:{} -- {}", entity_name, provider, channel, message);

            // Write to brain events for tracking
            if let Some(ref brain) = brain {
                brain.track(&format!("effect:notify:{}", entity_name), &json!({
                    "type": "notify",
                    "entity": entity_name,
                    "provider": provider,
                    "channel": channel,
                    "message": message,
                    "record_id": record.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                }));
            }

            // Broadcast via SSE so dashboards can react
            sse_hub.broadcast(sse::DataChangeEvent {
                entity: format!("_effect_notify_{}", entity_name.to_lowercase()),
                action: "notification".to_string(),
                id: record.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            });
        }
        _ => {
            // Generic/unknown action -- log it
            if let Some(ref brain) = brain {
                brain.track(&format!("effect:{}:{}", action.action_type, entity_name), &json!({
                    "type": action.action_type,
                    "entity": entity_name,
                    "args": action.args,
                    "record_id": record.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                }));
            }
        }
    }
}
