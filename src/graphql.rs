//! CRONUS GraphQL Engine — Auto-generated from EntityNode
//!
//! - GET  /graphql  → Playground HTML
//! - POST /graphql  → Execute queries/mutations
//!
//! Auto-generates types, queries, and mutations from entities.
//! Resolves via CronusDB.

use serde_json::{json, Map, Value};

use crate::access::{self, Access, ReadScope, WriteScope};
use crate::authz;
use crate::database::CronusDB;
use crate::parser::{EntityNode, FieldNode, FieldType};
use crate::relations;
use crate::validation::{self, Mode};

/// Output type: relations are the related entity (`[Tag!]`, `User`).
fn graphql_output_type(field: &FieldNode) -> String {
    if field.field_type == FieldType::Relation {
        let name = field.reference.as_deref().unwrap_or("String");
        if field.array {
            format!("[{name}!]")
        } else {
            name.to_string()
        }
    } else if field.enum_values.is_some() {
        "String".to_string()
    } else {
        field_type_to_graphql(&field.field_type).to_string()
    }
}

/// Input type: relations stay ids (`[String!]`, `String`). No nested mutations.
fn graphql_input_type(field: &FieldNode) -> String {
    if field.is_many() {
        "[String!]".to_string()
    } else if field.field_type == FieldType::Relation {
        "String".to_string()
    } else if field.enum_values.is_some() {
        "String".to_string()
    } else {
        field_type_to_graphql(&field.field_type).to_string()
    }
}

/// Never part of the generated output type (or introspection).
fn is_readable_field(field: &FieldNode) -> bool {
    !field.sensitive && field.name != "password" && field.name != "password_hash"
}

/// Only fields `authz::writable_body` would keep appear in create inputs.
fn is_writable_field(field: &FieldNode) -> bool {
    is_readable_field(field)
        && !authz::SYSTEM_FIELDS.contains(&field.name.as_str())
        && !authz::PRIVILEGED_FIELDS.contains(&field.name.as_str())
}

// ══════════════════════════════════════════════════
// SCHEMA GENERATION
// ══════════════════════════════════════════════════

/// A generated GraphQL schema with SDL and entity metadata.
pub struct GraphQLSchema {
    pub sdl: String,
    pub entities: Vec<EntityNode>,
}

impl GraphQLSchema {
    /// Build a complete SDL schema from entity definitions.
    pub fn from_entities(entities: &[EntityNode]) -> Self {
        let mut sdl = String::new();

        // Generate types + input types
        for entity in entities {
            sdl.push_str(&generate_type(entity, entities));
            sdl.push('\n');
            sdl.push_str(&generate_create_input(entity));
            sdl.push('\n');
            sdl.push_str(&generate_update_input(entity));
            sdl.push('\n');
        }

        // Generate Query type
        sdl.push_str("type Query {\n");
        for entity in entities {
            let lower = entity.name.to_lowercase();
            let plural = format!("{}s", lower);
            sdl.push_str(&format!(
                "  {plural}(limit: Int): [{name}!]!\n",
                name = entity.name
            ));
            sdl.push_str(&format!(
                "  {lower}(id: String!): {name}\n",
                name = entity.name
            ));
        }
        sdl.push_str("}\n\n");

        // Generate Mutation type
        sdl.push_str("type Mutation {\n");
        for entity in entities {
            sdl.push_str(&format!(
                "  create{name}(input: Create{name}Input!): {name}!\n",
                name = entity.name
            ));
            sdl.push_str(&format!(
                "  update{name}(id: String!, input: Update{name}Input!): {name}\n",
                name = entity.name
            ));
            sdl.push_str(&format!(
                "  delete{name}(id: String!): Boolean!\n",
                name = entity.name
            ));
        }
        sdl.push_str("}\n");

        Self {
            sdl,
            entities: entities.to_vec(),
        }
    }
}

fn field_type_to_graphql(ft: &FieldType) -> &'static str {
    match ft {
        FieldType::Number | FieldType::Money | FieldType::Percentage => "Int",
        FieldType::Boolean => "Boolean",
        FieldType::Ulid => "ID",
        _ => "String",
    }
}

fn generate_type(entity: &EntityNode, entities: &[EntityNode]) -> String {
    let mut out = format!("type {} {{\n", entity.name);
    out.push_str("  id: String!\n");
    for field in entity.fields.iter().filter(|f| is_readable_field(f)) {
        // A many-to-many list is always present (possibly empty).
        let bang = if field.required || field.is_many() {
            "!"
        } else {
            ""
        };
        let gql_type = graphql_output_type(field);
        out.push_str(&format!("  {}: {}{}\n", field.name, gql_type, bang));
    }
    for rev in relations::reverse_rels(entities, entity) {
        out.push_str(&format!("  {}: [{}!]!\n", rev.name, rev.source_entity));
    }
    out.push_str("  created_at: String\n");
    out.push_str("  updated_at: String\n");
    out.push_str("}\n");
    out
}

fn generate_create_input(entity: &EntityNode) -> String {
    let mut out = format!("input Create{}Input {{\n", entity.name);
    for field in entity.fields.iter().filter(|f| is_writable_field(f)) {
        let gql_type = graphql_input_type(field);
        let bang = if field.required { "!" } else { "" };
        out.push_str(&format!("  {}: {}{}\n", field.name, gql_type, bang));
    }
    out.push_str("}\n");
    out
}

fn generate_update_input(entity: &EntityNode) -> String {
    let mut out = format!("input Update{}Input {{\n", entity.name);
    for field in entity.fields.iter().filter(|f| is_writable_field(f)) {
        let gql_type = graphql_input_type(field);
        out.push_str(&format!("  {}: {}\n", field.name, gql_type));
    }
    out.push_str("}\n");
    out
}

