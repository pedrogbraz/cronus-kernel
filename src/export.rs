//! CRONUS Export Engine
//!
//! Exports project AST in multiple portable formats:
//! json, openapi, sql, typescript.

use crate::parser::*;
use serde_json::{json, Value};

// ══════════════════════════════════════════════════
// JSON EXPORT — Full AST dump
// ══════════════════════════════════════════════════

pub fn export_json(nodes: &[AstNode]) -> String {
    let mut app_info = json!(null);
    let mut entities = Vec::new();
    let mut pages = Vec::new();
    let mut apis = Vec::new();
    let mut webhooks = Vec::new();
    let mut style = json!(null);
    let mut constitution = json!(null);

    for node in nodes {
        match node {
            AstNode::App(a) => {
                let mut obj = json!({
                    "name": a.name,
                    "stack": a.stack,
                    "port": a.port,
                });
                if let Some(ref db) = a.database {
                    obj["database"] = json!({
                        "type": db.db_type,
                        "path": db.path,
                    });
                }
                if let Some(ref c) = a.constitution {
                    constitution = json!({
                        "must": c.must,
                        "never": c.never,
                    });
                }
                app_info = obj;
            }
            AstNode::Entity(e) => {
                entities.push(entity_to_json(e));
            }
            AstNode::Api(a) => {
                apis.push(api_to_json(a));
            }
            AstNode::Page(p) => {
                pages.push(page_to_json(p));
            }
            AstNode::Webhook(w) => {
                webhooks.push(webhook_to_json(w));
            }
            AstNode::Style(s) => {
                style = style_to_json(s);
            }
            _ => {}
        }
    }

    let output = json!({
        "cronus_version": "1.0",
        "app": app_info,
        "entities": entities,
        "pages": pages,
        "apis": apis,
        "webhooks": webhooks,
        "style": style,
        "constitution": constitution,
    });

    serde_json::to_string_pretty(&output).unwrap()
}

fn entity_to_json(e: &EntityNode) -> Value {
    let fields: Vec<Value> = e
        .fields
        .iter()
        .map(|f| {
            let mut obj = json!({
                "name": f.name,
                "type": field_type_str(&f.field_type),
                "required": f.required,
                "optional": f.optional,
            });
            if f.unique {
                obj["unique"] = json!(true);
            }
            if f.sensitive {
                obj["sensitive"] = json!(true);
            }
            if f.searchable {
                obj["searchable"] = json!(true);
            }
            if f.index {
                obj["index"] = json!(true);
            }
            if f.featured {
                obj["featured"] = json!(true);
            }
            if f.formatted {
                obj["formatted"] = json!(true);
            }
            if f.array {
                obj["array"] = json!(true);
            }
            if let Some(ref vals) = f.enum_values {
                obj["enum_values"] = json!(vals);
            }
            if let Some(ref r) = f.reference {
                obj["reference"] = json!(r);
            }
            if let Some(ref d) = f.doc {
                obj["doc"] = json!(d.summary);
            }
            obj
        })
        .collect();

    let mut obj = json!({
        "name": e.name,
        "fields": fields,
        "shared": e.shared,
    });
    if let Some(ref d) = e.doc {
        obj["doc"] = json!(d.summary);
    }
    if !e.transitions.is_empty() {
        let transitions: Vec<Value> = e
            .transitions
            .iter()
            .map(|t| {
                json!({
                    "field": t.field,
                    "rules": t.rules.iter().map(|r| json!({
                        "from": r.from,
                        "to": r.to,
                    })).collect::<Vec<Value>>(),
                })
            })
            .collect();
        obj["transitions"] = json!(transitions);
    }
    obj
}

fn api_to_json(a: &ApiNode) -> Value {
    let routes: Vec<Value> = a
        .routes
        .iter()
        .map(|r| {
            let mut obj = json!({
                "name": r.name,
                "method": format!("{:?}", r.method),
                "path": r.path,
                "auth": r.auth,
            });
            if !r.roles.is_empty() {
                obj["roles"] = json!(r.roles);
            }
            if let Some(ref d) = r.doc {
                obj["doc"] = json!(d.summary);
            }
            obj
        })
        .collect();

    let mut obj = json!({
        "prefix": a.prefix,
        "routes": routes,
    });
    if let Some(ref d) = a.doc {
        obj["doc"] = json!(d.summary);
    }
    obj
}

