// CRONUS Action Execution Engine
//
// Processes action instructions from button clicks / form submits.
// Returns an effect envelope (toast, navigate, refresh, etc.)
//
// SECURITY: the client never supplies instructions. Kernel-rendered buttons
// carry `data-action-id`; the runtime posts `{action_id, entity, id}` and the
// server executes its own copy of the declared block. Writes are owner-scoped
// in SQL (`access::scoped_*`). `/_form` creates and edits go through
// `handle_form` with the same declared-form + session + owner rules.

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use crate::access::{self, Access, Denial, WriteScope};
use crate::database::CronusDB;
use crate::parser::{ActionBlock, EntityNode, FieldType, PageNode, SectionNode};
use crate::server::state::AppState;

#[derive(Debug)]
pub struct ActionEffect {
    pub effect_type: String, // "toast", "navigate", "refresh", "open", "close"
    pub target: String,
    pub value: String,
    pub style: String,
}

/// Validate a single field value against its schema type.
/// Returns None if valid, Some(error_message) if invalid.
pub fn validate_field_value(field_name: &str, value: &str, entity: &EntityNode) -> Option<String> {
    let field = match entity
        .fields
        .iter()
        .find(|f| f.name.eq_ignore_ascii_case(field_name))
    {
        Some(f) => f,
        None => {
            return Some(format!(
                "Field '{}' does not exist on entity '{}'",
                field_name, entity.name
            ))
        }
    };

    // Empty values are allowed unless required (required check is separate)
    if value.is_empty() {
        return None;
    }

    match field.field_type {
        FieldType::Number | FieldType::Money | FieldType::Percentage => {
            if value.parse::<f64>().is_err() {
                return Some(format!("'{}' must be a number", field_name));
            }
        }
        FieldType::Boolean => {
            let lower = value.to_lowercase();
            if !["true", "false", "0", "1"].contains(&lower.as_str()) {
                return Some(format!("'{}' must be true/false", field_name));
            }
        }
        FieldType::Email => {
            if !value.contains('@') || !value.contains('.') || value.len() < 5 {
                return Some(format!("'{}' must be a valid email", field_name));
            }
        }
        FieldType::Enum => {
            if let Some(ref vals) = field.enum_values {
                if !vals.iter().any(|v| v.eq_ignore_ascii_case(value)) {
                    return Some(format!(
                        "'{}' must be one of: {}",
                        field_name,
                        vals.join(", ")
                    ));
                }
            }
        }
        FieldType::Url => {
            if !value.starts_with("http://") && !value.starts_with("https://") {
                return Some(format!("'{}' must be a valid URL", field_name));
            }
        }
        _ => {} // String, Text, Date, Slug, Phone, Ulid, Json, Ip, Relation — no strict validation
    }

    None
}

/// Validate form data for INSERT against entity schema.
/// Returns a map of field_name -> error_message for any validation failures.
pub fn validate_form_data(
    data: &Value,
    entity: &EntityNode,
) -> std::collections::HashMap<String, String> {
    let mut errors = std::collections::HashMap::new();
    let obj = match data.as_object() {
        Some(o) => o,
        None => return errors,
    };

    // Check required fields are present and non-empty
    for field in &entity.fields {
        if field.required {
            let val = obj
                .get(&field.name)
                .or_else(|| obj.get(&field.name.to_lowercase()))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if val.trim().is_empty() {
                errors.insert(field.name.clone(), format!("'{}' is required", field.name));
            }
        }
    }

    // Validate types for all provided fields
    for (key, val) in obj {
        let val_string = val.to_string();
        let str_val = val.as_str().unwrap_or(&val_string);
        if let Some(err) = validate_field_value(key, str_val, entity) {
            errors.insert(key.clone(), err);
        }
    }

    errors
}

// ══════════════════════════════════════════════════
// DECLARED ACTIONS / FORMS (AST registry)
// ══════════════════════════════════════════════════

