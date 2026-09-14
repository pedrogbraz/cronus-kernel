//! Who is asking and which rows they may touch — the single authorization
//! source for every data surface: REST (`api_crud.rs`), GraphQL, SSR
//! bindings, `/_form` + `/_action` (`actions.rs`) and SSE.
//!
//! Model (Sprint 1 security):
//! - no session → no data, unless the author explicitly opted in
//!   (`bind X { scope:public }` or an `auth:public` REST route), and never
//!   for the auth entity;
//! - admin → every row;
//! - authenticated non-admin → own rows (`_owner_id`), `shared` entities and
//!   public bindings/routes readable by everyone signed in; the auth entity
//!   only exposes the caller's own row;
//! - writes (update/delete) are constrained by `_owner_id` in SQL for
//!   non-admins and refused on the auth entity (REST additionally lets a
//!   user update their own account row, see `account_write_scope`).
//!
//! Callers turn a scope into SQL through `ReadScope::condition` /
//! `WriteScope::condition`; the column is always a kernel constant.

use serde_json::{Map, Value};

use crate::database::CronusDB;
use crate::parser::EntityNode;
use crate::sse::DataChangeEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct Viewer {
    pub id: String,
    pub role: String,
}

impl Viewer {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}

/// Request-scoped access context.
#[derive(Debug, Clone, Default)]
pub struct Access {
    pub viewer: Option<Viewer>,
    /// `auth { entity User }` — the table holding accounts.
    pub auth_entity: Option<String>,
}

impl Access {
    pub fn anonymous() -> Self {
        Self::default()
    }

    /// Session from `Authorization: Bearer` or the `cronus_token` cookie,
    /// verified with the kernel secret. Every HTTP surface builds its access
    /// context here.
    pub fn from_headers(headers: &hyper::HeaderMap, auth_entity: Option<String>) -> Self {
        Access {
            viewer: viewer_from_headers(headers, &crate::auth::default_secret()),
            auth_entity,
        }
    }

    /// Accounts table: the `auth { entity X }` entity, plus `User`/`Users`
    /// which are always treated as accounts.
    pub fn is_auth_entity(&self, entity: &str) -> bool {
        entity.eq_ignore_ascii_case("user")
            || entity.eq_ignore_ascii_case("users")
            || self
                .auth_entity
                .as_deref()
                .map_or(false, |name| name.eq_ignore_ascii_case(entity))
    }

    pub fn is_admin(&self) -> bool {
        self.viewer.as_ref().map_or(false, Viewer::is_admin)
    }

    pub fn auth_entity_name(&self) -> &str {
        self.auth_entity.as_deref().unwrap_or("User")
    }
}

/// Session token: `Authorization: Bearer` first, then the `cronus_token` cookie.
pub fn token_from_headers(headers: &hyper::HeaderMap) -> Option<String> {
    let bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|s| s.trim().to_string());
    bearer.or_else(|| {
        headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|c| {
                c.split(';').find_map(|part| {
                    part.trim()
                        .strip_prefix("cronus_token=")
                        .map(|s| s.to_string())
                })
            })
    })
}

pub fn viewer_from_headers(headers: &hyper::HeaderMap, secret: &str) -> Option<Viewer> {
    let token = token_from_headers(headers)?;
    crate::auth::verify_token(&token, secret)
        .ok()
        .map(|c| Viewer {
            id: c.sub,
            role: c.role,
        })
}

/// Row visibility for reads.
#[derive(Debug, Clone, PartialEq)]
pub enum ReadScope {
    Deny,
    All,
    /// `_owner_id = ?`
    Owner(String),
    /// auth entity: `id = ?`
    SelfRow(String),
}

/// Why a request was refused before touching data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Denial {
    /// No session → 401.
    Unauthenticated,
    /// Session present but not allowed → 403.
    Forbidden,
}

impl ReadScope {
    /// `"column" = ?` restriction, if any. Column is a kernel constant.
    pub fn condition(&self) -> Option<(&'static str, String)> {
        match self {
            ReadScope::Owner(id) => Some(("_owner_id", id.clone())),
            ReadScope::SelfRow(id) => Some(("id", id.clone())),
            _ => None,
        }
    }

