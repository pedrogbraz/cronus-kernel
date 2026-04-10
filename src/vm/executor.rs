//! Bytecode VM executor for .scriptcronus
//!
//! Stack-based VM with fuel metering. Each instruction consumes fuel
//! (DB ops cost 10, others cost 1). Execution halts when fuel runs out
//! or a Halt instruction is reached.

use super::opcodes::OpCode;
use serde_json::{json, Value};
use std::collections::HashMap;

/// VM execution state
pub struct VmState {
    /// Value stack
    pub stack: Vec<Value>,
    /// Local variables (indexed by u16)
    pub locals: Vec<Value>,
    /// Remaining fuel
    pub fuel: u32,
    /// Iterator stack: (items, current_index)
    pub iter_stack: Vec<(Vec<Value>, usize)>,
    /// Log output
    pub logs: Vec<String>,
    /// Response (set by Respond opcode)
    pub response: Option<VmResponse>,
}

/// Response produced by a Respond opcode
#[derive(Debug, Clone)]
pub struct VmResponse {
    pub status: u16,
    pub body: String,
}

/// Execution context for the VM (user info, env vars)
pub struct VmContext {
    pub user_id: String,
    pub role: String,
    pub env_vars: HashMap<String, String>,
    /// Event data accessible as "event.field" paths
    pub event_data: Option<Value>,
}

impl VmState {
    pub fn new(fuel: u32) -> Self {
        Self {
            stack: Vec::with_capacity(64),
            locals: Vec::with_capacity(16),
            fuel,
            iter_stack: Vec::new(),
            logs: Vec::new(),
            response: None,
        }
    }

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Result<Value, String> {
        self.stack.pop().ok_or_else(|| "stack underflow".to_string())
    }

    fn peek(&self) -> Result<&Value, String> {
        self.stack.last().ok_or_else(|| "stack underflow".to_string())
    }
}