/// An action block declared in a page section, with the entity it acts on.
#[derive(Debug, Clone)]
pub struct DeclaredAction {
    /// Stable identifier, see `action_id`.
    pub id: String,
    pub entity: String,
    pub block: ActionBlock,
}

fn section_entity(section: &SectionNode) -> String {
    section
        .config
        .get("entity")
        .cloned()
        .or_else(|| section.binding.as_ref().map(|b| b.entity.clone()))
        .unwrap_or_default()
}

/// Stable id of a declared action: sha256(entity | canonical block)[..16].
/// Computed the same way by the renderer (`data-action-id`) and by the
/// server registry. Two identical blocks on the same entity share an id,
/// which is harmless: they execute identically.
pub fn action_id(entity: &str, block: &ActionBlock) -> Option<String> {
    let canonical = serde_json::to_value(block).ok()?;
    let mut h = Sha256::new();
    h.update(format!("{}|{}", entity.to_ascii_lowercase(), canonical).as_bytes());
    Some(hex::encode(h.finalize())[..16].to_string())
}

/// Id for an item-level `on_*` block as the parser stores it (JSON string).
pub fn action_id_for_item(entity: &str, raw_block: &str) -> Option<String> {
    let block: ActionBlock = serde_json::from_str(raw_block).ok()?;
    action_id(entity, &block)
}

/// Every action block in the AST: item-level `on click { … }` (rendered as
/// a button with `data-action-id`) and section-level `on …` blocks.
pub fn declared_actions(pages: &[PageNode]) -> Vec<DeclaredAction> {
    let mut out = Vec::new();
    for page in pages {
        for section in &page.sections {
            let entity = section_entity(section);
            let mut push = |block: ActionBlock| {
                if let Some(id) = action_id(&entity, &block) {
                    out.push(DeclaredAction {
                        id,
                        entity: entity.clone(),
                        block,
                    });
                }
            };
            for item in &section.items {
                for (_, raw) in item.iter().filter(|(k, _)| k.starts_with("on_")) {
                    if let Ok(block) = serde_json::from_str::<ActionBlock>(raw) {
                        push(block);
                    }
                }
            }
            for block in &section.actions {
                push(block.clone());
            }
        }
    }
    out
}

/// Resolve the client's `action_id` to a declared action on `client_entity`.
/// There is no other way to name an action: client-sent instruction JSON is
/// never matched or executed.
pub fn find_declared_action<'a>(
    declared: &'a [DeclaredAction],
    action_id: &str,
    client_entity: &str,
) -> Option<&'a DeclaredAction> {
    if action_id.is_empty() {
        return None;
    }
    declared
        .iter()
        .find(|a| a.id == action_id && a.entity.eq_ignore_ascii_case(client_entity))
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeclaredForm {
    pub entity: String,
    /// `bind X { scope:public }` on a page without `requires:` — anonymous
    /// submissions allowed (rows get no `_owner_id`).
    pub public: bool,
}

/// A `/_form/<section_type>` post is only accepted for an entity bound by a
/// declared section of that type.
pub fn find_declared_form(
    pages: &[PageNode],
    section_type: &str,
    entity: &str,
) -> Option<DeclaredForm> {
    let mut found: Option<DeclaredForm> = None;
    for page in pages {
        let page_open = page.requires.is_none() && !page.config.contains_key("requires");
        for section in &page.sections {
            let declared = section_entity(section);
            if section.section_type != section_type
                || declared.is_empty()
                || !declared.eq_ignore_ascii_case(entity)
            {
                continue;
            }
            let public = page_open && section.binding.as_ref().map(|b| b.public).unwrap_or(false);
            match found.as_mut() {
                Some(f) => f.public |= public,
                None => {
                    found = Some(DeclaredForm {
                        entity: declared,
                        public,
                    })
                }
            }
        }
    }
    found
}

// ══════════════════════════════════════════════════
// EXECUTION
// ══════════════════════════════════════════════════

