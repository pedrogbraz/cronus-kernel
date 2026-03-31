#![allow(dead_code, unused_imports, unused_variables)]
//! TypeScript/JavaScript → .cronus entity extractor
//!
//! Scans TS/JS source files for interface/type declarations and
//! emits .cronus entity blocks with semantic field type detection.
//! v2: path-aware filtering to keep only domain entities (~30 vs 367).

use std::collections::HashMap;
use std::path::Path;

pub struct TsEntity {
    pub name: String,
    pub fields: Vec<TsField>,
}

pub struct TsField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
}

// Suffixes that indicate internal/framework types (skip unless in types/ or contracts/)
const SKIP_SUFFIXES: &[&str] = &[
    "Props", "State", "Actions", "ContextValue", "Options", "Params",
    "Response", "Request", "Input", "Output", "Hook", "Store",
    "Handler", "Listener", "Callback", "Ref", "Element", "Style",
    "Theme", "Animation", "Variant", "Config",
];

// Directories to skip entirely
const SKIP_DIRS: &[&str] = &[
    "src/components", "src/hooks", "src/game", "src/prompts",
    "src/parsers", "src/kernel/scaffold", "src/kernel/visualization",
    "src/kernel/blocks", "src/kernel/blueprints",
];

/// Check if a file path is in a domain-relevant location
pub fn is_domain_file(path: &Path) -> bool {
    let p = path.to_string_lossy();

    // Skip blacklisted directories
    for skip in SKIP_DIRS {
        if p.contains(skip) {
            return false;
        }
    }

    // Whitelist: always include these
    if p.contains("src/types") || p.contains("src/contracts") {
        return true;
    }

    // Include files with domain-relevant names
    let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
    if file_name.contains("types")
        || file_name.contains("models")
        || file_name.contains("entities")
        || file_name.contains("schema")
        || file_name == "api.ts"
        || file_name == "api.tsx"
    {
        return true;
    }

    // Include src/lib/api* files
    if p.contains("src/lib/") && file_name.starts_with("api") {
        return true;
    }

    false
}

/// Check if a file is in a privileged directory (types/ or contracts/)
fn is_privileged_dir(path: &Path) -> bool {
    let p = path.to_string_lossy();
    p.contains("src/types") || p.contains("src/contracts")
}

/// Extract entity-like interfaces/types from TypeScript source code
/// with optional path-based filtering
pub fn extract_entities_from_file(source: &str, file_path: &Path) -> Vec<TsEntity> {
    let privileged = is_privileged_dir(file_path);
    let mut entities = Vec::new();

    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Detect interface or type declaration
        let name = if let Some(n) = extract_type_name(line) {
            n
        } else {
            i += 1;
            continue;
        };

        // Skip non-entity patterns
        if should_skip_name(name, privileged) {
            i += 1;
            continue;
        }

        // Collect body until closing brace
        let mut body = String::new();
        let mut brace_depth = 0;
        let mut started = false;
        let mut ended = false;

        for j in i..lines.len() {
            let l = lines[j];
            for ch in l.chars() {
                if ch == '{' {
                    brace_depth += 1;
                    started = true;
                }
                if ch == '}' {
                    brace_depth -= 1;
                }
            }
            body.push_str(l);
            body.push('\n');
            if started && brace_depth <= 0 {
                i = j + 1;
                ended = true;
                break;
            }
        }

        // Extract fields from body
        let fields = extract_fields(&body);

        // Filter by field count: skip tiny (<2) or huge (>20) interfaces
        if fields.len() >= 2 && fields.len() <= 20 {
            entities.push(TsEntity {
                name: name.to_string(),
                fields,
            });
        }

        if !ended {
            i += 1;
        }
    }

    entities
}

/// Legacy: extract without path filtering (for backwards compat)
pub fn extract_entities(source: &str) -> Vec<TsEntity> {
    extract_entities_from_file(source, Path::new("unknown.ts"))
}

fn should_skip_name(name: &str, privileged: bool) -> bool {
    // Always skip IFoo hungarian notation
    if name.starts_with('I')
        && name
            .chars()
            .nth(1)
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    {
        return true;
    }

    // In privileged dirs (types/, contracts/), only skip Props/State/Context
    if privileged {
        return name.ends_with("Props")
            || name.ends_with("State")
            || name.ends_with("Context");
    }

    // Outside privileged dirs, skip all framework suffixes
    for suffix in SKIP_SUFFIXES {
        if name.ends_with(suffix) {
            return true;
        }
    }

    false
}

fn extract_type_name(line: &str) -> Option<&str> {
    let line = line.trim_start_matches("export").trim();

    if line.starts_with("interface ") {
        let rest = line.trim_start_matches("interface").trim();
        let name_end = rest
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(rest.len());
        if name_end > 0 {
            return Some(&rest[..name_end]);
        }
    }

    if line.starts_with("type ") && line.contains('=') {
        let rest = line.trim_start_matches("type").trim();
        let name_end = rest
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(rest.len());
        if name_end > 0 {
            return Some(&rest[..name_end]);
        }
    }

    None
}

