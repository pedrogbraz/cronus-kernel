use crate::access::{self, Access, ReadScope};
use crate::database::{CronusDB, SqlFilter};
use crate::parser::{
    BindingNode, BindingValue, EntityNode, FilterOp, OrderDirection, QueryType, SectionNode,
};
use serde_json::Value;
use std::collections::HashMap;

/// The result of resolving a binding against the database.
#[derive(Debug, Clone)]
pub enum ResolvedData {
    /// query all -> rows
    Rows(Vec<Value>),
    /// query one -> single record or None
    Record(Option<Value>),
    /// query count / `aggregate count` -> number
    Count(u64),
    /// `aggregate sum|avg|min|max field:x` without `group_by` -> one value
    Scalar(Value),
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
        FilterOp::EndsWith => "ends_with",
        FilterOp::In => "in",
    }
}

/// Resolve a filter value. `route.*` comes from URL params, `auth.id`,
/// `auth.role` and `auth.email` from the session. `None` means the filter
/// cannot be satisfied (no session, unknown ref) — the binding yields no data.
fn binding_value_to_string(
    val: &BindingValue,
    route_params: &HashMap<String, String>,
    access: &Access,
    db: &CronusDB,
) -> Option<String> {
    match val {
        BindingValue::Str(s) => Some(s.clone()),
        BindingValue::Num(n) => Some(n.clone()),
        BindingValue::Bool(b) => Some(if *b { "1".to_string() } else { "0".to_string() }),
        BindingValue::AuthRef(r) => {
            if let Some(param_name) = r.strip_prefix("route.") {
                return Some(route_params.get(param_name).cloned().unwrap_or_default());
            }
            let viewer = access.viewer.as_ref()?;
            match r.strip_prefix("auth.")? {
                "id" => Some(viewer.id.clone()),
                "role" => Some(viewer.role.clone()),
                "email" => db
                    .find_by_id(access.auth_entity_name(), &viewer.id)
                    .ok()
                    .flatten()
                    .and_then(|row| {
                        row.get("email")
                            .and_then(|v| v.as_str())
                            .map(str::to_string)
                    }),
                _ => None,
            }
        }
        BindingValue::List(_) => None,
    }
}

fn list_values(
    val: &BindingValue,
    route_params: &HashMap<String, String>,
    access: &Access,
    db: &CronusDB,
) -> Option<Vec<String>> {
    match val {
        BindingValue::List(items) => items
            .iter()
            .map(|item| binding_value_to_string(item, route_params, access, db))
            .collect(),
        other => Some(vec![binding_value_to_string(
            other,
            route_params,
            access,
            db,
        )?]),
    }
}

/// Prepare parameterized predicates for the query builder.
fn prepare_filters(
    binding: &BindingNode,
    route_params: &HashMap<String, String>,
    access: &Access,
    db: &CronusDB,
) -> Option<Vec<SqlFilter>> {
    binding
        .filters
        .iter()
        .map(|f| {
            let sql_op = crate::database::filter_op_to_sql(op_to_str(&f.operator)).to_string();
            if f.operator == FilterOp::In {
                let values = list_values(&f.value, route_params, access, db)?;
                return Some(SqlFilter {
                    field: f.field.clone(),
                    op: sql_op,
                    values,
                });
            }
            let mut value = binding_value_to_string(&f.value, route_params, access, db)?;
            match f.operator {
                FilterOp::Contains => {
                    value = format!("%{}%", value);
                }
                FilterOp::StartsWith => {
                    value = format!("{}%", value);
                }
                FilterOp::EndsWith => {
                    value = format!("%{}", value);
                }
                _ => {}
            }
            Some(SqlFilter::one(f.field.clone(), sql_op, value))
        })
        .collect()
}

fn empty_for(binding: &BindingNode) -> ResolvedData {
    if binding.group_by.is_some() {
        return ResolvedData::Rows(Vec::new());
    }
    if let Some(agg) = &binding.aggregate {
        return if agg.function == "count" {
            ResolvedData::Count(0)
        } else {
            ResolvedData::Scalar(Value::from(0))
        };
    }
    match binding.query {
        QueryType::All => ResolvedData::Rows(Vec::new()),
        QueryType::One => ResolvedData::Record(None),
        QueryType::Count => ResolvedData::Count(0),
    }
}

fn is_hidden_field(entity: Option<&EntityNode>, field: &str) -> bool {
    field == "password"
        || field == "password_hash"
        || entity
            .map(|e| crate::authz::sensitive_names(e).contains(&field))
            .unwrap_or(false)
}

