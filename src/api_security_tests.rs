//! HTTP-level regression tests for REST authorization (Sprint 1 security).
//! Drives `api_crud::handle_api` — the handler `handle_request_inner` calls
//! for `/api/<entity>` — against an in-memory SQLite built from `.cronus`.

use crate::access::{viewer_from_headers, Access, Viewer};
use crate::api_crud::{entity_matches_segment, handle_api};
use crate::auth::{create_token, Claims};
use crate::parser::{parse, AstNode};
use crate::server::state::{AppState, TraceBuffer};
use http_body_util::BodyExt;
use hyper::{HeaderMap, Method, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;

const SRC: &str = r#"app "Sec" { port 5175 }
auth {
  entity User
  login email + password
  session jwt
  roles [admin, user]
}
entity User {
  name string
  email email!
  role string
  password string sensitive
}
entity Note {
  title string!
  secret string sensitive
}
entity Post {
  title string!
}
entity Memo {
  title string!
}
entity Board shared {
  title string!
}
entity Report {
  title string!
}
api /notes {
  list   GET    /    auth:jwt
  create POST   /    auth:jwt
  detail GET    /:id auth:jwt
  update PATCH  /:id auth:jwt
  delete DELETE /:id auth:jwt
}
api /posts {
  list   GET  / auth:public
  create POST / auth:jwt
}
api /boards {
  list   GET   /    auth:jwt
  create POST  /    auth:jwt
  update PATCH /:id auth:jwt
}
api /reports {
  list GET / auth:role(admin)
}
"#;

fn state() -> AppState {
    state_from(SRC)
}

/// In-memory app state for any `.cronus` source (shared with `session` tests).
pub(crate) fn state_from(src: &str) -> AppState {
    let nodes = parse(src).expect("parse test app");
    let mut app = None;
    let mut entities = Vec::new();
    let mut apis = Vec::new();
    let mut auth_entity = None;
    let mut auth_roles = Vec::new();
    for node in nodes {
        match node {
            AstNode::App(a) => app = Some(a),
            AstNode::Entity(e) => entities.push(e),
            AstNode::Api(a) => apis.push(a),
            AstNode::Auth(a) => {
                auth_entity = Some(a.entity.clone());
                auth_roles = a.roles.clone();
            }
            _ => {}
        }
    }
    let db = crate::database::CronusDB::open_memory().expect("memory db");
    db.migrate(&entities).expect("migrate");
    AppState {
        app: app.expect("app node"),
        entities,
        pages: vec![],
        components: vec![],
        style: None,
        apis,
        db_path: ":memory:".to_string(),
        db,
        brain: None,
        auth_entity,
        auth_roles,
        auth_required_pages: vec![],
        auth_redirect: None,
        session_policy: Default::default(),
        layout: None,
        webhooks: vec![],
        rate_limiter: crate::rate_limit::RateLimiter::new(10_000, 60),
        auth_rate_limiter: crate::rate_limit::RateLimiter::new(10_000, 60),
        sse_hub: Arc::new(crate::sse::SseHub::new()),
        audit_trail: crate::audit::AuditTrail::open(":memory:").expect("audit"),
        trace_buffer: Arc::new(TraceBuffer::new()),
        script_registry: crate::scripting::ScriptRegistry::new(),
        zeus: Arc::new(crate::zeus::ZeusBuffer::new(10)),
    }
}

fn user(id: &str, role: &str) -> Claims {
    Claims {
        sub: id.to_string(),
        role: role.to_string(),
        exp: usize::MAX,
        iat: 0,
        jti: format!("test-{id}"),
    }
}

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: Value,
}

fn call(
    state: &AppState,
    method: Method,
    target: &str,
    body: Option<Value>,
    claims: Option<&Claims>,
) -> Reply {
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let access = Access {
        viewer: claims.map(|c| Viewer {
            id: c.sub.clone(),
            role: c.role.clone(),
        }),
        auth_entity: state.auth_entity.clone(),
    };
    let resp = handle_api(state, &method, path, query, body.as_ref(), &access);
    let status = resp.status();
    let headers = resp.headers().clone();
    let bytes = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("runtime")
        .block_on(resp.into_body().collect())
        .expect("body")
        .to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    Reply {
        status,
        headers,
        body,
    }
}

fn insert(state: &AppState, table: &str, row: Value) -> String {
    let row = state.db.insert(table, &row).expect("insert");
    row["id"].as_str().expect("id").to_string()
}

fn total_count(reply: &Reply) -> String {
    reply
        .headers
        .get("x-total-count")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string()
}

