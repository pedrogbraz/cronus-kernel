//! Compact JSON summary of a parsed `.cronus` AST for the `parse_ast` tool.
//!
//! Not the Rust `Debug` dump: only what an author reasons about (names, types,
//! modifiers, routes, auth, sections and bindings). Keys whose value is empty,
//! `false` or absent are omitted; maps are emitted in sorted order.

use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap};

use crate::parser::{
    self, ApiNode, AstNode, AuthNode, BindingNode, BindingValue, EntityNode, FieldNode, FieldType,
    FilterOp, HttpMethod, LayoutNode, OrderDirection, PageNode, QueryType, SectionNode, StyleNode,
};

pub(super) fn summarize(nodes: &[AstNode]) -> Value {
    let mut out = Map::new();
    let mut entities = Vec::new();
    let mut apis = Vec::new();
    let mut pages = Vec::new();
    let mut layouts = Vec::new();
    let mut other: BTreeMap<&str, usize> = BTreeMap::new();

    for node in nodes {
        match node {
            AstNode::App(a) => {
                let mut app = Map::new();
                let name = a.name.split('|').next().unwrap_or(&a.name).trim();
                app.insert("name".into(), json!(name));
                app.insert("port".into(), json!(a.port));
                if !a.graphql {
                    app.insert("graphql".into(), json!(false));
                }
                put_list(&mut app, "stack", &a.stack);
                if let Some(db) = &a.database {
                    let mut d = Map::new();
                    d.insert("type".into(), json!(db.db_type));
                    put_opt(&mut d, "path", &db.path);
                    app.insert("database".into(), Value::Object(d));
                }
                out.insert("app".into(), Value::Object(app));
            }
            AstNode::Auth(a) => {
                out.insert("auth".into(), auth(a));
            }
            AstNode::Entity(e) => entities.push(entity(e)),
            AstNode::Api(a) => apis.push(api(a)),
            AstNode::Page(p) => pages.push(page(p)),
            AstNode::Layout(l) => layouts.push(layout(l)),
            AstNode::Style(s) => {
                out.insert("style".into(), style(s));
            }
            AstNode::Service(_) => *other.entry("services").or_default() += 1,
            AstNode::Component(_) => *other.entry("components").or_default() += 1,
            AstNode::Import(_) => *other.entry("imports").or_default() += 1,
            AstNode::Event(_) => *other.entry("events").or_default() += 1,
            AstNode::Worker(_) => *other.entry("workers").or_default() += 1,
            AstNode::Middleware(_) => *other.entry("middleware").or_default() += 1,
            AstNode::Env(_) => *other.entry("env").or_default() += 1,
            AstNode::Test(_) => *other.entry("tests").or_default() += 1,
            AstNode::Compose(_) => *other.entry("compose").or_default() += 1,
            AstNode::Define(_) => *other.entry("defines").or_default() += 1,
            AstNode::Webhook(_) => *other.entry("webhooks").or_default() += 1,
            AstNode::Deploy(_) => *other.entry("deploy").or_default() += 1,
        }
    }

    out.insert("entities".into(), Value::Array(entities));
    out.insert("apis".into(), Value::Array(apis));
    out.insert("pages".into(), Value::Array(pages));
    if !layouts.is_empty() {
        out.insert("layouts".into(), Value::Array(layouts));
    }
    if !other.is_empty() {
        out.insert("other_blocks".into(), json!(other));
    }
    let (e, p, r) = parser::stats(nodes);
    out.insert(
        "stats".into(),
        json!({"entities": e, "pages": p, "routes": r}),
    );
    Value::Object(out)
}

fn put_opt<T: serde::Serialize>(m: &mut Map<String, Value>, key: &str, v: &Option<T>) {
    if let Some(v) = v {
        m.insert(key.into(), json!(v));
    }
}

fn put_list<T: serde::Serialize>(m: &mut Map<String, Value>, key: &str, v: &[T]) {
    if !v.is_empty() {
        m.insert(key.into(), json!(v));
    }
}

