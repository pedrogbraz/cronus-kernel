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
    /// Doc-comment attached to the app block
    pub doc: Option<DocComment>,
}

#[derive(Debug, Clone)]
pub struct EntityNode {
    pub name: String,
    pub fields: Vec<FieldNode>,
    pub transitions: Vec<TransitionNode>,
    pub effects: Vec<EffectBlock>,
    pub shared: bool,
    pub remote_url: Option<String>,
    pub doc: Option<DocComment>,
}

#[derive(Debug, Clone)]
pub struct EffectBlock {
    pub event: String,           // "create", "update", "delete"
    pub field: Option<String>,   // for "on update status" — which field triggers
    pub actions: Vec<EffectAction>,
}

#[derive(Debug, Clone)]
pub struct EffectAction {
    pub action_type: String,     // "log", "notify"
    pub args: Vec<String>,       // for log: [message]; for notify: [provider, channel, message]
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
    Ulid,
    Json,
    Enum,
    Ip,
    Relation,
}

impl FieldType {
    pub(crate) fn from_str(s: &str) -> Self {
        match s {
            "string" => FieldType::String,
            "text" => FieldType::Text,
            "email" => FieldType::Email,
            "url" => FieldType::Url,
            "slug" => FieldType::Slug,
            "phone" => FieldType::Phone,
            "number" => FieldType::Number,
            "money" => FieldType::Money,
            "percentage" => FieldType::Percentage,
            "boolean" => FieldType::Boolean,
            "date" => FieldType::Date,
            "ulid" => FieldType::Ulid,
            "json" => FieldType::Json,
            "enum" => FieldType::Enum,
            "ip" => FieldType::Ip,
            _ => FieldType::String,
        }
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
}

#[derive(Debug, Clone)]
pub struct WebhookNode {
    pub entity: String,
    pub hooks: Vec<WebhookHook>,
}

#[derive(Debug, Clone)]
pub struct WebhookHook {
    pub event: String,   // "create", "update", "delete"
    pub method: String,  // "POST", "PUT"
    pub url: String,
    pub headers: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct VisibilityCondition {
    pub field: String,
    pub operator: String,  // "==", "!=", ">", "<", ">=", "<="
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
    pub template: Option<String>,      // raw HTML template for visual preservation
    pub style_block: Option<String>,   // scoped CSS for visual preservation
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
    pub live: bool,  // real-time updates via SSE
}

#[derive(Debug, Clone)]
pub struct GroupByExpr {
    pub field: String,
    pub interval: Option<String>, // "month", "week", "day", "year"
}

#[derive(Debug, Clone)]
pub struct AggregateExpr {
    pub function: String,        // "sum", "count", "avg", "min", "max"
    pub field: Option<String>,   // None for count, Some("amount") for sum(amount)
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryType { All, One, Count }

#[derive(Debug, Clone)]
pub struct FilterExpr {
    pub field: String,
    pub operator: FilterOp,
    pub value: BindingValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterOp { Eq, Ne, Gt, Gte, Lt, Lte, Contains, StartsWith }

#[derive(Debug, Clone)]
pub struct OrderExpr {
    pub field: String,
    pub direction: OrderDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderDirection { Asc, Desc }

#[derive(Debug, Clone)]
pub enum BindingValue {
    Str(String),
    Num(String),
    Bool(bool),
    AuthRef(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActionInstruction {
    pub verb: String,        // "set", "toast", "navigate", "refresh", "create", "confirm", "delete", "validate", "open", "close"
    pub target: String,      // field name, URL, message text, section ref
    pub value: String,       // new value for "set", style for "toast"
    pub modifiers: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActionBlock {
    pub event: String,       // "click", "submit", "error", "change"
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
    pub components: Vec<String>,  // referenced component names via `use ComponentName`
    pub requires: Option<String>,  // "auth", "role(admin)", etc.
    pub doc: Option<DocComment>,
}

#[derive(Debug, Clone)]
pub struct StyleNode {
    pub theme: Option<String>,
    pub accent: Option<String>,
    pub radius: Option<String>,
    pub font: Option<String>,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ServiceNode {
    pub name: String,
    pub port: Option<u16>,
    pub config: HashMap<String, String>,
}

/// The 15 official component item types
pub const OFFICIAL_ITEM_TYPES: &[&str] = &[
    "label", "text", "title", "subtitle", "value", "trend", "icon",
    "action", "item", "tab", "plan", "field", "source", "columns", "slot",
    // Also allow common UI types:
    "link", "button", "badge", "cta", "dot", "meta",
];

/// Check if an item type is officially recognized
pub fn is_valid_item_type(t: &str) -> bool {
    OFFICIAL_ITEM_TYPES.contains(&t)
}

#[derive(Debug, Clone)]
pub struct ComponentItemNode {
    pub item_type: String,  // one of OFFICIAL_ITEM_TYPES
    pub text: String,
    pub link: Option<String>,
    pub tone: Option<String>,  // success, danger, primary, secondary, accent, default
    pub config: HashMap<String, String>,  // key:value pairs (price:$29/mo, featured:true, etc)
}

#[derive(Debug, Clone)]
pub struct ComponentParam {
    pub name: String,
    pub param_type: String,  // text, money, integer, boolean, etc. or "any"
    pub default: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct ComponentState {
    pub name: String,
    pub state_type: String,  // integer, text, boolean, etc.
    pub default: String,
}

#[derive(Debug, Clone)]
pub struct ComponentTest {
    pub name: String,        // test description
    pub steps: Vec<String>,  // "fill email \"test@test.com\"", "click \"Login\"", "expect visible \".error\""
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
    pub tests: Vec<ComponentTest>,       // co-located test blocks
}

#[derive(Debug, Clone)]
pub struct ImportNode {
    pub alias: String,
    pub source: String,
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
    pub name: String,
    pub vars: HashMap<String, String>,
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
}

#[derive(Debug, Clone)]
pub struct AuthNode {
    pub entity: String,           // "User"
    pub login_fields: Vec<String>, // ["email", "password"]
    pub session_type: String,      // "jwt"
    pub session_config: HashMap<String, String>, // expires: "24h"
    pub roles: Vec<String>,        // ["admin", "member", "viewer"]
}

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub name: String,
    pub sidebar_items: Vec<LayoutNavItem>,
    pub sidebar_config: HashMap<String, String>,  // brand, etc
    pub topbar_config: HashMap<String, String>,    // search placeholder, etc
}

#[derive(Debug, Clone)]
pub struct LayoutNavItem {
    pub label: String,
    pub route: String,
    pub icon: Option<String>,
    pub requires: Option<String>,  // role requirement
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