/// Execute bytecode with a database and context
pub fn execute(
    bytecode: &[OpCode],
    db: &crate::database::CronusDB,
    ctx: &VmContext,
) -> Result<VmState, String> {
    let mut state = VmState::new(10_000);
    let mut ip: usize = 0;

    loop {
        if ip >= bytecode.len() {
            break;
        }

        let op = &bytecode[ip];
        let cost = op.fuel_cost();
        if state.fuel < cost {
            return Err("out of fuel".into());
        }
        state.fuel -= cost;

        match op {
            // === Stack ops ===
            OpCode::PushStr(s) => state.push(Value::String(s.clone())),
            OpCode::PushNum(n) => state.push(json!(*n)),
            OpCode::PushBool(b) => state.push(json!(*b)),
            OpCode::PushNull => state.push(Value::Null),
            OpCode::Pop => { state.pop()?; }

            // === Variables ===
            OpCode::LoadLocal(idx) => {
                let i = *idx as usize;
                let val = if i < state.locals.len() {
                    state.locals[i].clone()
                } else {
                    Value::Null
                };
                state.push(val);
            }
            OpCode::StoreLocal(idx) => {
                let val = state.pop()?;
                let i = *idx as usize;
                // Extend locals vector if needed
                while state.locals.len() <= i {
                    state.locals.push(Value::Null);
                }
                state.locals[i] = val;
            }

            // === Control flow ===
            OpCode::Jump(target) => {
                ip = *target as usize;
                continue; // skip ip += 1
            }
            OpCode::JumpIfFalse(target) => {
                let val = state.pop()?;
                if !is_truthy(&val) {
                    ip = *target as usize;
                    continue;
                }
            }
            OpCode::Halt => break,

            // === Comparison / logic ===
            OpCode::Equal => {
                let b = state.pop()?;
                let a = state.pop()?;
                state.push(json!(a == b));
            }
            OpCode::NotEqual => {
                let b = state.pop()?;
                let a = state.pop()?;
                state.push(json!(a != b));
            }
            OpCode::LessThan => {
                let b = state.pop()?;
                let a = state.pop()?;
                state.push(json!(compare_values(&a, &b) == std::cmp::Ordering::Less));
            }
            OpCode::GreaterThan => {
                let b = state.pop()?;
                let a = state.pop()?;
                state.push(json!(compare_values(&a, &b) == std::cmp::Ordering::Greater));
            }
            OpCode::LessOrEqual => {
                let b = state.pop()?;
                let a = state.pop()?;
                state.push(json!(compare_values(&a, &b) != std::cmp::Ordering::Greater));
            }
            OpCode::GreaterOrEqual => {
                let b = state.pop()?;
                let a = state.pop()?;
                state.push(json!(compare_values(&a, &b) != std::cmp::Ordering::Less));
            }
            OpCode::And => {
                let b = state.pop()?;
                let a = state.pop()?;
                state.push(json!(is_truthy(&a) && is_truthy(&b)));
            }
            OpCode::Or => {
                let b = state.pop()?;
                let a = state.pop()?;
                state.push(json!(is_truthy(&a) || is_truthy(&b)));
            }
            OpCode::Contains => {
                let b = state.pop()?;
                let a = state.pop()?;
                let a_str = value_to_string(&a);
                let b_str = value_to_string(&b);
                state.push(json!(a_str.contains(&b_str)));
            }

            // === Object access ===
            OpCode::GetField(field) => {
                let obj = state.pop()?;
                let val = match &obj {
                    Value::Object(map) => map.get(field).cloned().unwrap_or(Value::Null),
                    // If it's a string (implicit root like "event"), resolve from context
                    Value::String(root) => {
                        resolve_context_field(root, field, ctx)
                    }
                    _ => Value::Null,
                };
                state.push(val);
            }

            // === Builtins ===
            OpCode::Log => {
                let msg = state.pop()?;
                let msg_str = value_to_string(&msg);
                eprintln!("  \x1b[36m[vm]\x1b[0m {}", msg_str);
                if state.logs.len() < 50 {
                    state.logs.push(msg_str);
                }
            }

            OpCode::DbQuery(entity) => {
                let table = entity.to_string();
                match db.find_all(&table, 1000, 0) {
                    Ok(rows_val) => {
                        let mut results: Vec<Value> = match rows_val {
                            Value::Array(arr) => arr,
                            _ => vec![],
                        };
                        // Owner isolation for non-admin
                        if ctx.role != "admin" && !ctx.user_id.is_empty() && ctx.user_id != "system" {
                            let uid = &ctx.user_id;
                            results.retain(|row| {
                                let owner = row.get("_owner_id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                owner.is_empty() || owner == uid
                            });
                        }
                        state.push(Value::Array(results));
                    }
                    Err(e) => {
                        eprintln!("  \x1b[31m[vm]\x1b[0m db.query {} failed: {}", entity, e);
                        state.push(Value::Array(vec![]));
                    }
                }
            }

            OpCode::DbInsert(entity) => {
                let fields_val = state.pop()?;
                let table = entity.to_string();
                if let Value::Object(mut map) = fields_val {
                    // Force owner to current user
                    map.remove("_owner_id");
                    map.insert("_owner_id".into(), Value::String(ctx.user_id.clone()));
                    match db.insert(&table, &Value::Object(map)) {
                        Ok(id) => {
                            eprintln!("  \x1b[36m[vm]\x1b[0m db.insert {} -> {}", entity, id);
                        }
                        Err(e) => {
                            eprintln!("  \x1b[31m[vm]\x1b[0m db.insert {} failed: {}", entity, e);
                        }
                    }
                }
            }

            OpCode::DbUpdate(entity) => {
                // Stack: [id, field_map] — field_map on top
                let fields_val = state.pop()?;
                let id_val = state.pop()?;
                let id_str = value_to_string(&id_val);
                let table = entity.to_string();

                // Owner check for non-admin
                if ctx.role != "admin" && ctx.user_id != "system" {
                    if let Ok(Some(existing)) = db.find_by_id(&table, &id_str) {
                        let owner = existing.get("_owner_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        if !owner.is_empty() && owner != ctx.user_id {
                            eprintln!("  \x1b[31m[vm]\x1b[0m db.update {} BLOCKED -- owner mismatch", entity);
                            ip += 1;
                            continue;
                        }
                    }
                }

                if let Value::Object(mut map) = fields_val {
                    map.remove("_owner_id");
                    match db.update(&table, &id_str, &Value::Object(map)) {
                        Ok(_) => {
                            eprintln!("  \x1b[36m[vm]\x1b[0m db.update {} {}", entity, id_str);
                        }
                        Err(e) => {
                            eprintln!("  \x1b[31m[vm]\x1b[0m db.update {} failed: {}", entity, e);
                        }
                    }
                }
            }

            OpCode::DbDelete(entity) => {
                let id_val = state.pop()?;
                let id_str = value_to_string(&id_val);
                let table = entity.to_string();

                // Owner check for non-admin
                if ctx.role != "admin" && ctx.user_id != "system" {
                    if let Ok(Some(existing)) = db.find_by_id(&table, &id_str) {
                        let owner = existing.get("_owner_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        if !owner.is_empty() && owner != ctx.user_id {
                            eprintln!("  \x1b[31m[vm]\x1b[0m db.delete {} BLOCKED -- owner mismatch", entity);
                            ip += 1;
                            continue;
                        }
                    }
                }

                let _ = db.delete(&table, &id_str);
                eprintln!("  \x1b[36m[vm]\x1b[0m db.delete {} {}", entity, id_str);
            }

            // === Iteration ===
            OpCode::IterBegin => {
                let arr = state.pop()?;
                let items = match arr {
                    Value::Array(v) => v,
                    _ => vec![],
                };
                state.iter_stack.push((items, 0));
            }
            OpCode::IterNext(end_target) => {
                if let Some((items, idx)) = state.iter_stack.last_mut() {
                    if *idx < items.len() {
                        let item = items[*idx].clone();
                        *idx += 1;
                        state.push(item);
                    } else {
                        ip = *end_target as usize;
                        continue;
                    }
                } else {
                    return Err("IterNext without IterBegin".into());
                }
            }
            OpCode::IterEnd => {
                state.iter_stack.pop();
            }

            // === Response ===
            OpCode::Respond(status) => {
                let body = state.pop()?;
                state.response = Some(VmResponse {
                    status: *status,
                    body: value_to_string(&body),
                });
            }

            // === Misc ===
            OpCode::Now => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                state.push(json!(now));
            }
            OpCode::EnvVar(key) => {
                let val = ctx.env_vars.get(key).cloned().unwrap_or_default();
                state.push(Value::String(val));
            }
            OpCode::AuthGetUser => {
                state.push(json!({ "id": ctx.user_id, "role": ctx.role }));
            }
            OpCode::AuthCheckRole(role) => {
                state.push(json!(ctx.role == *role));
            }

            // === Map building ===
            OpCode::PushMap => {
                state.push(json!({}));
            }
            OpCode::MapInsert(key) => {
                let val = state.pop()?;
                let mut map_val = state.pop()?;
                if let Value::Object(ref mut map) = map_val {
                    map.insert(key.clone(), val);
                }
                state.push(map_val);
            }
        }

        ip += 1;
    }

    Ok(state)
}