fn error_code(reply: &Reply) -> &str {
    reply.body["error"]["code"].as_str().unwrap_or("")
}

#[test]
fn anonymous_gets_401_on_jwt_routes() {
    let s = state();
    let list = call(&s, Method::GET, "/api/notes", None, None);
    assert_eq!(list.status, StatusCode::UNAUTHORIZED);
    assert_eq!(error_code(&list), "UNAUTHORIZED");
    let create = call(
        &s,
        Method::POST,
        "/api/notes",
        Some(json!({"title": "x"})),
        None,
    );
    assert_eq!(create.status, StatusCode::UNAUTHORIZED);
    assert_eq!(s.db.count("Note").expect("count"), 0);
}

#[test]
fn public_route_allows_anonymous_but_jwt_sibling_does_not() {
    let s = state();
    insert(&s, "Post", json!({"title": "hello", "_owner_id": "alice"}));
    let list = call(&s, Method::GET, "/api/posts", None, None);
    assert_eq!(list.status, StatusCode::OK);
    assert_eq!(list.body.as_array().map(Vec::len), Some(1));
    let create = call(
        &s,
        Method::POST,
        "/api/posts",
        Some(json!({"title": "x"})),
        None,
    );
    assert_eq!(create.status, StatusCode::UNAUTHORIZED);
}

#[test]
fn undeclared_routes_are_404_even_for_admin() {
    let s = state();
    let admin = user("root", "admin");
    let id = insert(&s, "Post", json!({"title": "p"}));
    assert_eq!(
        call(
            &s,
            Method::DELETE,
            &format!("/api/posts/{id}"),
            None,
            Some(&admin)
        )
        .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &s,
            Method::GET,
            &format!("/api/posts/{id}"),
            None,
            Some(&admin)
        )
        .status,
        StatusCode::NOT_FOUND
    );
    let note = insert(&s, "Note", json!({"title": "n", "_owner_id": "root"}));
    let put = call(
        &s,
        Method::PUT,
        &format!("/api/notes/{note}"),
        Some(json!({"title": "z"})),
        Some(&admin),
    );
    assert_eq!(put.status, StatusCode::NOT_FOUND);
    assert_eq!(s.db.count("Post").expect("count"), 1);
}

#[test]
fn auto_crud_without_api_block_requires_auth() {
    let s = state();
    assert_eq!(
        call(&s, Method::GET, "/api/memos", None, None).status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(
            &s,
            Method::POST,
            "/api/memos",
            Some(json!({"title": "m"})),
            None
        )
        .status,
        StatusCode::UNAUTHORIZED
    );
    let alice = user("alice", "user");
    assert_eq!(
        call(
            &s,
            Method::POST,
            "/api/memos",
            Some(json!({"title": "m"})),
            Some(&alice)
        )
        .status,
        StatusCode::CREATED
    );
    assert_eq!(
        call(&s, Method::GET, "/api/memos", None, Some(&alice)).status,
        StatusCode::OK
    );
}

#[test]
fn cross_owner_get_patch_delete_return_404_and_leave_row_intact() {
    let s = state();
    let alice = user("alice", "user");
    let bob = user("bob", "user");
    let created = call(
        &s,
        Method::POST,
        "/api/notes",
        Some(json!({"title": "mine"})),
        Some(&alice),
    );
    assert_eq!(created.status, StatusCode::CREATED);
    let id = created.body["id"].as_str().expect("id").to_string();
    let path = format!("/api/notes/{id}");

    assert_eq!(
        call(&s, Method::GET, &path, None, Some(&bob)).status,
        StatusCode::NOT_FOUND
    );
    let patch = call(
        &s,
        Method::PATCH,
        &path,
        Some(json!({"title": "pwned"})),
        Some(&bob),
    );
    assert_eq!(patch.status, StatusCode::NOT_FOUND);
    assert_eq!(
        call(&s, Method::DELETE, &path, None, Some(&bob)).status,
        StatusCode::NOT_FOUND
    );

    let row =
        s.db.find_by_id("Note", &id)
            .expect("query")
            .expect("row still exists");
    assert_eq!(row["title"], "mine");
    assert_eq!(row["_owner_id"], "alice");
    assert_eq!(
        call(&s, Method::GET, &path, None, Some(&alice)).status,
        StatusCode::OK
    );
}

