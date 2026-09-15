#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Language Parser — Rust Native
//!
//! Parses .cronus files into an AST (Vec<AstNode>).
//! Zero external dependencies for parsing.

pub mod ast;
pub use ast::*;

pub mod diagnostic;
use diagnostic::codes;
pub use diagnostic::ParseError;

pub(crate) mod tokenizer;
pub(crate) use tokenizer::*;

use std::collections::HashMap;

// ══════════════════════════════════════════════════
// PARSER
// ══════════════════════════════════════════════════

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    /// Recoverable diagnostics (e.g. unknown field types). Parsing continues
    /// so one `build` reports all of them; any entry makes the parse fail.
    diagnostics: Vec<ParseError>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            diagnostics: Vec::new(),
        }
    }

    /// Synthetic EOF positioned at the end of the real token stream.
    fn eof_token(&self) -> Token {
        let last = self.tokens.last();
        Token {
            kind: TokenKind::Eof,
            value: String::new(),
            line: last.map(|t| t.line).unwrap_or(1),
            col: last.map(|t| t.col + t.width()).unwrap_or(1),
        }
    }

    fn describe_kind(kind: &TokenKind) -> &'static str {
        match kind {
            TokenKind::Keyword => "a keyword",
            TokenKind::Identifier => "an identifier",
            TokenKind::StringLit => "a string literal",
            TokenKind::Number => "a number",
            TokenKind::LBrace => "'{'",
            TokenKind::RBrace => "'}'",
            TokenKind::LBracket => "'['",
            TokenKind::RBracket => "']'",
            TokenKind::LParen => "'('",
            TokenKind::RParen => "')'",
            TokenKind::Arrow => "'->'",
            TokenKind::ColonPair => "a key:value pair",
            TokenKind::Plus => "'+'",
            TokenKind::Comma => "','",
            TokenKind::Price => "a price",
            TokenKind::Method => "an HTTP method (GET, POST, PUT, PATCH, DELETE)",
            TokenKind::Path => "a path starting with '/'",
            TokenKind::EnvRef => "env(...)",
            TokenKind::Operator => "an operator",
            TokenKind::Pipe => "'|'",
            TokenKind::DocComment => "a doc comment",
            TokenKind::Eof => "end of file",
        }
    }

    fn unexpected(t: &Token, expected: &TokenKind) -> ParseError {
        let found = if t.kind == TokenKind::Eof {
            "end of file".to_string()
        } else {
            format!("'{}'", t.value)
        };
        let mut err = ParseError::new(
            codes::UNEXPECTED_TOKEN,
            format!(
                "expected {}, found {}",
                Self::describe_kind(expected),
                found
            ),
            t.line,
            t.col,
        )
        .with_len(t.width());
        if t.kind != TokenKind::Eof {
            err = err.with_target(t.value.clone());
        }
        if *expected == TokenKind::Method {
            let upper = t.value.to_uppercase();
            if METHODS.contains(&upper.as_str()) {
                err = err.with_replacement(upper);
            } else {
                err = err.with_hint(
                    "supported methods are GET, POST, PUT, PATCH, DELETE; HEAD and OPTIONS are handled by the runtime",
                );
            }
        }
        err
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
        self.tokens
            .get(self.pos)
            .cloned()
            .unwrap_or_else(|| self.eof_token())
    }

    fn advance(&mut self) -> Token {
        let t = self
            .tokens
            .get(self.pos)
            .cloned()
            .unwrap_or_else(|| self.eof_token());
        self.pos += 1;
        t
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        let t = self.advance();
        if t.kind != kind {
            return Err(Self::unexpected(&t, &kind));
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

    fn parse(&mut self) -> Result<Vec<AstNode>, ParseError> {
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

    fn parse_import(&mut self) -> Result<ImportNode, ParseError> {
        self.expect(TokenKind::Keyword)?;
        let alias = self.advance().value;
        if self.matches(TokenKind::Identifier, Some("from")) {
            self.advance();
        }
        let source = self.expect(TokenKind::StringLit)?.value;
        Ok(ImportNode { alias, source })
    }

    // ── compose ──

    fn parse_compose(&mut self) -> Result<ComposeNode, ParseError> {
        self.reject_unimplemented_block("compose");
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

    fn parse_app(&mut self) -> Result<AppNode, ParseError> {
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

    fn parse_entity(&mut self) -> Result<EntityNode, ParseError> {
        self.expect(TokenKind::Keyword)?;
        let name_token = self.peek().clone();
        let name = self.advance().value;

        // P040/P041: validate entity name
        Self::validate_identifier(&name, "entity name", (name_token.line, name_token.col))?;

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

    fn parse_field(&mut self) -> Result<Option<FieldNode>, ParseError> {
        let field_token = self.peek().clone();
        let name = self.advance().value;

        // P040/P041: validate field name
        Self::validate_identifier(&name, "entity field", (field_token.line, field_token.col))?;

        // relation: -> EntityName
        if self.matches(TokenKind::Arrow, None) {
            self.advance();
            let target = self.advance().value;
            // `-> Target[]` and same-line modifiers (`-> User required`).
            // Without this, `required` was parsed as a new field that took the
            // next line's field name as its type.
            let mut array = false;
            if self.matches(TokenKind::LBracket, None)
                && self
                    .tokens
                    .get(self.pos + 1)
                    .is_some_and(|t| t.kind == TokenKind::RBracket)
            {
                self.advance();
                self.advance();
                array = true;
            }
            let (mut required, mut unique, mut optional, mut index) = (false, false, false, false);
            while self.peek().line == field_token.line && self.peek().kind == TokenKind::Identifier
            {
                match self.peek().value.as_str() {
                    "required" | "!" => required = true,
                    "unique" => unique = true,
                    "optional" => optional = true,
                    "index" => index = true,
                    _ => break,
                }
                self.advance();
            }
            return Ok(Some(FieldNode {
                name,
                field_type: FieldType::Relation,
                required,
                unique,
                sensitive: false,
                optional,
                searchable: false,
                index,
                featured: false,
                formatted: false,
                array,
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

        // type — must sit on the same line as the field name
        let type_token = self.peek();
        if type_token.line != field_token.line
            || matches!(type_token.kind, TokenKind::RBrace | TokenKind::Eof)
        {
            self.diagnostics.push(
                ParseError::new(
                    codes::MISSING_FIELD_TYPE,
                    format!("field '{}' has no type", name),
                    field_token.line,
                    field_token.col,
                )
                .with_len(field_token.width())
                .with_target(name.clone())
                .with_hint(format!(
                    "write '{} <type>', e.g. '{} string!'; valid types: {}",
                    name,
                    name,
                    FIELD_TYPE_KEYWORDS.join(", ")
                )),
            );
            return Ok(None);
        }
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

        let field_type = match FieldType::from_keyword(&clean_type_str) {
            Some(t) => t,
            None => {
                self.diagnostics.push(Self::unknown_type_error(
                    &clean_type_str,
                    &name,
                    &type_token,
                ));
                FieldType::String
            }
        };
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
        let mut max_token: Option<Token> = None;

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
                            let is_type = FieldType::from_keyword(t).is_some();
                            if is_type || nxt.kind == TokenKind::Arrow {
                                break;
                            }
                        }
                    }
                }
                let mod_token = self.advance();
                let mod_val = mod_token.value.clone();
                // Check for colon-pair modifiers like default:"value"
                if mod_val.contains(':') {
                    let (k, v) = Self::split_colon_pair(&mod_val);
                    match k.as_str() {
                        "default" => {
                            default_value = Some(v);
                        }
                        "min" | "max" => match v.parse::<f64>() {
                            Ok(n) if n.is_finite() && k == "min" => min = Some(n),
                            Ok(n) if n.is_finite() => {
                                max = Some(n);
                                max_token = Some(mod_token.clone());
                            }
                            _ => self.diagnostics.push(
                                Self::token_error(
                                    codes::INVALID_BOUND,
                                    format!(
                                        "'{}:' of field '{}' must be a number, found '{}'",
                                        k, name, v
                                    ),
                                    &mod_token,
                                )
                                .with_hint(format!("write a number, e.g. '{}:0'", k)),
                            ),
                        },
                        "match" => {
                            if let Err(e) = regex::Regex::new(&v) {
                                let reason = e.to_string();
                                let reason = reason.lines().last().unwrap_or("").trim().to_string();
                                self.diagnostics.push(
                                    Self::token_error(
                                        codes::INVALID_PATTERN,
                                        format!(
                                            "'match:' of field '{}' is not a valid regular expression",
                                            name
                                        ),
                                        &mod_token,
                                    )
                                    .with_hint(format!(
                                        "fix the pattern ({}); it uses Rust regex syntax, without look-around or backreferences",
                                        reason
                                    )),
                                );
                            }
                            pattern = Some(v);
                        }
                        other => {
                            let shown = format!("{other}:");
                            let mut err = Self::token_error(
                                codes::UNKNOWN_FIELD_MODIFIER,
                                format!("unknown field modifier '{shown}' on field '{name}'"),
                                &mod_token,
                            )
                            .with_hint(
                                "recognised modifiers: required, unique, sensitive, optional, searchable, index, featured, formatted, default:, min:, max:, match:",
                            );
                            if other == "onupdate" || other == "computed" {
                                err =
                                    err.with_hint("this modifier is not in the language; drop it");
                            }
                            self.diagnostics.push(err);
                        }
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
                        other => {
                            let mut err = Self::token_error(
                                codes::UNKNOWN_FIELD_MODIFIER,
                                format!("unknown field modifier '{other}' on field '{name}'"),
                                &mod_token,
                            )
                            .with_hint(
                                "recognised modifiers: required, unique, sensitive, optional, searchable, index, featured, formatted, default:, min:, max:, match:",
                            );
                            if other == "indexed" {
                                err = err.with_replacement("index".to_string());
                            }
                            self.diagnostics.push(err);
                        }
                    }
                }
            } else {
                break;
            }
        }

        if let (Some(lo), Some(hi), Some(at)) = (min, max, &max_token) {
            if lo > hi {
                self.diagnostics.push(
                    Self::token_error(
                        codes::MIN_GREATER_THAN_MAX,
                        format!("field '{}' has min:{} greater than max:{}", name, lo, hi),
                        at,
                    )
                    .with_hint("min must be less than or equal to max"),
                );
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

    /// TYPE_001 with the closest valid type as a concrete replacement.
    fn unknown_type_error(type_str: &str, field: &str, at: &Token) -> ParseError {
        let aliases: Vec<&str> = FIELD_TYPE_ALIASES.iter().map(|(a, _)| *a).collect();
        let suggestion = diagnostic::closest(type_str, FIELD_TYPE_KEYWORDS).or_else(|| {
            diagnostic::closest(type_str, &aliases).and_then(|a| {
                FIELD_TYPE_ALIASES
                    .iter()
                    .find(|(alias, _)| *alias == a)
                    .map(|(_, c)| *c)
            })
        });
        let err = ParseError::new(
            codes::UNKNOWN_FIELD_TYPE,
            format!("unknown field type '{}' for field '{}'", type_str, field),
            at.line,
            at.col,
        )
        .with_len(type_str.chars().count())
        .with_target(type_str);
        match suggestion {
            Some(s) => err
                .with_replacement(s)
                .with_hint(format!("did you mean '{}'?", s)),
            None => err.with_hint(format!("valid types: {}", FIELD_TYPE_KEYWORDS.join(", "))),
        }
    }

    fn token_error(code: &'static str, message: String, at: &Token) -> ParseError {
        ParseError::new(code, message, at.line, at.col)
            .with_len(at.width())
            .with_target(at.value.clone())
    }

    /// `service` / `worker` / … parse so the rest of the file can be diagnosed,
    /// but `cronus build` treats them as errors: they have no runtime.
    fn reject_unimplemented_block(&mut self, kind: &'static str) {
        let at = self.peek().clone();
        self.diagnostics.push(
            Self::token_error(
                codes::UNIMPLEMENTED_BLOCK,
                format!("top-level '{kind}' is not implemented"),
                &at,
            )
            .with_hint(format!(
                "remove the '{kind}' block; the parser accepts it so the rest of the file can be checked, but the runtime ignores it"
            )),
        );
    }

    /// Transition state/target that is not an enum value.
    fn invalid_state_error(
        role: &str,
        value: &str,
        field: &str,
        enum_values: &[String],
        at: &Token,
    ) -> ParseError {
        let candidates: Vec<&str> = enum_values.iter().map(|s| s.as_str()).collect();
        let err = Self::token_error(
            codes::INVALID_TRANSITION_STATE,
            format!(
                "transition {} '{}' is not a valid value of enum field '{}' (valid values: {})",
                role,
                value,
                field,
                enum_values.join(", ")
            ),
            at,
        );
        match diagnostic::closest(value, &candidates) {
            Some(s) => err.with_replacement(s),
            None => err,
        }
    }

    // ── transition (state machine) ──

    fn parse_transition(&mut self, fields: &[FieldNode]) -> Result<TransitionNode, ParseError> {
        let kw_token = self.advance(); // consume "transition"
        let field_name_token = self.peek().clone();
        let field_name = self.advance().value;

        // Validate: field must exist in the entity
        let field = fields.iter().find(|f| f.name == field_name);
        let field = match field {
            Some(f) => f,
            None => {
                let names: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
                let err = Self::token_error(
                    codes::INVALID_TRANSITION_FIELD,
                    format!(
                        "transition references unknown field '{}' (transition blocks must come after the field they use)",
                        field_name
                    ),
                    &field_name_token,
                );
                return Err(match diagnostic::closest(&field_name, &names) {
                    Some(s) => err.with_replacement(s),
                    None => err,
                });
            }
        };

        // Validate: field must be an enum type
        if field.field_type != FieldType::Enum {
            return Err(Self::token_error(
                codes::INVALID_TRANSITION_FIELD,
                format!(
                    "transition field '{}' must be an enum type, got {:?}",
                    field_name, field.field_type
                ),
                &field_name_token,
            )
            .with_hint(format!("declare it as '{} enum [a, b, c]'", field_name)));
        }

        let enum_values = field.enum_values.as_ref().unwrap_or(&Vec::new()).clone();

        self.expect(TokenKind::LBrace)?;

        let mut rules = Vec::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            let from_token = self.peek().clone();
            let from = self.advance().value;

            // Validate: from state must exist in enum values
            if !enum_values.contains(&from) {
                return Err(Self::invalid_state_error(
                    "state",
                    &from,
                    &field_name,
                    &enum_values,
                    &from_token,
                ));
            }

            self.expect(TokenKind::Arrow)?;

            let mut to = Vec::new();
            let first_to_token = self.peek().clone();
            let first_target = self.advance().value;

            // Validate first target
            if !enum_values.contains(&first_target) {
                return Err(Self::invalid_state_error(
                    "target",
                    &first_target,
                    &field_name,
                    &enum_values,
                    &first_to_token,
                ));
            }
            to.push(first_target);

            // Parse additional targets separated by |
            while self.matches(TokenKind::Pipe, None) {
                self.advance(); // consume |
                let target_token = self.peek().clone();
                let target = self.advance().value;

                if !enum_values.contains(&target) {
                    return Err(Self::invalid_state_error(
                        "target",
                        &target,
                        &field_name,
                        &enum_values,
                        &target_token,
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

    fn parse_effect_block(&mut self) -> Result<EffectBlock, ParseError> {
        let on_token = self.advance(); // consume "on"
        let event_token = self.peek().clone();
        let event = self.advance().value.to_lowercase(); // create, update, delete

        const EVENTS: &[&str] = &["create", "update", "delete"];
        if !EVENTS.contains(&event.as_str()) {
            let err = Self::token_error(
                codes::INVALID_EFFECT_EVENT,
                format!(
                    "invalid effect event '{}', expected 'create', 'update', or 'delete'",
                    event
                ),
                &event_token,
            );
            return Err(match diagnostic::closest(&event, EVENTS) {
                Some(s) => err.with_replacement(s),
                None => err,
            });
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

    fn parse_api(&mut self) -> Result<ApiNode, ParseError> {
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

    fn parse_webhook(&mut self) -> Result<WebhookNode, ParseError> {
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

    fn parse_auth(&mut self) -> Result<AuthNode, ParseError> {
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

    fn parse_layout(&mut self) -> Result<LayoutNode, ParseError> {
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

    fn parse_page(&mut self) -> Result<PageNode, ParseError> {
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

    fn parse_section(&mut self) -> Result<SectionNode, ParseError> {
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

    fn parse_section_item(&mut self) -> Result<HashMap<String, String>, ParseError> {
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

    const ACTION_VERBS: &'static [&'static str] = &[
        "set", "toast", "navigate", "refresh", "create", "update", "delete", "validate", "confirm",
        "open", "close",
    ];

    /// Optional argument of an action verb. Never consumes `{`, `}`, end of
    /// file or the next verb — a bare `refresh` before `}` used to swallow the
    /// block's closing brace and nest every following page inside this one.
    fn action_arg(&mut self) -> Option<String> {
        let t = self.peek();
        let is_verb = matches!(t.kind, TokenKind::Identifier | TokenKind::Keyword)
            && Self::ACTION_VERBS.contains(&t.value.as_str());
        if is_verb
            || matches!(
                t.kind,
                TokenKind::LBrace | TokenKind::RBrace | TokenKind::Eof
            )
        {
            return None;
        }
        Some(self.advance().value)
    }

    /// Field literals on `create`/`update`: `title:"x"` pairs and/or `{ title "x" }`.
    fn parse_action_fields(&mut self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        while self.peek().kind == TokenKind::ColonPair {
            let (k, v) = Self::split_colon_pair(&self.advance().value);
            fields.insert(k, v.trim_matches('"').to_string());
        }
        if !self.matches(TokenKind::LBrace, None) {
            return fields;
        }
        self.advance();
        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            let tok = self.advance();
            if tok.kind == TokenKind::ColonPair {
                let (k, v) = Self::split_colon_pair(&tok.value);
                fields.insert(k, v.trim_matches('"').to_string());
                continue;
            }
            let key = tok.value;
            let val = self.action_arg().unwrap_or_default();
            fields.insert(key, val.trim_matches('"').to_string());
        }
        if self.matches(TokenKind::RBrace, None) {
            self.advance();
        }
        fields
    }

    fn parse_action_block(&mut self) -> Result<ActionBlock, ParseError> {
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
                    let field = self.action_arg().unwrap_or_default();
                    let value = self.action_arg().unwrap_or_default();
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
                    // Documented form: `toast "Saved" success` (bare style, not a verb).
                    if let Some(style) = self.action_arg() {
                        mods.insert("style".into(), style);
                    }
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
                    // "/path", /path or back
                    let url = self.action_arg().unwrap_or_default();
                    instructions.push(ActionInstruction {
                        verb: "navigate".into(),
                        target: url,
                        value: String::new(),
                        modifiers: HashMap::new(),
                    });
                }
                "refresh" => {
                    // "self" (default), "parent", "page"
                    let target = self.action_arg().unwrap_or_else(|| "self".into());
                    instructions.push(ActionInstruction {
                        verb: "refresh".into(),
                        target,
                        value: String::new(),
                        modifiers: HashMap::new(),
                    });
                }
                "create" | "update" => {
                    let target = self.action_arg().unwrap_or_default();
                    let modifiers = self.parse_action_fields();
                    instructions.push(ActionInstruction {
                        verb: verb.clone(),
                        target,
                        value: String::new(),
                        modifiers,
                    });
                }
                "delete" => {
                    let target = self.action_arg().unwrap_or_default();
                    instructions.push(ActionInstruction {
                        verb: verb.clone(),
                        target,
                        value: String::new(),
                        modifiers: HashMap::new(),
                    });
                }
                "validate" => {
                    let at = self.tokens.get(self.pos.saturating_sub(1)).cloned();
                    if let Some(at) = at {
                        self.diagnostics.push(
                            Self::token_error(
                                codes::UNIMPLEMENTED_ACTION,
                                "action verb 'validate' is not implemented".into(),
                                &at,
                            )
                            .with_hint(
                                "remove it; field validation already runs on /_form and REST writes",
                            ),
                        );
                    }
                    let target = self.action_arg().unwrap_or_else(|| "all".into());
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
                    let at = self.tokens.get(self.pos.saturating_sub(1)).cloned();
                    if let Some(at) = at {
                        const LIVE: &[&str] = &[
                            "set", "toast", "navigate", "refresh", "delete", "open", "close",
                            "confirm", "create", "update",
                        ];
                        let mut err = Self::token_error(
                            codes::UNIMPLEMENTED_ACTION,
                            format!("unknown action verb '{verb}'"),
                            &at,
                        )
                        .with_hint(
                            "implemented verbs: set, toast, navigate, refresh, delete, open, close; create/update are parsed for forms",
                        );
                        if let Some(s) = diagnostic::closest(&verb, LIVE) {
                            err = err.with_replacement(s.to_string());
                        }
                        self.diagnostics.push(err);
                    }
                    // Skip to next known verb or closing brace so the rest of
                    // the block can still be diagnosed.
                    while !self.matches(TokenKind::RBrace, None)
                        && !self.matches(TokenKind::Eof, None)
                    {
                        let next = self.peek();
                        if matches!(next.kind, TokenKind::Identifier | TokenKind::Keyword)
                            && Self::ACTION_VERBS.contains(&next.value.as_str())
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

    fn parse_plan(&mut self) -> Result<PlanNode, ParseError> {
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

    fn parse_binding(&mut self) -> Result<BindingNode, ParseError> {
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
                    let qt_token = self.peek().clone();
                    let qt = self.advance().value;
                    query = match qt.as_str() {
                        "all" => QueryType::All,
                        "one" => QueryType::One,
                        "count" => QueryType::Count,
                        other => {
                            const KINDS: &[&str] = &["all", "one", "count"];
                            let mut err = Self::token_error(
                                codes::UNKNOWN_QUERY,
                                format!("unknown query '{other}'"),
                                &qt_token,
                            )
                            .with_hint("query must be all, one or count");
                            if let Some(s) = diagnostic::closest(other, KINDS) {
                                err = err.with_replacement(s.to_string());
                            }
                            self.diagnostics.push(err);
                            QueryType::All
                        }
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
                        "eq" => Some(FilterOp::Eq),
                        "ne" | "neq" => Some(FilterOp::Ne),
                        "gt" => Some(FilterOp::Gt),
                        "gte" => Some(FilterOp::Gte),
                        "lt" => Some(FilterOp::Lt),
                        "lte" => Some(FilterOp::Lte),
                        "contains" => Some(FilterOp::Contains),
                        "starts_with" => Some(FilterOp::StartsWith),
                        other => {
                            const OPS: &[&str] = &[
                                "eq",
                                "ne",
                                "neq",
                                "gt",
                                "gte",
                                "lt",
                                "lte",
                                "contains",
                                "starts_with",
                            ];
                            let mut err = Self::token_error(
                                codes::UNKNOWN_FILTER_OP,
                                format!("unknown where operator '{other}'"),
                                &op_token,
                            )
                            .with_hint(
                                "supported operators: eq, ne (neq), gt, gte, lt, lte, contains, starts_with",
                            );
                            if let Some(s) = diagnostic::closest(other, OPS) {
                                err = err.with_replacement(s.to_string());
                            }
                            self.diagnostics.push(err);
                            None
                        }
                    };
                    if let Some(operator) = op {
                        filters.push(FilterExpr {
                            field,
                            operator,
                            value,
                        });
                    }
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

    fn parse_style(&mut self) -> Result<StyleNode, ParseError> {
        self.expect(TokenKind::Keyword)?;
        self.expect(TokenKind::LBrace)?;

        let mut theme = None;
        let mut accent = None;
        let mut radius = None;
        let mut font = None;
        let mut config = HashMap::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            // `mode` is an alias of `theme` (`light` | `dark` | `system`).
            if self.matches(TokenKind::Identifier, Some("theme"))
                || self.matches(TokenKind::Identifier, Some("mode"))
            {
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

    fn parse_service(&mut self) -> Result<ServiceNode, ParseError> {
        self.reject_unimplemented_block("service");
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
    fn parse_define(&mut self) -> Result<DefineNode, ParseError> {
        self.reject_unimplemented_block("define");
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

    fn parse_component(&mut self) -> Result<ComponentNode, ParseError> {
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

    fn parse_event(&mut self) -> Result<EventNode, ParseError> {
        self.reject_unimplemented_block("on");
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

    fn parse_worker(&mut self) -> Result<WorkerNode, ParseError> {
        self.reject_unimplemented_block("worker");
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

    fn parse_middleware(&mut self) -> Result<MiddlewareNode, ParseError> {
        self.reject_unimplemented_block("middleware");
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

    /// `env { APP_KEY string! sensitive  APP_FLAG boolean default:false }`
    /// declares typed variables (checked by `cronus run`); the legacy
    /// `env name { KEY value }` form keeps plain pairs. A line is a
    /// declaration when the token after the name is a type keyword.
    fn parse_env(&mut self) -> Result<EnvNode, ParseError> {
        self.expect(TokenKind::Keyword)?;
        let name = if self.matches(TokenKind::LBrace, None) {
            String::new()
        } else {
            self.advance().value
        };
        self.expect(TokenKind::LBrace)?;

        let mut vars = HashMap::new();
        let mut schema = Vec::new();

        while !self.matches(TokenKind::RBrace, None) && !self.matches(TokenKind::Eof, None) {
            if self.peek().kind != TokenKind::Identifier {
                self.advance();
                continue;
            }
            let key = self.advance();
            let next = self.peek();
            let type_word = next.value.trim_end_matches('!').to_string();
            let declares_type = next.kind == TokenKind::Identifier
                && next.line == key.line
                && FieldType::from_keyword(&type_word).is_some();
            if !declares_type {
                let val = self.advance().value;
                vars.insert(key.value, val);
                continue;
            }

            let type_token = self.advance();
            let env_type = EnvType::from_keyword(&type_word);
            if env_type.is_none() {
                self.diagnostics.push(
                    Self::token_error(
                        codes::UNSUPPORTED_ENV_TYPE,
                        format!(
                            "unsupported type '{}' for environment variable '{}'",
                            type_word, key.value
                        ),
                        &type_token,
                    )
                    .with_hint(format!("use one of: {}", EnvType::KEYWORDS.join(", "))),
                );
            }
            let mut required = type_token.value.ends_with('!');
            let mut sensitive = false;
            let mut default = None;
            loop {
                let t = self.peek();
                match (&t.kind, t.value.as_str()) {
                    (TokenKind::Identifier, "!" | "required") => required = true,
                    (TokenKind::Identifier, "optional") => required = false,
                    (TokenKind::Identifier, "sensitive") => sensitive = true,
                    (TokenKind::ColonPair, raw) if raw.starts_with("default:") => {
                        let (_, value) = Self::split_colon_pair(raw);
                        if let Some(ty) = env_type.filter(|ty| !ty.accepts(&value)) {
                            // The value is not echoed: defaults of sensitive
                            // variables are secrets too.
                            self.diagnostics.push(
                                Self::token_error(
                                    codes::INVALID_ENV_DEFAULT,
                                    format!(
                                        "default of environment variable '{}' is not a valid {}",
                                        key.value,
                                        ty.keyword()
                                    ),
                                    &t,
                                )
                                .with_hint(format!(
                                    "write a {} default or remove it",
                                    ty.keyword()
                                )),
                            );
                        }
                        default = Some(value);
                    }
                    _ => break,
                }
                self.advance();
            }
            if let Some(env_type) = env_type {
                schema.push(EnvVarSpec {
                    name: key.value,
                    env_type,
                    required,
                    sensitive,
                    default,
                    line: key.line,
                    col: key.col,
                });
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(EnvNode { name, vars, schema })
    }

    // ── test ──

    fn parse_test(&mut self) -> Result<TestNode, ParseError> {
        self.reject_unimplemented_block("test");
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
    fn validate_identifier(
        name: &str,
        context: &str,
        at: (usize, usize),
    ) -> Result<(), ParseError> {
        Self::validate_identifier_pattern(name, context, at)?;
        Self::reject_sql_reserved(name, context, at)
    }

    /// P040 only: `[a-zA-Z][a-zA-Z0-9_]{0,63}`. Used for bracketed array values
    /// (enum values, roles, config lists). Those are data bound as parameters,
    /// never SQL identifiers, so P041 does not apply: `enum [create, update]` is legal.
    fn validate_identifier_pattern(
        name: &str,
        context: &str,
        at: (usize, usize),
    ) -> Result<(), ParseError> {
        const SHAPE: &str = "identifiers must start with a letter and contain only [a-zA-Z0-9_]";
        let err = |message: String| {
            let e = ParseError::new(codes::INVALID_IDENTIFIER, message, at.0, at.1)
                .with_len(name.chars().count());
            let sanitized: String = name
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect();
            if !name.is_empty()
                && sanitized != name
                && sanitized.as_bytes()[0].is_ascii_alphabetic()
            {
                e.with_target(name).with_replacement(sanitized)
            } else {
                e.with_target(name).with_hint(SHAPE)
            }
        };
        // P040: Must match [a-zA-Z][a-zA-Z0-9_]{0,63}
        if name.is_empty() {
            return Err(err(format!("empty identifier in {}", context)));
        }

        let bytes = name.as_bytes();
        if !bytes[0].is_ascii_alphabetic() {
            return Err(err(format!("invalid identifier '{}' in {}", name, context)));
        }

        if name.len() > 64 {
            return Err(err(format!(
                "identifier '{}' in {} exceeds 64 characters",
                name, context
            ))
            .with_hint("identifiers must be at most 64 characters long"));
        }

        if bytes
            .iter()
            .skip(1)
            .any(|&b| !(b.is_ascii_alphanumeric() || b == b'_'))
        {
            return Err(err(format!("invalid identifier '{}' in {}", name, context)));
        }

        Ok(())
    }

    /// P041: No SQL reserved words (entity/field names become SQL identifiers).
    fn reject_sql_reserved(
        name: &str,
        context: &str,
        at: (usize, usize),
    ) -> Result<(), ParseError> {
        const SQL_RESERVED: &[&str] = &[
            "SELECT", "DROP", "INSERT", "DELETE", "UPDATE", "TABLE", "FROM", "WHERE", "OR", "AND",
            "UNION", "ALTER", "CREATE", "INDEX", "EXEC", "EXECUTE", "INTO", "VALUES", "SET",
            "NULL", "TRUE", "FALSE",
        ];

        let upper = name.to_uppercase();
        if SQL_RESERVED.contains(&upper.as_str()) {
            return Err(ParseError::new(
                codes::RESERVED_IDENTIFIER,
                format!(
                    "'{}' is a SQL reserved word and cannot be used as {}",
                    name, context
                ),
                at.0,
                at.1,
            )
            .with_len(name.chars().count())
            .with_target(name)
            .with_hint("choose a different name"));
        }

        Ok(())
    }

    // ── helpers ──

    fn parse_array(&mut self) -> Result<Vec<String>, ParseError> {
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
                Self::validate_identifier_pattern(
                    &token.value,
                    "enum value",
                    (token.line, token.col),
                )?;
            }
            items.push(self.advance().value);
        }

        self.expect(TokenKind::RBracket)?;
        Ok(items)
    }

    fn parse_string_array(&mut self) -> Result<Vec<String>, ParseError> {
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

    fn parse_deploy(&mut self) -> Result<DeployNode, ParseError> {
        self.reject_unimplemented_block("deploy");
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
    parse_diagnostics(source).map_err(|errors| diagnostic::join(&errors))
}

/// Parse a .cronus source string, returning every structured diagnostic on
/// failure: all recoverable ones (e.g. unknown field types) plus the first
/// fatal syntax error, in source order of discovery.
pub fn parse_diagnostics(source: &str) -> Result<Vec<AstNode>, Vec<ParseError>> {
    let mut parser = Parser::new(tokenize(source));
    match parser.parse() {
        Ok(nodes) if parser.diagnostics.is_empty() => Ok(nodes),
        Ok(_) => Err(parser.diagnostics),
        Err(fatal) => {
            let mut all = parser.diagnostics;
            all.push(fatal);
            Err(all)
        }
    }
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

    fn style_theme(body: &str) -> Option<String> {
        let nodes = parse(&format!("style {{\n  {body}\n}}\n")).expect("parse");
        let AstNode::Style(s) = &nodes[0] else {
            panic!("expected style")
        };
        assert!(
            !s.config.contains_key("mode"),
            "mode must not land in config"
        );
        s.theme.clone()
    }

    #[test]
    fn style_mode_is_an_alias_of_theme() {
        assert_eq!(style_theme("theme light").as_deref(), Some("light"));
        assert_eq!(style_theme("theme system").as_deref(), Some("system"));
        assert_eq!(style_theme("mode light").as_deref(), Some("light"));
        assert_eq!(style_theme("mode system").as_deref(), Some("system"));
        assert_eq!(
            style_theme("mode dark\n  preset aurora").as_deref(),
            Some("dark")
        );
        assert_eq!(style_theme("accent blue"), None);
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

    fn diag_codes(src: &str) -> Vec<&'static str> {
        match parse_diagnostics(src) {
            Ok(_) => Vec::new(),
            Err(errs) => errs.into_iter().map(|e| e.code).collect(),
        }
    }

    #[test]
    fn unknown_where_operator_is_bind_001() {
        let src = "page \"/p\" type:custom {\n  section table {\n    bind Task { query all where status in:\"paid\" }\n  }\n}\n";
        let codes = diag_codes(src);
        assert!(codes.contains(&codes::UNKNOWN_FILTER_OP), "{codes:?}");
        assert!(!codes.is_empty());
    }

    #[test]
    fn unknown_query_is_bind_002() {
        let src =
            "page \"/p\" type:custom {\n  section table {\n    bind Task { query every }\n  }\n}\n";
        assert!(diag_codes(src).contains(&codes::UNKNOWN_QUERY));
    }

    #[test]
    fn indexed_modifier_is_field_004() {
        let src = "entity Task {\n  title string indexed\n}\n";
        let errs = match parse_diagnostics(src) {
            Err(e) => e,
            Ok(_) => panic!("expected FIELD_004"),
        };
        let e = errs
            .iter()
            .find(|e| e.code == codes::UNKNOWN_FIELD_MODIFIER)
            .expect("FIELD_004");
        assert_eq!(e.replacement.as_deref(), Some("index"));
    }

    #[test]
    fn create_action_parses_field_literals() {
        let src = "entity Task { title string! }\npage \"/p\" {\n  section card {\n    bind Task { query all }\n    on click { create Task { title \"hello\" status \"open\" } toast \"ok\" success }\n  }\n}\n";
        let nodes = match parse_diagnostics(src) {
            Ok(n) => n,
            Err(e) => panic!(
                "{}",
                e.iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        };
        let AstNode::Page(p) = &nodes[1] else {
            panic!("expected page");
        };
        let create = &p.sections[0].actions[0].instructions[0];
        assert_eq!(create.verb, "create");
        assert_eq!(create.target, "Task");
        assert_eq!(
            create.modifiers.get("title").map(String::as_str),
            Some("hello")
        );
        assert_eq!(
            create.modifiers.get("status").map(String::as_str),
            Some("open")
        );
    }

    #[test]
    fn toast_bare_style_is_not_an_action_verb() {
        let src = "entity Task { title string! }\npage \"/p\" {\n  section form {\n    bind Task { query all }\n    on submit { create Task toast \"Saved\" success refresh page }\n  }\n}\n";
        match parse_diagnostics(src) {
            Ok(nodes) => {
                let AstNode::Page(p) = &nodes[1] else {
                    panic!("expected page");
                };
                let toast = &p.sections[0].actions[0].instructions[1];
                assert_eq!(toast.verb, "toast");
                assert_eq!(
                    toast.modifiers.get("style").map(String::as_str),
                    Some("success")
                );
            }
            Err(e) => panic!(
                "{}",
                e.iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        }
    }

    #[test]
    fn unknown_action_verb_is_action_001_create_is_not() {
        let bad = "page \"/p\" {\n  section form {\n    on submit { log \"x\" }\n  }\n}\n";
        assert!(diag_codes(bad).contains(&codes::UNIMPLEMENTED_ACTION));
        let ok = "entity Task { title string! }\npage \"/p\" {\n  section form {\n    bind Task { query all }\n    on submit { create Task }\n  }\n}\n";
        match parse_diagnostics(ok) {
            Ok(_) => {}
            Err(e) => panic!(
                "create must stay parseable: {}",
                e.iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        }
    }

    #[test]
    fn unimplemented_top_level_blocks_are_lang_001() {
        for src in [
            "service mailer { }\n",
            "worker jobs { }\n",
            "middleware auth { }\n",
            "compose App { }\n",
            "define \"Box\" { }\n",
        ] {
            let codes = diag_codes(src);
            assert!(
                codes.contains(&codes::UNIMPLEMENTED_BLOCK),
                "{src} -> {codes:?}"
            );
        }
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
        assert!(Parser::validate_identifier_pattern("9lives", "enum value", (1, 1)).is_err());
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
        assert!(Parser::validate_identifier("name", "test", (1, 1)).is_ok());
        assert!(Parser::validate_identifier("User", "test", (1, 1)).is_ok());
        assert!(Parser::validate_identifier("deploy_id", "test", (1, 1)).is_ok());
        assert!(Parser::validate_identifier("status", "test", (1, 1)).is_ok());
        assert!(Parser::validate_identifier("myField2", "test", (1, 1)).is_ok());
        assert!(Parser::validate_identifier("A", "test", (1, 1)).is_ok());
        // 64 chars exactly — should pass
        let long = "a".repeat(64);
        assert!(Parser::validate_identifier(&long, "test", (1, 1)).is_ok());
    }

    #[test]
    fn p040_reject_starts_with_number() {
        let err = Parser::validate_identifier("2bad", "entity field", (5, 1)).unwrap_err();
        assert!(err.contains("invalid identifier"), "got: {}", err);
        assert_eq!(err.code, "PARSE_002");
        assert!(err.contains("2bad"));
    }

    #[test]
    fn p040_reject_special_chars() {
        let err = Parser::validate_identifier("status;DROP", "entity field", (1, 1)).unwrap_err();
        assert!(err.contains("invalid identifier"), "got: {}", err);
        assert_eq!(err.code, "PARSE_002");
    }

    #[test]
    fn p040_reject_dash_in_identifier() {
        let err = Parser::validate_identifier("my-field", "entity field", (1, 1)).unwrap_err();
        assert!(err.contains("invalid identifier"), "got: {}", err);
        assert_eq!(err.code, "PARSE_002");
    }

    #[test]
    fn p040_reject_too_long() {
        let long = "a".repeat(65);
        let err = Parser::validate_identifier(&long, "test", (1, 1)).unwrap_err();
        assert!(err.contains("exceeds 64 characters"), "got: {}", err);
    }

    #[test]
    fn p040_reject_empty() {
        let err = Parser::validate_identifier("", "test", (1, 1)).unwrap_err();
        assert!(err.contains("empty identifier"), "got: {}", err);
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
            let err = Parser::validate_identifier(word, "entity name", (1, 1)).unwrap_err();
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
        let err = Parser::validate_identifier("select", "entity name", (1, 1)).unwrap_err();
        assert!(err.contains("SQL reserved word"), "got: {}", err);
        let err = Parser::validate_identifier("Drop", "entity name", (1, 1)).unwrap_err();
        assert!(err.contains("SQL reserved word"), "got: {}", err);
    }

    #[test]
    fn p041_allow_non_reserved_common_names() {
        // These are common field names that must NOT be rejected
        assert!(Parser::validate_identifier("name", "test", (1, 1)).is_ok());
        assert!(Parser::validate_identifier("status", "test", (1, 1)).is_ok());
        assert!(Parser::validate_identifier("type", "test", (1, 1)).is_ok());
        assert!(Parser::validate_identifier("value", "test", (1, 1)).is_ok());
        assert!(Parser::validate_identifier("email", "test", (1, 1)).is_ok());
        assert!(Parser::validate_identifier("User", "test", (1, 1)).is_ok());
        assert!(Parser::validate_identifier("Deployment", "test", (1, 1)).is_ok());
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

    #[test]
    fn field_constraint_errors_are_located_diagnostics() {
        let src = "entity T {\n  slug slug match:\"^[a-z\"\n  age number min:10 max:1\n  qty number min:abc\n  name string min:9 max:2\n}\n";
        let Err(errs) = parse_diagnostics(src) else {
            panic!("invalid constraints must not parse")
        };
        let found: Vec<(&str, usize)> = errs.iter().map(|e| (e.code, e.line)).collect();
        assert_eq!(
            found,
            vec![
                ("FIELD_001", 2),
                ("FIELD_002", 3),
                ("FIELD_003", 4),
                ("FIELD_002", 5)
            ],
            "{errs:?}"
        );
        assert_eq!(errs[0].col, 13, "points at match:");
        assert_eq!(errs[1].col, 21, "points at max:");
        assert!(errs.iter().all(|e| e.hint.is_some()));
        assert!(parse(
            "entity T {\n  slug slug match:\"^[a-z0-9-]+$\"\n  n number min:-5 max:5\n}\n"
        )
        .is_ok());
    }

    #[test]
    fn many_to_many_relation_syntax() {
        let ast = parse("entity Post {\n  tags -> Tag[] required\n  author -> User\n}\n").unwrap();
        let AstNode::Entity(ref e) = ast[0] else {
            panic!("entity")
        };
        assert!(e.fields[0].is_many() && e.fields[0].required);
        assert_eq!(e.fields[0].reference.as_deref(), Some("Tag"));
        assert!(!e.fields[1].is_many());
    }

    #[test]
    fn env_block_declares_typed_variables_and_keeps_legacy_pairs() {
        let src = "env {\n  APP_STRIPE_KEY string! sensitive  APP_FEATURE_X boolean default:false\n  APP_PORT int default:8080\n}\nenv production {\n  DATABASE_URL \"postgres://db\"\n}\n";
        let ast = parse(src).unwrap();
        let AstNode::Env(ref env) = ast[0] else {
            panic!("env")
        };
        assert_eq!(env.name, "");
        let names: Vec<&str> = env.schema.iter().map(|v| v.name.as_str()).collect();
        assert_eq!(names, ["APP_STRIPE_KEY", "APP_FEATURE_X", "APP_PORT"]);
        let key = &env.schema[0];
        assert!(key.required && key.sensitive && key.env_type == EnvType::String);
        assert_eq!((key.line, key.col), (2, 3));
        let flag = &env.schema[1];
        assert!(!flag.required && !flag.sensitive);
        assert_eq!(flag.env_type, EnvType::Boolean);
        assert_eq!(flag.default.as_deref(), Some("false"));
        assert_eq!(env.schema[2].env_type, EnvType::Number);
        let AstNode::Env(ref legacy) = ast[1] else {
            panic!("env")
        };
        assert_eq!(legacy.name, "production");
        assert!(legacy.schema.is_empty());
        assert_eq!(legacy.vars["DATABASE_URL"], "postgres://db");
    }

    #[test]
    fn env_unsupported_type_and_bad_default_are_errors() {
        let src = "env {\n  APP_WHEN date\n  APP_ON boolean default:maybe\n}\n";
        let Err(errs) = parse_diagnostics(src) else {
            panic!("invalid env must not parse")
        };
        let found: Vec<(&str, usize)> = errs.iter().map(|e| (e.code, e.line)).collect();
        assert_eq!(found, vec![("ENV_001", 2), ("ENV_002", 3)], "{errs:?}");
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
            err.contains("expected an HTTP method"),
            "error should say an HTTP method was expected, got: {}",
            err
        );
        assert!(err.starts_with("PARSE_001: "), "got: {}", err);
    }

    // ── Sprint 3: structured, English, located diagnostics ──

    fn diags(source: &str) -> Vec<ParseError> {
        match parse_diagnostics(source) {
            Err(e) => e,
            Ok(_) => panic!("expected diagnostics for:\n{}", source),
        }
    }

    #[test]
    fn unknown_field_type_is_type_001_with_closest_suggestion() {
        let src = "entity Task {\n  title strin!\n  qty   numbr\n}\n";
        let errs = diags(src);
        assert_eq!(errs.len(), 2, "both typos are reported: {:?}", errs);
        let a = &errs[0];
        assert_eq!(a.code, "TYPE_001");
        assert_eq!((a.line, a.col, a.len), (2, 9, 5));
        assert_eq!(a.target.as_deref(), Some("strin"));
        assert_eq!(a.replacement.as_deref(), Some("string"));
        assert_eq!(a.message, "unknown field type 'strin' for field 'title'");
        let b = &errs[1];
        assert_eq!(b.code, "TYPE_001");
        assert_eq!((b.line, b.col), (3, 9));
        assert_eq!(b.replacement.as_deref(), Some("number"));
    }

    #[test]
    fn unknown_field_type_without_near_match_lists_valid_types() {
        let errs = diags("entity Task {\n  title zzzzzzzz\n}\n");
        assert_eq!(errs[0].code, "TYPE_001");
        assert!(errs[0].replacement.is_none());
        assert!(errs[0].hint.as_deref().unwrap().contains("string, text"));
    }

    #[test]
    fn unknown_field_type_fails_string_parse_api() {
        let Err(err) = parse("entity Task {\n  title strin!\n}\n") else {
            panic!("typo must fail the string parse API")
        };
        assert_eq!(
            err,
            "TYPE_001: unknown field type 'strin' for field 'title' (line 2, col 9)"
        );
    }

    #[test]
    fn field_type_aliases_map_to_canonical_types() {
        let src = "entity M {\n  a int\n  b integer\n  c float\n  d decimal\n  e bool\n  f datetime\n  g timestamp\n}\n";
        let nodes = parse(src).expect("aliases must parse");
        let AstNode::Entity(e) = &nodes[0] else {
            panic!("entity expected")
        };
        let types: Vec<FieldType> = e.fields.iter().map(|f| f.field_type.clone()).collect();
        assert_eq!(
            types,
            vec![
                FieldType::Number,
                FieldType::Number,
                FieldType::Number,
                FieldType::Number,
                FieldType::Boolean,
                FieldType::Date,
                FieldType::Date,
            ]
        );
    }

    #[test]
    fn every_canonical_field_type_keyword_parses() {
        for kw in FIELD_TYPE_KEYWORDS {
            let src = format!("entity M {{\n  a {}\n}}\n", kw);
            assert!(parse(&src).is_ok(), "type '{}' should parse", kw);
        }
    }

    #[test]
    fn relation_with_trailing_modifiers_does_not_swallow_next_field() {
        let src = "entity Field {\n  kind  -> Kind  required\n  items -> Item[] unique\n  width number   required\n}\n";
        let nodes = parse(src).unwrap_or_else(|e| panic!("{}", e));
        let AstNode::Entity(e) = &nodes[0] else {
            panic!("entity expected")
        };
        let names: Vec<&str> = e.fields.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["kind", "items", "width"]);
        assert!(e.fields[0].required);
        assert_eq!(e.fields[0].reference.as_deref(), Some("Kind"));
        assert!(e.fields[1].array && e.fields[1].unique);
        assert_eq!(e.fields[2].field_type, FieldType::Number);
        assert!(e.fields[2].required);
    }

    #[test]
    fn bare_action_verbs_do_not_swallow_closing_braces() {
        for body in [
            "create Task  refresh",
            "delete Task  refresh",
            "refresh",
            "delete",
            "refresh  toast \"Saved\"",
            "navigate",
            "close \"x\"",
        ] {
            let src = format!(
                "entity Task {{\n  title string!\n}}\npage \"/a\" {{\n  section form {{\n    bind Task {{ query all }}\n    on submit {{ {} }}\n  }}\n}}\npage \"/b\" {{\n  section hero {{ }}\n}}\nentity Note {{\n  body text\n}}\n",
                body
            );
            let nodes = parse(&src).unwrap_or_else(|e| panic!("{}: {}", body, e));
            let (entities, pages, _) = stats(&nodes);
            assert_eq!((entities, pages), (2, 2), "`{}` merged blocks", body);
        }
        // defaults when the argument is omitted
        let src =
            "page \"/a\" {\n  section form {\n    on submit { refresh  toast \"ok\" }\n  }\n}\n";
        let nodes = parse(src).unwrap_or_else(|e| panic!("{}", e));
        let AstNode::Page(p) = &nodes[0] else {
            panic!("page expected")
        };
        let instr = &p.sections[0].actions[0].instructions;
        assert_eq!(
            (instr[0].verb.as_str(), instr[0].target.as_str()),
            ("refresh", "self")
        );
        assert_eq!(
            (instr[1].verb.as_str(), instr[1].target.as_str()),
            ("toast", "ok")
        );
    }

    #[test]
    fn field_without_type_is_type_002() {
        let errs = diags("entity Task {\n  title\n  done boolean\n}\n");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert_eq!(errs[0].code, "TYPE_002");
        assert_eq!((errs[0].line, errs[0].col), (2, 3));
    }

    #[test]
    fn unexpected_token_error_has_column_and_replacement_for_lowercase_method() {
        let src = "api /items {\n  list get /\n}\n";
        let errs = diags(src);
        let e = errs.last().unwrap();
        assert_eq!(e.code, "PARSE_001");
        assert_eq!((e.line, e.col, e.len), (2, 8, 3));
        assert_eq!(e.replacement.as_deref(), Some("GET"));
    }

    #[test]
    fn unexpected_end_of_file_points_past_last_token() {
        let errs = diags("entity Task {\n  title string");
        let e = errs.last().unwrap();
        assert_eq!(e.code, "PARSE_001");
        assert_eq!(e.message, "expected '}', found end of file");
        assert_eq!((e.line, e.col), (2, 15));
    }

    #[test]
    fn recoverable_and_fatal_diagnostics_are_both_reported() {
        let errs = diags("entity Task {\n  title strin\n}\napi /x {\n  list HEAD /\n}\n");
        let codes: Vec<&str> = errs.iter().map(|e| e.code).collect();
        assert_eq!(codes, vec!["TYPE_001", "PARSE_001"]);
        assert_eq!((errs[1].line, errs[1].col), (5, 8));
    }

    #[test]
    fn parser_messages_are_english_and_never_carry_location_text() {
        let cases = [
            "entity Task {\n  title strin\n}\n",
            "api /x {\n  list HEAD /\n}\n",
            "entity select {\n  a string\n}\n",
            "entity Task {\n  my-field string\n}\n",
            "entity T {\n  s enum [a, b]\n  transition s {\n    a -> c\n  }\n}\n",
            "entity T {\n  s string\n  transition s {\n    a -> b\n  }\n}\n",
            "entity T {\n  transition nope {\n  }\n}\n",
            "entity T {\n  on explode {\n  }\n}\n",
        ];
        for src in cases {
            for e in diags(src) {
                for banned in [
                    "Linha",
                    "esperava",
                    "encontrou",
                    "Parse error",
                    "line ",
                    "Line ",
                ] {
                    assert!(
                        !e.message.contains(banned),
                        "message {:?} contains {:?}",
                        e.message,
                        banned
                    );
                }
                assert!(e.line >= 1 && e.col >= 1, "{:?}", e);
                assert!(
                    e.code.starts_with("PARSE_") || e.code.starts_with("TYPE_"),
                    "{:?}",
                    e
                );
            }
        }
    }

    #[test]
    fn transition_state_typo_suggests_enum_value() {
        let errs = diags(
            "entity T {\n  s enum [pending, paid]\n  transition s {\n    pendng -> paid\n  }\n}\n",
        );
        let e = errs.last().unwrap();
        assert_eq!(e.code, "PARSE_005");
        assert_eq!((e.line, e.col), (4, 5));
        assert_eq!(e.replacement.as_deref(), Some("pending"));
    }

    #[test]
    fn invalid_identifier_offers_sanitized_replacement() {
        let errs = diags("entity Task {\n  my-field string\n}\n");
        let e = errs.last().unwrap();
        assert_eq!(e.code, "PARSE_002");
        assert_eq!((e.line, e.col), (2, 3));
        assert_eq!(e.replacement.as_deref(), Some("my_field"));
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