#[derive(Debug, PartialEq)]
pub enum ActionDenied {
    Unauthenticated,
    Forbidden,
    NotFound,
    Invalid(String),
    Failed,
}

impl ActionDenied {
    pub fn status(&self) -> u16 {
        match self {
            ActionDenied::Unauthenticated => 401,
            ActionDenied::Forbidden => 403,
            ActionDenied::NotFound => 404,
            ActionDenied::Invalid(_) => 400,
            ActionDenied::Failed => 500,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            ActionDenied::Unauthenticated => "UNAUTHENTICATED",
            ActionDenied::Forbidden => "FORBIDDEN",
            ActionDenied::NotFound => "NOT_FOUND",
            ActionDenied::Invalid(_) => "INVALID",
            ActionDenied::Failed => "ACTION_FAILED",
        }
    }

    pub fn message(&self) -> String {
        match self {
            ActionDenied::Unauthenticated => "Sign in required".into(),
            ActionDenied::Forbidden => "Not allowed".into(),
            ActionDenied::NotFound => "Not found".into(),
            ActionDenied::Invalid(m) => m.clone(),
            ActionDenied::Failed => "Action failed".into(),
        }
    }

    /// `authz::error_body` plus a toast effect for the client runtime.
    pub fn to_json(&self) -> Value {
        let mut body = crate::authz::error_body(self.code(), &self.message());
        body["ok"] = json!(false);
        body["effects"] = json!([{ "type": "toast", "target": self.message(), "style": "error" }]);
        body
    }
}

fn effect(effect_type: &str, target: &str, style: &str) -> ActionEffect {
    ActionEffect {
        effect_type: effect_type.into(),
        target: target.into(),
        value: String::new(),
        style: style.into(),
    }
}

fn set_body(entity: &EntityNode, target: &str, value: &str) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert(target.to_string(), Value::String(value.to_string()));
    crate::authz::writable_body(entity, &m)
}

/// Execute a declared action for `record_id`. Requires a session; `set` and
/// `delete` act on the declared entity only, on rows in the viewer's write
/// scope, and `set` targets must be writable fields.
pub fn execute_declared_action(
    action: &DeclaredAction,
    record_id: &str,
    db: &CronusDB,
    entities: &[EntityNode],
    access: &Access,
) -> Result<Vec<ActionEffect>, ActionDenied> {
    if access.viewer.is_none() {
        return Err(ActionDenied::Unauthenticated);
    }
    let instructions = &action.block.instructions;
    let touches_data = instructions
        .iter()
        .any(|i| i.verb == "set" || i.verb == "delete");

    let mut target: Option<(&EntityNode, WriteScope)> = None;
    if touches_data {
        let entity = entities
            .iter()
            .find(|e| !action.entity.is_empty() && e.name.eq_ignore_ascii_case(&action.entity))
            .ok_or(ActionDenied::Forbidden)?;
        if record_id.is_empty() {
            return Err(ActionDenied::Invalid("Missing record id".into()));
        }
        let scope = access::write_scope(access, &entity.name);
        if scope == WriteScope::Deny {
            return Err(ActionDenied::Forbidden);
        }
        for i in instructions.iter().filter(|i| i.verb == "set") {
            if let Some(err) = validate_field_value(&i.target, &i.value, entity) {
                return Err(ActionDenied::Invalid(err));
            }
            if set_body(entity, &i.target, &i.value).is_empty() {
                return Err(ActionDenied::Forbidden);
            }
        }
        let mut filters = vec![("id".to_string(), "=".to_string(), record_id.to_string())];
        if let Some((column, value)) = scope.condition() {
            filters.push((column.to_string(), "=".into(), value));
        }
        match db.find_one(&entity.name, &filters, None, None) {
            Ok(Some(_)) => {}
            Ok(None) => return Err(ActionDenied::NotFound),
            Err(e) => {
                eprintln!("  action lookup on {} failed: {}", entity.name, e);
                return Err(ActionDenied::Failed);
            }
        }
        target = Some((entity, scope));
    }

    let mut effects = Vec::new();
    for instr in instructions {
        let style = instr
            .modifiers
            .get("style")
            .map(|s| s.as_str())
            .unwrap_or("info");
        match instr.verb.as_str() {
            "set" => {
                if let Some((entity, scope)) = &target {
                    let body = set_body(entity, &instr.target, &instr.value);
                    match access::scoped_update(db, &entity.name, record_id, &body, scope) {
                        Ok(Some(_)) => {}
                        Ok(None) => return Err(ActionDenied::NotFound),
                        Err(e) => {
                            eprintln!("  action set on {} failed: {}", entity.name, e);
                            return Err(ActionDenied::Failed);
                        }
                    }
                }
            }
            "delete" => {
                if let Some((entity, scope)) = &target {
                    match access::scoped_delete(db, &entity.name, record_id, scope) {
                        Ok(true) => effects.push(effect("toast", "Deleted", "success")),
                        Ok(false) => return Err(ActionDenied::NotFound),
                        Err(e) => {
                            eprintln!("  action delete on {} failed: {}", entity.name, e);
                            return Err(ActionDenied::Failed);
                        }
                    }
                }
            }
            "toast" => effects.push(effect("toast", &instr.target, style)),
            "navigate" => effects.push(effect("navigate", &instr.target, "")),
            "refresh" => effects.push(effect("refresh", "", "")),
            "open" => effects.push(effect("open", &instr.target, "")),
            "close" => effects.push(effect("close", &instr.target, "")),
            _ => {}
        }
    }

    Ok(effects)
}

