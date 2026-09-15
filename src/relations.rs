//! Many-to-many relations: `tags -> Tag[]` inside `entity Post`.
//!
//! Storage (created by `CronusDB::migrate`): join table `Post_tags` with
//! `source_id` → `Post.id` and `target_id` → `Tag.id`, both
//! `ON DELETE CASCADE`, primary key `(source_id, target_id)`, an index on
//! each id column, plus `_owner_id` and `created_at`.
//!
//! Security (LANGUAGE.md §3.7):
//! - join rows carry the parent row's `_owner_id`, whoever writes them;
//! - writes accept only ids the caller may read on the target entity
//!   (`access::read_scope`), so a user cannot attach another user's private
//!   record. A missing id and an unreadable id get the same message;
//! - reads return only linked rows the viewer may read. The parent row has
//!   already been scoped by the caller (REST/GraphQL/forms).

use std::collections::HashMap;

use rusqlite::Connection;
use serde_json::{Map, Value};

use crate::access::{self, Access, ReadScope};
use crate::database::CronusDB;
use crate::parser::{EntityNode, FieldNode};
use crate::security::is_safe_identifier;
use crate::validation::FieldErrors;

pub const SOURCE_COLUMN: &str = "source_id";
pub const TARGET_COLUMN: &str = "target_id";

/// `Post` + `tags` → `Post_tags`.
pub fn join_table(entity: &str, field: &str) -> String {
    format!("{entity}_{field}")
}

/// `CREATE TABLE` + indexes for one many-to-many field, or `None` when a
/// name is not a safe identifier.
pub fn join_table_ddl(entity: &str, field: &str, target: &str) -> Option<String> {
    let table = join_table(entity, field);
    if ![entity, field, target, table.as_str()]
        .iter()
        .all(|n| is_safe_identifier(n))
    {
        return None;
    }
    Some(format!(
        "CREATE TABLE IF NOT EXISTS \"{table}\" (\
         \"{SOURCE_COLUMN}\" TEXT NOT NULL REFERENCES \"{entity}\"(\"id\") ON DELETE CASCADE, \
         \"{TARGET_COLUMN}\" TEXT NOT NULL REFERENCES \"{target}\"(\"id\") ON DELETE CASCADE, \
         \"_owner_id\" TEXT, \
         \"created_at\" TEXT DEFAULT (datetime('now')), \
         PRIMARY KEY (\"{SOURCE_COLUMN}\", \"{TARGET_COLUMN}\"));\n\
         CREATE INDEX IF NOT EXISTS \"idx_{table}_{SOURCE_COLUMN}\" ON \"{table}\"(\"{SOURCE_COLUMN}\");\n\
         CREATE INDEX IF NOT EXISTS \"idx_{table}_{TARGET_COLUMN}\" ON \"{table}\"(\"{TARGET_COLUMN}\");"
    ))
}

/// The ids to store for one many-to-many field of a write (deduplicated,
/// in submitted order). Replaces the field's existing links.
#[derive(Debug, Clone, PartialEq)]
pub struct LinkSet {
    pub field: String,
    pub ids: Vec<String>,
}

/// Splits a write body into `(columns, many_to_many)`.
pub fn split(
    entity: &EntityNode,
    body: Map<String, Value>,
) -> (Map<String, Value>, Map<String, Value>) {
    body.into_iter()
        .partition(|(k, _)| !entity.fields.iter().any(|f| f.is_many() && f.name == *k))
}

/// HTML forms post many-to-many values as `"id1,id2"`; turn them into arrays.
pub fn normalize_form_values(entity: &EntityNode, body: &mut Map<String, Value>) {
    for field in entity.fields.iter().filter(|f| f.is_many()) {
        if let Some(Value::String(raw)) = body.get(&field.name) {
            let ids = raw
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| Value::String(s.to_string()))
                .collect();
            body.insert(field.name.clone(), Value::Array(ids));
        }
    }
}

/// Link sets of a many-to-many body that passed `validation::field_errors`.
/// Absent or `null` fields are left untouched.
pub fn link_sets(many: &Map<String, Value>) -> Vec<LinkSet> {
    many.iter()
        .filter_map(|(field, value)| {
            let mut ids: Vec<String> = Vec::new();
            for id in value.as_array()?.iter().filter_map(Value::as_str) {
                if !ids.iter().any(|seen| seen == id) {
                    ids.push(id.to_string());
                }
            }
            Some(LinkSet {
                field: field.clone(),
                ids,
            })
        })
        .collect()
}