fn mutation_input(field: &ParsedField, variables: &Value) -> Value {
    field
        .args
        .iter()
        .find(|(k, _)| k == "input")
        .and_then(|(_, v)| match v {
            ArgValue::Variable(var_name) => variables.get(var_name).cloned(),
            _ => None,
        })
        .unwrap_or_else(|| variables.get("input").cloned().unwrap_or(json!({})))
}

// ══════════════════════════════════════════════════
// QUERY PARSER (minimal)
// ══════════════════════════════════════════════════

#[derive(Debug)]
enum Operation {
    Query,
    Mutation,
}

#[derive(Debug)]
struct ParsedField {
    name: String,
    args: Vec<(String, ArgValue)>,
    sub_fields: Vec<ParsedField>,
}

#[derive(Debug)]
enum ArgValue {
    StringVal(String),
    IntVal(i64),
    BoolVal,
    Variable(String),
}

/// Parse a minimal GraphQL query string into operations.
/// Supports: query { users { id name } } and mutation { createUser(input: $input) { id } }
fn parse_query(query: &str) -> Result<(Operation, Vec<ParsedField>), String> {
    let trimmed = query.trim();

    let (op, body) = if trimmed.starts_with("mutation") {
        let rest = trimmed.strip_prefix("mutation").unwrap().trim();
        // Skip optional variable declarations: mutation($input: CreateUserInput!) { ... }
        let rest = skip_parens_if_present(rest);
        (Operation::Mutation, extract_braces(rest)?)
    } else {
        let rest = if trimmed.starts_with("query") {
            let r = trimmed.strip_prefix("query").unwrap().trim();
            skip_parens_if_present(r)
        } else {
            trimmed
        };
        (Operation::Query, extract_braces(rest)?)
    };

    let fields = parse_fields(&body)?;
    Ok((op, fields))
}

fn skip_parens_if_present(s: &str) -> &str {
    let s = s.trim();
    if s.starts_with('(') {
        // Find matching closing paren
        let mut depth = 0;
        for (i, c) in s.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return s[i + 1..].trim();
                    }
                }
                _ => {}
            }
        }
        s // fallback
    } else {
        s
    }
}

fn extract_braces(s: &str) -> Result<String, String> {
    let s = s.trim();
    if !s.starts_with('{') {
        return Err("Expected '{'".into());
    }
    // Find matching closing brace
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(s[1..i].trim().to_string());
                }
            }
            _ => {}
        }
    }
    Err("Unmatched '{'".into())
}

fn parse_fields(body: &str) -> Result<Vec<ParsedField>, String> {
    let mut fields = Vec::new();
    let mut chars = body.chars().peekable();

    while chars.peek().is_some() {
        // Skip whitespace
        while chars.peek().map_or(false, |c| c.is_whitespace()) {
            chars.next();
        }
        if chars.peek().is_none() {
            break;
        }

        // Read field name
        let mut name = String::new();
        while chars
            .peek()
            .map_or(false, |c| c.is_alphanumeric() || *c == '_')
        {
            name.push(chars.next().unwrap());
        }
        if name.is_empty() {
            // Skip unexpected chars
            chars.next();
            continue;
        }

        // Skip whitespace
        while chars.peek().map_or(false, |c| c.is_whitespace()) {
            chars.next();
        }

        // Parse args if present
        let mut args = Vec::new();
        if chars.peek() == Some(&'(') {
            chars.next(); // consume '('
            args = parse_args_inline(&mut chars)?;
        }

        // Skip whitespace
        while chars.peek().map_or(false, |c| c.is_whitespace()) {
            chars.next();
        }

        // Parse sub-fields if present
        let mut sub_fields = Vec::new();
        if chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let mut depth = 1;
            let mut inner = String::new();
            while let Some(c) = chars.next() {
                match c {
                    '{' => {
                        depth += 1;
                        inner.push(c);
                    }
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        inner.push(c);
                    }
                    _ => inner.push(c),
                }
            }
            sub_fields = parse_fields(&inner)?;
        }

        fields.push(ParsedField {
            name,
            args,
            sub_fields,
        });
    }

    Ok(fields)
}

fn parse_args_inline(
    chars: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<Vec<(String, ArgValue)>, String> {
    let mut args = Vec::new();

    loop {
        // Skip whitespace
        while chars
            .peek()
            .map_or(false, |c| c.is_whitespace() || *c == ',')
        {
            chars.next();
        }
        if chars.peek() == Some(&')') {
            chars.next();
            break;
        }
        if chars.peek().is_none() {
            break;
        }

        // Read key
        let mut key = String::new();
        while chars
            .peek()
            .map_or(false, |c| c.is_alphanumeric() || *c == '_')
        {
            key.push(chars.next().unwrap());
        }

        // Skip ':'
        while chars
            .peek()
            .map_or(false, |c| c.is_whitespace() || *c == ':')
        {
            chars.next();
        }

        // Read value
        let val = parse_arg_value(chars)?;
        args.push((key, val));
    }

    Ok(args)
}

fn parse_arg_value(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<ArgValue, String> {
    // Skip whitespace
    while chars.peek().map_or(false, |c| c.is_whitespace()) {
        chars.next();
    }

    match chars.peek() {
        Some('$') => {
            chars.next(); // consume $
            let mut name = String::new();
            while chars
                .peek()
                .map_or(false, |c| c.is_alphanumeric() || *c == '_')
            {
                name.push(chars.next().unwrap());
            }
            Ok(ArgValue::Variable(name))
        }
        Some('"') => {
            chars.next(); // consume "
            let mut val = String::new();
            while let Some(c) = chars.next() {
                if c == '"' {
                    break;
                }
                val.push(c);
            }
            Ok(ArgValue::StringVal(val))
        }
        Some(c) if c.is_ascii_digit() || *c == '-' => {
            let mut num = String::new();
            while chars
                .peek()
                .map_or(false, |c| c.is_ascii_digit() || *c == '-')
            {
                num.push(chars.next().unwrap());
            }
            Ok(ArgValue::IntVal(num.parse().map_err(|_| "invalid number")?))
        }
        Some('t') | Some('f') => {
            let mut word = String::new();
            while chars.peek().map_or(false, |c| c.is_alphabetic()) {
                word.push(chars.next().unwrap());
            }
            Ok(ArgValue::BoolVal)
        }
        _ => Err("unexpected arg value".into()),
    }
}

// ══════════════════════════════════════════════════
// EXECUTOR
// ══════════════════════════════════════════════════

/// A mutation failure: `{"message", "extensions": {"code"[, "fields"]}}`.
/// `fields` carries per-field messages for `VALIDATION_FAILED`.
struct GqlError {
    code: &'static str,
    message: String,
    fields: Option<Value>,
}

impl GqlError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        GqlError {
            code,
            message: message.into(),
            fields: None,
        }
    }

    fn to_json(&self) -> Value {
        let mut extensions = json!({ "code": self.code });
        if let Some(fields) = &self.fields {
            extensions["fields"] = fields.clone();
        }
        json!({ "message": self.message, "extensions": extensions })
    }
}

