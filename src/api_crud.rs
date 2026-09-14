//! REST CRUD for `/api/<entity>` — the live handler behind
//! `main.rs::handle_request_inner`.
//!
//! Sprint 1 security semantics (documented in LANGUAGE.md §4):
//! - the entity is resolved by the exact first path segment (`/notes`, not
//!   `/notesarchive`);
//! - when the app declares an `api` block for the entity, only declared
//!   method + path shapes are served and each route's `auth:` is enforced;
//!   without a block, auto CRUD requires an authenticated user;
//! - owner scope lives in the SQL `WHERE` of every SELECT/UPDATE/DELETE;
//! - writes go through `authz::writable_body`, responses through
//!   `authz::redact_sensitive`, errors through `authz::error_body`.

use crate::auth::Claims;
use crate::authz;
use crate::parser::{EntityNode, HttpMethod, RouteNode};
use crate::server::response::json_response;
use crate::server::state::AppState;
use bytes::Bytes;
use http_body_util::Full;
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Method, Response, StatusCode};
use serde_json::{json, Map, Value};

type ApiResponse = Response<Full<Bytes>>;

/// `(column, value)` pairs rendered as `"column" = ?` in a WHERE clause.
/// Columns are kernel constants (`id`, `_owner_id`), never client input.
type Scope = Vec<(&'static str, String)>;

const DEFAULT_LIMIT: usize = 100;
const MAX_LIMIT: usize = 1000;

/// Field names `CronusDB::migrate` never turns into columns.
const NON_COLUMN_FIELDS: &[&str] = &[
    "id", "createdat", "created_at", "updatedat", "updated_at", "string", "integer", "text",
    "real", "blob", "null", "primary", "table", "index", "select", "from", "where",
];

/// Reads the session from `Authorization: Bearer` or the `cronus_token`
/// cookie (bearer wins when present) and verifies it.
pub(crate) fn claims_from_headers(authorization: Option<&str>, cookie: &str, secret: &str) -> Option<Claims> {
    let bearer = authorization.and_then(|h| h.strip_prefix("Bearer ")).map(str::to_string);
    let cookie_token = cookie
        .split(';')
        .find_map(|c| c.trim().strip_prefix("cronus_token=").map(str::to_string));
    let token = bearer.or(cookie_token)?;
    crate::auth::verify_token(&token, secret).ok()
}

enum Operation<'a> {
    List,
    Detail(&'a str),
    Create,
    Update(&'a str),
    Delete(&'a str),
}

/// Handles `/api/<entity>[/<id>]`. `path` excludes the query string.
pub(crate) fn handle_api(
    state: &AppState,
    method: &Method,
    path: &str,
    query: &str,
    body: Option<&Value>,
    claims: Option<&Claims>,
) -> ApiResponse {
    let segments: Vec<&str> = path
        .strip_prefix("/api")
        .unwrap_or(path)
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let Some((first, rest)) = segments.split_first() else {
        return not_found();
    };
    let Some(entity) = resolve_entity(&state.entities, first) else {
        return not_found();
    };
    let Some(op) = operation(method, rest) else {
        return not_found();
    };

    let declared: Vec<&RouteNode> = state
        .apis
        .iter()
        .filter(|api| {
            let prefix = api.prefix.strip_prefix("/api").unwrap_or(&api.prefix).trim_matches('/');
            !prefix.is_empty() && !prefix.contains('/') && entity_matches_segment(&entity.name, prefix)
        })
        .flat_map(|api| api.routes.iter())
        .collect();

    let route = if declared.is_empty() {
        None
    } else {
        match declared.into_iter().find(|r| route_matches(r, method, rest)) {
            Some(r) => Some(r),
            None => return not_found(),
        }
    };

    let public = match authorize(route, claims) {
        Ok(public) => public,
        Err(resp) => return resp,
    };

    let user_entity = is_user_entity(state, entity);
    let admin = claims.map_or(false, |c| c.role == "admin");

    match op {
        Operation::List => {
            let scope = match read_scope(entity, user_entity, public, admin, claims, true) {
                Ok(s) => s,
                Err(resp) => return resp,
            };
            list(state, entity, &scope, query)
        }
        Operation::Detail(id) => {
            let scope = match read_scope(entity, user_entity, public, admin, claims, false) {
                Ok(s) => s,
                Err(resp) => return resp,
            };
            match select_one(state, entity, id, &scope) {
                Ok(Some(mut row)) => {
                    authz::redact_sensitive(entity, &mut row);
                    json_response(StatusCode::OK, row)
                }
                Ok(None) => not_found(),
                Err(detail) => internal("detail", &entity.name, &detail),
            }
        }
        Operation::Create => {
            if user_entity {
                // Accounts are created only through /api/auth/signup.
                return match claims {
                    None => unauthorized(),
                    Some(_) => forbidden(),
                };
            }
            create(state, entity, body, claims)
        }
        Operation::Update(id) => {
            let scope = match write_scope(user_entity, admin, claims, false) {
                Ok(s) => s,
                Err(resp) => return resp,
            };
            update(state, entity, id, &scope, body, claims)
        }
        Operation::Delete(id) => {
            let scope = match write_scope(user_entity, admin, claims, true) {
                Ok(s) => s,
                Err(resp) => return resp,
            };
            delete(state, entity, id, &scope, claims)
        }
    }
}

// ── Resolution ──────────────────────────────────────────────

/// `Note` matches `note`, `notes`; `Category` matches `categories`;
/// `BlogPost` matches `blog-posts`. Never a prefix match.
pub(crate) fn entity_matches_segment(entity_name: &str, segment: &str) -> bool {
    let name = entity_name.to_lowercase();
    let seg: String = segment.to_lowercase().chars().filter(|c| *c != '-' && *c != '_').collect();
    if seg == name || seg == format!("{name}s") || seg == format!("{name}es") {
        return true;
    }
    name.strip_suffix('y').map_or(false, |stem| seg == format!("{stem}ies"))
}

fn resolve_entity<'a>(entities: &'a [EntityNode], segment: &str) -> Option<&'a EntityNode> {
    entities
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case(segment))
        .or_else(|| entities.iter().find(|e| entity_matches_segment(&e.name, segment)))
}