fn target_entity<'a>(
    entities: &'a [EntityNode],
    entity: &EntityNode,
    field: &str,
) -> Option<&'a EntityNode> {
    let target = entity
        .fields
        .iter()
        .find(|f| f.is_many() && f.name == field)?
        .reference
        .as_deref()?;
    entities
        .iter()
        .find(|e| e.name == target && is_safe_identifier(&e.name))
}

/// Read restriction on the target, `None` when the viewer may read nothing.
fn target_scope(access: &Access, target: &EntityNode) -> Option<Option<(&'static str, String)>> {
    match access::read_scope(access, &target.name, Some(target), false) {
        ReadScope::Deny => None,
        scope => Some(scope.condition()),
    }
}

fn placeholders(n: usize) -> String {
    vec!["?"; n].join(", ")
}

/// `{"tags": ["contains an unknown id"]}` for every link set naming an id
/// that does not exist or that the caller may not read.
pub fn access_errors(
    db: &CronusDB,
    entities: &[EntityNode],
    entity: &EntityNode,
    links: &[LinkSet],
    access: &Access,
) -> FieldErrors {
    let mut errors = FieldErrors::new();
    for link in links.iter().filter(|l| !l.ids.is_empty()) {
        let readable = match target_entity(entities, entity, &link.field) {
            None => false,
            Some(target) => all_readable(db, target, &link.ids, access).unwrap_or_else(|e| {
                eprintln!(
                    "  \x1b[31m✗\x1b[0m relation check {}.{} failed: {}",
                    entity.name, link.field, e
                );
                false
            }),
        };
        if !readable {
            errors.insert(link.field.clone(), vec!["contains an unknown id".into()]);
        }
    }
    errors
}

fn all_readable(
    db: &CronusDB,
    target: &EntityNode,
    ids: &[String],
    access: &Access,
) -> Result<bool, String> {
    let Some(condition) = target_scope(access, target) else {
        return Ok(false);
    };
    let mut sql = format!(
        "SELECT COUNT(*) AS n FROM \"{}\" WHERE \"id\" IN ({})",
        target.name,
        placeholders(ids.len())
    );
    let mut params = ids.to_vec();
    if let Some((column, value)) = condition {
        sql.push_str(&format!(" AND \"{column}\" = ?"));
        params.push(value);
    }
    let found = db
        .query_raw_params(&sql, &params)?
        .first()
        .and_then(|r| r["n"].as_i64())
        .unwrap_or(0);
    Ok(usize::try_from(found).is_ok_and(|n| n == ids.len()))
}