pub fn gql_error(code: &str, message: &str) -> Value {
    json!({ "errors": [{ "message": message, "extensions": { "code": code } }] })
}

/// Execute a GraphQL request. Returns JSON response { data, errors }.
///
/// SECURITY: requires an authenticated viewer; reads use the same owner scope
/// as REST, mutations write through `authz::writable_body` with a server-set
/// `_owner_id`, deletes are owner-constrained in SQL, and sensitive fields
/// are redacted. DB error text is logged, never returned.
pub fn execute_graphql(
    query: &str,
    variables: &Value,
    schema: &GraphQLSchema,
    db: &CronusDB,
    access: &Access,
    db_path: &str,
) -> Value {
    if access.viewer.is_none() {
        return gql_error("UNAUTHENTICATED", "authentication required");
    }
    let (op, fields) = match parse_query(query) {
        Ok(r) => r,
        Err(e) => {
            return json!({
                "errors": [{ "message": e }]
            });
        }
    };

    let mut data = Map::new();
    let mut errors: Vec<Value> = Vec::new();

    for field in &fields {
        match op {
            Operation::Query => {
                if let Some(result) = resolve_query(field, schema, db, access, variables) {
                    data.insert(field.name.clone(), result);
                } else {
                    errors.push(json!({
                        "message": format!("Unknown field: {}", field.name)
                    }));
                }
            }
            Operation::Mutation => {
                match resolve_mutation(field, schema, db, variables, access, db_path) {
                    Ok(result) => {
                        data.insert(field.name.clone(), result);
                    }
                    Err(failure) => errors.push(failure.to_json()),
                }
            }
        }
    }

    let mut result = json!({ "data": Value::Object(data) });
    if !errors.is_empty() {
        result["errors"] = Value::Array(errors);
    }
    result
}

fn string_arg(field: &ParsedField, name: &str, variables: &Value) -> Option<String> {
    field
        .args
        .iter()
        .find(|(k, _)| k == name)
        .and_then(|(_, v)| match v {
            ArgValue::StringVal(s) => Some(s.clone()),
            ArgValue::Variable(var) => variables
                .get(var)
                .and_then(Value::as_str)
                .map(str::to_string),
            ArgValue::IntVal(n) => Some(n.to_string()),
            _ => None,
        })
}

/// Relation and reverse names the selection asked to expand (bind `expand:`).
fn expand_selection(
    entity: &EntityNode,
    entities: &[EntityNode],
    sub_fields: &[ParsedField],
) -> Vec<String> {
    let mut out = Vec::new();
    for field in entity
        .fields
        .iter()
        .filter(|f| f.field_type == FieldType::Relation)
    {
        if sub_fields.iter().any(|s| s.name == field.name) {
            out.push(field.name.clone());
        }
    }
    for rev in relations::reverse_rels(entities, entity) {
        if sub_fields.iter().any(|s| s.name == rev.name) {
            out.push(rev.name);
        }
    }
    out
}

fn related_entity<'a>(
    entity: &EntityNode,
    entities: &'a [EntityNode],
    name: &str,
) -> Option<&'a EntityNode> {
    if let Some(target) = entity
        .fields
        .iter()
        .find(|f| f.name == name && f.field_type == FieldType::Relation)
        .and_then(|f| f.reference.as_deref())
    {
        return entities.iter().find(|e| e.name == target);
    }
    relations::reverse_rels(entities, entity)
        .into_iter()
        .find(|r| r.name == name)
        .and_then(|r| entities.iter().find(|e| e.name == r.source_entity))
}

fn attach_selection(
    db: &CronusDB,
    entities: &[EntityNode],
    entity: &EntityNode,
    rows: &mut Value,
    access: &Access,
    fields: &[ParsedField],
) {
    relations::attach(
        db,
        entities,
        entity,
        rows,
        access,
        &expand_selection(entity, entities, fields),
    );
    for sf in fields.iter().filter(|f| !f.sub_fields.is_empty()) {
        let Some(target) = related_entity(entity, entities, &sf.name) else {
            continue;
        };
        match rows {
            Value::Array(items) => {
                for item in items {
                    attach_nested(db, entities, target, item, access, sf);
                }
            }
            Value::Object(_) => attach_nested(db, entities, target, rows, access, sf),
            _ => {}
        }
    }
}

