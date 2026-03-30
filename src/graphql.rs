#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS GraphQL Engine — Auto-generated from EntityNode
//!
//! - GET  /graphql  → Playground HTML
//! - POST /graphql  → Execute queries/mutations
//!
//! Auto-generates types, queries, and mutations from entities.
//! Resolves via CronusDB.

use std::sync::Arc;
use serde_json::{json, Value, Map};

use crate::database::CronusDB;
use crate::parser::{EntityNode, FieldType};

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
            sdl.push_str(&generate_type(entity));
            sdl.push('\n');
            sdl.push_str(&generate_create_input(entity));
            sdl.push('\n');
        }

        // Generate Query type
        sdl.push_str("type Query {\n");
        for entity in entities {
            let lower = entity.name.to_lowercase();
            let plural = format!("{}s", lower);
            sdl.push_str(&format!("  {plural}(limit: Int): [{name}!]!\n", name = entity.name));
            sdl.push_str(&format!("  {lower}(id: String!): {name}\n", name = entity.name));
        }
        sdl.push_str("}\n\n");

        // Generate Mutation type
        sdl.push_str("type Mutation {\n");
        for entity in entities {
            let lower = entity.name.to_lowercase();
            sdl.push_str(&format!(
                "  create{name}(input: Create{name}Input!): {name}!\n",
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

fn generate_type(entity: &EntityNode) -> String {
    let mut out = format!("type {} {{\n", entity.name);
    out.push_str("  id: String!\n");
    for field in &entity.fields {
        let gql_type = if field.enum_values.is_some() {
            "String"
        } else {
            field_type_to_graphql(&field.field_type)
        };
        let bang = if field.required { "!" } else { "" };
        out.push_str(&format!("  {}: {}{}\n", field.name, gql_type, bang));
    }
    out.push_str("  created_at: String\n");
    out.push_str("  updated_at: String\n");
    out.push_str("}\n");
    out
}

fn generate_create_input(entity: &EntityNode) -> String {
    let mut out = format!("input Create{}Input {{\n", entity.name);
    for field in &entity.fields {
        let gql_type = if field.enum_values.is_some() {
            "String"
        } else {
            field_type_to_graphql(&field.field_type)
        };
        let bang = if field.required { "!" } else { "" };
        out.push_str(&format!("  {}: {}{}\n", field.name, gql_type, bang));
    }
    out.push_str("}\n");
    out
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
    sub_fields: Vec<String>,
}

#[derive(Debug)]
enum ArgValue {
    StringVal(String),
    IntVal(i64),
    BoolVal(bool),
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
        while chars.peek().map_or(false, |c| c.is_alphanumeric() || *c == '_') {
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
            // Parse inner as simple field names (no nesting for now)
            for token in inner.split_whitespace() {
                let clean = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                if !clean.is_empty() {
                    sub_fields.push(clean.to_string());
                }
            }
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
        while chars.peek().map_or(false, |c| c.is_whitespace() || *c == ',') {
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
        while chars.peek().map_or(false, |c| c.is_alphanumeric() || *c == '_') {
            key.push(chars.next().unwrap());
        }

        // Skip ':'
        while chars.peek().map_or(false, |c| c.is_whitespace() || *c == ':') {
            chars.next();
        }

        // Read value
        let val = parse_arg_value(chars)?;
        args.push((key, val));
    }

    Ok(args)
}

fn parse_arg_value(
    chars: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<ArgValue, String> {
    // Skip whitespace
    while chars.peek().map_or(false, |c| c.is_whitespace()) {
        chars.next();
    }

    match chars.peek() {
        Some('$') => {
            chars.next(); // consume $
            let mut name = String::new();
            while chars.peek().map_or(false, |c| c.is_alphanumeric() || *c == '_') {
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
            while chars.peek().map_or(false, |c| c.is_ascii_digit() || *c == '-') {
                num.push(chars.next().unwrap());
            }
            Ok(ArgValue::IntVal(
                num.parse().map_err(|_| "invalid number")?,
            ))
        }
        Some('t') | Some('f') => {
            let mut word = String::new();
            while chars.peek().map_or(false, |c| c.is_alphabetic()) {
                word.push(chars.next().unwrap());
            }
            Ok(ArgValue::BoolVal(word == "true"))
        }
        _ => Err("unexpected arg value".into()),
    }
}

// ══════════════════════════════════════════════════
// EXECUTOR
// ══════════════════════════════════════════════════

/// Execute a GraphQL request. Returns JSON response { data, errors }.
pub fn execute_graphql(
    query: &str,
    variables: &Value,
    schema: &GraphQLSchema,
    db: &Arc<CronusDB>,
) -> Value {
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
                if let Some(result) = resolve_query(field, schema, db, variables) {
                    data.insert(field.name.clone(), result);
                } else {
                    errors.push(json!({
                        "message": format!("Unknown field: {}", field.name)
                    }));
                }
            }
            Operation::Mutation => {
                match resolve_mutation(field, schema, db, variables) {
                    Ok(result) => {
                        data.insert(field.name.clone(), result);
                    }
                    Err(e) => {
                        errors.push(json!({ "message": e }));
                    }
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

fn resolve_query(
    field: &ParsedField,
    schema: &GraphQLSchema,
    db: &Arc<CronusDB>,
    _variables: &Value,
) -> Option<Value> {
    // Check if this is a list query (e.g. "users", "products")
    for entity in &schema.entities {
        let lower = entity.name.to_lowercase();
        let plural = format!("{}s", lower);

        if field.name == plural {
            // List query
            let limit = field
                .args
                .iter()
                .find(|(k, _)| k == "limit")
                .and_then(|(_, v)| match v {
                    ArgValue::IntVal(n) => Some(*n as usize),
                    _ => None,
                })
                .unwrap_or(100);

            match db.find_all(&entity.name, limit, 0) {
                Ok(rows) => {
                    if field.sub_fields.is_empty() {
                        return Some(rows);
                    }
                    // Filter to requested fields
                    return Some(filter_fields_array(&rows, &field.sub_fields));
                }
                Err(_) => return Some(json!([])),
            }
        }

        if field.name == lower {
            // Single query
            let id = field
                .args
                .iter()
                .find(|(k, _)| k == "id")
                .and_then(|(_, v)| match v {
                    ArgValue::StringVal(s) => Some(s.clone()),
                    _ => None,
                });

            if let Some(id) = id {
                match db.find_by_id(&entity.name, &id) {
                    Ok(Some(row)) => {
                        if field.sub_fields.is_empty() {
                            return Some(row);
                        }
                        return Some(filter_fields_object(&row, &field.sub_fields));
                    }
                    _ => return Some(Value::Null),
                }
            }
            return Some(Value::Null);
        }
    }

    None
}

fn resolve_mutation(
    field: &ParsedField,
    schema: &GraphQLSchema,
    db: &Arc<CronusDB>,
    variables: &Value,
) -> Result<Value, String> {
    for entity in &schema.entities {
        let create_name = format!("create{}", entity.name);
        let delete_name = format!("delete{}", entity.name);

        if field.name == create_name {
            // Get input from args or variables
            let input = field
                .args
                .iter()
                .find(|(k, _)| k == "input")
                .and_then(|(_, v)| match v {
                    ArgValue::Variable(var_name) => {
                        variables.get(var_name).cloned()
                    }
                    _ => None,
                })
                .unwrap_or_else(|| {
                    // Try to get from variables directly
                    variables
                        .get("input")
                        .cloned()
                        .unwrap_or(json!({}))
                });

            return db.insert(&entity.name, &input);
        }

        if field.name == delete_name {
            let id = field
                .args
                .iter()
                .find(|(k, _)| k == "id")
                .and_then(|(_, v)| match v {
                    ArgValue::StringVal(s) => Some(s.clone()),
                    _ => None,
                })
                .ok_or("delete requires id argument")?;

            let deleted = db.delete(&entity.name, &id)?;
            return Ok(json!(deleted));
        }
    }

    Err(format!("Unknown mutation: {}", field.name))
}

fn filter_fields_array(arr: &Value, fields: &[String]) -> Value {
    match arr.as_array() {
        Some(items) => {
            Value::Array(items.iter().map(|item| filter_fields_object(item, fields)).collect())
        }
        None => arr.clone(),
    }
}

fn filter_fields_object(obj: &Value, fields: &[String]) -> Value {
    match obj.as_object() {
        Some(map) => {
            let mut filtered = Map::new();
            for f in fields {
                if let Some(val) = map.get(f) {
                    filtered.insert(f.clone(), val.clone());
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
</html>"#.to_string()
}