fn put_map(m: &mut Map<String, Value>, key: &str, v: &HashMap<String, String>) {
    if !v.is_empty() {
        let sorted: BTreeMap<_, _> = v.iter().collect();
        m.insert(key.into(), json!(sorted));
    }
}

fn field_type(t: &FieldType) -> &'static str {
    match t {
        FieldType::String => "string",
        FieldType::Text => "text",
        FieldType::Email => "email",
        FieldType::Url => "url",
        FieldType::Slug => "slug",
        FieldType::Phone => "phone",
        FieldType::Number => "number",
        FieldType::Money => "money",
        FieldType::Percentage => "percentage",
        FieldType::Boolean => "boolean",
        FieldType::Date => "date",
        FieldType::DateTime => "datetime",
        FieldType::File => "file",
        FieldType::Ulid => "ulid",
        FieldType::Json => "json",
        FieldType::Enum => "enum",
        FieldType::Ip => "ip",
        FieldType::Relation => "relation",
    }
}

fn method(m: &HttpMethod) -> &'static str {
    match m {
        HttpMethod::GET => "GET",
        HttpMethod::POST => "POST",
        HttpMethod::PATCH => "PATCH",
        HttpMethod::PUT => "PUT",
        HttpMethod::DELETE => "DELETE",
    }
}

fn auth(a: &AuthNode) -> Value {
    let mut m = Map::new();
    m.insert("entity".into(), json!(a.entity));
    put_list(&mut m, "login", &a.login_fields);
    m.insert("session".into(), json!(a.session_type));
    put_map(&mut m, "session_config", &a.session_config);
    put_list(&mut m, "roles", &a.roles);
    Value::Object(m)
}

fn field(f: &FieldNode) -> Value {
    let mut m = Map::new();
    m.insert("name".into(), json!(f.name));
    m.insert("type".into(), json!(field_type(&f.field_type)));
    m.insert("required".into(), json!(f.required));
    let modifiers: Vec<&str> = [
        ("unique", f.unique),
        ("sensitive", f.sensitive),
        ("optional", f.optional),
        ("searchable", f.searchable),
        ("index", f.index),
        ("featured", f.featured),
        ("formatted", f.formatted),
        ("array", f.array),
    ]
    .into_iter()
    .filter_map(|(name, on)| on.then_some(name))
    .collect();
    put_list(&mut m, "modifiers", &modifiers);
    put_opt(&mut m, "enum_values", &f.enum_values);
    put_opt(&mut m, "reference", &f.reference);
    put_opt(&mut m, "default", &f.default_value);
    put_opt(&mut m, "min", &f.min);
    put_opt(&mut m, "max", &f.max);
    put_opt(&mut m, "min_length", &f.min_length);
    put_opt(&mut m, "max_length", &f.max_length);
    put_opt(&mut m, "pattern", &f.pattern);
    Value::Object(m)
}

fn entity(e: &EntityNode) -> Value {
    let mut m = Map::new();
    m.insert("name".into(), json!(e.name));
    if e.shared {
        m.insert("shared".into(), json!(true));
    }
    m.insert(
        "fields".into(),
        Value::Array(e.fields.iter().map(field).collect()),
    );
    let transitions: Vec<Value> = e
        .transitions
        .iter()
        .map(|t| {
            let rules: Vec<Value> = t
                .rules
                .iter()
                .map(|r| json!({"from": r.from, "to": r.to}))
                .collect();
            json!({"field": t.field, "rules": rules})
        })
        .collect();
    put_list(&mut m, "transitions", &transitions);
    let effects: Vec<Value> = e
        .effects
        .iter()
        .map(|fx| {
            let mut x = Map::new();
            x.insert("on".into(), json!(fx.event));
            put_opt(&mut x, "field", &fx.field);
            let actions: Vec<&str> = fx.actions.iter().map(|a| a.action_type.as_str()).collect();
            put_list(&mut x, "actions", &actions);
            Value::Object(x)
        })
        .collect();
    put_list(&mut m, "effects", &effects);
    put_opt(&mut m, "remote_url", &e.remote_url);
    Value::Object(m)
}