fn attach_nested(
    db: &CronusDB,
    entities: &[EntityNode],
    target: &EntityNode,
    row: &mut Value,
    access: &Access,
    field: &ParsedField,
) {
    match row.get_mut(&field.name) {
        Some(nested @ (Value::Array(_) | Value::Object(_))) => {
            attach_selection(db, entities, target, nested, access, &field.sub_fields);
        }
        _ => {}
    }
}

fn select(value: Value, sub_fields: &[ParsedField]) -> Value {
    if sub_fields.is_empty() {
        value
    } else if value.is_array() {
        select_array(&value, sub_fields)
    } else {
        select_object(&value, sub_fields)
    }
}

fn resolve_query(
    field: &ParsedField,
    schema: &GraphQLSchema,
    db: &CronusDB,
    access: &Access,
    variables: &Value,
) -> Option<Value> {
    for entity in &schema.entities {
        let lower = entity.name.to_lowercase();
        let is_list = field.name == format!("{}s", lower);
        if !is_list && field.name != lower {
            continue;
        }
        let empty = if is_list { json!([]) } else { Value::Null };
        let scope = access::read_scope(access, &entity.name, Some(entity), false);
        if scope == ReadScope::Deny {
            return Some(empty);
        }
        let mut filters: Vec<crate::database::SqlFilter> = scope
            .filter()
            .into_iter()
            .map(crate::database::SqlFilter::from_triple)
            .collect();

        if is_list {
            let limit = field
                .args
                .iter()
                .find(|(k, _)| k == "limit")
                .and_then(|(_, v)| match v {
                    ArgValue::IntVal(n) => usize::try_from(*n).ok(),
                    _ => None,
                })
                .unwrap_or(100)
                .min(1000);
            return Some(
                match db.find_many(&entity.name, &filters, None, None, Some(limit), Some(0)) {
                    Ok(mut rows) => {
                        authz::redact_sensitive(entity, &mut rows);
                        attach_selection(
                            db,
                            &schema.entities,
                            entity,
                            &mut rows,
                            access,
                            &field.sub_fields,
                        );
                        select(rows, &field.sub_fields)
                    }
                    Err(e) => {
                        eprintln!("  graphql {} list failed: {}", entity.name, e);
                        empty
                    }
                },
            );
        }

        let id = match string_arg(field, "id", variables) {
            Some(id) => id,
            None => return Some(Value::Null),
        };
        filters.push(crate::database::SqlFilter::one("id", "=", id));
        return Some(match db.find_one(&entity.name, &filters, None, None) {
            Ok(Some(mut row)) => {
                authz::redact_sensitive(entity, &mut row);
                attach_selection(
                    db,
                    &schema.entities,
                    entity,
                    &mut row,
                    access,
                    &field.sub_fields,
                );
                select(row, &field.sub_fields)
            }
            Ok(None) => Value::Null,
            Err(e) => {
                eprintln!("  graphql {} by id failed: {}", entity.name, e);
                Value::Null
            }
        });
    }

    None
}

