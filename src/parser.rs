#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Language Parser — Rust Native
//!
//! Parses .cronus files into an AST (Vec<AstNode>).
//! Zero external dependencies for parsing.

use std::collections::HashMap;

// ══════════════════════════════════════════════════
// AST TYPES
// ══════════════════════════════════════════════════

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub db_type: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppNode {
    pub name: String,
    pub stack: Vec<String>,
    pub port: u16,
    pub database: Option<DatabaseConfig>,
}

#[derive(Debug, Clone)]
pub struct EntityNode {
    pub name: String,
    pub fields: Vec<FieldNode>,
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
    fn from_str(s: &str) -> Self {
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
    fn from_str(s: &str) -> Option<Self> {
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
}

#[derive(Debug, Clone)]
pub struct ApiNode {
    pub prefix: String,
    pub routes: Vec<RouteNode>,
}

#[derive(Debug, Clone)]
pub struct SectionNode {
    pub section_type: String,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub config: HashMap<String, String>,
    pub items: Vec<HashMap<String, String>>,
    pub plans: Vec<PlanNode>,
}

#[derive(Debug, Clone)]
pub struct PlanNode {
    pub name: String,
    pub price: String,
    pub featured: bool,
    pub features: Vec<String>,
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
pub struct ComponentNode {
    pub name: String,
    pub layout: Option<String>,  // inline, stack, grid, table, hero, modal, sidebar, tabs
    pub style: Option<String>,   // dark+solid+lg, card+metric, etc
    pub items: Vec<ComponentItemNode>,
    pub props: HashMap<String, String>,  // generic key-value props
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

// ══════════════════════════════════════════════════
// TOKENIZER
// ══════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Keyword,
    Identifier,
    StringLit,
    Number,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Arrow,
    ColonPair,
    Plus,
    Comma,
    Price,
    Method,
    Path,
    EnvRef,
    Eof,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    value: String,
    line: usize,
}

const KEYWORDS: &[&str] = &[
    "app", "entity", "api", "page", "style", "service", "section",
    "import", "compose", "use", "merge", "on", "worker", "component",
    "middleware", "env", "test",
];

const METHODS: &[&str] = &["GET", "POST", "PATCH", "PUT", "DELETE"];

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == '.'
}

fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx + 1;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // skip whitespace
            if chars[i] == ' ' || chars[i] == '\t' {
                i += 1;
                continue;
            }

            // comment
            if chars[i] == '#' {
                break;
            }

            // string literal
            if chars[i] == '"' {
                let start = i;
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' { i += 1; }
                    i += 1;
                }
                i += 1; // closing quote
                let raw: String = chars[start..i].iter().collect();
                let value = raw[1..raw.len()-1].to_string();
                tokens.push(Token { kind: TokenKind::StringLit, value, line: line_num });
                continue;
            }

            // braces/brackets/comma/plus
            match chars[i] {
                '{' => { tokens.push(Token { kind: TokenKind::LBrace, value: "{".into(), line: line_num }); i += 1; continue; }
                '}' => { tokens.push(Token { kind: TokenKind::RBrace, value: "}".into(), line: line_num }); i += 1; continue; }
                '[' => { tokens.push(Token { kind: TokenKind::LBracket, value: "[".into(), line: line_num }); i += 1; continue; }
                ']' => { tokens.push(Token { kind: TokenKind::RBracket, value: "]".into(), line: line_num }); i += 1; continue; }
                ',' => { tokens.push(Token { kind: TokenKind::Comma, value: ",".into(), line: line_num }); i += 1; continue; }
                '+' => { tokens.push(Token { kind: TokenKind::Plus, value: "+".into(), line: line_num }); i += 1; continue; }
                _ => {}
            }

            // arrow ->
            if chars[i] == '-' && i + 1 < chars.len() && chars[i + 1] == '>' {
                tokens.push(Token { kind: TokenKind::Arrow, value: "->".into(), line: line_num });
                i += 2;
                continue;
            }

            // price: $29/mo
            if chars[i] == '$' {
                let start = i;
                i += 1;
                while i < chars.len() && chars[i] != ' ' && chars[i] != '\t' && chars[i] != '[' && chars[i] != ']' {
                    i += 1;
                }
                let value: String = chars[start..i].iter().collect();
                tokens.push(Token { kind: TokenKind::Price, value, line: line_num });
                continue;
            }

            // path or word
            if chars[i] == '/' || is_word_char(chars[i]) {
                let start = i;

                // path starts with /
                if chars[i] == '/' {
                    i += 1;
                    while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '{' && chars[i] != '}' && chars[i] != '[' && chars[i] != ']' {
                        i += 1;
                    }
                    let value: String = chars[start..i].iter().collect();
                    tokens.push(Token { kind: TokenKind::Path, value, line: line_num });
                    continue;
                }

                // read full word
                while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '{' && chars[i] != '}' && chars[i] != '[' && chars[i] != ']' && chars[i] != ',' && chars[i] != '+' {
                    if chars[i] == '(' {
                        i += 1;
                        while i < chars.len() && chars[i] != ')' { i += 1; }
                        if i < chars.len() { i += 1; }
                        continue;
                    }
                    if chars[i] == '-' && i + 1 < chars.len() && chars[i + 1] == '>' {
                        break;
                    }
                    i += 1;
                }

                let word: String = chars[start..i].iter().collect();
                if word.is_empty() { i += 1; continue; }

                // classify
                if METHODS.contains(&word.as_str()) {
                    tokens.push(Token { kind: TokenKind::Method, value: word, line: line_num });
                } else if word.starts_with("env(") && word.ends_with(')') {
                    tokens.push(Token { kind: TokenKind::EnvRef, value: word, line: line_num });
                } else if word.contains(':') && !word.starts_with('/') {
                    tokens.push(Token { kind: TokenKind::ColonPair, value: word, line: line_num });
                } else if word.starts_with('/') {
                    tokens.push(Token { kind: TokenKind::Path, value: word, line: line_num });
                } else if word.chars().all(|c| c.is_ascii_digit()) {
                    tokens.push(Token { kind: TokenKind::Number, value: word, line: line_num });
                } else if KEYWORDS.contains(&word.as_str()) {
                    tokens.push(Token { kind: TokenKind::Keyword, value: word, line: line_num });
                } else {
                    tokens.push(Token { kind: TokenKind::Identifier, value: word, line: line_num });
                }
                continue;
            }

            i += 1;
        }
    }

    tokens.push(Token { kind: TokenKind::Eof, value: String::new(), line: lines.len() });
    tokens
}

