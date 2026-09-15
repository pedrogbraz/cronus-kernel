//! HTTP-level tests for field validation (field-level `422` errors) and
//! many-to-many relations (`tags -> Tag[]`) on REST (`api_crud::handle_api`).
//! Forms/actions and GraphQL cover the same rules in their own modules.

use crate::access::{Access, Viewer};
use crate::api_crud::handle_api;
use crate::server::state::AppState;
use http_body_util::BodyExt;
use hyper::{Method, StatusCode};
use serde_json::{json, Value};

pub(crate) const SRC: &str = r#"app "V" { port 5175 }
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
entity Post {
  title   string! min:3 max:120
  age     number  min:0 max:150
  slug    slug    match:"^[a-z0-9-]+$"
  contact email   unique
  tags    -> Tag[]
  secret  string  sensitive
}
entity Tag {
  label  string!
  secret string sensitive
}
entity Topic shared {
  name string!
}
entity Article {
  title  string!
  topics -> Topic[] required
}
page "/posts" type:custom requires:auth {
  section form { bind Post { query all } }
}
"#;

pub(crate) fn state() -> AppState {
    crate::api_security_tests::state_from(SRC)
}

pub(crate) fn viewer(id: &str, role: &str) -> Access {
    Access {
        viewer: Some(Viewer {
            id: id.into(),
            role: role.into(),
        }),
        auth_entity: Some("User".into()),
    }
}

struct Reply {
    status: StatusCode,
    body: Value,
}

fn call(
    state: &AppState,
    method: Method,
    target: &str,
    body: Option<Value>,
    who: &Access,
) -> Reply {
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let resp = handle_api(state, &method, path, query, body.as_ref(), who);
    let status = resp.status();
    let bytes = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("runtime")
        .block_on(resp.into_body().collect())
        .expect("body")
        .to_bytes();
    Reply {
        status,
        body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    }
}

fn create(state: &AppState, table: &str, body: Value, who: &Access) -> String {
    let r = call(
        state,
        Method::POST,
        &format!("/api/{table}"),
        Some(body),
        who,
    );
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    r.body["id"].as_str().expect("id").to_string()
}

fn fields(r: &Reply) -> &Value {
    &r.body["error"]["fields"]
}

// ── Field validation ────────────────────────────────────────

#[test]
fn create_returns_422_with_every_field_error() {
    let s = state();
    let alice = viewer("alice", "user");
    let r = call(
        &s,
        Method::POST,
        "/api/posts",
        Some(json!({"title": "ab", "age": 200, "slug": "Bad Slug", "contact": "nope"})),
        &alice,
    );
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", r.body);
    assert_eq!(r.body["error"]["code"], "VALIDATION_FAILED");
    assert!(r.body["error"]["message"].is_string());
    assert_eq!(
        fields(&r)["title"],
        json!(["must be at least 3 characters"])
    );
    assert_eq!(fields(&r)["age"], json!(["must be at most 150"]));
    assert_eq!(
        fields(&r)["slug"],
        json!(["must match the pattern ^[a-z0-9-]+$"])
    );
    assert_eq!(fields(&r)["contact"], json!(["must be a valid email"]));
    assert_eq!(s.db.count("Post").unwrap(), 0);
}

