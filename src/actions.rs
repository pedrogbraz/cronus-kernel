// CRONUS Action Execution Engine
//
// Processes action instructions from button clicks / form submits.
// Returns an effect envelope (toast, navigate, refresh, etc.)
//
// SECURITY: the client never supplies instructions. Kernel-rendered buttons
// carry `data-action-id`; the runtime posts `{action_id, entity, id}` and the
// server executes its own copy of the declared block. `create`/`update` field
// values are AST literals. Writes are owner-scoped in SQL (`access::scoped_*`).
// `/_form` creates and edits go through `handle_form`; `on submit` supplies
// toast/navigate only and is refused on `/_action`.

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use crate::access::{self, Access, Denial, WriteScope};
use crate::database::CronusDB;
use crate::parser::{ActionBlock, EntityNode, PageNode, SectionNode};
use crate::relations;
use crate::server::state::AppState;
use crate::validation::{self, FieldErrors, Mode};

#[derive(Debug)]
pub struct ActionEffect {
    pub effect_type: String, // "toast", "navigate", "refresh", "open", "close"
    pub target: String,
    pub value: String,
    pub style: String,
}

/// Validate one `set` value against the field's type and constraints
/// (`validation::value_errors`). `None` when valid; otherwise the first
/// message as `'field' message`. Empty values are accepted, as before.
pub fn validate_field_value(field_name: &str, value: &str, entity: &EntityNode) -> Option<String> {
    let Some(field) = entity
        .fields
        .iter()
        .find(|f| f.name.eq_ignore_ascii_case(field_name))
    else {
        return Some(format!(
            "Field '{}' does not exist on entity '{}'",
            field_name, entity.name
        ));
    };
    if field.is_many() {
        return None;
    }
    if value.is_empty() {
        return None;
    }
    validation::value_errors(field, &Value::String(value.to_string()))
        .into_iter()
        .next()
        .map(|message| format!("'{}' {}", field.name, message))
}

/// `/_form` validation failure: the form envelope (`ok`, the legacy
/// `errors` map the runtime renders under each input, a toast) plus
/// `error.code`/`error.message`/`error.fields` from `authz`.
fn form_invalid(status: u16, errors: &FieldErrors) -> (u16, Value) {
    let mut body = validation::error_body(errors);
    body["ok"] = json!(false);
    body["errors"] = json!(validation::first_messages(errors));
    body["effects"] = json!([{"type": "toast", "target": "Validation failed", "style": "error"}]);
    (status, body)
}

