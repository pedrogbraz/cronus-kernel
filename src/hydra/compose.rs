//! Block Composer — translates Hydra blocks into .cronus applications.
//!
//! Four modes:
//! - Translate: Hydra block → .cronus/.scriptcronus text
//! - Adapt: remap entity names in a block
//! - Compose: merge N blocks into a full app
//! - Distill: extract blocks from a running app (future)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Block Template (from Hydra skill-blocks.json) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillBlock {
    pub id: String,
    pub name: String,
    pub category: String,
    pub template: Option<String>,
    pub variables: Vec<String>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub composable_with: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillBlockFile {
    pub blocks: Vec<SkillBlock>,
}

// ── Entity Remap ──

#[derive(Debug, Clone)]
pub struct Transition {
    pub field: String,
    pub rules: Vec<(String, String)>,   // ("active", "churned")
}

#[derive(Debug, Clone)]
pub struct EntityEffect {
    pub event: String,          // "create", "update status"
    pub actions: Vec<String>,   // ["notify \"log\" \"message\""]
    pub when_clauses: Vec<(String, Vec<String>)>, // [("canceled", ["notify ..."])]
}

#[derive(Debug, Clone)]
pub struct EntityRemap {
    pub entity: String,         // "Customer"
    pub entity_lower: String,   // "customer"
    pub table: String,          // "Customer" (CRONUS uses entity name as table)
    pub shared: bool,
    pub fields: Vec<FieldDef>,
    pub transitions: Vec<Transition>,
    pub effects: Vec<EntityEffect>,
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub field_type: String,     // "text", "number", "money", "email", "boolean", "date", "slug", "enum"
    pub required: bool,
    pub unique: bool,
    pub searchable: bool,
    pub sensitive: bool,
    pub default: Option<String>,
    pub enum_values: Option<Vec<String>>,
    pub reference: Option<String>,    // "-> EntityName"
}

impl FieldDef {
    pub fn new(name: &str, field_type: &str) -> Self {
        Self {
            name: name.to_string(),
            field_type: field_type.to_string(),
            required: false,
            unique: false,
            searchable: false,
            sensitive: false,
            default: None,
            enum_values: None,
            reference: None,
        }
    }

    pub fn req(mut self) -> Self { self.required = true; self }
    pub fn uniq(mut self) -> Self { self.unique = true; self }
    pub fn search(mut self) -> Self { self.searchable = true; self }
    pub fn sens(mut self) -> Self { self.sensitive = true; self }
    pub fn default_val(mut self, v: &str) -> Self { self.default = Some(v.to_string()); self }
    pub fn enums(mut self, vals: &[&str]) -> Self {
        self.enum_values = Some(vals.iter().map(|s| s.to_string()).collect());
        self
    }
    pub fn refs(mut self, entity: &str) -> Self {
        self.reference = Some(entity.to_string());
        self.field_type = "relation".to_string();
        self
    }
}

impl EntityRemap {
    pub fn new(entity: &str, fields: Vec<FieldDef>) -> Self {
        Self {
            entity: entity.to_string(),
            entity_lower: entity.to_lowercase(),
            table: entity.to_string(),
            shared: false,
            fields,
            transitions: Vec::new(),
            effects: Vec::new(),
        }
    }

    pub fn shared(mut self) -> Self { self.shared = true; self }

    pub fn transition(mut self, field: &str, rules: &[(&str, &str)]) -> Self {
        self.transitions.push(Transition {
            field: field.to_string(),
            rules: rules.iter().map(|(f, t)| (f.to_string(), t.to_string())).collect(),
        });
        self
    }

    pub fn on_create(mut self, actions: &[&str]) -> Self {
        self.effects.push(EntityEffect {
            event: "create".to_string(),
            actions: actions.iter().map(|s| s.to_string()).collect(),
            when_clauses: Vec::new(),
        });
        self
    }

    pub fn on_update(mut self, field: &str, whens: &[(&str, &[&str])]) -> Self {
        self.effects.push(EntityEffect {
            event: format!("update {}", field),
            actions: Vec::new(),
            when_clauses: whens.iter().map(|(v, a)| (v.to_string(), a.iter().map(|s| s.to_string()).collect())).collect(),
        });
        self
    }
}

// ── Translate: skill category → .cronus text ──