/// Resolve a section's binding against the database.
///
/// This is the ONLY place where binding -> database query happens.
/// Authorization (see `access.rs` and LANGUAGE.md §9.5): anonymous viewers get
/// data only from `scope:public` bindings (never the auth entity); signed-in
/// non-admins are owner-scoped except for `shared` entities and `scope:public`;
/// admins see every row. Sensitive fields never leave this function.
pub fn resolve_binding(
    section: &SectionNode,
    db: &CronusDB,
    route_params: &HashMap<String, String>,
    access: &Access,
    entities: &[EntityNode],
) -> ResolvedData {
    let binding = match &section.binding {
        Some(b) => b,
        None => return ResolvedData::None,
    };

    let table = binding.entity.clone();
    let entity = entities.iter().find(|e| e.name == table);

    let scope_filter = match access::read_scope(access, &table, entity, binding.public) {
        ReadScope::Deny => return empty_for(binding),
        scope => scope.filter(),
    };
    let mut filters = match prepare_filters(binding, route_params, access, db) {
        Some(f) => f,
        None => return empty_for(binding),
    };
    if let Some(triple) = scope_filter {
        filters.push(SqlFilter::from_triple(triple));
    }

    let order_field = binding.order.as_ref().map(|o| o.field.as_str());
    let order_dir = binding.order.as_ref().map(|o| match o.direction {
        OrderDirection::Asc => "ASC",
        OrderDirection::Desc => "DESC",
    });

    if binding.group_by.is_some() {
        return resolve_aggregation(binding, entity, &table, &filters, db);
    }
    if let Some(agg) = &binding.aggregate {
        return resolve_scalar_aggregate(agg, entity, &table, &filters, db);
    }

    let redact = |mut v: Value| {
        if let Some(e) = entity {
            crate::authz::redact_sensitive(e, &mut v);
        }
        v
    };
    let attach = |mut v: Value| {
        if let Some(e) = entity {
            crate::relations::attach(db, entities, e, &mut v, access, &binding.expand);
        }
        v
    };

    match binding.query {
        QueryType::All => {
            match db.find_many(
                &table,
                &filters,
                order_field,
                order_dir,
                binding.limit,
                binding.offset,
            ) {
                Ok(Value::Array(rows)) => {
                    let rows: Vec<Value> = rows.into_iter().map(redact).collect();
                    let mut wrapped = Value::Array(rows);
                    wrapped = attach(wrapped);
                    match wrapped {
                        Value::Array(rows) => ResolvedData::Rows(rows),
                        other => ResolvedData::Rows(vec![other]),
                    }
                }
                Ok(other) => ResolvedData::Rows(vec![attach(redact(other))]),
                Err(e) => {
                    eprintln!("  \x1b[31m✗\x1b[0m Binding error ({}): {}", table, e);
                    ResolvedData::Rows(Vec::new())
                }
            }
        }
        QueryType::One => match db.find_one(&table, &filters, order_field, order_dir) {
            Ok(record) => ResolvedData::Record(record.map(|r| attach(redact(r)))),
            Err(e) => {
                eprintln!("  \x1b[31m✗\x1b[0m Binding error ({}): {}", table, e);
                ResolvedData::Record(None)
            }
        },
        QueryType::Count => match db.count_where(&table, &filters) {
            Ok(n) => ResolvedData::Count(n),
            Err(e) => {
                eprintln!("  \x1b[31m✗\x1b[0m Binding error ({}): {}", table, e);
                ResolvedData::Count(0)
            }
        },
    }
}

/// ` WHERE "f" op ?1 AND …` with bound values. `None` when a filter cannot be
/// expressed safely: it must not be silently dropped, since that would widen
/// the result (e.g. lose the owner scope).
fn where_params(table: &str, filters: &[SqlFilter]) -> Option<(String, Vec<String>)> {
    match crate::database::sql_where(filters) {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!(
                "  \x1b[31m✗\x1b[0m SECURITY: invalid aggregation filter on {}: {}",
                table, e
            );
            None
        }
    }
}