fn resolve_mutation(
    field: &ParsedField,
    schema: &GraphQLSchema,
    db: &CronusDB,
    variables: &Value,
    access: &Access,
    db_path: &str,
) -> Result<Value, GqlError> {
    if access.viewer.is_none() {
        return Err(GqlError::new("UNAUTHENTICATED", "authentication required"));
    }
    for entity in &schema.entities {
        let create_name = format!("create{}", entity.name);
        let update_name = format!("update{}", entity.name);
        let delete_name = format!("delete{}", entity.name);

        if field.name == create_name {
            let owner = match access::create_owner(access, &entity.name, false) {
                Ok(owner) => owner,
                Err(access::Denial::Unauthenticated) => {
                    return Err(GqlError::new("UNAUTHENTICATED", "authentication required"))
                }
                Err(access::Denial::Forbidden) => {
                    return Err(GqlError::new("FORBIDDEN", "not allowed"))
                }
            };
            let input = mutation_input(field, variables);
            let obj = input
                .as_object()
                .ok_or_else(|| GqlError::new("BAD_USER_INPUT", "input must be an object"))?;
            let (columns, many) = relations::split(entity, authz::writable_body(entity, obj));
            let mut body: Map<String, Value> =
                columns.into_iter().filter(|(_, v)| !v.is_null()).collect();
            crate::files::persist_uploads(entity, &mut body, &crate::files::dir_for(db_path))
                .map_err(|errors| GqlError {
                    code: "VALIDATION_FAILED",
                    message: validation::message(&errors),
                    fields: Some(validation::fields_json(&errors)),
                })?;
            for f in entity.fields.iter().filter(|f| !f.is_many()) {
                if let Some(default) = &f.default_value {
                    if !body.contains_key(&f.name) {
                        body.insert(f.name.clone(), Value::String(default.clone()));
                    }
                }
            }
            let links = validation::check_write(
                db,
                &schema.entities,
                entity,
                &body,
                &many,
                Mode::Create,
                None,
                access,
            )
            .map_err(|(_, errors)| GqlError {
                code: "VALIDATION_FAILED",
                message: validation::message(&errors),
                fields: Some(validation::fields_json(&errors)),
            })?;
            if let Some(owner) = &owner {
                body.insert("_owner_id".into(), json!(owner));
            }
            let mut row = db
                .insert_with(&entity.name, &Value::Object(body), |conn, id| {
                    links.iter().try_for_each(|link| {
                        relations::replace_links(conn, entity, link, id, owner.as_deref())
                    })
                })
                .map_err(|e| {
                    eprintln!("  graphql {} failed: {}", create_name, e);
                    GqlError::new("CREATE_FAILED", format!("could not create {}", entity.name))
                })?;
            authz::redact_sensitive(entity, &mut row);
            attach_selection(
                db,
                &schema.entities,
                entity,
                &mut row,
                access,
                &field.sub_fields,
            );
            return Ok(select(row, &field.sub_fields));
        }

        if field.name == update_name {
            let scope = access::write_scope(access, &entity.name);
            if scope == WriteScope::Deny {
                return Err(GqlError::new(
                    if access.viewer.is_none() {
                        "UNAUTHENTICATED"
                    } else {
                        "FORBIDDEN"
                    },
                    "not allowed",
                ));
            }
            let id = string_arg(field, "id", variables)
                .ok_or_else(|| GqlError::new("BAD_USER_INPUT", "update requires id argument"))?;
            let input = mutation_input(field, variables);
            let obj = input
                .as_object()
                .ok_or_else(|| GqlError::new("BAD_USER_INPUT", "input must be an object"))?;
            let (columns, many) = relations::split(entity, authz::writable_body(entity, obj));
            let mut body: Map<String, Value> =
                columns.into_iter().filter(|(_, v)| !v.is_null()).collect();
            crate::files::persist_uploads(entity, &mut body, &crate::files::dir_for(db_path))
                .map_err(|errors| GqlError {
                    code: "VALIDATION_FAILED",
                    message: validation::message(&errors),
                    fields: Some(validation::fields_json(&errors)),
                })?;
            if body.is_empty() && many.is_empty() {
                return Err(GqlError::new("BAD_USER_INPUT", "nothing to update"));
            }
            let mut filters = vec![crate::database::SqlFilter::one("id", "=", id.clone())];
            if let Some((column, value)) = scope.condition() {
                filters.push(crate::database::SqlFilter::one(column, "=", value));
            }
            let prev = match db.find_one(&entity.name, &filters, None, None) {
                Ok(Some(row)) => row,
                Ok(None) => return Ok(Value::Null),
                Err(e) => {
                    eprintln!("  graphql {} failed: {}", update_name, e);
                    return Err(GqlError::new(
                        "UPDATE_FAILED",
                        format!("could not update {}", entity.name),
                    ));
                }
            };
            let links = validation::check_write(
                db,
                &schema.entities,
                entity,
                &body,
                &many,
                Mode::Update,
                Some(&id),
                access,
            )
            .map_err(|(_, errors)| GqlError {
                code: "VALIDATION_FAILED",
                message: validation::message(&errors),
                fields: Some(validation::fields_json(&errors)),
            })?;
            if !entity.transitions.is_empty() {
                if crate::effects::validate_transitions(entity, &Value::Object(body.clone()), &prev)
                    .is_err()
                {
                    return Err(GqlError::new(
                        "CONFLICT",
                        format!("invalid transition on {}", entity.name),
                    ));
                }
            }
            if !body.is_empty() {
                match access::scoped_update(db, &entity.name, &id, &body, &scope) {
                    Ok(None) => return Ok(Value::Null),
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("  graphql {} failed: {}", update_name, e);
                        return Err(GqlError::new(
                            "UPDATE_FAILED",
                            format!("could not update {}", entity.name),
                        ));
                    }
                }
            }
            if !links.is_empty() {
                let parent_owner = prev.get("_owner_id").and_then(Value::as_str);
                if let Err(e) = db.transaction(|conn| {
                    links.iter().try_for_each(|link| {
                        relations::replace_links(conn, entity, link, &id, parent_owner)
                    })
                }) {
                    eprintln!("  graphql {} failed: {}", update_name, e);
                    return Err(GqlError::new(
                        "UPDATE_FAILED",
                        format!("could not update {}", entity.name),
                    ));
                }
            }
            let mut row = match db.find_one(&entity.name, &filters, None, None) {
                Ok(Some(row)) => row,
                Ok(None) => return Ok(Value::Null),
                Err(e) => {
                    eprintln!("  graphql {} failed: {}", update_name, e);
                    return Err(GqlError::new(
                        "UPDATE_FAILED",
                        format!("could not update {}", entity.name),
                    ));
                }
            };
            authz::redact_sensitive(entity, &mut row);
            attach_selection(
                db,
                &schema.entities,
                entity,
                &mut row,
                access,
                &field.sub_fields,
            );
            return Ok(select(row, &field.sub_fields));
        }

        if field.name == delete_name {
            let id = string_arg(field, "id", variables)
                .ok_or_else(|| GqlError::new("BAD_USER_INPUT", "delete requires id argument"))?;
            let scope = access::write_scope(access, &entity.name);
            if scope == WriteScope::Deny {
                return Err(GqlError::new("FORBIDDEN", "not allowed"));
            }
            let deleted = access::scoped_delete(db, &entity.name, &id, &scope).map_err(|e| {
                eprintln!("  graphql {} failed: {}", delete_name, e);
                GqlError::new("DELETE_FAILED", format!("could not delete {}", entity.name))
            })?;
            return Ok(json!(deleted));
        }
    }

    Err(GqlError::new(
        "UNKNOWN_MUTATION",
        format!("Unknown mutation: {}", field.name),
    ))
}

fn select_array(arr: &Value, fields: &[ParsedField]) -> Value {
    match arr.as_array() {
        Some(items) => Value::Array(
            items
                .iter()
                .map(|item| select_object(item, fields))
                .collect(),
        ),
        None => arr.clone(),
    }
}

fn select_object(obj: &Value, fields: &[ParsedField]) -> Value {
    match obj.as_object() {
        Some(map) => {
            let mut filtered = Map::new();
            for f in fields {
                if let Some(val) = map.get(&f.name) {
                    let nested = if f.sub_fields.is_empty() {
                        val.clone()
                    } else if val.is_array() {
                        select_array(val, &f.sub_fields)
                    } else {
                        select_object(val, &f.sub_fields)
                    };
                    filtered.insert(f.name.clone(), nested);
                }
            }
            Value::Object(filtered)
        }
        None => obj.clone(),
    }
}