/// Generate a complete .cronus app from an entity definition.
/// This is the core of the Block Composer — takes an entity config and
/// produces a full-stack app (entity + API + pages + auth).
pub fn compose_app(
    app_name: &str,
    entities: &[EntityRemap],
    options: &ComposeOptions,
) -> String {
    let mut output = String::new();

    // App block with constitution
    output.push_str(&format!("app \"{}\" {{\n", app_name));
    output.push_str(&format!("  port {}\n", options.port));
    output.push_str("  database sqlite \"./data.db\"\n");
    output.push_str("  theme dark\n");

    if !options.constitution.is_empty() {
        output.push_str("\n  constitution {\n");
        for rule in &options.constitution {
            if rule.starts_with("never") {
                output.push_str(&format!("    {}\n", rule));
            } else {
                output.push_str(&format!("    must \"{}\"\n", rule));
            }
        }
        output.push_str("  }\n");
    }
    output.push_str("}\n\n");

    // Auth
    if options.auth {
        let auth = options.auth_config.as_ref();
        let login = auth.map(|a| a.login_fields.as_str()).unwrap_or("email + password");
        let session = auth.map(|a| a.session_type.as_str()).unwrap_or("jwt");
        let expires = auth.map(|a| a.expires.as_str()).unwrap_or("24h");
        let roles = auth.map(|a| a.roles.as_slice()).unwrap_or(&[]);

        output.push_str("auth {\n");
        output.push_str("  entity User\n");
        output.push_str(&format!("  login {}\n", login));
        output.push_str(&format!("  session {} expires:{}\n", session, expires));
        if !roles.is_empty() {
            output.push_str(&format!("  roles [{}]\n", roles.join(", ")));
        }
        output.push_str("}\n\n");

        // User entity
        output.push_str("entity User {\n");
        output.push_str("  name string!\n");
        output.push_str("  email email! unique\n");
        output.push_str("  password_hash string sensitive\n");
        if !roles.is_empty() {
            let default_role = roles.last().map(|s| s.as_str()).unwrap_or("user");
            output.push_str(&format!("  role enum [{}]! default:\"{}\"\n",
                roles.iter().map(|r| format!("\"{}\"", r)).collect::<Vec<_>>().join(", "),
                default_role
            ));
        }
        output.push_str("}\n\n");
    }

    // Style
    if let Some(ref style) = options.style {
        output.push_str("style {\n");
        output.push_str(&format!("  theme {}\n", style.theme));
        output.push_str(&format!("  accent {}\n", style.accent));
        output.push_str(&format!("  font \"{}\"\n", style.font));
        output.push_str("}\n\n");
    }

    // Entities
    for e in entities {
        output.push_str(&generate_entity(e));
        output.push('\n');
    }

    // API
    output.push_str(&generate_api(entities, options));

    // Admin pages
    if options.admin_pages {
        output.push_str(&generate_admin_pages(entities, app_name));
    }

    // Script integrations
    if options.scripts {
        output.push_str(&generate_scripts(entities, app_name));
    }

    output
}

fn generate_entity(e: &EntityRemap) -> String {
    let shared_str = if e.shared { " shared" } else { "" };
    let mut out = format!("entity {}{} {{\n", e.entity, shared_str);

    for f in &e.fields {
        // Relation fields use -> syntax
        if let Some(ref target) = f.reference {
            out.push_str(&format!("  {} -> {}\n", f.name, target));
            continue;
        }

        // Field type with ! for required
        let req_mark = if f.required { "!" } else { "" };
        out.push_str(&format!("  {} {}{}", f.name, f.field_type, req_mark));

        // Modifiers
        if f.unique { out.push_str(" unique"); }
        if f.searchable { out.push_str(" searchable"); }
        if f.sensitive { out.push_str(" sensitive"); }

        // Enum values
        if let Some(ref vals) = f.enum_values {
            out.push_str(&format!(" [{}]", vals.iter().map(|v| format!("\"{}\"", v)).collect::<Vec<_>>().join(", ")));
        }

        // Default value
        if let Some(ref d) = f.default {
            out.push_str(&format!(" default:\"{}\"", d));
        }

        out.push('\n');
    }

    // Transitions
    for t in &e.transitions {
        out.push_str(&format!("\n  transition {} {{\n", t.field));
        for (from, to) in &t.rules {
            out.push_str(&format!("    {} -> {}\n", from, to));
        }
        out.push_str("  }\n");
    }

    // Effects
    for eff in &e.effects {
        if !eff.actions.is_empty() {
            out.push_str(&format!("\n  on {} {{\n", eff.event));
            for a in &eff.actions {
                out.push_str(&format!("    {}\n", a));
            }
            out.push_str("  }\n");
        }
        if !eff.when_clauses.is_empty() {
            out.push_str(&format!("\n  on {} {{\n", eff.event));
            for (val, actions) in &eff.when_clauses {
                out.push_str(&format!("    when \"{}\" {{\n", val));
                for a in actions {
                    out.push_str(&format!("      {}\n", a));
                }
                out.push_str("    }\n");
            }
            out.push_str("  }\n");
        }
    }

    out.push_str("}\n");
    out
}