/// Replaces the links of `source_id` for one field inside the caller's
/// transaction. `owner` is the parent row's `_owner_id`.
pub fn replace_links(
    conn: &Connection,
    entity: &EntityNode,
    link: &LinkSet,
    source_id: &str,
    owner: Option<&str>,
) -> Result<(), String> {
    let table = join_table(&entity.name, &link.field);
    if !is_safe_identifier(&table) {
        return Err(format!("invalid join table for {}", link.field));
    }
    conn.execute(
        &format!("DELETE FROM \"{table}\" WHERE \"{SOURCE_COLUMN}\" = ?1"),
        [source_id],
    )
    .map_err(|e| e.to_string())?;
    let insert = format!(
        "INSERT OR IGNORE INTO \"{table}\" (\"{SOURCE_COLUMN}\", \"{TARGET_COLUMN}\", \"_owner_id\") VALUES (?1, ?2, ?3)"
    );
    for id in &link.ids {
        conn.execute(&insert, rusqlite::params![source_id, id, owner])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// `?expand=tags,topics` → the named many-to-many fields.
pub fn expand_param(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Sets every many-to-many field on `rows` (one object or an array): an id
/// array, or the linked rows (redacted) for fields listed in `expand`. One
/// query per field for the whole page, never per row.
pub fn attach(
    db: &CronusDB,
    entities: &[EntityNode],
    entity: &EntityNode,
    rows: &mut Value,
    access: &Access,
    expand: &[String],
) {
    let many: Vec<&FieldNode> = entity.fields.iter().filter(|f| f.is_many()).collect();
    if many.is_empty() {
        return;
    }
    let mut targets: Vec<&mut Map<String, Value>> = match rows {
        Value::Array(items) => items.iter_mut().filter_map(Value::as_object_mut).collect(),
        Value::Object(row) => vec![row],
        _ => return,
    };
    let ids: Vec<String> = targets
        .iter()
        .filter_map(|r| r.get("id").and_then(Value::as_str).map(str::to_string))
        .collect();
    for field in many {
        let expanded = expand.iter().any(|e| *e == field.name);
        let mut linked = if ids.is_empty() {
            HashMap::new()
        } else {
            load(db, entities, entity, &field.name, &ids, access, expanded).unwrap_or_else(|e| {
                eprintln!(
                    "  \x1b[31m✗\x1b[0m relation load {}.{} failed: {}",
                    entity.name, field.name, e
                );
                HashMap::new()
            })
        };
        for row in targets.iter_mut() {
            let id = row
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let values = linked.remove(&id).unwrap_or_default();
            row.insert(field.name.clone(), Value::Array(values));
        }
    }

    let to_one: Vec<&FieldNode> = entity
        .fields
        .iter()
        .filter(|f| {
            f.field_type == crate::parser::FieldType::Relation
                && !f.array
                && f.reference.is_some()
                && expand.iter().any(|e| *e == f.name)
        })
        .collect();
    for field in to_one {
        let fks: Vec<String> = targets
            .iter()
            .filter_map(|r| {
                r.get(&field.name)
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .filter(|s| !s.is_empty())
            .collect();
        let loaded = load_to_one(db, entities, field, &fks, access).unwrap_or_default();
        for row in targets.iter_mut() {
            let fk = row
                .get(&field.name)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            if let Some(obj) = loaded.get(&fk) {
                row.insert(field.name.clone(), obj.clone());
            }
        }
    }
}

fn load_to_one(
    db: &CronusDB,
    entities: &[EntityNode],
    field: &FieldNode,
    ids: &[String],
    access: &Access,
) -> Result<HashMap<String, Value>, String> {
    let mut out = HashMap::new();
    let Some(target_name) = field.reference.as_deref() else {
        return Ok(out);
    };
    let Some(target) = entities
        .iter()
        .find(|e| e.name == target_name && is_safe_identifier(&e.name))
    else {
        return Ok(out);
    };
    let Some(condition) = target_scope(access, target) else {
        return Ok(out);
    };
    let mut unique = Vec::new();
    for id in ids {
        if !id.is_empty() && !unique.iter().any(|s: &String| s == id) {
            unique.push(id.clone());
        }
    }
    if unique.is_empty() {
        return Ok(out);
    }
    let mut sql = format!(
        "SELECT * FROM \"{}\" WHERE \"id\" IN ({})",
        target.name,
        placeholders(unique.len())
    );
    let mut params = unique.clone();
    if let Some((column, value)) = condition {
        sql.push_str(&format!(" AND \"{column}\" = ?"));
        params.push(value);
    }
    for mut row in db.query_raw_params(&sql, &params)? {
        crate::authz::redact_sensitive(target, &mut row);
        if let Some(id) = row.get("id").and_then(Value::as_str).map(str::to_string) {
            out.insert(id, row);
        }
    }
    Ok(out)
}

/// Text for a bound cell: scalars as-is; related objects as label/name/title/id.
pub fn display_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Array(items) => items
            .iter()
            .map(display_value)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", "),
        Value::Object(o) => o
            .get("label")
            .or_else(|| o.get("name"))
            .or_else(|| o.get("title"))
            .or_else(|| o.get("id"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        _ => String::new(),
    }
}

const SOURCE_ALIAS: &str = "__cronus_source";

fn load(
    db: &CronusDB,
    entities: &[EntityNode],
    entity: &EntityNode,
    field: &str,
    source_ids: &[String],
    access: &Access,
    expanded: bool,
) -> Result<HashMap<String, Vec<Value>>, String> {
    let mut out: HashMap<String, Vec<Value>> = HashMap::new();
    let Some(target) = target_entity(entities, entity, field) else {
        return Ok(out);
    };
    let Some(condition) = target_scope(access, target) else {
        return Ok(out);
    };
    let table = join_table(&entity.name, field);
    if !is_safe_identifier(&table) {
        return Ok(out);
    }
    let mut sql = format!(
        "SELECT j.\"{SOURCE_COLUMN}\" AS \"{SOURCE_ALIAS}\", t.* FROM \"{table}\" j \
         JOIN \"{}\" t ON t.\"id\" = j.\"{TARGET_COLUMN}\" \
         WHERE j.\"{SOURCE_COLUMN}\" IN ({})",
        target.name,
        placeholders(source_ids.len())
    );
    let mut params = source_ids.to_vec();
    if let Some((column, value)) = condition {
        sql.push_str(&format!(" AND t.\"{column}\" = ?"));
        params.push(value);
    }
    sql.push_str(" ORDER BY j.rowid");
    for mut row in db.query_raw_params(&sql, &params)? {
        let source = row
            .as_object_mut()
            .and_then(|m| m.remove(SOURCE_ALIAS))
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default();
        let item = if expanded {
            crate::authz::redact_sensitive(target, &mut row);
            row
        } else {
            row.get("id").cloned().unwrap_or(Value::Null)
        };
        out.entry(source).or_default().push(item);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::access::test_support::{as_admin, as_user};
    use crate::parser::{parse, AstNode};
    use serde_json::json;

    const SRC: &str = "entity Post {\n  title string!\n  tags -> Tag[]\n}\nentity Tag {\n  label string!\n  secret string sensitive\n}\n";

    fn setup() -> (CronusDB, Vec<EntityNode>) {
        let ents: Vec<EntityNode> = parse(SRC)
            .unwrap()
            .into_iter()
            .filter_map(|n| match n {
                AstNode::Entity(e) => Some(e),
                _ => None,
            })
            .collect();
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&ents).unwrap();
        (db, ents)
    }

    fn row(db: &CronusDB, table: &str, v: Value) -> String {
        db.insert(table, &v).unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn ddl_rejects_unsafe_names() {
        assert!(join_table_ddl("Post", "tags", "Tag").is_some());
        assert!(join_table_ddl("Post", "tags\"; DROP", "Tag").is_none());
    }

    #[test]
    fn split_and_link_sets_dedupe_and_keep_order() {
        let (_, ents) = setup();
        let post = &ents[0];
        let body = json!({"title": "t", "tags": ["b", "a", "b"]});
        let (cols, many) = split(post, body.as_object().unwrap().clone());
        assert_eq!(Value::Object(cols), json!({"title": "t"}));
        assert_eq!(
            link_sets(&many),
            vec![LinkSet {
                field: "tags".into(),
                ids: vec!["b".into(), "a".into()]
            }]
        );
        let mut form = json!({"tags": " a, ,b "}).as_object().unwrap().clone();
        normalize_form_values(post, &mut form);
        assert_eq!(form["tags"], json!(["a", "b"]));
    }

    #[test]
    fn access_errors_use_the_targets_read_scope() {
        let (db, ents) = setup();
        let mine = row(&db, "Tag", json!({"label": "m", "_owner_id": "alice"}));
        let theirs = row(&db, "Tag", json!({"label": "t", "_owner_id": "bob"}));
        let link = |ids: Vec<String>| {
            vec![LinkSet {
                field: "tags".into(),
                ids,
            }]
        };
        let post = &ents[0];
        assert!(access_errors(
            &db,
            &ents,
            post,
            &link(vec![mine.clone()]),
            &as_user("alice")
        )
        .is_empty());
        let denied = access_errors(
            &db,
            &ents,
            post,
            &link(vec![mine.clone(), theirs.clone()]),
            &as_user("alice"),
        );
        assert_eq!(denied["tags"], vec!["contains an unknown id"]);
        assert!(access_errors(&db, &ents, post, &link(vec![mine, theirs]), &as_admin()).is_empty());
    }

    #[test]
    fn attach_batches_ids_and_expands_redacted_rows_in_scope() {
        let (db, ents) = setup();
        let post = &ents[0];
        let p1 = row(&db, "Post", json!({"title": "1", "_owner_id": "alice"}));
        let p2 = row(&db, "Post", json!({"title": "2", "_owner_id": "alice"}));
        let t1 = row(
            &db,
            "Tag",
            json!({"label": "x", "secret": "s", "_owner_id": "alice"}),
        );
        let t2 = row(&db, "Tag", json!({"label": "y", "_owner_id": "bob"}));
        db.transaction(|conn| {
            let set = LinkSet {
                field: "tags".into(),
                ids: vec![t1.clone(), t2.clone()],
            };
            replace_links(conn, post, &set, &p1, Some("alice"))
        })
        .unwrap();

        let mut rows = json!([{"id": p1}, {"id": p2}]);
        attach(&db, &ents, post, &mut rows, &as_user("alice"), &[]);
        assert_eq!(
            rows[0]["tags"],
            json!([t1]),
            "bob's tag is not visible to alice"
        );
        assert_eq!(rows[1]["tags"], json!([]));

        let mut one = json!({"id": p1});
        attach(&db, &ents, post, &mut one, &as_admin(), &["tags".into()]);
        let tags = one["tags"].as_array().unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0]["label"], "x");
        assert!(tags[0].get("secret").is_none());
    }
}