fn operation<'a>(method: &Method, rest: &[&'a str]) -> Option<Operation<'a>> {
    match (method, rest) {
        (&Method::GET, []) => Some(Operation::List),
        (&Method::GET, [id]) => Some(Operation::Detail(id)),
        (&Method::POST, []) => Some(Operation::Create),
        (&Method::PATCH, [id]) | (&Method::PUT, [id]) => Some(Operation::Update(id)),
        (&Method::DELETE, [id]) => Some(Operation::Delete(id)),
        _ => None,
    }
}

fn route_matches(route: &RouteNode, method: &Method, rest: &[&str]) -> bool {
    let route_method = match route.method {
        HttpMethod::GET => Method::GET,
        HttpMethod::POST => Method::POST,
        HttpMethod::PATCH => Method::PATCH,
        HttpMethod::PUT => Method::PUT,
        HttpMethod::DELETE => Method::DELETE,
    };
    if route_method != *method {
        return false;
    }
    let parts: Vec<&str> = route.path.split('/').filter(|s| !s.is_empty()).collect();
    parts.len() == rest.len() && parts.iter().zip(rest).all(|(p, r)| p.starts_with(':') || p == r)
}

fn is_user_entity(state: &AppState, entity: &EntityNode) -> bool {
    let lower = entity.name.to_lowercase();
    lower == "user"
        || lower == "users"
        || state.auth_entity.as_deref().map_or(false, |a| a.eq_ignore_ascii_case(&entity.name))
}

// ── Authorization ───────────────────────────────────────────