fn extract_fields(body: &str) -> Vec<TsField> {
    let mut fields = Vec::new();

    for line in body.lines() {
        let line = line.trim().trim_end_matches([',', ';']);
        if line.is_empty()
            || line.starts_with("//")
            || line.starts_with("/*")
            || line == "{"
            || line == "}"
        {
            continue;
        }

        // Pattern: fieldName?: TypeName
        if let Some(colon_pos) = line.find(':') {
            let name_part = line[..colon_pos].trim();
            let type_part = line[colon_pos + 1..].trim();

            // Clean name
            let required = !name_part.ends_with('?');
            let name = name_part.trim_end_matches('?').trim();

            // Skip system fields
            if name == "id"
                || name == "createdAt"
                || name == "updatedAt"
                || name == "created_at"
                || name == "updated_at"
            {
                continue;
            }

            // Skip if name is empty or starts with special chars
            if name.is_empty()
                || !name
                    .chars()
                    .next()
                    .map(|c| c.is_alphabetic())
                    .unwrap_or(false)
            {
                continue;
            }

            let field_type = map_ts_to_cronus(name, type_part);

            fields.push(TsField {
                name: name.to_string(),
                field_type,
                required,
            });
        }
    }

    fields
}

fn map_ts_to_cronus(field_name: &str, ts_type: &str) -> String {
    let ts_lower = ts_type.to_lowercase();
    let name_lower = field_name.to_lowercase();

    // Check by field name first (semantic detection)
    if name_lower.contains("email") {
        return "email".to_string();
    }
    if name_lower.contains("phone") || name_lower.contains("tel") {
        return "phone".to_string();
    }
    if name_lower.contains("url")
        || name_lower.contains("link")
        || name_lower.contains("href")
        || name_lower.contains("website")
    {
        return "url".to_string();
    }
    // FIX: money detection — require BOTH a money-related name AND a numeric TS type
    if (name_lower.contains("price")
        || name_lower.contains("amount")
        || name_lower.contains("cost")
        || name_lower.contains("fee")
        || name_lower.contains("revenue"))
        && (ts_lower.contains("number") || ts_lower.contains("decimal") || ts_lower.contains("float"))
    {
        return "money".to_string();
    }
    if name_lower.contains("slug") {
        return "slug".to_string();
    }
    if name_lower.contains("password") || name_lower.contains("secret") || name_lower.contains("token") {
        return "string".to_string();
    }
    if name_lower.contains("description")
        || name_lower.contains("content")
        || name_lower.contains("body")
        || name_lower.contains("bio")
        || name_lower.contains("notes")
    {
        return "text".to_string();
    }

    // Check by TypeScript type
    if ts_lower.contains("boolean") || ts_lower == "bool" {
        return "boolean".to_string();
    }
    if ts_lower.contains("number") || ts_lower == "int" || ts_lower == "float" || ts_lower == "decimal" {
        return "number".to_string();
    }
    if ts_lower.contains("date") {
        return "date".to_string();
    }
    if ts_lower.contains("record") || ts_lower.contains("object") || ts_lower.contains("json") {
        return "json".to_string();
    }

    // Check for union types (enum detection)
    if ts_type.contains('|') {
        let variants: Vec<&str> = ts_type
            .split('|')
            .map(|v| v.trim().trim_matches(|c: char| c == '\'' || c == '"'))
            .filter(|v| !v.is_empty() && *v != "null" && *v != "undefined")
            .collect();
        if variants.len() >= 2 && variants.iter().all(|v| v.len() < 30 && !v.contains(' ')) {
            return format!("enum [{}]", variants.join(", "));
        }
    }

    // Check for entity reference (PascalCase type that's not a primitive)
    let non_entity_types = [
        "String", "Number", "Boolean", "Date", "Array", "Object", "Record", "Map", "Set",
        "Promise", "Partial", "Required", "Omit", "Pick",
    ];
    if ts_type
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
        && !non_entity_types.contains(&ts_type.split('<').next().unwrap_or(""))
    {
        return format!("-> {}", ts_type.split('<').next().unwrap_or(ts_type));
    }

    "string".to_string()
}

/// Emit .cronus entity blocks from extracted entities
pub fn emit_entities(entities: &[TsEntity]) -> String {
    let mut out = String::new();

    for entity in entities {
        out.push_str(&format!("entity {} {{\n", entity.name));

        // Find max lengths for alignment
        let max_name = entity.fields.iter().map(|f| f.name.len()).max().unwrap_or(10);
        let max_type = entity
            .fields
            .iter()
            .map(|f| f.field_type.len())
            .max()
            .unwrap_or(8);

        for field in &entity.fields {
            out.push_str(&format!(
                "  {:<width_n$}  {:<width_t$}",
                field.name,
                field.field_type,
                width_n = max_name,
                width_t = max_type
            ));

            if field.required {
                out.push_str("  required");
            }
            out.push('\n');
        }

        out.push_str("}\n\n");
    }

    out
}
