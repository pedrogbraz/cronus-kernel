// CRONUS Action Execution Engine
//
// Processes action instructions from button clicks / form submits.
// Returns an effect envelope (toast, navigate, refresh, etc.)

use serde_json::{json, Value};
use crate::database::CronusDB;
use crate::parser::{EntityNode, FieldType};

#[derive(Debug)]
pub struct ActionEffect {
    pub effect_type: String, // "toast", "navigate", "refresh", "open", "close"
    pub target: String,
    pub value: String,
    pub style: String,
}

/// Validate a single field value against its schema type.
/// Returns None if valid, Some(error_message) if invalid.
pub fn validate_field_value(field_name: &str, value: &str, entity: &EntityNode) -> Option<String> {
    let field = match entity.fields.iter().find(|f| f.name.eq_ignore_ascii_case(field_name)) {
        Some(f) => f,
        None => return Some(format!("Field '{}' does not exist on entity '{}'", field_name, entity.name)),
    };

    // Empty values are allowed unless required (required check is separate)
    if value.is_empty() {
        return None;
    }

    match field.field_type {
        FieldType::Number | FieldType::Money | FieldType::Percentage => {
            if value.parse::<f64>().is_err() {
                return Some(format!("'{}' must be a number", field_name));
            }
        }
        FieldType::Boolean => {
            let lower = value.to_lowercase();
            if !["true", "false", "0", "1"].contains(&lower.as_str()) {
                return Some(format!("'{}' must be true/false", field_name));
            }
        }
        FieldType::Email => {
            if !value.contains('@') || !value.contains('.') || value.len() < 5 {
                return Some(format!("'{}' must be a valid email", field_name));
            }
        }
        FieldType::Enum => {
            if let Some(ref vals) = field.enum_values {
                if !vals.iter().any(|v| v.eq_ignore_ascii_case(value)) {
                    return Some(format!("'{}' must be one of: {}", field_name, vals.join(", ")));
                }
            }
        }
        FieldType::Url => {
            if !value.starts_with("http://") && !value.starts_with("https://") {
                return Some(format!("'{}' must be a valid URL", field_name));
            }
        }
        _ => {} // String, Text, Date, Slug, Phone, Ulid, Json, Ip, Relation — no strict validation
    }

    None
}

/// Validate form data for INSERT against entity schema.
/// Returns a map of field_name -> error_message for any validation failures.
pub fn validate_form_data(data: &Value, entity: &EntityNode) -> std::collections::HashMap<String, String> {
    let mut errors = std::collections::HashMap::new();
    let obj = match data.as_object() {
        Some(o) => o,
        None => return errors,
    };

    // Check required fields are present and non-empty
    for field in &entity.fields {
        if field.required {
            let val = obj.get(&field.name)
                .or_else(|| obj.get(&field.name.to_lowercase()))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if val.trim().is_empty() {
                errors.insert(field.name.clone(), format!("'{}' is required", field.name));
            }
        }
    }

    // Validate types for all provided fields
    for (key, val) in obj {
        let val_string = val.to_string();
        let str_val = val.as_str().unwrap_or(&val_string);
        if let Some(err) = validate_field_value(key, str_val, entity) {
            errors.insert(key.clone(), err);
        }
    }

    errors
}

/// Execute a serialized ActionBlock (JSON) against the database.
/// Returns (success, effects).
/// `entities` is optional — when provided, field validation is enforced.
pub fn execute_action(
    instructions_json: &str,
    entity_name: &str,
    record_id: &str,
    db: &CronusDB,
) -> (bool, Vec<ActionEffect>) {
    execute_action_validated(instructions_json, entity_name, record_id, db, &[])
}

/// Execute with entity schema validation.
pub fn execute_action_validated(
    instructions_json: &str,
    entity_name: &str,
    record_id: &str,
    db: &CronusDB,
    entities: &[EntityNode],
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
                    // Validate field against entity schema if available
                    if let Some(entity) = entities.iter().find(|e| e.name.eq_ignore_ascii_case(entity_name)) {
                        if let Some(err) = validate_field_value(target, value, entity) {
                            effects.push(ActionEffect {
                                effect_type: "toast".into(),
                                target: err,
                                value: String::new(),
                                style: "error".into(),
                            });
                            return (false, effects);
                        }
                    }
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