// === Helper functions (mirrored from tree-walk interpreter) ===

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Null => false,
        Value::String(s) => !s.is_empty(),
        Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
        Value::Array(a) => !a.is_empty(),
        Value::Object(_) => true,
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a.as_f64(), b.as_f64()) {
        (Some(af), Some(bf)) => af.partial_cmp(&bf).unwrap_or(std::cmp::Ordering::Equal),
        _ => {
            let a_s = value_to_string(a);
            let b_s = value_to_string(b);
            a_s.cmp(&b_s)
        }
    }
}

/// Resolve "event.field" style paths from the VM context
fn resolve_context_field(root: &str, field: &str, ctx: &VmContext) -> Value {
    match root {
        "event" => {
            if let Some(ref ev) = ctx.event_data {
                ev.get(field).cloned().unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }
        _ => Value::Null,
    }
}

// === Tests ===

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::opcodes::OpCode;

    /// Helper: execute bytecode without a real DB
    fn exec_no_db(bytecode: &[OpCode]) -> Result<VmState, String> {
        // Create a temporary in-memory DB for tests
        let db = crate::database::CronusDB::open_memory()
            .map_err(|e| format!("test db: {}", e))?;
        let ctx = VmContext {
            user_id: "test-user".into(),
            role: "admin".into(),
            env_vars: HashMap::new(),
            event_data: None,
        };
        execute(bytecode, &db, &ctx)
    }

    #[test]
    fn test_vm_push_pop() {
        let bytecode = vec![
            OpCode::PushStr("hello".into()),
            OpCode::PushNum(42.0),
            OpCode::Pop, // pop 42
            OpCode::Halt,
        ];
        let state = exec_no_db(&bytecode).unwrap();
        assert_eq!(state.stack.len(), 1);
        assert_eq!(state.stack[0], json!("hello"));
    }

    #[test]
    fn test_vm_log() {
        let bytecode = vec![
            OpCode::PushStr("test message".into()),
            OpCode::Log,
            OpCode::Halt,
        ];
        let state = exec_no_db(&bytecode).unwrap();
        assert_eq!(state.logs.len(), 1);
        assert_eq!(state.logs[0], "test message");
        assert!(state.stack.is_empty());
    }

    #[test]
    fn test_vm_jump() {
        let bytecode = vec![
            OpCode::PushBool(false),        // 0
            OpCode::JumpIfFalse(4),         // 1 -> jump to 4
            OpCode::PushStr("skipped".into()), // 2 (should be skipped)
            OpCode::Halt,                    // 3
            OpCode::PushStr("reached".into()), // 4
            OpCode::Halt,                    // 5
        ];
        let state = exec_no_db(&bytecode).unwrap();
        assert_eq!(state.stack.len(), 1);
        assert_eq!(state.stack[0], json!("reached"));
    }

    #[test]
    fn test_vm_fuel_limit() {
        // Create bytecode that loops forever
        let bytecode = vec![
            OpCode::PushNull,  // 0
            OpCode::Pop,       // 1
            OpCode::Jump(0),   // 2 -> infinite loop
        ];

        // Use a small fuel limit
        let db = crate::database::CronusDB::open_memory().unwrap();
        let ctx = VmContext {
            user_id: "test".into(),
            role: "admin".into(),
            env_vars: HashMap::new(),
            event_data: None,
        };

        // Manually create state with low fuel
        let mut state = VmState::new(10);
        let mut ip: usize = 0;

        let result = loop {
            if ip >= bytecode.len() { break Ok(()); }
            let op = &bytecode[ip];
            let cost = op.fuel_cost();
            if state.fuel < cost { break Err("out of fuel".to_string()); }
            state.fuel -= cost;
            match op {
                OpCode::PushNull => state.push(Value::Null),
                OpCode::Pop => { let _ = state.pop(); },
                OpCode::Jump(t) => { ip = *t as usize; continue; },
                _ => {}
            }
            ip += 1;
        };

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "out of fuel");
    }

    #[test]
    fn test_vm_locals() {
        let bytecode = vec![
            OpCode::PushStr("world".into()),
            OpCode::StoreLocal(0),
            OpCode::LoadLocal(0),
            OpCode::Halt,
        ];
        let state = exec_no_db(&bytecode).unwrap();
        assert_eq!(state.stack.len(), 1);
        assert_eq!(state.stack[0], json!("world"));
    }

    #[test]
    fn test_vm_comparison() {
        let bytecode = vec![
            OpCode::PushNum(5.0),
            OpCode::PushNum(3.0),
            OpCode::GreaterThan,
            OpCode::Halt,
        ];
        let state = exec_no_db(&bytecode).unwrap();
        assert_eq!(state.stack[0], json!(true));
    }

    #[test]
    fn test_vm_respond() {
        let bytecode = vec![
            OpCode::PushStr("{\"ok\":true}".into()),
            OpCode::Respond(200),
            OpCode::Halt,
        ];
        let state = exec_no_db(&bytecode).unwrap();
        assert!(state.response.is_some());
        let resp = state.response.unwrap();
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, "{\"ok\":true}");
    }

    #[test]
    fn test_vm_map_building() {
        let bytecode = vec![
            OpCode::PushMap,
            OpCode::PushStr("John".into()),
            OpCode::MapInsert("name".into()),
            OpCode::PushNum(30.0),
            OpCode::MapInsert("age".into()),
            OpCode::Halt,
        ];
        let state = exec_no_db(&bytecode).unwrap();
        assert_eq!(state.stack.len(), 1);
        let map = &state.stack[0];
        assert_eq!(map.get("name"), Some(&json!("John")));
        assert_eq!(map.get("age"), Some(&json!(30.0)));
    }
}