fn api(a: &ApiNode) -> Value {
    let routes: Vec<Value> = a
        .routes
        .iter()
        .map(|r| {
            let mut m = Map::new();
            m.insert("name".into(), json!(r.name));
            m.insert("method".into(), json!(method(&r.method)));
            m.insert("path".into(), json!(r.path));
            m.insert("auth".into(), json!(r.auth));
            put_list(&mut m, "roles", &r.roles);
            Value::Object(m)
        })
        .collect();
    json!({"prefix": a.prefix, "routes": routes})
}

fn binding(b: &BindingNode) -> Value {
    let mut m = Map::new();
    m.insert("entity".into(), json!(b.entity));
    let query = match b.query {
        QueryType::All => "all",
        QueryType::One => "one",
        QueryType::Count => "count",
    };
    m.insert("query".into(), json!(query));
    if !b.expand.is_empty() {
        m.insert("expand".into(), json!(b.expand));
    }
    let filters: Vec<Value> = b
        .filters
        .iter()
        .map(|f| {
            let op = match f.operator {
                FilterOp::Eq => "eq",
                FilterOp::Ne => "ne",
                FilterOp::Gt => "gt",
                FilterOp::Gte => "gte",
                FilterOp::Lt => "lt",
                FilterOp::Lte => "lte",
                FilterOp::Contains => "contains",
                FilterOp::StartsWith => "starts_with",
                FilterOp::EndsWith => "ends_with",
                FilterOp::In => "in",
            };
            let value = match &f.value {
                BindingValue::Str(s) => json!(s),
                BindingValue::Num(n) => n
                    .parse::<f64>()
                    .ok()
                    .and_then(serde_json::Number::from_f64)
                    .map_or_else(|| json!(n), Value::Number),
                BindingValue::Bool(v) => json!(v),
                BindingValue::AuthRef(r) => json!({ "ref": r }),
                BindingValue::List(items) => json!(items
                    .iter()
                    .map(|v| match v {
                        BindingValue::Str(s) => json!(s),
                        BindingValue::Num(n) => json!(n),
                        BindingValue::Bool(b) => json!(b),
                        BindingValue::AuthRef(r) => json!({ "ref": r }),
                        BindingValue::List(_) => json!([]),
                    })
                    .collect::<Vec<_>>()),
            };
            json!({"field": f.field, "op": op, "value": value})
        })
        .collect();
    put_list(&mut m, "filters", &filters);
    if let Some(o) = &b.order {
        let dir = match o.direction {
            OrderDirection::Asc => "asc",
            OrderDirection::Desc => "desc",
        };
        m.insert("order".into(), json!({"field": o.field, "direction": dir}));
    }
    put_opt(&mut m, "limit", &b.limit);
    put_opt(&mut m, "offset", &b.offset);
    if let Some(a) = &b.aggregate {
        let mut x = Map::new();
        x.insert("function".into(), json!(a.function));
        put_opt(&mut x, "field", &a.field);
        m.insert("aggregate".into(), Value::Object(x));
    }
    if let Some(g) = &b.group_by {
        let mut x = Map::new();
        x.insert("field".into(), json!(g.field));
        put_opt(&mut x, "interval", &g.interval);
        m.insert("group_by".into(), Value::Object(x));
    }
    if b.live {
        m.insert("live".into(), json!(true));
    }
    if b.public {
        m.insert("public".into(), json!(true));
    }
    Value::Object(m)
}

