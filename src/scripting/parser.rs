//! Parser for .scriptcronus files
//!
//! Tokenizes and parses imperative scripting syntax into ScriptFile AST.

use super::ast::*;
use std::collections::HashMap;

// ── Tokens ──

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    // Keywords
    Script, On, Schedule, Endpoint, Let, For, In, If, Else,
    Db, Http, Sse, Log, Auth, Format, Env, Respond, Webhook,
    // Literals
    Str(String),
    Num(f64),
    Bool(bool),
    // Symbols
    LBrace, RBrace, LBracket, RBracket, LParen, RParen,
    Dot, Comma, Eq, EqEq, Ne, Lt, Gt, Lte, Gte,
    And, Or,
    // Identifiers
    Ident(String),
    // Special
    Eof,
}

// ── Tokenizer ──

fn tokenize(src: &str) -> Vec<Tok> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        // Skip whitespace
        if c.is_whitespace() { i += 1; continue; }
        // Comments
        if c == '#' {
            while i < chars.len() && chars[i] != '\n' { i += 1; }
            continue;
        }
        // Strings
        if c == '"' {
            i += 1;
            let mut s = String::new();
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 1;
                    match chars[i] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        other => { s.push('\\'); s.push(other); }
                    }
                } else {
                    s.push(chars[i]);
                }
                i += 1;
            }
            i += 1; // closing quote
            tokens.push(Tok::Str(s));
            continue;
        }
        // Numbers
        if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') { i += 1; }
            let num: f64 = chars[start..i].iter().collect::<String>().parse().unwrap_or(0.0);
            tokens.push(Tok::Num(num));
            continue;
        }
        // Symbols
        match c {
            '{' => { tokens.push(Tok::LBrace); i += 1; continue; }
            '}' => { tokens.push(Tok::RBrace); i += 1; continue; }
            '[' => { tokens.push(Tok::LBracket); i += 1; continue; }
            ']' => { tokens.push(Tok::RBracket); i += 1; continue; }
            '(' => { tokens.push(Tok::LParen); i += 1; continue; }
            ')' => { tokens.push(Tok::RParen); i += 1; continue; }
            '.' => { tokens.push(Tok::Dot); i += 1; continue; }
            ',' => { tokens.push(Tok::Comma); i += 1; continue; }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Tok::EqEq); i += 2;
                } else {
                    tokens.push(Tok::Eq); i += 1;
                }
                continue;
            }
            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Tok::Ne); i += 2;
                } else {
                    i += 1; // skip bare !
                }
                continue;
            }
            '<' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Tok::Lte); i += 2;
                } else {
                    tokens.push(Tok::Lt); i += 1;
                }
                continue;
            }
            '>' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Tok::Gte); i += 2;
                } else {
                    tokens.push(Tok::Gt); i += 1;
                }
                continue;
            }
            _ => {}
        }
        // Identifiers & keywords
        if c.is_alphanumeric() || c == '_' || c == '/' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '/' || chars[i] == ':') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let tok = match word.as_str() {
                "script" => Tok::Script,
                "on" => Tok::On,
                "schedule" => Tok::Schedule,
                "endpoint" => Tok::Endpoint,
                "let" => Tok::Let,
                "for" => Tok::For,
                "in" => Tok::In,
                "if" => Tok::If,
                "else" => Tok::Else,
                "db" => Tok::Db,
                "http" => Tok::Http,
                "sse" => Tok::Sse,
                "log" => Tok::Log,
                "auth" => Tok::Auth,
                "format" => Tok::Format,
                "env" => Tok::Env,
                "respond" => Tok::Respond,
                "webhook" => Tok::Webhook,
                "true" => Tok::Bool(true),
                "false" => Tok::Bool(false),
                "and" => Tok::And,
                "or" => Tok::Or,
                _ => Tok::Ident(word),
            };
            tokens.push(tok);
            continue;
        }
        i += 1; // skip unknown
    }
    tokens.push(Tok::Eof);
    tokens
}

// ── Parser ──

pub struct ScriptParser {
    tokens: Vec<Tok>,
    pos: usize,
}

impl ScriptParser {
    pub fn parse(source: &str) -> Result<ScriptFile, String> {
        let tokens = tokenize(source);
        let mut p = ScriptParser { tokens, pos: 0 };
        p.parse_file()
    }

    fn peek(&self) -> &Tok {
        self.tokens.get(self.pos).unwrap_or(&Tok::Eof)
    }