/// Convert effects list to JSON envelope for the client.
pub fn effects_to_json(effects: &[ActionEffect]) -> Value {
    let arr: Vec<Value> = effects
        .iter()
        .map(|e| {
            json!({
                "type": e.effect_type,
                "target": e.target,
                "value": e.value,
                "style": e.style,
            })
        })
        .collect();
    json!({ "ok": true, "effects": arr })
}

// ══════════════════════════════════════════════════
// HTTP HANDLERS (called from main.rs::handle_request_inner)
// ══════════════════════════════════════════════════

/// `POST /_action/<section>` with `{action_id, entity, id}`. Returns
/// `(status, body)`.
pub fn handle_action(state: &AppState, body: &Value, access: &Access) -> (u16, Value) {
    let field = |k: &str| body.get(k).and_then(Value::as_str).unwrap_or("");
    let deny = |d: ActionDenied| (d.status(), d.to_json());
    if access.viewer.is_none() {
        return deny(ActionDenied::Unauthenticated);
    }
    let declared = declared_actions(&state.pages);
    let Some(action) = find_declared_action(&declared, field("action_id"), field("entity")) else {
        return deny(ActionDenied::Forbidden);
    };
    match execute_declared_action(action, field("id"), &state.db, &state.entities, access) {
        Ok(effects) => (200, effects_to_json(&effects)),
        Err(d) => deny(d),
    }
}

fn form_error(status: u16, code: &str, message: &str) -> (u16, Value) {
    let mut body = crate::authz::error_body(code, message);
    body["ok"] = json!(false);
    body["effects"] = json!([{"type": "toast", "target": message, "style": "error"}]);
    (status, body)
}

fn form_denied(denial: Denial) -> (u16, Value) {
    match denial {
        Denial::Unauthenticated => form_error(401, "UNAUTHENTICATED", "Sign in required"),
        Denial::Forbidden => form_error(403, "FORBIDDEN", "Not allowed"),
    }
}