    pub fn filter(&self) -> Option<(String, String, String)> {
        self.condition()
            .map(|(col, val)| (col.to_string(), "=".to_string(), val))
    }
}

pub fn read_scope(
    access: &Access,
    entity_name: &str,
    entity: Option<&EntityNode>,
    explicit_public: bool,
) -> ReadScope {
    let is_auth = access.is_auth_entity(entity_name);
    match &access.viewer {
        None if explicit_public && !is_auth => ReadScope::All,
        None => ReadScope::Deny,
        Some(v) if v.is_admin() => ReadScope::All,
        Some(v) if is_auth => ReadScope::SelfRow(v.id.clone()),
        Some(v) => {
            let shared = entity.map(|e| e.shared).unwrap_or(false);
            if shared || explicit_public {
                ReadScope::All
            } else {
                ReadScope::Owner(v.id.clone())
            }
        }
    }
}

/// REST read on a declared/auto route: `public_route` is `auth:public`.
/// Listing accounts is admin-only; a user may read only their own account.
pub fn rest_read_scope(
    access: &Access,
    entity: &EntityNode,
    public_route: bool,
    listing: bool,
) -> Result<ReadScope, Denial> {
    match read_scope(access, &entity.name, Some(entity), public_route) {
        ReadScope::Deny => Err(Denial::Unauthenticated),
        ReadScope::SelfRow(_) if listing => Err(Denial::Forbidden),
        scope => Ok(scope),
    }
}

/// Row scope for update/delete.
#[derive(Debug, Clone, PartialEq)]
pub enum WriteScope {
    Deny,
    Any,
    Owner(String),
    /// auth entity: `id = ?` (REST self-service update only)
    SelfRow(String),
}

impl WriteScope {
    /// `"column" = ?` restriction, if any. `Deny` has no condition — callers
    /// must check it first.
    pub fn condition(&self) -> Option<(&'static str, String)> {
        match self {
            WriteScope::Owner(id) => Some(("_owner_id", id.clone())),
            WriteScope::SelfRow(id) => Some(("id", id.clone())),
            _ => None,
        }
    }
}

