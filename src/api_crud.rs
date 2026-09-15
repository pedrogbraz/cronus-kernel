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
//!   who may see/touch which rows is decided by `access.rs` (shared with
//!   GraphQL, bindings, forms/actions and SSE);
//! - writes go through `authz::writable_body`, responses through
//!   `authz::redact_sensitive`, errors through `authz::error_body`;
//! - field rules come from `validation.rs` (`422` with `error.fields`,
//!   `409` for `unique`); many-to-many fields (`tags -> Tag[]`) are written
//!   and read through `relations.rs`.

use crate::access::{self, Access, Denial, WriteOp};
use crate::authz;
use crate::parser::{EntityNode, HttpMethod, RouteNode};
use crate::relations;
use crate::server::response::json_response;
use crate::server::state::AppState;
use crate::validation::{self, FieldErrors, Mode};
use bytes::Bytes;
use http_body_util::Full;
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Method, Response, StatusCode};
use serde_json::{json, Map, Value};

type ApiResponse = Response<Full<Bytes>>;

/// Row restriction from `access` rendered as `"column" = ?` in a WHERE
/// clause. The column is a kernel constant (`id`, `_owner_id`).
type Scope = Option<(&'static str, String)>;

const DEFAULT_LIMIT: usize = 100;
const MAX_LIMIT: usize = 1000;

/// Field names `CronusDB::migrate` never turns into columns.
const NON_COLUMN_FIELDS: &[&str] = &[
    "id",
    "createdat",
    "created_at",
    "updatedat",
    "updated_at",
    "string",
    "integer",
    "text",
    "real",
    "blob",
    "null",
    "primary",
    "table",
    "index",
    "select",
    "from",
    "where",
];

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
    access: &Access,
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
            let prefix = api
                .prefix
                .strip_prefix("/api")
                .unwrap_or(&api.prefix)
                .trim_matches('/');
            !prefix.is_empty()
                && !prefix.contains('/')
                && entity_matches_segment(&entity.name, prefix)
        })
        .flat_map(|api| api.routes.iter())
        .collect();

    let route = if declared.is_empty() {
        None
    } else {
        match declared
            .into_iter()
            .find(|r| route_matches(r, method, rest))
        {
            Some(r) => Some(r),
            None => return not_found(),
        }
    };

    let public = match authorize(route, access) {
        Ok(public) => public,
        Err(resp) => return resp,
    };

    match op {
        Operation::List => {
            let scope = match access::rest_read_scope(access, entity, public, true) {
                Ok(s) => s.condition(),
                Err(d) => return denied(d),
            };
            list(state, entity, &scope, query, access)
        }
        Operation::Detail(id) => {
            let scope = match access::rest_read_scope(access, entity, public, false) {
                Ok(s) => s.condition(),
                Err(d) => return denied(d),
            };
            match select_one(state, entity, id, &scope) {
                Ok(Some(mut row)) => {
                    authz::redact_sensitive(entity, &mut row);
                    let pairs = query_pairs(query);
                    let expand = relations::expand_param(query_param(&pairs, "expand"));
                    relations::attach(
                        &state.db,
                        &state.entities,
                        entity,
                        &mut row,
                        access,
                        &expand,
                    );
                    json_response(StatusCode::OK, row)
                }
                Ok(None) => not_found(),
                Err(detail) => internal("detail", &entity.name, &detail),
            }
        }
        Operation::Create => {
            if access.is_auth_entity(&entity.name) {
                // Accounts are created only through /api/auth/signup.
                return match access.viewer {
                    None => unauthorized(),
                    Some(_) => forbidden(),
                };
            }
            // `authorize` already required a session unless the route is public.
            let owner = match access::create_owner(access, &entity.name, true) {
                Ok(owner) => owner,
                Err(d) => return denied(d),
            };
            create(state, entity, body, owner.as_deref(), access)
        }
        Operation::Update(id) => {
            let scope = match access::account_write_scope(access, &entity.name, WriteOp::Update) {
                Ok(s) => s.condition(),
                Err(d) => return denied(d),
            };
            update(state, entity, id, &scope, body, access)
        }
        Operation::Delete(id) => {
            let scope = match access::account_write_scope(access, &entity.name, WriteOp::Delete) {
                Ok(s) => s.condition(),
                Err(d) => return denied(d),
            };
            delete(state, entity, id, &scope, viewer_id(access))
        }
    }
}

fn viewer_id(access: &Access) -> &str {
    access.viewer.as_ref().map_or("", |v| v.id.as_str())
}

// ── Resolution ──────────────────────────────────────────────