fn page_to_json(p: &PageNode) -> Value {
    let mut obj = json!({
        "route": p.route,
        "type": p.page_type,
    });
    if let Some(ref e) = p.entity {
        obj["entity"] = json!(e);
    }
    if let Some(ref t) = p.title {
        obj["title"] = json!(t);
    }
    if let Some(ref r) = p.requires {
        obj["requires"] = json!(r);
    }
    if !p.components.is_empty() {
        obj["components"] = json!(p.components);
    }
    if let Some(ref d) = p.doc {
        obj["doc"] = json!(d.summary);
    }
    obj
}

fn webhook_to_json(w: &WebhookNode) -> Value {
    let hooks: Vec<Value> = w
        .hooks
        .iter()
        .map(|h| {
            json!({
                "event": h.event,
                "method": h.method,
                "url": h.url,
                "headers": h.headers.iter().map(|(k, v)| json!({k: v})).collect::<Vec<_>>(),
            })
        })
        .collect();

    json!({
        "entity": w.entity,
        "hooks": hooks,
    })
}

fn style_to_json(s: &StyleNode) -> Value {
    let mut obj = json!({});
    if let Some(ref t) = s.theme {
        obj["theme"] = json!(t);
    }
    if let Some(ref a) = s.accent {
        obj["accent"] = json!(a);
    }
    if let Some(ref r) = s.radius {
        obj["radius"] = json!(r);
    }
    if let Some(ref f) = s.font {
        obj["font"] = json!(f);
    }
    if !s.config.is_empty() {
        obj["config"] = json!(s.config);
    }
    obj
}

// ══════════════════════════════════════════════════
// OPENAPI EXPORT — OpenAPI 3.0 spec from APIs
// ══════════════════════════════════════════════════

pub fn export_openapi(nodes: &[AstNode]) -> String {
    let mut app_name = "CRONUS App".to_string();
    let mut paths = serde_json::Map::new();
    let mut entities: Vec<&EntityNode> = Vec::new();

    for node in nodes {
        match node {
            AstNode::App(a) => app_name = a.name.clone(),
            AstNode::Entity(e) => entities.push(e),
            AstNode::Api(a) => {
                for route in &a.routes {
                    // Route path already contains the full path (e.g., "/deployments/:id")
                    // Just prepend /api
                    let full_path = format!("/api{}", route.path);

                    let method_str = match route.method {
                        HttpMethod::GET => "get",
                        HttpMethod::POST => "post",
                        HttpMethod::PATCH => "patch",
                        HttpMethod::PUT => "put",
                        HttpMethod::DELETE => "delete",
                    };

                    let mut operation = json!({
                        "summary": route.name,
                    });

                    if route.auth != "none" && !route.auth.is_empty() {
                        operation["security"] = json!([{"bearerAuth": []}]);
                    }

                    // Infer response description from method
                    let resp_desc = match route.method {
                        HttpMethod::GET => "Successful response",
                        HttpMethod::POST => "Created",
                        HttpMethod::PATCH | HttpMethod::PUT => "Updated",
                        HttpMethod::DELETE => "Deleted",
                    };

                    let resp_code = match route.method {
                        HttpMethod::POST => "201",
                        HttpMethod::DELETE => "204",
                        _ => "200",
                    };

                    operation["responses"] = json!({
                        resp_code: {
                            "description": resp_desc
                        }
                    });

                    if let Some(ref d) = route.doc {
                        if !d.summary.is_empty() {
                            operation["description"] = json!(d.summary);
                        }
                    }

                    // Add to paths
                    let path_entry = paths.entry(full_path).or_insert_with(|| json!({}));
                    path_entry[method_str] = operation;
                }
            }
            _ => {}
        }
    }

    // Build schemas from entities
    let mut schemas = serde_json::Map::new();
    for e in &entities {
        let mut properties = serde_json::Map::new();
        let mut required_fields = Vec::new();

        // Always include id, created_at, updated_at
        properties.insert("id".to_string(), json!({"type": "string"}));
        properties.insert(
            "created_at".to_string(),
            json!({"type": "string", "format": "date-time"}),
        );
        properties.insert(
            "updated_at".to_string(),
            json!({"type": "string", "format": "date-time"}),
        );
        required_fields.push(json!("id"));

        for f in &e.fields {
            let prop = field_to_openapi_property(f);
            if f.required && !f.optional {
                required_fields.push(json!(f.name));
            }
            properties.insert(f.name.clone(), prop);
        }

        schemas.insert(
            e.name.clone(),
            json!({
                "type": "object",
                "properties": properties,
                "required": required_fields,
            }),
        );
    }

    let spec = json!({
        "openapi": "3.0.0",
        "info": {
            "title": app_name,
            "version": "1.0.0"
        },
        "paths": paths,
        "components": {
            "schemas": schemas,
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            }
        }
    });

    serde_json::to_string_pretty(&spec).unwrap()
}