fn generate_api(entities: &[EntityRemap], options: &ComposeOptions) -> String {
    let mut out = String::new();
    for e in entities {
        let auth = if options.auth { " auth:jwt" } else { "" };
        out.push_str(&format!("api /{}s {{\n", e.entity_lower));
        out.push_str(&format!("  list GET /{}s{}\n", e.entity_lower, auth));
        out.push_str(&format!("  get GET /{}s/:id{}\n", e.entity_lower, auth));
        out.push_str(&format!("  create POST /{}s{}\n", e.entity_lower, auth));
        out.push_str(&format!("  update PATCH /{}s/:id{}\n", e.entity_lower, auth));
        out.push_str(&format!("  remove DELETE /{}s/:id{}\n", e.entity_lower, auth));
        out.push_str("}\n\n");
    }
    out
}

fn generate_admin_pages(entities: &[EntityRemap], _app_name: &str) -> String {
    let mut out = String::new();

    // Dashboard page
    out.push_str("page \"/\" requires:auth {\n");
    out.push_str("  section kpi {\n");
    for e in entities {
        out.push_str(&format!("    stat \"{}s\" bind {} {{ query count }}\n", e.entity, e.entity));
    }
    out.push_str("  }\n");
    // Recent table for first entity
    if let Some(first) = entities.first() {
        out.push_str(&format!("  section table {{\n    title \"Recent {}s\"\n", first.entity));
        let cols: Vec<&str> = first.fields.iter().take(5).map(|f| f.name.as_str()).collect();
        out.push_str(&format!("    columns \"{}\"\n", cols.join(", ")));
        out.push_str(&format!("    bind {} {{ query all order created_at desc limit 10 }}\n", first.entity));
        out.push_str("  }\n");
    }
    out.push_str("}\n\n");

    // CRUD pages per entity
    for e in entities {
        let cols: Vec<&str> = e.fields.iter().map(|f| f.name.as_str()).collect();
        out.push_str(&format!(r#"page "/{}s" requires:auth {{
  section table {{
    title "All {}s"
    columns "{}"
    bind {} {{ query all order created_at desc }}
    search true
    pagination 20
  }}
}}

"#, e.entity_lower, e.entity, cols.join(", "), e.entity));

        // Create form
        out.push_str(&format!("page \"/{}s/new\" requires:auth {{\n", e.entity_lower));
        out.push_str(&format!("  section form entity:{} {{\n", e.entity));
        out.push_str(&format!("    title \"New {}\"\n", e.entity));
        for f in &e.fields {
            let ftype = match f.field_type.as_str() {
                "email" => "email",
                "money" => "money",
                "number" => "number",
                "boolean" => "checkbox",
                "date" => "date",
                "text" => "text",
                _ => "text",
            };
            out.push_str(&format!("    field \"{}\" type:{}", f.name, ftype));
            if f.required { out.push_str(" required"); }
            out.push('\n');
        }
        out.push_str(&format!("    on submit {{\n      create entity\n      toast \"{} created\" style:success\n      navigate \"/{}s\"\n    }}\n", e.entity, e.entity_lower));
        out.push_str("  }\n}\n\n");
    }

    out
}

