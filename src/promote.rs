//! Promotion pipeline: .scriptcronus -> .cronus
//!
//! Converts ScriptFile AST into .cronus text output.
//! V1 uses direct text generation; full AST conversion deferred to v2.

use crate::scripting::ast::*;
use crate::trust;

/// Promote a ScriptFile to .cronus text output.
///
/// Generates valid .cronus syntax from script blocks, embedding
/// trust metadata and preserving original logic as comments where
/// full semantic conversion isn't yet possible.
pub fn promote_to_text(script: &ScriptFile) -> String {
    let trust_profile = trust::compute_trust(&script.name);
    let (score_str, status_str) = match &trust_profile {
        Some(tp) => (format!("{:.3}", tp.score()), format!("{:?}", tp.status())),
        None => ("0.000".to_string(), "Sandbox".to_string()),
    };

    let mut out = String::new();

    // Header
    out.push_str(&format!("# Promoted from \"{}\" (.scriptcronus)\n", script.name));
    out.push_str(&format!("# Version: {}\n", script.version));
    out.push_str(&format!("# Trust score: {} ({})\n", score_str, status_str));
    out.push_str(&format!("# Promoted at: {}\n", chrono_date()));
    out.push('\n');

    // Collect blocks by type for grouping
    let mut entity_effects: Vec<&OnEventBlock> = Vec::new();
    let mut endpoints: Vec<&EndpointBlock> = Vec::new();
    let mut schedules: Vec<&ScheduleBlock> = Vec::new();
    let mut webhooks: Vec<&OnWebhookBlock> = Vec::new();

    for block in &script.blocks {
        match block {
            ScriptBlock::OnEvent(e) => entity_effects.push(e),
            ScriptBlock::Endpoint(e) => endpoints.push(e),
            ScriptBlock::Schedule(s) => schedules.push(s),
            ScriptBlock::OnWebhook(w) => webhooks.push(w),
        }
    }

    // Group entity effects by entity name
    let mut entity_map: std::collections::HashMap<&str, Vec<&OnEventBlock>> =
        std::collections::HashMap::new();
    for effect in &entity_effects {
        entity_map.entry(&effect.entity).or_default().push(effect);
    }

    // Emit entity blocks with effects
    for (entity_name, effects) in &entity_map {
        out.push_str(&format!("entity {} {{\n", entity_name));
        for effect in effects {
            out.push_str(&format!("  on {} {{\n", effect.event));
            for stmt in &effect.body {
                emit_statement(&mut out, stmt, 4);
            }
            out.push_str("  }\n");
        }
        out.push_str("}\n\n");
    }

    // Emit api block
    if !endpoints.is_empty() {
        out.push_str("api {\n");
        for ep in &endpoints {
            let auth_str = match &ep.auth {
                Some(role) => format!(" auth:{}", role),
                None => String::new(),
            };
            out.push_str(&format!(
                "  {} {}{} {{\n",
                ep.method, ep.path, auth_str
            ));
            for stmt in &ep.body {
                emit_statement(&mut out, stmt, 4);
            }
            out.push_str("  }\n");
        }
        out.push_str("}\n\n");
    }

    // Emit schedule blocks as worker comments (workers not fully in AST)
    for sched in &schedules {
        out.push_str(&format!("# Worker: \"{}\" (every {})\n", sched.name, sched.interval));
        out.push_str("# Schedule blocks require manual conversion to worker nodes.\n");
        out.push_str(&format!("worker {} {{\n", sanitize_name(&sched.name)));
        out.push_str(&format!("  # interval: {}\n", sched.interval));
        for stmt in &sched.body {
            emit_statement(&mut out, stmt, 2);
        }
        out.push_str("}\n\n");
    }

    // Emit incoming webhook blocks
    for wh in &webhooks {
        let name = extract_webhook_name(&wh.path);
        out.push_str(&format!("# Incoming webhook: {}\n", wh.path));
        out.push_str(&format!("endpoint POST {} {{\n", wh.path));
        for stmt in &wh.body {
            emit_statement(&mut out, stmt, 2);
        }
        out.push_str("}\n\n");
    }

    out
}

/// Convert a sanitized name from a schedule name (spaces -> underscores, lowercase).
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect()
}

/// Extract a meaningful name from a webhook path like "/hooks/stripe" -> "Stripe".
fn extract_webhook_name(path: &str) -> String {
    path.split('/')
        .last()
        .unwrap_or("webhook")
        .to_string()
}