/// `aggregate count` → `Count`; `aggregate sum|avg|min|max field:x` → `Scalar`.
/// Same filters (owner scope included) and field checks as grouped aggregations.
fn resolve_scalar_aggregate(
    agg: &crate::parser::AggregateExpr,
    entity: Option<&EntityNode>,
    table: &str,
    filters: &[SqlFilter],
    db: &CronusDB,
) -> ResolvedData {
    if agg.function == "count" {
        return match db.count_where(table, filters) {
            Ok(n) => ResolvedData::Count(n),
            Err(e) => {
                eprintln!("  \x1b[31m✗\x1b[0m Aggregate error ({}): {}", table, e);
                ResolvedData::Count(0)
            }
        };
    }
    let zero = ResolvedData::Scalar(Value::from(0));
    let func = match agg.function.as_str() {
        "sum" => "SUM",
        "avg" => "AVG",
        "min" => "MIN",
        "max" => "MAX",
        other => {
            eprintln!(
                "  \x1b[31m✗\x1b[0m Unknown aggregate '{}' on {}",
                other, table
            );
            return zero;
        }
    };
    let Some(field) = agg.field.as_deref() else {
        eprintln!(
            "  \x1b[31m✗\x1b[0m aggregate {} on {} needs field:<name>",
            agg.function, table
        );
        return zero;
    };
    if !crate::security::is_safe_identifier(table)
        || !crate::security::is_safe_identifier(field)
        || is_hidden_field(entity, field)
    {
        eprintln!(
            "  \x1b[31m✗\x1b[0m SECURITY: invalid aggregate on {}.{}",
            table, field
        );
        return zero;
    }
    let Some((where_clause, params)) = where_params(table, filters) else {
        return zero;
    };
    let sql = format!(
        "SELECT {}(\"{}\") AS value FROM \"{}\"{}",
        func, field, table, where_clause
    );
    match db.query_raw_params(&sql, &params) {
        Ok(rows) => match rows
            .into_iter()
            .next()
            .and_then(|r| r.get("value").cloned())
        {
            Some(Value::Null) | None => zero,
            Some(v) => ResolvedData::Scalar(v),
        },
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Aggregate error ({}): {}", table, e);
            zero
        }
    }
}