/// Returns `Ok(true)` when the route is public. Auto CRUD (`route == None`)
/// and any `auth:` value other than `public` require a valid session;
/// `auth:admin`, `auth:role(a|b)` and `[roles]` also require the role
/// (admin always qualifies).
fn authorize(route: Option<&RouteNode>, claims: Option<&Claims>) -> Result<bool, ApiResponse> {
    let Some(route) = route else {
        return claims.map(|_| false).ok_or_else(unauthorized);
    };
    let auth = route.auth.trim();
    let mut roles: Vec<String> = route.roles.clone();
    if auth == "admin" {
        roles.push("admin".to_string());
    } else if let Some(inner) = auth.strip_prefix("role(").and_then(|s| s.strip_suffix(')')) {
        roles.extend(inner.split(|c| c == '|' || c == ',').map(|r| r.trim().to_string()).filter(|r| !r.is_empty()));
    }
    if auth == "public" && roles.is_empty() {
        return Ok(true);
    }
    let claims = claims.ok_or_else(unauthorized)?;
    if !roles.is_empty() && claims.role != "admin" && !roles.iter().any(|r| *r == claims.role) {
        return Err(forbidden());
    }
    Ok(false)
}

fn read_scope(
    entity: &EntityNode,
    user_entity: bool,
    public: bool,
    admin: bool,
    claims: Option<&Claims>,
    listing: bool,
) -> Result<Scope, ApiResponse> {
    if user_entity {
        let claims = claims.ok_or_else(unauthorized)?;
        return match (admin, listing) {
            (true, _) => Ok(vec![]),
            (false, true) => Err(forbidden()),
            (false, false) => Ok(vec![("id", claims.sub.clone())]),
        };
    }
    if admin || public {
        return Ok(vec![]);
    }
    let claims = claims.ok_or_else(unauthorized)?;
    if entity.shared {
        return Ok(vec![]);
    }
    Ok(vec![("_owner_id", claims.sub.clone())])
}

fn write_scope(user_entity: bool, admin: bool, claims: Option<&Claims>, deleting: bool) -> Result<Scope, ApiResponse> {
    let claims = claims.ok_or_else(unauthorized)?;
    if admin {
        return Ok(vec![]);
    }
    if user_entity {
        return if deleting { Err(forbidden()) } else { Ok(vec![("id", claims.sub.clone())]) };
    }
    Ok(vec![("_owner_id", claims.sub.clone())])
}

// ── Queries ─────────────────────────────────────────────────

fn scope_conditions(scope: &Scope, conditions: &mut Vec<String>, params: &mut Vec<String>) {
    for (column, value) in scope {
        conditions.push(format!("\"{column}\" = ?"));
        params.push(value.clone());
    }
}

fn where_clause(conditions: &[String]) -> String {
    if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    }
}

fn select_one(state: &AppState, entity: &EntityNode, id: &str, scope: &Scope) -> Result<Option<Value>, String> {
    let mut conditions = vec!["\"id\" = ?".to_string()];
    let mut params = vec![id.to_string()];
    scope_conditions(scope, &mut conditions, &mut params);
    let sql = format!("SELECT * FROM \"{}\"{} LIMIT 1", entity.name, where_clause(&conditions));
    Ok(state.db.query_raw_params(&sql, &params)?.into_iter().next())
}

