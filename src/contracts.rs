#![allow(dead_code)]

use crate::parser::SectionNode;

// ── Core Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fallback {
    Warn,
    Error,
    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Layer {
    Core,
    Stdlib,
    Pattern,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stability {
    Stable,
    Experimental,
    Internal,
    Deprecated,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyDef {
    pub name: &'static str,
    pub required: bool,
}

pub struct SectionContract {
    pub name: &'static str,
    pub layer: Layer,
    pub stability: Stability,
    pub requires_title: bool,
    pub requires_items: bool,
    pub min_items: usize,
    pub config_keys: &'static [&'static str],
    pub structural_keys: &'static [KeyDef],
    pub entity_binding: bool,
    pub on_unknown_key: Fallback,
    pub on_missing_required: Fallback,
}

// ── Helpers ─────────────────────────────────────────────────────────────────

const fn key(name: &'static str, required: bool) -> KeyDef {
    KeyDef { name, required }
}

const fn req(name: &'static str) -> KeyDef {
    key(name, true)
}

const fn opt(name: &'static str) -> KeyDef {
    key(name, false)
}

// ── Contracts ───────────────────────────────────────────────────────────────

static TABLE_CONTRACT: SectionContract = SectionContract {
    name: "table",
    layer: Layer::Stdlib,
    stability: Stability::Stable,
    requires_title: false,
    requires_items: true,
    min_items: 1,
    config_keys: &["title", "entity", "responsive", "live"],
    structural_keys: &[req("name"), opt("column"), opt("badge"), opt("status"), opt("customer"), opt("amount"), opt("date")],
    entity_binding: true,
    on_unknown_key: Fallback::Warn,
    on_missing_required: Fallback::Error,
};

static FORM_CONTRACT: SectionContract = SectionContract {
    name: "form",
    layer: Layer::Stdlib,
    stability: Stability::Stable,
    requires_title: false,
    requires_items: true,
    min_items: 1,
    config_keys: &["title", "entity", "action", "method"],
    structural_keys: &[req("name"), opt("type"), opt("placeholder"), opt("required"), opt("options"), opt("disabled")],
    entity_binding: true,
    on_unknown_key: Fallback::Warn,
    on_missing_required: Fallback::Error,
};

static CARD_CONTRACT: SectionContract = SectionContract {
    name: "card",
    layer: Layer::Stdlib,
    stability: Stability::Stable,
    requires_title: false,
    requires_items: true,
    min_items: 0,
    config_keys: &["icon", "id"],
    structural_keys: &[req("name"), opt("subtitle"), opt("icon"), opt("link")],
    entity_binding: false,
    on_unknown_key: Fallback::Warn,
    on_missing_required: Fallback::Error,
};

static KPI_CONTRACT: SectionContract = SectionContract {
    name: "kpi",
    layer: Layer::Stdlib,
    stability: Stability::Stable,
    requires_title: false,
    requires_items: true,
    min_items: 1,
    config_keys: &["cols", "cols-md", "cols-sm", "entity", "live", "interval"],
    structural_keys: &[req("name"), opt("icon"), opt("trend"), opt("meta"), opt("value")],
    entity_binding: false,
    on_unknown_key: Fallback::Warn,
    on_missing_required: Fallback::Error,
};

static MODAL_CONTRACT: SectionContract = SectionContract {
    name: "modal",
    layer: Layer::Stdlib,
    stability: Stability::Stable,
    requires_title: true,
    requires_items: true,
    min_items: 0,
    config_keys: &["id"],
    structural_keys: &[req("name"), opt("type"), opt("placeholder"), opt("required"), opt("options")],
    entity_binding: false,
    on_unknown_key: Fallback::Warn,
    on_missing_required: Fallback::Error,
};

static KANBAN_CONTRACT: SectionContract = SectionContract {
    name: "kanban",
    layer: Layer::Stdlib,
    stability: Stability::Stable,
    requires_title: false,
    requires_items: true,
    min_items: 1,
    config_keys: &["title"],
    structural_keys: &[req("name"), opt("column"), opt("color"), opt("assignee"), opt("priority"), opt("label")],
    entity_binding: false,
    on_unknown_key: Fallback::Warn,
    on_missing_required: Fallback::Error,
};

static COMMAND_CONTRACT: SectionContract = SectionContract {
    name: "command",
    layer: Layer::Stdlib,
    stability: Stability::Stable,
    requires_title: false,
    requires_items: true,
    min_items: 1,
    config_keys: &["id"],
    structural_keys: &[req("name"), opt("shortcut"), opt("icon"), opt("link")],
    entity_binding: false,
    on_unknown_key: Fallback::Warn,
    on_missing_required: Fallback::Error,
};

static DROPDOWN_CONTRACT: SectionContract = SectionContract {
    name: "dropdown",
    layer: Layer::Stdlib,
    stability: Stability::Stable,
    requires_title: false,
    requires_items: true,
    min_items: 1,
    config_keys: &["trigger", "id"],
    structural_keys: &[req("name"), opt("icon"), opt("danger"), opt("link")],
    entity_binding: false,
    on_unknown_key: Fallback::Warn,
    on_missing_required: Fallback::Error,
};

static TOAST_CONTRACT: SectionContract = SectionContract {
    name: "toast",
    layer: Layer::Stdlib,
    stability: Stability::Stable,
    requires_title: true,
    requires_items: false,
    min_items: 0,
    config_keys: &["type", "duration", "id"],
    structural_keys: &[],
    entity_binding: false,
    on_unknown_key: Fallback::Ignore,
    on_missing_required: Fallback::Error,
};

static EMPTY_CONTRACT: SectionContract = SectionContract {
    name: "empty",
    layer: Layer::Stdlib,
    stability: Stability::Stable,
    requires_title: true,
    requires_items: true,
    min_items: 0,
    config_keys: &["icon"],
    structural_keys: &[req("name"), opt("icon"), opt("link")],
    entity_binding: false,
    on_unknown_key: Fallback::Warn,
    on_missing_required: Fallback::Error,
};

// ── Registry ────────────────────────────────────────────────────────────────

static ALL_CONTRACTS: &[&SectionContract] = &[
    &TABLE_CONTRACT,
    &FORM_CONTRACT,
    &CARD_CONTRACT,
    &KPI_CONTRACT,
    &MODAL_CONTRACT,
    &KANBAN_CONTRACT,
    &COMMAND_CONTRACT,
    &DROPDOWN_CONTRACT,
    &TOAST_CONTRACT,
    &EMPTY_CONTRACT,
];

static ALL_NAMES: &[&str] = &[
    "table", "form", "card", "kpi", "modal",
    "kanban", "command", "dropdown", "toast", "empty",
];

pub struct ContractRegistry;

impl ContractRegistry {
    pub fn get(name: &str) -> Option<&'static SectionContract> {
        let canonical = Self::resolve_alias(name).unwrap_or(name);
        ALL_CONTRACTS.iter().find(|c| c.name == canonical).copied()
    }

    pub fn resolve_alias(name: &str) -> Option<&'static str> {
        match name {
            "stats" => Some("kpi"),
            "stat-cards" => Some("kpi"),
            "status-card" => Some("card"),
            "activity-table" => Some("table"),
            "team-list" => Some("table"),
            "policies" => Some("form"),
            "live-keys" => Some("card"),
            "test-keys" => Some("card"),
            "webhooks" => Some("card"),
            "quick-links" => Some("links"),
            "promo" => Some("card"),
            "info-bar" => Some("alert"),
            "edge" => Some("features"),
            "bento" => Some("features"),
            "features-split" => Some("features"),
            "product-grid" => Some("card"),
            _ => None,
        }
    }

    pub fn is_known(name: &str) -> bool {
        Self::get(name).is_some() || Self::resolve_alias(name).is_some()
    }

    pub fn all_names() -> &'static [&'static str] {
        ALL_NAMES
    }
}

