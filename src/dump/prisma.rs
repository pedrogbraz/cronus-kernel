#![allow(dead_code, unused_imports, unused_variables)]
//! Prisma schema → .cronus converter
//!
//! Parses a Prisma `.prisma` schema file and emits a valid .cronus file
//! with entities (from models) and enum types.

use std::collections::{HashMap, HashSet};

/// Main entry point: takes raw Prisma schema string, returns .cronus source
pub fn dump_prisma(schema: &str) -> String {
    // First pass: collect enums
    let enums = collect_enums(schema);

    // Second pass: collect models
    let models = collect_models(schema, &enums);

    // Emit .cronus
    let mut out = String::new();
    out.push_str("# Generated from Prisma schema\n\n");
    out.push_str("app \"Prisma Import\" {\n");
    out.push_str("  stack react + tailwind\n");
    out.push_str("  port 5175\n");
    out.push_str("}\n");

    for model in &models {
        out.push('\n');
        emit_entity(&mut out, model);
    }

    out
}

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

struct PrismaEnum {
    name: String,
    variants: Vec<String>,
}

struct PrismaModel {
    name: String,
    fields: Vec<PrismaField>,
}

struct PrismaField {
    name: String,
    cronus_type: String,
    required: bool,
    unique: bool,
    is_relation: bool,
    relation_target: Option<String>,
    enum_variants: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// Enum collection (first pass)
// ---------------------------------------------------------------------------

fn collect_enums(schema: &str) -> HashMap<String, Vec<String>> {
    let mut enums: HashMap<String, Vec<String>> = HashMap::new();
    let mut current_enum: Option<String> = None;
    let mut variants: Vec<String> = Vec::new();

    for line in schema.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("enum ") && trimmed.ends_with('{') {
            let name = trimmed
                .strip_prefix("enum ")
                .unwrap_or("")
                .trim()
                .trim_end_matches('{')
                .trim()
                .to_string();
            current_enum = Some(name);
            variants.clear();
        } else if current_enum.is_some() && trimmed == "}" {
            if let Some(name) = current_enum.take() {
                enums.insert(name, variants.clone());
            }
            variants.clear();
        } else if current_enum.is_some() {
            let variant = trimmed.split_whitespace().next().unwrap_or("").to_string();
            if !variant.is_empty() && !variant.starts_with("//") {
                variants.push(variant);
            }
        }
    }

    enums
}

// ---------------------------------------------------------------------------
// Model collection (second pass)
// ---------------------------------------------------------------------------

fn collect_models(schema: &str, enums: &HashMap<String, Vec<String>>) -> Vec<PrismaModel> {
    let mut models: Vec<PrismaModel> = Vec::new();
    let mut current_model: Option<String> = None;
    let mut fields: Vec<PrismaField> = Vec::new();

    for line in schema.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("model ") && trimmed.ends_with('{') {
            let name = trimmed
                .strip_prefix("model ")
                .unwrap_or("")
                .trim()
                .trim_end_matches('{')
                .trim()
                .to_string();
            current_model = Some(name);
            fields.clear();
        } else if current_model.is_some() && trimmed == "}" {
            if let Some(name) = current_model.take() {
                models.push(PrismaModel {
                    name,
                    fields: fields.clone(),
                });
            }
            fields.clear();
        } else if current_model.is_some() {
            if let Some(field) = parse_field(trimmed, enums) {
                fields.push(field);
            }
        }
    }

    models
}

