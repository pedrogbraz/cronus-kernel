use std::collections::HashMap;

// ══════════════════════════════════════════════════
// AST TYPES
// ══════════════════════════════════════════════════

#[derive(Debug, Clone, Default)]
pub struct DocComment {
    pub summary: String,
    pub description: String,
    pub tags: Vec<DocTag>,
}

#[derive(Debug, Clone)]
pub struct DocTag {
    pub name: String,
    pub value: String,
}

/// Source location of a top-level declaration (1-based).
#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Default for Span {
    fn default() -> Self {
        Self { line: 1, col: 1 }
    }
}

pub enum AstNode {
    App(AppNode),
    Entity(EntityNode),
    Api(ApiNode),
    Page(PageNode),
    Style(StyleNode),
    Service(ServiceNode),
    Component(ComponentNode),
    Import(ImportNode),
    Event(EventNode),
    Worker(WorkerNode),
    Middleware(MiddlewareNode),
    Env(EnvNode),
    Test(TestNode),
    Compose(ComposeNode),
    Auth(AuthNode),
    Layout(LayoutNode),
    Define(DefineNode),
    Webhook(WebhookNode),
    Deploy(DeployNode),
}

/// A reusable section definition: `define sidebar "Name" { ... }`
/// Pages reference via `use Name` which expands to the defined sections.
#[derive(Debug, Clone)]
pub struct DefineNode {
    pub name: String,
    pub sections: Vec<SectionNode>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub db_type: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConstitutionNode {
    pub must: Vec<String>,
    pub never: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AppNode {
    pub name: String,
    pub stack: Vec<String>,
    pub port: u16,
    pub database: Option<DatabaseConfig>,
    /// Inline Tailwind config JS (extracted from dumped sites)
    pub tailwind_config: Option<String>,
    /// Unbreakable rules defined inline in the app block
    pub constitution: Option<ConstitutionNode>,
    /// `app { graphql false }` unmounts `/graphql`. Default true.
    pub graphql: bool,
    /// Doc-comment attached to the app block
    pub doc: Option<DocComment>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EntityNode {
    pub name: String,
    pub fields: Vec<FieldNode>,
    /// `jobs <- Job.client` — named reverse of another entity's relation.
    pub reverses: Vec<ReverseDecl>,
    pub transitions: Vec<TransitionNode>,
    pub effects: Vec<EffectBlock>,
    pub shared: bool,
    pub remote_url: Option<String>,
    pub doc: Option<DocComment>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReverseDecl {
    pub name: String,
    pub source_entity: String,
    pub source_field: String,
}

#[derive(Debug, Clone)]
pub struct EffectBlock {
    pub event: String,         // "create", "update", "delete"
    pub field: Option<String>, // for "on update status" — which field triggers
    pub actions: Vec<EffectAction>,
}

#[derive(Debug, Clone)]
pub struct EffectAction {
    pub action_type: String,       // "log", "notify"
    pub args: Vec<String>,         // for log: [message]; for notify: [provider, channel, message]
    pub condition: Option<String>, // "when" value (e.g., "Failed")
}

#[derive(Debug, Clone)]
pub struct TransitionNode {
    pub field: String,
    pub rules: Vec<TransitionRule>,
}

#[derive(Debug, Clone)]
pub struct TransitionRule {
    pub from: String,
    pub to: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FieldNode {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub unique: bool,
    pub sensitive: bool,
    pub optional: bool,
    pub searchable: bool,
    pub index: bool,
    pub featured: bool,
    pub formatted: bool,
    pub array: bool,
    pub enum_values: Option<Vec<String>>,
    pub reference: Option<String>,
    pub doc: Option<DocComment>,
    pub default_value: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
}

impl FieldNode {
    /// `tags -> Tag[]`: a many-to-many relation stored in the join table
    /// `<Entity>_<field>` (see `relations.rs`), never a column.
    pub fn is_many(&self) -> bool {
        self.field_type == FieldType::Relation && self.array
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Text,
    Email,
    Url,
    Slug,
    Phone,
    Number,
    Money,
    Percentage,
    Boolean,
    Date,
    DateTime,
    Ulid,
    Json,
    Enum,
    Ip,
    File,
    Relation,
}

/// Canonical field type keywords (relation is written `-> Entity`, not a keyword).
pub const FIELD_TYPE_KEYWORDS: &[&str] = &[
    "string",
    "text",
    "email",
    "url",
    "file",
    "slug",
    "phone",
    "number",
    "money",
    "percentage",
    "boolean",
    "date",
    "datetime",
    "ulid",
    "json",
    "enum",
    "ip",
];

/// Accepted aliases → canonical keyword. Documented in LANGUAGE.md §2.3.
/// These are the spellings emitted by TypeScript/OpenAPI-style sources.
pub const FIELD_TYPE_ALIASES: &[(&str, &str)] = &[
    ("int", "number"),
    ("integer", "number"),
    ("float", "number"),
    ("decimal", "number"),
    ("bool", "boolean"),
    ("timestamp", "datetime"),
];

impl FieldType {
    /// Look up a field type keyword or alias. `None` for unknown spellings —
    /// the parser reports those as `TYPE_001` instead of guessing.
    pub(crate) fn from_keyword(s: &str) -> Option<Self> {
        let canonical = FIELD_TYPE_ALIASES
            .iter()
            .find(|(alias, _)| *alias == s)
            .map(|(_, c)| *c)
            .unwrap_or(s);
        Some(match canonical {
            "string" => FieldType::String,
            "text" => FieldType::Text,
            "email" => FieldType::Email,
            "url" => FieldType::Url,
            "file" => FieldType::File,
            "slug" => FieldType::Slug,
            "phone" => FieldType::Phone,
            "number" => FieldType::Number,
            "money" => FieldType::Money,
            "percentage" => FieldType::Percentage,
            "boolean" => FieldType::Boolean,
            "date" => FieldType::Date,
            "datetime" => FieldType::DateTime,
            "ulid" => FieldType::Ulid,
            "json" => FieldType::Json,
            "enum" => FieldType::Enum,
            "ip" => FieldType::Ip,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PATCH,
    PUT,
    DELETE,
}

impl HttpMethod {
    pub(crate) fn from_str(s: &str) -> Option<Self> {
        match s {
            "GET" => Some(HttpMethod::GET),
            "POST" => Some(HttpMethod::POST),
            "PATCH" => Some(HttpMethod::PATCH),
            "PUT" => Some(HttpMethod::PUT),
            "DELETE" => Some(HttpMethod::DELETE),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouteNode {
    pub name: String,
    pub method: HttpMethod,
    pub path: String,
    pub auth: String,
    pub roles: Vec<String>,
    pub doc: Option<DocComment>,
}

#[derive(Debug, Clone)]
pub struct ApiNode {
    pub prefix: String,
    pub routes: Vec<RouteNode>,
    pub doc: Option<DocComment>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WebhookNode {
    pub entity: String,
    pub hooks: Vec<WebhookHook>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WebhookHook {
    pub event: String,  // "create", "update", "delete"
    pub method: String, // "POST", "PUT"
    pub url: String,
    pub headers: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct VisibilityCondition {
    pub field: String,
    pub operator: String, // "==", "!=", ">", "<", ">=", "<="
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct SectionNode {
    pub section_type: String,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub config: HashMap<String, String>,
    pub items: Vec<HashMap<String, String>>,
    pub plans: Vec<PlanNode>,
    pub binding: Option<BindingNode>,
    pub actions: Vec<ActionBlock>,
    pub visibility: Option<VisibilityCondition>,
    pub template: Option<String>, // raw HTML template for visual preservation
    pub style_block: Option<String>, // scoped CSS for visual preservation
    pub doc: Option<DocComment>,
}

#[derive(Debug, Clone)]
pub struct PlanNode {
    pub name: String,
    pub price: String,
    pub featured: bool,
    pub features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BindingNode {
    pub entity: String,
    pub query: QueryType,
    pub filters: Vec<FilterExpr>,
    pub order: Option<OrderExpr>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub group_by: Option<GroupByExpr>,
    pub aggregate: Option<AggregateExpr>,
    pub live: bool, // real-time updates via SSE
    /// `bind X { scope:public }` — skip `_owner_id` filter (marketing, shared catalogs).
    pub public: bool,
    /// `expand:tags,author` — related rows (not just ids) on those fields.
    pub expand: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GroupByExpr {
    pub field: String,
    pub interval: Option<String>, // "month", "week", "day", "year"
}

#[derive(Debug, Clone)]
pub struct AggregateExpr {
    pub function: String,      // "sum", "count", "avg", "min", "max"
    pub field: Option<String>, // None for count, Some("amount") for sum(amount)
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryType {
    All,
    One,
    Count,
}

#[derive(Debug, Clone)]
pub struct FilterExpr {
    pub field: String,
    pub operator: FilterOp,
    pub value: BindingValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterOp {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
    StartsWith,
    EndsWith,
    In,
}

#[derive(Debug, Clone)]
pub struct OrderExpr {
    pub field: String,
    pub direction: OrderDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub enum BindingValue {
    Str(String),
    Num(String),
    Bool(bool),
    AuthRef(String),
    /// `where status in:["paid", "shipped"]`
    List(Vec<BindingValue>),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ActionInstruction {
    pub verb: String, // "set", "toast", "navigate", "refresh", "create", "confirm", "delete", "validate", "open", "close"
    pub target: String, // field name, URL, message text, section ref
    pub value: String, // new value for "set", style for "toast"
    /// Toast `style:`; `create`/`update` field literals (`title:"x"` or `{ title "x" }`).
    pub modifiers: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ActionBlock {
    pub event: String, // "click", "submit", "error", "change"
    pub confirm: Option<String>,
    pub instructions: Vec<ActionInstruction>,
}

#[derive(Debug, Clone)]
pub struct PageNode {
    pub route: String,
    pub page_type: String,
    pub entity: Option<String>,
    pub title: Option<String>,
    pub sections: Vec<SectionNode>,
    pub config: HashMap<String, String>,
    pub components: Vec<String>, // referenced component names via `use ComponentName`
    pub requires: Option<String>, // "auth", "role(admin)", etc.
    pub doc: Option<DocComment>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StyleNode {
    pub theme: Option<String>,
    pub accent: Option<String>,
    pub radius: Option<String>,
    pub font: Option<String>,
    pub config: HashMap<String, String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ServiceNode {
    pub name: String,
    pub port: Option<u16>,
    pub config: HashMap<String, String>,
}

/// The 15 official component item types
pub const OFFICIAL_ITEM_TYPES: &[&str] = &[
    "label", "text", "title", "subtitle", "value", "trend", "icon", "action", "item", "tab",
    "plan", "field", "source", "columns", "slot", // Also allow common UI types:
    "link", "button", "badge", "cta", "dot", "meta",
];

/// Check if an item type is officially recognized
pub fn is_valid_item_type(t: &str) -> bool {
    OFFICIAL_ITEM_TYPES.contains(&t)
}

#[derive(Debug, Clone)]
pub struct ComponentItemNode {
    pub item_type: String, // one of OFFICIAL_ITEM_TYPES
    pub text: String,
    pub link: Option<String>,
    pub tone: Option<String>, // success, danger, primary, secondary, accent, default
    pub config: HashMap<String, String>, // key:value pairs (price:$29/mo, featured:true, etc)
}

#[derive(Debug, Clone)]
pub struct ComponentParam {
    pub name: String,
    pub param_type: String, // text, money, integer, boolean, etc. or "any"
    pub default: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct ComponentState {
    pub name: String,
    pub state_type: String, // integer, text, boolean, etc.
    pub default: String,
}

#[derive(Debug, Clone)]
pub struct ComponentTest {
    pub name: String,       // test description
    pub steps: Vec<String>, // "fill email \"test@test.com\"", "click \"Login\"", "expect visible \".error\""
}

#[derive(Debug, Clone)]
pub struct ComponentNode {
    pub name: String,
    pub layout: Option<String>,
    pub style: Option<String>,
    pub items: Vec<ComponentItemNode>,
    pub props: HashMap<String, String>,
    pub params: Vec<ComponentParam>,
    pub template: Option<String>,
    pub sections: Vec<SectionNode>,
    pub state: Vec<ComponentState>,
    pub tests: Vec<ComponentTest>, // co-located test blocks
    /// Optional `bind Entity { ... }` so a widget can read live rows/count.
    pub binding: Option<BindingNode>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ImportNode {
    pub alias: String,
    pub source: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct EventNode {
    pub name: String,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkerNode {
    pub name: String,
    pub queue: Option<String>,
    pub concurrency: Option<u32>,
    pub retry: Option<u32>,
    pub timeout: Option<String>,
    pub entity: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MiddlewareNode {
    pub name: String,
    pub applies_to: Option<Vec<String>>,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct EnvNode {
    /// Optional block name (`env production { … }`); empty for `env { … }`.
    pub name: String,
    /// Legacy `KEY value` pairs.
    pub vars: HashMap<String, String>,
    /// Declared variables (`APP_KEY string! sensitive`), checked by `cronus run`.
    pub schema: Vec<EnvVarSpec>,
    pub span: Span,
}

/// Types an `env { … }` variable may declare.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvType {
    String,
    Number,
    Boolean,
    Url,
    Email,
}

impl EnvType {
    pub const KEYWORDS: &'static [&'static str] = &["string", "number", "boolean", "url", "email"];

    /// Accepts the canonical keyword or a field-type alias (`int`, `bool`).
    pub fn from_keyword(s: &str) -> Option<Self> {
        match FieldType::from_keyword(s)? {
            FieldType::String => Some(EnvType::String),
            FieldType::Number => Some(EnvType::Number),
            FieldType::Boolean => Some(EnvType::Boolean),
            FieldType::Url => Some(EnvType::Url),
            FieldType::Email => Some(EnvType::Email),
            _ => None,
        }
    }

    pub fn keyword(self) -> &'static str {
        match self {
            EnvType::String => "string",
            EnvType::Number => "number",
            EnvType::Boolean => "boolean",
            EnvType::Url => "url",
            EnvType::Email => "email",
        }
    }

    /// Whether a raw environment value has this type.
    pub fn accepts(self, value: &str) -> bool {
        let v = value.trim();
        match self {
            EnvType::String => true,
            EnvType::Number => v.parse::<f64>().is_ok_and(f64::is_finite),
            EnvType::Boolean => {
                matches!(
                    v.to_ascii_lowercase().as_str(),
                    "true" | "false" | "1" | "0"
                )
            }
            EnvType::Url => {
                (v.starts_with("http://") || v.starts_with("https://")) && !v.contains(' ')
            }
            EnvType::Email => v
                .split_once('@')
                .is_some_and(|(l, d)| !l.is_empty() && d.contains('.') && !d.ends_with('.')),
        }
    }
}

/// One declared environment variable: `APP_STRIPE_KEY string! sensitive`.
#[derive(Debug, Clone, PartialEq)]
pub struct EnvVarSpec {
    pub name: String,
    pub env_type: EnvType,
    pub required: bool,
    /// Value is never printed (also true in practice for every value: startup
    /// errors name variables, never their values).
    pub sensitive: bool,
    pub default: Option<String>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct TestStepNode {
    pub action: String,
    pub entity: String,
    pub body: HashMap<String, String>,
    pub expect: u16,
    pub expect_config: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TestNode {
    pub name: String,
    pub steps: Vec<TestStepNode>,
}

#[derive(Debug, Clone)]
pub struct ComposeNode {
    pub name: String,
    pub uses: Vec<String>,
    pub merges: Vec<(String, HashMap<String, String>)>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct AuthNode {
    pub entity: String,                          // "User"
    pub login_fields: Vec<String>,               // ["email", "password"]
    pub session_type: String,                    // "jwt"
    pub session_config: HashMap<String, String>, // expires: "24h"
    pub roles: Vec<String>,                      // ["admin", "member", "viewer"]
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub name: String,
    pub sidebar_items: Vec<LayoutNavItem>,
    pub sidebar_config: HashMap<String, String>, // brand, etc
    pub topbar_config: HashMap<String, String>,  // search placeholder, etc
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LayoutNavItem {
    pub label: String,
    pub route: String,
    pub icon: Option<String>,
    pub requires: Option<String>, // role requirement
    pub is_divider: bool,
}

#[derive(Debug, Clone)]
pub struct DeployNode {
    pub mode: String,
    pub gateway: Option<GatewayConfig>,
    pub services: Vec<DeployServiceDef>,
}

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub port: u16,
    pub provider: String,
    pub cors: Option<String>,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct DeployServiceDef {
    pub name: String,
    pub port: u16,
    pub db: Option<String>,
    pub entities: Vec<String>,
    pub apis: Vec<String>,
    pub pages: Vec<String>,
    pub config: HashMap<String, String>,
}