/// Emit a statement as .cronus-flavored text with proper indentation.
fn emit_statement(out: &mut String, stmt: &Statement, indent: usize) {
    let pad = " ".repeat(indent);
    match stmt {
        Statement::Let { name, value } => {
            out.push_str(&format!("{}let {} = {}\n", pad, name, emit_expr(value)));
        }
        Statement::DbCreate { entity, fields } => {
            out.push_str(&format!("{}create {} {{\n", pad, entity));
            for (k, v) in fields {
                out.push_str(&format!("{}  {} {}\n", pad, k, emit_expr(v)));
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        Statement::DbUpdate { entity, id, fields } => {
            out.push_str(&format!("{}update {} {} {{\n", pad, entity, emit_expr(id)));
            for (k, v) in fields {
                out.push_str(&format!("{}  {} {}\n", pad, k, emit_expr(v)));
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        Statement::DbDelete { entity, id } => {
            out.push_str(&format!("{}delete {} {}\n", pad, entity, emit_expr(id)));
        }
        Statement::Log { message } => {
            out.push_str(&format!("{}log {}\n", pad, emit_expr(message)));
        }
        Statement::SseBroadcast { event, data } => {
            out.push_str(&format!("{}# sse.broadcast \"{}\" {{ ", pad, event));
            for (k, v) in data {
                out.push_str(&format!("{}: {}, ", k, emit_expr(v)));
            }
            out.push_str("}\n");
        }
        Statement::For { var, iter, body } => {
            out.push_str(&format!("{}for {} in {} {{\n", pad, var, emit_expr(iter)));
            for s in body {
                emit_statement(out, s, indent + 2);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        Statement::If { condition, then_body, else_body } => {
            out.push_str(&format!("{}if {} {{\n", pad, emit_expr(condition)));
            for s in then_body {
                emit_statement(out, s, indent + 2);
            }
            if !else_body.is_empty() {
                out.push_str(&format!("{}}} else {{\n", pad));
                for s in else_body {
                    emit_statement(out, s, indent + 2);
                }
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        Statement::Respond { status, body, headers } => {
            out.push_str(&format!("{}respond {} {}\n", pad, status, emit_expr(body)));
            if !headers.is_empty() {
                for (k, v) in headers {
                    out.push_str(&format!("{}  # header: {}: {}\n", pad, k, v));
                }
            }
        }
        Statement::ExprStatement(expr) => {
            out.push_str(&format!("{}# expr: {}\n", pad, emit_expr(expr)));
        }
    }
}

/// Emit an expression as text.
fn emit_expr(expr: &Expr) -> String {
    match expr {
        Expr::StringLit(s) => format!("\"{}\"", s),
        Expr::NumberLit(n) => format!("{}", n),
        Expr::BoolLit(b) => format!("{}", b),
        Expr::Path(parts) => parts.join("."),
        Expr::DbQuery { entity, filters, order, limit } => {
            let mut s = format!("query {} {{ all", entity);
            if !filters.is_empty() {
                s.push_str(" where ");
                let f: Vec<String> = filters.iter()
                    .map(|f| format!("{} {} {}", f.field, emit_binop(&f.op), emit_expr(&f.value)))
                    .collect();
                s.push_str(&f.join(", "));
            }
            if let Some(ord) = order {
                s.push_str(&format!(" order {}", ord));
            }
            if let Some(lim) = limit {
                s.push_str(&format!(" limit {}", lim));
            }
            s.push_str(" }");
            s
        }
        Expr::DbCount { entity, filters } => {
            let mut s = format!("query {} {{ count", entity);
            if !filters.is_empty() {
                s.push_str(" where ");
                let f: Vec<String> = filters.iter()
                    .map(|f| format!("{} {} {}", f.field, emit_binop(&f.op), emit_expr(&f.value)))
                    .collect();
                s.push_str(&f.join(", "));
            }
            s.push_str(" }");
            s
        }
        Expr::HttpCall { method, url, .. } => {
            format!("http.{} {}", method, emit_expr(url))
        }
        Expr::FormatCsv { data, fields } => {
            format!("format.csv {} [{}]", emit_expr(data), fields.join(", "))
        }
        Expr::FormatJson { data } => {
            format!("format.json {}", emit_expr(data))
        }
        Expr::Now => "now()".to_string(),
        Expr::EnvVar(key) => format!("env.{}", key),
        Expr::BinOp { left, op, right } => {
            format!("{} {} {}", emit_expr(left), emit_binop(op), emit_expr(right))
        }
        Expr::AuthCheckRole(role) => format!("auth.check_role \"{}\"", role),
        Expr::AuthGetUser => "auth.get_user".to_string(),
    }
}

/// Emit a binary operator as text.
fn emit_binop(op: &BinOperator) -> &'static str {
    match op {
        BinOperator::Eq => "==",
        BinOperator::Ne => "!=",
        BinOperator::Lt => "<",
        BinOperator::Gt => ">",
        BinOperator::Lte => "<=",
        BinOperator::Gte => ">=",
        BinOperator::And => "and",
        BinOperator::Or => "or",
        BinOperator::Contains => "contains",
    }
}

/// Get current date as YYYY-MM-DD string (no external deps).
fn chrono_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple date calculation
    let days = secs / 86400;
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let months = [31, if is_leap(y) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 1;
    for &d in &months {
        if remaining < d {
            break;
        }
        remaining -= d;
        m += 1;
    }
    format!("{:04}-{:02}-{:02}", y, m, remaining + 1)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_test_script() -> ScriptFile {
        ScriptFile {
            name: "CustomerAutomation".to_string(),
            version: "1.0".to_string(),
            blocks: vec![
                ScriptBlock::OnEvent(OnEventBlock {
                    entity: "Customer".to_string(),
                    event: "create".to_string(),
                    body: vec![
                        Statement::Log {
                            message: Expr::StringLit("New customer created".to_string()),
                        },
                    ],
                }),
                ScriptBlock::Endpoint(EndpointBlock {
                    method: "GET".to_string(),
                    path: "/api/export/customers".to_string(),
                    auth: Some("admin".to_string()),
                    body: vec![
                        Statement::Let {
                            name: "data".to_string(),
                            value: Expr::DbQuery {
                                entity: "Customer".to_string(),
                                filters: vec![],
                                order: None,
                                limit: None,
                            },
                        },
                        Statement::Respond {
                            status: 200,
                            body: Expr::Path(vec!["data".to_string()]),
                            headers: HashMap::new(),
                        },
                    ],
                }),
                ScriptBlock::Schedule(ScheduleBlock {
                    name: "Daily Report".to_string(),
                    interval: "1d".to_string(),
                    body: vec![
                        Statement::Log {
                            message: Expr::StringLit("Running daily report".to_string()),
                        },
                    ],
                }),
                ScriptBlock::OnWebhook(OnWebhookBlock {
                    path: "/hooks/stripe".to_string(),
                    body: vec![
                        Statement::DbCreate {
                            entity: "Payment".to_string(),
                            fields: {
                                let mut m = HashMap::new();
                                m.insert("amount".to_string(), Expr::Path(vec!["event".to_string(), "amount".to_string()]));
                                m.insert("status".to_string(), Expr::StringLit("pending".to_string()));
                                m
                            },
                        },
                    ],
                }),
            ],
        }
    }

    #[test]
    fn test_promote_to_text_generates_valid_output() {
        let script = make_test_script();
        let output = promote_to_text(&script);

        // Header present
        assert!(output.contains("# Promoted from \"CustomerAutomation\""));
        assert!(output.contains("# Trust score:"));

        // Entity block with effect
        assert!(output.contains("entity Customer {"));
        assert!(output.contains("on create {"));
        assert!(output.contains("log \"New customer created\""));

        // API block
        assert!(output.contains("api {"));
        assert!(output.contains("GET /api/export/customers auth:admin"));
        assert!(output.contains("let data = query Customer"));
        assert!(output.contains("respond 200"));

        // Schedule -> worker
        assert!(output.contains("worker daily_report"));
        assert!(output.contains("# interval: 1d"));

        // Webhook -> endpoint
        assert!(output.contains("endpoint POST /hooks/stripe"));
        assert!(output.contains("create Payment"));
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("Daily Report"), "daily_report");
        assert_eq!(sanitize_name("sync-data"), "sync_data");
        assert_eq!(sanitize_name("test123"), "test123");
    }

    #[test]
    fn test_emit_expr_coverage() {
        assert_eq!(emit_expr(&Expr::StringLit("hello".into())), "\"hello\"");
        assert_eq!(emit_expr(&Expr::NumberLit(42.0)), "42");
        assert_eq!(emit_expr(&Expr::BoolLit(true)), "true");
        assert_eq!(emit_expr(&Expr::Now), "now()");
        assert_eq!(emit_expr(&Expr::EnvVar("SECRET".into())), "env.SECRET");
        assert_eq!(emit_expr(&Expr::AuthGetUser), "auth.get_user");
        assert_eq!(
            emit_expr(&Expr::Path(vec!["event".into(), "id".into()])),
            "event.id"
        );
    }
}
