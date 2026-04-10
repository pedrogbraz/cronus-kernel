//! AST nodes for .scriptcronus files
//!
//! Three top-level constructs:
//! - `on Entity.event { ... }` — react to entity CRUD events
//! - `schedule "name" every:"interval" { ... }` — cron-like jobs
//! - `endpoint METHOD /path auth:role { ... }` — custom HTTP endpoints
//! - `on webhook "/path" { ... }` — receive external webhooks

use std::collections::HashMap;

/// Root AST for a .scriptcronus file
#[derive(Debug, Clone)]
pub struct ScriptFile {
    pub name: String,
    pub version: String,
    pub blocks: Vec<ScriptBlock>,
}

/// Top-level block types
#[derive(Debug, Clone)]
pub enum ScriptBlock {
    OnEvent(OnEventBlock),
    OnWebhook(OnWebhookBlock),
    Schedule(ScheduleBlock),
    Endpoint(EndpointBlock),
}

/// `on Entity.event { ... }`
#[derive(Debug, Clone)]
pub struct OnEventBlock {
    pub entity: String,
    pub event: String, // "create", "update", "delete"
    pub body: Vec<Statement>,
}

/// `on webhook "/path" { ... }`
#[derive(Debug, Clone)]
pub struct OnWebhookBlock {
    pub path: String,
    pub body: Vec<Statement>,
}

/// `schedule "name" every:"interval" { ... }`
#[derive(Debug, Clone)]
pub struct ScheduleBlock {
    pub name: String,
    pub interval: String, // "1h", "30m", "1d"
    pub body: Vec<Statement>,
}

/// `endpoint METHOD /path auth:role { ... }`
#[derive(Debug, Clone)]
pub struct EndpointBlock {
    pub method: String,
    pub path: String,
    pub auth: Option<String>,
    pub body: Vec<Statement>,
}

/// Imperative statements inside blocks
#[derive(Debug, Clone)]
pub enum Statement {
    /// `let name = expr`
    Let {
        name: String,
        value: Expr,
    },
    /// `db.create Entity { field value, ... }`
    DbCreate {
        entity: String,
        fields: HashMap<String, Expr>,
    },
    /// `db.update Entity id { field value, ... }`
    DbUpdate {
        entity: String,
        id: Expr,
        fields: HashMap<String, Expr>,
    },
    /// `db.delete Entity id`
    DbDelete {
        entity: String,
        id: Expr,
    },
    /// `log "message"`
    Log {
        message: Expr,
    },
    /// `sse.broadcast "event" { key value, ... }`
    SseBroadcast {
        event: String,
        data: HashMap<String, Expr>,
    },
    /// `for item in expr { ... }`
    For {
        var: String,
        iter: Expr,
        body: Vec<Statement>,
    },
    /// `if expr { ... } else { ... }`
    If {
        condition: Expr,
        then_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    /// `respond status expr { headers }`
    Respond {
        status: u16,
        body: Expr,
        headers: HashMap<String, String>,
    },
    /// Raw expression as statement (function calls, etc.)
    ExprStatement(Expr),
}

/// Expressions
#[derive(Debug, Clone)]
pub enum Expr {
    /// String literal: `"hello"` or template `"id={{event.id}}"`
    StringLit(String),
    /// Number literal: `42`, `3.14`
    NumberLit(f64),
    /// Boolean: `true`, `false`
    BoolLit(bool),
    /// Variable/path access: `event.record.email`, `stripe.json.id`
    Path(Vec<String>),
    /// `db.query Entity { filter ..., order ..., limit ... }`
    DbQuery {
        entity: String,
        filters: Vec<Filter>,
        order: Option<String>,
        limit: Option<u64>,
    },
    /// `db.count Entity { filter ... }`
    DbCount {
        entity: String,
        filters: Vec<Filter>,
    },
    /// `http.METHOD "url" { headers {}, body/json {} }`
    HttpCall {
        method: String,
        url: Box<Expr>,
        headers: HashMap<String, Expr>,
        body: Option<Box<Expr>>,
        json: Option<HashMap<String, Expr>>,
    },
    /// `format.csv expr [fields]`
    FormatCsv {
        data: Box<Expr>,
        fields: Vec<String>,
    },
    /// `format.json expr`
    FormatJson {
        data: Box<Expr>,
    },
    /// `now()`
    Now,
    /// `env.KEY`
    EnvVar(String),
    /// Binary operation: `a == b`, `a < b`, `a and b`
    BinOp {
        left: Box<Expr>,
        op: BinOperator,
        right: Box<Expr>,
    },
    /// `auth.check_role "admin"`
    AuthCheckRole(String),
    /// `auth.get_user`
    AuthGetUser,
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub field: String,
    pub op: BinOperator,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOperator {
    Eq,
    Ne,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
    Contains,
}
