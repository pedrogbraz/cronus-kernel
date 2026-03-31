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
pub fn resolve_binding(section: &SectionNode, db: &CronusDB, route_params: &HashMap<String, String>) -> ResolvedData {
    let binding = match &section.binding {
        Some(b) => b,
        None => return ResolvedData::None,
    };

    let table = binding.entity.to_lowercase() + "s"; // pluralize: Order -> orders
    let filters = prepare_filters(binding, route_params);

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
fn resolve_aggregation(
    binding: &BindingNode,
    table: &str,
    filters: &[(String, String, String)],
    db: &CronusDB,
) -> ResolvedData {
    let group = binding.group_by.as_ref().unwrap();

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

    // Build the aggregate expression
    let agg_expr = match &binding.aggregate {
        Some(agg) => match agg.function.as_str() {
            "sum" => format!("SUM(\"{}\")", agg.field.as_deref().unwrap_or("id")),
            "count" => "COUNT(*)".to_string(),
            "avg" => format!("AVG(\"{}\")", agg.field.as_deref().unwrap_or("id")),
            "min" => format!("MIN(\"{}\")", agg.field.as_deref().unwrap_or("id")),
            "max" => format!("MAX(\"{}\")", agg.field.as_deref().unwrap_or("id")),
            _ => "COUNT(*)".to_string(),
        },
        None => "COUNT(*)".to_string(),
    };

    // Build WHERE clause from filters
    let where_clause = if filters.is_empty() {
        String::new()
    } else {
        let parts: Vec<String> = filters.iter().map(|(field, op, val)| {
            format!("\"{}\" {} '{}'", field, op, val.replace('\'', "''"))
        }).collect();
        format!(" WHERE {}", parts.join(" AND "))
    };

    let sql = format!(
        "SELECT {} as label, {} as value FROM \"{}\"{} GROUP BY {} ORDER BY label",
        group_expr, agg_expr, table, where_clause, group_expr
    );

    match db.query_raw(&sql) {
        Ok(rows) => ResolvedData::Rows(rows),
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Aggregation error ({}): {}", table, e);
            ResolvedData::Rows(Vec::new())
        }
    }
}