#[test]
fn create_reports_required_min_and_max_length() {
    let s = state();
    let alice = viewer("alice", "user");
    let missing = call(
        &s,
        Method::POST,
        "/api/posts",
        Some(json!({"age": -1})),
        &alice,
    );
    assert_eq!(missing.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(fields(&missing)["title"], json!(["is required"]));
    assert_eq!(fields(&missing)["age"], json!(["must be at least 0"]));
    let long = call(
        &s,
        Method::POST,
        "/api/posts",
        Some(json!({"title": "é".repeat(121)})),
        &alice,
    );
    assert_eq!(
        fields(&long)["title"],
        json!(["must be at most 120 characters"])
    );
    // 120 multi-byte characters are within max:120 (chars, not bytes).
    create(&s, "posts", json!({"title": "é".repeat(120)}), &alice);
    let wrong_type = call(
        &s,
        Method::POST,
        "/api/posts",
        Some(json!({"title": "okay", "age": "old"})),
        &alice,
    );
    assert_eq!(fields(&wrong_type)["age"], json!(["must be a number"]));
}

#[test]
fn unique_conflicts_are_409_with_field_errors_on_create_and_update() {
    let s = state();
    let alice = viewer("alice", "user");
    create(
        &s,
        "posts",
        json!({"title": "first", "contact": "a@b.co"}),
        &alice,
    );
    let dup = call(
        &s,
        Method::POST,
        "/api/posts",
        Some(json!({"title": "second", "contact": "a@b.co"})),
        &alice,
    );
    assert_eq!(dup.status, StatusCode::CONFLICT, "{}", dup.body);
    assert_eq!(dup.body["error"]["code"], "VALIDATION_FAILED");
    assert_eq!(fields(&dup)["contact"], json!(["already exists"]));

    let other = create(
        &s,
        "posts",
        json!({"title": "third", "contact": "c@d.co"}),
        &alice,
    );
    let upd = call(
        &s,
        Method::PATCH,
        &format!("/api/posts/{other}"),
        Some(json!({"contact": "a@b.co"})),
        &alice,
    );
    assert_eq!(upd.status, StatusCode::CONFLICT, "{}", upd.body);
    assert_eq!(fields(&upd)["contact"], json!(["already exists"]));
    // Re-saving a row's own unique value is not a conflict.
    let same = call(
        &s,
        Method::PATCH,
        &format!("/api/posts/{other}"),
        Some(json!({"contact": "c@d.co"})),
        &alice,
    );
    assert_eq!(same.status, StatusCode::OK, "{}", same.body);
}

#[test]
fn update_validates_only_the_fields_sent() {
    let s = state();
    let alice = viewer("alice", "user");
    let id = create(&s, "posts", json!({"title": "valid"}), &alice);
    let bad = call(
        &s,
        Method::PATCH,
        &format!("/api/posts/{id}"),
        Some(json!({"age": 151, "slug": "UPPER"})),
        &alice,
    );
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", bad.body);
    assert_eq!(fields(&bad)["age"], json!(["must be at most 150"]));
    assert!(fields(&bad)["slug"].is_array());
    assert!(fields(&bad).get("title").is_none(), "title not sent");
    let ok = call(
        &s,
        Method::PATCH,
        &format!("/api/posts/{id}"),
        Some(json!({"slug": "a-slug"})),
        &alice,
    );
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.body);
}

// ── Many-to-many ────────────────────────────────────────────

#[test]
fn owner_attaches_and_detaches_own_records() {
    let s = state();
    let alice = viewer("alice", "user");
    let t1 = create(&s, "tags", json!({"label": "one", "secret": "s"}), &alice);
    let t2 = create(&s, "tags", json!({"label": "two"}), &alice);
    let post = create(
        &s,
        "posts",
        json!({"title": "linked", "tags": [t1, t2]}),
        &alice,
    );

    let detail = call(&s, Method::GET, &format!("/api/posts/{post}"), None, &alice);
    assert_eq!(detail.status, StatusCode::OK);
    assert_eq!(detail.body["tags"], json!([t1, t2]));

    let listed = call(&s, Method::GET, "/api/posts", None, &alice);
    assert_eq!(listed.body[0]["tags"], json!([t1, t2]));

    let expanded = call(
        &s,
        Method::GET,
        &format!("/api/posts/{post}?expand=tags"),
        None,
        &alice,
    );
    let objs = expanded.body["tags"].as_array().expect("expanded tags");
    assert_eq!(objs.len(), 2);
    assert_eq!(objs[0]["label"], "one");
    assert!(
        objs[0].get("secret").is_none(),
        "expanded rows are redacted"
    );

    let detach = call(
        &s,
        Method::PATCH,
        &format!("/api/posts/{post}"),
        Some(json!({"tags": [t2]})),
        &alice,
    );
    assert_eq!(detach.status, StatusCode::OK, "{}", detach.body);
    assert_eq!(detach.body["tags"], json!([t2]));
    let untouched = call(
        &s,
        Method::PATCH,
        &format!("/api/posts/{post}"),
        Some(json!({"title": "renamed"})),
        &alice,
    );
    assert_eq!(
        untouched.body["tags"],
        json!([t2]),
        "absent field keeps links"
    );
    let cleared = call(
        &s,
        Method::PATCH,
        &format!("/api/posts/{post}"),
        Some(json!({"tags": []})),
        &alice,
    );
    assert_eq!(cleared.body["tags"], json!([]));
}