// ── Warnings ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ParseWarning {
    UnknownSection { name: String, line: usize },
    UnknownKey { section: String, key: String, item: String, line: usize },
    MissingRequired { section: String, key: String, item: String, line: usize },
    AliasUsed { alias: String, canonical: String, line: usize },
    MinItemsViolation { section: String, expected: usize, actual: usize, line: usize },
    UnknownConfig { section: String, key: String, line: usize },
}

// ── Validation ──────────────────────────────────────────────────────────────

pub fn validate_section(section: &SectionNode, _entities: &[String]) -> Vec<ParseWarning> {
    let mut warnings: Vec<ParseWarning> = Vec::new();
    let name = section.section_type.as_str();

    // 1. Check alias
    if let Some(canonical) = ContractRegistry::resolve_alias(name) {
        warnings.push(ParseWarning::AliasUsed {
            alias: name.to_string(),
            canonical: canonical.to_string(),
            line: 0,
        });
    }

    // 2. Try to get contract (resolves alias internally)
    let contract = match ContractRegistry::get(name) {
        Some(c) => c,
        None => {
            // Unknown section — not in contracts and not an alias to one
            // Only warn if it's also not a known alias that points to a section without a contract
            if ContractRegistry::resolve_alias(name).is_none() {
                warnings.push(ParseWarning::UnknownSection {
                    name: name.to_string(),
                    line: 0,
                });
            }
            return warnings;
        }
    };

    // 3. Validate items against structural keys
    for item in &section.items {
        let item_name = item.get("name").cloned().unwrap_or_default();

        // Check for unknown keys
        for key in item.keys() {
            let is_structural = contract.structural_keys.iter().any(|k| k.name == key.as_str());
            if !is_structural {
                match contract.on_unknown_key {
                    Fallback::Warn => {
                        warnings.push(ParseWarning::UnknownKey {
                            section: contract.name.to_string(),
                            key: key.clone(),
                            item: item_name.clone(),
                            line: 0,
                        });
                    }
                    Fallback::Error => {
                        warnings.push(ParseWarning::UnknownKey {
                            section: contract.name.to_string(),
                            key: key.clone(),
                            item: item_name.clone(),
                            line: 0,
                        });
                    }
                    Fallback::Ignore => {}
                }
            }
        }

        // Check required keys
        for key_def in contract.structural_keys {
            if key_def.required && !item.contains_key(key_def.name) {
                match contract.on_missing_required {
                    Fallback::Error | Fallback::Warn => {
                        warnings.push(ParseWarning::MissingRequired {
                            section: contract.name.to_string(),
                            key: key_def.name.to_string(),
                            item: item_name.clone(),
                            line: 0,
                        });
                    }
                    Fallback::Ignore => {}
                }
            }
        }
    }

    // 4. Check min_items
    if contract.min_items > 0 && section.items.len() < contract.min_items {
        warnings.push(ParseWarning::MinItemsViolation {
            section: contract.name.to_string(),
            expected: contract.min_items,
            actual: section.items.len(),
            line: 0,
        });
    }

    // 5. Check config keys
    for key in section.config.keys() {
        if !contract.config_keys.contains(&key.as_str()) {
            // Skip common keys that all sections can have
            if key == "title" || key == "subtitle" || key == "style" || key == "entity" {
                continue;
            }
            warnings.push(ParseWarning::UnknownConfig {
                section: contract.name.to_string(),
                key: key.clone(),
                line: 0,
            });
        }
    }

    warnings
}
