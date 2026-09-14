//! Shared authorization contract for every data surface (REST, GraphQL,
//! SSR bindings, form/actions, webhooks, audit trail). Sprint 1 security:
//! one place decides which fields leave the server and which fields a client
//! may write, and one shape for errors that reach users.

use crate::parser::EntityNode;
use serde_json::{json, Map, Value};

/// Columns the server owns; a client body can never set them.
pub const SYSTEM_FIELDS: &[&str] = &[
    "id",
    "_owner_id",
    "created_at",
    "updated_at",
    "createdAt",
    "updatedAt",
];

/// Fields a client may not write through generic data surfaces
/// (REST CRUD, GraphQL mutations, forms/actions). `role` is included so
/// privilege changes only happen through dedicated admin/auth code.
pub const PRIVILEGED_FIELDS: &[&str] = &["role", "password", "password_hash"];

pub fn sensitive_names(entity: &EntityNode) -> Vec<&str> {
    entity
        .fields
        .iter()
        .filter(|f| f.sensitive)
        .map(|f| f.name.as_str())
        .collect()
}

/// Removes `sensitive` fields (plus password columns) from a row or an
/// array of rows before it is serialized to any client.
pub fn redact_sensitive(entity: &EntityNode, value: &mut Value) {
    let hidden = sensitive_names(entity);
    match value {
        Value::Array(rows) => rows.iter_mut().for_each(|r| redact_row(&hidden, r)),
        row => redact_row(&hidden, row),
    }
}

fn redact_row(hidden: &[&str], row: &mut Value) {
    if let Value::Object(map) = row {
        map.retain(|k, _| !hidden.contains(&k.as_str()) && k != "password" && k != "password_hash");
    }
}

/// Keeps only fields the client may write: declared entity fields that are
/// neither system, privileged nor sensitive. Unknown keys are dropped.
pub fn writable_body(entity: &EntityNode, body: &Map<String, Value>) -> Map<String, Value> {
    body.iter()
        .filter(|(k, _)| {
            entity.fields.iter().any(|f| &f.name == *k && !f.sensitive)
                && !SYSTEM_FIELDS.contains(&k.as_str())
                && !PRIVILEGED_FIELDS.contains(&k.as_str())
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Stable, safe error body. Never pass database/IO error text as `message`;
/// log the detail once at the boundary instead.
pub fn error_body(code: &str, message: &str) -> Value {
    json!({ "error": { "code": code, "message": message } })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn user_entity() -> EntityNode {
        let src = "app \"T\" { port 5175 }\nentity User {\n  email email!\n  name string\n  role string\n  password string! sensitive\n}\n";
        parse(src)
            .expect("parse")
            .into_iter()
            .find_map(|n| match n {
                crate::parser::AstNode::Entity(e) if e.name == "User" => Some(e),
                _ => None,
            })
            .expect("User")
    }

    #[test]
    fn redacts_sensitive_and_password_from_rows_and_arrays() {
        let e = user_entity();
        let mut row = json!({"id":"1","email":"a@b","password":"$argon2id$x","password_hash":"h"});
        redact_sensitive(&e, &mut row);
        assert_eq!(row, json!({"id":"1","email":"a@b"}));
        let mut rows = json!([{"email":"a","password":"p"},{"email":"b","password":"q"}]);
        redact_sensitive(&e, &mut rows);
        assert_eq!(rows, json!([{"email":"a"},{"email":"b"}]));
    }

    #[test]
    fn writable_body_drops_system_privileged_sensitive_and_unknown() {
        let e = user_entity();
        let body = json!({"email":"x@y","name":"n","role":"admin","password":"p","_owner_id":"o","id":"i","bogus":1});
        let out = writable_body(&e, body.as_object().unwrap());
        assert_eq!(Value::Object(out), json!({"email":"x@y","name":"n"}));
    }

    #[test]
    fn error_body_has_stable_shape() {
        assert_eq!(
            error_body("NOT_FOUND", "Not found"),
            json!({"error":{"code":"NOT_FOUND","message":"Not found"}})
        );
    }
}
