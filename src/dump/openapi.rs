#![allow(dead_code, unused_imports, unused_variables)]
//! OpenAPI/Swagger JSON → .cronus converter
//!
//! Parses an OpenAPI 3.x or Swagger 2.x spec and emits a valid .cronus file
//! with entities (from components.schemas) and API routes (from paths).

use serde_json::Value;
use std::collections::BTreeMap;

/// Main entry point: takes raw JSON string, returns .cronus source
pub fn dump_openapi(json_str: &str) -> String {
    let doc: Value = serde_json::from_str(json_str).unwrap_or_default();
    let mut out = String::new();

    // --- App name from info.title ---
    let app_name = doc["info"]["title"]
        .as_str()
        .unwrap_or("App")
        .replace(' ', "")
        .replace('-', "");

    let description = doc["info"]["description"].as_str().unwrap_or_default();

    out.push_str(&format!("app \"{}\" {{\n", app_name));
    out.push_str("  port 5175\n");
    out.push_str("  theme dark\n");
    out.push_str("}\n\n");

    // --- Entities from components.schemas (OpenAPI 3.x) or definitions (Swagger 2.x) ---
    let schemas = if doc["components"]["schemas"].is_object() {
        &doc["components"]["schemas"]
    } else if doc["definitions"].is_object() {
        &doc["definitions"]
    } else {
        &Value::Null
    };

    if let Some(schemas_map) = schemas.as_object() {
        for (name, schema) in schemas_map {
            emit_entity(&mut out, name, schema);
        }
    }

    // --- API routes from paths ---
    if let Some(paths) = doc["paths"].as_object() {
        // Group routes by first path segment
        let mut groups: BTreeMap<String, Vec<RouteInfo>> = BTreeMap::new();

        for (path, methods) in paths {
            let group_name = extract_group_name(path);
            if let Some(methods_map) = methods.as_object() {
                for (method, op) in methods_map {
                    let method_upper = method.to_uppercase();
                    if !["GET", "POST", "PUT", "PATCH", "DELETE"].contains(&method_upper.as_str()) {
                        continue;
                    }
                    let op_id = op["operationId"].as_str().unwrap_or_default().to_string();
                    let summary = op["summary"].as_str().unwrap_or_default().to_string();
                    let auth = detect_auth(op, &doc);

                    groups
                        .entry(group_name.clone())
                        .or_default()
                        .push(RouteInfo {
                            method: method_upper,
                            path: path.clone(),
                            op_id,
                            summary,
                            auth,
                        });
                }
            }
        }

        for (group, routes) in &groups {
            emit_api_group(&mut out, group, routes);
        }
    }

    out
}

// ---------------------------------------------------------------------------
// Entity emission
// ---------------------------------------------------------------------------

fn emit_entity(out: &mut String, name: &str, schema: &Value) {
    out.push_str(&format!("entity {} {{\n", name));

    let properties = schema["properties"].as_object();
    let required: Vec<&str> = schema["required"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    if let Some(props) = properties {
        for (field_name, field_schema) in props {
            let cronus_type = map_type(field_schema);
            let is_required = required.contains(&field_name.as_str());
            let req_marker = if is_required { " required" } else { "" };
            out.push_str(&format!("  {} {}{}\n", field_name, cronus_type, req_marker));
        }
    }

    out.push_str("}\n\n");
}

fn map_type(schema: &Value) -> String {
    // Handle $ref
    if let Some(ref_path) = schema["$ref"].as_str() {
        let ref_name = ref_path.rsplit('/').next().unwrap_or("Unknown");
        return format!("-> {}", ref_name);
    }

    // Handle arrays
    if schema["type"].as_str() == Some("array") {
        if let Some(items) = schema.get("items") {
            let inner = map_type(items);
            return format!("[{}]", inner);
        }
        return "[string]".to_string();
    }

    let type_str = schema["type"].as_str().unwrap_or("string");
    let format_str = schema["format"].as_str().unwrap_or("");

    // Handle enums
    if schema["enum"].is_array() {
        let variants: Vec<&str> = schema["enum"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        return format!("enum({})", variants.join(", "));
    }

    match (type_str, format_str) {
        ("string", "email") => "email".to_string(),
        ("string", "uri") | ("string", "url") => "url".to_string(),
        ("string", "date-time") | ("string", "date") => "date".to_string(),
        ("string", "uuid") => "string".to_string(),
        ("string", "password") => "string".to_string(),
        ("string", _) => "string".to_string(),
        ("integer", _) | ("number", _) => "number".to_string(),
        ("boolean", _) => "boolean".to_string(),
        ("object", _) => "json".to_string(),
        _ => "string".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Route emission
// ---------------------------------------------------------------------------

struct RouteInfo {
    method: String,
    path: String,
    op_id: String,
    summary: String,
    auth: String,
}

fn extract_group_name(path: &str) -> String {
    // Return the prefix up to and including the first segment as a path
    // e.g. "/api/products/{id}" → "/api/products"
    //      "/products/{id}"     → "/products"
    let trimmed = path.trim_start_matches('/');
    let segments: Vec<&str> = trimmed.split('/').collect();

    // Find the last non-parameter segment to use as group
    // "/api/products/{id}" → group = "/api/products", relative = "/:id"
    let mut prefix_parts = Vec::new();
    for seg in &segments {
        if seg.starts_with('{') || seg.starts_with(':') {
            break;
        }
        prefix_parts.push(*seg);
    }

    if prefix_parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", prefix_parts.join("/"))
    }
}

fn detect_auth(operation: &Value, doc: &Value) -> String {
    // Check operation-level security
    if let Some(security) = operation["security"].as_array() {
        if security.is_empty() {
            return "auth:public".to_string();
        }
        for sec in security {
            if let Some(obj) = sec.as_object() {
                for key in obj.keys() {
                    let key_lower = key.to_lowercase();
                    if key_lower.contains("bearer")
                        || key_lower.contains("jwt")
                        || key_lower.contains("oauth")
                    {
                        return "auth:jwt".to_string();
                    }
                }
            }
        }
        return "auth:jwt".to_string();
    }

    // Check top-level security
    if doc["security"].is_array() {
        return "auth:jwt".to_string();
    }

    "auth:public".to_string()
}

fn emit_api_group(out: &mut String, group: &str, routes: &[RouteInfo]) {
    out.push_str(&format!("api {} {{\n", group));

    for route in routes {
        let method_upper = route.method.to_uppercase();
        let route_name = if !route.op_id.is_empty() {
            route.op_id.clone()
        } else {
            let method_lower = route.method.to_lowercase();
            format!("{}_{}", method_lower, group.trim_start_matches('/'))
        };
        let comment = if !route.summary.is_empty() {
            format!("  // {}", route.summary)
        } else {
            String::new()
        };

        // Convert path params {id} to :id
        let full_path = route.path.replace('{', ":").replace('}', "");

        // Make path relative to the group prefix
        let relative_path = full_path.strip_prefix(group).unwrap_or(&full_path);
        let relative_path = if relative_path.is_empty() {
            "/"
        } else {
            relative_path
        };

        // Parser expects: name METHOD path auth:...
        out.push_str(&format!(
            "  {}    {}    {}    {}{}\n",
            route_name, method_upper, relative_path, route.auth, comment
        ));
    }

    out.push_str("}\n\n");
}