/// Writable form body split into non-null columns and many-to-many ids
/// (`"a,b"` strings from HTML inputs become arrays).
fn form_parts(
    schema: &EntityNode,
    data: &Map<String, Value>,
) -> (Map<String, Value>, Map<String, Value>) {
    let mut body = crate::authz::writable_body(schema, data);
    relations::normalize_form_values(schema, &mut body);
    let (columns, many) = relations::split(schema, body);
    let columns = columns.into_iter().filter(|(_, v)| !v.is_null()).collect();
    (columns, many)
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
    /// `on submit { … }` on the form section. Write verbs are ignored here
    /// (`/_form` already wrote); toast/navigate/refresh are returned to the client.
    pub submit: Option<ActionBlock>,
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
            let submit = section
                .actions
                .iter()
                .find(|a| a.event == "submit")
                .cloned();
            match found.as_mut() {
                Some(f) => {
                    f.public |= public;
                    if f.submit.is_none() {
                        f.submit = submit;
                    }
                }
                None => {
                    found = Some(DeclaredForm {
                        entity: declared,
                        public,
                        submit,
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

fn ui_effects(block: &ActionBlock) -> Vec<ActionEffect> {
    let mut out = Vec::new();
    for instr in &block.instructions {
        let style = instr
            .modifiers
            .get("style")
            .map(|s| s.as_str())
            .unwrap_or("info");
        match instr.verb.as_str() {
            "toast" => out.push(effect("toast", &instr.target, style)),
            "navigate" => out.push(effect("navigate", &instr.target, "")),
            "refresh" => out.push(effect("refresh", "", "")),
            "open" => out.push(effect("open", &instr.target, "")),
            "close" => out.push(effect("close", &instr.target, "")),
            _ => {}
        }
    }
    out
}

fn effect_json_list(effects: &[ActionEffect]) -> Vec<Value> {
    effects
        .iter()
        .map(|e| {
            json!({
                "type": e.effect_type,
                "target": e.target,
                "value": e.value,
                "style": e.style,
            })
        })
        .collect()
}

fn form_success_effects(form: &DeclaredForm, fallback: &str) -> Vec<Value> {
    let fx = form.submit.as_ref().map(ui_effects).unwrap_or_default();
    if fx.is_empty() {
        vec![json!({"type": "toast", "target": fallback, "style": "success"})]
    } else {
        effect_json_list(&fx)
    }
}

fn after_entity_write(
    state: &AppState,
    schema: &EntityNode,
    event: &str,
    row: &Value,
    prev: Option<&Value>,
    access: &Access,
) {
    let table = schema.name.as_str();
    let row_id = row.get("id").and_then(Value::as_str).unwrap_or("");
    let owner = access.viewer.as_ref().map(|v| v.id.as_str()).unwrap_or("");
    let role = access
        .viewer
        .as_ref()
        .map(|v| v.role.as_str())
        .unwrap_or("");
    crate::effects::fire_webhooks(&state.webhooks, &state.entities, table, event, row);
    crate::effects::fire_effects(schema, event, row, prev, &state.brain, &state.sse_hub);
    crate::scripting::fire_scripts(
        &state.script_registry,
        table,
        event,
        row,
        row_id,
        prev,
        &state.db,
        owner,
        role,
        &std::collections::HashMap::new(),
    );
}

fn action_entity<'a>(
    action: &DeclaredAction,
    entities: &'a [EntityNode],
) -> Result<&'a EntityNode, ActionDenied> {
    entities
        .iter()
        .find(|e| !action.entity.is_empty() && e.name.eq_ignore_ascii_case(&action.entity))
        .ok_or(ActionDenied::Forbidden)
}

fn named_entity_matches(action: &DeclaredAction, name: &str) -> bool {
    name.is_empty() || name.eq_ignore_ascii_case(&action.entity)
}

fn pairs_to_body(entity: &EntityNode, pairs: &[(&str, &str)]) -> Map<String, Value> {
    let mut m = Map::new();
    for (k, v) in pairs {
        m.insert((*k).to_string(), json!(*v));
    }
    let mut m = crate::authz::writable_body(entity, &m);
    relations::normalize_form_values(entity, &mut m);
    m
}

fn check_write_denied(
    db: &CronusDB,
    entities: &[EntityNode],
    entity: &EntityNode,
    columns: &Map<String, Value>,
    many: &Map<String, Value>,
    mode: Mode,
    exclude_id: Option<&str>,
    access: &Access,
) -> Result<Vec<crate::relations::LinkSet>, ActionDenied> {
    match validation::check_write(
        db, entities, entity, columns, many, mode, exclude_id, access,
    ) {
        Ok(links) => Ok(links),
        Err((_, errors)) => {
            let msg = errors
                .iter()
                .next()
                .map(|(field, msgs)| {
                    format!("'{}' {}", field, msgs.first().cloned().unwrap_or_default())
                })
                .unwrap_or_else(|| "Invalid".into());
            Err(ActionDenied::Invalid(msg))
        }
    }
}

fn lookup_owned(
    db: &CronusDB,
    entity: &EntityNode,
    record_id: &str,
    scope: &WriteScope,
) -> Result<Value, ActionDenied> {
    if record_id.is_empty() {
        return Err(ActionDenied::Invalid("Missing record id".into()));
    }
    let mut filters = vec![crate::database::SqlFilter::one("id", "=", record_id)];
    if let Some((column, value)) = scope.condition() {
        filters.push(crate::database::SqlFilter::one(column, "=", value));
    }
    match db.find_one(&entity.name, &filters, None, None) {
        Ok(Some(row)) => Ok(row),
        Ok(None) => Err(ActionDenied::NotFound),
        Err(e) => {
            eprintln!("  action lookup on {} failed: {}", entity.name, e);
            Err(ActionDenied::Failed)
        }
    }
}

/// Execute a declared action for `record_id`. Requires a session. `create` uses
/// AST field literals (no client body). `update`/`set`/`delete` act on the
/// bound entity, owner-scoped. Submit blocks are not executed here (`/_form`).
pub fn execute_declared_action(
    action: &DeclaredAction,
    record_id: &str,
    db: &CronusDB,
    entities: &[EntityNode],
    access: &Access,
    state: Option<&AppState>,
) -> Result<Vec<ActionEffect>, ActionDenied> {
    if access.viewer.is_none() {
        return Err(ActionDenied::Unauthenticated);
    }
    let instructions = &action.block.instructions;
    let is_submit = action.block.event.eq_ignore_ascii_case("submit");
    let mut effects = Vec::new();

    let writes_existing = instructions
        .iter()
        .any(|i| i.verb == "set" || i.verb == "delete" || (i.verb == "update" && !is_submit));
    let creates = !is_submit && instructions.iter().any(|i| i.verb == "create");

    let entity = if writes_existing || creates {
        Some(action_entity(action, entities)?)
    } else {
        None
    };

    if creates {
        let entity = entity.ok_or(ActionDenied::Forbidden)?;
        let mut fields: Vec<(&str, &str)> = Vec::new();
        for instr in instructions.iter().filter(|i| i.verb == "create") {
            if !named_entity_matches(action, &instr.target) {
                return Err(ActionDenied::Forbidden);
            }
            for (k, v) in &instr.modifiers {
                fields.push((k.as_str(), v.as_str()));
            }
        }
        if fields.is_empty() {
            return Err(ActionDenied::Invalid(
                "create requires field values in the action block".into(),
            ));
        }
        let owner = match access::create_owner(access, &entity.name, false) {
            Ok(owner) => owner,
            Err(Denial::Unauthenticated) => return Err(ActionDenied::Unauthenticated),
            Err(Denial::Forbidden) => return Err(ActionDenied::Forbidden),
        };
        let mut body = pairs_to_body(entity, &fields);
        for field in entity.fields.iter().filter(|f| !f.is_many()) {
            if let Some(default) = &field.default_value {
                if !body.contains_key(&field.name) {
                    body.insert(field.name.clone(), Value::String(default.clone()));
                }
            }
        }
        let (columns, many) = relations::split(entity, body);
        let links = check_write_denied(
            db,
            entities,
            entity,
            &columns,
            &many,
            Mode::Create,
            None,
            access,
        )?;
        let mut row_data = columns;
        if let Some(owner) = &owner {
            row_data.insert("_owner_id".into(), json!(owner));
        }
        match db.insert_with(&entity.name, &Value::Object(row_data), |conn, id| {
            links.iter().try_for_each(|link| {
                relations::replace_links(conn, entity, link, id, owner.as_deref())
            })
        }) {
            Ok(row) => {
                if let Some(state) = state {
                    after_entity_write(state, entity, "create", &row, None, access);
                }
            }
            Err(e) => {
                eprintln!("  action create on {} failed: {}", entity.name, e);
                return Err(ActionDenied::Failed);
            }
        }
    }

    if writes_existing {
        let entity = entity.ok_or(ActionDenied::Forbidden)?;
        let scope = access::write_scope(access, &entity.name);
        if scope == WriteScope::Deny {
            return Err(ActionDenied::Forbidden);
        }
        let prev = lookup_owned(db, entity, record_id, &scope)?;
        let mut pairs: Vec<(&str, &str)> = Vec::new();
        for instr in instructions {
            if instr.verb == "set" {
                if let Some(err) = validate_field_value(&instr.target, &instr.value, entity) {
                    return Err(ActionDenied::Invalid(err));
                }
                pairs.push((instr.target.as_str(), instr.value.as_str()));
            } else if instr.verb == "update" && !is_submit {
                if !named_entity_matches(action, &instr.target) {
                    return Err(ActionDenied::Forbidden);
                }
                for (k, v) in &instr.modifiers {
                    pairs.push((k.as_str(), v.as_str()));
                }
            }
        }
        if !pairs.is_empty() {
            let body = pairs_to_body(entity, &pairs);
            if body.is_empty() {
                return Err(ActionDenied::Forbidden);
            }
            let (columns, many) = relations::split(entity, body);
            let links = check_write_denied(
                db,
                entities,
                entity,
                &columns,
                &many,
                Mode::Update,
                Some(record_id),
                access,
            )?;
            if !columns.is_empty() {
                match access::scoped_update(db, &entity.name, record_id, &columns, &scope) {
                    Ok(Some(_)) => {}
                    Ok(None) => return Err(ActionDenied::NotFound),
                    Err(e) => {
                        eprintln!("  action set on {} failed: {}", entity.name, e);
                        return Err(ActionDenied::Failed);
                    }
                }
            }
            if !links.is_empty() {
                let parent_owner = prev.get("_owner_id").and_then(Value::as_str);
                if let Err(e) = db.transaction(|conn| {
                    links.iter().try_for_each(|link| {
                        relations::replace_links(conn, entity, link, record_id, parent_owner)
                    })
                }) {
                    eprintln!("  action links on {} failed: {}", entity.name, e);
                    return Err(ActionDenied::Failed);
                }
            }
            if let Some(state) = state {
                if let Ok(Some(row)) = db.find_by_id(&entity.name, record_id) {
                    after_entity_write(state, entity, "update", &row, Some(&prev), access);
                }
            }
        }
        for instr in instructions.iter().filter(|i| i.verb == "delete") {
            if !named_entity_matches(action, &instr.target) {
                return Err(ActionDenied::Forbidden);
            }
            match access::scoped_delete(db, &entity.name, record_id, &scope) {
                Ok(true) => {
                    effects.push(effect("toast", "Deleted", "success"));
                    if let Some(state) = state {
                        after_entity_write(state, entity, "delete", &prev, Some(&prev), access);
                    }
                }
                Ok(false) => return Err(ActionDenied::NotFound),
                Err(e) => {
                    eprintln!("  action delete on {} failed: {}", entity.name, e);
                    return Err(ActionDenied::Failed);
                }
            }
        }
    }

    effects.extend(ui_effects(&action.block));
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
    if action.block.event.eq_ignore_ascii_case("submit") {
        return deny(ActionDenied::Forbidden);
    }
    match execute_declared_action(
        action,
        field("id"),
        &state.db,
        &state.entities,
        access,
        Some(state),
    ) {
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
        hyper::Method::PATCH => update_from_form(state, schema, record_id, &form, &data, access),
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
    let (mut row_data, many) = form_parts(schema, data_obj);
    for field in schema.fields.iter().filter(|f| !f.is_many()) {
        if let Some(default) = &field.default_value {
            if !row_data.contains_key(&field.name) {
                row_data.insert(field.name.clone(), Value::String(default.clone()));
            }
        }
    }
    let links = match validation::check_write(
        &state.db,
        &state.entities,
        schema,
        &row_data,
        &many,
        Mode::Create,
        None,
        access,
    ) {
        Ok(links) => links,
        Err((status, errors)) => return form_invalid(status, &errors),
    };
    if let Some(owner) = &owner {
        row_data.insert("_owner_id".into(), json!(owner));
    }
    match state
        .db
        .insert_with(&schema.name, &Value::Object(row_data), |conn, id| {
            links.iter().try_for_each(|link| {
                relations::replace_links(conn, schema, link, id, owner.as_deref())
            })
        }) {
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
            after_entity_write(state, schema, "create", &row, None, access);
            (
                201,
                json!({
                    "ok": true,
                    "id": row.get("id"),
                    "effects": form_success_effects(form, "Created successfully")
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
    form: &DeclaredForm,
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
    let (changes, many) = form_parts(schema, data_obj);
    if changes.is_empty() && many.is_empty() {
        return form_error(400, "INVALID", "Nothing to update");
    }

    let mut filters = vec![crate::database::SqlFilter::one("id", "=", record_id)];
    if let Some((column, value)) = scope.condition() {
        filters.push(crate::database::SqlFilter::one(column, "=", value));
    }
    let prev = match state.db.find_one(&schema.name, &filters, None, None) {
        Ok(Some(row)) => row,
        Ok(None) => return form_error(404, "NOT_FOUND", "Not found"),
        Err(e) => {
            eprintln!("  /_form lookup on {} failed: {}", schema.name, e);
            return form_error(500, "UPDATE_FAILED", "Could not save");
        }
    };
    let links = match validation::check_write(
        &state.db,
        &state.entities,
        schema,
        &changes,
        &many,
        Mode::Update,
        Some(record_id),
        access,
    ) {
        Ok(links) => links,
        Err((status, errors)) => return form_invalid(status, &errors),
    };
    if !schema.transitions.is_empty() {
        if let Err(mut err) =
            crate::effects::validate_transitions(schema, &Value::Object(changes.clone()), &prev)
        {
            err["ok"] = json!(false);
            return (409, err);
        }
    }

    let updated = if changes.is_empty() {
        Ok(Some(prev.clone()))
    } else {
        access::scoped_update(&state.db, &schema.name, record_id, &changes, &scope)
    };
    // Join rows keep the parent's owner, whoever edits the links.
    let parent_owner = prev.get("_owner_id").and_then(Value::as_str);
    let updated = updated.and_then(|row| match (row, links.is_empty()) {
        (Some(row), false) => state
            .db
            .transaction(|conn| {
                links.iter().try_for_each(|link| {
                    relations::replace_links(conn, schema, link, record_id, parent_owner)
                })
            })
            .map(|_| Some(row)),
        (row, _) => Ok(row),
    });

    match updated {
        Ok(Some(mut row)) => {
            crate::authz::redact_sensitive(schema, &mut row);
            relations::attach(&state.db, &state.entities, schema, &mut row, access, &[]);
            state.sse_hub.broadcast(crate::sse::DataChangeEvent {
                entity: schema.name.clone(),
                action: "updated".to_string(),
                id: record_id.to_string(),
            });
            after_entity_write(state, schema, "update", &row, Some(&prev), access);
            (
                200,
                json!({
                    "ok": true,
                    "id": record_id,
                    "record": row,
                    "effects": form_success_effects(form, "Updated successfully")
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
            execute_declared_action(action, &id, &db, &ents, &as_user("alice"), None).expect("ok");
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
            execute_declared_action(a, &id, &db, &ents, &anon(), None).unwrap_err(),
            ActionDenied::Unauthenticated
        );
        assert_eq!(
            execute_declared_action(a, &id, &db, &ents, &as_user("bob"), None).unwrap_err(),
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
            execute_declared_action(secret, &id, &db, &ents, &as_user("alice"), None).unwrap_err(),
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

    /// Real SSR path: `render_page` resolves the bindings against the DB for
    /// a logged-in viewer; every action button inside a bound card/table row
    /// carries that row's id, and the runtime payload built from the rendered
    /// attributes runs `set`/`delete` for the owner only.
    #[test]
    fn bound_row_action_buttons_carry_record_id_and_stay_owner_scoped() {
        const SRC: &str = "app \"F\" { port 5175 }\n\
auth { entity User  login email + password  session jwt  roles [admin, user] }\n\
entity User { name string  email email!  role string  password string sensitive }\n\
entity Note { title string!  status string  secret string sensitive  role string }\n\
page \"/notes\" type:custom requires:auth {\n\
  section card {\n\
    bind Note { query all }\n\
    item \"Archive\" {\n\
      on click {\n\
        set status \"archived\"\n\
      }\n\
    }\n\
  }\n\
  section table {\n\
    bind Note { query all }\n\
    columns \"title, status\"\n\
    item \"Remove\" {\n\
      on click {\n\
        delete Note\n\
      }\n\
    }\n\
  }\n\
}\n";
        let mut s = crate::api_security_tests::state_from(SRC);
        s.pages = parse(SRC)
            .expect("parse")
            .into_iter()
            .filter_map(|n| match n {
                AstNode::Page(p) => Some(p),
                _ => None,
            })
            .collect();
        let alice_note = note(&s, "alice");
        let bob_note = note(&s, "bob");

        let html = crate::ui::render_page(
            &s.pages[0],
            &s.entities,
            "amber",
            "light",
            Some(&s.db),
            &std::collections::HashMap::new(),
            &as_user("alice"),
        );
        let attr = |name: &str, button: &str| -> String {
            let marker = format!(" {name}=\"");
            button
                .find(&marker)
                .map(|i| &button[i + marker.len()..])
                .and_then(|rest| rest.split('"').next())
                .unwrap_or_default()
                .to_string()
        };
        let buttons: Vec<&str> = html
            .split("<button type=\"button\" data-cronus-action=")
            .skip(1)
            .map(|b| b.split('>').next().unwrap())
            .collect();
        assert_eq!(buttons.len(), 2, "one card + one table button: {html}");
        for b in &buttons {
            assert_eq!(attr("data-cronus-id", b), alice_note, "row id on {b}");
            assert_eq!(attr("data-cronus-entity", b), "Note");
            assert!(!attr("data-action-id", b).is_empty());
        }
        assert!(!html.contains(&bob_note), "other owner's row not rendered");

        // Runtime payload from the rendered attributes.
        let payload = |b: &str| {
            json!({
                "action_id": attr("data-action-id", b),
                "entity": attr("data-cronus-entity", b),
                "id": attr("data-cronus-id", b),
            })
        };
        let (archive, remove) = (payload(buttons[0]), payload(buttons[1]));

        // Another viewer replaying alice's ids: not found, nothing changes.
        let (status, body) = handle_action(&s, &archive, &as_user("bob"));
        assert_eq!(status, 404, "{body}");
        let (status, body) = handle_action(&s, &remove, &as_user("bob"));
        assert_eq!(status, 404, "{body}");
        let row = s.db.find_by_id("Note", &alice_note).unwrap().unwrap();
        assert!(row["status"].is_null(), "{row}");

        // Owner: set, then delete.
        let (status, body) = handle_action(&s, &archive, &as_user("alice"));
        assert_eq!(status, 200, "{body}");
        assert_eq!(
            s.db.find_by_id("Note", &alice_note).unwrap().unwrap()["status"],
            "archived"
        );
        let (status, body) = handle_action(&s, &remove, &as_user("alice"));
        assert_eq!(status, 200, "{body}");
        assert!(s.db.find_by_id("Note", &alice_note).unwrap().is_none());
        assert!(s.db.find_by_id("Note", &bob_note).unwrap().is_some());

        // Without the id (the pre-fix payload) the action is rejected.
        let mut missing = archive.clone();
        missing["id"] = json!("");
        let (status, _) = handle_action(&s, &missing, &as_user("bob"));
        assert_eq!(status, 400);
    }

    #[test]
    fn forms_must_be_declared_and_public_only_when_explicit() {
        let p = pages();
        assert_eq!(
            find_declared_form(&p, "form", "note"),
            Some(DeclaredForm {
                entity: "Note".into(),
                public: false,
                submit: None,
            })
        );
        assert_eq!(
            find_declared_form(&p, "form", "Tag"),
            Some(DeclaredForm {
                entity: "Tag".into(),
                public: true,
                submit: None,
            })
        );
        assert_eq!(find_declared_form(&p, "form", "User"), None);
        assert_eq!(find_declared_form(&p, "hero", "Note"), None);
    }

    // ── Field validation + many-to-many on /_form and /_action ──

    fn validation_state() -> crate::server::state::AppState {
        let mut s = crate::api_validation_tests::state();
        s.pages = parse(crate::api_validation_tests::SRC)
            .expect("parse")
            .into_iter()
            .filter_map(|n| match n {
                AstNode::Page(p) => Some(p),
                _ => None,
            })
            .collect();
        s
    }

    fn post_form(s: &crate::server::state::AppState, data: Value, who: &Access) -> (u16, Value) {
        handle_form(
            s,
            &hyper::Method::POST,
            "/_form/form",
            &json!({"entity": "Post", "data": data}),
            who,
        )
    }

    fn row_id(s: &crate::server::state::AppState, table: &str, v: Value) -> String {
        s.db.insert(table, &v).expect("insert")["id"]
            .as_str()
            .expect("id")
            .to_string()
    }

    #[test]
    fn form_create_returns_field_errors_in_the_form_envelope() {
        let s = validation_state();
        let alice = crate::api_validation_tests::viewer("alice", "user");
        let (status, body) = post_form(&s, json!({"title": "ab", "age": "200"}), &alice);
        assert_eq!(status, 422, "{body}");
        assert_eq!(body["ok"], false);
        assert_eq!(body["errors"]["title"], "must be at least 3 characters");
        assert_eq!(body["error"]["code"], "VALIDATION_FAILED");
        assert_eq!(
            body["error"]["fields"]["age"],
            json!(["must be at most 150"])
        );
        assert_eq!(body["effects"][0]["style"], "error");
        assert_eq!(s.db.count("Post").unwrap(), 0);

        let (status, body) = post_form(&s, json!({"title": "fine", "age": "42"}), &alice);
        assert_eq!(status, 201, "{body}");
    }

    #[test]
    fn form_update_validates_sent_fields_and_uniqueness() {
        let s = validation_state();
        let alice = crate::api_validation_tests::viewer("alice", "user");
        row_id(
            &s,
            "Post",
            json!({"title": "one", "contact": "a@b.co", "_owner_id": "alice"}),
        );
        let id = row_id(
            &s,
            "Post",
            json!({"title": "two", "contact": "c@d.co", "_owner_id": "alice"}),
        );
        let (status, body) = patch(&s, &id, "Post", json!({"contact": "a@b.co"}), &alice);
        assert_eq!(status, 409, "{body}");
        assert_eq!(
            body["error"]["fields"]["contact"],
            json!(["already exists"])
        );
        let (status, body) = patch(&s, &id, "Post", json!({"slug": "Bad Slug"}), &alice);
        assert_eq!(status, 422, "{body}");
        assert!(body["error"]["fields"]["slug"].is_array());
        assert!(body["error"]["fields"].get("title").is_none());
        let (status, body) = patch(&s, &id, "Post", json!({"slug": "good-slug"}), &alice);
        assert_eq!(status, 200, "{body}");
    }

    #[test]
    fn form_many_to_many_accepts_comma_separated_ids_in_callers_scope() {
        let s = validation_state();
        let alice = crate::api_validation_tests::viewer("alice", "user");
        let mine = row_id(&s, "Tag", json!({"label": "m", "_owner_id": "alice"}));
        let theirs = row_id(&s, "Tag", json!({"label": "t", "_owner_id": "bob"}));

        let (status, body) = post_form(&s, json!({"title": "steal", "tags": theirs}), &alice);
        assert_eq!(status, 422, "{body}");
        assert_eq!(
            body["error"]["fields"]["tags"],
            json!(["contains an unknown id"])
        );

        let (status, body) =
            post_form(&s, json!({"title": "linked", "tags": mine.clone()}), &alice);
        assert_eq!(status, 201, "{body}");
        let id = body["id"].as_str().unwrap().to_string();
        let (status, body) = patch(&s, &id, "Post", json!({"tags": ""}), &alice);
        assert_eq!(status, 200, "links-only update: {body}");
        assert_eq!(body["record"]["tags"], json!([]));
        let (status, body) = patch(&s, &id, "Post", json!({"tags": [mine.clone()]}), &alice);
        assert_eq!(status, 200, "{body}");
        assert_eq!(body["record"]["tags"], json!([mine]));
    }

    #[test]
    fn action_set_enforces_field_constraints() {
        let s = validation_state();
        let alice = crate::api_validation_tests::viewer("alice", "user");
        let id = row_id(&s, "Post", json!({"title": "valid", "_owner_id": "alice"}));
        let action = DeclaredAction {
            id: "manual".into(),
            entity: "Post".into(),
            block: ActionBlock {
                event: "click".into(),
                confirm: None,
                instructions: vec![crate::parser::ActionInstruction {
                    verb: "set".into(),
                    target: "title".into(),
                    value: "x".into(),
                    modifiers: std::collections::HashMap::new(),
                }],
            },
        };
        let err =
            execute_declared_action(&action, &id, &s.db, &s.entities, &alice, None).unwrap_err();
        assert_eq!(
            err,
            ActionDenied::Invalid("'title' must be at least 3 characters".into())
        );
        assert_eq!(
            s.db.find_by_id("Post", &id).unwrap().unwrap()["title"],
            "valid"
        );
    }

    #[test]
    fn form_submit_block_drives_toast_and_does_not_double_insert() {
        const SRC: &str = "app \"F\" { port 5175 }\n\
auth { entity User  login email + password  session jwt  roles [admin, user] }\n\
entity User { name string  email email!  role string  password string sensitive }\n\
entity Note { title string! }\n\
page \"/notes\" type:custom requires:auth {\n\
  section form {\n\
    bind Note { query all }\n\
    on submit {\n\
      create Note\n\
      toast \"Saved\" success\n\
      navigate \"/notes\"\n\
    }\n\
  }\n\
}\n";
        let mut s = crate::api_security_tests::state_from(SRC);
        s.pages = parse(SRC)
            .expect("parse")
            .into_iter()
            .filter_map(|n| match n {
                AstNode::Page(p) => Some(p),
                _ => None,
            })
            .collect();
        let alice = as_user("alice");
        let (status, body) = handle_form(
            &s,
            &hyper::Method::POST,
            "/_form/form",
            &json!({"entity": "Note", "data": {"title": "one"}}),
            &alice,
        );
        assert_eq!(status, 201, "{body}");
        assert_eq!(s.db.count("Note").unwrap(), 1);
        assert_eq!(body["effects"][0]["target"], "Saved");
        assert_eq!(body["effects"][1]["type"], "navigate");
        assert_eq!(body["effects"][1]["target"], "/notes");

        let declared = declared_actions(&s.pages);
        let submit = declared
            .iter()
            .find(|a| a.block.event == "submit")
            .expect("submit block is indexed");
        let (status, body) = handle_action(
            &s,
            &json!({"action_id": submit.id, "entity": "Note", "id": ""}),
            &alice,
        );
        assert_eq!(status, 403, "submit blocks are not /_action writes: {body}");
        assert_eq!(s.db.count("Note").unwrap(), 1, "no second insert");
    }

    #[test]
    fn action_create_uses_ast_literals_not_client_fields() {
        let s = validation_state();
        let alice = crate::api_validation_tests::viewer("alice", "user");
        let mut fields = std::collections::HashMap::new();
        fields.insert("title".into(), "from-ast".into());
        let action = DeclaredAction {
            id: "c".into(),
            entity: "Post".into(),
            block: ActionBlock {
                event: "click".into(),
                confirm: None,
                instructions: vec![crate::parser::ActionInstruction {
                    verb: "create".into(),
                    target: "Post".into(),
                    value: String::new(),
                    modifiers: fields,
                }],
            },
        };
        let effects =
            execute_declared_action(&action, "", &s.db, &s.entities, &alice, None).expect("ok");
        assert_eq!(s.db.count("Post").unwrap(), 1);
        let rows = s.db.find_all("Post", 10, 0).unwrap();
        let row = rows.as_array().unwrap()[0].clone();
        assert_eq!(row["title"], "from-ast");
        assert_eq!(row["_owner_id"], "alice");
        assert!(effects
            .iter()
            .all(|e| e.effect_type != "toast" || e.target != "from-ast"));

        let empty = DeclaredAction {
            id: "empty".into(),
            entity: "Post".into(),
            block: ActionBlock {
                event: "click".into(),
                confirm: None,
                instructions: vec![crate::parser::ActionInstruction {
                    verb: "create".into(),
                    target: "Post".into(),
                    value: String::new(),
                    modifiers: std::collections::HashMap::new(),
                }],
            },
        };
        assert_eq!(
            execute_declared_action(&empty, "", &s.db, &s.entities, &alice, None).unwrap_err(),
            ActionDenied::Invalid("create requires field values in the action block".into())
        );
        assert_eq!(s.db.count("Post").unwrap(), 1);
    }

    #[test]
    fn action_set_many_to_many_replaces_join_rows() {
        let s = validation_state();
        let alice = crate::api_validation_tests::viewer("alice", "user");
        let tag = row_id(&s, "Tag", json!({"label": "m", "_owner_id": "alice"}));
        let id = row_id(&s, "Post", json!({"title": "linked", "_owner_id": "alice"}));
        let action = DeclaredAction {
            id: "tags".into(),
            entity: "Post".into(),
            block: ActionBlock {
                event: "click".into(),
                confirm: None,
                instructions: vec![crate::parser::ActionInstruction {
                    verb: "set".into(),
                    target: "tags".into(),
                    value: tag.clone(),
                    modifiers: std::collections::HashMap::new(),
                }],
            },
        };
        execute_declared_action(&action, &id, &s.db, &s.entities, &alice, None).expect("ok");
        let mut row = s.db.find_by_id("Post", &id).unwrap().unwrap();
        let post = s.entities.iter().find(|e| e.name == "Post").unwrap();
        crate::relations::attach(&s.db, &s.entities, post, &mut row, &alice, &[]);
        assert_eq!(row["tags"], json!([tag]));
    }
}