#[test]
fn rows_without_owner_are_hidden_from_non_admins() {
    let s = state();
    let bob = user("bob", "user");
    let orphan = insert(&s, "Note", json!({"title": "orphan"}));
    assert_eq!(
        call(
            &s,
            Method::GET,
            &format!("/api/notes/{orphan}"),
            None,
            Some(&bob)
        )
        .status,
        StatusCode::NOT_FOUND
    );
    let list = call(&s, Method::GET, "/api/notes", None, Some(&bob));
    assert_eq!(list.body, json!([]));
    assert_eq!(total_count(&list), "0");
}

#[test]
fn owner_and_role_in_body_are_ignored() {
    let s = state();
    let alice = user("alice", "user");
    let created = call(
        &s,
        Method::POST,
        "/api/notes",
        Some(json!({"title": "t", "_owner_id": "bob", "role": "admin", "id": "chosen"})),
        Some(&alice),
    );
    assert_eq!(created.status, StatusCode::CREATED);
    let id = created.body["id"].as_str().expect("id").to_string();
    assert_ne!(id, "chosen");
    let patch = call(
        &s,
        Method::PATCH,
        &format!("/api/notes/{id}"),
        Some(json!({"_owner_id": "bob", "title": "u"})),
        Some(&alice),
    );
    assert_eq!(patch.status, StatusCode::OK);
    let row = s.db.find_by_id("Note", &id).expect("query").expect("row");
    assert_eq!(row["_owner_id"], "alice");
    assert_eq!(row["title"], "u");
    assert!(row.get("role").is_none());
}

#[test]
fn user_entity_is_self_only_through_generic_rest() {
    let s = state();
    let alice_id = insert(
        &s,
        "User",
        json!({"name": "A", "email": "a@x.io", "role": "user", "password": "$argon2id$hash"}),
    );
    let bob_id = insert(
        &s,
        "User",
        json!({"name": "B", "email": "b@x.io", "role": "user", "password": "$argon2id$hash"}),
    );
    let alice = user(&alice_id, "user");
    let admin = user("root", "admin");

    assert_eq!(
        call(&s, Method::GET, "/api/users", None, None).status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(&s, Method::GET, "/api/users", None, Some(&alice)).status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &s,
            Method::GET,
            &format!("/api/users/{bob_id}"),
            None,
            Some(&alice)
        )
        .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &s,
            Method::GET,
            &format!("/api/users/{alice_id}"),
            None,
            Some(&alice)
        )
        .status,
        StatusCode::OK
    );

    let anon_patch = call(
        &s,
        Method::PATCH,
        &format!("/api/users/{alice_id}"),
        Some(json!({"role": "admin"})),
        None,
    );
    assert_eq!(anon_patch.status, StatusCode::UNAUTHORIZED);
    let bob_patch = call(
        &s,
        Method::PATCH,
        &format!("/api/users/{bob_id}"),
        Some(json!({"name": "hacked"})),
        Some(&alice),
    );
    assert_eq!(bob_patch.status, StatusCode::NOT_FOUND);
    let self_patch = call(
        &s,
        Method::PATCH,
        &format!("/api/users/{alice_id}"),
        Some(json!({"role": "admin", "name": "Alice"})),
        Some(&alice),
    );
    assert_eq!(self_patch.status, StatusCode::OK);
    let row =
        s.db.find_by_id("User", &alice_id)
            .expect("query")
            .expect("row");
    assert_eq!(row["role"], "user");
    assert_eq!(row["name"], "Alice");
    assert_eq!(
        s.db.find_by_id("User", &bob_id)
            .expect("query")
            .expect("row")["name"],
        "B"
    );

    let signup_bypass = json!({"name": "E", "email": "e@x.io", "password": "x", "role": "admin"});
    assert_eq!(
        call(
            &s,
            Method::POST,
            "/api/users",
            Some(signup_bypass.clone()),
            None
        )
        .status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(
            &s,
            Method::POST,
            "/api/users",
            Some(signup_bypass.clone()),
            Some(&alice)
        )
        .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &s,
            Method::POST,
            "/api/users",
            Some(signup_bypass),
            Some(&admin)
        )
        .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &s,
            Method::DELETE,
            &format!("/api/users/{bob_id}"),
            None,
            Some(&alice)
        )
        .status,
        StatusCode::FORBIDDEN
    );
}

