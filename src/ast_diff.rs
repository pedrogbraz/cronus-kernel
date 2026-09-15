//! AST-level semantic diff for .cronus files.
//!
//! Compares two parsed ASTs and produces a list of meaningful, semantic changes
//! (not text-level diffs). Used by `cronus changelog` to show what changed.

use crate::parser::{AstNode, FieldType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ══════════════════════════════════════════════════
// CHANGE TYPES
// ══════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstChange {
    EntityAdded {
        name: String,
    },
    EntityRemoved {
        name: String,
    },
    EntitySharedChanged {
        name: String,
        now_shared: bool,
    },
    FieldAdded {
        entity: String,
        field: String,
        field_type: String,
    },
    FieldRemoved {
        entity: String,
        field: String,
    },
    FieldTypeChanged {
        entity: String,
        field: String,
        old_type: String,
        new_type: String,
    },
    PageAdded {
        route: String,
    },
    PageRemoved {
        route: String,
    },
    ApiAdded {
        prefix: String,
    },
    ApiRemoved {
        prefix: String,
    },
    ApiRouteAdded {
        prefix: String,
        method: String,
        path: String,
    },
    ApiRouteRemoved {
        prefix: String,
        method: String,
        path: String,
    },
    WebhookAdded {
        entity: String,
    },
    WebhookRemoved {
        entity: String,
    },
    StyleChanged {
        key: String,
        old_value: String,
        new_value: String,
    },
}

impl AstChange {
    /// Human-readable description of this change.
    pub fn describe(&self) -> String {
        match self {
            AstChange::EntityAdded { name } => format!("  + Entity '{}' added", name),
            AstChange::EntityRemoved { name } => format!("  - Entity '{}' removed", name),
            AstChange::EntitySharedChanged { name, now_shared } => {
                format!("  ~ Entity '{}' shared = {}", name, now_shared)
            }
            AstChange::FieldAdded {
                entity,
                field,
                field_type,
            } => format!("  + {}.{} ({}) added", entity, field, field_type),
            AstChange::FieldRemoved { entity, field } => {
                format!("  - {}.{} removed", entity, field)
            }
            AstChange::FieldTypeChanged {
                entity,
                field,
                old_type,
                new_type,
            } => format!(
                "  ~ {}.{} type: {} -> {}",
                entity, field, old_type, new_type
            ),
            AstChange::PageAdded { route } => format!("  + Page '{}' added", route),
            AstChange::PageRemoved { route } => format!("  - Page '{}' removed", route),
            AstChange::ApiAdded { prefix } => format!("  + API '{}' added", prefix),
            AstChange::ApiRemoved { prefix } => format!("  - API '{}' removed", prefix),
            AstChange::ApiRouteAdded {
                prefix,
                method,
                path,
            } => format!("  + API route {} {} in '{}'", method, path, prefix),
            AstChange::ApiRouteRemoved {
                prefix,
                method,
                path,
            } => format!("  - API route {} {} in '{}'", method, path, prefix),
            AstChange::WebhookAdded { entity } => format!("  + Webhook for '{}' added", entity),
            AstChange::WebhookRemoved { entity } => format!("  - Webhook for '{}' removed", entity),
            AstChange::StyleChanged {
                key,
                old_value,
                new_value,
            } => format!("  ~ Style '{}': '{}' -> '{}'", key, old_value, new_value),
        }
    }
}