    fn advance(&mut self) -> Tok {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Tok::Eof);
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Tok) -> Result<(), String> {
        let got = self.advance();
        if std::mem::discriminant(&got) != std::mem::discriminant(expected) {
            Err(format!("expected {:?}, got {:?}", expected, got))
        } else {
            Ok(())
        }
    }

    fn parse_file(&mut self) -> Result<ScriptFile, String> {
        let mut name = String::from("unnamed");
        let mut version = String::from("1.0");
        let mut blocks = Vec::new();

        while *self.peek() != Tok::Eof {
            match self.peek().clone() {
                Tok::Script => {
                    self.advance();
                    if let Tok::Str(n) = self.advance() { name = n; }
                    self.expect(&Tok::LBrace)?;
                    while *self.peek() != Tok::RBrace && *self.peek() != Tok::Eof {
                        if let Tok::Ident(key) = self.peek().clone() {
                            self.advance();
                            if key == "version" {
                                if let Tok::Str(v) = self.advance() { version = v; }
                            }
                        } else {
                            self.advance();
                        }
                    }
                    self.expect(&Tok::RBrace)?;
                }
                Tok::On => {
                    self.advance();
                    if *self.peek() == Tok::Webhook {
                        // on webhook "/path" { ... }
                        self.advance();
                        let path = match self.advance() {
                            Tok::Str(s) => s,
                            Tok::Ident(s) => s,
                            t => return Err(format!("expected webhook path, got {:?}", t)),
                        };
                        self.expect(&Tok::LBrace)?;
                        let body = self.parse_statements()?;
                        self.expect(&Tok::RBrace)?;
                        blocks.push(ScriptBlock::OnWebhook(OnWebhookBlock { path, body }));
                    } else {
                        // on Entity.event { ... }
                        let entity = match self.advance() {
                            Tok::Ident(s) => s,
                            t => return Err(format!("expected entity name, got {:?}", t)),
                        };
                        self.expect(&Tok::Dot)?;
                        let event = match self.advance() {
                            Tok::Ident(s) => s,
                            t => return Err(format!("expected event name, got {:?}", t)),
                        };
                        self.expect(&Tok::LBrace)?;
                        let body = self.parse_statements()?;
                        self.expect(&Tok::RBrace)?;
                        blocks.push(ScriptBlock::OnEvent(OnEventBlock { entity, event, body }));
                    }
                }
                Tok::Schedule => {
                    self.advance();
                    let sched_name = match self.advance() {
                        Tok::Str(s) => s,
                        t => return Err(format!("expected schedule name, got {:?}", t)),
                    };
                    // Parse every:"interval"
                    let mut interval = String::from("1h");
                    while *self.peek() != Tok::LBrace && *self.peek() != Tok::Eof {
                        if let Tok::Ident(key) = self.peek().clone() {
                            if key.starts_with("every:") {
                                interval = key.trim_start_matches("every:").trim_matches('"').to_string();
                                self.advance();
                            } else {
                                self.advance();
                            }
                        } else {
                            self.advance();
                        }
                    }
                    self.expect(&Tok::LBrace)?;
                    let body = self.parse_statements()?;
                    self.expect(&Tok::RBrace)?;
                    blocks.push(ScriptBlock::Schedule(ScheduleBlock { name: sched_name, interval, body }));
                }
                Tok::Endpoint => {
                    self.advance();
                    let method = match self.advance() {
                        Tok::Ident(s) => s.to_uppercase(),
                        t => return Err(format!("expected HTTP method, got {:?}", t)),
                    };
                    let path = match self.advance() {
                        Tok::Ident(s) => s,
                        Tok::Str(s) => s,
                        t => return Err(format!("expected endpoint path, got {:?}", t)),
                    };
                    let mut auth = None;
                    while *self.peek() != Tok::LBrace && *self.peek() != Tok::Eof {
                        if let Tok::Ident(key) = self.peek().clone() {
                            if key.starts_with("auth:") {
                                auth = Some(key.trim_start_matches("auth:").to_string());
                                self.advance();
                            } else {
                                self.advance();
                            }
                        } else {
                            self.advance();
                        }
                    }
                    self.expect(&Tok::LBrace)?;
                    let body = self.parse_statements()?;
                    self.expect(&Tok::RBrace)?;
                    blocks.push(ScriptBlock::Endpoint(EndpointBlock { method, path, auth, body }));
                }
                _ => { self.advance(); }
            }
        }

        Ok(ScriptFile { name, version, blocks })
    }

    fn parse_statements(&mut self) -> Result<Vec<Statement>, String> {
        let mut stmts = Vec::new();
        while *self.peek() != Tok::RBrace && *self.peek() != Tok::Eof {
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.peek().clone() {
            Tok::Let => {
                self.advance();
                let name = match self.advance() {
                    Tok::Ident(s) => s,
                    t => return Err(format!("expected variable name, got {:?}", t)),
                };
                self.expect(&Tok::Eq)?;
                let value = self.parse_expr()?;
                Ok(Statement::Let { name, value })
            }
            Tok::Log => {
                self.advance();
                let message = self.parse_expr()?;
                Ok(Statement::Log { message })
            }
            Tok::Db => {
                self.advance();
                self.expect(&Tok::Dot)?;
                let op = match self.advance() {
                    Tok::Ident(s) => s,
                    t => return Err(format!("expected db operation, got {:?}", t)),
                };
                match op.as_str() {
                    "create" => {
                        let entity = match self.advance() {
                            Tok::Ident(s) => s,
                            t => return Err(format!("expected entity, got {:?}", t)),
                        };
                        self.expect(&Tok::LBrace)?;
                        let fields = self.parse_field_map()?;
                        self.expect(&Tok::RBrace)?;
                        Ok(Statement::DbCreate { entity, fields })
                    }
                    "update" => {
                        let entity = match self.advance() {
                            Tok::Ident(s) => s,
                            t => return Err(format!("expected entity, got {:?}", t)),
                        };
                        let id = self.parse_expr()?;
                        self.expect(&Tok::LBrace)?;
                        let fields = self.parse_field_map()?;
                        self.expect(&Tok::RBrace)?;
                        Ok(Statement::DbUpdate { entity, id, fields })
                    }
                    "delete" => {
                        let entity = match self.advance() {
                            Tok::Ident(s) => s,
                            t => return Err(format!("expected entity, got {:?}", t)),
                        };
                        let id = self.parse_expr()?;
                        Ok(Statement::DbDelete { entity, id })
                    }
                    _ => Err(format!("unknown db operation: {}", op)),
                }
            }
            Tok::Sse => {
                self.advance();
                self.expect(&Tok::Dot)?;
                let _op = self.advance(); // "broadcast"
                let event = match self.advance() {
                    Tok::Str(s) => s,
                    t => return Err(format!("expected event name, got {:?}", t)),
                };
                let mut data = HashMap::new();
                if *self.peek() == Tok::LBrace {
                    self.advance();
                    data = self.parse_field_map()?;
                    self.expect(&Tok::RBrace)?;
                }
                Ok(Statement::SseBroadcast { event, data })
            }
            Tok::For => {
                self.advance();
                let var = match self.advance() {
                    Tok::Ident(s) => s,
                    t => return Err(format!("expected variable, got {:?}", t)),
                };
                self.expect(&Tok::In)?;
                let iter = self.parse_expr()?;
                self.expect(&Tok::LBrace)?;
                let body = self.parse_statements()?;
                self.expect(&Tok::RBrace)?;
                Ok(Statement::For { var, iter, body })
            }
            Tok::If => {
                self.advance();
                let condition = self.parse_expr()?;
                self.expect(&Tok::LBrace)?;
                let then_body = self.parse_statements()?;
                self.expect(&Tok::RBrace)?;
                let else_body = if *self.peek() == Tok::Else {
                    self.advance();
                    self.expect(&Tok::LBrace)?;
                    let eb = self.parse_statements()?;
                    self.expect(&Tok::RBrace)?;
                    eb
                } else {
                    Vec::new()
                };
                Ok(Statement::If { condition, then_body, else_body })
            }
            Tok::Respond => {
                self.advance();
                let status = match self.advance() {
                    Tok::Num(n) => n as u16,
                    t => return Err(format!("expected status code, got {:?}", t)),
                };
                let body = self.parse_expr()?;
                let mut headers = HashMap::new();
                if *self.peek() == Tok::LBrace {
                    self.advance();
                    while *self.peek() != Tok::RBrace && *self.peek() != Tok::Eof {
                        let key = match self.advance() {
                            Tok::Ident(s) | Tok::Str(s) => s,
                            _ => continue,
                        };
                        let val = match self.advance() {
                            Tok::Str(s) => s,
                            Tok::Ident(s) => s,
                            _ => continue,
                        };
                        headers.insert(key, val);
                    }
                    self.expect(&Tok::RBrace)?;
                }
                Ok(Statement::Respond { status, body, headers })
            }
            Tok::Http => {
                let expr = self.parse_expr()?;
                Ok(Statement::ExprStatement(expr))
            }
            _ => {
                self.advance(); // skip unrecognized tokens gracefully
                Ok(Statement::ExprStatement(Expr::BoolLit(false)))
            }
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let left = self.parse_primary()?;
        // Check for binary operators
        match self.peek() {
            Tok::EqEq | Tok::Ne | Tok::Lt | Tok::Gt | Tok::Lte | Tok::Gte | Tok::And | Tok::Or => {
                let op = match self.advance() {
                    Tok::EqEq => BinOperator::Eq,
                    Tok::Ne => BinOperator::Ne,
                    Tok::Lt => BinOperator::Lt,
                    Tok::Gt => BinOperator::Gt,
                    Tok::Lte => BinOperator::Lte,
                    Tok::Gte => BinOperator::Gte,
                    Tok::And => BinOperator::And,
                    Tok::Or => BinOperator::Or,
                    _ => unreachable!(),
                };
                let right = self.parse_primary()?;
                Ok(Expr::BinOp { left: Box::new(left), op, right: Box::new(right) })
            }
            _ => Ok(left),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek().clone() {
            Tok::Str(s) => { self.advance(); Ok(Expr::StringLit(s)) }
            Tok::Num(n) => { self.advance(); Ok(Expr::NumberLit(n)) }
            Tok::Bool(b) => { self.advance(); Ok(Expr::BoolLit(b)) }
            Tok::Ident(first) => {
                self.advance();
                // Check for now()
                if first == "now" && *self.peek() == Tok::LParen {
                    self.advance(); // (
                    self.advance(); // )
                    return Ok(Expr::Now);
                }
                // Build path: a.b.c
                let mut path = vec![first];
                while *self.peek() == Tok::Dot {
                    self.advance();
                    match self.advance() {
                        Tok::Ident(s) => path.push(s),
                        Tok::Db => path.push("db".into()),
                        Tok::Http => path.push("http".into()),
                        t => return Err(format!("expected identifier after dot, got {:?}", t)),
                    }
                }
                Ok(Expr::Path(path))
            }
            Tok::Db => {
                self.advance();
                self.expect(&Tok::Dot)?;
                let op = match self.advance() {
                    Tok::Ident(s) => s,
                    t => return Err(format!("expected db op, got {:?}", t)),
                };
                let entity = match self.advance() {
                    Tok::Ident(s) => s,
                    t => return Err(format!("expected entity, got {:?}", t)),
                };
                match op.as_str() {
                    "query" => {
                        let mut filters = Vec::new();
                        let mut order = None;
                        let mut limit = None;
                        if *self.peek() == Tok::LBrace {
                            self.advance();
                            while *self.peek() != Tok::RBrace && *self.peek() != Tok::Eof {
                                match self.peek().clone() {
                                    Tok::Ident(ref k) if k == "filter" => {
                                        self.advance();
                                        let field = match self.advance() {
                                            Tok::Ident(s) => s,
                                            t => return Err(format!("expected field, got {:?}", t)),
                                        };
                                        let fop = match self.advance() {
                                            Tok::EqEq => BinOperator::Eq,
                                            Tok::Ne => BinOperator::Ne,
                                            Tok::Lt => BinOperator::Lt,
                                            Tok::Gt => BinOperator::Gt,
                                            Tok::Lte => BinOperator::Lte,
                                            Tok::Gte => BinOperator::Gte,
                                            t => return Err(format!("expected operator, got {:?}", t)),
                                        };
                                        let value = self.parse_primary()?;
                                        filters.push(Filter { field, op: fop, value });
                                    }
                                    Tok::Ident(ref k) if k == "order" => {
                                        self.advance();
                                        if let Tok::Ident(s) = self.advance() { order = Some(s); }
                                    }
                                    Tok::Ident(ref k) if k == "limit" => {
                                        self.advance();
                                        if let Tok::Num(n) = self.advance() { limit = Some(n as u64); }
                                    }
                                    Tok::Ident(ref k) if k == "query" => {
                                        self.advance();
                                        // skip "all" or similar
                                        if let Tok::Ident(_) = self.peek() { self.advance(); }
                                    }
                                    _ => { self.advance(); }
                                }
                            }
                            self.expect(&Tok::RBrace)?;
                        }
                        Ok(Expr::DbQuery { entity, filters, order, limit })
                    }
                    "count" => {
                        let mut filters = Vec::new();
                        if *self.peek() == Tok::LBrace {
                            self.advance();
                            while *self.peek() != Tok::RBrace && *self.peek() != Tok::Eof {
                                if let Tok::Ident(ref k) = self.peek().clone() {
                                    if k == "filter" {
                                        self.advance();
                                        let field = match self.advance() {
                                            Tok::Ident(s) => s,
                                            t => return Err(format!("expected field, got {:?}", t)),
                                        };
                                        let fop = match self.advance() {
                                            Tok::EqEq => BinOperator::Eq,
                                            Tok::Ne => BinOperator::Ne,
                                            t => return Err(format!("expected op, got {:?}", t)),
                                        };
                                        let value = self.parse_primary()?;
                                        filters.push(Filter { field, op: fop, value });
                                    } else { self.advance(); }
                                } else { self.advance(); }
                            }
                            self.expect(&Tok::RBrace)?;
                        }
                        Ok(Expr::DbCount { entity, filters })
                    }
                    _ => Err(format!("unknown db query op: {}", op)),
                }
            }
            Tok::Http => {
                self.advance();
                self.expect(&Tok::Dot)?;
                let method = match self.advance() {
                    Tok::Ident(s) => s.to_lowercase(),
                    t => return Err(format!("expected method, got {:?}", t)),
                };
                let url = Box::new(self.parse_primary()?);
                let mut headers = HashMap::new();
                let mut body = None;
                let mut json = None;
                if *self.peek() == Tok::LBrace {
                    self.advance();
                    while *self.peek() != Tok::RBrace && *self.peek() != Tok::Eof {
                        match self.peek().clone() {
                            Tok::Ident(ref k) if k == "headers" => {
                                self.advance();
                                self.expect(&Tok::LBrace)?;
                                while *self.peek() != Tok::RBrace && *self.peek() != Tok::Eof {
                                    let key = match self.advance() {
                                        Tok::Ident(s) | Tok::Str(s) => s,
                                        _ => continue,
                                    };
                                    let val = self.parse_primary()?;
                                    headers.insert(key, val);
                                }
                                self.expect(&Tok::RBrace)?;
                            }
                            Tok::Ident(ref k) if k == "body" => {
                                self.advance();
                                body = Some(Box::new(self.parse_primary()?));
                            }
                            Tok::Ident(ref k) if k == "json" => {
                                self.advance();
                                self.expect(&Tok::LBrace)?;
                                let fields = self.parse_field_map()?;
                                self.expect(&Tok::RBrace)?;
                                json = Some(fields);
                            }
                            _ => { self.advance(); }
                        }
                    }
                    self.expect(&Tok::RBrace)?;
                }
                Ok(Expr::HttpCall { method, url, headers, body, json })
            }
            Tok::Format => {
                self.advance();
                self.expect(&Tok::Dot)?;
                let fmt = match self.advance() {
                    Tok::Ident(s) => s,
                    t => return Err(format!("expected format type, got {:?}", t)),
                };
                let data = Box::new(self.parse_primary()?);
                match fmt.as_str() {
                    "csv" => {
                        let mut fields = Vec::new();
                        if *self.peek() == Tok::LBracket {
                            self.advance();
                            while *self.peek() != Tok::RBracket && *self.peek() != Tok::Eof {
                                match self.advance() {
                                    Tok::Str(s) | Tok::Ident(s) => fields.push(s),
                                    Tok::Comma => continue,
                                    _ => {}
                                }
                            }
                            self.expect(&Tok::RBracket)?;
                        }
                        Ok(Expr::FormatCsv { data, fields })
                    }
                    "json" => Ok(Expr::FormatJson { data }),
                    _ => Err(format!("unknown format: {}", fmt)),
                }
            }
            Tok::Env => {
                self.advance();
                self.expect(&Tok::Dot)?;
                let key = match self.advance() {
                    Tok::Ident(s) => s,
                    t => return Err(format!("expected env key, got {:?}", t)),
                };
                Ok(Expr::EnvVar(key))
            }
            Tok::Auth => {
                self.advance();
                self.expect(&Tok::Dot)?;
                let op = match self.advance() {
                    Tok::Ident(s) => s,
                    t => return Err(format!("expected auth op, got {:?}", t)),
                };
                match op.as_str() {
                    "check_role" => {
                        let role = match self.advance() {
                            Tok::Str(s) => s,
                            t => return Err(format!("expected role, got {:?}", t)),
                        };
                        Ok(Expr::AuthCheckRole(role))
                    }
                    "get_user" => Ok(Expr::AuthGetUser),
                    _ => Err(format!("unknown auth op: {}", op)),
                }
            }
            t => Err(format!("unexpected token: {:?}", t)),
        }
    }

    fn parse_field_map(&mut self) -> Result<HashMap<String, Expr>, String> {
        let mut fields = HashMap::new();
        while *self.peek() != Tok::RBrace && *self.peek() != Tok::Eof {
            let key = match self.advance() {
                Tok::Ident(s) | Tok::Str(s) => s,
                Tok::Comma => continue,
                _ => continue,
            };
            let value = self.parse_expr()?;
            fields.insert(key, value);
        }
        Ok(fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_script() {
        let src = r#"
script "Test Script" {
  version "1.0"
}

on Customer.create {
  log "Customer created"
  db.update Customer event.id {
    synced true
  }
}

schedule "cleanup" every:1h {
  let old = db.query Order { filter status == "cancelled" }
  for o in old {
    db.delete Order o.id
  }
}

endpoint GET /api/health auth:admin {
  respond 200 "ok"
}
"#;
        let result = ScriptParser::parse(src);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());
        let file = result.unwrap();
        assert_eq!(file.name, "Test Script");
        assert_eq!(file.version, "1.0");
        assert_eq!(file.blocks.len(), 3);
    }

    #[test]
    fn test_parse_webhook() {
        let src = r#"
on webhook "/hooks/stripe" {
  db.create Payment {
    external_id event.body.id
    amount event.body.amount
  }
}
"#;
        let file = ScriptParser::parse(src).unwrap();
        assert_eq!(file.blocks.len(), 1);
        match &file.blocks[0] {
            ScriptBlock::OnWebhook(w) => assert_eq!(w.path, "/hooks/stripe"),
            _ => panic!("expected OnWebhook"),
        }
    }

    // ── Parse-positive: on event ──

    #[test]
    fn test_parse_on_event() {
        let src = r#"on Customer.create { log "test" }"#;
        let file = ScriptParser::parse(src).unwrap();
        assert_eq!(file.blocks.len(), 1);
        match &file.blocks[0] {
            ScriptBlock::OnEvent(e) => {
                assert_eq!(e.entity, "Customer");
                assert_eq!(e.event, "create");
                assert_eq!(e.body.len(), 1);
            }
            _ => panic!("expected OnEvent"),
        }
    }

    #[test]
    fn test_parse_on_update() {
        let src = r#"on Order.update { db.update Order event.id { status "processed" } }"#;
        let file = ScriptParser::parse(src).unwrap();
        assert_eq!(file.blocks.len(), 1);
        match &file.blocks[0] {
            ScriptBlock::OnEvent(e) => {
                assert_eq!(e.entity, "Order");
                assert_eq!(e.event, "update");
            }
            _ => panic!("expected OnEvent"),
        }
    }

    #[test]
    fn test_parse_on_delete() {
        let src = r#"on Item.delete { log "deleted" }"#;
        let file = ScriptParser::parse(src).unwrap();
        assert_eq!(file.blocks.len(), 1);
        match &file.blocks[0] {
            ScriptBlock::OnEvent(e) => {
                assert_eq!(e.entity, "Item");
                assert_eq!(e.event, "delete");
            }
            _ => panic!("expected OnEvent"),
        }
    }

    // ── Parse-positive: schedule ──

    #[test]
    fn test_parse_schedule() {
        let src = r#"schedule "cleanup" every:1h { let x = db.query Temp { query all } }"#;
        let file = ScriptParser::parse(src).unwrap();
        assert_eq!(file.blocks.len(), 1);
        match &file.blocks[0] {
            ScriptBlock::Schedule(s) => {
                assert_eq!(s.name, "cleanup");
                assert_eq!(s.interval, "1h");
                assert_eq!(s.body.len(), 1);
            }
            _ => panic!("expected Schedule"),
        }
    }

    // ── Parse-positive: endpoint ──

    #[test]
    fn test_parse_endpoint_get() {
        let src = r#"endpoint GET /api/health { respond 200 "ok" }"#;
        let file = ScriptParser::parse(src).unwrap();
        assert_eq!(file.blocks.len(), 1);
        match &file.blocks[0] {
            ScriptBlock::Endpoint(e) => {
                assert_eq!(e.method, "GET");
                assert_eq!(e.path, "/api/health");
                assert!(e.auth.is_none());
            }
            _ => panic!("expected Endpoint"),
        }
    }

    #[test]
    fn test_parse_endpoint_with_auth() {
        let src = r#"endpoint POST /api/admin auth:admin { respond 201 "created" }"#;
        let file = ScriptParser::parse(src).unwrap();
        assert_eq!(file.blocks.len(), 1);
        match &file.blocks[0] {
            ScriptBlock::Endpoint(e) => {
                assert_eq!(e.method, "POST");
                assert_eq!(e.path, "/api/admin");
                assert_eq!(e.auth.as_deref(), Some("admin"));
            }
            _ => panic!("expected Endpoint"),
        }
    }

    // ── Parse-positive: webhook ──

    #[test]
    fn test_parse_webhook_stripe() {
        let src = r#"on webhook "/hooks/stripe" { db.create Payment { amount event.body.amount } }"#;
        let file = ScriptParser::parse(src).unwrap();
        assert_eq!(file.blocks.len(), 1);
        match &file.blocks[0] {
            ScriptBlock::OnWebhook(w) => assert_eq!(w.path, "/hooks/stripe"),
            _ => panic!("expected OnWebhook"),
        }
    }

    // ── Parse-positive: let variable ──

    #[test]
    fn test_parse_let_variable() {
        let src = r#"on X.create { let a = "hello" }"#;
        let file = ScriptParser::parse(src).unwrap();
        match &file.blocks[0] {
            ScriptBlock::OnEvent(e) => {
                assert_eq!(e.body.len(), 1);
                match &e.body[0] {
                    Statement::Let { name, .. } => assert_eq!(name, "a"),
                    _ => panic!("expected Let"),
                }
            }
            _ => panic!("expected OnEvent"),
        }
    }

    // ── Parse-positive: for loop ──

    #[test]
    fn test_parse_for_loop() {
        let src = r#"on X.create { let items = db.query X { query all } for i in items { log "x" } }"#;
        let file = ScriptParser::parse(src).unwrap();
        match &file.blocks[0] {
            ScriptBlock::OnEvent(e) => {
                assert_eq!(e.body.len(), 2);
                match &e.body[1] {
                    Statement::For { var, .. } => assert_eq!(var, "i"),
                    _ => panic!("expected For"),
                }
            }
            _ => panic!("expected OnEvent"),
        }
    }

    // ── Parse-positive: if/else ──

    #[test]
    fn test_parse_if_else() {
        let src = r#"on X.create { if event.record.status == "active" { log "yes" } else { log "no" } }"#;
        let file = ScriptParser::parse(src).unwrap();
        match &file.blocks[0] {
            ScriptBlock::OnEvent(e) => {
                assert_eq!(e.body.len(), 1);
                match &e.body[0] {
                    Statement::If { then_body, else_body, .. } => {
                        assert_eq!(then_body.len(), 1);
                        assert_eq!(else_body.len(), 1);
                    }
                    _ => panic!("expected If"),
                }
            }
            _ => panic!("expected OnEvent"),
        }
    }

    // ── Parse-positive: db operations ──

    #[test]
    fn test_parse_db_query_with_filters() {
        let src = r#"on X.create { let r = db.query Order { filter status == "paid" limit 10 } }"#;
        let file = ScriptParser::parse(src).unwrap();
        match &file.blocks[0] {
            ScriptBlock::OnEvent(e) => {
                match &e.body[0] {
                    Statement::Let { value, .. } => {
                        match value {
                            Expr::DbQuery { entity, filters, limit, .. } => {
                                assert_eq!(entity, "Order");
                                assert_eq!(filters.len(), 1);
                                assert_eq!(filters[0].field, "status");
                                assert_eq!(*limit, Some(10));
                            }
                            _ => panic!("expected DbQuery"),
                        }
                    }
                    _ => panic!("expected Let"),
                }
            }
            _ => panic!("expected OnEvent"),
        }
    }

    #[test]
    fn test_parse_db_create() {
        let src = r#"on X.create { db.create Order { name "test" amount 100 } }"#;
        let file = ScriptParser::parse(src).unwrap();
        match &file.blocks[0] {
            ScriptBlock::OnEvent(e) => {
                match &e.body[0] {
                    Statement::DbCreate { entity, fields } => {
                        assert_eq!(entity, "Order");
                        assert!(fields.contains_key("name"));
                        assert!(fields.contains_key("amount"));
                    }
                    _ => panic!("expected DbCreate"),
                }
            }
            _ => panic!("expected OnEvent"),
        }
    }

    #[test]
    fn test_parse_db_delete() {
        let src = r#"on X.create { db.delete Order event.id }"#;
        let file = ScriptParser::parse(src).unwrap();
        match &file.blocks[0] {
            ScriptBlock::OnEvent(e) => {
                match &e.body[0] {
                    Statement::DbDelete { entity, .. } => assert_eq!(entity, "Order"),
                    _ => panic!("expected DbDelete"),
                }
            }
            _ => panic!("expected OnEvent"),
        }
    }

    // ── Parse-positive: http ──

    #[test]
    fn test_parse_http_post() {
        let src = r#"on X.create { let r = http.post "https://api.example.com" { headers { Authorization "Bearer key" } } }"#;
        let file = ScriptParser::parse(src).unwrap();
        match &file.blocks[0] {
            ScriptBlock::OnEvent(e) => {
                match &e.body[0] {
                    Statement::Let { value, .. } => {
                        match value {
                            Expr::HttpCall { method, headers, .. } => {
                                assert_eq!(method, "post");
                                assert!(headers.contains_key("Authorization"));
                            }
                            _ => panic!("expected HttpCall"),
                        }
                    }
                    _ => panic!("expected Let"),
                }
            }
            _ => panic!("expected OnEvent"),
        }
    }

    // ── Parse-positive: sse ──

    #[test]
    fn test_parse_sse_broadcast() {
        let src = r#"on X.create { sse.broadcast "update" { id event.id } }"#;
        let file = ScriptParser::parse(src).unwrap();
        match &file.blocks[0] {
            ScriptBlock::OnEvent(e) => {
                match &e.body[0] {
                    Statement::SseBroadcast { event, data } => {
                        assert_eq!(event, "update");
                        assert!(data.contains_key("id"));
                    }
                    _ => panic!("expected SseBroadcast"),
                }
            }
            _ => panic!("expected OnEvent"),
        }
    }

    // ── Parse-positive: env ──

    #[test]
    fn test_parse_env_var() {
        let src = r#"on X.create { let key = env.API_KEY }"#;
        let file = ScriptParser::parse(src).unwrap();
        match &file.blocks[0] {
            ScriptBlock::OnEvent(e) => {
                match &e.body[0] {
                    Statement::Let { value, .. } => {
                        match value {
                            Expr::EnvVar(k) => assert_eq!(k, "API_KEY"),
                            _ => panic!("expected EnvVar"),
                        }
                    }
                    _ => panic!("expected Let"),
                }
            }
            _ => panic!("expected OnEvent"),
        }
    }

    // ── Parse-positive: format.csv ──

    #[test]
    fn test_parse_format_csv() {
        let src = r#"endpoint GET /api/export { let d = db.query X { query all } let csv = format.csv d ["a","b"] respond 200 csv }"#;
        let file = ScriptParser::parse(src).unwrap();
        assert_eq!(file.blocks.len(), 1);
        match &file.blocks[0] {
            ScriptBlock::Endpoint(e) => {
                assert_eq!(e.body.len(), 3);
            }
            _ => panic!("expected Endpoint"),
        }
    }

    // ── Parse-positive: multiple blocks ──

    #[test]
    fn test_parse_multiple_blocks() {
        let src = r#"
on User.create { log "user created" }
schedule "daily" every:24h { log "tick" }
endpoint GET /api/ping { respond 200 "pong" }
"#;
        let file = ScriptParser::parse(src).unwrap();
        assert_eq!(file.blocks.len(), 3);
        assert!(matches!(&file.blocks[0], ScriptBlock::OnEvent(_)));
        assert!(matches!(&file.blocks[1], ScriptBlock::Schedule(_)));
        assert!(matches!(&file.blocks[2], ScriptBlock::Endpoint(_)));
    }

    // ── Parse-positive: script header ──

    #[test]
    fn test_parse_script_header() {
        let src = r#"script "Test" { version "2.0" }"#;
        let file = ScriptParser::parse(src).unwrap();
        assert_eq!(file.name, "Test");
        assert_eq!(file.version, "2.0");
        assert_eq!(file.blocks.len(), 0);
    }

    // ── Parse-positive: empty script ──

    #[test]
    fn test_parse_empty_script() {
        let src = r#"script "Empty" { version "1.0" }"#;
        let file = ScriptParser::parse(src).unwrap();
        assert_eq!(file.name, "Empty");
        assert_eq!(file.version, "1.0");
        assert_eq!(file.blocks.len(), 0);
    }

    // ── Parse-negative: error handling ──

    #[test]
    fn test_parse_missing_entity() {
        // `on .create { }` — dot without entity should error
        let result = ScriptParser::parse(r#"on .create { }"#);
        assert!(result.is_err(), "should fail: missing entity before dot");
    }

    #[test]
    fn test_parse_missing_brace() {
        // `on X.create log "test"` — no opening brace should error
        let result = ScriptParser::parse(r#"on X.create log "test""#);
        assert!(result.is_err(), "should fail: missing opening brace");
    }

    #[test]
    fn test_parse_empty_file() {
        // Empty string should parse OK (no blocks, default name)
        let result = ScriptParser::parse("");
        assert!(result.is_ok(), "empty file should parse without panic");
        let file = result.unwrap();
        assert_eq!(file.blocks.len(), 0);
    }

    #[test]
    fn test_parse_unknown_keyword() {
        // `foobar { }` — unknown top-level keyword should not panic
        let result = ScriptParser::parse(r#"foobar { }"#);
        // Parser skips unknown tokens, should not crash
        assert!(result.is_ok(), "unknown keyword should not panic");
    }

    #[test]
    fn test_parse_nested_error() {
        // `on X.create { db.create { } }` — missing entity in db.create
        let result = ScriptParser::parse(r#"on X.create { db.create { } }"#);
        assert!(result.is_err(), "should fail: db.create missing entity");
    }
}