#[test]
fn sensitive_fields_never_leave_rest() {
    let s = state();
    let admin = user("root", "admin");
    let uid = insert(
        &s,
        "User",
        json!({"name": "A", "email": "a@x.io", "role": "user", "password": "$argon2id$secret-hash"}),
    );

    let list = call(&s, Method::GET, "/api/users", None, Some(&admin));
    assert_eq!(list.status, StatusCode::OK);
    assert!(!list.body.to_string().contains("secret-hash"));
    let detail = call(
        &s,
        Method::GET,
        &format!("/api/users/{uid}"),
        None,
        Some(&admin),
    );
    assert_eq!(detail.status, StatusCode::OK);
    assert!(detail.body.get("password").is_none());
    let oracle = call(
        &s,
        Method::GET,
        "/api/users?search=secret-hash",
        None,
        Some(&admin),
    );
    assert_eq!(oracle.body, json!([]));

    let alice = user("alice", "user");
    let note = insert(
        &s,
        "Note",
        json!({"title": "n", "secret": "s3cr3t", "_owner_id": "alice"}),
    );
    for reply in [
        call(&s, Method::GET, "/api/notes", None, Some(&alice)),
        call(
            &s,
            Method::GET,
            &format!("/api/notes/{note}"),
            None,
            Some(&alice),
        ),
        call(
            &s,
            Method::PATCH,
            &format!("/api/notes/{note}"),
            Some(json!({"title": "m", "secret": "changed"})),
            Some(&alice),
        ),
    ] {
        assert_eq!(reply.status, StatusCode::OK);
        assert!(
            !reply.body.to_string().contains("s3cr3t"),
            "leaked in {}",
            reply.body
        );
    }
    let row = s.db.find_by_id("Note", &note).expect("query").expect("row");
    assert_eq!(
        row["secret"], "s3cr3t",
        "sensitive field must not be client-writable"
    );
}

#[test]
fn sensitive_fields_never_reach_the_audit_trail() {
    let s = state();
    let alice = user("alice", "user");
    let created = call(
        &s,
        Method::POST,
        "/api/notes",
        Some(json!({"title": "n"})),
        Some(&alice),
    );
    assert_eq!(created.status, StatusCode::CREATED);
    let note = insert(
        &s,
        "Note",
        json!({"title": "n", "secret": "s3cr3t", "_owner_id": "alice"}),
    );
    let patched = call(
        &s,
        Method::PATCH,
        &format!("/api/notes/{note}"),
        Some(json!({"title": "m"})),
        Some(&alice),
    );
    assert_eq!(patched.status, StatusCode::OK);
    let deleted = call(
        &s,
        Method::DELETE,
        &format!("/api/notes/{note}"),
        None,
        Some(&alice),
    );
    assert_eq!(deleted.status, StatusCode::OK);
    let trail = s.audit_trail.query(100).expect("audit query").to_string();
    assert!(
        trail.contains("UPDATE") && trail.contains("DELETE"),
        "audit entries missing: {trail}"
    );
    assert!(
        !trail.contains("s3cr3t"),
        "sensitive value stored in audit trail: {trail}"
    );
}

#[test]
fn admin_sees_every_owner() {
    let s = state();
    insert(&s, "Note", json!({"title": "a", "_owner_id": "alice"}));
    insert(&s, "Note", json!({"title": "b", "_owner_id": "bob"}));
    insert(&s, "Note", json!({"title": "orphan"}));
    let list = call(
        &s,
        Method::GET,
        "/api/notes",
        None,
        Some(&user("root", "admin")),
    );
    assert_eq!(list.body.as_array().map(Vec::len), Some(3));
    assert_eq!(total_count(&list), "3");
}

#[test]
fn total_count_is_the_scoped_total_not_page_size() {
    let s = state();
    for i in 0..5 {
        insert(
            &s,
            "Note",
            json!({"title": format!("a{i}"), "_owner_id": "alice"}),
        );
    }
    insert(&s, "Note", json!({"title": "b", "_owner_id": "bob"}));
    let page = call(
        &s,
        Method::GET,
        "/api/notes?limit=2&offset=2",
        None,
        Some(&user("alice", "user")),
    );
    assert_eq!(page.body.as_array().map(Vec::len), Some(2));
    assert_eq!(total_count(&page), "5");
    assert_eq!(
        page.headers.get("x-offset").and_then(|v| v.to_str().ok()),
        Some("2")
    );
}

#[test]
fn search_covers_all_rows_not_first_page() {
    let s = state();
    insert(
        &s,
        "Note",
        json!({"title": "the needle", "_owner_id": "alice"}),
    );
    insert(
        &s,
        "Note",
        json!({"title": "bob needle", "_owner_id": "bob"}),
    );
    for i in 0..150 {
        insert(
            &s,
            "Note",
            json!({"title": format!("hay {i}"), "_owner_id": "alice"}),
        );
    }
    let found = call(
        &s,
        Method::GET,
        "/api/notes?search=needle",
        None,
        Some(&user("alice", "user")),
    );
    assert_eq!(found.status, StatusCode::OK);
    assert_eq!(found.body.as_array().map(Vec::len), Some(1));
    assert_eq!(found.body[0]["title"], "the needle");
    assert_eq!(total_count(&found), "1");
    let wildcard = call(
        &s,
        Method::GET,
        "/api/notes?q=%25",
        None,
        Some(&user("alice", "user")),
    );
    assert_eq!(wildcard.body, json!([]));
}