// ══════════════════════════════════════════════════
// PLAYGROUND HTML
// ══════════════════════════════════════════════════

pub fn playground_html() -> String {
    crate::security::mark_kernel_scripts(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8" />
  <title>CRONUS GraphQL Playground</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body { font-family: 'JetBrains Mono', monospace; background: #050505; color: #fafafa; }
    .container { max-width: 1200px; margin: 0 auto; padding: 24px; }
    h1 { font-size: 14px; color: #71717a; text-transform: uppercase; letter-spacing: 0.15em; margin-bottom: 24px; }
    h1 span { color: #f59e0b; }
    .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; height: calc(100vh - 140px); }
    .panel { background: #0a0a0a; border: 1px solid #27272a; display: flex; flex-direction: column; }
    .panel-header { padding: 12px 16px; border-bottom: 1px solid #27272a; font-size: 10px; text-transform: uppercase; letter-spacing: 0.15em; color: #52525b; display: flex; justify-content: space-between; align-items: center; }
    textarea { flex: 1; background: transparent; color: #fafafa; border: none; padding: 16px; font-family: 'JetBrains Mono', monospace; font-size: 13px; resize: none; outline: none; }
    pre { flex: 1; padding: 16px; overflow: auto; font-size: 13px; color: #a1a1aa; }
    .btn { padding: 6px 16px; background: #f59e0b; color: #000; border: none; font-family: 'JetBrains Mono', monospace; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.1em; cursor: pointer; }
    .btn:hover { background: #fbbf24; }
    .schema-btn { padding: 6px 12px; background: transparent; color: #52525b; border: 1px solid #27272a; font-family: 'JetBrains Mono', monospace; font-size: 10px; cursor: pointer; text-transform: uppercase; letter-spacing: 0.1em; }
    .schema-btn:hover { color: #f59e0b; border-color: #f59e0b; }
    .bar { padding: 12px 16px; border-top: 1px solid #27272a; display: flex; gap: 8px; align-items: center; }
    .bar input { flex: 1; background: transparent; border: 1px solid #27272a; color: #fafafa; padding: 8px 12px; font-family: 'JetBrains Mono', monospace; font-size: 12px; outline: none; }
    .bar input:focus { border-color: #f59e0b; }
    .status { font-size: 10px; color: #52525b; padding: 8px 16px; border-top: 1px solid #27272a; display: flex; gap: 16px; }
    .status .ok { color: #22c55e; }
    .status .err { color: #ef4444; }
  </style>
</head>
<body>
  <div class="container">
    <h1><span>CRONUS</span> // GraphQL Playground</h1>
    <div class="grid">
      <div class="panel">
        <div class="panel-header">
          <span>Query</span>
          <div style="display:flex;gap:6px">
            <button class="schema-btn" onclick="showSchema()">Schema</button>
            <button class="btn" onclick="run()">Execute ▶</button>
          </div>
        </div>
        <textarea id="query" spellcheck="false">query {
  users(limit: 10) {
    id
    name
    email
  }
}</textarea>
        <div class="bar">
          <span style="font-size:10px;color:#52525b;text-transform:uppercase;letter-spacing:0.1em">Variables</span>
          <input id="vars" placeholder="{}" value="{}" />
        </div>
      </div>
      <div class="panel">
        <div class="panel-header">
          <span>Response</span>
          <span id="timing" style="color:#52525b">—</span>
        </div>
        <pre id="result">// Click Execute to run your query</pre>
        <div class="status">
          <span id="status-indicator">Ready</span>
        </div>
      </div>
    </div>
  </div>
  <script>
    async function run() {
      const q = document.getElementById('query').value;
      const v = document.getElementById('vars').value;
      const indicator = document.getElementById('status-indicator');
      const timing = document.getElementById('timing');
      indicator.className = '';
      indicator.textContent = 'Executing...';
      const start = performance.now();
      try {
        let vars = {};
        try { vars = JSON.parse(v); } catch(_) {}
        const res = await fetch('/graphql', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ query: q, variables: vars })
        });
        const data = await res.json();
        const ms = Math.round(performance.now() - start);
        timing.textContent = ms + 'ms';
        document.getElementById('result').textContent = JSON.stringify(data, null, 2);
        if (data.errors) {
          indicator.className = 'err';
          indicator.textContent = 'Error';
        } else {
          indicator.className = 'ok';
          indicator.textContent = 'Success';
        }
      } catch(e) {
        const ms = Math.round(performance.now() - start);
        timing.textContent = ms + 'ms';
        document.getElementById('result').textContent = 'Error: ' + e.message;
        indicator.className = 'err';
        indicator.textContent = 'Network Error';
      }
    }
    async function showSchema() {
      try {
        const res = await fetch('/graphql/schema');
        const text = await res.text();
        document.getElementById('result').textContent = text;
      } catch(e) {
        document.getElementById('result').textContent = 'Error: ' + e.message;
      }
    }
    document.getElementById('query').addEventListener('keydown', (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') { run(); }
    });
  </script>
</body>
</html>"#,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::access::test_support::*;

    fn run(q: &str, vars: Value, access: &Access, db: &CronusDB, ents: &[EntityNode]) -> Value {
        let schema = GraphQLSchema::from_entities(ents);
        execute_graphql(q, &vars, &schema, db, access, "data.db")
    }

    #[test]
    fn anonymous_requests_are_rejected() {
        let ents = entities();
        let db = db(&ents);
        db.insert(
            "User",
            &json!({"email":"a@b.co","password":"$argon2id$hash","role":"admin"}),
        )
        .unwrap();
        let out = run(
            "{ users { email password role } }",
            json!({}),
            &anon(),
            &db,
            &ents,
        );
        assert_eq!(out["errors"][0]["extensions"]["code"], "UNAUTHENTICATED");
        assert!(out.get("data").is_none());
        let out = run(
            "mutation { deleteNote(id: \"x\") }",
            json!({}),
            &anon(),
            &db,
            &ents,
        );
        assert_eq!(out["errors"][0]["extensions"]["code"], "UNAUTHENTICATED");
    }

    #[test]
    fn lists_and_by_id_are_owner_scoped() {
        let ents = entities();
        let db = db(&ents);
        let alice_note = insert_note(&db, "alice", "a1");
        insert_note(&db, "bob", "b1");

        let out = run(
            "{ notes { id title } }",
            json!({}),
            &as_user("bob"),
            &db,
            &ents,
        );
        let notes = out["data"]["notes"].as_array().unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0]["title"], "b1");

        let q = format!("{{ note(id: \"{}\") {{ title }} }}", alice_note);
        assert!(run(&q, json!({}), &as_user("bob"), &db, &ents)["data"]["note"].is_null());
        assert_eq!(
            run(&q, json!({}), &as_user("alice"), &db, &ents)["data"]["note"]["title"],
            "a1"
        );
        let all = run("{ notes { id } }", json!({}), &as_admin(), &db, &ents);
        assert_eq!(all["data"]["notes"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn sensitive_fields_absent_from_responses_and_schema() {
        let ents = entities();
        let db = db(&ents);
        insert_note(&db, "alice", "a1");
        let me = db
            .insert(
                "User",
                &json!({"email":"me@x.co","password":"$argon2id$me"}),
            )
            .unwrap();
        db.insert(
            "User",
            &json!({"email":"other@x.co","password":"$argon2id$other"}),
        )
        .unwrap();
        let me_id = me["id"].as_str().unwrap();

        let out = run(
            "{ notes { title secret } }",
            json!({}),
            &as_user("alice"),
            &db,
            &ents,
        );
        assert!(out["data"]["notes"][0].get("secret").is_none());
        let out = run("{ users }", json!({}), &as_user(me_id), &db, &ents);
        let users = out["data"]["users"].as_array().unwrap();
        assert_eq!(users.len(), 1, "a user only sees their own account");
        assert!(users[0].get("password").is_none());

        let sdl = GraphQLSchema::from_entities(&ents).sdl;
        assert!(!sdl.contains("secret"));
        assert!(!sdl.contains("password"));
        assert!(
            !sdl.contains("  role: String\n}\ninput"),
            "role is not writable"
        );
    }

    #[test]
    fn cross_owner_delete_is_refused() {
        let ents = entities();
        let db = db(&ents);
        let id = insert_note(&db, "alice", "a1");
        let q = format!("mutation {{ deleteNote(id: \"{}\") }}", id);
        assert_eq!(
            run(&q, json!({}), &as_user("bob"), &db, &ents)["data"]["deleteNote"],
            false
        );
        assert!(db.find_by_id("Note", &id).unwrap().is_some());
        assert_eq!(
            run(&q, json!({}), &as_user("alice"), &db, &ents)["data"]["deleteNote"],
            true
        );
    }

    #[test]
    fn create_sets_owner_server_side_and_drops_forbidden_fields() {
        let ents = entities();
        let db = db(&ents);
        let out = run(
            "mutation($input: CreateNoteInput!) { createNote(input: $input) { id } }",
            json!({"input": {"title": "t", "_owner_id": "alice", "secret": "leak", "id": "forced"}}),
            &as_user("bob"),
            &db,
            &ents,
        );
        let id = out["data"]["createNote"]["id"].as_str().expect("id");
        assert_ne!(id, "forced");
        let row = db.find_by_id("Note", id).unwrap().unwrap();
        assert_eq!(row["_owner_id"], "bob");
        assert!(row["secret"].is_null());

        let out = run(
            "mutation($input: CreateUserInput!) { createUser(input: $input) { id } }",
            json!({"input": {"email": "x@y.co"}}),
            &as_user("bob"),
            &db,
            &ents,
        );
        assert_eq!(out["errors"][0]["extensions"]["code"], "FORBIDDEN");
    }

    const CREATE_POST: &str =
        "mutation($input: CreatePostInput!) { createPost(input: $input) { id tags { id } } }";

    #[test]
    fn create_mutation_reports_field_errors() {
        let s = crate::api_validation_tests::state();
        let alice = crate::api_validation_tests::viewer("alice", "user");
        let out = run(
            CREATE_POST,
            json!({"input": {"title": "ab", "age": 999}}),
            &alice,
            &s.db,
            &s.entities,
        );
        let ext = &out["errors"][0]["extensions"];
        assert_eq!(ext["code"], "VALIDATION_FAILED", "{out}");
        assert_eq!(
            ext["fields"]["title"],
            json!(["must be at least 3 characters"])
        );
        assert_eq!(ext["fields"]["age"], json!(["must be at most 150"]));
        assert_eq!(s.db.count("Post").unwrap(), 0);

        s.db.insert("Post", &json!({"title": "taken", "contact": "a@b.co"}))
            .unwrap();
        let out = run(
            CREATE_POST,
            json!({"input": {"title": "dupe", "contact": "a@b.co"}}),
            &alice,
            &s.db,
            &s.entities,
        );
        assert_eq!(
            out["errors"][0]["extensions"]["fields"]["contact"],
            json!(["already exists"])
        );
    }

    #[test]
    fn many_to_many_is_a_list_field_scoped_to_the_caller() {
        let s = crate::api_validation_tests::state();
        let alice = crate::api_validation_tests::viewer("alice", "user");
        let sdl = GraphQLSchema::from_entities(&s.entities).sdl;
        assert!(
            sdl.contains("type Post {\n") && sdl.contains("  tags: [Tag!]!\n"),
            "{sdl}"
        );
        assert!(sdl.contains("input CreatePostInput {"), "{sdl}");
        assert!(sdl.contains("  tags: [String!]\n"), "{sdl}");

        let tag = |owner: &str| {
            s.db.insert("Tag", &json!({"label": "l", "_owner_id": owner}))
                .unwrap()["id"]
                .as_str()
                .unwrap()
                .to_string()
        };
        let (mine, theirs) = (tag("alice"), tag("bob"));
        let out = run(
            CREATE_POST,
            json!({"input": {"title": "steal", "tags": [theirs]}}),
            &alice,
            &s.db,
            &s.entities,
        );
        assert_eq!(
            out["errors"][0]["extensions"]["fields"]["tags"],
            json!(["contains an unknown id"])
        );
        let out = run(
            CREATE_POST,
            json!({"input": {"title": "linked", "tags": [mine.clone()]}}),
            &alice,
            &s.db,
            &s.entities,
        );
        assert_eq!(
            out["data"]["createPost"]["tags"],
            json!([{"id": mine.clone()}]),
            "{out}"
        );
        let listed = run(
            "{ posts { title tags { id } } }",
            json!({}),
            &alice,
            &s.db,
            &s.entities,
        );
        assert_eq!(listed["data"]["posts"][0]["tags"], json!([{"id": mine}]));
    }

    #[test]
    fn selection_expands_m2m_and_reverse_like_bind() {
        let s = crate::api_validation_tests::state();
        let alice = crate::api_validation_tests::viewer("alice", "user");
        let tag_id =
            s.db.insert("Tag", &json!({"label": "rust", "_owner_id": "alice"}))
                .unwrap()["id"]
                .as_str()
                .unwrap()
                .to_string();
        let created = run(
            CREATE_POST,
            json!({"input": {"title": "hello world", "tags": [tag_id]}}),
            &alice,
            &s.db,
            &s.entities,
        );
        assert!(created["data"]["createPost"]["id"].is_string(), "{created}");
        let sdl = GraphQLSchema::from_entities(&s.entities).sdl;
        assert!(sdl.contains("  posts: [Post!]!\n"), "{sdl}");
        let out = run(
            "{ tags { label posts { title } } }",
            json!({}),
            &alice,
            &s.db,
            &s.entities,
        );
        assert_eq!(out["data"]["tags"][0]["label"], "rust", "{out}");
        assert_eq!(
            out["data"]["tags"][0]["posts"][0]["title"], "hello world",
            "{out}"
        );
        let out = run(
            "{ posts { title tags { label } } }",
            json!({}),
            &alice,
            &s.db,
            &s.entities,
        );
        assert_eq!(out["data"]["posts"][0]["tags"][0]["label"], "rust", "{out}");
    }

    const UPDATE_NOTE: &str =
        "mutation($id: String!, $input: UpdateNoteInput!) { updateNote(id: $id, input: $input) { id title } }";

    #[test]
    fn update_mutation_is_owner_scoped_and_partial() {
        let ents = entities();
        let db = db(&ents);
        let id = insert_note(&db, "alice", "a1");
        insert_note(&db, "bob", "b1");
        let sdl = GraphQLSchema::from_entities(&ents).sdl;
        assert!(
            sdl.contains("updateNote(id: String!, input: UpdateNoteInput!): Note"),
            "{sdl}"
        );
        assert!(sdl.contains("input UpdateNoteInput {"), "{sdl}");

        let out = run(
            UPDATE_NOTE,
            json!({"id": id, "input": {"title": "renamed"}}),
            &as_user("bob"),
            &db,
            &ents,
        );
        assert!(out["data"]["updateNote"].is_null(), "{out}");
        assert_eq!(db.find_by_id("Note", &id).unwrap().unwrap()["title"], "a1");

        let out = run(
            UPDATE_NOTE,
            json!({"id": id, "input": {"title": "renamed"}}),
            &as_user("alice"),
            &db,
            &ents,
        );
        assert_eq!(out["data"]["updateNote"]["title"], "renamed", "{out}");
        assert_eq!(
            db.find_by_id("Note", &id).unwrap().unwrap()["title"],
            "renamed"
        );

        let out = run(
            "mutation($id: String!, $input: UpdateUserInput!) { updateUser(id: $id, input: $input) { id } }",
            json!({"id": "x", "input": {"email": "n@x.co"}}),
            &as_user("bob"),
            &db,
            &ents,
        );
        assert_eq!(out["errors"][0]["extensions"]["code"], "FORBIDDEN");
    }

    #[test]
    fn update_mutation_reports_field_errors() {
        let s = crate::api_validation_tests::state();
        let alice = crate::api_validation_tests::viewer("alice", "user");
        let id =
            s.db.insert(
                "Post",
                &json!({"title": "hello", "age": 20, "_owner_id": "alice"}),
            )
            .unwrap()["id"]
                .as_str()
                .unwrap()
                .to_string();
        let out = run(
            "mutation($id: String!, $input: UpdatePostInput!) { updatePost(id: $id, input: $input) { id } }",
            json!({"id": id, "input": {"title": "ab", "age": 999}}),
            &alice,
            &s.db,
            &s.entities,
        );
        let ext = &out["errors"][0]["extensions"];
        assert_eq!(ext["code"], "VALIDATION_FAILED", "{out}");
        assert_eq!(
            ext["fields"]["title"],
            json!(["must be at least 3 characters"])
        );
        assert_eq!(ext["fields"]["age"], json!(["must be at most 150"]));
    }
}
