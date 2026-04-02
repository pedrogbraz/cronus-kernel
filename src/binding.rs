#![allow(dead_code, unused_imports)]

use serde_json::Value;
use std::collections::HashMap;
use crate::parser::{BindingNode, QueryType, FilterOp, BindingValue, SectionNode, OrderDirection};
use crate::database::CronusDB;

/// The result of resolving a binding against the database.
#[derive(Debug, Clone)]
pub enum ResolvedData {
    /// query all -> rows
    Rows(Vec<Value>),
    /// query one -> single record or None
    Record(Option<Value>),
    /// query count -> number
    Count(u64),
    /// No binding on this section
    None,
}

/// Convert a FilterOp enum to the string name that filter_op_to_sql expects.
fn op_to_str(op: &FilterOp) -> &'static str {
    match op {
        FilterOp::Eq => "eq",
        FilterOp::Ne => "ne",
        FilterOp::Gt => "gt",
        FilterOp::Gte => "gte",
        FilterOp::Lt => "lt",
        FilterOp::Lte => "lte",
        FilterOp::Contains => "contains",
        FilterOp::StartsWith => "starts_with",
    }
}

/// Extract the string value from a BindingValue for SQL parameter.
/// Route params (route.id, route.slug, etc.) are resolved from the route_params map.
fn binding_value_to_string(val: &BindingValue, route_params: &HashMap<String, String>) -> String {
    match val {
        BindingValue::Str(s) => s.clone(),
        BindingValue::Num(n) => n.clone(),
        BindingValue::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
        BindingValue::AuthRef(r) => {
            // Resolve route.* references from URL params
            if let Some(param_name) = r.strip_prefix("route.") {
                return route_params.get(param_name).cloned().unwrap_or_default();
            }
            // TODO: resolve auth refs against current user session
            // For now, return the ref as placeholder
            format!("${}", r)
        }
    }
}

/// Prepare filter tuples for the database query builder.
/// Each tuple is (field_name, sql_operator, value_string).
fn prepare_filters(binding: &BindingNode, route_params: &HashMap<String, String>) -> Vec<(String, String, String)> {
    binding.filters.iter().map(|f| {
        let sql_op = crate::database::filter_op_to_sql(op_to_str(&f.operator)).to_string();
        let mut value = binding_value_to_string(&f.value, route_params);

        // For LIKE operators, wrap the value
        match f.operator {
            FilterOp::Contains => { value = format!("%{}%", value); }
            FilterOp::StartsWith => { value = format!("{}%", value); }
            _ => {}
        }

        (f.field.clone(), sql_op, value)
    }).collect()
}

/// Resolve a section's binding against the database.
/// Returns ResolvedData that renderers can consume.
///
/// This is the ONLY place where binding -> database query happens.
/// Renderers never touch the database directly.
/// `route_params` maps URL parameter names to their values (e.g. "id" -> "abc123").
pub fn resolve_binding(section: &SectionNode, db: &CronusDB, route_params: &HashMap<String, String>, owner_id: &str) -> ResolvedData {
    let binding = match &section.binding {
        Some(b) => b,
        None => return ResolvedData::None,
    };

    // Use entity name as-is — tables are created with the exact entity name from migrate()
    let table = binding.entity.clone();
    let mut filters = prepare_filters(binding, route_params);

    // SECURITY: Add owner_id filter for data isolation
    if !owner_id.is_empty() {
        filters.push(("_owner_id".to_string(), "=".to_string(), owner_id.to_string()));
    }

    let order_field = binding.order.as_ref().map(|o| o.field.as_str());
    let order_dir = binding.order.as_ref().map(|o| {
        match o.direction {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC",
        }
    });

    // If group_by is present, run an aggregation query instead of normal CRUD
    if binding.group_by.is_some() {
        return resolve_aggregation(binding, &table, &filters, db);
    }

    match binding.query {
        QueryType::All => {
            match db.find_many(&table, &filters, order_field, order_dir, binding.limit, binding.offset) {
                Ok(Value::Array(rows)) => ResolvedData::Rows(rows),
                Ok(other) => ResolvedData::Rows(vec![other]),
                Err(e) => {
                    eprintln!("  \x1b[31m✗\x1b[0m Binding error ({}): {}", table, e);
                    ResolvedData::Rows(Vec::new())
                }
            }
        }
        QueryType::One => {
            match db.find_one(&table, &filters, order_field, order_dir) {
                Ok(record) => ResolvedData::Record(record),
                Err(e) => {
                    eprintln!("  \x1b[31m✗\x1b[0m Binding error ({}): {}", table, e);
                    ResolvedData::Record(None)
                }
            }
        }
        QueryType::Count => {
            match db.count_where(&table, &filters) {
                Ok(n) => ResolvedData::Count(n),
                Err(e) => {
                    eprintln!("  \x1b[31m✗\x1b[0m Binding error ({}): {}", table, e);
                    ResolvedData::Count(0)
                }
            }
        }
    }
}