// ══════════════════════════════════════════════════
// SNAPSHOT — serializable extract of an AST
// ══════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstSnapshot {
    pub entities: Vec<SnapshotEntity>,
    pub pages: Vec<String>, // routes
    pub apis: Vec<SnapshotApi>,
    pub webhooks: Vec<String>, // entity names
    pub style: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotEntity {
    pub name: String,
    pub shared: bool,
    pub fields: Vec<SnapshotField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotField {
    pub name: String,
    pub field_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotApi {
    pub prefix: String,
    pub routes: Vec<SnapshotRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotRoute {
    pub method: String,
    pub path: String,
}

// ══════════════════════════════════════════════════
// EXTRACT SNAPSHOT FROM AST
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

fn http_method_str(m: &crate::parser::HttpMethod) -> &'static str {
    match m {
        crate::parser::HttpMethod::GET => "GET",
        crate::parser::HttpMethod::POST => "POST",
        crate::parser::HttpMethod::PATCH => "PATCH",
        crate::parser::HttpMethod::PUT => "PUT",
        crate::parser::HttpMethod::DELETE => "DELETE",
    }
}

pub fn snapshot_from_ast(nodes: &[AstNode]) -> AstSnapshot {
    let mut entities = Vec::new();
    let mut pages = Vec::new();
    let mut apis = Vec::new();
    let mut webhooks = Vec::new();
    let mut style = HashMap::new();

    for node in nodes {
        match node {
            AstNode::Entity(e) => {
                entities.push(SnapshotEntity {
                    name: e.name.clone(),
                    shared: e.shared,
                    fields: e
                        .fields
                        .iter()
                        .map(|f| SnapshotField {
                            name: f.name.clone(),
                            field_type: field_type_str(&f.field_type).to_string(),
                        })
                        .collect(),
                });
            }
            AstNode::Page(p) => {
                pages.push(p.route.clone());
            }
            AstNode::Api(a) => {
                apis.push(SnapshotApi {
                    prefix: a.prefix.clone(),
                    routes: a
                        .routes
                        .iter()
                        .map(|r| SnapshotRoute {
                            method: http_method_str(&r.method).to_string(),
                            path: r.path.clone(),
                        })
                        .collect(),
                });
            }
            AstNode::Webhook(w) => {
                webhooks.push(w.entity.clone());
            }
            AstNode::Style(s) => {
                if let Some(ref v) = s.theme {
                    style.insert("theme".to_string(), v.clone());
                }
                if let Some(ref v) = s.accent {
                    style.insert("accent".to_string(), v.clone());
                }
                if let Some(ref v) = s.radius {
                    style.insert("radius".to_string(), v.clone());
                }
                if let Some(ref v) = s.font {
                    style.insert("font".to_string(), v.clone());
                }
                for (k, v) in &s.config {
                    style.insert(k.clone(), v.clone());
                }
            }
            _ => {}
        }
    }

    AstSnapshot {
        entities,
        pages,
        apis,
        webhooks,
        style,
    }
}

// ══════════════════════════════════════════════════
// DIFF TWO ASTS
// ══════════════════════════════════════════════════

pub fn diff_snapshots(old: &AstSnapshot, new: &AstSnapshot) -> Vec<AstChange> {
    let mut changes = Vec::new();

    // 1. Entities
    diff_entities(old, new, &mut changes);

    // 2. Pages
    diff_pages(old, new, &mut changes);

    // 3. APIs
    diff_apis(old, new, &mut changes);

    // 4. Webhooks
    diff_webhooks(old, new, &mut changes);

    // 5. Style
    diff_style(old, new, &mut changes);

    changes
}

fn diff_entities(old: &AstSnapshot, new: &AstSnapshot, changes: &mut Vec<AstChange>) {
    let old_map: HashMap<&str, &SnapshotEntity> =
        old.entities.iter().map(|e| (e.name.as_str(), e)).collect();
    let new_map: HashMap<&str, &SnapshotEntity> =
        new.entities.iter().map(|e| (e.name.as_str(), e)).collect();

    // Removed entities
    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            changes.push(AstChange::EntityRemoved {
                name: name.to_string(),
            });
        }
    }

    // Added entities + changed entities
    for (name, new_ent) in &new_map {
        match old_map.get(name) {
            None => {
                changes.push(AstChange::EntityAdded {
                    name: name.to_string(),
                });
            }
            Some(old_ent) => {
                // shared changed?
                if old_ent.shared != new_ent.shared {
                    changes.push(AstChange::EntitySharedChanged {
                        name: name.to_string(),
                        now_shared: new_ent.shared,
                    });
                }

                // field diffs
                let old_fields: HashMap<&str, &SnapshotField> = old_ent
                    .fields
                    .iter()
                    .map(|f| (f.name.as_str(), f))
                    .collect();
                let new_fields: HashMap<&str, &SnapshotField> = new_ent
                    .fields
                    .iter()
                    .map(|f| (f.name.as_str(), f))
                    .collect();

                for fname in old_fields.keys() {
                    if !new_fields.contains_key(fname) {
                        changes.push(AstChange::FieldRemoved {
                            entity: name.to_string(),
                            field: fname.to_string(),
                        });
                    }
                }

                for (fname, new_f) in &new_fields {
                    match old_fields.get(fname) {
                        None => {
                            changes.push(AstChange::FieldAdded {
                                entity: name.to_string(),
                                field: fname.to_string(),
                                field_type: new_f.field_type.clone(),
                            });
                        }
                        Some(old_f) => {
                            if old_f.field_type != new_f.field_type {
                                changes.push(AstChange::FieldTypeChanged {
                                    entity: name.to_string(),
                                    field: fname.to_string(),
                                    old_type: old_f.field_type.clone(),
                                    new_type: new_f.field_type.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

fn diff_pages(old: &AstSnapshot, new: &AstSnapshot, changes: &mut Vec<AstChange>) {
    for route in &old.pages {
        if !new.pages.contains(route) {
            changes.push(AstChange::PageRemoved {
                route: route.clone(),
            });
        }
    }
    for route in &new.pages {
        if !old.pages.contains(route) {
            changes.push(AstChange::PageAdded {
                route: route.clone(),
            });
        }
    }
}

fn diff_apis(old: &AstSnapshot, new: &AstSnapshot, changes: &mut Vec<AstChange>) {
    let old_map: HashMap<&str, &SnapshotApi> =
        old.apis.iter().map(|a| (a.prefix.as_str(), a)).collect();
    let new_map: HashMap<&str, &SnapshotApi> =
        new.apis.iter().map(|a| (a.prefix.as_str(), a)).collect();

    for prefix in old_map.keys() {
        if !new_map.contains_key(prefix) {
            changes.push(AstChange::ApiRemoved {
                prefix: prefix.to_string(),
            });
        }
    }

    for (prefix, new_api) in &new_map {
        match old_map.get(prefix) {
            None => {
                changes.push(AstChange::ApiAdded {
                    prefix: prefix.to_string(),
                });
            }
            Some(old_api) => {
                // Compare routes within this API
                let old_routes: Vec<(&str, &str)> = old_api
                    .routes
                    .iter()
                    .map(|r| (r.method.as_str(), r.path.as_str()))
                    .collect();
                let new_routes: Vec<(&str, &str)> = new_api
                    .routes
                    .iter()
                    .map(|r| (r.method.as_str(), r.path.as_str()))
                    .collect();

                for &(method, path) in &old_routes {
                    if !new_routes.contains(&(method, path)) {
                        changes.push(AstChange::ApiRouteRemoved {
                            prefix: prefix.to_string(),
                            method: method.to_string(),
                            path: path.to_string(),
                        });
                    }
                }
                for &(method, path) in &new_routes {
                    if !old_routes.contains(&(method, path)) {
                        changes.push(AstChange::ApiRouteAdded {
                            prefix: prefix.to_string(),
                            method: method.to_string(),
                            path: path.to_string(),
                        });
                    }
                }
            }
        }
    }
}

fn diff_webhooks(old: &AstSnapshot, new: &AstSnapshot, changes: &mut Vec<AstChange>) {
    for entity in &old.webhooks {
        if !new.webhooks.contains(entity) {
            changes.push(AstChange::WebhookRemoved {
                entity: entity.clone(),
            });
        }
    }
    for entity in &new.webhooks {
        if !old.webhooks.contains(entity) {
            changes.push(AstChange::WebhookAdded {
                entity: entity.clone(),
            });
        }
    }
}

fn diff_style(old: &AstSnapshot, new: &AstSnapshot, changes: &mut Vec<AstChange>) {
    // Check for changed or removed keys
    for (key, old_val) in &old.style {
        match new.style.get(key) {
            None => {
                changes.push(AstChange::StyleChanged {
                    key: key.clone(),
                    old_value: old_val.clone(),
                    new_value: "(removed)".to_string(),
                });
            }
            Some(new_val) => {
                if old_val != new_val {
                    changes.push(AstChange::StyleChanged {
                        key: key.clone(),
                        old_value: old_val.clone(),
                        new_value: new_val.clone(),
                    });
                }
            }
        }
    }
    // Check for newly added keys
    for (key, new_val) in &new.style {
        if !old.style.contains_key(key) {
            changes.push(AstChange::StyleChanged {
                key: key.clone(),
                old_value: "(none)".to_string(),
                new_value: new_val.clone(),
            });
        }
    }
}