fn generate_scripts(entities: &[EntityRemap], app_name: &str) -> String {
    let mut out = format!(r#"## Auto-generated integration scripts
## Save as {}.scriptcronus

# script "{} Integrations" {{
#   version "1.0"
# }}
#
"#, app_name.to_lowercase(), app_name);

    for e in entities {
        out.push_str(&format!(r#"# on {}.create {{
#   log "New {}: {{{{event.record.{}}}}}"
# }}
#
"#, e.entity, e.entity_lower, e.fields.first().map(|f| f.name.as_str()).unwrap_or("id")));
    }

    out
}

// ── Compose Options ──

#[derive(Debug, Clone)]
pub struct StyleConfig {
    pub theme: String,
    pub accent: String,
    pub font: String,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub login_fields: String,       // "email + password"
    pub session_type: String,       // "jwt"
    pub expires: String,            // "24h"
    pub roles: Vec<String>,         // ["admin", "user"]
}

#[derive(Debug, Clone)]
pub struct ComposeOptions {
    pub port: u16,
    pub auth: bool,
    pub auth_config: Option<AuthConfig>,
    pub admin_pages: bool,
    pub scripts: bool,
    pub constitution: Vec<String>,
    pub style: Option<StyleConfig>,
}

impl Default for ComposeOptions {
    fn default() -> Self {
        Self {
            port: 5220,
            auth: true,
            auth_config: None,
            admin_pages: true,
            scripts: true,
            constitution: Vec::new(),
            style: None,
        }
    }
}

// ── Adapt: remap entity names ──

/// Replace all occurrences of one entity name with another in .cronus text.
pub fn adapt(source: &str, from: &str, to: &str) -> String {
    let from_lower = from.to_lowercase();
    let to_lower = to.to_lowercase();
    let from_upper = from.to_uppercase();
    let to_upper = to.to_uppercase();

    source
        .replace(from, to)
        .replace(&from_lower, &to_lower)
        .replace(&from_upper, &to_upper)
}

// ── Preset Templates ──

/// Predefined app templates that compose common block patterns.
/// Each template generates production-quality .cronus with enums, transitions,
/// relations, effects, constitution, and style — matching NovaPay reference quality.
pub fn get_template(name: &str) -> Option<(Vec<EntityRemap>, ComposeOptions)> {
    match name {
        "saas-billing" => Some(template_saas_billing()),
        "blog" => Some(template_blog()),
        "crm" => Some(template_crm()),
        "helpdesk" => Some(template_helpdesk()),
        "ecommerce" => Some(template_ecommerce()),
        _ => None,
    }
}

fn template_saas_billing() -> (Vec<EntityRemap>, ComposeOptions) {
    let entities = vec![
        EntityRemap::new("Customer", vec![
            FieldDef::new("name", "string").req().search(),
            FieldDef::new("email", "email").req().uniq().search(),
            FieldDef::new("company", "string").search(),
            FieldDef::new("phone", "string"),
            FieldDef::new("country", "string").default_val("BR"),
            FieldDef::new("plan_name", "string"),
            FieldDef::new("mrr", "money").default_val("0"),
            FieldDef::new("status", "enum").req().enums(&["active", "churned", "trial", "suspended"]).default_val("trial"),
        ]).shared().transition("status", &[
            ("trial", "active"), ("trial", "churned"),
            ("active", "suspended"), ("active", "churned"),
            ("suspended", "active"), ("churned", "active"),
        ]).on_create(&["notify \"log\" \"New customer registered\""]),

        EntityRemap::new("Plan", vec![
            FieldDef::new("name", "string").req().uniq(),
            FieldDef::new("slug", "slug").req().uniq(),
            FieldDef::new("price", "money").req(),
            FieldDef::new("interval", "enum").req().enums(&["monthly", "yearly"]).default_val("monthly"),
            FieldDef::new("trial_days", "number").default_val("14"),
            FieldDef::new("features", "text"),
            FieldDef::new("active", "boolean").default_val("true"),
            FieldDef::new("sort_order", "number").default_val("0"),
        ]).shared(),

        EntityRemap::new("Subscription", vec![
            FieldDef::new("customer", "relation").refs("Customer"),
            FieldDef::new("plan", "relation").refs("Plan"),
            FieldDef::new("status", "enum").req().enums(&["trialing", "active", "past_due", "canceled", "paused"]).default_val("trialing"),
            FieldDef::new("current_period_start", "string"),
            FieldDef::new("current_period_end", "string"),
            FieldDef::new("cancel_at", "string"),
            FieldDef::new("canceled_reason", "string"),
        ]).shared().transition("status", &[
            ("trialing", "active"), ("trialing", "canceled"),
            ("active", "past_due"), ("active", "canceled"), ("active", "paused"),
            ("past_due", "active"), ("past_due", "canceled"),
            ("paused", "active"), ("paused", "canceled"),
        ]).on_create(&["notify \"log\" \"Subscription created\""])
          .on_update("status", &[
              ("canceled", &["notify \"log\" \"Subscription canceled\""]),
              ("active", &["notify \"log\" \"Subscription activated\""]),
          ]),

        EntityRemap::new("Invoice", vec![
            FieldDef::new("subscription", "relation").refs("Subscription"),
            FieldDef::new("customer", "relation").refs("Customer"),
            FieldDef::new("amount", "money").req(),
            FieldDef::new("currency", "string").default_val("BRL"),
            FieldDef::new("status", "enum").req().enums(&["draft", "open", "paid", "void", "uncollectible"]).default_val("draft"),
            FieldDef::new("due_date", "string"),
            FieldDef::new("paid_at", "string"),
            FieldDef::new("invoice_number", "string").uniq(),
        ]).shared().transition("status", &[
            ("draft", "open"), ("open", "paid"), ("open", "void"),
            ("open", "uncollectible"), ("void", "draft"),
        ]).on_update("status", &[
            ("paid", &["notify \"log\" \"Invoice paid\""]),
        ]),

        EntityRemap::new("PaymentMethod", vec![
            FieldDef::new("customer", "relation").refs("Customer"),
            FieldDef::new("type", "enum").req().enums(&["credit_card", "pix", "boleto", "wire"]).default_val("credit_card"),
            FieldDef::new("last_four", "string"),
            FieldDef::new("brand", "string"),
            FieldDef::new("exp_month", "number"),
            FieldDef::new("exp_year", "number"),
            FieldDef::new("is_default", "boolean").default_val("true"),
        ]).shared(),

        EntityRemap::new("UsageRecord", vec![
            FieldDef::new("customer", "relation").refs("Customer"),
            FieldDef::new("metric", "string").req(),
            FieldDef::new("value", "number").req(),
            FieldDef::new("period", "string").req(),
            FieldDef::new("recorded_at", "string"),
        ]).shared(),
    ];

    let opts = ComposeOptions {
        port: 5220,
        auth: true,
        auth_config: Some(AuthConfig {
            login_fields: "email + password".into(),
            session_type: "jwt".into(),
            expires: "24h".into(),
            roles: vec!["admin".into(), "billing".into(), "viewer".into()],
        }),
        admin_pages: true,
        scripts: true,
        constitution: vec![
            "prices stored in centavos — 2990 = R$29.90".into(),
            "every subscription has exactly one active plan".into(),
            "never \"delete a customer with active subscriptions\"".into(),
        ],
        style: Some(StyleConfig { theme: "dark".into(), accent: "white".into(), font: "Inter".into() }),
    };

    (entities, opts)
}

fn template_blog() -> (Vec<EntityRemap>, ComposeOptions) {
    let entities = vec![
        EntityRemap::new("Post", vec![
            FieldDef::new("title", "string").req().search(),
            FieldDef::new("slug", "slug").req().uniq(),
            FieldDef::new("content", "text"),
            FieldDef::new("excerpt", "text"),
            FieldDef::new("author", "relation").refs("User"),
            FieldDef::new("category", "relation").refs("Category"),
            FieldDef::new("status", "enum").req().enums(&["draft", "review", "published", "archived"]).default_val("draft"),
            FieldDef::new("featured", "boolean").default_val("false"),
            FieldDef::new("published_at", "string"),
        ]).shared().transition("status", &[
            ("draft", "review"), ("review", "published"), ("review", "draft"),
            ("published", "archived"), ("archived", "draft"),
        ]),

        EntityRemap::new("Category", vec![
            FieldDef::new("name", "string").req().uniq(),
            FieldDef::new("slug", "slug").req().uniq(),
            FieldDef::new("description", "text"),
        ]).shared(),

        EntityRemap::new("Tag", vec![
            FieldDef::new("name", "string").req().uniq(),
            FieldDef::new("slug", "slug").req().uniq(),
        ]).shared(),

        EntityRemap::new("Comment", vec![
            FieldDef::new("post", "relation").refs("Post"),
            FieldDef::new("author_name", "string").req(),
            FieldDef::new("author_email", "email").req(),
            FieldDef::new("body", "text").req(),
            FieldDef::new("approved", "boolean").default_val("false"),
        ]).shared(),
    ];

    let opts = ComposeOptions {
        port: 5210,
        auth: true,
        auth_config: Some(AuthConfig {
            login_fields: "email + password".into(),
            session_type: "jwt".into(),
            expires: "7d".into(),
            roles: vec!["admin".into(), "editor".into(), "writer".into()],
        }),
        admin_pages: true,
        scripts: true,
        constitution: vec![
            "slugs must be URL-safe".into(),
            "published posts require content".into(),
            "comments require moderation before display".into(),
        ],
        style: Some(StyleConfig { theme: "dark".into(), accent: "#f472b6".into(), font: "Inter".into() }),
    };

    (entities, opts)
}

fn template_crm() -> (Vec<EntityRemap>, ComposeOptions) {
    let entities = vec![
        EntityRemap::new("Company", vec![
            FieldDef::new("name", "string").req().search(),
            FieldDef::new("domain", "url"),
            FieldDef::new("industry", "enum").enums(&["tech", "finance", "healthcare", "retail", "other"]),
            FieldDef::new("size", "enum").enums(&["1-10", "11-50", "51-200", "201-1000", "1000+"]),
            FieldDef::new("annual_revenue", "money"),
            FieldDef::new("phone", "string"),
        ]).shared(),

        EntityRemap::new("Contact", vec![
            FieldDef::new("name", "string").req().search(),
            FieldDef::new("email", "email").req().search(),
            FieldDef::new("phone", "string"),
            FieldDef::new("title", "string"),
            FieldDef::new("company", "relation").refs("Company"),
            FieldDef::new("source", "enum").enums(&["website", "referral", "linkedin", "cold", "event", "other"]),
        ]).shared(),

        EntityRemap::new("Deal", vec![
            FieldDef::new("title", "string").req().search(),
            FieldDef::new("value", "money").req(),
            FieldDef::new("company", "relation").refs("Company"),
            FieldDef::new("contact", "relation").refs("Contact"),
            FieldDef::new("stage", "enum").req().enums(&["lead", "qualified", "proposal", "negotiation", "won", "lost"]).default_val("lead"),
            FieldDef::new("probability", "number"),
            FieldDef::new("expected_close", "string"),
            FieldDef::new("lost_reason", "text"),
        ]).shared().transition("stage", &[
            ("lead", "qualified"), ("qualified", "proposal"),
            ("proposal", "negotiation"), ("negotiation", "won"), ("negotiation", "lost"),
            ("lead", "lost"), ("qualified", "lost"), ("proposal", "lost"),
        ]),

        EntityRemap::new("Activity", vec![
            FieldDef::new("type", "enum").req().enums(&["call", "email", "meeting", "note", "task"]),
            FieldDef::new("subject", "string").req(),
            FieldDef::new("description", "text"),
            FieldDef::new("contact", "relation").refs("Contact"),
            FieldDef::new("deal", "relation").refs("Deal"),
            FieldDef::new("completed", "boolean").default_val("false"),
            FieldDef::new("due_date", "string"),
        ]).shared(),
    ];

    let opts = ComposeOptions {
        port: 5220,
        auth: true,
        auth_config: Some(AuthConfig {
            login_fields: "email + password".into(),
            session_type: "jwt".into(),
            expires: "24h".into(),
            roles: vec!["admin".into(), "manager".into(), "rep".into()],
        }),
        admin_pages: true,
        scripts: true,
        constitution: vec![
            "prices stored in centavos".into(),
            "deal stage changes must be logged".into(),
            "contacts require email or phone".into(),
        ],
        style: Some(StyleConfig { theme: "dark".into(), accent: "#3b82f6".into(), font: "Inter".into() }),
    };

    (entities, opts)
}

fn template_helpdesk() -> (Vec<EntityRemap>, ComposeOptions) {
    let entities = vec![
        EntityRemap::new("Agent", vec![
            FieldDef::new("name", "string").req().search(),
            FieldDef::new("email", "email").req().uniq(),
            FieldDef::new("role", "enum").req().enums(&["admin", "lead", "agent"]).default_val("agent"),
            FieldDef::new("max_tickets", "number").default_val("20"),
            FieldDef::new("active", "boolean").default_val("true"),
        ]).shared(),

        EntityRemap::new("Customer", vec![
            FieldDef::new("name", "string").req().search(),
            FieldDef::new("email", "email").req().search(),
            FieldDef::new("phone", "string"),
            FieldDef::new("company", "string"),
        ]).shared(),

        EntityRemap::new("Ticket", vec![
            FieldDef::new("subject", "string").req().search(),
            FieldDef::new("description", "text").req(),
            FieldDef::new("customer", "relation").refs("Customer"),
            FieldDef::new("agent", "relation").refs("Agent"),
            FieldDef::new("priority", "enum").req().enums(&["critical", "high", "medium", "low"]).default_val("medium"),
            FieldDef::new("status", "enum").req().enums(&["open", "assigned", "in_progress", "waiting", "resolved", "closed"]).default_val("open"),
            FieldDef::new("category", "enum").enums(&["bug", "feature", "billing", "account", "other"]),
            FieldDef::new("sla_deadline", "string"),
            FieldDef::new("resolution", "text"),
            FieldDef::new("satisfaction", "number"),
        ]).shared().transition("status", &[
            ("open", "assigned"), ("assigned", "in_progress"),
            ("in_progress", "waiting"), ("in_progress", "resolved"),
            ("waiting", "in_progress"), ("waiting", "resolved"),
            ("resolved", "closed"), ("resolved", "open"),
        ]),

        EntityRemap::new("Message", vec![
            FieldDef::new("ticket", "relation").refs("Ticket"),
            FieldDef::new("sender_type", "enum").req().enums(&["agent", "customer", "system"]),
            FieldDef::new("sender_name", "string").req(),
            FieldDef::new("body", "text").req(),
            FieldDef::new("internal", "boolean").default_val("false"),
        ]).shared(),
    ];

    let opts = ComposeOptions {
        port: 5220,
        auth: true,
        auth_config: Some(AuthConfig {
            login_fields: "email + password".into(),
            session_type: "jwt".into(),
            expires: "24h".into(),
            roles: vec!["admin".into(), "lead".into(), "agent".into()],
        }),
        admin_pages: true,
        scripts: true,
        constitution: vec![
            "critical tickets must be responded within 1 hour".into(),
            "status changes must emit webhooks".into(),
            "resolved tickets require resolution note".into(),
        ],
        style: Some(StyleConfig { theme: "dark".into(), accent: "#f59e0b".into(), font: "Inter".into() }),
    };

    (entities, opts)
}

fn template_ecommerce() -> (Vec<EntityRemap>, ComposeOptions) {
    let entities = vec![
        EntityRemap::new("Category", vec![
            FieldDef::new("name", "string").req().uniq().search(),
            FieldDef::new("slug", "slug").req().uniq(),
            FieldDef::new("description", "text"),
            FieldDef::new("sort_order", "number").default_val("0"),
        ]).shared(),

        EntityRemap::new("Product", vec![
            FieldDef::new("name", "string").req().search(),
            FieldDef::new("slug", "slug").req().uniq(),
            FieldDef::new("price", "money").req(),
            FieldDef::new("compare_price", "money"),
            FieldDef::new("sku", "string").uniq(),
            FieldDef::new("stock", "number").default_val("0"),
            FieldDef::new("category", "relation").refs("Category"),
            FieldDef::new("description", "text"),
            FieldDef::new("image_url", "url"),
            FieldDef::new("active", "boolean").default_val("true"),
        ]).shared(),

        EntityRemap::new("Customer", vec![
            FieldDef::new("name", "string").req().search(),
            FieldDef::new("email", "email").req().uniq().search(),
            FieldDef::new("phone", "string"),
            FieldDef::new("address", "string"),
            FieldDef::new("city", "string"),
            FieldDef::new("state", "string"),
            FieldDef::new("zip", "string"),
        ]).shared(),

        EntityRemap::new("Order", vec![
            FieldDef::new("customer", "relation").refs("Customer"),
            FieldDef::new("total", "money").req(),
            FieldDef::new("status", "enum").req().enums(&["cart", "pending", "paid", "shipped", "delivered", "cancelled"]).default_val("pending"),
            FieldDef::new("shipping_address", "text"),
            FieldDef::new("tracking_code", "string"),
            FieldDef::new("notes", "text"),
        ]).shared().transition("status", &[
            ("cart", "pending"), ("pending", "paid"),
            ("paid", "shipped"), ("shipped", "delivered"),
            ("pending", "cancelled"), ("paid", "cancelled"),
        ]),

        EntityRemap::new("OrderItem", vec![
            FieldDef::new("order", "relation").refs("Order"),
            FieldDef::new("product", "relation").refs("Product"),
            FieldDef::new("quantity", "number").req(),
            FieldDef::new("unit_price", "money").req(),
        ]).shared(),
    ];

    let opts = ComposeOptions {
        port: 5200,
        auth: true,
        auth_config: Some(AuthConfig {
            login_fields: "email + password".into(),
            session_type: "jwt".into(),
            expires: "24h".into(),
            roles: vec!["admin".into(), "staff".into()],
        }),
        admin_pages: true,
        scripts: true,
        constitution: vec![
            "prices stored in centavos — 2990 = R$29.90".into(),
            "never \"negative stock\"".into(),
            "order total must equal sum of items".into(),
        ],
        style: Some(StyleConfig { theme: "dark".into(), accent: "#10b981".into(), font: "Inter".into() }),
    };

    (entities, opts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_app_generates_valid_output() {
        let entities = vec![
            EntityRemap::new("Product", vec![
                FieldDef::new("name", "string").req(),
                FieldDef::new("price", "money").req(),
            ]),
        ];
        let opts = ComposeOptions { port: 3000, auth: true, admin_pages: true, scripts: false, ..Default::default() };
        let output = compose_app("TestApp", &entities, &opts);

        assert!(output.contains("app \"TestApp\""), "should have app block");
        assert!(output.contains("entity Product"), "should have entity");
        assert!(output.contains("entity User"), "should have User entity (auth)");
        assert!(output.contains("name string!"), "should have typed fields with !");
        assert!(output.contains("GET /products"), "should have API routes");
        assert!(output.contains("POST /products"), "should have create route");
        assert!(output.contains("page \"/products\""), "should have list page");
        assert!(output.contains("page \"/products/new\""), "should have create page");
        assert!(output.contains("section form"), "should have form");
        assert!(output.contains("bind Product"), "should have binding");
    }

    #[test]
    fn test_adapt_renames_entity() {
        let source = "entity Product {\n  name string\n}\nGET /products { query Product { all } }";
        let adapted = adapt(source, "Product", "Service");
        assert!(adapted.contains("entity Service"));
        assert!(adapted.contains("/services"));
        assert!(adapted.contains("query Service"));
        assert!(!adapted.contains("Product"));
    }

    #[test]
    fn test_template_saas_billing() {
        let (entities, opts) = get_template("saas-billing").unwrap();
        assert_eq!(entities.len(), 6); // Customer, Plan, Subscription, Invoice, PaymentMethod, UsageRecord
        assert_eq!(entities[0].entity, "Customer");
        assert!(entities[0].shared);
        assert!(!entities[0].transitions.is_empty());
        assert!(opts.auth);
        assert!(!opts.constitution.is_empty());
    }

    #[test]
    fn test_all_templates_exist() {
        assert!(get_template("saas-billing").is_some());
        assert!(get_template("blog").is_some());
        assert!(get_template("crm").is_some());
        assert!(get_template("helpdesk").is_some());
        assert!(get_template("ecommerce").is_some());
        assert!(get_template("nonexistent").is_none());
    }

    #[test]
    fn test_compose_generates_transitions() {
        let (entities, opts) = get_template("saas-billing").unwrap();
        let output = compose_app("NovaPay", &entities, &opts);
        assert!(output.contains("transition status"), "should have transition blocks");
        assert!(output.contains("trial -> active"), "should have transition rules");
        assert!(output.contains("-> Customer"), "should have relations");
        assert!(output.contains("shared"), "should have shared modifier");
        assert!(output.contains("constitution"), "should have constitution");
        assert!(output.contains("style {"), "should have style block");
    }

    #[test]
    fn test_compose_generates_enums() {
        let (entities, opts) = get_template("saas-billing").unwrap();
        let output = compose_app("NovaPay", &entities, &opts);
        assert!(output.contains("enum"), "should have enum fields");
        assert!(output.contains("\"active\""), "should have enum values");
        assert!(output.contains("default:"), "should have default values");
    }
}