fn parse_field(line: &str, enums: &HashMap<String, Vec<String>>) -> Option<PrismaField> {
    let trimmed = line.trim();

    // Skip empty lines, comments, model-level attributes
    if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("@@") {
        return None;
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let field_name = parts[0];
    let raw_type = parts[1];

    // Skip id fields (CRONUS auto-generates)
    if trimmed.contains("@id") {
        return None;
    }

    // Skip updatedAt / createdAt auto-timestamps
    if trimmed.contains("@updatedAt") {
        return None;
    }
    if field_name == "createdAt" && raw_type == "DateTime" && trimmed.contains("@default") {
        return None;
    }

    // Skip array relations (Post[])
    if raw_type.ends_with("[]") {
        return None;
    }

    let is_optional = raw_type.ends_with('?');
    let base_type = raw_type.trim_end_matches('?');

    let is_unique = trimmed.contains("@unique");
    let has_relation = trimmed.contains("@relation");

    // Check if it's a relation (type references another model — not a primitive and not an enum)
    let is_known_type = matches!(
        base_type,
        "String"
            | "Int"
            | "Float"
            | "Decimal"
            | "Boolean"
            | "DateTime"
            | "Json"
            | "BigInt"
            | "Bytes"
    );
    let is_enum = enums.contains_key(base_type);

    if has_relation || (!is_known_type && !is_enum) {
        // It's a relation to another model
        if has_relation || !is_enum {
            // Extract target from type name
            let target = base_type.to_string();
            return Some(PrismaField {
                name: field_name.to_string(),
                cronus_type: String::new(),
                required: !is_optional,
                unique: is_unique,
                is_relation: true,
                relation_target: Some(target),
                enum_variants: None,
            });
        }
    }

    // Map type
    let cronus_type = map_prisma_type(field_name, base_type);

    // Enum variants
    let enum_variants = if is_enum {
        enums.get(base_type).cloned()
    } else {
        None
    };

    let final_type = if is_enum {
        "enum".to_string()
    } else {
        cronus_type
    };

    Some(PrismaField {
        name: field_name.to_string(),
        cronus_type: final_type,
        required: !is_optional,
        unique: is_unique,
        is_relation: false,
        relation_target: None,
        enum_variants,
    })
}

fn map_prisma_type(field_name: &str, prisma_type: &str) -> String {
    let name_lower = field_name.to_lowercase();

    // Smart type inference based on field name
    if prisma_type == "String" {
        if name_lower.contains("email") {
            return "email".to_string();
        }
        if name_lower.contains("url") || name_lower.contains("link") {
            return "url".to_string();
        }
        if name_lower.contains("phone") {
            return "phone".to_string();
        }
    }

    if (prisma_type == "Decimal" || prisma_type == "Float")
        && (name_lower.contains("price")
            || name_lower.contains("amount")
            || name_lower.contains("total"))
    {
        return "money".to_string();
    }

    match prisma_type {
        "String" => "string".to_string(),
        "Int" | "Float" | "Decimal" | "BigInt" => "number".to_string(),
        "Boolean" => "boolean".to_string(),
        "DateTime" => "date".to_string(),
        "Json" => "json".to_string(),
        "Bytes" => "string".to_string(),
        _ => "string".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Emission
// ---------------------------------------------------------------------------

fn emit_entity(out: &mut String, model: &PrismaModel) {
    out.push_str(&format!("entity {} {{\n", model.name));

    // Calculate padding for alignment
    let max_name_len = model.fields.iter().map(|f| f.name.len()).max().unwrap_or(0);

    let max_type_len = model
        .fields
        .iter()
        .map(|f| {
            if f.is_relation {
                // "-> Target" length
                2 + f.relation_target.as_ref().map(|t| t.len()).unwrap_or(0) + 1
            } else if f.enum_variants.is_some() {
                // "enum" length — variants go after
                4
            } else {
                f.cronus_type.len()
            }
        })
        .max()
        .unwrap_or(0);

    for field in &model.fields {
        let name_pad = max_name_len - field.name.len();
        let name_padding = " ".repeat(name_pad);

        if field.is_relation {
            let target = field.relation_target.as_deref().unwrap_or("Unknown");
            let type_str = format!("-> {}", target);
            out.push_str(&format!("  {}{} {}\n", field.name, name_padding, type_str));
        } else if let Some(ref variants) = field.enum_variants {
            let variants_str = format!("[{}]", variants.join(", "));
            let mut modifiers = Vec::new();
            if field.required {
                modifiers.push("required");
            }
            if field.unique {
                modifiers.push("unique");
            }
            let mod_str = if modifiers.is_empty() {
                String::new()
            } else {
                format!(" {}", modifiers.join(" "))
            };
            let type_pad = max_type_len - 4; // "enum" is 4 chars
            let type_padding = " ".repeat(type_pad);
            let line = format!(
                "  {}{} enum{}{} {}",
                field.name, name_padding, type_padding, mod_str, variants_str
            );
            out.push_str(line.trim_end());
            out.push('\n');
        } else {
            let type_pad = max_type_len - field.cronus_type.len();
            let type_padding = " ".repeat(type_pad);
            let mut modifiers = Vec::new();
            if field.required {
                modifiers.push("required");
            }
            if field.unique {
                modifiers.push("unique");
            }
            let mod_str = if modifiers.is_empty() {
                String::new()
            } else {
                format!(" {}", modifiers.join(" "))
            };
            let line = format!(
                "  {}{} {}{}{}",
                field.name, name_padding, field.cronus_type, type_padding, mod_str
            );
            out.push_str(line.trim_end());
            out.push('\n');
        }
    }

    out.push_str("}\n");
}

// We need Clone for PrismaField since we store them in a Vec that gets cloned
impl Clone for PrismaField {
    fn clone(&self) -> Self {
        PrismaField {
            name: self.name.clone(),
            cronus_type: self.cronus_type.clone(),
            required: self.required,
            unique: self.unique,
            is_relation: self.is_relation,
            relation_target: self.relation_target.clone(),
            enum_variants: self.enum_variants.clone(),
        }
    }
}