fn field_to_openapi_property(f: &FieldNode) -> Value {
    let base_type = match f.field_type {
        FieldType::String | FieldType::Text | FieldType::Slug => json!({"type": "string"}),
        FieldType::Email => json!({"type": "string", "format": "email"}),
        FieldType::Url => json!({"type": "string", "format": "uri"}),
        FieldType::File => json!({"type": "string", "format": "uri"}),
        FieldType::Phone => json!({"type": "string"}),
        FieldType::Number => json!({"type": "integer"}),
        FieldType::Money => json!({"type": "integer", "description": "Amount in cents"}),
        FieldType::Percentage => json!({"type": "number"}),
        FieldType::Boolean => json!({"type": "boolean"}),
        FieldType::Date => json!({"type": "string", "format": "date"}),
        FieldType::DateTime => json!({"type": "string", "format": "date-time"}),
        FieldType::Ulid => json!({"type": "string", "format": "ulid"}),
        FieldType::Json => json!({"type": "object"}),
        FieldType::Ip => json!({"type": "string", "format": "ipv4"}),
        FieldType::Enum => {
            if let Some(ref vals) = f.enum_values {
                json!({"type": "string", "enum": vals})
            } else {
                json!({"type": "string"})
            }
        }
        FieldType::Relation => {
            json!({"type": "string", "description": format!("Reference to {}", f.reference.as_deref().unwrap_or("unknown"))})
        }
    };

    if f.array {
        json!({"type": "array", "items": base_type})
    } else {
        base_type
    }
}

// ══════════════════════════════════════════════════
// SQL EXPORT — CREATE TABLE statements
// ══════════════════════════════════════════════════

pub fn export_sql(nodes: &[AstNode]) -> String {
    let mut output = String::new();
    output.push_str("-- CRONUS SQL Schema Export\n");
    output.push_str("-- Generated from .cronus definition\n\n");

    for node in nodes {
        if let AstNode::Entity(e) = node {
            output.push_str(&entity_to_sql(e));
            output.push('\n');
        }
    }

    output
}

fn entity_to_sql(e: &EntityNode) -> String {
    let table_name = to_snake_case(&e.name);
    let mut lines = Vec::new();

    // Primary key
    lines.push("  id TEXT PRIMARY KEY NOT NULL".to_string());

    for f in &e.fields {
        let col_name = to_snake_case(&f.name);
        let sql_type = sql_type_for(&f.field_type);

        let mut col = format!("  {} {}", col_name, sql_type);

        if f.required && !f.optional {
            col.push_str(" NOT NULL");
        }

        if f.unique {
            col.push_str(" UNIQUE");
        }

        // Default for booleans
        if f.field_type == FieldType::Boolean {
            col.push_str(" DEFAULT 0");
        }

        lines.push(col);
    }

    // Timestamps
    lines.push("  created_at TEXT NOT NULL DEFAULT (datetime('now'))".to_string());
    lines.push("  updated_at TEXT NOT NULL DEFAULT (datetime('now'))".to_string());

    let mut stmt = format!("CREATE TABLE IF NOT EXISTS {} (\n", table_name);
    stmt.push_str(&lines.join(",\n"));
    stmt.push_str("\n);\n");

    // Indexes for searchable/index fields
    for f in &e.fields {
        if f.searchable || f.index {
            let col_name = to_snake_case(&f.name);
            stmt.push_str(&format!(
                "CREATE INDEX IF NOT EXISTS idx_{}_{} ON {} ({});\n",
                table_name, col_name, table_name, col_name
            ));
        }
    }

    stmt
}

fn sql_type_for(ft: &FieldType) -> &'static str {
    match ft {
        FieldType::Number | FieldType::Money | FieldType::Percentage => "INTEGER",
        FieldType::Boolean => "INTEGER",
        FieldType::Json => "TEXT",
        FieldType::String
        | FieldType::Text
        | FieldType::Email
        | FieldType::Url
        | FieldType::Slug
        | FieldType::Phone
        | FieldType::Date
        | FieldType::DateTime
        | FieldType::File
        | FieldType::Ulid
        | FieldType::Enum
        | FieldType::Ip
        | FieldType::Relation => "TEXT",
    }
}

