//! Entity side effects after writes: transition validation, outbound
//! webhooks and `on create/update/delete` effect blocks.

use crate::parser::{self, EntityNode};
use crate::server::state::AppState;
use crate::{authz, brain, sse, webhook};
use serde_json::{json, Value};
use std::sync::Arc;

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
        let allowed: Vec<&str> = transition
            .rules
            .iter()
            .filter(|r| r.from == old_value)
            .flat_map(|r| r.to.iter().map(|s| s.as_str()))
            .collect();

        // If no rules found for the current state, check if it's a wildcard "*" rule
        let allowed = if allowed.is_empty() {
            transition
                .rules
                .iter()
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

/// Shared REST/GraphQL post-write pipeline: webhooks, entity effects,
/// `.scriptcronus` handlers, hash-chained audit, SSE broadcast.
pub(crate) fn after_write(
    state: &AppState,
    entity: &EntityNode,
    event: &str,
    row: &Value,
    prev: Option<&Value>,
    row_id: &str,
    owner: &str,
) {
    let table = entity.name.as_str();
    match event {
        "update" => {
            fire_webhooks(&state.webhooks, &state.entities, table, "update", row);
            fire_effects(entity, "update", row, prev, &state.brain, &state.sse_hub);
            crate::scripting::fire_scripts(
                &state.script_registry,
                table,
                "update",
                row,
                row_id,
                prev,
                &state.db,
                owner,
                "user",
                &std::collections::HashMap::new(),
            );
            let (mut logged_row, mut logged_prev) =
                (row.clone(), prev.cloned().unwrap_or(Value::Null));
            authz::redact_sensitive(entity, &mut logged_row);
            authz::redact_sensitive(entity, &mut logged_prev);
            if let Err(e) = state.audit_trail.log(
                "UPDATE",
                table,
                row_id,
                owner,
                &logged_row,
                Some(&logged_prev),
            ) {
                eprintln!(
                    "  \x1b[33m⚠\x1b[0m Audit log failed (UPDATE {}:{}): {}",
                    table, row_id, e
                );
            }
            state.sse_hub.broadcast(sse::DataChangeEvent {
                entity: table.to_string(),
                action: "updated".to_string(),
                id: row_id.to_string(),
            });
        }
        "delete" => {
            let payload = json!({"id": row_id, "entity": table});
            fire_webhooks(&state.webhooks, &state.entities, table, "delete", &payload);
            fire_effects(entity, "delete", row, None, &state.brain, &state.sse_hub);
            crate::scripting::fire_scripts(
                &state.script_registry,
                table,
                "delete",
                row,
                row_id,
                None,
                &state.db,
                owner,
                "user",
                &std::collections::HashMap::new(),
            );
            let mut logged_prev = row.clone();
            authz::redact_sensitive(entity, &mut logged_prev);
            if let Err(e) = state.audit_trail.log(
                "DELETE",
                table,
                row_id,
                owner,
                &json!({"id": row_id}),
                Some(&logged_prev),
            ) {
                eprintln!(
                    "  \x1b[33m⚠\x1b[0m Audit log failed (DELETE {}:{}): {}",
                    table, row_id, e
                );
            }
            state.sse_hub.broadcast(sse::DataChangeEvent {
                entity: table.to_string(),
                action: "deleted".to_string(),
                id: row_id.to_string(),
            });
        }
        _ => {
            fire_webhooks(&state.webhooks, &state.entities, table, "create", row);
            fire_effects(entity, "create", row, None, &state.brain, &state.sse_hub);
            crate::scripting::fire_scripts(
                &state.script_registry,
                table,
                "create",
                row,
                row_id,
                None,
                &state.db,
                owner,
                "user",
                &std::collections::HashMap::new(),
            );
            let mut logged_row = row.clone();
            authz::redact_sensitive(entity, &mut logged_row);
            if let Err(e) = state
                .audit_trail
                .log("INSERT", table, row_id, owner, &logged_row, None)
            {
                eprintln!(
                    "  \x1b[33m⚠\x1b[0m Audit log failed (INSERT {}:{}): {}",
                    table, row_id, e
                );
            }
            state.sse_hub.broadcast(sse::DataChangeEvent {
                entity: table.to_string(),
                action: "created".to_string(),
                id: row_id.to_string(),
            });
        }
    }
}

/// Fire webhooks in background for a given entity + event.
/// Sends the payload as JSON body to each matching webhook URL.
/// Fire-and-forget webhooks for an entity event. Validation, SSRF blocking,
/// redaction, signing and timeouts live in `webhook.rs`.
pub(crate) fn fire_webhooks(
    webhooks: &[parser::WebhookNode],
    entities: &[parser::EntityNode],
    entity: &str,
    event: &str,
    payload: &serde_json::Value,
) {
    for wh in webhooks
        .iter()
        .filter(|wh| webhook::names_match(&wh.entity, entity))
    {
        let hooks: Vec<_> = wh.hooks.iter().filter(|h| h.event == event).collect();
        if hooks.is_empty() {
            continue;
        }
        let body = webhook::redacted_body(entities, entity, payload);
        for hook in hooks {
            tokio::spawn(webhook::fire(hook.clone(), event.to_string(), body.clone()));
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

        // For "on update <field>" — check if the specific field changed
        if event == "update" {
            if let Some(ref watched_field) = effect.field {
                let new_val = record
                    .get(watched_field)
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
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
                    execute_effect_action(action, &entity.name, record, brain, sse_hub);
                }
                continue;
            }
        }

        // For "on create" / "on delete" / "on update" (no specific field) — run all actions
        for action in &effect.actions {
            execute_effect_action(action, &entity.name, record, brain, sse_hub);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{parse, AstNode};
    use serde_json::json;
    use std::sync::Arc;

    fn order_entity() -> parser::EntityNode {
        let src = "entity Order {\n  number string!\n  on create {\n    log \"Order {{number}} created\"\n    notify slack \"#ops\" \"n={{number}}\"\n  }\n}\n";
        parse(src)
            .expect("parse")
            .into_iter()
            .find_map(|n| match n {
                AstNode::Entity(e) => Some(e),
                _ => None,
            })
            .expect("entity")
    }

    #[test]
    fn fire_effects_interpolates_log_and_broadcasts_notify() {
        let entity = order_entity();
        let hub = Arc::new(sse::SseHub::new());
        let mut rx = hub.subscribe_raw();
        fire_effects(
            &entity,
            "create",
            &json!({"id": "abc", "number": "42"}),
            None,
            &None,
            &hub,
        );
        let ev = rx.try_recv().expect("notify sse");
        assert!(ev.entity.contains("order"), "{}", ev.entity);
        assert_eq!(ev.action, "notification");
        assert_eq!(ev.id, "abc");
    }

    #[test]
    fn after_write_audits_and_broadcasts_create() {
        let state = crate::api_security_tests::state_from(
            r#"app "E" { port 5175 }
entity Note { title string! secret string sensitive }
"#,
        );
        let entity = state.entities.iter().find(|e| e.name == "Note").unwrap();
        let mut rx = state.sse_hub.subscribe_raw();
        let row = json!({"id": "n1", "title": "hello", "secret": "s3cr3t"});
        after_write(&state, entity, "create", &row, None, "n1", "alice");
        let ev = rx.try_recv().expect("sse");
        assert_eq!(ev.action, "created");
        assert_eq!(ev.entity, "Note");
        assert_eq!(ev.id, "n1");
        let trail = state.audit_trail.query(10).unwrap().to_string();
        assert!(trail.contains("INSERT"), "{trail}");
        assert!(!trail.contains("s3cr3t"), "sensitive leaked: {trail}");
    }
}

/// Interpolate `{{field_name}}` placeholders in a message with actual record values.
fn interpolate_effect_message(template: &str, record: &serde_json::Value) -> String {
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
            result = format!(
                "{}{}{}",
                &result[..start],
                value,
                &result[start + end + 2..]
            );
        } else {
            break;
        }
    }
    result
}

/// Execute a single effect action (log or notify).
fn execute_effect_action(
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
                eprintln!("  \x1b[36m[effect]\x1b[0m {} → {}", entity_name, message);
                // Write to brain events
                if let Some(ref brain) = brain {
                    brain.track(
                        &format!("effect:{}", entity_name),
                        &json!({
                            "type": "log",
                            "entity": entity_name,
                            "message": message,
                            "record_id": record.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                        }),
                    );
                }
            }
        }
        "notify" => {
            // args: [provider, channel/severity, message]
            let provider = action.args.get(0).cloned().unwrap_or_default();
            let channel = action.args.get(1).cloned().unwrap_or_default();
            let msg_template = action.args.get(2).cloned().unwrap_or_default();
            let message = interpolate_effect_message(&msg_template, record);

            eprintln!(
                "  \x1b[35m[notify]\x1b[0m {} → {}:{} — {}",
                entity_name, provider, channel, message
            );

            // Write to brain events for tracking
            if let Some(ref brain) = brain {
                brain.track(
                    &format!("effect:notify:{}", entity_name),
                    &json!({
                        "type": "notify",
                        "entity": entity_name,
                        "provider": provider,
                        "channel": channel,
                        "message": message,
                        "record_id": record.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                    }),
                );
            }

            // Broadcast via SSE so dashboards can react
            sse_hub.broadcast(sse::DataChangeEvent {
                entity: format!("_effect_notify_{}", entity_name.to_lowercase()),
                action: "notification".to_string(),
                id: record
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            });
        }
        _ => {
            // Generic/unknown action — log it
            if let Some(ref brain) = brain {
                brain.track(
                    &format!("effect:{}:{}", action.action_type, entity_name),
                    &json!({
                        "type": action.action_type,
                        "entity": entity_name,
                        "args": action.args,
                        "record_id": record.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                    }),
                );
            }
        }
    }
}