/// `/_form/<section_type>` (POST, create) and `/_form/<section_type>/<id>`
/// (PATCH, edit-mode forms) with `{entity, data}`. Both require a declared
/// form section bound to `entity`; the session/owner rules come from
/// `access.rs`; bodies go through `authz::writable_body`.
pub fn handle_form(
    state: &AppState,
    method: &hyper::Method,
    path: &str,
    body: &Value,
    access: &Access,
) -> (u16, Value) {
    let mut segments = path.trim_start_matches("/_form/").split('/');
    let section_type = segments.next().unwrap_or("");
    let record_id = segments.next().unwrap_or("");
    let entity = body.get("entity").and_then(Value::as_str).unwrap_or("");
    let data = body.get("data").cloned().unwrap_or(json!({}));

    let Some(form) = find_declared_form(&state.pages, section_type, entity) else {
        return form_error(403, "UNKNOWN_FORM", "Not allowed");
    };
    let Some(schema) = state.entities.iter().find(|e| e.name == form.entity) else {
        return form_error(403, "UNKNOWN_FORM", "Not allowed");
    };
    match *method {
        hyper::Method::POST => create_from_form(state, schema, &form, &data, access),
        hyper::Method::PATCH => update_from_form(state, schema, record_id, &data, access),
        _ => form_error(405, "METHOD_NOT_ALLOWED", "Not allowed"),
    }
}

fn create_from_form(
    state: &AppState,
    schema: &EntityNode,
    form: &DeclaredForm,
    data: &Value,
    access: &Access,
) -> (u16, Value) {
    let owner = match access::create_owner(access, &schema.name, form.public) {
        Ok(owner) => owner,
        Err(d) => return form_denied(d),
    };
    let Some(data_obj) = data.as_object() else {
        return form_error(400, "INVALID", "Missing form data");
    };
    let errors = validate_form_data(data, schema);
    if !errors.is_empty() {
        return (
            400,
            json!({
                "ok": false,
                "errors": errors,
                "effects": [{"type": "toast", "target": "Validation failed", "style": "error"}]
            }),
        );
    }
    let mut row_data = crate::authz::writable_body(schema, data_obj);
    if let Some(owner) = owner {
        row_data.insert("_owner_id".into(), json!(owner));
    }
    match state.db.insert(&schema.name, &Value::Object(row_data)) {
        Ok(row) => {
            let row_id = row
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            state.sse_hub.broadcast(crate::sse::DataChangeEvent {
                entity: schema.name.clone(),
                action: "created".to_string(),
                id: row_id,
            });
            (
                201,
                json!({
                    "ok": true,
                    "id": row.get("id"),
                    "effects": [{"type": "toast", "target": "Created successfully", "style": "success"}]
                }),
            )
        }
        Err(e) => {
            eprintln!("  /_form insert into {} failed: {}", schema.name, e);
            form_error(400, "CREATE_FAILED", "Could not save")
        }
    }
}