// ══════════════════════════════════════════════════
// PARSER
// ══════════════════════════════════════════════════

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or(Token { kind: TokenKind::Eof, value: String::new(), line: 0 })
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens.get(self.pos).cloned().unwrap_or(Token { kind: TokenKind::Eof, value: String::new(), line: 0 });
        self.pos += 1;
        t
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, String> {
        let t = self.advance();
        if t.kind != kind {
            return Err(format!("Linha {}: esperava {:?}, encontrou '{}' ({:?})", t.line, kind, t.value, t.kind));
        }
        Ok(t)
    }

    fn matches(&self, kind: TokenKind, value: Option<&str>) -> bool {
        let t = self.peek();
        t.kind == kind && value.map_or(true, |v| t.value.as_str() == v)
    }

    fn try_consume(&mut self, kind: TokenKind, value: Option<&str>) -> Option<Token> {
        if self.matches(kind, value) {
            Some(self.advance())
        } else {
            None
        }
    }

    fn split_colon_pair(pair: &str) -> (String, String) {
        if let Some(idx) = pair.find(':') {
            (pair[..idx].to_string(), pair[idx+1..].to_string())
        } else {
            (pair.to_string(), String::new())
        }
    }

    // ── Main parse ──

    fn parse(&mut self) -> Result<Vec<AstNode>, String> {
        let mut nodes = Vec::new();

        while !self.matches(TokenKind::Eof, None) {
            if self.matches(TokenKind::Keyword, Some("import")) {
                nodes.push(AstNode::Import(self.parse_import()?));
            } else if self.matches(TokenKind::Keyword, Some("compose")) {
                nodes.push(AstNode::Compose(self.parse_compose()?));
            } else if self.matches(TokenKind::Keyword, Some("app")) {
                nodes.push(AstNode::App(self.parse_app()?));
            } else if self.matches(TokenKind::Keyword, Some("entity")) {
                nodes.push(AstNode::Entity(self.parse_entity()?));
            } else if self.matches(TokenKind::Keyword, Some("api")) {
                nodes.push(AstNode::Api(self.parse_api()?));
            } else if self.matches(TokenKind::Keyword, Some("page")) {
                nodes.push(AstNode::Page(self.parse_page()?));
            } else if self.matches(TokenKind::Keyword, Some("style")) {
                nodes.push(AstNode::Style(self.parse_style()?));
            } else if self.matches(TokenKind::Keyword, Some("service")) {
                nodes.push(AstNode::Service(self.parse_service()?));
            } else if self.matches(TokenKind::Keyword, Some("component")) {
                nodes.push(AstNode::Component(self.parse_component()?));
            } else if self.matches(TokenKind::Keyword, Some("on")) {
                nodes.push(AstNode::Event(self.parse_event()?));
            } else if self.matches(TokenKind::Keyword, Some("worker")) {
                nodes.push(AstNode::Worker(self.parse_worker()?));
            } else if self.matches(TokenKind::Keyword, Some("middleware")) {
                nodes.push(AstNode::Middleware(self.parse_middleware()?));
            } else if self.matches(TokenKind::Keyword, Some("env")) {
                nodes.push(AstNode::Env(self.parse_env()?));
            } else if self.matches(TokenKind::Keyword, Some("test")) {
                nodes.push(AstNode::Test(self.parse_test()?));
            } else {
                self.advance();
            }
        }

        Ok(nodes)
    }

    // ── import ──

    fn parse_import(&mut self) -> Result<ImportNode, String> {
        self.expect(TokenKind::Keyword)?;
        let alias = self.advance().value;
        if self.matches(TokenKind::Identifier, Some("from")) { self.advance(); }
        let source = self.expect(TokenKind::StringLit)?.value;
        Ok(ImportNode { alias, source })
    }

    // ── compose ──

    fn parse_compose(&mut self) -> Result<ComposeNode, String> {
        self.expect(TokenKind::Keyword)?;
        let name = self.advance().value;
        self.expect(TokenKind::LBrace)?;

        let mut uses = Vec::new();
        let mut merges = Vec::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.matches(TokenKind::Keyword, Some("use")) {
                self.advance();
                uses.push(self.advance().value);
            } else if self.matches(TokenKind::Keyword, Some("merge")) {
                self.advance();
                let source = self.advance().value;
                let mut config = HashMap::new();
                if self.matches(TokenKind::LBrace, None) {
                    self.advance();
                    while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
                        if self.peek().kind == TokenKind::ColonPair {
                            let (k, v) = Self::split_colon_pair(&self.advance().value);
                            if v.is_empty() && (self.peek().kind == TokenKind::Identifier || self.peek().kind == TokenKind::StringLit) {
                                config.insert(k, self.advance().value);
                            } else {
                                config.insert(k, v);
                            }
                        } else {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                }
                merges.push((source, config));
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(ComposeNode { name, uses, merges })
    }

    // ── app ──

    fn parse_app(&mut self) -> Result<AppNode, String> {
        self.expect(TokenKind::Keyword)?;
        let name = self.expect(TokenKind::StringLit)?.value;
        self.expect(TokenKind::LBrace)?;

        let mut stack = Vec::new();
        let mut port: u16 = 5175;
        let mut database = None;

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.matches(TokenKind::Identifier, Some("stack")) {
                self.advance();
                stack.push(self.advance().value);
                while self.try_consume(TokenKind::Plus, None).is_some() {
                    stack.push(self.advance().value);
                }
            } else if self.matches(TokenKind::Identifier, Some("port")) {
                self.advance();
                port = self.advance().value.parse().unwrap_or(5175);
            } else if self.matches(TokenKind::Identifier, Some("database")) {
                self.advance();
                let db_type = self.advance().value;
                let path = if self.peek().kind == TokenKind::StringLit {
                    Some(self.advance().value)
                } else {
                    None
                };
                database = Some(DatabaseConfig { db_type, path });
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(AppNode { name, stack, port, database })
    }

    // ── entity ──

    fn parse_entity(&mut self) -> Result<EntityNode, String> {
        self.expect(TokenKind::Keyword)?;
        let name = self.advance().value;
        self.expect(TokenKind::LBrace)?;

        let mut fields = Vec::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.peek().kind == TokenKind::Identifier {
                if let Some(field) = self.parse_field()? {
                    fields.push(field);
                }
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(EntityNode { name, fields })
    }

    fn parse_field(&mut self) -> Result<Option<FieldNode>, String> {
        let field_token = self.peek().clone();
        let name = self.advance().value;

        // relation: -> EntityName
        if self.matches(TokenKind::Arrow, None) {
            self.advance();
            let target = self.advance().value;
            return Ok(Some(FieldNode {
                name,
                field_type: FieldType::Relation,
                required: false, unique: false, sensitive: false,
                optional: false, searchable: false, index: false,
                featured: false, formatted: false, array: false,
                enum_values: None,
                reference: Some(target),
            }));
        }

        // type
        let type_str = self.advance().value;
        let mut array = false;

        // Check for type[] (array)
        if self.matches(TokenKind::LBracket, None) {
            if let Some(next) = self.tokens.get(self.pos + 1) {
                if next.kind == TokenKind::RBracket {
                    self.advance(); // [
                    self.advance(); // ]
                    array = true;
                }
            }
        }

        let field_type = FieldType::from_str(&type_str);
        let mut required = false;
        let mut unique = false;
        let mut sensitive = false;
        let mut optional = false;
        let mut searchable = false;
        let mut index = false;
        let mut featured = false;
        let mut formatted = false;
        let mut enum_values = None;

        let field_line = field_token.line;

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.peek().line != field_line { break; }

            if self.matches(TokenKind::LBracket, None) {
                enum_values = Some(self.parse_array()?);
            } else if self.peek().kind == TokenKind::Identifier || self.peek().kind == TokenKind::ColonPair {
                let mod_val = self.advance().value;
                match mod_val.as_str() {
                    "required" => required = true,
                    "unique" => unique = true,
                    "sensitive" => sensitive = true,
                    "optional" => optional = true,
                    "searchable" => searchable = true,
                    "index" => index = true,
                    "featured" => featured = true,
                    "formatted" => formatted = true,
                    _ => {}
                }
            } else {
                break;
            }
        }

        Ok(Some(FieldNode {
            name, field_type, required, unique, sensitive, optional,
            searchable, index, featured, formatted, array,
            enum_values, reference: None,
        }))
    }

    // ── api ──

    fn parse_api(&mut self) -> Result<ApiNode, String> {
        self.expect(TokenKind::Keyword)?;
        let prefix = self.expect(TokenKind::Path)?.value;
        self.expect(TokenKind::LBrace)?;

        let mut routes = Vec::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.peek().kind == TokenKind::Identifier {
                let name = self.advance().value;
                let method_tok = self.expect(TokenKind::Method)?;
                let method = HttpMethod::from_str(&method_tok.value).unwrap_or(HttpMethod::GET);
                let path = self.expect(TokenKind::Path)?.value;

                let mut auth = String::new();
                let mut roles = Vec::new();

                while self.peek().kind == TokenKind::ColonPair {
                    let (k, v) = Self::split_colon_pair(&self.advance().value);
                    if k == "auth" { auth = v; }
                }

                if self.matches(TokenKind::LBracket, None) {
                    roles = self.parse_array()?;
                }

                routes.push(RouteNode { name, method, path, auth, roles });
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(ApiNode { prefix, routes })
    }

    // ── page ──

    fn parse_page(&mut self) -> Result<PageNode, String> {
        self.expect(TokenKind::Keyword)?;
        let route = self.expect(TokenKind::StringLit)?.value;

        let mut page_type = "custom".to_string();
        let mut entity = None;
        let mut inline_config = HashMap::new();

        while self.peek().kind == TokenKind::ColonPair {
            let (k, v) = Self::split_colon_pair(&self.advance().value);
            if k == "type" { page_type = v.clone(); }
            if k == "entity" { entity = Some(v.clone()); }
            inline_config.insert(k, v);
        }

        self.expect(TokenKind::LBrace)?;

        let mut title = None;
        let mut sections = Vec::new();
        let mut config = inline_config;
        let mut components = Vec::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            // `use ComponentName` — reference a defined component
            if self.matches(TokenKind::Keyword, Some("use")) {
                self.advance();
                components.push(self.advance().value);
                continue;
            }
            if self.matches(TokenKind::Identifier, Some("title")) {
                self.advance();
                title = Some(self.expect(TokenKind::StringLit)?.value);
            } else if self.matches(TokenKind::Identifier, Some("columns"))
                   || self.matches(TokenKind::Identifier, Some("stats"))
                   || self.matches(TokenKind::Identifier, Some("actions"))
                   || self.matches(TokenKind::Identifier, Some("fields")) {
                let key = self.advance().value;
                let arr = self.parse_array()?;
                config.insert(key, arr.join(","));
            } else if self.matches(TokenKind::Identifier, Some("search"))
                   || self.matches(TokenKind::Identifier, Some("filters")) {
                let key = self.advance().value;
                if self.matches(TokenKind::LBracket, None) {
                    config.insert(key, self.parse_array()?.join(","));
                } else {
                    config.insert(key, self.advance().value);
                }
            } else if self.matches(TokenKind::Identifier, Some("recent")) {
                self.advance();
                let ent = self.advance().value;
                config.insert("recent".into(), ent);
                if self.peek().kind == TokenKind::ColonPair {
                    let (k, v) = Self::split_colon_pair(&self.advance().value);
                    if k == "limit" { config.insert("recent_limit".into(), v); }
                }
            } else if self.matches(TokenKind::Keyword, Some("section")) {
                sections.push(self.parse_section()?);
            } else if self.peek().kind == TokenKind::ColonPair {
                let (k, v) = Self::split_colon_pair(&self.advance().value);
                config.insert(k, v);
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(PageNode { route, page_type, entity, title, sections, config, components })
    }

    // ── section ──

    fn parse_section(&mut self) -> Result<SectionNode, String> {
        self.expect(TokenKind::Keyword)?;
        let section_type = self.advance().value;

        let mut config = HashMap::new();
        while !self.matches(TokenKind::LBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.peek().kind == TokenKind::ColonPair {
                let (k, v) = Self::split_colon_pair(&self.advance().value);
                config.insert(k, v);
            } else if self.peek().kind == TokenKind::Identifier {
                // Bare identifier before { → boolean flag (e.g. "grid-pattern")
                let flag = self.advance().value;
                config.insert(flag, "true".to_string());
            } else if self.peek().kind == TokenKind::StringLit {
                // String literal as section title shorthand
                config.insert("inline_title".to_string(), self.advance().value);
            } else {
                break;
            }
        }

        self.expect(TokenKind::LBrace)?;

        let mut title = None;
        let mut subtitle = None;
        let mut items = Vec::new();
        let mut plans = Vec::new();

        let mut cta_count = 0;
        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.matches(TokenKind::Identifier, Some("title")) {
                self.advance();
                title = Some(self.expect(TokenKind::StringLit)?.value);
            } else if self.matches(TokenKind::Identifier, Some("subtitle")) {
                self.advance();
                subtitle = Some(self.expect(TokenKind::StringLit)?.value);
            } else if self.matches(TokenKind::Identifier, Some("badge")) {
                self.advance();
                config.insert("badge".into(), self.expect(TokenKind::StringLit)?.value);
            } else if self.matches(TokenKind::Identifier, Some("copyright")) {
                self.advance();
                config.insert("copyright".into(), self.expect(TokenKind::StringLit)?.value);
            } else if self.matches(TokenKind::Identifier, Some("icon")) {
                self.advance();
                config.insert("icon".into(), self.advance().value);
            } else if self.matches(TokenKind::Identifier, Some("nav")) {
                self.advance();
                config.insert("nav".into(), self.expect(TokenKind::StringLit)?.value);
            } else if self.matches(TokenKind::Identifier, Some("brand")) {
                self.advance();
                config.insert("brand".into(), self.expect(TokenKind::StringLit)?.value);
            } else if self.matches(TokenKind::Identifier, Some("footnote")) {
                self.advance();
                config.insert("footnote".into(), self.expect(TokenKind::StringLit)?.value);
            } else if self.matches(TokenKind::Identifier, Some("bullets")) {
                self.advance();
                let arr = self.parse_string_array()?;
                config.insert("bullets".into(), arr.join("||"));
            } else if self.matches(TokenKind::Identifier, Some("columns")) {
                // columns "Col1, Col2, Col3" or columns ["Col1", "Col2"]
                self.advance();
                if self.peek().kind == TokenKind::StringLit {
                    config.insert("columns".into(), self.advance().value);
                } else if self.peek().kind == TokenKind::LBracket {
                    let arr = self.parse_string_array()?;
                    config.insert("columns".into(), arr.join(","));
                } else {
                    config.insert("columns".into(), String::new());
                }
            } else if self.matches(TokenKind::Identifier, Some("display")) {
                self.advance();
                config.insert("display".into(), self.advance().value);
            } else if self.matches(TokenKind::Identifier, Some("metrics")) {
                self.advance();
                config.insert("metrics".into(), self.parse_array()?.join(","));
            } else if self.matches(TokenKind::Keyword, Some("entity")) {
                self.advance();
                config.insert("entity".into(), self.advance().value);
            } else if self.matches(TokenKind::Identifier, Some("cta")) {
                self.advance();
                let text = self.expect(TokenKind::StringLit)?.value;
                let mut link = String::new();
                if self.try_consume(TokenKind::Arrow, None).is_some() {
                    link = self.expect(TokenKind::StringLit)?.value;
                }
                let mut style = String::new();
                // consume trailing modifiers (primary/secondary/pill/ghost/text + icon:x)
                while self.peek().kind == TokenKind::Identifier || self.peek().kind == TokenKind::ColonPair {
                    if self.peek().kind == TokenKind::ColonPair {
                        let (k, v) = Self::split_colon_pair(&self.advance().value);
                        config.insert(format!("cta{}_{}", cta_count, k), v);
                    } else {
                        let val = &self.peek().value;
                        if val == "primary" || val == "secondary" || val == "pill" || val == "ghost" || val == "text" {
                            style = self.advance().value;
                        } else {
                            break;
                        }
                    }
                }
                // Support multiple CTAs: cta_text, cta_link, cta2_text, cta2_link
                if cta_count == 0 {
                    config.insert("cta_text".into(), text);
                    config.insert("cta_link".into(), link);
                    config.insert("cta_style".into(), style);
                } else {
                    config.insert(format!("cta{}_text", cta_count + 1), text);
                    config.insert(format!("cta{}_link", cta_count + 1), link);
                    config.insert(format!("cta{}_style", cta_count + 1), style);
                }
                cta_count += 1;
            } else if self.matches(TokenKind::Identifier, Some("action")) {
                // action "Text" -> "/link" icon:x — stored as item with _type=action
                self.advance();
                let text = self.expect(TokenKind::StringLit)?.value;
                let mut map = HashMap::new();
                map.insert("_type".into(), "action".into());
                map.insert("title".into(), text);
                if self.try_consume(TokenKind::Arrow, None).is_some() {
                    map.insert("link".into(), self.expect(TokenKind::StringLit)?.value);
                }
                while self.peek().kind == TokenKind::ColonPair {
                    let (k, v) = Self::split_colon_pair(&self.advance().value);
                    map.insert(k, v);
                }
                // consume trailing identifiers (style:outline etc)
                while self.peek().kind == TokenKind::Identifier {
                    let val = &self.peek().value;
                    if val == "primary" || val == "secondary" || val == "outline" {
                        map.insert("variant".into(), self.advance().value);
                    } else {
                        break;
                    }
                }
                items.push(map);
            } else if self.matches(TokenKind::Identifier, Some("row")) {
                // row "Label" { key "value"; key "value"; status success }
                self.advance();
                let label = self.expect(TokenKind::StringLit)?.value;
                let mut map = HashMap::new();
                map.insert("_type".into(), "row".into());
                map.insert("title".into(), label);
                // Parse key:value pairs before brace
                while self.peek().kind == TokenKind::ColonPair {
                    let (k, v) = Self::split_colon_pair(&self.advance().value);
                    map.insert(k, v);
                }
                if self.matches(TokenKind::LBrace, None) {
                    self.advance();
                    while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
                        if self.peek().kind == TokenKind::Identifier {
                            let key = self.advance().value;
                            if self.peek().kind == TokenKind::StringLit {
                                map.insert(key, self.advance().value);
                            } else if self.peek().kind == TokenKind::Identifier {
                                map.insert(key, self.advance().value);
                            }
                        } else {
                            self.advance();
                        }
                    }
                    if self.matches(TokenKind::RBrace, None) { self.advance(); }
                }
                items.push(map);
            } else if self.matches(TokenKind::Identifier, Some("policy")) {
                // policy "Name" toggle:on OR policy "Name" value:"X"
                self.advance();
                let pname = self.expect(TokenKind::StringLit)?.value;
                let mut map = HashMap::new();
                map.insert("_type".into(), "policy".into());
                map.insert("title".into(), pname);
                while self.peek().kind == TokenKind::ColonPair {
                    let (k, v) = Self::split_colon_pair(&self.advance().value);
                    map.insert(k, v);
                }
                items.push(map);
            } else if self.matches(TokenKind::Identifier, Some("line"))
                    || self.matches(TokenKind::Identifier, Some("output"))
                    || self.matches(TokenKind::Identifier, Some("success"))
                    || self.matches(TokenKind::Identifier, Some("prompt"))
                    || self.matches(TokenKind::Identifier, Some("chip"))
                    || self.matches(TokenKind::Identifier, Some("code"))
                    || self.matches(TokenKind::Identifier, Some("image"))
                    || self.matches(TokenKind::Identifier, Some("link"))
                    || self.matches(TokenKind::Identifier, Some("meter"))
                    || self.matches(TokenKind::Identifier, Some("label"))
                    || self.matches(TokenKind::Identifier, Some("metric"))
                    || self.matches(TokenKind::Identifier, Some("detail"))
            {
                // Rich content items: line/output/success/prompt/chip/code/image/link/meter/label/metric/detail
                let item_type = self.advance().value;
                let mut map = HashMap::new();
                map.insert("_type".into(), item_type.clone());
                if self.peek().kind == TokenKind::StringLit {
                    map.insert("title".into(), self.advance().value);
                }
                // Arrow for link items: link "Text" -> "/url"
                if self.try_consume(TokenKind::Arrow, None).is_some() {
                    if self.peek().kind == TokenKind::StringLit {
                        map.insert("link".into(), self.advance().value);
                    }
                }
                // Parse trailing key:value pairs
                while self.peek().kind == TokenKind::ColonPair || self.peek().kind == TokenKind::Price {
                    if self.peek().kind == TokenKind::Price {
                        map.insert("price".into(), self.advance().value);
                    } else {
                        let (k, v) = Self::split_colon_pair(&self.advance().value);
                        // If value is empty and next token is StringLit, consume it as the value
                        // This handles src:"url" where tokenizer splits at :
                        if v.is_empty() && self.peek().kind == TokenKind::StringLit {
                            map.insert(k, self.advance().value);
                        } else {
                            map.insert(k, v);
                        }
                    }
                }
                // Parse trailing identifiers as flags
                while self.peek().kind == TokenKind::Identifier {
                    let val = &self.peek().value;
                    if val == "primary" || val == "secondary" || val == "blink" || val == "active" || val == "success" || val == "danger" {
                        map.insert("style".into(), self.advance().value);
                    } else {
                        break;
                    }
                }
                // Optional { "content" } block
                if self.matches(TokenKind::LBrace, None) {
                    self.advance();
                    let mut parts = Vec::new();
                    while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
                        if self.peek().kind == TokenKind::StringLit {
                            parts.push(self.advance().value);
                        } else {
                            self.advance();
                        }
                    }
                    if !parts.is_empty() {
                        map.insert("description".into(), parts.join("\n"));
                    }
                    if self.matches(TokenKind::RBrace, None) { self.advance(); }
                }
                items.push(map);
            } else if self.matches(TokenKind::Identifier, Some("field")) {
                // field "Label" type:email required placeholder:"you@example.com" options:["A","B"]
                self.advance();
                let field_title = self.expect(TokenKind::StringLit)?.value;
                let mut map = HashMap::new();
                map.insert("_type".into(), "field".into());
                map.insert("title".into(), field_title);
                // Parse key:value pairs and bare flags (required, disabled, readonly)
                loop {
                    if self.peek().kind == TokenKind::ColonPair {
                        let (k, v) = Self::split_colon_pair(&self.advance().value);
                        if k == "options" && v.is_empty() && self.matches(TokenKind::LBracket, None) {
                            let arr = self.parse_string_array()?;
                            map.insert(k, arr.join("||"));
                        } else if k == "options" && v.starts_with('[') {
                            // options:["A","B"] already tokenized as single value — strip brackets
                            let clean = v.trim_start_matches('[').trim_end_matches(']');
                            let opts: Vec<&str> = clean.split(',').map(|s| s.trim().trim_matches('"').trim_matches('\'')).collect();
                            map.insert(k, opts.join("||"));
                        } else {
                            map.insert(k, v);
                        }
                    } else if self.peek().kind == TokenKind::Identifier {
                        let val = &self.peek().value;
                        if val == "required" || val == "disabled" || val == "readonly" {
                            let flag = self.advance().value;
                            map.insert(flag, "true".into());
                        } else {
                            break;
                        }
                    } else if self.peek().kind == TokenKind::LBracket {
                        // Bare [...] after options: already handled above, but just in case
                        break;
                    } else {
                        break;
                    }
                }
                // Optional { } block for description/help text
                if self.matches(TokenKind::LBrace, None) {
                    self.advance();
                    let mut parts = Vec::new();
                    while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
                        if self.peek().kind == TokenKind::StringLit {
                            parts.push(self.advance().value);
                        } else {
                            self.advance();
                        }
                    }
                    if !parts.is_empty() {
                        map.insert("description".into(), parts.join("\n"));
                    }
                    if self.matches(TokenKind::RBrace, None) { self.advance(); }
                }
                items.push(map);
            } else if self.matches(TokenKind::Identifier, Some("item")) {
                let item = self.parse_section_item()?;
                items.push(item);
            } else if self.matches(TokenKind::Identifier, Some("plan")) {
                plans.push(self.parse_plan()?);
            } else if self.matches(TokenKind::LBrace, None) {
                // Skip unknown nested blocks
                self.advance();
                let mut depth = 1;
                while depth > 0 && !self.matches(TokenKind::Eof, None) {
                    if self.peek().kind == TokenKind::LBrace { depth += 1; }
                    if self.peek().kind == TokenKind::RBrace { depth -= 1; }
                    if depth > 0 { self.advance(); }
                }
                if self.matches(TokenKind::RBrace, None) { self.advance(); }
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(SectionNode { section_type, title, subtitle, config, items, plans })
    }

    fn parse_section_item(&mut self) -> Result<HashMap<String, String>, String> {
        self.advance(); // consume "item"
        let title = self.expect(TokenKind::StringLit)?.value;
        let mut map = HashMap::new();
        map.insert("title".into(), title);

        // Parse inline attributes: icon:x status:active etc
        while self.peek().kind == TokenKind::ColonPair || self.peek().kind == TokenKind::StringLit || self.peek().kind == TokenKind::Price {
            if self.peek().kind == TokenKind::Price {
                map.insert("price".into(), self.advance().value);
            } else if self.peek().kind == TokenKind::ColonPair {
                let (k, v) = Self::split_colon_pair(&self.advance().value);
                map.insert(k, v);
            } else {
                map.insert("description".into(), self.advance().value);
            }
        }

        // Handle { ... } block — capture descriptions AND sub-items
        if self.matches(TokenKind::LBrace, None) {
            self.advance();
            let mut desc_parts: Vec<String> = Vec::new();
            while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
                if self.peek().kind == TokenKind::StringLit {
                    desc_parts.push(self.advance().value);
                } else if self.peek().kind == TokenKind::Identifier {
                    // Sub-items inside {}: action "text" icon:x, price "$99", etc
                    let key = self.advance().value;
                    if key == "action" || key == "price" || key == "description" || key == "meta" || key == "detail" || key == "footer" || key == "link" {
                        let key_clone = key.clone();
                        if self.peek().kind == TokenKind::StringLit {
                            let val = self.advance().value;
                            map.insert(key, val);
                        } else if self.peek().kind == TokenKind::Price {
                            map.insert(key, self.advance().value);
                        }
                        // consume trailing key:value pairs for this sub-item
                        while self.peek().kind == TokenKind::ColonPair {
                            let (k, v) = Self::split_colon_pair(&self.advance().value);
                            map.insert(format!("{}_{}", key_clone, k), v);
                        }
                    } else if self.peek().kind == TokenKind::StringLit {
                        map.insert(key, self.advance().value);
                    } else if self.peek().kind == TokenKind::ColonPair {
                        let key_clone = key.clone();
                        let (k, v) = Self::split_colon_pair(&self.advance().value);
                        map.insert(format!("{}_{}", key_clone, k), v);
                    }
                } else if self.peek().kind == TokenKind::ColonPair {
                    let (k, v) = Self::split_colon_pair(&self.advance().value);
                    map.insert(k, v);
                } else if self.peek().kind == TokenKind::LBrace {
                    // Skip nested sub-blocks
                    self.advance();
                    let mut depth = 1;
                    while depth > 0 && !self.matches(TokenKind::Eof, None) {
                        if self.peek().kind == TokenKind::LBrace { depth += 1; }
                        if self.peek().kind == TokenKind::RBrace { depth -= 1; }
                        if depth > 0 { self.advance(); }
                    }
                    if self.matches(TokenKind::RBrace, None) { self.advance(); }
                } else {
                    self.advance();
                }
            }
            if !desc_parts.is_empty() {
                map.insert("description".into(), desc_parts.join("\n"));
            }
            self.expect(TokenKind::RBrace)?;
        }

        Ok(map)
    }

    fn parse_plan(&mut self) -> Result<PlanNode, String> {
        self.advance(); // consume "plan"
        let name = self.expect(TokenKind::StringLit)?.value;
        let price = self.expect(TokenKind::Price)?.value;

        let featured = self.try_consume(TokenKind::Identifier, Some("featured")).is_some();

        let features = if self.matches(TokenKind::LBracket, None) {
            self.parse_string_array()?
        } else {
            Vec::new()
        };

        Ok(PlanNode { name, price, featured, features })
    }

    // ── style ──

    fn parse_style(&mut self) -> Result<StyleNode, String> {
        self.expect(TokenKind::Keyword)?;
        self.expect(TokenKind::LBrace)?;

        let mut theme = None;
        let mut accent = None;
        let mut radius = None;
        let mut font = None;
        let mut config = HashMap::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.matches(TokenKind::Identifier, Some("theme")) { self.advance(); theme = Some(self.advance().value); }
            else if self.matches(TokenKind::Identifier, Some("accent")) { self.advance(); accent = Some(self.advance().value); }
            else if self.matches(TokenKind::Identifier, Some("radius")) { self.advance(); radius = Some(self.advance().value); }
            else if self.matches(TokenKind::Identifier, Some("font")) { self.advance(); font = Some(self.advance().value); }
            else if self.peek().kind == TokenKind::Identifier {
                let key = self.advance().value;
                let val = self.advance().value;
                config.insert(key, val);
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(StyleNode { theme, accent, radius, font, config })
    }

    // ── service ──

    fn parse_service(&mut self) -> Result<ServiceNode, String> {
        self.expect(TokenKind::Keyword)?;
        let name = self.advance().value;

        let mut port = None;
        let mut config = HashMap::new();

        while self.peek().kind == TokenKind::ColonPair {
            let (k, v) = Self::split_colon_pair(&self.advance().value);
            if k == "port" { port = v.parse().ok(); }
            else { config.insert(k, v); }
        }

        self.expect(TokenKind::LBrace)?;

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.peek().kind == TokenKind::Identifier {
                let key = self.advance().value;
                let key_line = self.tokens.get(self.pos - 1).map(|t| t.line).unwrap_or(0);

                while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
                    let next_line = self.peek().line;
                    if next_line != key_line && self.peek().kind == TokenKind::Identifier { break; }

                    if self.peek().kind == TokenKind::ColonPair {
                        let (k, v) = Self::split_colon_pair(&self.advance().value);
                        config.insert(format!("{}.{}", key, k), v);
                    } else if self.peek().kind == TokenKind::EnvRef {
                        config.insert(key.clone(), self.advance().value);
                        break;
                    } else if self.peek().kind == TokenKind::LBracket {
                        let arr = self.parse_string_array()?;
                        config.insert(key.clone(), arr.join(","));
                        break;
                    } else if matches!(self.peek().kind, TokenKind::Identifier | TokenKind::StringLit | TokenKind::Number | TokenKind::Path) {
                        config.insert(key.clone(), self.advance().value);
                        break;
                    } else {
                        break;
                    }
                }
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(ServiceNode { name, port, config })
    }

    // ── component ──

    fn parse_component(&mut self) -> Result<ComponentNode, String> {
        self.expect(TokenKind::Keyword)?;
        let name = self.advance().value;

        // Parse attributes before the opening brace: layout:inline style:topbar+light
        let mut layout = None;
        let mut style = None;
        let mut items = Vec::new();
        let mut props = HashMap::new();

        while !self.matches(TokenKind::LBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.peek().kind == TokenKind::ColonPair {
                let (k, v) = Self::split_colon_pair(&self.advance().value);
                if k == "layout" {
                    let mut val = v;
                    if val.is_empty() && (self.peek().kind == TokenKind::Identifier || self.peek().kind == TokenKind::StringLit) {
                        val = self.advance().value;
                    }
                    layout = Some(val);
                } else if k == "style" {
                    let mut parts = vec![if v.is_empty() { self.advance().value } else { v }];
                    while self.try_consume(TokenKind::Plus, None).is_some() {
                        parts.push(self.advance().value);
                    }
                    style = Some(parts.join("+"));
                } else {
                    if v.is_empty() && (self.peek().kind == TokenKind::Identifier || self.peek().kind == TokenKind::StringLit) {
                        props.insert(k, self.advance().value);
                    } else {
                        props.insert(k, v);
                    }
                }
            } else if self.peek().kind == TokenKind::Identifier {
                // Handle bare identifiers before brace
                let ident = self.advance().value;
                if self.peek().kind == TokenKind::StringLit || self.peek().kind == TokenKind::Identifier {
                    props.insert(ident, self.advance().value);
                }
            } else {
                break;
            }
        }

        self.expect(TokenKind::LBrace)?;

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.matches(TokenKind::Identifier, Some("layout")) {
                self.advance();
                layout = Some(self.advance().value);
            } else if self.matches(TokenKind::Identifier, Some("style")) || self.matches(TokenKind::Keyword, Some("style")) {
                self.advance();
                let mut parts = vec![self.advance().value];
                while self.try_consume(TokenKind::Plus, None).is_some() {
                    parts.push(self.advance().value);
                }
                style = Some(parts.join("+"));
            } else if self.matches(TokenKind::Identifier, Some("items")) {
                self.advance();
                self.expect(TokenKind::LBracket)?;
                while !self.matches(TokenKind::RBracket, None) && !self.matches(TokenKind::Eof, None) {
                    if self.peek().kind == TokenKind::Identifier || self.peek().kind == TokenKind::Keyword {
                        let item_type = self.advance().value;
                        let text = if self.peek().kind == TokenKind::StringLit { self.advance().value } else { String::new() };
                        let link = if self.try_consume(TokenKind::Arrow, None).is_some() {
                            Some(if self.peek().kind == TokenKind::StringLit { self.advance().value } else { self.advance().value })
                        } else {
                            None
                        };
                        let mut item_config = HashMap::new();
                        let mut tone = None;
                        // Parse key:value pairs including tone and price
                        while self.peek().kind == TokenKind::ColonPair || self.peek().kind == TokenKind::Price {
                            if self.peek().kind == TokenKind::Price {
                                item_config.insert("price".to_string(), self.advance().value);
                                continue;
                            }
                            let (k, v) = Self::split_colon_pair(&self.advance().value);
                            if k == "tone" {
                                tone = Some(if v.is_empty() && self.peek().kind == TokenKind::Identifier { self.advance().value } else { v });
                            } else if v.is_empty() && (self.peek().kind == TokenKind::Identifier || self.peek().kind == TokenKind::StringLit) {
                                item_config.insert(k, self.advance().value);
                            } else {
                                item_config.insert(k, v);
                            }
                        }
                        items.push(ComponentItemNode { item_type, text, link, tone, config: item_config });
                    } else {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBracket)?;
            } else if self.peek().kind == TokenKind::ColonPair {
                // Generic props: key:value at component level
                let (k, v) = Self::split_colon_pair(&self.advance().value);
                if v.is_empty() && (self.peek().kind == TokenKind::Identifier || self.peek().kind == TokenKind::StringLit) {
                    props.insert(k, self.advance().value);
                } else {
                    props.insert(k, v);
                }
            } else if self.peek().kind == TokenKind::Identifier && is_valid_item_type(&self.peek().value) {
                // Recognized item type: parse as ComponentItemNode
                let item_type = self.advance().value;
                let text = if self.peek().kind == TokenKind::StringLit { self.advance().value } else { String::new() };
                let link = if self.try_consume(TokenKind::Arrow, None).is_some() {
                    Some(if self.peek().kind == TokenKind::StringLit { self.advance().value } else { self.advance().value })
                } else {
                    None
                };
                let mut item_config = HashMap::new();
                let mut tone = None;
                while self.peek().kind == TokenKind::ColonPair || self.peek().kind == TokenKind::Price {
                    if self.peek().kind == TokenKind::Price {
                        item_config.insert("price".to_string(), self.advance().value);
                        continue;
                    }
                    let (k, v) = Self::split_colon_pair(&self.advance().value);
                    if k == "tone" {
                        tone = Some(if v.is_empty() && self.peek().kind == TokenKind::Identifier { self.advance().value } else { v });
                    } else if v.is_empty() && (self.peek().kind == TokenKind::Identifier || self.peek().kind == TokenKind::StringLit) {
                        item_config.insert(k, self.advance().value);
                    } else {
                        item_config.insert(k, v);
                    }
                }
                // Also store single-value item types (brand, subtitle, title) as props for easy access
                if (item_type == "brand" || item_type == "subtitle" || item_type == "title") && !text.is_empty() {
                    props.entry(item_type.clone()).or_insert_with(|| text.clone());
                }
                items.push(ComponentItemNode { item_type, text, link, tone, config: item_config });
            } else if self.peek().kind == TokenKind::Identifier {
                // Unknown identifier props: key value
                let key = self.advance().value;
                if self.peek().kind == TokenKind::StringLit || self.peek().kind == TokenKind::Number || self.peek().kind == TokenKind::Identifier {
                    props.insert(key, self.advance().value);
                }
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(ComponentNode { name, layout, style, items, props })
    }

    // ── event ──

    fn parse_event(&mut self) -> Result<EventNode, String> {
        self.expect(TokenKind::Keyword)?;
        let name = self.advance().value;
        self.expect(TokenKind::LBrace)?;

        let mut actions = Vec::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            let mut parts = Vec::new();
            let action_line = self.peek().line;
            while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) && self.peek().line == action_line {
                parts.push(self.advance().value);
            }
            if !parts.is_empty() {
                actions.push(parts.join(" "));
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(EventNode { name, actions })
    }

    // ── worker ──

    fn parse_worker(&mut self) -> Result<WorkerNode, String> {
        self.expect(TokenKind::Keyword)?;
        let name = self.advance().value;

        let mut queue = None;
        while self.peek().kind == TokenKind::ColonPair {
            let (k, v) = Self::split_colon_pair(&self.advance().value);
            if k == "queue" { queue = Some(v); }
        }

        self.expect(TokenKind::LBrace)?;

        let mut concurrency = None;
        let mut retry = None;
        let mut timeout = None;
        let mut entity = None;

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.matches(TokenKind::Identifier, Some("concurrency")) { self.advance(); concurrency = self.advance().value.parse().ok(); }
            else if self.matches(TokenKind::Identifier, Some("retry")) { self.advance(); retry = self.advance().value.parse().ok(); }
            else if self.matches(TokenKind::Identifier, Some("timeout")) { self.advance(); timeout = Some(self.advance().value); }
            else if self.matches(TokenKind::Identifier, Some("process")) { self.advance(); entity = Some(self.advance().value); }
            else { self.advance(); }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(WorkerNode { name, queue, concurrency, retry, timeout, entity })
    }

    // ── middleware ──

    fn parse_middleware(&mut self) -> Result<MiddlewareNode, String> {
        self.expect(TokenKind::Keyword)?;
        let name = self.advance().value;
        self.expect(TokenKind::LBrace)?;

        let mut applies_to = None;
        let mut config = HashMap::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.matches(TokenKind::Identifier, Some("applies_to")) {
                self.advance();
                applies_to = Some(self.parse_string_array()?);
            } else if self.peek().kind == TokenKind::Identifier {
                let key = self.advance().value;
                let key_line = self.tokens.get(self.pos - 1).map(|t| t.line).unwrap_or(0);
                let mut parts = Vec::new();
                while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) && self.peek().line == key_line {
                    parts.push(self.advance().value);
                }
                if !parts.is_empty() { config.insert(key, parts.join(" ")); }
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(MiddlewareNode { name, applies_to, config })
    }

    // ── env ──

    fn parse_env(&mut self) -> Result<EnvNode, String> {
        self.expect(TokenKind::Keyword)?;
        let name = self.advance().value;
        self.expect(TokenKind::LBrace)?;

        let mut vars = HashMap::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.peek().kind == TokenKind::Identifier {
                let key = self.advance().value;
                let val = self.advance().value;
                vars.insert(key, val);
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(EnvNode { name, vars })
    }

    // ── test ──

    fn parse_test(&mut self) -> Result<TestNode, String> {
        self.expect(TokenKind::Keyword)?;
        let name = self.expect(TokenKind::StringLit)?.value;
        self.expect(TokenKind::LBrace)?;

        let mut steps = Vec::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.peek().kind == TokenKind::Identifier {
                let action = self.advance().value;
                let entity = self.advance().value;

                let mut body = HashMap::new();
                if self.matches(TokenKind::LBrace, None) {
                    self.advance();
                    while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
                        if self.peek().kind == TokenKind::ColonPair {
                            let (k, v) = Self::split_colon_pair(&self.advance().value);
                            if v.is_empty() && (self.peek().kind == TokenKind::StringLit || self.peek().kind == TokenKind::Identifier) {
                                body.insert(k, self.advance().value);
                            } else {
                                body.insert(k, v.trim_matches('"').to_string());
                            }
                        } else if self.peek().kind == TokenKind::Comma {
                            self.advance();
                        } else {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                }

                let mut expect_code: u16 = 200;
                let mut expect_config = HashMap::new();

                if self.try_consume(TokenKind::Arrow, None).is_some() {
                    if self.matches(TokenKind::Identifier, Some("expect")) {
                        self.advance();
                        expect_code = self.advance().value.parse().unwrap_or(200);
                        while self.peek().kind == TokenKind::ColonPair {
                            let (k, v) = Self::split_colon_pair(&self.advance().value);
                            expect_config.insert(k, v);
                        }
                    }
                }

                steps.push(TestStepNode { action, entity, body, expect: expect_code, expect_config });
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(TestNode { name, steps })
    }

    // ── helpers ──

    fn parse_array(&mut self) -> Result<Vec<String>, String> {
        self.expect(TokenKind::LBracket)?;
        let mut items = Vec::new();

        while !self.matches(TokenKind::RBracket, None) && !self.matches(TokenKind::Eof, None) {
            if self.peek().kind == TokenKind::Comma { self.advance(); continue; }
            items.push(self.advance().value);
        }

        self.expect(TokenKind::RBracket)?;
        Ok(items)
    }

    fn parse_string_array(&mut self) -> Result<Vec<String>, String> {
        self.expect(TokenKind::LBracket)?;
        let mut items = Vec::new();

        while !self.matches(TokenKind::RBracket, None) && !self.matches(TokenKind::Eof, None) {
            if self.peek().kind == TokenKind::Comma { self.advance(); continue; }
            items.push(self.advance().value);
        }

        self.expect(TokenKind::RBracket)?;
        Ok(items)
    }
}

// ══════════════════════════════════════════════════
// PUBLIC API
// ══════════════════════════════════════════════════

/// Parse a .cronus source string into an AST.
pub fn parse(source: &str) -> Result<Vec<AstNode>, String> {
    let tokens = tokenize(source);
    let mut parser = Parser::new(tokens);
    parser.parse()
}

/// Parse a .cronus file with import resolution.
/// Reads imported files relative to `base_dir` and merges all nodes.
/// Each import is resolved exactly once (no cycles).
/// This is the key feature for multi-agent collaboration:
/// each agent owns a file, the kernel composes them all.
pub fn parse_with_imports(source: &str, base_dir: &str) -> Result<Vec<AstNode>, String> {
    use std::collections::HashSet;
    use std::path::Path;

    let mut all_nodes = Vec::new();
    let mut resolved: HashSet<String> = HashSet::new();
    let mut queue: Vec<(String, String)> = vec![("main".into(), source.to_string())];

    while let Some((name, src)) = queue.pop() {
        if resolved.contains(&name) {
            continue;
        }
        resolved.insert(name.clone());

        let nodes = parse(&src)?;

        for node in &nodes {
            if let AstNode::Import(imp) = node {
                let import_path = if imp.source.starts_with('/') {
                    imp.source.clone()
                } else {
                    format!("{}/{}", base_dir, imp.source)
                };

                // Resolve .cronus extension
                let full_path = if import_path.ends_with(".cronus") {
                    import_path
                } else {
                    format!("{}.cronus", import_path)
                };

                if !resolved.contains(&full_path) {
                    match std::fs::read_to_string(&full_path) {
                        Ok(content) => {
                            queue.push((full_path, content));
                        }
                        Err(e) => {
                            eprintln!("  [warning] import '{}' not found: {}", imp.source, e);
                        }
                    }
                }
            }
        }

        // Add all non-import nodes
        for node in nodes {
            if !matches!(node, AstNode::Import(_)) {
                all_nodes.push(node);
            }
        }
    }

    // Deduplicate: if two files define same entity name, last wins
    // But for API routes, pages, services — all are kept (additive)
    let mut seen_entities: HashSet<String> = HashSet::new();
    let mut seen_app = false;
    let mut seen_style = false;
    let mut deduped = Vec::new();

    // Process in reverse so last definition wins for entities
    for node in all_nodes.into_iter().rev() {
        match &node {
            AstNode::Entity(e) => {
                if seen_entities.contains(&e.name) {
                    continue; // skip duplicate entity
                }
                seen_entities.insert(e.name.clone());
                deduped.push(node);
            }
            AstNode::App(_) => {
                if seen_app { continue; }
                seen_app = true;
                deduped.push(node);
            }
            AstNode::Style(_) => {
                if seen_style { continue; }
                seen_style = true;
                deduped.push(node);
            }
            _ => deduped.push(node), // API, Page, Service, etc are additive
        }
    }

    deduped.reverse(); // Restore original order
    Ok(deduped)
}

/// Parse all .cronus files in a directory and merge them.
/// This is the multi-agent mode: each agent writes its own file,
/// the kernel composes everything automatically.
pub fn parse_directory(dir: &str) -> Result<Vec<AstNode>, String> {
    let mut files: Vec<String> = Vec::new();

    // Find all .cronus files
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".cronus") {
                files.push(entry.path().to_string_lossy().to_string());
            }
        }
    }

    files.sort(); // Deterministic order

    if files.is_empty() {
        return Err("No .cronus files found in directory".into());
    }

    // Concatenate all files with comments showing source
    let mut combined = String::new();
    for file in &files {
        let content = std::fs::read_to_string(file)
            .map_err(|e| format!("Error reading {}: {}", file, e))?;
        combined.push_str(&format!("# [source: {}]\n", file));
        combined.push_str(&content);
        combined.push('\n');
    }

    parse_with_imports(&combined, dir)
}

/// Count entities, pages, api routes for quick stats.
pub fn stats(nodes: &[AstNode]) -> (usize, usize, usize) {
    let entities = nodes.iter().filter(|n| matches!(n, AstNode::Entity(_))).count();
    let pages = nodes.iter().filter(|n| matches!(n, AstNode::Page(_))).count();
    let api_routes: usize = nodes.iter().filter_map(|n| {
        if let AstNode::Api(api) = n { Some(api.routes.len()) } else { None }
    }).sum();
    (entities, pages, api_routes)
}

/// Generate a complete SaaS .cronus file with component primitives
pub fn generate_saas() -> String {
    r#"# Generated by `cronus generate saas`

app "MyApp" {
  stack react + tailwind
  port 4800
  database sqlite "./data.db"
  theme dark
}

style {
  theme dark
  accent indigo
  background neutral-950
  font "Inter"
}

# ── Entities ──

entity User {
  name        string    required
  email       email     required unique
  role        enum      [admin, member, viewer]
  avatarUrl   url       optional
  createdAt   date
}

entity Project {
  title       string    required
  description text
  owner       -> User
  status      enum      [active, archived, draft]
  createdAt   date
}

entity Task {
  title       string    required
  project     -> Project
  assignee    -> User
  status      enum      [todo, in_progress, done, blocked]
  priority    enum      [low, medium, high, urgent]
  dueDate     date      optional
  createdAt   date
}

# ── Components ──

component Hero {
  layout stack
  items [
    title "Welcome to MyApp"
    subtitle "Manage your projects and tasks effortlessly"
    action "Get Started" -> "/projects" tone:primary
  ]
}

component Sidebar {
  layout stack
  items [
    link "Dashboard" -> "/" icon:grid
    link "Projects" -> "/projects" icon:folder
    link "Tasks" -> "/tasks" icon:check
    link "Settings" -> "/settings" icon:gear
  ]
}

component CardMetric {
  layout inline
  style card+metric
  items [
    label "Total Projects"
    value "$count"
    trend "+12%" tone:success
  ]
}

component Table {
  layout table
  items [
    columns "Name, Status, Priority, Assignee"
    source "Task"
  ]
}

component EmptyState {
  layout stack
  items [
    icon "inbox"
    title "No items yet"
    subtitle "Create your first item to get started"
    action "Create" -> "/new" tone:primary
  ]
}

# ── Pages ──

page "/" type:custom {
  title "Dashboard"
  use Hero
  use CardMetric
}

page "/projects" type:list entity:Project {
  title "Projects"
  columns [title, status, createdAt]
  search [title, description]
  actions [create, edit, delete]
  filters [status]
  use Sidebar
  use EmptyState
}

page "/tasks" type:list entity:Task {
  title "Tasks"
  columns [title, status, priority, dueDate]
  search [title]
  actions [create, edit, delete]
  filters [status, priority]
  use Sidebar
  use EmptyState
}

page "/settings" type:form entity:User {
  title "Settings"
  fields [name, email, avatarUrl]
}

page "/login" type:form entity:User {
  title "Sign In"
  fields [email]
}

# ── API Routes ──

api /auth {
  login     POST   /login     auth:public
  register  POST   /register  auth:public
  me        GET    /me        auth:jwt
}

api /users {
  list    GET    /        auth:jwt
  detail  GET    /:id     auth:jwt
  update  PATCH  /:id     auth:jwt
}

api /projects {
  list    GET    /        auth:jwt
  create  POST   /        auth:jwt
  detail  GET    /:id     auth:jwt
  update  PATCH  /:id     auth:jwt
  delete  DELETE /:id     auth:jwt
}

api /tasks {
  list    GET    /        auth:jwt
  create  POST   /        auth:jwt
  detail  GET    /:id     auth:jwt
  update  PATCH  /:id     auth:jwt
  delete  DELETE /:id     auth:jwt
}

# ── Events ──

on task.created {
  notify assignee "new-task"
}

on project.archived {
  send email "project-archived"
}

# ── Environments ──

env development {
  DATABASE_URL "sqlite:./dev.db"
  JWT_SECRET "dev-secret"
}

env production {
  DATABASE_URL env(DATABASE_URL)
  JWT_SECRET env(JWT_SECRET)
}
"#.to_string()
}
