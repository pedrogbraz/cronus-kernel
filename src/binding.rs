#![allow(dead_code, unused_imports)]

use serde_json::Value;
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
fn binding_value_to_string(val: &BindingValue) -> String {
    match val {
        BindingValue::Str(s) => s.clone(),
        BindingValue::Num(n) => n.clone(),
        BindingValue::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
        BindingValue::AuthRef(r) => {
            // TODO: resolve auth refs against current user session
            // For now, return the ref as placeholder
            format!("${}", r)
        }
    }
}

/// Prepare filter tuples for the database query builder.
/// Each tuple is (field_name, sql_operator, value_string).
fn prepare_filters(binding: &BindingNode) -> Vec<(String, String, String)> {
    binding.filters.iter().map(|f| {
        let sql_op = crate::database::filter_op_to_sql(op_to_str(&f.operator)).to_string();
        let mut value = binding_value_to_string(&f.value);

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
pub fn resolve_binding(section: &SectionNode, db: &CronusDB) -> ResolvedData {
    let binding = match &section.binding {
        Some(b) => b,
        None => return ResolvedData::None,
    };

    let table = binding.entity.to_lowercase() + "s"; // pluralize: Order -> orders
    let filters = prepare_filters(binding);

    let order_field = binding.order.as_ref().map(|o| o.field.as_str());
    let order_dir = binding.order.as_ref().map(|o| {
        match o.direction {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC",
        }
    });

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