// ══════════════════════════════════════════════════
// TYPESCRIPT EXPORT — Interfaces for all entities
// ══════════════════════════════════════════════════

pub fn export_typescript(nodes: &[AstNode]) -> String {
    let mut output = String::new();
    output.push_str("// CRONUS TypeScript Interfaces\n");
    output.push_str("// Generated from .cronus definition\n\n");

    for node in nodes {
        if let AstNode::Entity(e) = node {
            output.push_str(&entity_to_typescript(e));
            output.push('\n');
        }
    }

    output
}

fn entity_to_typescript(e: &EntityNode) -> String {
    let mut lines = Vec::new();

    if let Some(ref d) = e.doc {
        if !d.summary.is_empty() {
            lines.push(format!("/** {} */", d.summary));
        }
    }

    lines.push(format!("export interface {} {{", e.name));

    // id is always present
    lines.push("  id: string;".to_string());

    for f in &e.fields {
        let ts_type = field_to_ts_type(f);
        let optional = if f.optional { "?" } else { "" };

        // Build JSDoc with constraints
        let mut jsdoc_parts: Vec<String> = Vec::new();
        if let Some(ref d) = f.doc {
            if !d.summary.is_empty() {
                jsdoc_parts.push(d.summary.clone());
            }
        }
        if let Some(min_val) = f.min {
            jsdoc_parts.push(format!("@minimum {}", min_val));
        }
        if let Some(max_val) = f.max {
            jsdoc_parts.push(format!("@maximum {}", max_val));
        }
        if let Some(min_len) = f.min_length {
            jsdoc_parts.push(format!("@minLength {}", min_len));
        }
        if let Some(max_len) = f.max_length {
            jsdoc_parts.push(format!("@maxLength {}", max_len));
        }
        if let Some(ref pat) = f.pattern {
            jsdoc_parts.push(format!("@pattern {}", pat));
        }

        if !jsdoc_parts.is_empty() {
            if jsdoc_parts.len() == 1 {
                lines.push(format!("  /** {} */", jsdoc_parts[0]));
            } else {
                lines.push("  /**".to_string());
                for part in &jsdoc_parts {
                    lines.push(format!("   * {}", part));
                }
                lines.push("   */".to_string());
            }
        }

        lines.push(format!("  {}{}: {};", f.name, optional, ts_type));
    }

    // Timestamps
    lines.push("  created_at: string;".to_string());
    lines.push("  updated_at: string;".to_string());
    lines.push("}".to_string());

    lines.join("\n") + "\n"
}

fn field_to_ts_type(f: &FieldNode) -> String {
    let base = match f.field_type {
        FieldType::String
        | FieldType::Text
        | FieldType::Email
        | FieldType::Url
        | FieldType::Slug
        | FieldType::Phone
        | FieldType::Date
        | FieldType::DateTime
        | FieldType::File
        | FieldType::Ulid
        | FieldType::Ip => "string".to_string(),
        FieldType::Number | FieldType::Money | FieldType::Percentage => "number".to_string(),
        FieldType::Boolean => "boolean".to_string(),
        FieldType::Json => "Record<string, unknown>".to_string(),
        FieldType::Enum => {
            if let Some(ref vals) = f.enum_values {
                vals.iter()
                    .map(|v| format!("\"{}\"", v))
                    .collect::<Vec<_>>()
                    .join(" | ")
            } else {
                "string".to_string()
            }
        }
        FieldType::Relation => "string".to_string(),
    };

    if f.array {
        format!("{}[]", base)
    } else {
        base
    }
}

// ══════════════════════════════════════════════════
// HELPERS
// ══════════════════════════════════════════════════

fn field_type_str(ft: &FieldType) -> &'static str {
    match ft {
        FieldType::String => "string",
        FieldType::Text => "text",
        FieldType::Email => "email",
        FieldType::Url => "url",
        FieldType::Slug => "slug",
        FieldType::Phone => "phone",
        FieldType::Number => "number",
        FieldType::Money => "money",
        FieldType::Percentage => "percentage",
        FieldType::Boolean => "boolean",
        FieldType::Date => "date",
        FieldType::DateTime => "datetime",
        FieldType::File => "file",
        FieldType::Ulid => "ulid",
        FieldType::Json => "json",
        FieldType::Enum => "enum",
        FieldType::Ip => "ip",
        FieldType::Relation => "relation",
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}