#[test]
fn entity_is_resolved_by_exact_segment() {
    let s = state();
    let alice = user("alice", "user");
    insert(&s, "Note", json!({"title": "a", "_owner_id": "alice"}));
    assert_eq!(
        call(&s, Method::GET, "/api/notesarchive", None, Some(&alice)).status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(&s, Method::GET, "/api/noteszzz/1", None, Some(&alice)).status,
        StatusCode::NOT_FOUND
    );
    // The singular alias still hits Note and still obeys the declared api block.
    assert_eq!(
        call(&s, Method::GET, "/api/note", None, None).status,
        StatusCode::UNAUTHORIZED
    );
    assert!(entity_matches_segment("Category", "categories"));
    assert!(entity_matches_segment("BlogPost", "blog-posts"));
    assert!(!entity_matches_segment("Note", "notesarchive"));
}

#[test]
fn shared_entity_reads_are_open_but_writes_stay_owner_scoped() {
    let s = state();
    let id = insert(&s, "Board", json!({"title": "team", "_owner_id": "alice"}));
    let bob = user("bob", "user");
    let list = call(&s, Method::GET, "/api/boards", None, Some(&bob));
    assert_eq!(list.body.as_array().map(Vec::len), Some(1));
    let patch = call(
        &s,
        Method::PATCH,
        &format!("/api/boards/{id}"),
        Some(json!({"title": "bob's"})),
        Some(&bob),
    );
    assert_eq!(patch.status, StatusCode::NOT_FOUND);
    assert_eq!(
        s.db.find_by_id("Board", &id).expect("query").expect("row")["title"],
        "team"
    );
    assert_eq!(
        call(&s, Method::GET, "/api/boards", None, None).status,
        StatusCode::UNAUTHORIZED
    );
}

#[test]
fn role_routes_return_403_for_other_roles() {
    let s = state();
    assert_eq!(
        call(&s, Method::GET, "/api/reports", None, None).status,
        StatusCode::UNAUTHORIZED
    );
    let denied = call(
        &s,
        Method::GET,
        "/api/reports",
        None,
        Some(&user("alice", "user")),
    );
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    assert_eq!(error_code(&denied), "FORBIDDEN");
    assert_eq!(
        call(
            &s,
            Method::GET,
            "/api/reports",
            None,
            Some(&user("root", "admin"))
        )
        .status,
        StatusCode::OK
    );
}

#[test]
fn errors_use_stable_codes_without_driver_text() {
    let s = state();
    let alice = user("alice", "user");
    let missing = call(
        &s,
        Method::POST,
        "/api/notes",
        Some(json!({"secret": "x"})),
        Some(&alice),
    );
    assert_eq!(missing.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error_code(&missing), "VALIDATION_FAILED");
    assert_eq!(
        missing.body["error"]["fields"]["title"],
        json!(["is required"])
    );
    let not_object = call(
        &s,
        Method::POST,
        "/api/notes",
        Some(json!(["x"])),
        Some(&alice),
    );
    assert_eq!(error_code(&not_object), "VALIDATION_FAILED");
    let gone = call(&s, Method::GET, "/api/notes/nope", None, Some(&alice));
    assert_eq!(
        gone.body,
        json!({"error": {"code": "NOT_FOUND", "message": "Not found"}})
    );
}

#[test]
fn claims_come_from_bearer_or_cookie() {
    let secret = "test-secret";
    let token = create_token("u1", "user", secret);
    let headers = |name: &'static str, value: String| {
        let mut h = HeaderMap::new();
        h.insert(name, value.parse().unwrap());
        h
    };
    let from_bearer =
        viewer_from_headers(&headers("authorization", format!("Bearer {token}")), secret);
    assert_eq!(from_bearer.map(|v| v.id), Some("u1".to_string()));
    let from_cookie = viewer_from_headers(
        &headers("cookie", format!("theme=dark; cronus_token={token}")),
        secret,
    );
    assert_eq!(from_cookie.map(|v| v.id), Some("u1".to_string()));
    assert!(
        viewer_from_headers(&headers("authorization", "Bearer forged".into()), secret).is_none()
    );
    assert!(viewer_from_headers(&HeaderMap::new(), secret).is_none());
}