fn list(state: &AppState, entity: &EntityNode, scope: &Scope, query: &str) -> ApiResponse {
    let params_in: Vec<(String, String)> = query
        .split('&')
        .filter_map(|p| p.split_once('='))
        .map(|(k, v)| (decode_component(k), decode_component(v)))
        .collect();
    let param = |name: &str| params_in.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str());

    let limit = param("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT);
    let offset = param("offset").and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
    let search = param("search").or_else(|| param("q")).filter(|s| !s.is_empty());

    let mut conditions = Vec::new();
    let mut params = Vec::new();
    scope_conditions(scope, &mut conditions, &mut params);
    if let Some(term) = search {
        let columns = searchable_columns(entity);
        if columns.is_empty() {
            conditions.push("0".to_string());
        } else {
            let pattern = format!("%{}%", escape_like(term));
            let ors: Vec<String> = columns.iter().map(|c| format!("\"{c}\" LIKE ? ESCAPE '\\'")).collect();
            conditions.push(format!("({})", ors.join(" OR ")));
            params.extend(columns.iter().map(|_| pattern.clone()));
        }
    }
    let filter = where_clause(&conditions);

    let count_sql = format!("SELECT COUNT(*) AS total FROM \"{}\"{}", entity.name, filter);
    let total = match state.db.query_raw_params(&count_sql, &params) {
        Ok(rows) => rows.first().and_then(|r| r.get("total")).and_then(Value::as_i64).unwrap_or(0),
        Err(detail) => return internal("count", &entity.name, &detail),
    };

    let rows_sql = format!("SELECT * FROM \"{}\"{} ORDER BY rowid DESC LIMIT ? OFFSET ?", entity.name, filter);
    let mut row_params = params;
    row_params.push(limit.to_string());
    row_params.push(offset.to_string());
    let mut rows = match state.db.query_raw_params(&rows_sql, &row_params) {
        Ok(rows) => Value::Array(rows),
        Err(detail) => return internal("list", &entity.name, &detail),
    };
    authz::redact_sensitive(entity, &mut rows);

    let mut resp = json_response(StatusCode::OK, rows);
    let headers = resp.headers_mut();
    headers.insert(
        HeaderName::from_static("access-control-expose-headers"),
        HeaderValue::from_static("X-Total-Count, X-Limit, X-Offset"),
    );
    headers.insert(HeaderName::from_static("x-total-count"), HeaderValue::from(total.max(0) as u64));
    headers.insert(HeaderName::from_static("x-limit"), HeaderValue::from(limit as u64));
    headers.insert(HeaderName::from_static("x-offset"), HeaderValue::from(offset as u64));
    resp
}

/// Declared, persisted, non-sensitive, non-privileged columns. Sensitive
/// columns are excluded so `?search=` can't be used as an oracle on them.
fn searchable_columns(entity: &EntityNode) -> Vec<&str> {
    entity
        .fields
        .iter()
        .filter(|f| !f.sensitive && !authz::PRIVILEGED_FIELDS.contains(&f.name.as_str()) && is_column(&f.name))
        .map(|f| f.name.as_str())
        .collect()
}

fn is_column(name: &str) -> bool {
    !NON_COLUMN_FIELDS.contains(&name.to_lowercase().as_str()) && crate::security::is_safe_identifier(name)
}

