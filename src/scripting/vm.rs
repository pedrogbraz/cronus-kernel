//! Tree-walking interpreter for .scriptcronus
//!
//! Executes ScriptFile AST with sandboxed context per user.
//! Max 1000 statements per execution to prevent infinite loops.

use super::ast::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

const MAX_STATEMENTS: usize = 1000;
const MAX_SCOPE_VARS: usize = 100;
const MAX_LOG_ENTRIES: usize = 50;

/// Per-execution context — fresh scope, user isolation
pub struct ScriptContext {
    pub user_id: String,
    pub role: String,
    pub scope: HashMap<String, Value>,
    pub env_vars: HashMap<String, String>,
    pub statement_count: usize,
    /// Output for endpoint `respond` statements
    pub response: Option<ScriptResponse>,
    /// Log output
    pub logs: Vec<String>,
}

pub struct ScriptResponse {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

/// Event data passed to `on Entity.event` blocks
#[derive(Debug, Clone)]
pub struct EventData {
    pub entity: String,
    pub event: String, // "create", "update", "delete"
    pub record: Value,
    pub record_id: String,
    pub prev_record: Option<Value>,
}

impl ScriptContext {
    pub fn new(user_id: &str, role: &str, env_vars: HashMap<String, String>) -> Self {
        Self {
            user_id: user_id.to_string(),
            role: role.to_string(),
            scope: HashMap::new(),
            env_vars,
            statement_count: 0,
            response: None,
            logs: Vec::new(),
        }
    }

