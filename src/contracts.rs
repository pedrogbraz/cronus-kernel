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

// Unconstructed variants and unread fields below are part of the schema that
// `cronus spec` emits into contracts_generated.rs (see cli/spec.rs).
#[allow(dead_code)]
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

#[allow(dead_code)] // layer/stability/requires_* are codegen schema, see Stability
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
    config_keys: &[
        "title",
        "entity",
        "responsive",
        "live",
        "columns",
        "search",
        "paginate",
    ],
    structural_keys: &[
        req("name"),
        opt("title"),
        opt("column"),
        opt("badge"),
        opt("status"),
        opt("_type"),
    ],
    entity_binding: true,
    on_unknown_key: Fallback::Ignore,
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
    structural_keys: &[
        req("name"),
        opt("title"),
        opt("type"),
        opt("placeholder"),
        opt("required"),
        opt("options"),
        opt("disabled"),
    ],
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
    config_keys: &["icon", "id", "cols", "cta_text", "cta_link", "cta_style"],
    structural_keys: &[
        req("name"),
        opt("subtitle"),
        opt("icon"),
        opt("link"),
        opt("action"),
        opt("style"),
        opt("status"),
        opt("value"),
        opt("action_icon"),
        opt("href"),
        opt("description"),
        opt("on_click"),
    ],
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
    structural_keys: &[
        req("name"),
        opt("title"),
        opt("icon"),
        opt("trend"),
        opt("meta"),
        opt("value"),
        opt("badge"),
        opt("subtitle"),
        opt("description"),
        opt("span"),
    ],
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
    structural_keys: &[
        req("name"),
        opt("type"),
        opt("placeholder"),
        opt("required"),
        opt("options"),
    ],
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
    structural_keys: &[
        req("name"),
        opt("column"),
        opt("color"),
        opt("assignee"),
        opt("priority"),
        opt("label"),
    ],
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

// ── New Contracts (sections with dedicated renderers) ───────────────────────

static PAGE_HEADER_CONTRACT: SectionContract = SectionContract {
    name: "page-header",
    layer: Layer::Stdlib,
    stability: Stability::Stable,
    requires_title: true,
    requires_items: false,
    min_items: 0,
    structural_keys: &[opt("title"), opt("action"), opt("icon")],
    entity_binding: false,
    on_unknown_key: Fallback::Warn,
    on_missing_required: Fallback::Error,
    config_keys: &["action_text", "eyebrow"],
};

static TABS_CONTRACT: SectionContract = SectionContract {
    name: "tabs",
    layer: Layer::Stdlib,
    stability: Stability::Stable,
    requires_title: false,
    requires_items: true,
    min_items: 1,
    structural_keys: &[
        opt("title"),
        opt("name"),
        opt("icon"),
        opt("active"),
        opt("style"),
        opt("description"),
    ],
    entity_binding: false,
    on_unknown_key: Fallback::Warn,
    on_missing_required: Fallback::Error,
    config_keys: &[],
};

static ALERT_CONTRACT: SectionContract = SectionContract {
    name: "alert",
    layer: Layer::Stdlib,
    stability: Stability::Stable,
    requires_title: true,
    requires_items: false,
    min_items: 0,
    structural_keys: &[],
    entity_binding: false,
    on_unknown_key: Fallback::Warn,
    on_missing_required: Fallback::Error,
    config_keys: &["style"],
};

static ACCORDION_CONTRACT: SectionContract = SectionContract {
    name: "accordion",
    layer: Layer::Stdlib,
    stability: Stability::Stable,
    requires_title: false,
    requires_items: true,
    min_items: 1,
    structural_keys: &[opt("title"), opt("name"), opt("description")],
    entity_binding: false,
    on_unknown_key: Fallback::Warn,
    on_missing_required: Fallback::Error,
    config_keys: &[],
};

static BREADCRUMB_CONTRACT: SectionContract = SectionContract {
    name: "breadcrumb",
    layer: Layer::Stdlib,
    stability: Stability::Stable,
    requires_title: false,
    requires_items: true,
    min_items: 1,
    structural_keys: &[opt("title"), opt("name"), opt("link")],
    entity_binding: false,
    on_unknown_key: Fallback::Warn,
    on_missing_required: Fallback::Error,
    config_keys: &[],
};