fn escape_like(term: &str) -> String {
    term.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

fn decode_component(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' => {
                let decoded = bytes
                    .get(i + 1..i + 3)
                    .and_then(|hex| std::str::from_utf8(hex).ok())
                    .and_then(|hex| u8::from_str_radix(hex, 16).ok());
                match decoded {
                    Some(b) => {
                        out.push(b);
                        i += 3;
                    }
                    None => {
                        out.push(b'%');
                        i += 1;
                    }
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn sql_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Client-writable, persisted columns with non-null values.
fn writable_columns(entity: &EntityNode, body: &Map<String, Value>) -> Map<String, Value> {
    authz::writable_body(entity, body)
        .into_iter()
        .filter(|(k, v)| is_column(k) && !v.is_null())
        .collect()
}

// ── Writes ──────────────────────────────────────────────────

fn create(state: &AppState, entity: &EntityNode, body: Option<&Value>, claims: Option<&Claims>) -> ApiResponse {
    let Some(obj) = body.and_then(Value::as_object) else {
        return validation_failed(StatusCode::BAD_REQUEST, "Expected a JSON object body");
    };
    let mut data = writable_columns(entity, obj);
    for field in &entity.fields {
        if let Some(default) = &field.default_value {
            if is_column(&field.name) && !data.contains_key(&field.name) {
                data.insert(field.name.clone(), Value::String(default.clone()));
            }
        }
    }
    if let Some(c) = claims {
        data.insert("_owner_id".to_string(), json!(c.sub));
    }
    let data = Value::Object(data);

    if let Err(message) = state.db.validate(entity, &data) {
        return validation_failed(StatusCode::BAD_REQUEST, &message);
    }
    if let Err(message) = state.db.check_unique(entity, &data) {
        return validation_failed(StatusCode::CONFLICT, &message);
    }
    let row = match state.db.insert(&entity.name, &data) {
        Ok(row) => row,
        Err(detail) => return write_error("create", &entity.name, &detail),
    };

    let table = entity.name.as_str();
    let owner = claims.map(|c| c.sub.as_str()).unwrap_or("");
    let row_id = row.get("id").and_then(Value::as_str).unwrap_or("").to_string();
    crate::fire_webhooks(&state.webhooks, table, "create", &row);
    crate::fire_effects(entity, "create", &row, None, &state.brain, &state.sse_hub);
    crate::scripting::fire_scripts(&state.script_registry, table, "create", &row, &row_id, None, &state.db, owner, "user", &std::collections::HashMap::new());
    let mut row = row;
    authz::redact_sensitive(entity, &mut row);
    if let Err(e) = state.audit_trail.log("INSERT", table, &row_id, owner, &row, None) {
        eprintln!("  \x1b[33m⚠\x1b[0m Audit log failed (INSERT {}:{}): {}", table, row_id, e);
    }
    state.sse_hub.broadcast(crate::sse::DataChangeEvent { entity: table.to_string(), action: "created".to_string(), id: row_id });

    json_response(StatusCode::CREATED, row)
}

fn update(
    state: &AppState,
    entity: &EntityNode,
    id: &str,
    scope: &Scope,
    body: Option<&Value>,
    claims: Option<&Claims>,
) -> ApiResponse {
    let Some(obj) = body.and_then(Value::as_object) else {
        return validation_failed(StatusCode::BAD_REQUEST, "Expected a JSON object body");
    };
    let data = writable_columns(entity, obj);

    let prev = match select_one(state, entity, id, scope) {
        Ok(Some(row)) => row,
        Ok(None) => return not_found(),
        Err(detail) => return internal("update lookup", &entity.name, &detail),
    };

    let data_value = Value::Object(data.clone());
    let mut partial = entity.clone();
    partial.fields.retain(|f| data.contains_key(&f.name));
    if let Err(message) = state.db.validate(&partial, &data_value) {
        return validation_failed(StatusCode::BAD_REQUEST, &message);
    }
    if !entity.transitions.is_empty() {
        if let Err(err_body) = crate::validate_transitions(entity, &data_value, &prev) {
            return json_response(StatusCode::CONFLICT, err_body);
        }
    }

    if !data.is_empty() {
        let mut sets: Vec<String> = data.keys().map(|k| format!("\"{k}\" = ?")).collect();
        let has_declared_updated_at = entity
            .fields
            .iter()
            .any(|f| matches!(f.name.to_lowercase().as_str(), "updatedat" | "updated_at"));
        if !has_declared_updated_at {
            sets.push("\"updated_at\" = datetime('now')".to_string());
        }
        let mut conditions = vec!["\"id\" = ?".to_string()];
        let mut params: Vec<String> = data.values().map(sql_text).collect();
        params.push(id.to_string());
        scope_conditions(scope, &mut conditions, &mut params);
        let sql = format!("UPDATE \"{}\" SET {}{}", entity.name, sets.join(", "), where_clause(&conditions));
        match state.db.transaction(|conn| {
            conn.execute(&sql, rusqlite::params_from_iter(params.iter())).map_err(|e| e.to_string())
        }) {
            Ok(0) => return not_found(),
            Ok(_) => {}
            Err(detail) => return write_error("update", &entity.name, &detail),
        }
    }

    let row = match select_one(state, entity, id, scope) {
        Ok(Some(row)) => row,
        Ok(None) => return not_found(),
        Err(detail) => return internal("update reload", &entity.name, &detail),
    };

    if !data.is_empty() {
        let table = entity.name.as_str();
        let owner = claims.map(|c| c.sub.as_str()).unwrap_or("");
        crate::fire_webhooks(&state.webhooks, table, "update", &row);
        crate::fire_effects(entity, "update", &row, Some(&prev), &state.brain, &state.sse_hub);
        crate::scripting::fire_scripts(&state.script_registry, table, "update", &row, id, Some(&prev), &state.db, owner, "user", &std::collections::HashMap::new());
        let (mut logged_row, mut logged_prev) = (row.clone(), prev.clone());
        authz::redact_sensitive(entity, &mut logged_row);
        authz::redact_sensitive(entity, &mut logged_prev);
        if let Err(e) = state.audit_trail.log("UPDATE", table, id, owner, &logged_row, Some(&logged_prev)) {
            eprintln!("  \x1b[33m⚠\x1b[0m Audit log failed (UPDATE {}:{}): {}", table, id, e);
        }
        state.sse_hub.broadcast(crate::sse::DataChangeEvent { entity: table.to_string(), action: "updated".to_string(), id: id.to_string() });
    }

    let mut row = row;
    authz::redact_sensitive(entity, &mut row);
    json_response(StatusCode::OK, row)
}

fn delete(state: &AppState, entity: &EntityNode, id: &str, scope: &Scope, claims: Option<&Claims>) -> ApiResponse {
    let prev = match select_one(state, entity, id, scope) {
        Ok(Some(row)) => row,
        Ok(None) => return not_found(),
        Err(detail) => return internal("delete lookup", &entity.name, &detail),
    };

    let mut conditions = vec!["\"id\" = ?".to_string()];
    let mut params = vec![id.to_string()];
    scope_conditions(scope, &mut conditions, &mut params);
    let sql = format!("DELETE FROM \"{}\"{}", entity.name, where_clause(&conditions));
    match state.db.transaction(|conn| {
        conn.execute(&sql, rusqlite::params_from_iter(params.iter())).map_err(|e| e.to_string())
    }) {
        Ok(0) => return not_found(),
        Ok(_) => {}
        Err(detail) => return internal("delete", &entity.name, &detail),
    }

    let table = entity.name.as_str();
    let owner = claims.map(|c| c.sub.as_str()).unwrap_or("");
    let payload = json!({"id": id, "entity": table});
    crate::fire_webhooks(&state.webhooks, table, "delete", &payload);
    crate::fire_effects(entity, "delete", &prev, None, &state.brain, &state.sse_hub);
    crate::scripting::fire_scripts(&state.script_registry, table, "delete", &prev, id, None, &state.db, owner, "user", &std::collections::HashMap::new());
    let mut logged_prev = prev.clone();
    authz::redact_sensitive(entity, &mut logged_prev);
    if let Err(e) = state.audit_trail.log("DELETE", table, id, owner, &json!({"id": id}), Some(&logged_prev)) {
        eprintln!("  \x1b[33m⚠\x1b[0m Audit log failed (DELETE {}:{}): {}", table, id, e);
    }
    state.sse_hub.broadcast(crate::sse::DataChangeEvent { entity: table.to_string(), action: "deleted".to_string(), id: id.to_string() });

    json_response(StatusCode::OK, json!({"deleted": id}))
}

// ── Errors ──────────────────────────────────────────────────

fn error(status: StatusCode, code: &str, message: &str) -> ApiResponse {
    json_response(status, authz::error_body(code, message))
}

fn not_found() -> ApiResponse {
    error(StatusCode::NOT_FOUND, "NOT_FOUND", "Not found")
}

fn unauthorized() -> ApiResponse {
    error(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "Authentication required")
}

fn forbidden() -> ApiResponse {
    error(StatusCode::FORBIDDEN, "FORBIDDEN", "Not allowed")
}

/// `message` must come from entity validation (field names/rules), never
/// from the database driver.
fn validation_failed(status: StatusCode, message: &str) -> ApiResponse {
    error(status, "VALIDATION_FAILED", message)
}

fn internal(action: &str, entity: &str, detail: &str) -> ApiResponse {
    eprintln!("  \x1b[31m✗\x1b[0m api {} {} failed: {}", action, entity, detail);
    error(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL", "Internal error")
}

/// Unique-constraint races surface as a safe 409; anything else is logged.
fn write_error(action: &str, entity: &str, detail: &str) -> ApiResponse {
    if detail.contains("UNIQUE constraint failed") {
        return validation_failed(StatusCode::CONFLICT, "A record with this value already exists");
    }
    internal(action, entity, detail)
}