    fn check_limit(&mut self) -> Result<(), String> {
        self.statement_count += 1;
        if self.statement_count > MAX_STATEMENTS {
            Err("script exceeded max statement limit (1000)".into())
        } else {
            Ok(())
        }
    }
}

/// Execute a list of statements in a context
pub fn execute_statements(
    stmts: &[Statement],
    ctx: &mut ScriptContext,
    db: &crate::database::CronusDB,
    event: Option<&EventData>,
) -> Result<(), String> {
    for stmt in stmts {
        ctx.check_limit()?;
        execute_statement(stmt, ctx, db, event)?;
    }
    Ok(())
}

fn execute_statement(
    stmt: &Statement,
    ctx: &mut ScriptContext,
    db: &crate::database::CronusDB,
    event: Option<&EventData>,
) -> Result<(), String> {
    match stmt {
        Statement::Let { name, value } => {
            // SECURITY: limit scope size to prevent memory exhaustion
            if ctx.scope.len() >= MAX_SCOPE_VARS && !ctx.scope.contains_key(name) {
                return Err(format!("scope limit exceeded ({} vars max)", MAX_SCOPE_VARS));
            }
            let val = eval_expr(value, ctx, db, event)?;
            ctx.scope.insert(name.clone(), val);
        }
        Statement::Log { message } => {
            let val = eval_expr(message, ctx, db, event)?;
            let msg = value_to_string(&val);
            let interpolated = interpolate_template(&msg, ctx, event);
            // SECURITY: truncate log messages to prevent log flooding
            let safe_msg = if interpolated.len() > 2048 {
                format!("{}...[truncated]", &interpolated[..2048])
            } else {
                interpolated
            };
            eprintln!("  \x1b[36m[script]\x1b[0m {}", safe_msg);
            // SECURITY: cap log entries to prevent memory exhaustion
            if ctx.logs.len() < MAX_LOG_ENTRIES {
                ctx.logs.push(safe_msg);
            }
        }
        Statement::DbCreate { entity, fields } => {
            let mut record = serde_json::Map::new();
            for (k, v) in fields {
                let val = eval_expr(v, ctx, db, event)?;
                record.insert(k.clone(), val);
            }
            // SECURITY: force owner to current user — prevent spoofing
            record.remove("_owner_id");
            record.insert("_owner_id".into(), Value::String(ctx.user_id.clone()));
            let table = entity.to_string();
            match db.insert(&table, &Value::Object(record)) {
                Ok(id) => {
                    eprintln!("  \x1b[36m[script]\x1b[0m db.create {} → {}", entity, id);
                }
                Err(e) => {
                    eprintln!("  \x1b[31m[script]\x1b[0m db.create {} failed: {}", entity, e);
                }
            }
        }
        Statement::DbUpdate { entity, id, fields } => {
            let id_val = eval_expr(id, ctx, db, event)?;
            let id_str = value_to_string(&id_val);
            let table = entity.to_string();
            // SECURITY: verify record belongs to current user before update
            if !ctx.role.eq("admin") && !ctx.user_id.eq("system") {
                if let Ok(Some(existing)) = db.find_by_id(&table, &id_str) {
                    let owner = existing.get("_owner_id").and_then(|v| v.as_str()).unwrap_or("");
                    if !owner.is_empty() && owner != ctx.user_id {
                        eprintln!("  \x1b[31m[script]\x1b[0m db.update {} BLOCKED — owner mismatch", entity);
                        return Ok(()); // silent deny
                    }
                }
            }
            let mut updates = serde_json::Map::new();
            for (k, v) in fields {
                let val = eval_expr(v, ctx, db, event)?;
                updates.insert(k.clone(), val);
            }
            // SECURITY: prevent overwriting _owner_id
            updates.remove("_owner_id");
            match db.update(&table, &id_str, &Value::Object(updates)) {
                Ok(_) => {
                    eprintln!("  \x1b[36m[script]\x1b[0m db.update {} {}", entity, id_str);
                }
                Err(e) => {
                    eprintln!("  \x1b[31m[script]\x1b[0m db.update {} failed: {}", entity, e);
                }
            }
        }
        Statement::DbDelete { entity, id } => {
            let id_val = eval_expr(id, ctx, db, event)?;
            let id_str = value_to_string(&id_val);
            let table = entity.to_string();
            // SECURITY: verify record belongs to current user before delete
            if !ctx.role.eq("admin") && !ctx.user_id.eq("system") {
                if let Ok(Some(existing)) = db.find_by_id(&table, &id_str) {
                    let owner = existing.get("_owner_id").and_then(|v| v.as_str()).unwrap_or("");
                    if !owner.is_empty() && owner != ctx.user_id {
                        eprintln!("  \x1b[31m[script]\x1b[0m db.delete {} BLOCKED — owner mismatch", entity);
                        return Ok(());
                    }
                }
            }
            let _ = db.delete(&table, &id_str);
            eprintln!("  \x1b[36m[script]\x1b[0m db.delete {} {}", entity, id_str);
        }
        Statement::SseBroadcast { event: evt, data } => {
            let mut obj = serde_json::Map::new();
            for (k, v) in data {
                obj.insert(k.clone(), eval_expr(v, ctx, db, event)?);
            }
            eprintln!("  \x1b[36m[script]\x1b[0m sse.broadcast \"{}\" {:?}", evt, obj);
            // Actual SSE broadcast happens via the SseHub passed from the caller
        }
        Statement::For { var, iter, body } => {
            let iter_val = eval_expr(iter, ctx, db, event)?;
            if let Value::Array(items) = iter_val {
                for item in &items {
                    ctx.check_limit()?;
                    ctx.scope.insert(var.clone(), item.clone());
                    execute_statements(body, ctx, db, event)?;
                }
                ctx.scope.remove(var);
            }
        }
        Statement::If { condition, then_body, else_body } => {
            let cond_val = eval_expr(condition, ctx, db, event)?;
            let is_true = match &cond_val {
                Value::Bool(b) => *b,
                Value::Null => false,
                Value::String(s) => !s.is_empty(),
                Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
                _ => true,
            };
            if is_true {
                execute_statements(then_body, ctx, db, event)?;
            } else {
                execute_statements(else_body, ctx, db, event)?;
            }
        }
        Statement::Respond { status, body, headers } => {
            let body_val = eval_expr(body, ctx, db, event)?;
            ctx.response = Some(ScriptResponse {
                status: *status,
                body: value_to_string(&body_val),
                headers: headers.clone(),
            });
        }
        Statement::ExprStatement(expr) => {
            // Execute for side effects (e.g., http calls)
            let _ = eval_expr(expr, ctx, db, event)?;
        }
    }
    Ok(())
}

fn eval_expr(
    expr: &Expr,
    ctx: &mut ScriptContext,
    db: &crate::database::CronusDB,
    event: Option<&EventData>,
) -> Result<Value, String> {
    match expr {
        Expr::StringLit(s) => {
            let interpolated = interpolate_template(s, ctx, event);
            Ok(Value::String(interpolated))
        }
        Expr::NumberLit(n) => Ok(json!(*n)),
        Expr::BoolLit(b) => Ok(json!(*b)),
        Expr::Now => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            Ok(json!(now))
        }
        Expr::EnvVar(key) => {
            Ok(Value::String(ctx.env_vars.get(key).cloned().unwrap_or_default()))
        }
        Expr::Path(parts) => {
            resolve_path(parts, ctx, event)
        }
        Expr::DbQuery { entity, filters, order, limit } => {
            let table = entity.to_string();
            let query_limit = limit.unwrap_or(1000) as usize;
            match db.find_all(&table, query_limit, 0) {
                Ok(rows_val) => {
                    let mut results: Vec<Value> = match rows_val {
                        Value::Array(arr) => arr,
                        _ => vec![],
                    };
                    // SECURITY: owner isolation — non-admin users only see their own data
                    if !ctx.role.eq("admin") && !ctx.user_id.eq("system") && !ctx.user_id.is_empty() {
                        let uid = &ctx.user_id;
                        results.retain(|row| {
                            let owner = row.get("_owner_id").and_then(|v| v.as_str()).unwrap_or("");
                            owner.is_empty() || owner == uid
                        });
                    }
                    // Apply filters
                    for f in filters {
                        let field = &f.field;
                        results.retain(|row| {
                            let row_val = row.get(field);
                            let filter_val = match &f.value {
                                Expr::StringLit(s) => Value::String(s.clone()),
                                Expr::NumberLit(n) => json!(*n),
                                Expr::BoolLit(b) => json!(*b),
                                Expr::Now => {
                                    let now = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs();
                                    json!(now)
                                }
                                _ => Value::Null,
                            };
                            match &f.op {
                                BinOperator::Eq => row_val == Some(&filter_val),
                                BinOperator::Ne => row_val != Some(&filter_val),
                                _ => true,
                            }
                        });
                    }
                    // Limit already applied via query_limit above
                    Ok(Value::Array(results))
                }
                Err(e) => {
                    eprintln!("  \x1b[31m[script]\x1b[0m db.query {} failed: {}", entity, e);
                    Ok(Value::Array(vec![]))
                }
            }
        }
        Expr::DbCount { entity, filters: _ } => {
            let table = entity.to_string();
            match db.count(&table) {
                Ok(n) => Ok(json!(n)),
                Err(_) => Ok(json!(0)),
            }
        }
        Expr::HttpCall { method, url, headers, body, json: json_body } => {
            // For now, log the HTTP call — actual reqwest integration in Phase 4
            let url_val = eval_expr(url, ctx, db, event)?;
            let url_str = value_to_string(&url_val);
            eprintln!("  \x1b[36m[script]\x1b[0m http.{} {}", method, url_str);

            // Build a mock response for now
            let mut resp = serde_json::Map::new();
            resp.insert("status".into(), json!(200));
            resp.insert("json".into(), json!({}));
            resp.insert("body".into(), json!(""));
            Ok(Value::Object(resp))
        }
        Expr::FormatCsv { data, fields } => {
            let data_val = eval_expr(data, ctx, db, event)?;
            if let Value::Array(rows) = data_val {
                let mut csv = String::new();
                csv.push_str(&fields.join(","));
                csv.push('\n');
                for row in &rows {
                    let line: Vec<String> = fields.iter()
                        .map(|f| value_to_string(row.get(f).unwrap_or(&Value::Null)))
                        .collect();
                    csv.push_str(&line.join(","));
                    csv.push('\n');
                }
                Ok(Value::String(csv))
            } else {
                Ok(Value::String(String::new()))
            }
        }
        Expr::FormatJson { data } => {
            let val = eval_expr(data, ctx, db, event)?;
            Ok(Value::String(val.to_string()))
        }
        Expr::BinOp { left, op, right } => {
            let l = eval_expr(left, ctx, db, event)?;
            let r = eval_expr(right, ctx, db, event)?;
            let result = match op {
                BinOperator::Eq => l == r,
                BinOperator::Ne => l != r,
                BinOperator::Lt => compare_values(&l, &r) == std::cmp::Ordering::Less,
                BinOperator::Gt => compare_values(&l, &r) == std::cmp::Ordering::Greater,
                BinOperator::Lte => compare_values(&l, &r) != std::cmp::Ordering::Greater,
                BinOperator::Gte => compare_values(&l, &r) != std::cmp::Ordering::Less,
                BinOperator::And => is_truthy(&l) && is_truthy(&r),
                BinOperator::Or => is_truthy(&l) || is_truthy(&r),
                BinOperator::Contains => {
                    let ls = value_to_string(&l);
                    let rs = value_to_string(&r);
                    ls.contains(&rs)
                }
            };
            Ok(json!(result))
        }
        Expr::AuthCheckRole(role) => {
            Ok(json!(ctx.role == *role))
        }
        Expr::AuthGetUser => {
            Ok(json!({ "id": ctx.user_id, "role": ctx.role }))
        }
    }
}