static CHART_CONTRACT: SectionContract = SectionContract {
    name: "chart",
    layer: Layer::Stdlib,
    stability: Stability::Stable,
    requires_title: false,
    requires_items: true,
    min_items: 1,
    structural_keys: &[opt("title"), opt("name"), opt("value"), opt("color")],
    entity_binding: false,
    on_unknown_key: Fallback::Warn,
    on_missing_required: Fallback::Error,
    config_keys: &["style", "type", "periods", "x_axis", "y_axis"],
};

// ── Shell / Template Contracts ─────────────────────────────────────────────

static HERO_CONTRACT: SectionContract = SectionContract {
    name: "hero",
    layer: Layer::Pattern,
    stability: Stability::Stable,
    requires_title: false,
    requires_items: false,
    min_items: 0,
    config_keys: &[
        "title", "subtitle", "cta_text", "cta_link", "style", "image",
    ],
    structural_keys: &[
        opt("title"),
        opt("name"),
        opt("description"),
        opt("link"),
        opt("icon"),
    ],
    entity_binding: false,
    on_unknown_key: Fallback::Ignore,
    on_missing_required: Fallback::Error,
};

static TOPBAR_CONTRACT: SectionContract = SectionContract {
    name: "topbar",
    layer: Layer::Core,
    stability: Stability::Stable,
    requires_title: false,
    requires_items: false,
    min_items: 0,
    config_keys: &["title", "logo", "style"],
    structural_keys: &[opt("name"), opt("title"), opt("link"), opt("icon")],
    entity_binding: false,
    on_unknown_key: Fallback::Ignore,
    on_missing_required: Fallback::Error,
};

static SIDEBAR_CONTRACT: SectionContract = SectionContract {
    name: "sidebar",
    layer: Layer::Core,
    stability: Stability::Stable,
    requires_title: false,
    requires_items: false,
    min_items: 0,
    config_keys: &["title", "style", "collapsed"],
    structural_keys: &[
        opt("name"),
        opt("title"),
        opt("link"),
        opt("icon"),
        opt("badge"),
    ],
    entity_binding: false,
    on_unknown_key: Fallback::Ignore,
    on_missing_required: Fallback::Error,
};

static FOOTER_CONTRACT: SectionContract = SectionContract {
    name: "footer",
    layer: Layer::Core,
    stability: Stability::Stable,
    requires_title: false,
    requires_items: false,
    min_items: 0,
    config_keys: &["title", "style", "copyright"],
    structural_keys: &[opt("name"), opt("title"), opt("link"), opt("icon")],
    entity_binding: false,
    on_unknown_key: Fallback::Ignore,
    on_missing_required: Fallback::Error,
};

// ── Registry (hardcoded fallback) ──────────────────────────────────────────

static HARDCODED_CONTRACTS: &[&SectionContract] = &[
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
    &PAGE_HEADER_CONTRACT,
    &TABS_CONTRACT,
    &ALERT_CONTRACT,
    &ACCORDION_CONTRACT,
    &BREADCRUMB_CONTRACT,
    &CHART_CONTRACT,
    &HERO_CONTRACT,
    &TOPBAR_CONTRACT,
    &SIDEBAR_CONTRACT,
    &FOOTER_CONTRACT,
];

static HARDCODED_NAMES: &[&str] = &[
    "table",
    "form",
    "card",
    "kpi",
    "modal",
    "kanban",
    "command",
    "dropdown",
    "toast",
    "empty",
    "page-header",
    "tabs",
    "alert",
    "accordion",
    "breadcrumb",
    "chart",
    "sidebar",
    "topbar",
    "footer",
    "hero",
    "features",
    "pricing",
    "promo",
    "info-bar",
    "bento",
    "features-split",
    "team-list",
    "status-card",
    "policies",
    "activity-table",
    "edge",
    "links",
    "quick-links",
    "skeleton",
    "loading",
    "error",
    "not-found",
    "404",
    "timeline",
    "progress",
    "pagination",
    "filters",
    "notifications",
    "dark-mode",
    "layout",
    "sheet",
    "product-grid",
];

// ── Generated Contracts (from spec.toml codegen) ──────────────────────────
// If contracts_generated.rs exists, include it. Generated contracts take
// precedence over hardcoded ones when names overlap.

#[cfg(feature = "generated-contracts")]
mod generated {
    include!("contracts_generated.rs");
}

