#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Language Parser — Rust Native
//!
//! Parses .cronus files into an AST (Vec<AstNode>).
//! Zero external dependencies for parsing.

pub mod ast;
pub use ast::*;

pub(crate) mod tokenizer;
pub(crate) use tokenizer::*;

use std::collections::HashMap;

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

    /// Consume all consecutive DocComment tokens and build a DocComment struct.
    fn collect_doc_comments(&mut self) -> Option<DocComment> {
        let mut lines: Vec<String> = Vec::new();
        while self.peek().kind == TokenKind::DocComment {
            lines.push(self.advance().value);
        }
        if lines.is_empty() {
            return None;
        }

        let mut summary = String::new();
        let mut desc_lines: Vec<String> = Vec::new();
        let mut tags: Vec<DocTag> = Vec::new();

        for line in &lines {
            if line.starts_with('@') {
                // Parse tag: @name value
                let mut parts = line[1..].splitn(2, ' ');
                let name = parts.next().unwrap_or("").to_string();
                let value = parts.next().unwrap_or("").to_string();
                tags.push(DocTag { name, value });
            } else if summary.is_empty() {
                summary = line.clone();
            } else {
                desc_lines.push(line.clone());
            }
        }

        Some(DocComment {
            summary,
            description: desc_lines.join("\n"),
            tags,
        })
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or(Token {
            kind: TokenKind::Eof,
            value: String::new(),
            line: 0,
        })
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens.get(self.pos).cloned().unwrap_or(Token {
            kind: TokenKind::Eof,
            value: String::new(),
            line: 0,
        });
        self.pos += 1;
        t
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, String> {
        let t = self.advance();
        if t.kind != kind {
            return Err(format!(
                "Linha {}: esperava {:?}, encontrou '{}' ({:?})",
                t.line, kind, t.value, t.kind
            ));
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
            let key = pair[..idx].to_string();
            let raw_val = &pair[idx + 1..];
            // Strip surrounding quotes from the value (e.g. value:"12,842" → 12,842)
            let val = if raw_val.starts_with('"') && raw_val.ends_with('"') && raw_val.len() >= 2 {
                raw_val[1..raw_val.len() - 1].to_string()
            } else {
                raw_val.to_string()
            };
            (key, val)
        } else {
            (pair.to_string(), String::new())
        }
    }

    /// `where` value written as its own token (`eq auth.id`, `eq "x"`, `eq 5`).
    fn filter_value(token: &Token) -> BindingValue {
        if token.value.starts_with("auth.") || token.value.starts_with("route.") {
            BindingValue::AuthRef(token.value.clone())
        } else if token.kind == TokenKind::StringLit {
            BindingValue::Str(token.value.clone())
        } else if token.kind == TokenKind::Number {
            BindingValue::Num(token.value.clone())
        } else if token.value == "true" || token.value == "false" {
            BindingValue::Bool(token.value == "true")
        } else {
            BindingValue::Str(token.value.clone())
        }
    }

    /// `where` value glued to its operator (`eq:auth.id`, `eq:"x"`, `gt:5`).
    /// Classified exactly like the space-separated form.
    fn filter_value_from_colon(raw: &str) -> BindingValue {
        let quoted = raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"');
        if quoted {
            return BindingValue::Str(raw[1..raw.len() - 1].to_string());
        }
        if raw.starts_with("auth.") || raw.starts_with("route.") {
            BindingValue::AuthRef(raw.to_string())
        } else if !raw.is_empty() && raw.chars().all(|c| c.is_ascii_digit()) {
            BindingValue::Num(raw.to_string())
        } else if raw == "true" || raw == "false" {
            BindingValue::Bool(raw == "true")
        } else {
            BindingValue::Str(raw.to_string())
        }
    }

    // ── Main parse ──

    fn parse(&mut self) -> Result<Vec<AstNode>, String> {
        let mut nodes = Vec::new();

        while !self.matches(TokenKind::Eof, None) {
            // Collect doc-comments before each block
            let pending_doc = self.collect_doc_comments();

            if self.matches(TokenKind::Keyword, Some("import")) {
                nodes.push(AstNode::Import(self.parse_import()?));
            } else if self.matches(TokenKind::Keyword, Some("compose")) {
                nodes.push(AstNode::Compose(self.parse_compose()?));
            } else if self.matches(TokenKind::Keyword, Some("app")) {
                let mut a = self.parse_app()?;
                a.doc = pending_doc.clone();
                nodes.push(AstNode::App(a));
            } else if self.matches(TokenKind::Keyword, Some("entity")) {
                let mut e = self.parse_entity()?;
                e.doc = pending_doc.clone();
                nodes.push(AstNode::Entity(e));
            } else if self.matches(TokenKind::Keyword, Some("api")) {
                let mut a = self.parse_api()?;
                a.doc = pending_doc.clone();
                nodes.push(AstNode::Api(a));
            } else if self.matches(TokenKind::Keyword, Some("webhook")) {
                nodes.push(AstNode::Webhook(self.parse_webhook()?));
            } else if self.matches(TokenKind::Keyword, Some("page")) {
                let mut p = self.parse_page()?;
                p.doc = pending_doc.clone();
                nodes.push(AstNode::Page(p));
            } else if self.matches(TokenKind::Keyword, Some("style")) {
                nodes.push(AstNode::Style(self.parse_style()?));
            } else if self.matches(TokenKind::Keyword, Some("service")) {
                nodes.push(AstNode::Service(self.parse_service()?));
            } else if self.matches(TokenKind::Identifier, Some("define")) {
                nodes.push(AstNode::Define(self.parse_define()?));
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
            } else if self.matches(TokenKind::Keyword, Some("deploy")) {
                nodes.push(AstNode::Deploy(self.parse_deploy()?));
            } else if self.matches(TokenKind::Identifier, Some("auth")) {
                nodes.push(AstNode::Auth(self.parse_auth()?));
            } else if self.matches(TokenKind::Identifier, Some("layout")) {
                self.advance();
                nodes.push(AstNode::Layout(self.parse_layout()?));
            } else if self.matches(TokenKind::Identifier, Some("tailwind_config")) {
                self.advance();
                let config_js = self.expect(TokenKind::StringLit)?.value;
                // Attach to the most recent App node
                for node in nodes.iter_mut().rev() {
                    if let AstNode::App(ref mut app) = node {
                        app.tailwind_config = Some(config_js.clone());
                        break;
                    }
                }
            } else {
                let unknown = self.peek();
                if !unknown.value.is_empty() && unknown.kind != TokenKind::Eof {
                    eprintln!(
                        "  \x1b[33m⚠\x1b[0m Line {}: unknown top-level token '{}' (skipped)",
                        unknown.line, unknown.value
                    );
                }
                self.advance();
            }
        }

        Ok(nodes)
    }

    // ── import ──

    fn parse_import(&mut self) -> Result<ImportNode, String> {
        self.expect(TokenKind::Keyword)?;
        let alias = self.advance().value;
        if self.matches(TokenKind::Identifier, Some("from")) {
            self.advance();
        }
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
                    while !self.matches(TokenKind::RBrace, None)
                        && !self.matches(TokenKind::Eof, None)
                    {
                        if self.peek().kind == TokenKind::ColonPair {
                            let (k, v) = Self::split_colon_pair(&self.advance().value);
                            if v.is_empty()
                                && (self.peek().kind == TokenKind::Identifier
                                    || self.peek().kind == TokenKind::StringLit)
                            {
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
        let mut constitution = None;

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
            } else if self.matches(TokenKind::Keyword, Some("constitution")) {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut must_rules = Vec::new();
                let mut never_rules = Vec::new();
                while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None)
                {
                    if self.matches(TokenKind::Keyword, Some("must")) {
                        self.advance();
                        must_rules.push(self.expect(TokenKind::StringLit)?.value);
                    } else if self.matches(TokenKind::Keyword, Some("never")) {
                        self.advance();
                        never_rules.push(self.expect(TokenKind::StringLit)?.value);
                    } else {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBrace)?;
                constitution = Some(ConstitutionNode {
                    must: must_rules,
                    never: never_rules,
                });
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(AppNode {
            name,
            stack,
            port,
            database,
            tailwind_config: None,
            constitution,
            doc: None,
        })
    }

    // ── entity ──

    fn parse_entity(&mut self) -> Result<EntityNode, String> {
        self.expect(TokenKind::Keyword)?;
        let name_token = self.peek().clone();
        let name = self.advance().value;

        // P040/P041: validate entity name
        Self::validate_identifier(&name, "entity name", name_token.line)?;

        // Check for "shared" modifier before the brace
        let mut shared = false;
        if self.matches(TokenKind::Identifier, Some("shared")) {
            self.advance();
            shared = true;
        }

        self.expect(TokenKind::LBrace)?;

        let mut fields = Vec::new();
        let mut transitions = Vec::new();
        let mut effects = Vec::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            // Collect doc-comments for the next field
            let field_doc = self.collect_doc_comments();

            // Check for transition block
            if self.matches(TokenKind::Keyword, Some("transition")) {
                let transition = self.parse_transition(&fields)?;
                transitions.push(transition);
                continue;
            }

            // Check for effect block: on create/update/delete { ... }
            if self.matches(TokenKind::Keyword, Some("on"))
                || self.matches(TokenKind::Identifier, Some("on"))
            {
                let effect = self.parse_effect_block()?;
                effects.push(effect);
                continue;
            }

            let pk = self.peek().kind;
            if pk == TokenKind::Identifier || pk == TokenKind::Keyword {
                if let Some(mut field) = self.parse_field()? {
                    field.doc = field_doc;
                    fields.push(field);
                }
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(EntityNode {
            name,
            fields,
            transitions,
            effects,
            shared,
            remote_url: None,
            doc: None,
        })
    }

    fn parse_field(&mut self) -> Result<Option<FieldNode>, String> {
        let field_token = self.peek().clone();
        let name = self.advance().value;

        // P040/P041: validate field name
        Self::validate_identifier(&name, "entity field", field_token.line)?;

        // relation: -> EntityName
        if self.matches(TokenKind::Arrow, None) {
            self.advance();
            let target = self.advance().value;
            return Ok(Some(FieldNode {
                name,
                field_type: FieldType::Relation,
                required: false,
                unique: false,
                sensitive: false,
                optional: false,
                searchable: false,
                index: false,
                featured: false,
                formatted: false,
                array: false,
                enum_values: None,
                reference: Some(target),
                doc: None,
                default_value: None,
                min: None,
                max: None,
                min_length: None,
                max_length: None,
                pattern: None,
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

        // Handle ! suffix on type (e.g. "string!" → type="string", required=true)
        let (clean_type_str, bang_required) = if type_str.ends_with('!') {
            (type_str[..type_str.len() - 1].to_string(), true)
        } else {
            (type_str.clone(), false)
        };

        let field_type = FieldType::from_str(&clean_type_str);
        let mut required = bang_required;
        let mut unique = false;
        let mut sensitive = false;
        let mut optional = false;
        let mut searchable = false;
        let mut index = false;
        let mut featured = false;
        let mut formatted = false;
        let mut enum_values = None;
        let mut default_value: Option<String> = None;
        let mut min: Option<f64> = None;
        let mut max: Option<f64> = None;
        let mut pattern: Option<String> = None;

        let field_line = field_token.line;

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.peek().line != field_line {
                break;
            }

            if self.matches(TokenKind::LBracket, None) {
                enum_values = Some(self.parse_array()?);
            } else if self.peek().kind == TokenKind::Identifier
                || self.peek().kind == TokenKind::ColonPair
            {
                // Same-line `author string! note text!` is a second field, not a modifier.
                // `email` is both a type and a common field name — only break when
                // the current token is NOT a known modifier.
                if self.peek().kind == TokenKind::Identifier {
                    let cur = self.peek().value.clone();
                    const MODIFIERS: &[&str] = &[
                        "required",
                        "unique",
                        "sensitive",
                        "optional",
                        "searchable",
                        "index",
                        "featured",
                        "formatted",
                    ];
                    if !MODIFIERS.contains(&cur.as_str()) {
                        if let Some(nxt) = self.tokens.get(self.pos + 1) {
                            let t = nxt.value.trim_end_matches('!');
                            let is_type = matches!(
                                t,
                                "string"
                                    | "text"
                                    | "email"
                                    | "url"
                                    | "slug"
                                    | "phone"
                                    | "number"
                                    | "money"
                                    | "percentage"
                                    | "boolean"
                                    | "date"
                                    | "ulid"
                                    | "json"
                                    | "enum"
                                    | "ip"
                            );
                            if is_type || nxt.kind == TokenKind::Arrow {
                                break;
                            }
                        }
                    }
                }
                let mod_val = self.advance().value;
                // Check for colon-pair modifiers like default:"value"
                if mod_val.contains(':') {
                    let (k, v) = Self::split_colon_pair(&mod_val);
                    match k.as_str() {
                        "default" => {
                            default_value = Some(v);
                        }
                        "min" => {
                            min = v.parse::<f64>().ok();
                        }
                        "max" => {
                            max = v.parse::<f64>().ok();
                        }
                        "match" => {
                            pattern = Some(v);
                        }
                        _ => {}
                    }
                } else {
                    match mod_val.as_str() {
                        "!" => required = true,
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
                }
            } else {
                break;
            }
        }

        // For string/text types, min/max map to min_length/max_length
        let is_string_type = matches!(
            field_type,
            FieldType::String
                | FieldType::Text
                | FieldType::Email
                | FieldType::Url
                | FieldType::Slug
                | FieldType::Phone
        );
        let (num_min, num_max, str_min_len, str_max_len) = if is_string_type {
            (None, None, min.map(|v| v as usize), max.map(|v| v as usize))
        } else {
            (min, max, None, None)
        };

        Ok(Some(FieldNode {
            name,
            field_type,
            required,
            unique,
            sensitive,
            optional,
            searchable,
            index,
            featured,
            formatted,
            array,
            enum_values,
            reference: None,
            doc: None,
            default_value,
            min: num_min,
            max: num_max,
            min_length: str_min_len,
            max_length: str_max_len,
            pattern,
        }))
    }

    // ── transition (state machine) ──

    fn parse_transition(&mut self, fields: &[FieldNode]) -> Result<TransitionNode, String> {
        let kw_token = self.advance(); // consume "transition"
        let field_name_token = self.peek().clone();
        let field_name = self.advance().value;

        // Validate: field must exist in the entity
        let field = fields.iter().find(|f| f.name == field_name);
        let field = match field {
            Some(f) => f,
            None => {
                return Err(format!(
                    "Line {}: transition references unknown field '{}'",
                    field_name_token.line, field_name
                ))
            }
        };

        // Validate: field must be an enum type
        if field.field_type != FieldType::Enum {
            return Err(format!(
                "Line {}: transition field '{}' must be an enum type, got {:?}",
                field_name_token.line, field_name, field.field_type
            ));
        }

        let enum_values = field.enum_values.as_ref().unwrap_or(&Vec::new()).clone();

        self.expect(TokenKind::LBrace)?;

        let mut rules = Vec::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            let from_token = self.peek().clone();
            let from = self.advance().value;

            // Validate: from state must exist in enum values
            if !enum_values.contains(&from) {
                return Err(format!(
                    "Line {}: transition state '{}' is not a valid value of enum field '{}'. Valid values: {:?}",
                    from_token.line, from, field_name, enum_values
                ));
            }

            self.expect(TokenKind::Arrow)?;

            let mut to = Vec::new();
            let first_to_token = self.peek().clone();
            let first_target = self.advance().value;

            // Validate first target
            if !enum_values.contains(&first_target) {
                return Err(format!(
                    "Line {}: transition target '{}' is not a valid value of enum field '{}'. Valid values: {:?}",
                    first_to_token.line, first_target, field_name, enum_values
                ));
            }
            to.push(first_target);

            // Parse additional targets separated by |
            while self.matches(TokenKind::Pipe, None) {
                self.advance(); // consume |
                let target_token = self.peek().clone();
                let target = self.advance().value;

                if !enum_values.contains(&target) {
                    return Err(format!(
                        "Line {}: transition target '{}' is not a valid value of enum field '{}'. Valid values: {:?}",
                        target_token.line, target, field_name, enum_values
                    ));
                }
                to.push(target);
            }

            rules.push(TransitionRule { from, to });
        }

        self.expect(TokenKind::RBrace)?;

        Ok(TransitionNode {
            field: field_name,
            rules,
        })
    }

    // ── effect block (on create/update/delete) ──

    fn parse_effect_block(&mut self) -> Result<EffectBlock, String> {
        let on_token = self.advance(); // consume "on"
        let event_token = self.peek().clone();
        let event = self.advance().value.to_lowercase(); // create, update, delete

        if !["create", "update", "delete"].contains(&event.as_str()) {
            return Err(format!(
                "Line {}: invalid effect event '{}', expected 'create', 'update', or 'delete'",
                event_token.line, event
            ));
        }

        // For "on update <field>", check if next token is a field name (not a brace)
        let field = if event == "update" && !self.matches(TokenKind::LBrace, None) {
            let f = self.advance().value;
            Some(f)
        } else {
            None
        };

        self.expect(TokenKind::LBrace)?;

        let mut actions = Vec::new();
        let mut current_condition: Option<String> = None;
        let mut in_when_block = false;

        loop {
            if self.matches(TokenKind::Eof, None) {
                break;
            }

            // If we see RBrace and we're inside a "when" block, close the when block
            if self.matches(TokenKind::RBrace, None) {
                if in_when_block {
                    self.advance(); // consume closing brace of when block
                    current_condition = None;
                    in_when_block = false;
                    continue;
                } else {
                    break; // closing brace of the effect block itself
                }
            }

            let peeked = self.peek().clone();

            // Handle "when" blocks: when "Value" { ... }
            if (peeked.kind == TokenKind::Identifier || peeked.kind == TokenKind::Keyword)
                && peeked.value == "when"
            {
                self.advance(); // consume "when"
                let condition_value = if self.peek().kind == TokenKind::StringLit {
                    self.advance().value
                } else {
                    self.advance().value
                };
                current_condition = Some(condition_value);
                self.expect(TokenKind::LBrace)?;
                in_when_block = true;
                continue;
            }

            // Parse action: log "message" or notify "provider" "channel" "message"
            if peeked.kind == TokenKind::Identifier || peeked.kind == TokenKind::Keyword {
                let action_type = self.advance().value.to_lowercase();

                match action_type.as_str() {
                    "log" => {
                        let msg = if self.peek().kind == TokenKind::StringLit {
                            self.advance().value
                        } else {
                            self.advance().value
                        };
                        actions.push(EffectAction {
                            action_type: "log".to_string(),
                            args: vec![msg],
                            condition: current_condition.clone(),
                        });
                    }
                    "notify" => {
                        // notify "provider" "channel" "message"
                        let mut args = Vec::new();
                        // Collect up to 3 string arguments
                        for _ in 0..3 {
                            if self.matches(TokenKind::RBrace, None)
                                || self.matches(TokenKind::Eof, None)
                            {
                                break;
                            }
                            let arg = if self.peek().kind == TokenKind::StringLit {
                                self.advance().value
                            } else if self.peek().kind == TokenKind::Identifier
                                || self.peek().kind == TokenKind::Keyword
                            {
                                // Don't consume if it's "when", "log", "notify" (next action)
                                let next = self.peek().value.clone();
                                if next == "when" || next == "log" || next == "notify" {
                                    break;
                                }
                                self.advance().value
                            } else {
                                break;
                            };
                            args.push(arg);
                        }
                        actions.push(EffectAction {
                            action_type: "notify".to_string(),
                            args,
                            condition: current_condition.clone(),
                        });
                    }
                    _ => {
                        // Unknown action — collect string args generically
                        let mut args = Vec::new();
                        while self.peek().kind == TokenKind::StringLit {
                            args.push(self.advance().value);
                        }
                        actions.push(EffectAction {
                            action_type: action_type.clone(),
                            args,
                            condition: current_condition.clone(),
                        });
                    }
                }
            } else {
                self.advance(); // skip unknown token
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(EffectBlock {
            event,
            field,
            actions,
        })
    }

    // ── api ──

    fn parse_api(&mut self) -> Result<ApiNode, String> {
        self.expect(TokenKind::Keyword)?;
        let prefix = self.expect(TokenKind::Path)?.value;
        self.expect(TokenKind::LBrace)?;

        let mut routes = Vec::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            let route_doc = self.collect_doc_comments();
            if self.peek().kind == TokenKind::Identifier {
                let name = self.advance().value;
                let method_tok = self.expect(TokenKind::Method)?;
                let method = HttpMethod::from_str(&method_tok.value).unwrap_or(HttpMethod::GET);
                let path = self.expect(TokenKind::Path)?.value;

                let mut auth = String::new();
                let mut roles = Vec::new();

                while self.peek().kind == TokenKind::ColonPair {
                    let (k, v) = Self::split_colon_pair(&self.advance().value);
                    if k == "auth" {
                        auth = v;
                    }
                }

                if self.matches(TokenKind::LBracket, None) {
                    roles = self.parse_array()?;
                }

                routes.push(RouteNode {
                    name,
                    method,
                    path,
                    auth,
                    roles,
                    doc: route_doc,
                });
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(ApiNode {
            prefix,
            routes,
            doc: None,
        })
    }

    // ── webhook ──

    fn parse_webhook(&mut self) -> Result<WebhookNode, String> {
        self.expect(TokenKind::Keyword)?; // consume "webhook"
        let entity = self.advance().value; // entity name or path
                                           // Strip leading / if present
        let entity = entity.trim_start_matches('/').to_string();
        self.expect(TokenKind::LBrace)?;

        let mut hooks = Vec::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            // Expect: on <event> -> <METHOD> "<url>"
            if self.matches(TokenKind::Keyword, Some("on"))
                || self.matches(TokenKind::Identifier, Some("on"))
            {
                self.advance(); // consume "on"
                let event = self.advance().value.to_lowercase(); // create, update, delete
                                                                 // Expect -> or =>
                if self.peek().kind == TokenKind::Arrow
                    || self.peek().value == "->"
                    || self.peek().value == "=>"
                {
                    self.advance();
                }
                let method = if self.peek().kind == TokenKind::Method {
                    self.advance().value.to_uppercase()
                } else {
                    self.advance().value.to_uppercase()
                };
                let url = if self.peek().kind == TokenKind::StringLit {
                    self.advance().value
                } else {
                    self.advance().value
                };

                // Optional headers: header "Key" "Value"
                let mut headers = Vec::new();
                while self.matches(TokenKind::Identifier, Some("header")) {
                    self.advance();
                    let key = if self.peek().kind == TokenKind::StringLit {
                        self.advance().value
                    } else {
                        self.advance().value
                    };
                    let val = if self.peek().kind == TokenKind::StringLit {
                        self.advance().value
                    } else {
                        self.advance().value
                    };
                    headers.push((key, val));
                }

                hooks.push(WebhookHook {
                    event,
                    method,
                    url,
                    headers,
                });
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(WebhookNode { entity, hooks })
    }

    // ── auth ──

    fn parse_auth(&mut self) -> Result<AuthNode, String> {
        self.advance(); // consume "auth" (tokenized as Identifier, not Keyword)
        self.expect(TokenKind::LBrace)?;

        let mut entity = String::new();
        let mut login_fields = Vec::new();
        let mut session_type = String::new();
        let mut session_config = HashMap::new();
        let mut roles = Vec::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.matches(TokenKind::Identifier, Some("entity"))
                || self.matches(TokenKind::Keyword, Some("entity"))
            {
                self.advance();
                entity = self.advance().value;
            } else if self.matches(TokenKind::Identifier, Some("login")) {
                self.advance();
                // Parse: email + password
                login_fields.push(self.advance().value);
                while self.try_consume(TokenKind::Plus, None).is_some() {
                    login_fields.push(self.advance().value);
                }
            } else if self.matches(TokenKind::Identifier, Some("session")) {
                self.advance();
                session_type = self.advance().value; // "jwt"
                                                     // Parse trailing key:value pairs like expires:24h
                while self.peek().kind == TokenKind::ColonPair {
                    let (k, v) = Self::split_colon_pair(&self.advance().value);
                    session_config.insert(k, v);
                }
            } else if self.matches(TokenKind::Identifier, Some("roles")) {
                self.advance();
                roles = self.parse_array()?;
            } else if self.matches(TokenKind::Identifier, Some("redirect")) {
                self.advance();
                let dest = if self.peek().kind == TokenKind::StringLit {
                    self.advance().value
                } else {
                    self.advance().value
                };
                session_config.insert("redirect".into(), dest);
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(AuthNode {
            entity,
            login_fields,
            session_type,
            session_config,
            roles,
        })
    }

    // ── layout ──

    fn parse_layout(&mut self) -> Result<LayoutNode, String> {
        // "layout" already consumed
        let name = self.advance().value;
        self.expect(TokenKind::LBrace)?;

        let mut sidebar_items = Vec::new();
        let mut sidebar_config = HashMap::new();
        let mut topbar_config = HashMap::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.matches(TokenKind::Identifier, Some("sidebar")) {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None)
                {
                    if self.matches(TokenKind::Identifier, Some("brand")) {
                        self.advance();
                        sidebar_config
                            .insert("brand".into(), self.expect(TokenKind::StringLit)?.value);
                    } else if self.matches(TokenKind::Identifier, Some("nav"))
                        || self.peek().kind == TokenKind::StringLit
                    {
                        // `nav` keyword is optional: `"Label" -> "/route"` also works
                        if self.matches(TokenKind::Identifier, Some("nav")) {
                            self.advance();
                        }
                        let label = self.expect(TokenKind::StringLit)?.value;
                        let mut route = String::new();
                        if self.try_consume(TokenKind::Arrow, None).is_some() {
                            route = if self.peek().kind == TokenKind::StringLit {
                                self.advance().value
                            } else {
                                self.advance().value // Path token like /orders
                            };
                        }
                        let mut icon = None;
                        let mut requires = None;
                        while self.peek().kind == TokenKind::ColonPair {
                            let (k, v) = Self::split_colon_pair(&self.advance().value);
                            if k == "icon" {
                                icon = Some(v.clone());
                            }
                            if k == "requires" {
                                requires = Some(v);
                            }
                        }
                        sidebar_items.push(LayoutNavItem {
                            label,
                            route,
                            icon,
                            requires,
                            is_divider: false,
                        });
                    } else if self.matches(TokenKind::Identifier, Some("divider")) {
                        self.advance();
                        sidebar_items.push(LayoutNavItem {
                            label: String::new(),
                            route: String::new(),
                            icon: None,
                            requires: None,
                            is_divider: true,
                        });
                    } else {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBrace)?;
            } else if self.matches(TokenKind::Identifier, Some("topbar")) {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None)
                {
                    if self.peek().kind == TokenKind::ColonPair {
                        let (k, v) = Self::split_colon_pair(&self.advance().value);
                        topbar_config.insert(k, v);
                    } else if self.peek().kind == TokenKind::Identifier {
                        let key = self.advance().value;
                        // handle "search placeholder:..." pattern
                        if self.peek().kind == TokenKind::ColonPair {
                            let (k, v) = Self::split_colon_pair(&self.advance().value);
                            topbar_config.insert(format!("{}_{}", key, k), v);
                        } else {
                            topbar_config.insert(key, String::new());
                        }
                    } else {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBrace)?;
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(LayoutNode {
            name,
            sidebar_items,
            sidebar_config,
            topbar_config,
        })
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
            if k == "type" {
                page_type = v.clone();
            }
            if k == "entity" {
                entity = Some(v.clone());
            }
            inline_config.insert(k, v);
        }

        self.expect(TokenKind::LBrace)?;

        let mut title = None;
        let mut sections = Vec::new();
        let mut config = inline_config;
        let mut components = Vec::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            // Collect doc-comments that may precede a section
            let inner_doc = self.collect_doc_comments();

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
                || self.matches(TokenKind::Identifier, Some("fields"))
            {
                let key = self.advance().value;
                let arr = self.parse_array()?;
                config.insert(key, arr.join(","));
            } else if self.matches(TokenKind::Identifier, Some("search"))
                || self.matches(TokenKind::Identifier, Some("filters"))
            {
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
                    if k == "limit" {
                        config.insert("recent_limit".into(), v);
                    }
                }
            } else if self.matches(TokenKind::Keyword, Some("section")) {
                let mut sec = self.parse_section()?;
                if sec.doc.is_none() {
                    sec.doc = inner_doc;
                }
                sections.push(sec);
            } else if self.peek().kind == TokenKind::Identifier
                && self
                    .peek()
                    .value
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                && !self
                    .peek()
                    .value
                    .chars()
                    .all(|c| c.is_uppercase() || c == '_')
            {
                // PascalCase identifier = component invocation: KPICard label:"Active" value:"1234"
                let comp_name = self.advance().value;
                let mut comp_props = HashMap::new();
                // Parse props: key:"value" or key:value
                while self.peek().kind == TokenKind::ColonPair
                    && !self.matches(TokenKind::Eof, None)
                {
                    let (k, v) = Self::split_colon_pair(&self.advance().value);
                    if v.is_empty()
                        && (self.peek().kind == TokenKind::StringLit
                            || self.peek().kind == TokenKind::Identifier
                            || self.peek().kind == TokenKind::Number)
                    {
                        comp_props.insert(k, self.advance().value);
                    } else {
                        comp_props.insert(k, v);
                    }
                }
                // Create a section that references the component
                let mut sec_config = comp_props;
                sec_config.insert("_component".to_string(), comp_name.clone());
                sections.push(SectionNode {
                    section_type: comp_name,
                    title: None,
                    subtitle: None,
                    config: sec_config,
                    items: Vec::new(),
                    plans: Vec::new(),
                    binding: None,
                    actions: Vec::new(),
                    visibility: None,
                    template: None,
                    style_block: None,
                    doc: inner_doc,
                });
            } else if self.peek().kind == TokenKind::ColonPair {
                let (k, v) = Self::split_colon_pair(&self.advance().value);
                config.insert(k, v);
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        let requires = config.remove("requires");
        Ok(PageNode {
            route,
            page_type,
            entity,
            title,
            sections,
            config,
            components,
            requires,
            doc: None,
        })
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

        // Parse visibility condition: show:when field == "value"
        let mut visibility = None;
        if config.get("show").map(|v| v.as_str()) == Some("when") {
            config.remove("show");
            let field = self.advance().value;
            let op_raw = if self.peek().kind == TokenKind::Operator {
                self.advance().value
            } else {
                "==".to_string() // default operator
            };
            let value = if self.peek().kind == TokenKind::StringLit {
                self.advance().value
            } else {
                self.advance().value
            };
            visibility = Some(VisibilityCondition {
                field,
                operator: op_raw,
                value,
            });
        }

        self.expect(TokenKind::LBrace)?;

        let mut title = None;
        let mut subtitle = None;
        let mut items = Vec::new();
        let mut plans = Vec::new();
        let mut binding: Option<BindingNode> = None;
        let mut section_actions: Vec<ActionBlock> = Vec::new();
        let mut template: Option<String> = None;
        let mut style_block: Option<String> = None;

        let mut cta_count = 0;
        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.matches(TokenKind::Identifier, Some("title")) {
                self.advance();
                title = Some(self.expect(TokenKind::StringLit)?.value);
            } else if self.matches(TokenKind::Identifier, Some("subtitle")) {
                self.advance();
                subtitle = Some(self.expect(TokenKind::StringLit)?.value);
            } else if self.matches(TokenKind::Identifier, Some("template")) {
                self.advance();
                template = Some(self.expect(TokenKind::StringLit)?.value);
            } else if self.matches(TokenKind::Identifier, Some("style_block")) {
                self.advance();
                style_block = Some(self.expect(TokenKind::StringLit)?.value);
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
            } else if self.matches(TokenKind::Identifier, Some("card_brand")) {
                self.advance();
                config.insert(
                    "card_brand".into(),
                    self.expect(TokenKind::StringLit)?.value,
                );
            } else if self.matches(TokenKind::Identifier, Some("card_number")) {
                self.advance();
                config.insert(
                    "card_number".into(),
                    self.expect(TokenKind::StringLit)?.value,
                );
            } else if self.matches(TokenKind::Identifier, Some("card_holder")) {
                self.advance();
                config.insert(
                    "card_holder".into(),
                    self.expect(TokenKind::StringLit)?.value,
                );
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
                while self.peek().kind == TokenKind::Identifier
                    || self.peek().kind == TokenKind::ColonPair
                {
                    if self.peek().kind == TokenKind::ColonPair {
                        let (k, v) = Self::split_colon_pair(&self.advance().value);
                        config.insert(format!("cta{}_{}", cta_count, k), v);
                    } else {
                        let val = &self.peek().value;
                        if val == "primary"
                            || val == "secondary"
                            || val == "pill"
                            || val == "ghost"
                            || val == "text"
                        {
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
                    while !self.matches(TokenKind::RBrace, None)
                        && !self.matches(TokenKind::Eof, None)
                    {
                        if self.peek().kind == TokenKind::ColonPair {
                            // Direct key:value pair (e.g. Status:"Fulfilled")
                            let (k, v) = Self::split_colon_pair(&self.advance().value);
                            map.insert(k.to_lowercase(), v);
                        } else if self.peek().kind == TokenKind::Identifier {
                            let key = self.advance().value;
                            if self.peek().kind == TokenKind::ColonPair {
                                // Multi-word key: "Asset" + "Type:\"Enterprise SaaS\""
                                // Combine identifier with colon-pair key to form full key
                                let (k2, v) = Self::split_colon_pair(&self.advance().value);
                                let full_key = format!("{} {}", key, k2);
                                map.insert(full_key.to_lowercase(), v);
                            } else if self.peek().kind == TokenKind::StringLit {
                                map.insert(key.to_lowercase(), self.advance().value);
                            } else if self.peek().kind == TokenKind::Identifier {
                                map.insert(key.to_lowercase(), self.advance().value);
                            }
                        } else {
                            self.advance();
                        }
                    }
                    if self.matches(TokenKind::RBrace, None) {
                        self.advance();
                    }
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
                || self.matches(TokenKind::Identifier, Some("tab"))
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
                while self.peek().kind == TokenKind::ColonPair
                    || self.peek().kind == TokenKind::Price
                {
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
                    if val == "primary"
                        || val == "secondary"
                        || val == "blink"
                        || val == "active"
                        || val == "success"
                        || val == "danger"
                    {
                        map.insert("style".into(), self.advance().value);
                    } else {
                        break;
                    }
                }
                // Optional { "content" } block
                if self.matches(TokenKind::LBrace, None) {
                    self.advance();
                    let mut parts = Vec::new();
                    while !self.matches(TokenKind::RBrace, None)
                        && !self.matches(TokenKind::Eof, None)
                    {
                        if self.peek().kind == TokenKind::StringLit {
                            parts.push(self.advance().value);
                        } else {
                            self.advance();
                        }
                    }
                    if !parts.is_empty() {
                        map.insert("description".into(), parts.join("\n"));
                    }
                    if self.matches(TokenKind::RBrace, None) {
                        self.advance();
                    }
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
                        if k == "options" && v.is_empty() && self.matches(TokenKind::LBracket, None)
                        {
                            let arr = self.parse_string_array()?;
                            map.insert(k, arr.join("||"));
                        } else if k == "options" && v.starts_with('[') {
                            // options:["A","B"] already tokenized as single value — strip brackets
                            let clean = v.trim_start_matches('[').trim_end_matches(']');
                            let opts: Vec<&str> = clean
                                .split(',')
                                .map(|s| s.trim().trim_matches('"').trim_matches('\''))
                                .collect();
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
                    while !self.matches(TokenKind::RBrace, None)
                        && !self.matches(TokenKind::Eof, None)
                    {
                        if self.peek().kind == TokenKind::StringLit {
                            parts.push(self.advance().value);
                        } else {
                            self.advance();
                        }
                    }
                    if !parts.is_empty() {
                        map.insert("description".into(), parts.join("\n"));
                    }
                    if self.matches(TokenKind::RBrace, None) {
                        self.advance();
                    }
                }
                items.push(map);
            } else if self.matches(TokenKind::Identifier, Some("search")) {
                // search "field1,field2" — store in config for renderer
                self.advance();
                if self.peek().kind == TokenKind::StringLit {
                    config.insert("search".into(), self.advance().value);
                } else {
                    config.insert("search".into(), String::new());
                }
            } else if self.matches(TokenKind::Identifier, Some("paginate")) {
                // paginate 25 — store per-page count in config
                self.advance();
                if self.peek().kind == TokenKind::Number {
                    config.insert("paginate".into(), self.advance().value);
                } else if self.peek().kind == TokenKind::Identifier {
                    config.insert("paginate".into(), self.advance().value);
                } else {
                    config.insert("paginate".into(), "25".into());
                }
            } else if self.matches(TokenKind::Keyword, Some("on"))
                || self.matches(TokenKind::Identifier, Some("on"))
            {
                self.advance(); // consume "on"
                let action_block = self.parse_action_block()?;
                section_actions.push(action_block);
            } else if self.matches(TokenKind::Identifier, Some("live")) {
                self.advance(); // consume "live"
                                // "live bind Entity { ... }" or "live list Entity { ... }"
                if self.matches(TokenKind::Identifier, Some("bind"))
                    || self.matches(TokenKind::Identifier, Some("list"))
                {
                    let mut b = self.parse_binding()?;
                    b.live = true;
                    binding = Some(b);
                }
            } else if self.matches(TokenKind::Identifier, Some("bind")) {
                binding = Some(self.parse_binding()?);
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
                    if self.peek().kind == TokenKind::LBrace {
                        depth += 1;
                    }
                    if self.peek().kind == TokenKind::RBrace {
                        depth -= 1;
                    }
                    if depth > 0 {
                        self.advance();
                    }
                }
                if self.matches(TokenKind::RBrace, None) {
                    self.advance();
                }
            } else if self.peek().kind == TokenKind::Identifier {
                // Generic: unknown identifier followed by string literal → store as config
                let key = self.advance().value;
                if self.peek().kind == TokenKind::StringLit {
                    config.insert(key, self.advance().value);
                }
                // else: bare identifier, skip it
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;

        // Backward compat: synthesize binding from entity config
        if binding.is_none() {
            if let Some(entity) = config.get("entity") {
                binding = Some(BindingNode {
                    entity: entity.clone(),
                    query: QueryType::All,
                    filters: Vec::new(),
                    order: None,
                    limit: None,
                    offset: None,
                    group_by: None,
                    aggregate: None,
                    live: false,
                    public: false,
                });
            }
        }

        Ok(SectionNode {
            section_type,
            title,
            subtitle,
            config,
            items,
            plans,
            binding,
            actions: section_actions,
            visibility,
            template,
            style_block,
            doc: None,
        })
    }

    fn parse_section_item(&mut self) -> Result<HashMap<String, String>, String> {
        self.advance(); // consume "item"
        let title = self.expect(TokenKind::StringLit)?.value;
        let mut map = HashMap::new();
        map.insert("title".into(), title);

        // Arrow for link: item "Text" -> "/url"
        if self.try_consume(TokenKind::Arrow, None).is_some() {
            if self.peek().kind == TokenKind::StringLit {
                map.insert("href".into(), self.advance().value);
            }
        }

        // Parse inline attributes: icon:x status:active etc
        while self.peek().kind == TokenKind::ColonPair
            || self.peek().kind == TokenKind::StringLit
            || self.peek().kind == TokenKind::Price
        {
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
                } else if self.peek().kind == TokenKind::Keyword && self.peek().value == "on" {
                    // Item-level action: on click { set ... }
                    self.advance(); // consume "on"
                    let action = self.parse_action_block()?;
                    let serialized = serde_json::to_string(&action).unwrap_or_default();
                    map.insert(format!("on_{}", action.event), serialized);
                } else if self.peek().kind == TokenKind::Identifier {
                    // Sub-items inside {}: action "text" icon:x, price "$99", etc
                    let key = self.advance().value;
                    if key == "on" {
                        // Item-level action (as identifier): on click { set ... }
                        let action = self.parse_action_block()?;
                        let serialized = serde_json::to_string(&action).unwrap_or_default();
                        map.insert(format!("on_{}", action.event), serialized);
                    } else if key == "action"
                        || key == "price"
                        || key == "description"
                        || key == "meta"
                        || key == "detail"
                        || key == "footer"
                        || key == "link"
                        || key == "subtitle"
                        || key == "badge"
                    {
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
                        if self.peek().kind == TokenKind::LBrace {
                            depth += 1;
                        }
                        if self.peek().kind == TokenKind::RBrace {
                            depth -= 1;
                        }
                        if depth > 0 {
                            self.advance();
                        }
                    }
                    if self.matches(TokenKind::RBrace, None) {
                        self.advance();
                    }
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

    fn parse_action_block(&mut self) -> Result<ActionBlock, String> {
        // Already consumed "on" keyword before calling this
        let event = self.advance().value; // "click", "submit", "error", "change"
        self.expect(TokenKind::LBrace)?;

        let mut instructions = Vec::new();
        let mut confirm_msg = None;

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            let verb = self.advance().value;
            match verb.as_str() {
                "confirm" => {
                    confirm_msg = Some(self.expect(TokenKind::StringLit)?.value);
                }
                "set" => {
                    let field = self.advance().value;
                    let value = if self.peek().kind == TokenKind::StringLit {
                        self.advance().value
                    } else {
                        self.advance().value
                    };
                    let mut mods = HashMap::new();
                    while self.peek().kind == TokenKind::ColonPair {
                        let (k, v) = Self::split_colon_pair(&self.advance().value);
                        mods.insert(k, v);
                    }
                    instructions.push(ActionInstruction {
                        verb: "set".into(),
                        target: field,
                        value,
                        modifiers: mods,
                    });
                }
                "toast" => {
                    let message = self.expect(TokenKind::StringLit)?.value;
                    let mut mods = HashMap::new();
                    while self.peek().kind == TokenKind::ColonPair {
                        let (k, v) = Self::split_colon_pair(&self.advance().value);
                        mods.insert(k, v);
                    }
                    instructions.push(ActionInstruction {
                        verb: "toast".into(),
                        target: message,
                        value: String::new(),
                        modifiers: mods,
                    });
                }
                "navigate" => {
                    let url = if self.peek().kind == TokenKind::StringLit {
                        self.advance().value
                    } else if self.peek().kind == TokenKind::Path {
                        self.advance().value
                    } else {
                        self.advance().value // "back"
                    };
                    instructions.push(ActionInstruction {
                        verb: "navigate".into(),
                        target: url,
                        value: String::new(),
                        modifiers: HashMap::new(),
                    });
                }
                "refresh" => {
                    let target = self.advance().value; // "self", "parent", "page"
                    instructions.push(ActionInstruction {
                        verb: "refresh".into(),
                        target,
                        value: String::new(),
                        modifiers: HashMap::new(),
                    });
                }
                "create" | "update" | "delete" => {
                    let target = self.advance().value; // "entity" or entity name
                    instructions.push(ActionInstruction {
                        verb: verb.clone(),
                        target,
                        value: String::new(),
                        modifiers: HashMap::new(),
                    });
                }
                "validate" => {
                    let target = self.advance().value; // "all" or field name
                    instructions.push(ActionInstruction {
                        verb: "validate".into(),
                        target,
                        value: String::new(),
                        modifiers: HashMap::new(),
                    });
                }
                "open" | "close" => {
                    let target = self.expect(TokenKind::StringLit)?.value;
                    instructions.push(ActionInstruction {
                        verb: verb.clone(),
                        target,
                        value: String::new(),
                        modifiers: HashMap::new(),
                    });
                }
                _ => {
                    // Unknown verb — skip to next known verb or closing brace
                    while !self.matches(TokenKind::RBrace, None)
                        && !self.matches(TokenKind::Eof, None)
                    {
                        let next = self.peek();
                        if next.kind == TokenKind::Identifier
                            && [
                                "set", "toast", "navigate", "refresh", "create", "update",
                                "delete", "validate", "confirm", "open", "close",
                            ]
                            .contains(&next.value.as_str())
                        {
                            break;
                        }
                        if next.kind == TokenKind::Keyword
                            && [
                                "set", "toast", "navigate", "refresh", "create", "update",
                                "delete", "validate", "confirm", "open", "close",
                            ]
                            .contains(&next.value.as_str())
                        {
                            break;
                        }
                        self.advance();
                    }
                }
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(ActionBlock {
            event,
            confirm: confirm_msg,
            instructions,
        })
    }

    fn parse_plan(&mut self) -> Result<PlanNode, String> {
        self.advance(); // consume "plan"
        let name = self.expect(TokenKind::StringLit)?.value;
        let price = if self.peek().kind == TokenKind::Price {
            self.advance().value.clone()
        } else {
            String::new()
        };

        let featured = self
            .try_consume(TokenKind::Identifier, Some("featured"))
            .is_some();

        let features = if self.matches(TokenKind::LBracket, None) {
            self.parse_string_array()?
        } else {
            Vec::new()
        };

        Ok(PlanNode {
            name,
            price,
            featured,
            features,
        })
    }

    // ── binding ──

    fn parse_binding(&mut self) -> Result<BindingNode, String> {
        self.advance(); // consume "bind"

        // Next token: ColonPair "entity:Order" or bare Identifier "Order"
        let entity_token = self.advance();
        let entity_name = if entity_token.kind == TokenKind::ColonPair {
            let (_key, val) = Self::split_colon_pair(&entity_token.value);
            val
        } else {
            // Bare identifier — this IS the entity name (e.g. "Deployment")
            entity_token.value.clone()
        };

        self.expect(TokenKind::LBrace)?;

        let mut query = QueryType::All;
        let mut filters = Vec::new();
        let mut order = None;
        let mut limit = None;
        let mut offset = None;
        let mut group_by = None;
        let mut aggregate = None;
        let mut public = false;
        let mut live = false;
        let mut agg_field: Option<String> = None;
        let mut group_interval: Option<String> = None;

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            let kw = self.advance();
            let (kw_name, kw_val) = if kw.value.contains(':') {
                let (k, v) = Self::split_colon_pair(&kw.value);
                (k, Some(v))
            } else {
                (kw.value.clone(), None)
            };
            match kw_name.as_str() {
                "query" => {
                    let qt = self.advance().value;
                    query = match qt.as_str() {
                        "one" => QueryType::One,
                        "count" => QueryType::Count,
                        _ => QueryType::All,
                    };
                }
                "where" => {
                    let field = self.advance().value.clone();
                    // Two equivalent forms: `status eq "active"` (operator and
                    // value as separate tokens) and `status eq:"active"` (the
                    // tokenizer emits one ColonPair). Both yield the same FilterExpr.
                    let op_token = self.advance();
                    let (op_str, value) = if op_token.kind == TokenKind::ColonPair {
                        let (op, raw) = op_token
                            .value
                            .split_once(':')
                            .map(|(o, r)| (o.to_string(), r.to_string()))
                            .unwrap_or_default();
                        (op, Self::filter_value_from_colon(&raw))
                    } else {
                        let val_token = self.advance();
                        (op_token.value.clone(), Self::filter_value(&val_token))
                    };
                    let op = match op_str.as_str() {
                        "eq" => FilterOp::Eq,
                        "ne" | "neq" => FilterOp::Ne,
                        "gt" => FilterOp::Gt,
                        "gte" => FilterOp::Gte,
                        "lt" => FilterOp::Lt,
                        "lte" => FilterOp::Lte,
                        "contains" => FilterOp::Contains,
                        "starts_with" => FilterOp::StartsWith,
                        _ => FilterOp::Eq,
                    };
                    filters.push(FilterExpr {
                        field,
                        operator: op,
                        value,
                    });
                }
                "order" => {
                    let field = self.advance().value.clone();
                    let dir = if self.matches(TokenKind::Identifier, Some("desc")) {
                        self.advance();
                        OrderDirection::Desc
                    } else {
                        if self.matches(TokenKind::Identifier, Some("asc")) {
                            self.advance();
                        }
                        OrderDirection::Asc
                    };
                    order = Some(OrderExpr {
                        field,
                        direction: dir,
                    });
                }
                "limit" => {
                    limit = Some(self.advance().value.parse::<usize>().unwrap_or(100));
                }
                "offset" => {
                    offset = Some(self.advance().value.parse::<usize>().unwrap_or(0));
                }
                "group" => {
                    let field = self.advance().value.clone();
                    let mut interval = None;
                    if self.peek().kind == TokenKind::ColonPair {
                        let (k, v) = Self::split_colon_pair(&self.advance().value);
                        if k == "by" {
                            interval = Some(v);
                        }
                    }
                    group_by = Some(GroupByExpr { field, interval });
                }
                "aggregate" => {
                    let func_or_expr = self.advance().value.clone();
                    if func_or_expr.contains('(') {
                        let paren = func_or_expr.find('(').unwrap();
                        let func = func_or_expr[..paren].to_string();
                        let agg_field = func_or_expr[paren + 1..].trim_end_matches(')').to_string();
                        aggregate = Some(AggregateExpr {
                            function: func,
                            field: Some(agg_field),
                        });
                    } else {
                        aggregate = Some(AggregateExpr {
                            function: func_or_expr,
                            field: None,
                        });
                    }
                }
                // Colon forms used by docs/templates:
                // `aggregate sum field:total group_by:created_at interval:month`.
                "field" => {
                    agg_field = Some(kw_val.unwrap_or_else(|| self.advance().value));
                }
                "group_by" => {
                    let field = kw_val.unwrap_or_else(|| self.advance().value);
                    group_by = Some(GroupByExpr {
                        field,
                        interval: None,
                    });
                }
                "interval" => {
                    group_interval = Some(kw_val.unwrap_or_else(|| self.advance().value));
                }
                "scope" => {
                    let v = kw_val.unwrap_or_else(|| self.advance().value);
                    public = v.eq_ignore_ascii_case("public");
                }
                "live" => {
                    live = match kw_val.as_deref() {
                        Some("false") | Some("off") => false,
                        Some("true") | Some("on") => true,
                        Some(_) => true,
                        None => {
                            let v = self.peek().value.clone();
                            if v == "true" || v == "on" || v == "false" || v == "off" {
                                self.advance();
                                v == "true" || v == "on"
                            } else {
                                true
                            }
                        }
                    };
                }
                _ => {} // skip unknown
            }
        }

        if let (Some(agg), Some(field)) = (aggregate.as_mut(), agg_field) {
            if agg.field.is_none() {
                agg.field = Some(field);
            }
        }
        if let (Some(group), Some(interval)) = (group_by.as_mut(), group_interval) {
            if group.interval.is_none() {
                group.interval = Some(interval);
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(BindingNode {
            entity: entity_name,
            query,
            filters,
            order,
            limit,
            offset,
            group_by,
            aggregate,
            live,
            public,
        })
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
            if self.matches(TokenKind::Identifier, Some("theme")) {
                self.advance();
                theme = Some(self.advance().value);
            } else if self.matches(TokenKind::Identifier, Some("accent")) {
                self.advance();
                accent = Some(self.advance().value);
            } else if self.matches(TokenKind::Identifier, Some("radius")) {
                self.advance();
                radius = Some(self.advance().value);
            } else if self.matches(TokenKind::Identifier, Some("font")) {
                self.advance();
                font = Some(self.advance().value);
            } else if self.peek().kind == TokenKind::Identifier {
                let key = self.advance().value;
                let val = self.advance().value;
                config.insert(key, val);
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(StyleNode {
            theme,
            accent,
            radius,
            font,
            config,
        })
    }

    // ── service ──

    fn parse_service(&mut self) -> Result<ServiceNode, String> {
        self.expect(TokenKind::Keyword)?;
        let name = self.advance().value;

        let mut port = None;
        let mut config = HashMap::new();

        while self.peek().kind == TokenKind::ColonPair {
            let (k, v) = Self::split_colon_pair(&self.advance().value);
            if k == "port" {
                port = v.parse().ok();
            } else {
                config.insert(k, v);
            }
        }

        self.expect(TokenKind::LBrace)?;

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.peek().kind == TokenKind::Identifier {
                let key = self.advance().value;
                let key_line = self.tokens.get(self.pos - 1).map(|t| t.line).unwrap_or(0);

                while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None)
                {
                    let next_line = self.peek().line;
                    if next_line != key_line && self.peek().kind == TokenKind::Identifier {
                        break;
                    }

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
                    } else if matches!(
                        self.peek().kind,
                        TokenKind::Identifier
                            | TokenKind::StringLit
                            | TokenKind::Number
                            | TokenKind::Path
                    ) {
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

    /// Parse `define "Name" { section ... section ... }`
    /// Stores one or more reusable sections under a name.
    fn parse_define(&mut self) -> Result<DefineNode, String> {
        self.advance(); // consume "define"
        let name = if self.peek().kind == TokenKind::StringLit {
            self.advance().value
        } else {
            self.advance().value
        };

        self.expect(TokenKind::LBrace)?;

        let mut sections = Vec::new();
        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            let inner_doc = self.collect_doc_comments();
            if self.matches(TokenKind::Keyword, Some("section")) {
                let mut sec = self.parse_section()?;
                if sec.doc.is_none() {
                    sec.doc = inner_doc;
                }
                sections.push(sec);
            } else {
                self.advance(); // skip unknown tokens
            }
        }
        self.expect(TokenKind::RBrace)?;

        Ok(DefineNode { name, sections })
    }

    fn parse_component(&mut self) -> Result<ComponentNode, String> {
        self.expect(TokenKind::Keyword)?;
        let name = self.advance().value;

        // Parse params: component Name(label: text, value: money, icon?: text)
        let mut params = Vec::new();
        if self.matches(TokenKind::LParen, None) {
            self.advance(); // consume (
            while !self.matches(TokenKind::RParen, None) && !self.matches(TokenKind::Eof, None) {
                if self.matches(TokenKind::Comma, None) {
                    self.advance();
                    continue;
                }
                if self.matches(TokenKind::RParen, None) {
                    break;
                }
                let raw = self.advance().value;
                // Handle colon pair: "label:text" or bare "label"
                if raw.contains(':') {
                    let (pname, ptype) = Self::split_colon_pair(&raw);
                    let (clean_name, required) = if pname.ends_with('?') {
                        (pname.trim_end_matches('?').to_string(), false)
                    } else {
                        (pname, true)
                    };
                    let param_type = if ptype.is_empty() {
                        if self.peek().kind == TokenKind::Identifier {
                            self.advance().value
                        } else {
                            "any".to_string()
                        }
                    } else {
                        ptype
                    };
                    params.push(ComponentParam {
                        name: clean_name,
                        param_type,
                        default: None,
                        required,
                    });
                } else {
                    // Bare name, check for ColonPair next
                    let (clean_name, required) = if raw.ends_with('?') {
                        (raw.trim_end_matches('?').to_string(), false)
                    } else {
                        (raw, true)
                    };
                    let param_type = if self.peek().kind == TokenKind::ColonPair {
                        let (_, v) = Self::split_colon_pair(&self.advance().value);
                        if v.is_empty() {
                            "any".to_string()
                        } else {
                            v
                        }
                    } else {
                        "any".to_string()
                    };
                    params.push(ComponentParam {
                        name: clean_name,
                        param_type,
                        default: None,
                        required,
                    });
                }
            }
            if self.matches(TokenKind::RParen, None) {
                self.advance();
            }
        }

        // Parse attributes before the opening brace: layout:inline style:topbar+light
        let mut layout = None;
        let mut style = None;
        let mut template: Option<String> = None;
        let mut items = Vec::new();
        let mut props = HashMap::new();
        let mut state_vars: Vec<ComponentState> = Vec::new();
        let mut tests: Vec<ComponentTest> = Vec::new();
        let mut binding: Option<BindingNode> = None;

        while !self.matches(TokenKind::LBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.peek().kind == TokenKind::ColonPair {
                let (k, v) = Self::split_colon_pair(&self.advance().value);
                if k == "layout" {
                    let mut val = v;
                    if val.is_empty()
                        && (self.peek().kind == TokenKind::Identifier
                            || self.peek().kind == TokenKind::StringLit)
                    {
                        val = self.advance().value;
                    }
                    layout = Some(val);
                } else if k == "style" {
                    let mut parts = vec![if v.is_empty() {
                        self.advance().value
                    } else {
                        v
                    }];
                    while self.try_consume(TokenKind::Plus, None).is_some() {
                        parts.push(self.advance().value);
                    }
                    style = Some(parts.join("+"));
                } else {
                    if v.is_empty()
                        && (self.peek().kind == TokenKind::Identifier
                            || self.peek().kind == TokenKind::StringLit)
                    {
                        props.insert(k, self.advance().value);
                    } else {
                        props.insert(k, v);
                    }
                }
            } else if self.peek().kind == TokenKind::Identifier {
                // Handle bare identifiers before brace
                let ident = self.advance().value;
                if self.peek().kind == TokenKind::StringLit
                    || self.peek().kind == TokenKind::Identifier
                {
                    props.insert(ident, self.advance().value);
                }
            } else {
                break;
            }
        }

        self.expect(TokenKind::LBrace)?;

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            // state count: integer = 0
            if self.matches(TokenKind::Identifier, Some("bind")) {
                binding = Some(self.parse_binding()?);
                continue;
            }
            if self.matches(TokenKind::Identifier, Some("state"))
                || self.matches(TokenKind::Keyword, Some("state"))
            {
                self.advance();
                let raw = self.advance().value;
                // Handle "count:integer" (colon pair) or "count" then next token
                let (sname, stype) = if raw.contains(':') {
                    let parts: Vec<&str> = raw.splitn(2, ':').collect();
                    let mut t = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
                    if t.is_empty()
                        && (self.peek().kind == TokenKind::Identifier
                            || self.peek().kind == TokenKind::Keyword)
                    {
                        t = self.advance().value;
                    }
                    (
                        parts[0].to_string(),
                        if t.is_empty() { "any".to_string() } else { t },
                    )
                } else if self.peek().kind == TokenKind::ColonPair {
                    let (_, v) = Self::split_colon_pair(&self.advance().value);
                    (
                        raw,
                        if v.is_empty() {
                            self.advance().value
                        } else {
                            v
                        },
                    )
                } else {
                    (raw, "any".to_string())
                };
                // Check for = default
                let default = if self.peek().value == "="
                    || self.peek().kind == TokenKind::Operator && self.peek().value == "="
                {
                    self.advance();
                    self.advance().value
                } else {
                    match stype.as_str() {
                        "integer" | "number" => "0".to_string(),
                        "boolean" => "false".to_string(),
                        _ => String::new(),
                    }
                };
                state_vars.push(ComponentState {
                    name: sname,
                    state_type: stype,
                    default,
                });
                continue;
            }
            if self.matches(TokenKind::Identifier, Some("layout")) {
                self.advance();
                layout = Some(self.advance().value);
            } else if self.matches(TokenKind::Identifier, Some("style"))
                || self.matches(TokenKind::Keyword, Some("style"))
            {
                self.advance();
                let mut parts = vec![self.advance().value];
                while self.try_consume(TokenKind::Plus, None).is_some() {
                    parts.push(self.advance().value);
                }
                style = Some(parts.join("+"));
            } else if self.matches(TokenKind::Identifier, Some("items")) {
                self.advance();
                self.expect(TokenKind::LBracket)?;
                while !self.matches(TokenKind::RBracket, None)
                    && !self.matches(TokenKind::Eof, None)
                {
                    if self.peek().kind == TokenKind::Identifier
                        || self.peek().kind == TokenKind::Keyword
                    {
                        let item_type = self.advance().value;
                        let text = if self.peek().kind == TokenKind::StringLit {
                            self.advance().value
                        } else {
                            String::new()
                        };
                        let link = if self.try_consume(TokenKind::Arrow, None).is_some() {
                            Some(if self.peek().kind == TokenKind::StringLit {
                                self.advance().value
                            } else {
                                self.advance().value
                            })
                        } else {
                            None
                        };
                        let mut item_config = HashMap::new();
                        let mut tone = None;
                        // Parse key:value pairs including tone and price
                        while self.peek().kind == TokenKind::ColonPair
                            || self.peek().kind == TokenKind::Price
                        {
                            if self.peek().kind == TokenKind::Price {
                                item_config.insert("price".to_string(), self.advance().value);
                                continue;
                            }
                            let (k, v) = Self::split_colon_pair(&self.advance().value);
                            if k == "tone" {
                                tone = Some(
                                    if v.is_empty() && self.peek().kind == TokenKind::Identifier {
                                        self.advance().value
                                    } else {
                                        v
                                    },
                                );
                            } else if v.is_empty()
                                && (self.peek().kind == TokenKind::Identifier
                                    || self.peek().kind == TokenKind::StringLit)
                            {
                                item_config.insert(k, self.advance().value);
                            } else {
                                item_config.insert(k, v);
                            }
                        }
                        items.push(ComponentItemNode {
                            item_type,
                            text,
                            link,
                            tone,
                            config: item_config,
                        });
                    } else {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBracket)?;
            } else if self.peek().kind == TokenKind::ColonPair {
                // Generic props: key:value at component level
                let (k, v) = Self::split_colon_pair(&self.advance().value);
                if v.is_empty()
                    && (self.peek().kind == TokenKind::Identifier
                        || self.peek().kind == TokenKind::StringLit)
                {
                    props.insert(k, self.advance().value);
                } else {
                    props.insert(k, v);
                }
            } else if self.peek().kind == TokenKind::Identifier
                && is_valid_item_type(&self.peek().value)
            {
                // Recognized item type: parse as ComponentItemNode
                let item_type = self.advance().value;
                let text = if self.peek().kind == TokenKind::StringLit {
                    self.advance().value
                } else {
                    String::new()
                };
                let link = if self.try_consume(TokenKind::Arrow, None).is_some() {
                    Some(if self.peek().kind == TokenKind::StringLit {
                        self.advance().value
                    } else {
                        self.advance().value
                    })
                } else {
                    None
                };
                let mut item_config = HashMap::new();
                let mut tone = None;
                while self.peek().kind == TokenKind::ColonPair
                    || self.peek().kind == TokenKind::Price
                {
                    if self.peek().kind == TokenKind::Price {
                        item_config.insert("price".to_string(), self.advance().value);
                        continue;
                    }
                    let (k, v) = Self::split_colon_pair(&self.advance().value);
                    if k == "tone" {
                        tone = Some(
                            if v.is_empty() && self.peek().kind == TokenKind::Identifier {
                                self.advance().value
                            } else {
                                v
                            },
                        );
                    } else if v.is_empty()
                        && (self.peek().kind == TokenKind::Identifier
                            || self.peek().kind == TokenKind::StringLit)
                    {
                        item_config.insert(k, self.advance().value);
                    } else {
                        item_config.insert(k, v);
                    }
                }
                // Also store single-value item types (brand, subtitle, title) as props for easy access
                if (item_type == "brand" || item_type == "subtitle" || item_type == "title")
                    && !text.is_empty()
                {
                    props
                        .entry(item_type.clone())
                        .or_insert_with(|| text.clone());
                }
                items.push(ComponentItemNode {
                    item_type,
                    text,
                    link,
                    tone,
                    config: item_config,
                });
            } else if self.matches(TokenKind::Identifier, Some("template"))
                || self.matches(TokenKind::Keyword, Some("template"))
            {
                self.advance();
                if self.peek().kind == TokenKind::StringLit {
                    template = Some(self.advance().value);
                }
            } else if self.matches(TokenKind::Identifier, Some("test"))
                || self.matches(TokenKind::Keyword, Some("test"))
            {
                // test "description" { step1; step2; ... }
                self.advance();
                let test_name = if self.peek().kind == TokenKind::StringLit {
                    self.advance().value
                } else {
                    format!("test_{}", tests.len() + 1)
                };
                let mut steps = Vec::new();
                if self.matches(TokenKind::LBrace, None) {
                    self.advance();
                    while !self.matches(TokenKind::RBrace, None)
                        && !self.matches(TokenKind::Eof, None)
                    {
                        // Each step is a line of text tokens until newline (approximated by reading until next keyword or })
                        let mut step = Vec::new();
                        while !self.matches(TokenKind::RBrace, None)
                            && !self.matches(TokenKind::Eof, None)
                        {
                            let tok = self.peek();
                            // Heuristic: a new step starts with a keyword like fill, click, expect, navigate
                            if !step.is_empty()
                                && (tok.value == "fill"
                                    || tok.value == "click"
                                    || tok.value == "expect"
                                    || tok.value == "navigate"
                                    || tok.value == "wait"
                                    || tok.value == "assert")
                            {
                                break;
                            }
                            step.push(self.advance().value);
                        }
                        if !step.is_empty() {
                            steps.push(step.join(" "));
                        }
                    }
                    if self.matches(TokenKind::RBrace, None) {
                        self.advance();
                    }
                }
                tests.push(ComponentTest {
                    name: test_name,
                    steps,
                });
            } else if self.peek().kind == TokenKind::Identifier {
                // Unknown identifier props: key value
                let key = self.advance().value;
                if self.peek().kind == TokenKind::StringLit
                    || self.peek().kind == TokenKind::Number
                    || self.peek().kind == TokenKind::Identifier
                {
                    props.insert(key, self.advance().value);
                }
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(ComponentNode {
            name,
            layout,
            style,
            items,
            props,
            params,
            template,
            sections: Vec::new(),
            state: state_vars,
            tests,
            binding,
        })
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
            while !self.matches(TokenKind::RBrace, None)
                && !self.matches(TokenKind::Eof, None)
                && self.peek().line == action_line
            {
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
            if k == "queue" {
                queue = Some(v);
            }
        }

        self.expect(TokenKind::LBrace)?;

        let mut concurrency = None;
        let mut retry = None;
        let mut timeout = None;
        let mut entity = None;

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.matches(TokenKind::Identifier, Some("concurrency")) {
                self.advance();
                concurrency = self.advance().value.parse().ok();
            } else if self.matches(TokenKind::Identifier, Some("retry")) {
                self.advance();
                retry = self.advance().value.parse().ok();
            } else if self.matches(TokenKind::Identifier, Some("timeout")) {
                self.advance();
                timeout = Some(self.advance().value);
            } else if self.matches(TokenKind::Identifier, Some("process")) {
                self.advance();
                entity = Some(self.advance().value);
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(WorkerNode {
            name,
            queue,
            concurrency,
            retry,
            timeout,
            entity,
        })
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
                while !self.matches(TokenKind::RBrace, None)
                    && !self.matches(TokenKind::Eof, None)
                    && self.peek().line == key_line
                {
                    parts.push(self.advance().value);
                }
                if !parts.is_empty() {
                    config.insert(key, parts.join(" "));
                }
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(MiddlewareNode {
            name,
            applies_to,
            config,
        })
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
                    while !self.matches(TokenKind::RBrace, None)
                        && !self.matches(TokenKind::Eof, None)
                    {
                        if self.peek().kind == TokenKind::ColonPair {
                            let (k, v) = Self::split_colon_pair(&self.advance().value);
                            if v.is_empty()
                                && (self.peek().kind == TokenKind::StringLit
                                    || self.peek().kind == TokenKind::Identifier)
                            {
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

                steps.push(TestStepNode {
                    action,
                    entity,
                    body,
                    expect: expect_code,
                    expect_config,
                });
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(TestNode { name, steps })
    }

    // ── P040/P041: SQL identifier validation ──

    /// Validate an identifier against P040 (valid pattern) and P041 (no SQL reserved words).
    fn validate_identifier(name: &str, context: &str, line: usize) -> Result<(), String> {
        Self::validate_identifier_pattern(name, context, line)?;
        Self::reject_sql_reserved(name, context, line)
    }

    /// P040 only: `[a-zA-Z][a-zA-Z0-9_]{0,63}`. Used for bracketed array values
    /// (enum values, roles, config lists). Those are data bound as parameters,
    /// never SQL identifiers, so P041 does not apply: `enum [create, update]` is legal.
    fn validate_identifier_pattern(name: &str, context: &str, line: usize) -> Result<(), String> {
        // P040: Must match [a-zA-Z][a-zA-Z0-9_]{0,63}
        if name.is_empty() {
            return Err(format!(
                "Parse error at line {}: Empty identifier in {}\n  Identifiers must start with a letter and contain only [a-zA-Z0-9_]",
                line, context
            ));
        }

        let bytes = name.as_bytes();
        let first = bytes[0];
        if !(first.is_ascii_alphabetic()) {
            return Err(format!(
                "Parse error at line {}: Invalid identifier '{}' in {}\n  Identifiers must start with a letter and contain only [a-zA-Z0-9_]",
                line, name, context
            ));
        }

        if name.len() > 64 {
            return Err(format!(
                "Parse error at line {}: Identifier '{}' in {} exceeds 64 characters\n  Identifiers must be at most 64 characters long",
                line, name, context
            ));
        }

        for (i, &b) in bytes.iter().enumerate().skip(1) {
            if !(b.is_ascii_alphanumeric() || b == b'_') {
                return Err(format!(
                    "Parse error at line {}: Invalid identifier '{}' in {}\n  Identifiers must start with a letter and contain only [a-zA-Z0-9_]",
                    line, name, context
                ));
            }
        }

        Ok(())
    }

    /// P041: No SQL reserved words (entity/field names become SQL identifiers).
    fn reject_sql_reserved(name: &str, context: &str, line: usize) -> Result<(), String> {
        const SQL_RESERVED: &[&str] = &[
            "SELECT", "DROP", "INSERT", "DELETE", "UPDATE", "TABLE", "FROM", "WHERE", "OR", "AND",
            "UNION", "ALTER", "CREATE", "INDEX", "EXEC", "EXECUTE", "INTO", "VALUES", "SET",
            "NULL", "TRUE", "FALSE",
        ];

        let upper = name.to_uppercase();
        if SQL_RESERVED.contains(&upper.as_str()) {
            return Err(format!(
                "Parse error at line {}: '{}' is a SQL reserved word and cannot be used as {}\n  Choose a different name",
                line, name, context
            ));
        }

        Ok(())
    }

    // ── helpers ──

    fn parse_array(&mut self) -> Result<Vec<String>, String> {
        self.expect(TokenKind::LBracket)?;
        let mut items = Vec::new();

        while !self.matches(TokenKind::RBracket, None) && !self.matches(TokenKind::Eof, None) {
            if self.peek().kind == TokenKind::Comma {
                self.advance();
                continue;
            }
            let token = self.peek().clone();
            // P040: validate unquoted identifiers in arrays (enum values, roles).
            // P041 is skipped on purpose: array values are data, not SQL identifiers.
            if token.kind == TokenKind::Identifier {
                Self::validate_identifier_pattern(&token.value, "enum value", token.line)?;
            }
            items.push(self.advance().value);
        }

        self.expect(TokenKind::RBracket)?;
        Ok(items)
    }

    fn parse_string_array(&mut self) -> Result<Vec<String>, String> {
        self.expect(TokenKind::LBracket)?;
        let mut items = Vec::new();

        while !self.matches(TokenKind::RBracket, None) && !self.matches(TokenKind::Eof, None) {
            if self.peek().kind == TokenKind::Comma {
                self.advance();
                continue;
            }
            items.push(self.advance().value);
        }

        self.expect(TokenKind::RBracket)?;
        Ok(items)
    }

    // ── deploy ──

    fn parse_deploy(&mut self) -> Result<DeployNode, String> {
        self.expect(TokenKind::Keyword)?; // consume "deploy"
        let mode = self.advance().value; // e.g. "microservices"
        self.expect(TokenKind::LBrace)?;

        let mut gateway = None;
        let mut services = Vec::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.matches(TokenKind::Identifier, Some("gateway")) {
                self.advance();
                let mut port: u16 = 0;
                // Parse inline colon pairs before brace (port:N)
                while self.peek().kind == TokenKind::ColonPair {
                    let (k, v) = Self::split_colon_pair(&self.advance().value);
                    if k == "port" {
                        port = v.parse().unwrap_or(0);
                    }
                }
                self.expect(TokenKind::LBrace)?;
                let mut provider = String::new();
                let mut cors = None;
                let mut config = HashMap::new();
                while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None)
                {
                    if self.matches(TokenKind::Identifier, Some("provider")) {
                        self.advance();
                        provider = self.advance().value;
                    } else if self.matches(TokenKind::Identifier, Some("cors")) {
                        self.advance();
                        cors = Some(self.advance().value);
                    } else if self.peek().kind == TokenKind::ColonPair {
                        let (k, v) = Self::split_colon_pair(&self.advance().value);
                        config.insert(k, v);
                    } else {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBrace)?;
                gateway = Some(GatewayConfig {
                    port,
                    provider,
                    cors,
                    config,
                });
            } else if self.matches(TokenKind::Keyword, Some("service"))
                || self.matches(TokenKind::Identifier, Some("service"))
            {
                self.advance();
                let name = self.advance().value; // service name (string or identifier)
                let mut port: u16 = 0;
                let mut db = None;
                // Parse inline colon pairs before brace (port:N db:"path")
                while self.peek().kind == TokenKind::ColonPair {
                    let (k, v) = Self::split_colon_pair(&self.advance().value);
                    match k.as_str() {
                        "port" => port = v.parse().unwrap_or(0),
                        "db" => db = Some(v),
                        _ => {}
                    }
                }
                self.expect(TokenKind::LBrace)?;
                let mut entities = Vec::new();
                let mut apis = Vec::new();
                let mut pages = Vec::new();
                let mut config = HashMap::new();
                while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None)
                {
                    if self.matches(TokenKind::Keyword, Some("entity"))
                        || self.matches(TokenKind::Identifier, Some("entities"))
                    {
                        self.advance();
                        entities = self.parse_string_array()?;
                    } else if self.matches(TokenKind::Keyword, Some("api"))
                        || self.matches(TokenKind::Identifier, Some("apis"))
                    {
                        self.advance();
                        apis = self.parse_string_array()?;
                    } else if self.matches(TokenKind::Identifier, Some("pages")) {
                        self.advance();
                        pages = self.parse_string_array()?;
                    } else if self.peek().kind == TokenKind::ColonPair {
                        let (k, v) = Self::split_colon_pair(&self.advance().value);
                        config.insert(k, v);
                    } else {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBrace)?;
                services.push(DeployServiceDef {
                    name,
                    port,
                    db,
                    entities,
                    apis,
                    pages,
                    config,
                });
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(DeployNode {
            mode,
            gateway,
            services,
        })
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
                if seen_app {
                    continue;
                }
                seen_app = true;
                deduped.push(node);
            }
            AstNode::Style(_) => {
                if seen_style {
                    continue;
                }
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
            if name.ends_with(".cronus") && entry.path().is_file() {
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
        let content =
            std::fs::read_to_string(file).map_err(|e| format!("Error reading {}: {}", file, e))?;
        combined.push_str(&format!("# [source: {}]\n", file));
        combined.push_str(&content);
        combined.push('\n');
    }

    parse_with_imports(&combined, dir)
}

/// Count entities, pages, api routes for quick stats.
pub fn stats(nodes: &[AstNode]) -> (usize, usize, usize) {
    let entities = nodes
        .iter()
        .filter(|n| matches!(n, AstNode::Entity(_)))
        .count();
    let pages = nodes
        .iter()
        .filter(|n| matches!(n, AstNode::Page(_)))
        .count();
    let api_routes: usize = nodes
        .iter()
        .filter_map(|n| {
            if let AstNode::Api(api) = n {
                Some(api.routes.len())
            } else {
                None
            }
        })
        .sum();
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
"#
    .to_string()
}

// ══════════════════════════════════════════════════
// TESTS — P040 / P041
// ══════════════════════════════════════════════════

#[cfg(test)]
mod parser_tests {
    use super::*;

    fn bind_filters(clause: &str) -> Vec<FilterExpr> {
        let src = format!(
            "page \"/p\" type:custom {{\n  section table {{\n    bind Task {{ query all where {} }}\n  }}\n}}\n",
            clause
        );
        let nodes = parse(&src).expect("parse");
        let AstNode::Page(p) = &nodes[0] else {
            panic!("expected page")
        };
        p.sections[0]
            .binding
            .as_ref()
            .expect("binding")
            .filters
            .clone()
    }

    fn describe(f: &FilterExpr) -> String {
        format!("{} {:?} {:?}", f.field, f.operator, f.value)
    }

    // ── `where x op:value` must equal `where x op value` ──

    #[test]
    fn where_colon_form_matches_space_form() {
        let cases = [
            ("owner eq:auth.id", "owner eq auth.id"),
            ("id eq:route.id", "id eq route.id"),
            ("status eq:\"active\"", "status eq \"active\""),
            ("count gt:5", "count gt 5"),
            ("count gte:5", "count gte 5"),
            ("count lt:5", "count lt 5"),
            ("count lte:5", "count lte 5"),
            ("status ne:\"done\"", "status ne \"done\""),
            ("done eq:true", "done eq true"),
            ("title contains:\"a b\"", "title contains \"a b\""),
            ("title starts_with:abc", "title starts_with abc"),
        ];
        for (colon, spaced) in cases {
            let a = bind_filters(colon);
            let b = bind_filters(spaced);
            assert_eq!(a.len(), 1, "{colon}");
            assert_eq!(describe(&a[0]), describe(&b[0]), "{colon} vs {spaced}");
        }
    }

    #[test]
    fn aggregate_colon_forms_keep_field_group_and_interval() {
        let src = "page \"/p\" type:custom {\n  section chart {\n    bind Order { aggregate sum field:total group_by:created_at interval:month }\n  }\n  section kpi {\n    bind Order { aggregate count }\n  }\n}\n";
        let nodes = parse(src).expect("parse");
        let AstNode::Page(p) = &nodes[0] else {
            panic!("expected page")
        };
        let chart = p.sections[0].binding.as_ref().unwrap();
        let agg = chart.aggregate.as_ref().unwrap();
        assert_eq!(agg.function, "sum");
        assert_eq!(agg.field.as_deref(), Some("total"));
        let group = chart.group_by.as_ref().expect("group_by");
        assert_eq!(group.field, "created_at");
        assert_eq!(group.interval.as_deref(), Some("month"));
        let kpi = p.sections[1].binding.as_ref().unwrap();
        assert_eq!(kpi.aggregate.as_ref().unwrap().function, "count");
        assert!(kpi.group_by.is_none());
    }

    #[test]
    fn where_colon_form_keeps_value_and_following_keywords() {
        let f = bind_filters("_owner_id eq:auth.id order created_at desc");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].field, "_owner_id");
        assert_eq!(f[0].operator, FilterOp::Eq);
        assert!(matches!(&f[0].value, BindingValue::AuthRef(r) if r == "auth.id"));

        let f = bind_filters("status eq:\"active\"");
        assert!(matches!(&f[0].value, BindingValue::Str(s) if s == "active"));
        let f = bind_filters("n gt:5");
        assert_eq!(f[0].operator, FilterOp::Gt);
        assert!(matches!(&f[0].value, BindingValue::Num(n) if n == "5"));
        let f = bind_filters("status neq:\"x\"");
        assert_eq!(f[0].operator, FilterOp::Ne);
    }

    // ── P041 does not apply to enum values (data, not identifiers) ──

    #[test]
    fn p041_enum_values_may_be_sql_reserved_words() {
        let src = "entity Audit {\n  action enum [create, update, delete, select]\n}\n";
        let nodes = parse(src).expect("SQL keywords are legal enum values");
        let AstNode::Entity(e) = &nodes[0] else {
            panic!("expected entity")
        };
        let values = e.fields[0].enum_values.as_ref().unwrap();
        assert_eq!(values, &vec!["create", "update", "delete", "select"]);
    }

    #[test]
    fn p041_still_rejects_reserved_field_and_entity_names() {
        assert!(parse("entity Thing {\n  select string\n}\n").is_err());
        assert!(parse("entity Table {\n  name string\n}\n").is_err());
    }

    #[test]
    fn p040_still_applies_to_enum_values() {
        assert!(Parser::validate_identifier_pattern("9lives", "enum value", 1).is_err());
    }

    /// Every shipped `.cronus` under `templates/` and `demos/` must parse.
    #[test]
    fn every_template_and_demo_parses() {
        use std::path::{Path, PathBuf};
        fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if name == "node_modules" || name == "target" || name.starts_with('.') {
                        continue;
                    }
                    collect(&p, out);
                } else if p.extension().and_then(|e| e.to_str()) == Some("cronus") {
                    out.push(p);
                }
            }
        }
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files = Vec::new();
        collect(&root.join("templates"), &mut files);
        collect(&root.join("demos"), &mut files);
        files.sort();
        assert!(
            !files.is_empty(),
            "no .cronus files found under templates/ or demos/"
        );

        let mut offenders = Vec::new();
        for file in &files {
            let src = std::fs::read_to_string(file).unwrap();
            let base = file.parent().unwrap().to_string_lossy().to_string();
            if let Err(e) = parse_with_imports(&src, &base) {
                let rel = file.strip_prefix(root).unwrap_or(file);
                offenders.push(format!(
                    "{}: {}",
                    rel.display(),
                    e.lines().next().unwrap_or("")
                ));
            }
        }
        assert!(
            offenders.is_empty(),
            "{} of {} .cronus files fail to parse:\n  {}",
            offenders.len(),
            files.len(),
            offenders.join("\n  ")
        );
    }

    // ── P040: Valid identifier pattern ──

    #[test]
    fn p040_valid_identifiers() {
        assert!(Parser::validate_identifier("name", "test", 1).is_ok());
        assert!(Parser::validate_identifier("User", "test", 1).is_ok());
        assert!(Parser::validate_identifier("deploy_id", "test", 1).is_ok());
        assert!(Parser::validate_identifier("status", "test", 1).is_ok());
        assert!(Parser::validate_identifier("myField2", "test", 1).is_ok());
        assert!(Parser::validate_identifier("A", "test", 1).is_ok());
        // 64 chars exactly — should pass
        let long = "a".repeat(64);
        assert!(Parser::validate_identifier(&long, "test", 1).is_ok());
    }

    #[test]
    fn p040_reject_starts_with_number() {
        let err = Parser::validate_identifier("2bad", "entity field", 5).unwrap_err();
        assert!(err.contains("Invalid identifier"), "got: {}", err);
        assert!(err.contains("2bad"));
    }

    #[test]
    fn p040_reject_special_chars() {
        let err = Parser::validate_identifier("status;DROP", "entity field", 1).unwrap_err();
        assert!(err.contains("Invalid identifier"), "got: {}", err);
    }

    #[test]
    fn p040_reject_dash_in_identifier() {
        let err = Parser::validate_identifier("my-field", "entity field", 1).unwrap_err();
        assert!(err.contains("Invalid identifier"), "got: {}", err);
    }

    #[test]
    fn p040_reject_too_long() {
        let long = "a".repeat(65);
        let err = Parser::validate_identifier(&long, "test", 1).unwrap_err();
        assert!(err.contains("exceeds 64 characters"), "got: {}", err);
    }

    #[test]
    fn p040_reject_empty() {
        let err = Parser::validate_identifier("", "test", 1).unwrap_err();
        assert!(err.contains("Empty identifier"), "got: {}", err);
    }

    // ── P041: SQL reserved words ──

    #[test]
    fn p041_reject_sql_reserved_words() {
        let reserved = vec![
            "SELECT", "DROP", "INSERT", "DELETE", "UPDATE", "TABLE", "FROM", "WHERE", "OR", "AND",
            "UNION", "ALTER", "CREATE", "INDEX", "EXEC", "EXECUTE", "INTO", "VALUES", "SET",
            "NULL", "TRUE", "FALSE",
        ];
        for word in reserved {
            let err = Parser::validate_identifier(word, "entity name", 1).unwrap_err();
            assert!(
                err.contains("SQL reserved word"),
                "Expected rejection for '{}', got: {}",
                word,
                err
            );
        }
    }

    #[test]
    fn p041_reject_case_insensitive() {
        let err = Parser::validate_identifier("select", "entity name", 1).unwrap_err();
        assert!(err.contains("SQL reserved word"), "got: {}", err);
        let err = Parser::validate_identifier("Drop", "entity name", 1).unwrap_err();
        assert!(err.contains("SQL reserved word"), "got: {}", err);
    }

    #[test]
    fn p041_allow_non_reserved_common_names() {
        // These are common field names that must NOT be rejected
        assert!(Parser::validate_identifier("name", "test", 1).is_ok());
        assert!(Parser::validate_identifier("status", "test", 1).is_ok());
        assert!(Parser::validate_identifier("type", "test", 1).is_ok());
        assert!(Parser::validate_identifier("value", "test", 1).is_ok());
        assert!(Parser::validate_identifier("email", "test", 1).is_ok());
        assert!(Parser::validate_identifier("User", "test", 1).is_ok());
        assert!(Parser::validate_identifier("Deployment", "test", 1).is_ok());
    }

    // ── Integration: parse rejects bad entity names ──

    #[test]
    fn p040_parse_rejects_bad_entity_name() {
        let source = r#"entity 123Bad { name string }"#;
        let result = parse(source);
        assert!(result.is_err(), "Should reject entity with invalid name");
    }

    #[test]
    fn p041_parse_rejects_reserved_entity_name() {
        let source = r#"entity SELECT { name string }"#;
        let result = parse(source);
        assert!(result.is_err(), "Should reject entity named SELECT");
        if let Err(err) = result {
            assert!(err.contains("SQL reserved word"), "got: {}", err);
        }
    }

    #[test]
    fn parse_accepts_valid_entity() {
        let source = r#"entity User { name string required email email unique }"#;
        let result = parse(source);
        if let Err(ref e) = result {
            panic!("Valid entity should parse, got error: {}", e);
        }
    }

    #[test]
    fn parse_bang_required_syntax() {
        let source = r#"entity User { name string! }"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            let field = &e.fields[0];
            assert_eq!(field.name, "name");
            assert_eq!(field.field_type, FieldType::String);
            assert!(field.required, "string! should set required=true");
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn parse_bang_with_other_modifiers() {
        let source = r#"entity User { email email! unique }"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            let field = &e.fields[0];
            assert_eq!(field.name, "email");
            assert_eq!(field.field_type, FieldType::Email);
            assert!(field.required, "email! should set required=true");
            assert!(field.unique, "unique modifier should still work after !");
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn parse_bang_backward_compat_required() {
        let source = r#"entity User { name string required }"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            assert!(
                e.fields[0].required,
                "old 'required' keyword should still work"
            );
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn parse_default_value_string() {
        let source = r#"entity User { role enum ["admin", "user"] default:"user" }"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            let field = &e.fields[0];
            assert_eq!(field.name, "role");
            assert_eq!(field.default_value, Some("user".to_string()));
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn parse_default_value_number() {
        let source = r#"entity Product { stock number default:0 }"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            let field = &e.fields[0];
            assert_eq!(field.name, "stock");
            assert_eq!(field.field_type, FieldType::Number);
            assert_eq!(field.default_value, Some("0".to_string()));
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn parse_bang_with_default() {
        let source = r#"entity User { role enum ["admin", "user"]! default:"user" }"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            let field = &e.fields[0];
            assert_eq!(field.name, "role");
            assert!(field.required, "! should set required");
            assert_eq!(field.default_value, Some("user".to_string()));
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn parse_no_default_value() {
        let source = r#"entity User { name string required }"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            assert_eq!(e.fields[0].default_value, None);
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn parse_number_min_max_constraints() {
        let source = r#"entity Product { age number min:0 max:150 }"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            let field = &e.fields[0];
            assert_eq!(field.name, "age");
            assert_eq!(field.min, Some(0.0));
            assert_eq!(field.max, Some(150.0));
            assert!(field.min_length.is_none());
            assert!(field.max_length.is_none());
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn parse_string_min_max_as_length() {
        let source = r#"entity User { username string min:3 max:30 }"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            let field = &e.fields[0];
            assert_eq!(field.name, "username");
            assert!(
                field.min.is_none(),
                "string field should not have numeric min"
            );
            assert!(
                field.max.is_none(),
                "string field should not have numeric max"
            );
            assert_eq!(field.min_length, Some(3));
            assert_eq!(field.max_length, Some(30));
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn parse_match_pattern() {
        let source = r#"entity User { username string! match:"^[a-z0-9_]+$" }"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            let field = &e.fields[0];
            assert_eq!(field.name, "username");
            assert!(field.required);
            assert_eq!(field.pattern, Some("^[a-z0-9_]+$".to_string()));
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn parse_money_min_constraint() {
        let source = r#"entity Product { price money! min:0 }"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            let field = &e.fields[0];
            assert_eq!(field.name, "price");
            assert!(field.required);
            assert_eq!(field.min, Some(0.0));
            assert!(field.max.is_none());
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn parse_text_max_length() {
        let source = r#"entity Post { description text max:5000 }"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            let field = &e.fields[0];
            assert_eq!(field.name, "description");
            assert_eq!(field.max_length, Some(5000));
            assert!(field.min_length.is_none());
        } else {
            panic!("Expected entity node");
        }
    }

    // ── Transition block tests ──

    #[test]
    fn transition_basic_parsing() {
        let source = r#"
entity Order {
  status enum ["draft", "pending", "paid"]! default:"draft"

  transition status {
    draft   -> pending
    pending -> paid
  }
}
"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            assert_eq!(e.transitions.len(), 1);
            let t = &e.transitions[0];
            assert_eq!(t.field, "status");
            assert_eq!(t.rules.len(), 2);
            assert_eq!(t.rules[0].from, "draft");
            assert_eq!(t.rules[0].to, vec!["pending"]);
            assert_eq!(t.rules[1].from, "pending");
            assert_eq!(t.rules[1].to, vec!["paid"]);
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn transition_multiple_targets_with_pipe() {
        let source = r#"
entity Order {
  status enum ["draft", "pending", "paid", "cancelled"]! default:"draft"

  transition status {
    draft   -> pending
    pending -> paid | cancelled
  }
}
"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            let t = &e.transitions[0];
            assert_eq!(t.rules[1].from, "pending");
            assert_eq!(t.rules[1].to, vec!["paid", "cancelled"]);
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn transition_entity_without_transition_works() {
        let source = r#"entity User { name string! }"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            assert!(e.transitions.is_empty());
            assert_eq!(e.fields.len(), 1);
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn transition_invalid_from_state() {
        let source = r#"
entity Order {
  status enum ["draft", "pending"]! default:"draft"

  transition status {
    nonexistent -> pending
  }
}
"#;
        let err = parse(source).err().expect("Expected parse error");
        assert!(
            err.contains("nonexistent"),
            "Error should mention invalid state: {}",
            err
        );
        assert!(
            err.contains("not a valid value"),
            "Error should explain it's not valid: {}",
            err
        );
    }

    #[test]
    fn transition_invalid_to_state() {
        let source = r#"
entity Order {
  status enum ["draft", "pending"]! default:"draft"

  transition status {
    draft -> nonexistent
  }
}
"#;
        let err = parse(source).err().expect("Expected parse error");
        assert!(
            err.contains("nonexistent"),
            "Error should mention invalid target: {}",
            err
        );
    }

    #[test]
    fn transition_field_does_not_exist() {
        let source = r#"
entity Order {
  name string!

  transition status {
    draft -> pending
  }
}
"#;
        let err = parse(source).err().expect("Expected parse error");
        assert!(
            err.contains("unknown field"),
            "Error should mention unknown field: {}",
            err
        );
        assert!(
            err.contains("status"),
            "Error should mention 'status': {}",
            err
        );
    }

    #[test]
    fn transition_on_non_enum_field() {
        let source = r#"
entity Order {
  status string!

  transition status {
    draft -> pending
  }
}
"#;
        let err = parse(source).err().expect("Expected parse error");
        assert!(
            err.contains("must be an enum"),
            "Error should require enum type: {}",
            err
        );
    }

    #[test]
    fn transition_full_order_example() {
        let source = r#"
entity Order {
  status enum ["draft", "pending", "paid", "shipped", "delivered", "cancelled"]! default:"draft"

  transition status {
    draft     -> pending
    pending   -> paid | cancelled
    paid      -> shipped | cancelled
    shipped   -> delivered
  }
}
"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            assert_eq!(e.fields.len(), 1);
            assert_eq!(e.transitions.len(), 1);
            let t = &e.transitions[0];
            assert_eq!(t.rules.len(), 4);
            assert_eq!(t.rules[0].from, "draft");
            assert_eq!(t.rules[0].to, vec!["pending"]);
            assert_eq!(t.rules[1].from, "pending");
            assert_eq!(t.rules[1].to, vec!["paid", "cancelled"]);
            assert_eq!(t.rules[2].from, "paid");
            assert_eq!(t.rules[2].to, vec!["shipped", "cancelled"]);
            assert_eq!(t.rules[3].from, "shipped");
            assert_eq!(t.rules[3].to, vec!["delivered"]);
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn transition_multiple_pipe_targets() {
        let source = r#"
entity Ticket {
  priority enum ["low", "medium", "high", "critical"]! default:"low"

  transition priority {
    low -> medium | high | critical
  }
}
"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            let t = &e.transitions[0];
            assert_eq!(t.rules[0].to, vec!["medium", "high", "critical"]);
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn transition_invalid_pipe_target() {
        let source = r#"
entity Order {
  status enum ["draft", "pending", "paid"]! default:"draft"

  transition status {
    draft -> pending | nonexistent
  }
}
"#;
        let err = parse(source).err().expect("Expected parse error");
        assert!(
            err.contains("nonexistent"),
            "Error should mention invalid pipe target: {}",
            err
        );
    }

    // ══════════════════════════════════════════════════
    // Effect block tests
    // ══════════════════════════════════════════════════

    #[test]
    fn effect_on_create_basic() {
        let source = r#"
entity Deployment shared {
  deploy_id string!
  cluster string

  on create {
    log "Deploy {{deploy_id}} started"
  }
}
"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            assert_eq!(e.effects.len(), 1);
            assert_eq!(e.effects[0].event, "create");
            assert!(e.effects[0].field.is_none());
            assert_eq!(e.effects[0].actions.len(), 1);
            assert_eq!(e.effects[0].actions[0].action_type, "log");
            assert_eq!(
                e.effects[0].actions[0].args[0],
                "Deploy {{deploy_id}} started"
            );
            assert!(e.effects[0].actions[0].condition.is_none());
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn effect_on_delete_basic() {
        let source = r#"
entity Item shared {
  name string!

  on delete {
    log "Item {{name}} removed"
  }
}
"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            assert_eq!(e.effects.len(), 1);
            assert_eq!(e.effects[0].event, "delete");
            assert!(e.effects[0].field.is_none());
            assert_eq!(e.effects[0].actions[0].action_type, "log");
            assert_eq!(e.effects[0].actions[0].args[0], "Item {{name}} removed");
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn effect_on_update_field_with_when() {
        let source = r#"
entity Deployment shared {
  deploy_id string!
  status enum ["Pending", "Rolling", "Live", "Failed"]! default:"Pending"

  on update status {
    when "Failed" {
      log "ALERT: {{deploy_id}} failed"
    }
    when "Live" {
      log "{{deploy_id}} is live"
    }
  }
}
"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            assert_eq!(e.effects.len(), 1);
            let eff = &e.effects[0];
            assert_eq!(eff.event, "update");
            assert_eq!(eff.field, Some("status".to_string()));
            assert_eq!(eff.actions.len(), 2);
            assert_eq!(eff.actions[0].condition, Some("Failed".to_string()));
            assert_eq!(eff.actions[0].args[0], "ALERT: {{deploy_id}} failed");
            assert_eq!(eff.actions[1].condition, Some("Live".to_string()));
            assert_eq!(eff.actions[1].args[0], "{{deploy_id}} is live");
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn effect_notify_action() {
        let source = r##"
entity Deploy shared {
  deploy_id string!
  cluster string

  on create {
    notify "slack" "#deploys" "New deploy: {{deploy_id}}"
  }
}
"##;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            let action = &e.effects[0].actions[0];
            assert_eq!(action.action_type, "notify");
            assert_eq!(action.args.len(), 3);
            assert_eq!(action.args[0], "slack");
            assert_eq!(action.args[1], "#deploys");
            assert_eq!(action.args[2], "New deploy: {{deploy_id}}");
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn effect_multiple_blocks() {
        let source = r#"
entity Order shared {
  order_id string!
  status enum ["new", "shipped", "delivered"]! default:"new"

  on create {
    log "Order {{order_id}} created"
  }

  on update status {
    when "shipped" {
      log "Order {{order_id}} shipped"
    }
  }

  on delete {
    log "Order {{order_id}} deleted"
  }
}
"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            assert_eq!(e.effects.len(), 3);
            assert_eq!(e.effects[0].event, "create");
            assert_eq!(e.effects[1].event, "update");
            assert_eq!(e.effects[1].field, Some("status".to_string()));
            assert_eq!(e.effects[2].event, "delete");
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn effect_mixed_actions_in_when() {
        let source = r#"
entity Deploy shared {
  deploy_id string!
  cluster string
  status enum ["Pending", "Failed"]! default:"Pending"

  on update status {
    when "Failed" {
      notify "pagerduty" "critical" "Deploy {{deploy_id}} FAILED"
      log "ALERT: {{deploy_id}} failed"
    }
  }
}
"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            let eff = &e.effects[0];
            assert_eq!(eff.actions.len(), 2);
            assert_eq!(eff.actions[0].action_type, "notify");
            assert_eq!(eff.actions[0].condition, Some("Failed".to_string()));
            assert_eq!(eff.actions[0].args[0], "pagerduty");
            assert_eq!(eff.actions[1].action_type, "log");
            assert_eq!(eff.actions[1].condition, Some("Failed".to_string()));
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn effect_coexists_with_transition() {
        let source = r#"
entity Order shared {
  status enum ["draft", "pending", "paid"]! default:"draft"

  transition status {
    draft -> pending
    pending -> paid
  }

  on create {
    log "Order created"
  }

  on update status {
    when "paid" {
      log "Order paid"
    }
  }
}
"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            assert_eq!(e.transitions.len(), 1);
            assert_eq!(e.transitions[0].rules.len(), 2);
            assert_eq!(e.effects.len(), 2);
            assert_eq!(e.effects[0].event, "create");
            assert_eq!(e.effects[1].event, "update");
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn effect_invalid_event_type() {
        let source = r#"
entity Item shared {
  name string!

  on explode {
    log "boom"
  }
}
"#;
        let err = parse(source)
            .err()
            .expect("Expected parse error for invalid effect event");
        assert!(
            err.contains("invalid effect event"),
            "Error should mention invalid event: {}",
            err
        );
        assert!(
            err.contains("explode"),
            "Error should mention 'explode': {}",
            err
        );
    }

    #[test]
    fn effect_entity_without_effects() {
        let source = r#"entity User { name string! email email! unique }"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            assert!(e.effects.is_empty());
        } else {
            panic!("Expected entity node");
        }
    }

    #[test]
    fn effect_on_update_without_field() {
        let source = r#"
entity Item shared {
  name string!
  price number

  on update {
    log "Item {{name}} was updated"
  }
}
"#;
        let ast = parse(source).unwrap();
        if let AstNode::Entity(ref e) = ast[0] {
            assert_eq!(e.effects.len(), 1);
            assert_eq!(e.effects[0].event, "update");
            assert!(
                e.effects[0].field.is_none(),
                "on update without field should have field=None"
            );
            assert_eq!(e.effects[0].actions[0].args[0], "Item {{name}} was updated");
        } else {
            panic!("Expected entity node");
        }
    }

    // ── 2026-04-10: `nav` keyword is optional in layout sidebar items ──

    #[test]
    fn layout_sidebar_nav_optional() {
        let source =
            r#"layout Main { sidebar { brand "Acme" "Home" -> "/" nav "About" -> "/about" } }"#;
        let ast = parse(source).expect("layout with mixed nav/no-nav sidebar items should parse");

        let layout = ast
            .iter()
            .find_map(|node| {
                if let AstNode::Layout(l) = node {
                    Some(l)
                } else {
                    None
                }
            })
            .expect("expected a Layout node in AST");

        assert_eq!(layout.name, "Main");
        assert_eq!(
            layout.sidebar_config.get("brand").map(String::as_str),
            Some("Acme")
        );
        assert_eq!(
            layout.sidebar_items.len(),
            2,
            "expected 2 sidebar items (one without `nav` keyword, one with), got {:?}",
            layout.sidebar_items
        );

        assert_eq!(layout.sidebar_items[0].label, "Home");
        assert_eq!(layout.sidebar_items[0].route, "/");
        assert!(!layout.sidebar_items[0].is_divider);

        assert_eq!(layout.sidebar_items[1].label, "About");
        assert_eq!(layout.sidebar_items[1].route, "/about");
        assert!(!layout.sidebar_items[1].is_divider);
    }

    // ── 2026-04-10: api rejects unknown HTTP methods (HEAD/OPTIONS/etc.) ──

    #[test]
    fn api_rejects_unknown_http_method() {
        // HEAD is NOT in tokenizer METHODS, so it tokenizes as Identifier and
        // `expect(TokenKind::Method)` in parse_api must reject it cleanly.
        let source = r#"app "T" { port 5175 } entity Item { name string! } api /items { check HEAD / list GET / }"#;
        let err = match parse(source) {
            Err(e) => e,
            Ok(_) => panic!("api route with HEAD method should fail to parse, got Ok"),
        };
        assert!(
            err.contains("HEAD"),
            "error should mention the offending token 'HEAD', got: {}",
            err
        );
        assert!(
            err.contains("Method"),
            "error should mention expected kind 'Method', got: {}",
            err
        );
    }

    #[test]
    fn same_line_fields_are_two_fields() {
        let ast = parse(r#"entity Message { author string! note text! }"#).unwrap();
        let AstNode::Entity(e) = &ast[0] else {
            panic!("entity")
        };
        assert_eq!(
            e.fields.len(),
            2,
            "got {:?}",
            e.fields.iter().map(|f| &f.name).collect::<Vec<_>>()
        );
        assert_eq!(e.fields[0].name, "author");
        assert_eq!(e.fields[1].name, "note");
        assert!(e.fields[0].required);
        assert!(e.fields[1].required);
    }

    #[test]
    fn email_field_named_email_stays_one_field() {
        let ast = parse(
            r#"
entity Lead {
  name string! searchable
  email email! unique
  company string
}
"#,
        )
        .unwrap();
        let AstNode::Entity(e) = &ast[0] else {
            panic!("entity")
        };
        let names: Vec<_> = e.fields.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["name", "email", "company"], "got {names:?}");
        assert!(e.fields[1].unique);
        assert!(e.fields[1].required);
    }

    #[test]
    fn bind_scope_public() {
        let ast = parse(
            r#"
page "/" {
  section kpi {
    bind Node { query count scope:public }
    item "Nodes" value:bind
  }
}
"#,
        )
        .unwrap();
        let AstNode::Page(p) = &ast[0] else {
            panic!("page")
        };
        let b = p.sections[0].binding.as_ref().expect("binding");
        assert!(b.public);
        assert_eq!(b.entity, "Node");
        assert!(matches!(b.query, QueryType::Count));
    }

    #[test]
    fn page_use_component_is_recorded() {
        let ast = parse(
            r#"
component Save layout:inline style:button+primary+md { label "Save" }
page "/" type:custom {
  title "Home"
  use Save
}
"#,
        )
        .unwrap();
        let page = ast
            .iter()
            .find_map(|n| {
                if let AstNode::Page(p) = n {
                    Some(p)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(page.components, vec!["Save".to_string()]);
    }

    #[test]
    fn auth_redirect_is_stored() {
        let ast = parse(
            r#"
auth {
  entity User
  login email + password
  session jwt expires:24h
  redirect "/"
}
"#,
        )
        .unwrap();
        let AstNode::Auth(a) = &ast[0] else {
            panic!("auth")
        };
        assert_eq!(
            a.session_config.get("redirect").map(String::as_str),
            Some("/")
        );
    }
}