/// Build and execute a GROUP BY + aggregate SQL query.
/// Returns Rows with {label, value} objects for chart consumption.
///
/// SECURITY: All field names are validated as safe identifiers before use in SQL.
/// Filter values (including the owner scope) are bound parameters.
fn resolve_aggregation(
    binding: &BindingNode,
    entity: Option<&EntityNode>,
    table: &str,
    filters: &[SqlFilter],
    db: &CronusDB,
) -> ResolvedData {
    let group = match binding.group_by.as_ref() {
        Some(g) => g,
        None => return ResolvedData::Rows(Vec::new()),
    };

    if !crate::security::is_safe_identifier(&group.field) || is_hidden_field(entity, &group.field) {
        eprintln!(
            "  \x1b[31m✗\x1b[0m SECURITY: invalid group field name: {}",
            group.field
        );
        return ResolvedData::Rows(Vec::new());
    }
    if !crate::security::is_safe_identifier(table) {
        eprintln!("  \x1b[31m✗\x1b[0m SECURITY: invalid table name: {}", table);
        return ResolvedData::Rows(Vec::new());
    }

    let group_expr = match &group.interval {
        Some(interval) => match interval.as_str() {
            "month" => format!("strftime('%Y-%m', \"{}\")", group.field),
            "week" => format!("strftime('%Y-W%W', \"{}\")", group.field),
            "day" => format!("strftime('%Y-%m-%d', \"{}\")", group.field),
            "year" => format!("strftime('%Y', \"{}\")", group.field),
            _ => format!("\"{}\"", group.field),
        },
        None => format!("\"{}\"", group.field),
    };

    let agg_expr = match &binding.aggregate {
        Some(agg) => {
            let agg_field = agg.field.as_deref().unwrap_or("id");
            if !crate::security::is_safe_identifier(agg_field) || is_hidden_field(entity, agg_field)
            {
                eprintln!(
                    "  \x1b[31m✗\x1b[0m SECURITY: invalid agg field: {}",
                    agg_field
                );
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
        }
        None => "COUNT(*)".to_string(),
    };

    let Some((where_clause, param_values)) = where_params(table, filters) else {
        return ResolvedData::Rows(Vec::new());
    };

    let sql = format!(
        "SELECT {} as label, {} as value FROM \"{}\"{} GROUP BY {} ORDER BY label",
        group_expr, agg_expr, table, where_clause, group_expr
    );

    match db.query_raw_params(&sql, &param_values) {
        Ok(rows) => ResolvedData::Rows(rows),
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Aggregation error ({}): {}", table, e);
            ResolvedData::Rows(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::access::test_support::*;
    use crate::parser::{parse, AstNode};

    fn section(bind: &str) -> SectionNode {
        let src = format!("app \"T\" {{ port 5175 }}\npage \"/p\" type:custom {{\n  section table {{\n    {}\n  }}\n}}\n", bind);
        parse(&src)
            .expect("parse")
            .into_iter()
            .find_map(|n| match n {
                AstNode::Page(p) => p.sections.into_iter().next(),
                _ => None,
            })
            .expect("section")
    }

    fn rows(data: ResolvedData) -> Vec<Value> {
        match data {
            ResolvedData::Rows(r) => r,
            other => panic!("expected rows, got {:?}", other),
        }
    }

    fn fixture() -> (Vec<EntityNode>, CronusDB) {
        let ents = entities();
        let db = db(&ents);
        insert_note(&db, "alice", "a1");
        insert_note(&db, "alice", "a2");
        insert_note(&db, "bob", "b1");
        (ents, db)
    }

    #[test]
    fn anonymous_gets_no_rows_without_scope_public() {
        let (ents, db) = fixture();
        let p = HashMap::new();
        assert!(rows(resolve_binding(
            &section("bind Note { query all }"),
            &db,
            &p,
            &anon(),
            &ents
        ))
        .is_empty());
        match resolve_binding(
            &section("bind Note { query count }"),
            &db,
            &p,
            &anon(),
            &ents,
        ) {
            ResolvedData::Count(n) => assert_eq!(n, 0),
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn anonymous_scope_public_sees_rows_but_never_the_auth_entity() {
        let (ents, db) = fixture();
        db.insert(
            "User",
            &serde_json::json!({"email":"a@b.co","password":"h"}),
        )
        .unwrap();
        let p = HashMap::new();
        assert_eq!(
            rows(resolve_binding(
                &section("bind Note { query all scope:public }"),
                &db,
                &p,
                &anon(),
                &ents
            ))
            .len(),
            3
        );
        assert!(rows(resolve_binding(
            &section("bind User { query all scope:public }"),
            &db,
            &p,
            &anon(),
            &ents
        ))
        .is_empty());
    }

    #[test]
    fn owner_sees_only_own_rows_and_admin_sees_all() {
        let (ents, db) = fixture();
        let p = HashMap::new();
        let s = section("bind Note { query all }");
        let alice = rows(resolve_binding(&s, &db, &p, &as_user("alice"), &ents));
        assert_eq!(alice.len(), 2);
        assert!(alice.iter().all(|r| r["_owner_id"] == "alice"));
        assert_eq!(
            rows(resolve_binding(&s, &db, &p, &as_admin(), &ents)).len(),
            3
        );
    }

    #[test]
    fn sensitive_fields_are_redacted() {
        let (ents, db) = fixture();
        let p = HashMap::new();
        let r = rows(resolve_binding(
            &section("bind Note { query all }"),
            &db,
            &p,
            &as_user("alice"),
            &ents,
        ));
        assert!(r
            .iter()
            .all(|row| row.get("secret").is_none() && row.get("title").is_some()));
    }

    #[test]
    fn auth_refs_resolve_from_session_and_fail_closed_without_one() {
        let (ents, db) = fixture();
        let p = HashMap::new();
        let s = section("bind Note { query all where _owner_id eq auth.id scope:public }");
        let bob = rows(resolve_binding(&s, &db, &p, &as_user("bob"), &ents));
        assert_eq!(bob.len(), 1);
        assert_eq!(bob[0]["title"], "b1");
        assert!(rows(resolve_binding(&s, &db, &p, &anon(), &ents)).is_empty());
    }

    #[test]
    fn aggregate_count_without_group_by_is_a_scoped_count() {
        let (ents, db) = fixture();
        let p = HashMap::new();
        let s = section("bind Note { aggregate count }");
        let count = |a: &Access| match resolve_binding(&s, &db, &p, a, &ents) {
            ResolvedData::Count(n) => n,
            other => panic!("expected count, got {:?}", other),
        };
        assert_eq!(count(&as_user("alice")), 2);
        assert_eq!(count(&as_user("carol")), 0);
        assert_eq!(count(&anon()), 0);
        assert_eq!(count(&as_admin()), 3);
    }

    #[test]
    fn scalar_aggregate_uses_field_and_owner_scope() {
        let (ents, db) = fixture();
        let p = HashMap::new();
        let s = section("bind Note { aggregate max field:title }");
        match resolve_binding(&s, &db, &p, &as_user("alice"), &ents) {
            ResolvedData::Scalar(v) => assert_eq!(v, "a2"),
            other => panic!("expected scalar, got {:?}", other),
        }
        let hidden = section("bind Note { aggregate max field:secret }");
        match resolve_binding(&hidden, &db, &p, &as_admin(), &ents) {
            ResolvedData::Scalar(v) => assert_eq!(v, 0, "sensitive field never aggregated"),
            other => panic!("expected scalar, got {:?}", other),
        }
    }

    #[test]
    fn aggregation_binds_filters_and_applies_owner_scope() {
        let (ents, db) = fixture();
        let p = HashMap::new();
        let s = section("bind Note { aggregate count group_by:title }");
        let alice = rows(resolve_binding(&s, &db, &p, &as_user("alice"), &ents));
        assert_eq!(alice.len(), 2, "{:?}", alice);
        assert!(rows(resolve_binding(&s, &db, &p, &anon(), &ents)).is_empty());
    }

    #[test]
    fn ends_with_and_in_filter_rows() {
        let (ents, db) = fixture();
        let p = HashMap::new();
        let alice = as_user("alice");
        let ends = rows(resolve_binding(
            &section("bind Note { query all where title ends_with \"1\" }"),
            &db,
            &p,
            &alice,
            &ents,
        ));
        assert_eq!(ends.len(), 1, "{ends:?}");
        assert_eq!(ends[0]["title"], "a1");
        let listed = rows(resolve_binding(
            &section("bind Note { query all where title in:[a1, missing] }"),
            &db,
            &p,
            &alice,
            &ents,
        ));
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0]["title"], "a1");
        let empty = rows(resolve_binding(
            &section("bind Note { query all where title in:[] }"),
            &db,
            &p,
            &alice,
            &ents,
        ));
        assert!(empty.is_empty());
    }

    #[test]
    fn expand_loads_related_rows_for_ssr() {
        let src = "entity Post { title string!  tags -> Tag[] }\nentity Tag { label string!  secret string sensitive }\n";
        let ents: Vec<EntityNode> = parse(src)
            .unwrap()
            .into_iter()
            .filter_map(|n| match n {
                AstNode::Entity(e) => Some(e),
                _ => None,
            })
            .collect();
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&ents).unwrap();
        let tag = db
            .insert(
                "Tag",
                &serde_json::json!({"label": "rust", "secret": "s", "_owner_id": "alice"}),
            )
            .unwrap();
        let tag_id = tag["id"].as_str().unwrap();
        let post = db
            .insert(
                "Post",
                &serde_json::json!({"title": "hello", "_owner_id": "alice"}),
            )
            .unwrap();
        let post_id = post["id"].as_str().unwrap();
        db.transaction(|conn| {
            crate::relations::replace_links(
                conn,
                &ents[0],
                &crate::relations::LinkSet {
                    field: "tags".into(),
                    ids: vec![tag_id.to_string()],
                },
                post_id,
                Some("alice"),
            )
        })
        .unwrap();
        let p = HashMap::new();
        let ids_only = rows(resolve_binding(
            &section("bind Post { query all }"),
            &db,
            &p,
            &as_user("alice"),
            &ents,
        ));
        assert_eq!(ids_only[0]["tags"], serde_json::json!([tag_id]));
        let expanded = rows(resolve_binding(
            &section("bind Post { query all expand:tags }"),
            &db,
            &p,
            &as_user("alice"),
            &ents,
        ));
        let tags = expanded[0]["tags"].as_array().unwrap();
        assert_eq!(tags[0]["label"], "rust");
        assert!(tags[0].get("secret").is_none());
        assert_eq!(
            crate::relations::display_value(&expanded[0]["tags"]),
            "rust"
        );
    }

    #[test]
    fn expand_reverse_relation_on_ssr() {
        let src = "entity Customer { name string! }\nentity Order { title string!  customer -> Customer }\n";
        let ents: Vec<EntityNode> = parse(src)
            .unwrap()
            .into_iter()
            .filter_map(|n| match n {
                AstNode::Entity(e) => Some(e),
                _ => None,
            })
            .collect();
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&ents).unwrap();
        let customer = db
            .insert(
                "Customer",
                &serde_json::json!({"name": "Ada", "_owner_id": "alice"}),
            )
            .unwrap();
        let cid = customer["id"].as_str().unwrap();
        db.insert(
            "Order",
            &serde_json::json!({"title": "one", "customer": cid, "_owner_id": "alice"}),
        )
        .unwrap();
        let p = HashMap::new();
        let listed = rows(resolve_binding(
            &section("bind Customer { query all expand:orders }"),
            &db,
            &p,
            &as_user("alice"),
            &ents,
        ));
        let orders = listed[0]["orders"].as_array().unwrap();
        assert_eq!(orders[0]["title"], "one");
    }
}