/// `Note` matches `note`, `notes`; `Category` matches `categories`;
/// `BlogPost` matches `blog-posts`. Never a prefix match.
pub(crate) fn entity_matches_segment(entity_name: &str, segment: &str) -> bool {
    let name = entity_name.to_lowercase();
    let seg: String = segment
        .to_lowercase()
        .chars()
        .filter(|c| *c != '-' && *c != '_')
        .collect();
    if seg == name || seg == format!("{name}s") || seg == format!("{name}es") {
        return true;
    }
    name.strip_suffix('y')
        .map_or(false, |stem| seg == format!("{stem}ies"))
}

fn resolve_entity<'a>(entities: &'a [EntityNode], segment: &str) -> Option<&'a EntityNode> {
    entities
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case(segment))
        .or_else(|| {
            entities
                .iter()
                .find(|e| entity_matches_segment(&e.name, segment))
        })
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
    parts.len() == rest.len()
        && parts
            .iter()
            .zip(rest)
            .all(|(p, r)| p.starts_with(':') || p == r)
}

// ── Authorization ───────────────────────────────────────────

/// Route-level gate (REST only; row scope comes from `access.rs`).
/// Returns `Ok(true)` when the route is public. Auto CRUD (`route == None`)
/// and any `auth:` value other than `public` require a valid session;
/// `auth:admin`, `auth:role(a|b)` and `[roles]` also require the role
/// (admin always qualifies).
fn authorize(route: Option<&RouteNode>, access: &Access) -> Result<bool, ApiResponse> {
    let Some(route) = route else {
        return access
            .viewer
            .as_ref()
            .map(|_| false)
            .ok_or_else(unauthorized);
    };
    let auth = route.auth.trim();
    let mut roles: Vec<String> = route.roles.clone();
    if auth == "admin" {
        roles.push("admin".to_string());
    } else if let Some(inner) = auth.strip_prefix("role(").and_then(|s| s.strip_suffix(')')) {
        roles.extend(
            inner
                .split(|c| c == '|' || c == ',')
                .map(|r| r.trim().to_string())
                .filter(|r| !r.is_empty()),
        );
    }
    if auth == "public" && roles.is_empty() {
        return Ok(true);
    }
    let viewer = access.viewer.as_ref().ok_or_else(unauthorized)?;
    if !roles.is_empty() && !viewer.is_admin() && !roles.iter().any(|r| *r == viewer.role) {
        return Err(forbidden());
    }
    Ok(false)
}

// ── Queries ─────────────────────────────────────────────────

fn scope_conditions(scope: &Scope, conditions: &mut Vec<String>, params: &mut Vec<String>) {
    if let Some((column, value)) = scope {
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

fn select_one(
    state: &AppState,
    entity: &EntityNode,
    id: &str,
    scope: &Scope,
) -> Result<Option<Value>, String> {
    let mut conditions = vec!["\"id\" = ?".to_string()];
    let mut params = vec![id.to_string()];
    scope_conditions(scope, &mut conditions, &mut params);
    let sql = format!(
        "SELECT * FROM \"{}\"{} LIMIT 1",
        entity.name,
        where_clause(&conditions)
    );
    Ok(state.db.query_raw_params(&sql, &params)?.into_iter().next())
}

fn query_pairs(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter_map(|p| p.split_once('='))
        .map(|(k, v)| (decode_component(k), decode_component(v)))
        .collect()
}

fn query_param<'a>(pairs: &'a [(String, String)], name: &str) -> Option<&'a str> {
    pairs
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
}