/// Build and execute a GROUP BY + aggregate SQL query.
/// Returns Rows with {label, value} objects for chart consumption.
///
/// SECURITY: All field names are validated as safe identifiers before use in SQL.
/// Filter values use parameterized queries (no string interpolation).
fn resolve_aggregation(
    binding: &BindingNode,
    table: &str,
    filters: &[(String, String, String)],
    db: &CronusDB,
) -> ResolvedData {
    let group = binding.group_by.as_ref().unwrap();

    // SECURITY: Validate all field names are safe identifiers
    if !crate::security::is_safe_identifier(&group.field) {
        eprintln!("  \x1b[31m✗\x1b[0m SECURITY: invalid group field name: {}", group.field);
        return ResolvedData::Rows(Vec::new());
    }
    if !crate::security::is_safe_identifier(table) {
        eprintln!("  \x1b[31m✗\x1b[0m SECURITY: invalid table name: {}", table);
        return ResolvedData::Rows(Vec::new());
    }

    // Build the GROUP BY expression (with optional time interval)
    let group_expr = match &group.interval {
        Some(interval) => match interval.as_str() {
            "month" => format!("strftime('%Y-%m', \"{}\")", group.field),
            "week"  => format!("strftime('%Y-W%W', \"{}\")", group.field),
            "day"   => format!("strftime('%Y-%m-%d', \"{}\")", group.field),
            "year"  => format!("strftime('%Y', \"{}\")", group.field),
            _       => format!("\"{}\"", group.field),
        },
        None => format!("\"{}\"", group.field),
    };

    // Build the aggregate expression with validated field
    let agg_expr = match &binding.aggregate {
        Some(agg) => {
            let agg_field = agg.field.as_deref().unwrap_or("id");
            if !crate::security::is_safe_identifier(agg_field) {
                eprintln!("  \x1b[31m✗\x1b[0m SECURITY: invalid agg field: {}", agg_field);
                return ResolvedData::Rows(Vec::new());
            }
            match agg.function.as_str() {
                "sum" => format!("SUM(\"{}\")", agg_field),
                "count" => "COUNT(*)".to_string(),
                "avg" => format!("AVG(\"{}\")", agg_field),
                "min" => format!("MIN(\"{}\")", agg_field),
                "max" => format!("MAX(\"{}\")", agg_field),
                _ => "COUNT(*)".to_string(),
            }
        },
        None => "COUNT(*)".to_string(),
    };

    // SECURITY: Build WHERE clause with parameterized values
    let mut param_values: Vec<String> = Vec::new();
    let where_clause = if filters.is_empty() {
        String::new()
    } else {
        let mut parts: Vec<String> = Vec::new();
        for (field, op, val) in filters {
            if !crate::security::is_safe_identifier(field) {
                continue; // skip invalid field names
            }
            let idx = param_values.len() + 1;
            parts.push(format!("\"{}\" {} ?{}", field, op, idx));
            param_values.push(val.clone());
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", parts.join(" AND "))
        }
    };

    let sql = format!(
        "SELECT {} as label, {} as value FROM \"{}\"{} GROUP BY {} ORDER BY label",
        group_expr, agg_expr, table, where_clause, group_expr
    );

    // Use parameterized query
    match db.query_raw_params(&sql, &param_values) {
        Ok(rows) => ResolvedData::Rows(rows),
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Aggregation error ({}): {}", table, e);
            // Fallback: try without params (for backwards compat with non-parameterized query_raw)
            match db.query_raw(&sql.replace(|c: char| c == '?' && false, "")) {
                Ok(rows) => ResolvedData::Rows(rows),
                Err(_) => ResolvedData::Rows(Vec::new()),
            }
        }
    }
}