/// Generic write surfaces (GraphQL, actions, forms): the auth entity is
/// never writable by non-admins.
pub fn write_scope(access: &Access, entity_name: &str) -> WriteScope {
    match &access.viewer {
        None => WriteScope::Deny,
        Some(v) if v.is_admin() => WriteScope::Any,
        Some(_) if access.is_auth_entity(entity_name) => WriteScope::Deny,
        Some(v) => WriteScope::Owner(v.id.clone()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WriteOp {
    Update,
    Delete,
}

/// REST writes: like `write_scope`, except a user may update (never delete)
/// their own account row. Field-level limits come from `authz::writable_body`.
pub fn account_write_scope(
    access: &Access,
    entity_name: &str,
    op: WriteOp,
) -> Result<WriteScope, Denial> {
    let viewer = access.viewer.as_ref().ok_or(Denial::Unauthenticated)?;
    match write_scope(access, entity_name) {
        WriteScope::Deny if op == WriteOp::Update => Ok(WriteScope::SelfRow(viewer.id.clone())),
        WriteScope::Deny => Err(Denial::Forbidden),
        scope => Ok(scope),
    }
}

/// Create policy for generic surfaces. `public` is the author's explicit
/// opt-in for anonymous creates. Returns the `_owner_id` to stamp (none for
/// anonymous rows and for accounts created by an admin).
pub fn create_owner(
    access: &Access,
    entity_name: &str,
    public: bool,
) -> Result<Option<String>, Denial> {
    let is_auth = access.is_auth_entity(entity_name);
    match &access.viewer {
        None if public && !is_auth => Ok(None),
        None => Err(Denial::Unauthenticated),
        Some(v) if is_auth && !v.is_admin() => Err(Denial::Forbidden),
        Some(_) if is_auth => Ok(None),
        Some(v) => Ok(Some(v.id.clone())),
    }
}

fn scope_clause(scope: &WriteScope, params: &mut Vec<String>) -> Result<String, String> {
    if *scope == WriteScope::Deny {
        return Err("write denied".into());
    }
    Ok(match scope.condition() {
        None => String::new(),
        Some((column, value)) => {
            params.push(value);
            format!(" AND \"{}\" = ?{}", column, params.len())
        }
    })
}

/// `UPDATE … WHERE id = ? [AND _owner_id = ?]`. `Ok(None)` when no row in
/// scope matched. Callers pass an already-filtered body (`authz::writable_body`).
pub fn scoped_update(
    db: &CronusDB,
    table: &str,
    id: &str,
    data: &Map<String, Value>,
    scope: &WriteScope,
) -> Result<Option<Value>, String> {
    if !crate::security::is_safe_identifier(table) {
        return Err("invalid table".into());
    }
    let mut sets = Vec::new();
    let mut params: Vec<String> = Vec::new();
    for (key, val) in data {
        if !crate::security::is_safe_identifier(key) || val.is_null() {
            continue;
        }
        params.push(match val {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        });
        sets.push(format!("\"{}\" = ?{}", key, params.len()));
    }
    if sets.is_empty() {
        return Err("nothing to update".into());
    }
    sets.push("\"updated_at\" = datetime('now')".into());
    params.push(id.to_string());
    let id_idx = params.len();
    let owner = scope_clause(scope, &mut params)?;
    let sql = format!(
        "UPDATE \"{}\" SET {} WHERE \"id\" = ?{}{} RETURNING \"id\"",
        table,
        sets.join(", "),
        id_idx,
        owner
    );
    let touched = db.query_raw_params(&sql, &params)?;
    if touched.is_empty() {
        return Ok(None);
    }
    db.find_by_id(table, id)
}

/// `DELETE … WHERE id = ? [AND _owner_id = ?]`. `Ok(false)` when no row in scope.
pub fn scoped_delete(
    db: &CronusDB,
    table: &str,
    id: &str,
    scope: &WriteScope,
) -> Result<bool, String> {
    if !crate::security::is_safe_identifier(table) {
        return Err("invalid table".into());
    }
    let mut params = vec![id.to_string()];
    let owner = scope_clause(scope, &mut params)?;
    let sql = format!(
        "DELETE FROM \"{}\" WHERE \"id\" = ?1{} RETURNING \"id\"",
        table, owner
    );
    Ok(!db.query_raw_params(&sql, &params)?.is_empty())
}

/// Exact route-pattern match: `/orders/:id` matches `/orders/42`, never
/// `/orders/42/edit`; `/admin` never matches `/admin/x`.
pub fn route_pattern_matches(pattern: &str, path: &str) -> bool {
    if pattern == path {
        return true;
    }
    if !pattern.contains(':') {
        return false;
    }
    let parts: Vec<&str> = pattern.split('/').collect();
    let req_parts: Vec<&str> = path.split('/').collect();
    parts.len() == req_parts.len()
        && parts
            .iter()
            .zip(req_parts.iter())
            .all(|(p, r)| (p.starts_with(':') && !r.is_empty()) || p == r)
}

/// Whether an SSE `data_change` event may reach this viewer. Deleted rows of
/// owner-scoped entities cannot be looked up anymore, so they only reach admins.
pub fn can_see_event(
    db: &CronusDB,
    access: &Access,
    entities: &[EntityNode],
    event: &DataChangeEvent,
) -> bool {
    let entity = match entities.iter().find(|e| e.name == event.entity) {
        Some(e) => e,
        None => {
            return access
                .viewer
                .as_ref()
                .map(|v| v.is_admin())
                .unwrap_or(false)
        }
    };
    let (column, expected) = match read_scope(access, &entity.name, Some(entity), false) {
        ReadScope::Deny => return false,
        ReadScope::All => return true,
        ReadScope::Owner(id) => ("_owner_id", id),
        ReadScope::SelfRow(id) => ("id", id),
    };
    match db.find_by_id(&entity.name, &event.id) {
        Ok(Some(row)) => row.get(column).and_then(|v| v.as_str()) == Some(expected.as_str()),
        _ => false,
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use crate::parser::{parse, AstNode};

    pub const SRC: &str = "app \"T\" { port 5175 }\n\
entity User {\n  email email!\n  role string\n  password string! sensitive\n}\n\
entity Note {\n  title string!\n  secret string sensitive\n}\n\
entity Tag shared {\n  label string!\n}\n";

    pub fn entities() -> Vec<EntityNode> {
        parse(SRC)
            .expect("parse")
            .into_iter()
            .filter_map(|n| match n {
                AstNode::Entity(e) => Some(e),
                _ => None,
            })
            .collect()
    }

    pub fn db(entities: &[EntityNode]) -> CronusDB {
        let db = CronusDB::open_memory().expect("memory db");
        db.migrate(entities).expect("migrate");
        db
    }

    pub fn as_user(id: &str) -> Access {
        Access {
            viewer: Some(Viewer {
                id: id.into(),
                role: "user".into(),
            }),
            auth_entity: Some("User".into()),
        }
    }

    pub fn as_admin() -> Access {
        Access {
            viewer: Some(Viewer {
                id: "root".into(),
                role: "admin".into(),
            }),
            auth_entity: Some("User".into()),
        }
    }

    pub fn anon() -> Access {
        Access {
            viewer: None,
            auth_entity: Some("User".into()),
        }
    }

    pub fn insert_note(db: &CronusDB, owner: &str, title: &str) -> String {
        let row = db
            .insert(
                "Note",
                &serde_json::json!({"title": title, "secret": "s3cret", "_owner_id": owner}),
            )
            .expect("insert");
        row.get("id")
            .and_then(|v| v.as_str())
            .expect("id")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;

    #[test]
    fn route_patterns_match_exactly_not_by_prefix() {
        assert!(route_pattern_matches("/admin", "/admin"));
        assert!(!route_pattern_matches("/admin", "/admin/users"));
        assert!(!route_pattern_matches("/admin", "/administrator"));
        assert!(route_pattern_matches("/orders/:id", "/orders/42"));
        assert!(!route_pattern_matches("/orders/:id", "/orders/42/edit"));
        assert!(!route_pattern_matches("/orders/:id", "/orders/"));
    }

    #[test]
    fn token_prefers_bearer_then_cookie() {
        let mut h = hyper::HeaderMap::new();
        h.insert("cookie", "a=1; cronus_token=cookie-tok".parse().unwrap());
        assert_eq!(token_from_headers(&h).as_deref(), Some("cookie-tok"));
        h.insert("authorization", "Bearer bearer-tok".parse().unwrap());
        assert_eq!(token_from_headers(&h).as_deref(), Some("bearer-tok"));
    }

    #[test]
    fn read_scope_follows_model() {
        let ents = entities();
        let note = ents.iter().find(|e| e.name == "Note");
        let tag = ents.iter().find(|e| e.name == "Tag");
        assert_eq!(read_scope(&anon(), "Note", note, false), ReadScope::Deny);
        assert_eq!(read_scope(&anon(), "Note", note, true), ReadScope::All);
        assert_eq!(read_scope(&anon(), "User", None, true), ReadScope::Deny);
        assert_eq!(
            read_scope(&as_user("u1"), "Note", note, false),
            ReadScope::Owner("u1".into())
        );
        assert_eq!(
            read_scope(&as_user("u1"), "Tag", tag, false),
            ReadScope::All
        );
        assert_eq!(
            read_scope(&as_user("u1"), "User", None, true),
            ReadScope::SelfRow("u1".into())
        );
        assert_eq!(read_scope(&as_admin(), "Note", note, false), ReadScope::All);
    }

    #[test]
    fn rest_read_scope_matches_rest_model() {
        let ents = entities();
        let note = ents.iter().find(|e| e.name == "Note").unwrap();
        let user = ents.iter().find(|e| e.name == "User").unwrap();
        assert_eq!(
            rest_read_scope(&anon(), note, false, true),
            Err(Denial::Unauthenticated)
        );
        assert_eq!(
            rest_read_scope(&anon(), note, true, true),
            Ok(ReadScope::All)
        );
        assert_eq!(
            rest_read_scope(&anon(), user, true, false),
            Err(Denial::Unauthenticated),
            "auth:public never opens the accounts table"
        );
        assert_eq!(
            rest_read_scope(&as_user("u1"), user, false, true),
            Err(Denial::Forbidden),
            "listing accounts is admin-only"
        );
        assert_eq!(
            rest_read_scope(&as_user("u1"), user, false, false)
                .unwrap()
                .condition(),
            Some(("id", "u1".to_string()))
        );
        assert_eq!(
            rest_read_scope(&as_user("u1"), note, false, false)
                .unwrap()
                .condition(),
            Some(("_owner_id", "u1".to_string()))
        );
        assert_eq!(
            rest_read_scope(&as_admin(), user, false, true),
            Ok(ReadScope::All)
        );
    }

    #[test]
    fn account_write_scope_allows_only_self_update() {
        assert_eq!(
            account_write_scope(&anon(), "Note", WriteOp::Update),
            Err(Denial::Unauthenticated)
        );
        assert_eq!(
            account_write_scope(&as_user("u1"), "User", WriteOp::Update),
            Ok(WriteScope::SelfRow("u1".into()))
        );
        assert_eq!(
            account_write_scope(&as_user("u1"), "User", WriteOp::Delete),
            Err(Denial::Forbidden)
        );
        assert_eq!(
            account_write_scope(&as_user("u1"), "Note", WriteOp::Delete),
            Ok(WriteScope::Owner("u1".into()))
        );
        assert_eq!(
            account_write_scope(&as_admin(), "User", WriteOp::Delete),
            Ok(WriteScope::Any)
        );
    }

    #[test]
    fn create_owner_follows_model() {
        assert_eq!(
            create_owner(&anon(), "Note", false),
            Err(Denial::Unauthenticated)
        );
        assert_eq!(create_owner(&anon(), "Note", true), Ok(None));
        assert_eq!(
            create_owner(&anon(), "User", true),
            Err(Denial::Unauthenticated)
        );
        assert_eq!(
            create_owner(&as_user("u1"), "Note", false),
            Ok(Some("u1".into()))
        );
        assert_eq!(
            create_owner(&as_user("u1"), "User", false),
            Err(Denial::Forbidden)
        );
        assert_eq!(create_owner(&as_admin(), "User", false), Ok(None));
    }

    #[test]
    fn user_entity_is_an_account_table_even_with_another_auth_entity() {
        let access = Access {
            viewer: None,
            auth_entity: Some("Account".into()),
        };
        assert!(access.is_auth_entity("account"));
        assert!(access.is_auth_entity("User"));
        assert!(access.is_auth_entity("users"));
        assert!(!access.is_auth_entity("Note"));
    }

    #[test]
    fn scoped_writes_refuse_other_owners_rows() {
        let ents = entities();
        let db = db(&ents);
        let id = insert_note(&db, "alice", "a");
        let mut body = Map::new();
        body.insert("title".into(), Value::String("hacked".into()));

        let bob = write_scope(&as_user("bob"), "Note");
        assert_eq!(scoped_update(&db, "Note", &id, &body, &bob).unwrap(), None);
        assert!(!scoped_delete(&db, "Note", &id, &bob).unwrap());
        let row = db.find_by_id("Note", &id).unwrap().unwrap();
        assert_eq!(row["title"], "a");

        let alice = write_scope(&as_user("alice"), "Note");
        let updated = scoped_update(&db, "Note", &id, &body, &alice)
            .unwrap()
            .unwrap();
        assert_eq!(updated["title"], "hacked");
        assert!(scoped_delete(&db, "Note", &id, &alice).unwrap());
        assert_eq!(write_scope(&anon(), "Note"), WriteScope::Deny);
        assert_eq!(write_scope(&as_user("alice"), "User"), WriteScope::Deny);
    }

    #[test]
    fn sse_events_only_reach_viewers_who_can_see_the_row() {
        let ents = entities();
        let db = db(&ents);
        let id = insert_note(&db, "alice", "a");
        let ev = DataChangeEvent {
            entity: "Note".into(),
            action: "created".into(),
            id: id.clone(),
        };
        assert!(can_see_event(&db, &as_user("alice"), &ents, &ev));
        assert!(!can_see_event(&db, &as_user("bob"), &ents, &ev));
        assert!(!can_see_event(&db, &anon(), &ents, &ev));
        assert!(can_see_event(&db, &as_admin(), &ents, &ev));
        let tag = DataChangeEvent {
            entity: "Tag".into(),
            action: "deleted".into(),
            id: "gone".into(),
        };
        assert!(can_see_event(&db, &as_user("bob"), &ents, &tag));
        assert!(!can_see_event(&db, &anon(), &ents, &tag));
    }
}