fn section(s: &SectionNode) -> Value {
    let mut m = Map::new();
    m.insert("type".into(), json!(s.section_type));
    put_opt(&mut m, "title", &s.title);
    if let Some(b) = &s.binding {
        m.insert("binding".into(), binding(b));
    }
    if !s.items.is_empty() {
        m.insert("items".into(), json!(s.items.len()));
    }
    if !s.plans.is_empty() {
        m.insert("plans".into(), json!(s.plans.len()));
    }
    put_map(&mut m, "config", &s.config);
    let actions: Vec<Value> = s
        .actions
        .iter()
        .map(|a| {
            let verbs: Vec<&str> = a.instructions.iter().map(|i| i.verb.as_str()).collect();
            json!({"on": a.event, "verbs": verbs})
        })
        .collect();
    put_list(&mut m, "actions", &actions);
    Value::Object(m)
}

fn page(p: &PageNode) -> Value {
    let mut m = Map::new();
    m.insert("route".into(), json!(p.route));
    m.insert("type".into(), json!(p.page_type));
    put_opt(&mut m, "title", &p.title);
    put_opt(&mut m, "entity", &p.entity);
    put_opt(&mut m, "requires", &p.requires);
    put_list(&mut m, "components", &p.components);
    m.insert(
        "sections".into(),
        Value::Array(p.sections.iter().map(section).collect()),
    );
    Value::Object(m)
}

fn layout(l: &LayoutNode) -> Value {
    let nav: Vec<Value> = l
        .sidebar_items
        .iter()
        .filter(|i| !i.is_divider)
        .map(|i| {
            let mut m = Map::new();
            m.insert("label".into(), json!(i.label));
            m.insert("route".into(), json!(i.route));
            put_opt(&mut m, "icon", &i.icon);
            put_opt(&mut m, "requires", &i.requires);
            Value::Object(m)
        })
        .collect();
    let mut m = Map::new();
    m.insert("name".into(), json!(l.name));
    put_list(&mut m, "nav", &nav);
    put_map(&mut m, "sidebar", &l.sidebar_config);
    put_map(&mut m, "topbar", &l.topbar_config);
    Value::Object(m)
}

fn style(s: &StyleNode) -> Value {
    let mut m = Map::new();
    put_opt(&mut m, "theme", &s.theme);
    put_opt(&mut m, "accent", &s.accent);
    put_opt(&mut m, "radius", &s.radius);
    put_opt(&mut m, "font", &s.font);
    put_map(&mut m, "config", &s.config);
    Value::Object(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_ast_summary_covers_blocks_and_omits_empty_keys() {
        let src = "app \"Crm\" {\n  port 5180\n}\nauth {\n  entity User\n  login email + password\n  session jwt expires:24h\n  roles [admin, user]\n}\nentity User {\n  email email! unique\n  password string! sensitive\n}\nentity Deal {\n  title string!\n  stage enum [lead, won]\n  owner -> User\n  transition stage { lead -> won }\n}\npage \"/deals\" requires:auth {\n  section kpi { bind Deal { aggregate count } item \"Deals\" value:bind }\n}\n";
        let nodes = parser::parse(src).expect("parses");
        let s = summarize(&nodes);
        assert_eq!(s["app"]["port"], 5180);
        assert_eq!(s["auth"]["entity"], "User");
        assert_eq!(s["auth"]["roles"], json!(["admin", "user"]));
        let user = &s["entities"][0];
        assert_eq!(user["fields"][1]["modifiers"], json!(["sensitive"]));
        let deal = &s["entities"][1];
        assert_eq!(deal["fields"][2]["type"], "relation");
        assert_eq!(deal["fields"][2]["reference"], "User");
        assert_eq!(deal["transitions"][0]["rules"][0]["to"], json!(["won"]));
        assert!(deal.get("shared").is_none());
        let kpi = &s["pages"][0]["sections"][0];
        assert_eq!(kpi["binding"]["aggregate"]["function"], "count");
        assert_eq!(kpi["items"], 1);
        assert_eq!(s["pages"][0]["requires"], "auth");
        assert_eq!(s["stats"]["entities"], 2);
    }
}