fn list(
    state: &AppState,
    entity: &EntityNode,
    scope: &Scope,
    query: &str,
    access: &Access,
) -> ApiResponse {
    let params_in = query_pairs(query);
    let param = |name: &str| query_param(&params_in, name);

    let limit = param("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT);
    let offset = param("offset")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    let search = param("search")
        .or_else(|| param("q"))
        .filter(|s| !s.is_empty());

    let mut conditions = Vec::new();
    let mut params = Vec::new();
    scope_conditions(scope, &mut conditions, &mut params);
    if let Some(term) = search {
        let columns = searchable_columns(entity);
        if columns.is_empty() {
            conditions.push("0".to_string());
        } else {
            let pattern = format!("%{}%", escape_like(term));
            let ors: Vec<String> = columns
                .iter()
                .map(|c| format!("\"{c}\" LIKE ? ESCAPE '\\'"))
                .collect();
            conditions.push(format!("({})", ors.join(" OR ")));
            params.extend(columns.iter().map(|_| pattern.clone()));
        }
    }
    let filter = where_clause(&conditions);

    let count_sql = format!(
        "SELECT COUNT(*) AS total FROM \"{}\"{}",
        entity.name, filter
    );
    let total = match state.db.query_raw_params(&count_sql, &params) {
        Ok(rows) => rows
            .first()
            .and_then(|r| r.get("total"))
            .and_then(Value::as_i64)
            .unwrap_or(0),
        Err(detail) => return internal("count", &entity.name, &detail),
    };

    let rows_sql = format!(
        "SELECT * FROM \"{}\"{} ORDER BY rowid DESC LIMIT ? OFFSET ?",
        entity.name, filter
    );
    let mut row_params = params;
    row_params.push(limit.to_string());
    row_params.push(offset.to_string());
    let mut rows = match state.db.query_raw_params(&rows_sql, &row_params) {
        Ok(rows) => Value::Array(rows),
        Err(detail) => return internal("list", &entity.name, &detail),
    };
    authz::redact_sensitive(entity, &mut rows);
    let expand = relations::expand_param(param("expand"));
    relations::attach(
        &state.db,
        &state.entities,
        entity,
        &mut rows,
        access,
        &expand,
    );

    let mut resp = json_response(StatusCode::OK, rows);
    let headers = resp.headers_mut();
    headers.insert(
        HeaderName::from_static("access-control-expose-headers"),
        HeaderValue::from_static("X-Total-Count, X-Limit, X-Offset"),
    );
    headers.insert(
        HeaderName::from_static("x-total-count"),
        HeaderValue::from(total.max(0) as u64),
    );
    headers.insert(
        HeaderName::from_static("x-limit"),
        HeaderValue::from(limit as u64),
    );
    headers.insert(
        HeaderName::from_static("x-offset"),
        HeaderValue::from(offset as u64),
    );
    resp
}

/// Declared, persisted, non-sensitive, non-privileged columns. Sensitive
/// columns are excluded so `?search=` can't be used as an oracle on them.
fn searchable_columns(entity: &EntityNode) -> Vec<&str> {
    entity
        .fields
        .iter()
        .filter(|f| {
            !f.sensitive
                && !authz::PRIVILEGED_FIELDS.contains(&f.name.as_str())
                && is_column(&f.name)
        })
        .map(|f| f.name.as_str())
        .collect()
}

fn is_column(name: &str) -> bool {
    !NON_COLUMN_FIELDS.contains(&name.to_lowercase().as_str())
        && crate::security::is_safe_identifier(name)
}

fn escape_like(term: &str) -> String {
    term.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
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

/// Client-writable body split into persisted non-null columns and
/// many-to-many values (`tags -> Tag[]`).
fn writable_parts(
    entity: &EntityNode,
    body: &Map<String, Value>,
) -> (Map<String, Value>, Map<String, Value>) {
    let (columns, many) = relations::split(entity, authz::writable_body(entity, body));
    let columns = columns
        .into_iter()
        .filter(|(k, v)| is_column(k) && !v.is_null())
        .collect();
    (columns, many)
}

/// Field rules, then relation ids (exist + readable by the caller), then
/// uniqueness. `Err` is the ready response (`422`, or `409` for `unique`).
fn check_write(
    state: &AppState,
    entity: &EntityNode,
    columns: &Map<String, Value>,
    many: &Map<String, Value>,
    mode: Mode,
    exclude_id: Option<&str>,
    access: &Access,
) -> Result<Vec<relations::LinkSet>, ApiResponse> {
    validation::check_write(
        &state.db,
        &state.entities,
        entity,
        columns,
        many,
        mode,
        exclude_id,
        access,
    )
    .map_err(|(status, errors)| {
        let status = StatusCode::from_u16(status).unwrap_or(StatusCode::UNPROCESSABLE_ENTITY);
        invalid_fields(status, &errors)
    })
}

// ── Writes ──────────────────────────────────────────────────

fn create(
    state: &AppState,
    entity: &EntityNode,
    body: Option<&Value>,
    owner: Option<&str>,
    access: &Access,
) -> ApiResponse {
    let Some(obj) = body.and_then(Value::as_object) else {
        return validation_failed(StatusCode::BAD_REQUEST, "Expected a JSON object body");
    };
    let (mut data, many) = writable_parts(entity, obj);
    for field in &entity.fields {
        if let Some(default) = &field.default_value {
            if is_column(&field.name) && !field.is_many() && !data.contains_key(&field.name) {
                data.insert(field.name.clone(), Value::String(default.clone()));
            }
        }
    }
    let links = match check_write(state, entity, &data, &many, Mode::Create, None, access) {
        Ok(links) => links,
        Err(resp) => return resp,
    };
    if let Some(owner) = owner {
        data.insert("_owner_id".to_string(), json!(owner));
    }
    let data = Value::Object(data);

    let row = match state.db.insert_with(&entity.name, &data, |conn, id| {
        links
            .iter()
            .try_for_each(|link| relations::replace_links(conn, entity, link, id, owner))
    }) {
        Ok(row) => row,
        Err(detail) => return write_error("create", &entity.name, &detail),
    };

    let table = entity.name.as_str();
    let owner = owner.unwrap_or("");
    let row_id = row
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    crate::effects::fire_webhooks(&state.webhooks, &state.entities, table, "create", &row);
    crate::effects::fire_effects(entity, "create", &row, None, &state.brain, &state.sse_hub);
    crate::scripting::fire_scripts(
        &state.script_registry,
        table,
        "create",
        &row,
        &row_id,
        None,
        &state.db,
        owner,
        "user",
        &std::collections::HashMap::new(),
    );
    let mut row = row;
    authz::redact_sensitive(entity, &mut row);
    relations::attach(&state.db, &state.entities, entity, &mut row, access, &[]);
    if let Err(e) = state
        .audit_trail
        .log("INSERT", table, &row_id, owner, &row, None)
    {
        eprintln!(
            "  \x1b[33m⚠\x1b[0m Audit log failed (INSERT {}:{}): {}",
            table, row_id, e
        );
    }
    state.sse_hub.broadcast(crate::sse::DataChangeEvent {
        entity: table.to_string(),
        action: "created".to_string(),
        id: row_id,
    });

    json_response(StatusCode::CREATED, row)
}

fn update(
    state: &AppState,
    entity: &EntityNode,
    id: &str,
    scope: &Scope,
    body: Option<&Value>,
    access: &Access,
) -> ApiResponse {
    let owner = viewer_id(access);
    let Some(obj) = body.and_then(Value::as_object) else {
        return validation_failed(StatusCode::BAD_REQUEST, "Expected a JSON object body");
    };
    let (data, many) = writable_parts(entity, obj);

    let prev = match select_one(state, entity, id, scope) {
        Ok(Some(row)) => row,
        Ok(None) => return not_found(),
        Err(detail) => return internal("update lookup", &entity.name, &detail),
    };

    let links = match check_write(state, entity, &data, &many, Mode::Update, Some(id), access) {
        Ok(links) => links,
        Err(resp) => return resp,
    };
    let data_value = Value::Object(data.clone());
    if !entity.transitions.is_empty() {
        if let Err(err_body) = crate::effects::validate_transitions(entity, &data_value, &prev) {
            return json_response(StatusCode::CONFLICT, err_body);
        }
    }

    let changed = !data.is_empty() || !links.is_empty();
    if changed {
        let mut sets: Vec<String> = data.keys().map(|k| format!("\"{k}\" = ?")).collect();
        let has_declared_updated_at = entity
            .fields
            .iter()
            .any(|f| matches!(f.name.to_lowercase().as_str(), "updatedat" | "updated_at"));
        if !has_declared_updated_at {
            sets.push("\"updated_at\" = datetime('now')".to_string());
        }
        if sets.is_empty() {
            sets.push("\"id\" = \"id\"".to_string());
        }
        let mut conditions = vec!["\"id\" = ?".to_string()];
        let mut params: Vec<String> = data.values().map(sql_text).collect();
        params.push(id.to_string());
        scope_conditions(scope, &mut conditions, &mut params);
        let sql = format!(
            "UPDATE \"{}\" SET {}{}",
            entity.name,
            sets.join(", "),
            where_clause(&conditions)
        );
        // Join rows keep the parent's owner, whoever edits the links.
        let parent_owner = prev.get("_owner_id").and_then(Value::as_str);
        match state.db.transaction(|conn| {
            let touched = conn
                .execute(&sql, rusqlite::params_from_iter(params.iter()))
                .map_err(|e| e.to_string())?;
            if touched > 0 {
                for link in &links {
                    relations::replace_links(conn, entity, link, id, parent_owner)?;
                }
            }
            Ok(touched)
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

    if changed {
        let table = entity.name.as_str();
        crate::effects::fire_webhooks(&state.webhooks, &state.entities, table, "update", &row);
        crate::effects::fire_effects(
            entity,
            "update",
            &row,
            Some(&prev),
            &state.brain,
            &state.sse_hub,
        );
        crate::scripting::fire_scripts(
            &state.script_registry,
            table,
            "update",
            &row,
            id,
            Some(&prev),
            &state.db,
            owner,
            "user",
            &std::collections::HashMap::new(),
        );
        let (mut logged_row, mut logged_prev) = (row.clone(), prev.clone());
        authz::redact_sensitive(entity, &mut logged_row);
        authz::redact_sensitive(entity, &mut logged_prev);
        if let Err(e) =
            state
                .audit_trail
                .log("UPDATE", table, id, owner, &logged_row, Some(&logged_prev))
        {
            eprintln!(
                "  \x1b[33m⚠\x1b[0m Audit log failed (UPDATE {}:{}): {}",
                table, id, e
            );
        }
        state.sse_hub.broadcast(crate::sse::DataChangeEvent {
            entity: table.to_string(),
            action: "updated".to_string(),
            id: id.to_string(),
        });
    }

    let mut row = row;
    authz::redact_sensitive(entity, &mut row);
    relations::attach(&state.db, &state.entities, entity, &mut row, access, &[]);
    json_response(StatusCode::OK, row)
}

fn delete(
    state: &AppState,
    entity: &EntityNode,
    id: &str,
    scope: &Scope,
    owner: &str,
) -> ApiResponse {
    let prev = match select_one(state, entity, id, scope) {
        Ok(Some(row)) => row,
        Ok(None) => return not_found(),
        Err(detail) => return internal("delete lookup", &entity.name, &detail),
    };

    let mut conditions = vec!["\"id\" = ?".to_string()];
    let mut params = vec![id.to_string()];
    scope_conditions(scope, &mut conditions, &mut params);
    let sql = format!(
        "DELETE FROM \"{}\"{}",
        entity.name,
        where_clause(&conditions)
    );
    match state.db.transaction(|conn| {
        conn.execute(&sql, rusqlite::params_from_iter(params.iter()))
            .map_err(|e| e.to_string())
    }) {
        Ok(0) => return not_found(),
        Ok(_) => {}
        Err(detail) => return internal("delete", &entity.name, &detail),
    }

    let table = entity.name.as_str();
    let payload = json!({"id": id, "entity": table});
    crate::effects::fire_webhooks(&state.webhooks, &state.entities, table, "delete", &payload);
    crate::effects::fire_effects(entity, "delete", &prev, None, &state.brain, &state.sse_hub);
    crate::scripting::fire_scripts(
        &state.script_registry,
        table,
        "delete",
        &prev,
        id,
        None,
        &state.db,
        owner,
        "user",
        &std::collections::HashMap::new(),
    );
    let mut logged_prev = prev.clone();
    authz::redact_sensitive(entity, &mut logged_prev);
    if let Err(e) = state.audit_trail.log(
        "DELETE",
        table,
        id,
        owner,
        &json!({"id": id}),
        Some(&logged_prev),
    ) {
        eprintln!(
            "  \x1b[33m⚠\x1b[0m Audit log failed (DELETE {}:{}): {}",
            table, id, e
        );
    }
    state.sse_hub.broadcast(crate::sse::DataChangeEvent {
        entity: table.to_string(),
        action: "deleted".to_string(),
        id: id.to_string(),
    });

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
    error(
        StatusCode::UNAUTHORIZED,
        "UNAUTHORIZED",
        "Authentication required",
    )
}

fn forbidden() -> ApiResponse {
    error(StatusCode::FORBIDDEN, "FORBIDDEN", "Not allowed")
}

fn denied(denial: Denial) -> ApiResponse {
    match denial {
        Denial::Unauthenticated => unauthorized(),
        Denial::Forbidden => forbidden(),
    }
}

/// `message` must come from entity validation (field names/rules), never
/// from the database driver.
fn validation_failed(status: StatusCode, message: &str) -> ApiResponse {
    error(status, "VALIDATION_FAILED", message)
}

/// `VALIDATION_FAILED` with `error.fields` (field → messages).
fn invalid_fields(status: StatusCode, errors: &FieldErrors) -> ApiResponse {
    json_response(status, validation::error_body(errors))
}

fn internal(action: &str, entity: &str, detail: &str) -> ApiResponse {
    eprintln!(
        "  \x1b[31m✗\x1b[0m api {} {} failed: {}",
        action, entity, detail
    );
    error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL",
        "Internal error",
    )
}

/// Unique-constraint races surface as a safe 409; anything else is logged.
fn write_error(action: &str, entity: &str, detail: &str) -> ApiResponse {
    if detail.contains("UNIQUE constraint failed") {
        return validation_failed(
            StatusCode::CONFLICT,
            "A record with this value already exists",
        );
    }
    internal(action, entity, detail)
}