fn update_from_form(
    state: &AppState,
    schema: &EntityNode,
    record_id: &str,
    data: &Value,
    access: &Access,
) -> (u16, Value) {
    let scope = access::write_scope(access, &schema.name);
    if scope == WriteScope::Deny {
        return form_denied(if access.viewer.is_none() {
            Denial::Unauthenticated
        } else {
            Denial::Forbidden
        });
    }
    if record_id.is_empty() {
        return form_error(404, "NOT_FOUND", "Not found");
    }
    let Some(data_obj) = data.as_object() else {
        return form_error(400, "INVALID", "Missing form data");
    };

    // Partial update: validate only the writable fields being sent.
    let changes: Map<String, Value> = crate::authz::writable_body(schema, data_obj)
        .into_iter()
        .filter(|(_, v)| !v.is_null())
        .collect();
    let errors: std::collections::HashMap<String, String> = changes
        .iter()
        .filter_map(|(k, v)| {
            let text = v
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| v.to_string());
            validate_field_value(k, &text, schema).map(|e| (k.clone(), e))
        })
        .collect();
    if !errors.is_empty() {
        return (
            400,
            json!({
                "ok": false,
                "errors": errors,
                "effects": [{"type": "toast", "target": "Validation failed", "style": "error"}]
            }),
        );
    }
    if changes.is_empty() {
        return form_error(400, "INVALID", "Nothing to update");
    }

    let mut filters = vec![("id".to_string(), "=".to_string(), record_id.to_string())];
    if let Some((column, value)) = scope.condition() {
        filters.push((column.to_string(), "=".into(), value));
    }
    let prev = match state.db.find_one(&schema.name, &filters, None, None) {
        Ok(Some(row)) => row,
        Ok(None) => return form_error(404, "NOT_FOUND", "Not found"),
        Err(e) => {
            eprintln!("  /_form lookup on {} failed: {}", schema.name, e);
            return form_error(500, "UPDATE_FAILED", "Could not save");
        }
    };
    if !schema.transitions.is_empty() {
        if let Err(mut err) =
            crate::validate_transitions(schema, &Value::Object(changes.clone()), &prev)
        {
            err["ok"] = json!(false);
            return (409, err);
        }
    }

    match access::scoped_update(&state.db, &schema.name, record_id, &changes, &scope) {
        Ok(Some(mut row)) => {
            crate::authz::redact_sensitive(schema, &mut row);
            state.sse_hub.broadcast(crate::sse::DataChangeEvent {
                entity: schema.name.clone(),
                action: "updated".to_string(),
                id: record_id.to_string(),
            });
            (
                200,
                json!({
                    "ok": true,
                    "id": record_id,
                    "record": row,
                    "effects": [{"type": "toast", "target": "Updated successfully", "style": "success"}]
                }),
            )
        }
        Ok(None) => form_error(404, "NOT_FOUND", "Not found"),
        Err(e) => {
            eprintln!("  /_form update on {} failed: {}", schema.name, e);
            form_error(400, "UPDATE_FAILED", "Could not save")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::access::test_support::*;
    use crate::parser::{parse, AstNode};

    const PAGES: &str = "app \"T\" { port 5175 }\n\
page \"/notes\" type:custom requires:auth {\n\
  section card {\n\
    bind Note { query all }\n\
    on click {\n\
      set title \"archived\"\n\
      toast \"Archived\" success\n\
    }\n\
  }\n\
  section form {\n\
    bind Note { query all }\n\
    on click {\n\
      set secret \"x\"\n\
    }\n\
  }\n\
}\n\
page \"/contact\" type:custom {\n\
  section form {\n\
    bind Tag { query all scope:public }\n\
  }\n\
}\n";

    fn pages() -> Vec<PageNode> {
        parse(PAGES)
            .expect("parse")
            .into_iter()
            .filter_map(|n| match n {
                AstNode::Page(p) => Some(p),
                _ => None,
            })
            .collect()
    }

    fn archive(declared: &[DeclaredAction]) -> &DeclaredAction {
        declared
            .iter()
            .find(|a| a.block.instructions.iter().any(|i| i.target == "title"))
            .unwrap_or_else(|| panic!("archive action not declared: {:?}", declared))
    }

    #[test]
    fn undeclared_action_is_refused() {
        let declared = declared_actions(&pages());
        assert!(find_declared_action(&declared, "", "Note").is_none());
        assert!(find_declared_action(&declared, "deadbeefdeadbeef", "Note").is_none());
        let legit = &archive(&declared).id;
        assert!(find_declared_action(&declared, legit, "Note").is_some());
        assert!(
            find_declared_action(&declared, legit, "User").is_none(),
            "entity must match the declaration"
        );
    }

    #[test]
    fn client_instructions_are_ignored() {
        let ents = entities();
        let db = db(&ents);
        let id = insert_note(&db, "alice", "draft");
        let declared = declared_actions(&pages());
        let action =
            find_declared_action(&declared, &archive(&declared).id, "Note").expect("declared");
        let effects =
            execute_declared_action(action, &id, &db, &ents, &as_user("alice")).expect("ok");
        assert_eq!(
            db.find_by_id("Note", &id).unwrap().unwrap()["title"],
            "archived"
        );
        assert_eq!(effects[0].target, "Archived");
    }

    #[test]
    fn actions_require_session_owner_and_writable_fields() {
        let ents = entities();
        let db = db(&ents);
        let id = insert_note(&db, "alice", "draft");
        let declared = declared_actions(&pages());
        let a = archive(&declared);
        assert_eq!(
            execute_declared_action(a, &id, &db, &ents, &anon()).unwrap_err(),
            ActionDenied::Unauthenticated
        );
        assert_eq!(
            execute_declared_action(a, &id, &db, &ents, &as_user("bob")).unwrap_err(),
            ActionDenied::NotFound
        );
        assert_eq!(
            db.find_by_id("Note", &id).unwrap().unwrap()["title"],
            "draft"
        );
        let secret = declared
            .iter()
            .find(|a| a.block.instructions.iter().any(|i| i.target == "secret"))
            .expect("secret action");
        assert_eq!(
            execute_declared_action(secret, &id, &db, &ents, &as_user("alice")).unwrap_err(),
            ActionDenied::Forbidden
        );
    }

    // ── HTTP-level: handle_form / handle_action against AppState ──

    const APP: &str = "app \"F\" { port 5175 }\n\
auth { entity User  login email + password  session jwt  roles [admin, user] }\n\
entity User { name string  email email!  role string  password string sensitive }\n\
entity Note { title string!  status string  secret string sensitive  role string }\n\
entity Post { title string! }\n\
page \"/notes/:id\" type:custom requires:auth {\n\
  section form { bind Note { query one where id eq:route.id } }\n\
  section card {\n\
    bind Note { query all }\n\
    item \"Archive\" {\n\
      on click {\n\
        set status \"archived\"\n\
        toast \"Archived\" success\n\
      }\n\
    }\n\
  }\n\
}\n";

    fn app_state() -> crate::server::state::AppState {
        let mut state = crate::api_security_tests::state_from(APP);
        state.pages = parse(APP)
            .expect("parse")
            .into_iter()
            .filter_map(|n| match n {
                AstNode::Page(p) => Some(p),
                _ => None,
            })
            .collect();
        state
    }

    fn note(state: &crate::server::state::AppState, owner: &str) -> String {
        state
            .db
            .insert(
                "Note",
                &json!({"title": "draft", "secret": "s3cret", "_owner_id": owner}),
            )
            .expect("insert")["id"]
            .as_str()
            .expect("id")
            .to_string()
    }

    fn patch(
        state: &crate::server::state::AppState,
        id: &str,
        entity: &str,
        data: Value,
        access: &Access,
    ) -> (u16, Value) {
        handle_form(
            state,
            &hyper::Method::PATCH,
            &format!("/_form/form/{id}"),
            &json!({"entity": entity, "data": data}),
            access,
        )
    }

    #[test]
    fn form_patch_updates_only_the_owners_row_with_writable_fields() {
        let s = app_state();
        let id = note(&s, "alice");
        let hostile = json!({
            "title": "edited", "role": "admin", "secret": "leak",
            "_owner_id": "bob", "id": "other", "bogus": 1
        });

        let (status, body) = patch(&s, &id, "Note", hostile.clone(), &as_user("alice"));
        assert_eq!(status, 200, "{body}");
        assert_eq!(body["record"]["title"], "edited");
        assert!(
            body["record"].get("secret").is_none(),
            "response redacted: {body}"
        );
        let row = s.db.find_by_id("Note", &id).unwrap().unwrap();
        assert_eq!(row["title"], "edited");
        assert_eq!(row["secret"], "s3cret", "sensitive field ignored");
        assert_eq!(row["_owner_id"], "alice", "owner not writable");
        assert!(row["role"].is_null(), "privileged field ignored: {row}");

        let (status, _) = patch(&s, &id, "Note", json!({"title": "bob"}), &as_user("bob"));
        assert_eq!(status, 404, "other user's row is not found");
        let (status, body) = patch(&s, &id, "Note", json!({"title": "anon"}), &anon());
        assert_eq!(status, 401, "{body}");
        assert_eq!(body["error"]["code"], "UNAUTHENTICATED");
        let (status, body) = patch(&s, &id, "Post", json!({"title": "x"}), &as_user("alice"));
        assert_eq!(status, 403, "undeclared entity: {body}");
        assert_eq!(body["error"]["code"], "UNKNOWN_FORM");
        let (status, _) = patch(&s, &id, "Note", json!({"role": "admin"}), &as_user("alice"));
        assert_eq!(status, 400, "only privileged fields → nothing to update");

        assert_eq!(
            s.db.find_by_id("Note", &id).unwrap().unwrap()["title"],
            "edited"
        );
        let (status, _) = patch(&s, &id, "Note", json!({"title": "root"}), &as_admin());
        assert_eq!(status, 200, "admin edits any row");
    }

    #[test]
    fn form_post_still_creates_owned_rows() {
        let s = app_state();
        let (status, body) = handle_form(
            &s,
            &hyper::Method::POST,
            "/_form/form",
            &json!({"entity": "Note", "data": {"title": "new", "role": "admin"}}),
            &as_user("alice"),
        );
        assert_eq!(status, 201, "{body}");
        let row =
            s.db.find_by_id("Note", body["id"].as_str().unwrap())
                .unwrap()
                .unwrap();
        assert_eq!(row["_owner_id"], "alice");
        assert!(row["role"].is_null());
    }

    #[test]
    fn rendered_button_carries_the_declared_action_id_and_runs_by_id_only() {
        let mut s = app_state();
        let id = note(&s, "alice");
        // The generic item parser stores `on click` as JSON; mark it as an
        // action item the card renderer turns into a button.
        let card = &mut s.pages[0].sections[1];
        card.items[0].insert("_type".into(), "action".into());
        let html =
            crate::ui::render_section(card, "amber", "light", &crate::binding::ResolvedData::None);
        let marker = "data-action-id=\"";
        let start = html.find(marker).expect("button has data-action-id") + marker.len();
        let rendered_id = html[start..start + 16].to_string();
        let declared = declared_actions(&s.pages);
        assert!(
            declared
                .iter()
                .any(|a| a.id == rendered_id && a.entity == "Note"),
            "renderer and server agree on the id"
        );

        // Runtime payload: {action_id, entity, id}.
        let (status, body) = handle_action(
            &s,
            &json!({"action_id": rendered_id, "entity": "Note", "id": id}),
            &as_user("alice"),
        );
        assert_eq!(status, 200, "{body}");
        assert_eq!(
            s.db.find_by_id("Note", &id).unwrap().unwrap()["status"],
            "archived"
        );

        // Client-sent instruction JSON is no longer accepted.
        let block = s.pages[0].sections[1].items[0]["on_click"].clone();
        let (status, _) = handle_action(
            &s,
            &json!({"action": block, "entity": "Note", "id": id}),
            &as_user("alice"),
        );
        assert_eq!(status, 403);
        let (status, _) = handle_action(
            &s,
            &json!({"action_id": rendered_id, "entity": "Note", "id": id}),
            &anon(),
        );
        assert_eq!(status, 401);
    }

    #[test]
    fn forms_must_be_declared_and_public_only_when_explicit() {
        let p = pages();
        assert_eq!(
            find_declared_form(&p, "form", "note"),
            Some(DeclaredForm {
                entity: "Note".into(),
                public: false
            })
        );
        assert_eq!(
            find_declared_form(&p, "form", "Tag"),
            Some(DeclaredForm {
                entity: "Tag".into(),
                public: true
            })
        );
        assert_eq!(find_declared_form(&p, "form", "User"), None);
        assert_eq!(find_declared_form(&p, "hero", "Note"), None);
    }
}
