// CRONUS Action Execution Engine
//
// Processes action instructions from button clicks / form submits.
// Returns an effect envelope (toast, navigate, refresh, etc.)
//
// SECURITY (Sprint 1): the client never supplies instructions. It references
// an action declared in the AST (by id, or — for already-rendered pages — by
// the exact serialized block the server rendered), and the server executes
// its own copy. Writes are owner-scoped in SQL (`access::scoped_*`).

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use crate::access::{self, Access, WriteScope};
use crate::database::CronusDB;
use crate::parser::{ActionBlock, EntityNode, FieldType, PageNode, SectionNode};

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
    let field = match entity.fields.iter().find(|f| f.name.eq_ignore_ascii_case(field_name)) {
        Some(f) => f,
        None => return Some(format!("Field '{}' does not exist on entity '{}'", field_name, entity.name)),
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
                    return Some(format!("'{}' must be one of: {}", field_name, vals.join(", ")));
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
pub fn validate_form_data(data: &Value, entity: &EntityNode) -> std::collections::HashMap<String, String> {
    let mut errors = std::collections::HashMap::new();
    let obj = match data.as_object() {
        Some(o) => o,
        None => return errors,
    };

    // Check required fields are present and non-empty
    for field in &entity.fields {
        if field.required {
            let val = obj.get(&field.name)
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
    /// Stable identifier: sha256(route | section index | slot | block)[..16].
    pub id: String,
    pub entity: String,
    pub block: ActionBlock,
    json: Value,
}

fn section_entity(section: &SectionNode) -> String {
    section
        .config
        .get("entity")
        .cloned()
        .or_else(|| section.binding.as_ref().map(|b| b.entity.clone()))
        .unwrap_or_default()
}

fn action_id(route: &str, section_idx: usize, slot: &str, json: &Value) -> String {
    let mut h = Sha256::new();
    h.update(format!("{}|{}|{}|{}", route, section_idx, slot, json).as_bytes());
    hex::encode(h.finalize())[..16].to_string()
}

/// Every action block in the AST: item-level `on click { … }` (rendered as
/// `data-cronus-action`) and section-level `on …` blocks.
pub fn declared_actions(pages: &[PageNode]) -> Vec<DeclaredAction> {
    let mut out = Vec::new();
    for page in pages {
        for (si, section) in page.sections.iter().enumerate() {
            let entity = section_entity(section);
            let mut push = |slot: String, block: ActionBlock| {
                if let Ok(json) = serde_json::to_value(&block) {
                    out.push(DeclaredAction {
                        id: action_id(&page.route, si, &slot, &json),
                        entity: entity.clone(),
                        block,
                        json,
                    });
                }
            };
            for (ii, item) in section.items.iter().enumerate() {
                for (key, raw) in item.iter().filter(|(k, _)| k.starts_with("on_")) {
                    if let Ok(block) = serde_json::from_str::<ActionBlock>(raw) {
                        push(format!("item{}:{}", ii, key), block);
                    }
                }
            }
            for (ai, block) in section.actions.iter().enumerate() {
                push(format!("action{}", ai), block.clone());
            }
        }
    }
    out
}

/// Resolve the client's reference to a declared action. With `action_id` the
/// client payload is ignored entirely; otherwise the client's serialized
/// block must equal a declared one. The entity must be the declared entity.
pub fn find_declared_action<'a>(
    declared: &'a [DeclaredAction],
    action_id: Option<&str>,
    client_action: Option<&str>,
    client_entity: &str,
) -> Option<&'a DeclaredAction> {
    let entity_ok = |a: &DeclaredAction| a.entity.eq_ignore_ascii_case(client_entity);
    if let Some(id) = action_id.filter(|s| !s.is_empty()) {
        return declared.iter().find(|a| a.id == id && entity_ok(a));
    }
    let wanted: Value = serde_json::from_str(client_action?).ok()?;
    declared.iter().find(|a| a.json == wanted && entity_ok(a))
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
pub fn find_declared_form(pages: &[PageNode], section_type: &str, entity: &str) -> Option<DeclaredForm> {
    let mut found: Option<DeclaredForm> = None;
    for page in pages {
        let page_open = page.requires.is_none() && !page.config.contains_key("requires");
        for section in &page.sections {
            let declared = section_entity(section);
            if section.section_type != section_type || declared.is_empty() || !declared.eq_ignore_ascii_case(entity) {
                continue;
            }
            let public = page_open && section.binding.as_ref().map(|b| b.public).unwrap_or(false);
            match found.as_mut() {
                Some(f) => f.public |= public,
                None => found = Some(DeclaredForm { entity: declared, public }),
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
    let touches_data = instructions.iter().any(|i| i.verb == "set" || i.verb == "delete");

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
        if let WriteScope::Owner(owner) = &scope {
            filters.push(("_owner_id".into(), "=".into(), owner.clone()));
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
        let style = instr.modifiers.get("style").map(|s| s.as_str()).unwrap_or("info");
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
        let forged = r#"{"event":"click","confirm":null,"instructions":[{"verb":"delete","target":"","value":"","modifiers":{}}]}"#;
        assert!(find_declared_action(&declared, None, Some(forged), "Note").is_none());
        assert!(find_declared_action(&declared, Some("deadbeefdeadbeef"), Some(forged), "Note").is_none());
        let legit = serde_json::to_string(&archive(&declared).block).unwrap();
        assert!(find_declared_action(&declared, None, Some(&legit), "Note").is_some());
        assert!(find_declared_action(&declared, None, Some(&legit), "User").is_none(), "entity must match the declaration");
    }

    #[test]
    fn client_instructions_are_ignored() {
        let ents = entities();
        let db = db(&ents);
        let id = insert_note(&db, "alice", "draft");
        let declared = declared_actions(&pages());
        let forged = r#"{"event":"click","instructions":[{"verb":"set","target":"title","value":"pwned","modifiers":{}}]}"#;
        let action = find_declared_action(&declared, Some(&archive(&declared).id), Some(forged), "Note").expect("declared");
        let effects = execute_declared_action(action, &id, &db, &ents, &as_user("alice")).expect("ok");
        assert_eq!(db.find_by_id("Note", &id).unwrap().unwrap()["title"], "archived");
        assert_eq!(effects[0].target, "Archived");
    }

    #[test]
    fn actions_require_session_owner_and_writable_fields() {
        let ents = entities();
        let db = db(&ents);
        let id = insert_note(&db, "alice", "draft");
        let declared = declared_actions(&pages());
        let a = archive(&declared);
        assert_eq!(execute_declared_action(a, &id, &db, &ents, &anon()).unwrap_err(), ActionDenied::Unauthenticated);
        assert_eq!(execute_declared_action(a, &id, &db, &ents, &as_user("bob")).unwrap_err(), ActionDenied::NotFound);
        assert_eq!(db.find_by_id("Note", &id).unwrap().unwrap()["title"], "draft");
        let secret = declared
            .iter()
            .find(|a| a.block.instructions.iter().any(|i| i.target == "secret"))
            .expect("secret action");
        assert_eq!(execute_declared_action(secret, &id, &db, &ents, &as_user("alice")).unwrap_err(), ActionDenied::Forbidden);
    }

    #[test]
    fn forms_must_be_declared_and_public_only_when_explicit() {
        let p = pages();
        assert_eq!(find_declared_form(&p, "form", "note"), Some(DeclaredForm { entity: "Note".into(), public: false }));
        assert_eq!(find_declared_form(&p, "form", "Tag"), Some(DeclaredForm { entity: "Tag".into(), public: true }));
        assert_eq!(find_declared_form(&p, "form", "User"), None);
        assert_eq!(find_declared_form(&p, "hero", "Note"), None);
    }
}