fn resolve_path(parts: &[String], ctx: &ScriptContext, event: Option<&EventData>) -> Result<Value, String> {
    if parts.is_empty() {
        return Ok(Value::Null);
    }

    let root = &parts[0];
    let mut current = match root.as_str() {
        "event" => {
            if let Some(ev) = event {
                json!({
                    "id": ev.record_id,
                    "entity": ev.entity,
                    "event": ev.event,
                    "record": ev.record,
                    "body": ev.record, // alias for webhook
                    "prev": ev.prev_record,
                })
            } else {
                Value::Null
            }
        }
        _ => {
            ctx.scope.get(root).cloned().unwrap_or(Value::Null)
        }
    };

    for part in &parts[1..] {
        current = match current {
            Value::Object(ref map) => map.get(part).cloned().unwrap_or(Value::Null),
            _ => Value::Null,
        };
    }

    Ok(current)
}

fn interpolate_template(template: &str, ctx: &ScriptContext, event: Option<&EventData>) -> String {
    let mut result = template.to_string();
    // Replace {{path.to.value}} patterns
    while let Some(start) = result.find("{{") {
        if let Some(end) = result[start..].find("}}") {
            let path_str = &result[start + 2..start + end];
            let parts: Vec<String> = path_str.split('.').map(|s| s.to_string()).collect();
            let value = resolve_path(&parts, ctx, event).unwrap_or(Value::Null);
            let replacement = value_to_string(&value);
            result = format!("{}{}{}", &result[..start], replacement, &result[start + end + 2..]);
        } else {
            break;
        }
    }
    result
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

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

fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a.as_f64(), b.as_f64()) {
        (Some(af), Some(bf)) => af.partial_cmp(&bf).unwrap_or(std::cmp::Ordering::Equal),
        _ => {
            let as_str = value_to_string(a);
            let bs_str = value_to_string(b);
            as_str.cmp(&bs_str)
        }
    }
}
