// CRONUS Action Execution Engine
//
// Processes action instructions from button clicks / form submits.
// Returns an effect envelope (toast, navigate, refresh, etc.)

use serde_json::{json, Value};
use crate::database::CronusDB;

#[derive(Debug)]
pub struct ActionEffect {
    pub effect_type: String, // "toast", "navigate", "refresh", "open", "close"
    pub target: String,
    pub value: String,
    pub style: String,
}

/// Execute a serialized ActionBlock (JSON) against the database.
/// Returns (success, effects).
pub fn execute_action(
    instructions_json: &str,
    entity_name: &str,
    record_id: &str,
    db: &CronusDB,
) -> (bool, Vec<ActionEffect>) {
    let mut effects = Vec::new();

    let parsed: Value = match serde_json::from_str(instructions_json) {
        Ok(v) => v,
        Err(_) => return (false, vec![ActionEffect {
            effect_type: "toast".into(),
            target: "Invalid action data".into(),
            value: String::new(),
            style: "error".into(),
        }]),
    };

    let instructions = parsed
        .get("instructions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    for instr in &instructions {
        let verb = instr.get("verb").and_then(|v| v.as_str()).unwrap_or("");
        let target = instr.get("target").and_then(|v| v.as_str()).unwrap_or("");
        let value = instr.get("value").and_then(|v| v.as_str()).unwrap_or("");
        let style = instr
            .get("modifiers")
            .and_then(|m| m.get("style"))
            .and_then(|v| v.as_str())
            .unwrap_or("info");

        match verb {
            "set" => {
                // Update a single field on the current record
                if !entity_name.is_empty() && !record_id.is_empty() && !target.is_empty() {
                    let update_data = json!({ target: value });
                    let _ = db.update(entity_name, record_id, &update_data);
                }
            }
            "toast" => {
                effects.push(ActionEffect {
                    effect_type: "toast".into(),
                    target: target.into(),
                    value: String::new(),
                    style: style.into(),
                });
            }
            "navigate" => {
                effects.push(ActionEffect {
                    effect_type: "navigate".into(),
                    target: target.into(),
                    value: String::new(),
                    style: String::new(),
                });
            }
            "refresh" => {
                effects.push(ActionEffect {
                    effect_type: "refresh".into(),
                    target: String::new(),
                    value: String::new(),
                    style: String::new(),
                });
            }
            "delete" => {
                if !entity_name.is_empty() && !record_id.is_empty() {
                    let _ = db.delete(entity_name, record_id);
                }
                effects.push(ActionEffect {
                    effect_type: "toast".into(),
                    target: "Deleted".into(),
                    value: String::new(),
                    style: "success".into(),
                });
            }
            "open" => {
                effects.push(ActionEffect {
                    effect_type: "open".into(),
                    target: target.into(),
                    value: String::new(),
                    style: String::new(),
                });
            }
            "close" => {
                effects.push(ActionEffect {
                    effect_type: "close".into(),
                    target: target.into(),
                    value: String::new(),
                    style: String::new(),
                });
            }
            _ => {}
        }
    }

    (true, effects)
}

/// Convert effects list to JSON envelope for the client.
pub fn effects_to_json(effects: &[ActionEffect]) -> Value {
    let arr: Vec<Value> = effects
        .iter()
        .map(|e| {
            json!({
                "type": e.effect_type,
                "target": e.target,
                "value": e.value,
                "style": e.style,
            })
        })
        .collect();
    json!({ "ok": true, "effects": arr })
}