pub struct ContractRegistry;

impl ContractRegistry {
    /// Look up a contract by name. Generated contracts take precedence over hardcoded.
    pub fn get(name: &str) -> Option<&'static SectionContract> {
        let canonical = Self::resolve_alias(name).unwrap_or(name);

        // Try generated first (if available)
        #[cfg(feature = "generated-contracts")]
        {
            if let Some(c) = generated::GENERATED_CONTRACTS
                .iter()
                .find(|c| c.name == canonical)
            {
                return Some(c);
            }
        }

        // Fallback to hardcoded
        HARDCODED_CONTRACTS
            .iter()
            .find(|c| c.name == canonical)
            .copied()
    }

    /// Resolve alias to canonical name. Generated aliases take precedence.
    pub fn resolve_alias(name: &str) -> Option<&'static str> {
        #[cfg(feature = "generated-contracts")]
        {
            if let Some(canonical) = generated::generated_resolve_alias(name) {
                return Some(canonical);
            }
        }

        // Hardcoded alias fallback
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

    pub fn all_names() -> &'static [&'static str] {
        #[cfg(feature = "generated-contracts")]
        {
            return generated::GENERATED_NAMES;
        }
        #[cfg(not(feature = "generated-contracts"))]
        {
            HARDCODED_NAMES
        }
    }
}

// ── Warnings ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ParseWarning {
    UnknownSection {
        name: String,
        line: usize,
    },
    UnknownKey {
        section: String,
        key: String,
        item: String,
        line: usize,
    },
    MissingRequired {
        section: String,
        key: String,
        item: String,
        line: usize,
    },
    AliasUsed {
        alias: String,
        canonical: String,
        line: usize,
    },
    MinItemsViolation {
        section: String,
        expected: usize,
        actual: usize,
        line: usize,
    },
    UnknownConfig {
        section: String,
        key: String,
        line: usize,
    },
}

// ── Validation ──────────────────────────────────────────────────────────────

pub fn validate_section(section: &SectionNode, entity_fields: &[String]) -> Vec<ParseWarning> {
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
            // Unknown section — not in contracts and not an alias to one.
            // cronus-ui families are valid section types (runtime dispatch in
            // `ui::render_section_inner`); they have no SectionContract.
            if ContractRegistry::resolve_alias(name).is_none()
                && !crate::cronus_ui_widgets::FAMILIES.contains(&name)
            {
                warnings.push(ParseWarning::UnknownSection {
                    name: name.to_string(),
                    line: 0,
                });
            }
            return warnings;
        }
    };

    // TWO-NAMESPACE KEY VALIDATION:
    // Keys in entity-bound sections (table, form) come from two namespaces:
    //   1. Structural keys — defined in the contract (e.g. "column", "badge")
    //   2. Entity fields — from the bound entity (e.g. "customer", "amount")
    // If entity_binding is true but no entity fields were provided,
    // we can't validate entity-bound keys — skip unknown key warnings entirely.
    // TODO: Thread entity fields from page context through render_section.
    let has_entity_ref = section.config.contains_key("entity") || section.binding.is_some();
    let skip_entity_key_check =
        contract.entity_binding && entity_fields.is_empty() && has_entity_ref;

    // 3. Validate items against structural keys + entity fields
    for item in &section.items {
        let item_name = item.get("name").cloned().unwrap_or_default();

        // Check for unknown keys
        for key in item.keys() {
            let is_structural = contract
                .structural_keys
                .iter()
                .any(|k| k.name == key.as_str());
            let is_entity_field = contract.entity_binding && entity_fields.contains(key);
            let is_meta = key == "_type" || key == "title" || key == "name" || key == "description";

            if !is_structural && !is_entity_field && !is_meta {
                if skip_entity_key_check {
                    // Can't validate — entity fields not available yet. Skip.
                    continue;
                }
                match contract.on_unknown_key {
                    Fallback::Warn | Fallback::Error => {
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
            let has_key = item.contains_key(key_def.name)
                || (key_def.name == "name" && item.contains_key("title"));
            if key_def.required && !has_key {
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

    // 4. Check min_items (skip if section has a binding — data comes from DB)
    if contract.min_items > 0
        && section.items.len() < contract.min_items
        && section.binding.is_none()
    {
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