#[test]
fn cannot_attach_another_users_private_record_or_unknown_ids() {
    let s = state();
    let alice = viewer("alice", "user");
    let bob = viewer("bob", "user");
    let bobs_tag = create(&s, "tags", json!({"label": "private"}), &bob);
    for bad in [json!([bobs_tag]), json!(["missing-id"])] {
        let r = call(
            &s,
            Method::POST,
            "/api/posts",
            Some(json!({"title": "steal", "tags": bad})),
            &alice,
        );
        assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", r.body);
        assert_eq!(fields(&r)["tags"], json!(["contains an unknown id"]));
    }
    let not_array = call(
        &s,
        Method::POST,
        "/api/posts",
        Some(json!({"title": "shape", "tags": "x"})),
        &alice,
    );
    assert_eq!(
        fields(&not_array)["tags"],
        json!(["must be an array of ids"])
    );
    assert_eq!(s.db.count("Post").unwrap(), 0);

    // Shared entities are readable by every signed-in user, so attachable.
    let topic = create(&s, "topics", json!({"name": "rust"}), &bob);
    create(
        &s,
        "articles",
        json!({"title": "a", "topics": [topic]}),
        &alice,
    );
    let empty = call(
        &s,
        Method::POST,
        "/api/articles",
        Some(json!({"title": "b", "topics": []})),
        &alice,
    );
    assert_eq!(fields(&empty)["topics"], json!(["is required"]));
}

#[test]
fn join_rows_carry_the_parent_owner_and_reads_follow_parent_scope() {
    let s = state();
    let alice = viewer("alice", "user");
    let admin = viewer("root", "admin");
    let tag = create(&s, "tags", json!({"label": "mine"}), &alice);
    let post = create(&s, "posts", json!({"title": "owned"}), &alice);
    let r = call(
        &s,
        Method::PATCH,
        &format!("/api/posts/{post}"),
        Some(json!({"tags": [tag]})),
        &admin,
    );
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    let rows =
        s.db.query_raw_params(
            "SELECT \"_owner_id\" FROM \"Post_tags\" WHERE \"source_id\" = ?",
            &[post.clone()],
        )
        .unwrap();
    assert_eq!(rows, vec![json!({"_owner_id": "alice"})]);
    let bob = viewer("bob", "user");
    let hidden = call(&s, Method::GET, &format!("/api/posts/{post}"), None, &bob);
    assert_eq!(hidden.status, StatusCode::NOT_FOUND);
}

#[test]
fn deleting_either_side_cascades_join_rows() {
    let s = state();
    let alice = viewer("alice", "user");
    let t1 = create(&s, "tags", json!({"label": "one"}), &alice);
    let t2 = create(&s, "tags", json!({"label": "two"}), &alice);
    let post = create(
        &s,
        "posts",
        json!({"title": "linked", "tags": [t1, t2]}),
        &alice,
    );
    let join_count = |s: &AppState| {
        s.db.query_raw("SELECT COUNT(*) AS n FROM \"Post_tags\"")
            .unwrap()[0]["n"]
            .clone()
    };
    assert_eq!(join_count(&s), json!(2));

    let del = call(&s, Method::DELETE, &format!("/api/tags/{t1}"), None, &alice);
    assert_eq!(del.status, StatusCode::OK);
    assert_eq!(join_count(&s), json!(1));
    let detail = call(&s, Method::GET, &format!("/api/posts/{post}"), None, &alice);
    assert_eq!(detail.body["tags"], json!([t2]));

    let del = call(
        &s,
        Method::DELETE,
        &format!("/api/posts/{post}"),
        None,
        &alice,
    );
    assert_eq!(del.status, StatusCode::OK);
    assert_eq!(join_count(&s), json!(0));
}

#[test]
fn migrate_creates_join_table_with_indexes_on_both_columns() {
    let s = state();
    let idx = s
        .db
        .query_raw("SELECT name FROM sqlite_master WHERE type = 'index' AND tbl_name = 'Post_tags'")
        .unwrap();
    let names: Vec<&str> = idx.iter().filter_map(|r| r["name"].as_str()).collect();
    assert!(names.contains(&"idx_Post_tags_source_id"), "{names:?}");
    assert!(names.contains(&"idx_Post_tags_target_id"), "{names:?}");
    let cols = s.db.query_raw("SELECT * FROM \"Post\" LIMIT 0").unwrap();
    assert!(cols.is_empty());
    let info = s.db.query_raw("PRAGMA table_info(\"Post\")").unwrap();
    assert!(
        info.iter().all(|c| c["name"] != "tags"),
        "many-to-many fields are not columns"
    );
}
