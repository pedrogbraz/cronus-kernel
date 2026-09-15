//! CRONUS Resolve Pass — Symbol Table & Cross-Reference Validation
//!
//! Runs after parsing, before lint. Two-pass analysis:
//!   Pass 1: Collect all symbols (entities, pages, APIs, routes)
//!   Pass 2: Resolve all cross-references and report errors with suggestions
//!
//! Resolve errors are always fatal — they block build/run.

use crate::parser::{AstNode, FieldType, SectionNode};
use std::collections::{HashMap, HashSet};

// ══════════════════════════════════════════════════
// TYPES
// ══════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub entities: HashMap<String, EntityInfo>,
    pub pages: HashSet<String>,
    pub api_prefixes: HashSet<String>,
    pub kernel_routes: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct EntityInfo {
    pub fields: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct ResolveError {
    pub message: String,
    pub suggestion: Option<String>,
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "  \x1b[31m✗\x1b[0m [resolve] {}", self.message)?;
        if let Some(ref sug) = self.suggestion {
            write!(f, " Did you mean '{}'?", sug)?;
        }
        Ok(())
    }
}

impl SymbolTable {
    fn new() -> Self {
        let mut kernel_routes = HashSet::new();
        kernel_routes.insert("/docs".to_string());
        kernel_routes.insert("/docs/design".to_string());
        kernel_routes.insert("/graphql".to_string());

        Self {
            entities: HashMap::new(),
            pages: HashSet::new(),
            api_prefixes: HashSet::new(),
            kernel_routes,
        }
    }

    /// All known routes (page routes + kernel routes)
    fn all_routes(&self) -> Vec<&str> {
        self.pages
            .iter()
            .map(|s| s.as_str())
            .chain(self.kernel_routes.iter().map(|s| s.as_str()))
            .collect()
    }
}

// ══════════════════════════════════════════════════
// MAIN ENTRY POINT
// ══════════════════════════════════════════════════

/// Resolve all cross-references in the AST. Returns the symbol table and any errors.
/// Errors are fatal — caller should abort if non-empty.
pub fn resolve(nodes: &[AstNode]) -> (SymbolTable, Vec<ResolveError>) {
    let mut table = SymbolTable::new();
    let mut errors = Vec::new();

    // ── Pass 1: Collect symbols ──
    collect_symbols(nodes, &mut table);

    // ── Pass 2: Resolve references ──
    resolve_references(nodes, &table, &mut errors);

    (table, errors)
}

// ══════════════════════════════════════════════════
// PASS 1 — COLLECT SYMBOLS
// ══════════════════════════════════════════════════

fn collect_symbols(nodes: &[AstNode], table: &mut SymbolTable) {
    for node in nodes {
        match node {
            AstNode::Entity(e) => {
                let fields = e.fields.iter().map(|f| f.name.clone()).collect();
                table.entities.insert(e.name.clone(), EntityInfo { fields });
            }
            AstNode::Page(p) => {
                table.pages.insert(p.route.clone());
            }
            AstNode::Api(a) => {
                table.api_prefixes.insert(a.prefix.clone());
                // Register API routes as known routes
                for route in &a.routes {
                    let full = format!("{}{}", a.prefix, route.path);
                    table.kernel_routes.insert(full);
                }
            }
            _ => {}
        }
    }
}

// ══════════════════════════════════════════════════
// PASS 2 — RESOLVE REFERENCES
// ══════════════════════════════════════════════════

fn resolve_references(nodes: &[AstNode], table: &SymbolTable, errors: &mut Vec<ResolveError>) {
    let entity_names: Vec<&str> = table.entities.keys().map(|s| s.as_str()).collect();

    for node in nodes {
        match node {
            AstNode::Page(p) => {
                // Check page-level entity reference
                if let Some(ref ent) = p.entity {
                    if !table.entities.contains_key(ent) {
                        errors.push(ResolveError {
                            message: format!("Entity '{}' not found (page '{}')", ent, p.route),
                            suggestion: find_closest(ent, &entity_names),
                        });
                    }
                }

                // Check section-level bindings and columns
                for section in &p.sections {
                    resolve_section(section, &p.route, table, &entity_names, errors);
                }
            }
            AstNode::Entity(e) => {
                // Check relation references (-> Entity)
                for field in &e.fields {
                    if field.field_type == FieldType::Relation {
                        if let Some(ref target) = field.reference {
                            if !table.entities.contains_key(target) {
                                errors.push(ResolveError {
                                    message: format!(
                                        "Entity '{}' not found in relation '{}' -> '{}' (entity '{}')",
                                        target, field.name, target, e.name
                                    ),
                                    suggestion: find_closest(target, &entity_names),
                                });
                            }
                        }
                    }
                }

                // Check transition fields exist and are enum
                for transition in &e.transitions {
                    let field_opt = e.fields.iter().find(|f| f.name == transition.field);
                    match field_opt {
                        None => {
                            let field_names: Vec<&str> =
                                e.fields.iter().map(|f| f.name.as_str()).collect();
                            errors.push(ResolveError {
                                message: format!(
                                    "Transition field '{}' not found in entity '{}'",
                                    transition.field, e.name
                                ),
                                suggestion: find_closest(&transition.field, &field_names),
                            });
                        }
                        Some(f) if f.field_type != FieldType::Enum => {
                            errors.push(ResolveError {
                                message: format!(
                                    "Transition field '{}' in entity '{}' must be an enum, found {:?}",
                                    transition.field, e.name, f.field_type
                                ),
                                suggestion: None,
                            });
                        }
                        _ => {} // OK
                    }
                }
            }
            AstNode::Define(d) => {
                for section in &d.sections {
                    resolve_section(
                        section,
                        &format!("define:{}", d.name),
                        table,
                        &entity_names,
                        errors,
                    );
                }
            }
            AstNode::Webhook(w) => {
                // Webhook entity can be path-style ("deployments") or name-style ("Deployment")
                let wh_entity = w.entity.trim_start_matches('/');
                let entity_matches = table.entities.keys().any(|e| {
                    e == wh_entity
                        || e.to_lowercase() == wh_entity.to_lowercase()
                        || format!("{}s", e.to_lowercase()) == wh_entity.to_lowercase()
                        || e.to_lowercase() == wh_entity.to_lowercase().trim_end_matches('s')
                });
                if !entity_matches {
                    errors.push(ResolveError {
                        message: format!("Entity '{}' not found (webhook)", w.entity),
                        suggestion: find_closest(wh_entity, &entity_names),
                    });
                }
            }
            _ => {}
        }
    }
}

fn resolve_section(
    section: &SectionNode,
    context: &str,
    table: &SymbolTable,
    entity_names: &[&str],
    errors: &mut Vec<ResolveError>,
) {
    // Check binding entity exists
    if let Some(ref binding) = section.binding {
        if !table.entities.contains_key(&binding.entity) {
            errors.push(ResolveError {
                message: format!(
                    "Entity '{}' not found in bind (section '{}', {})",
                    binding.entity, section.section_type, context
                ),
                suggestion: find_closest(&binding.entity, entity_names),
            });
        } else {
            // Entity exists — check filter/order/group_by fields resolve
            let entity = &table.entities[&binding.entity];
            let auto_fields = ["id", "created_at", "updated_at", "_owner_id"];
            let mut all_field_names: Vec<String> = entity.fields.iter().cloned().collect();
            for af in &auto_fields {
                all_field_names.push(af.to_string());
            }
            let field_refs: Vec<&str> = all_field_names.iter().map(|s| s.as_str()).collect();

            for filter in &binding.filters {
                if !all_field_names.contains(&filter.field) {
                    errors.push(ResolveError {
                        message: format!(
                            "Field '{}' not found in entity '{}' (filter in {}, {})",
                            filter.field, binding.entity, section.section_type, context
                        ),
                        suggestion: find_closest(&filter.field, &field_refs),
                    });
                }
            }

            // Check order field
            if let Some(ref order) = binding.order {
                if !all_field_names.contains(&order.field) {
                    errors.push(ResolveError {
                        message: format!(
                            "Field '{}' not found in entity '{}' (order in {}, {})",
                            order.field, binding.entity, section.section_type, context
                        ),
                        suggestion: find_closest(&order.field, &field_refs),
                    });
                }
            }

            // Check group_by field
            if let Some(ref group) = binding.group_by {
                if !all_field_names.contains(&group.field) {
                    errors.push(ResolveError {
                        message: format!(
                            "Field '{}' not found in entity '{}' (group_by in {}, {})",
                            group.field, binding.entity, section.section_type, context
                        ),
                        suggestion: find_closest(&group.field, &field_refs),
                    });
                }
            }

            for name in &binding.expand {
                if !all_field_names.contains(name) {
                    errors.push(ResolveError {
                        message: format!(
                            "Field '{}' not found in entity '{}' (expand in {}, {})",
                            name, binding.entity, section.section_type, context
                        ),
                        suggestion: find_closest(name, &field_refs),
                    });
                }
            }

            // Check aggregate field
            if let Some(ref agg) = binding.aggregate {
                if let Some(ref agg_field) = agg.field {
                    if !all_field_names.contains(agg_field) {
                        errors.push(ResolveError {
                            message: format!(
                                "Field '{}' not found in entity '{}' (aggregate in {}, {})",
                                agg_field, binding.entity, section.section_type, context
                            ),
                            suggestion: find_closest(agg_field, &field_refs),
                        });
                    }
                }
            }
        }
    }

    // Check columns reference valid fields of the bound entity
    if let Some(columns_str) = section.config.get("columns") {
        if let Some(ref binding) = section.binding {
            if let Some(entity) = table.entities.get(&binding.entity) {
                // Include auto-generated fields that the kernel adds
                let auto_fields = vec!["id", "created_at", "updated_at", "_owner_id"];
                let mut all_fields: Vec<String> = entity.fields.iter().cloned().collect();
                for af in &auto_fields {
                    all_fields.push(af.to_string());
                }
                let columns: Vec<&str> = columns_str.split(',').map(|c| c.trim()).collect();
                for col in columns {
                    if col.is_empty() {
                        continue;
                    }
                    // Normalize: "Deploy ID" → "deploy_id", "Source IP" → "source_ip"
                    let normalized = col.to_lowercase().replace(' ', "_");
                    let matches = all_fields
                        .iter()
                        .any(|f| f == col || f == &normalized || f.to_lowercase() == normalized);
                    if !matches {
                        let field_refs: Vec<&str> = all_fields.iter().map(|s| s.as_str()).collect();
                        errors.push(ResolveError {
                            message: format!(
                                "Column '{}' not found as field in entity '{}' ({}, {})",
                                col, binding.entity, section.section_type, context
                            ),
                            suggestion: find_closest(&normalized, &field_refs),
                        });
                    }
                }
            }
        }
    }

    // Check href references in templates
    if let Some(ref template) = section.template {
        check_template_hrefs(template, context, &section.section_type, table, errors);
    }
}

/// Scan HTML template for href="/path" and verify routes exist
fn check_template_hrefs(
    template: &str,
    context: &str,
    section_type: &str,
    table: &SymbolTable,
    errors: &mut Vec<ResolveError>,
) {
    // Match href="..." patterns (skip dynamic {{ }}, external URLs, anchors)
    let mut pos = 0;
    let bytes = template.as_bytes();
    while pos < bytes.len() {
        if let Some(idx) = template[pos..].find("href=\"") {
            let start = pos + idx + 6;
            if let Some(end_offset) = template[start..].find('"') {
                let href = &template[start..start + end_offset];
                // Skip dynamic bindings, external URLs, anchors, javascript:
                if !href.starts_with("{{")
                    && !href.starts_with("http")
                    && !href.starts_with("#")
                    && !href.starts_with("javascript:")
                    && !href.starts_with("mailto:")
                    && !href.is_empty()
                {
                    // Normalize: strip query string and trailing slash
                    let route = href.split('?').next().unwrap_or(href);
                    let route = route.trim_end_matches('/');
                    let route = if route.is_empty() { "/" } else { route };

                    // Check if route exists in pages or kernel routes
                    if !table.pages.contains(route) && !table.kernel_routes.contains(route) {
                        let all_routes = table.all_routes();
                        errors.push(ResolveError {
                            message: format!(
                                "Route '{}' not found (href in template, {} {})",
                                route, section_type, context
                            ),
                            suggestion: find_closest(route, &all_routes),
                        });
                    }
                }
                pos = start + end_offset + 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }
}

// ══════════════════════════════════════════════════
// STRING SIMILARITY
// ══════════════════════════════════════════════════

/// Levenshtein edit distance
fn edit_distance(a: &str, b: &str) -> usize {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let m = a_bytes.len();
    let n = b_bytes.len();

    let mut prev = (0..=n).collect::<Vec<_>>();
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_bytes[i - 1] == b_bytes[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

/// Find the closest matching string using edit distance
fn find_closest<'a>(needle: &str, candidates: &[&'a str]) -> Option<String> {
    if candidates.is_empty() {
        return None;
    }

    let needle_lower = needle.to_lowercase();
    let mut best: Option<(&str, usize)> = None;

    for &candidate in candidates {
        let dist = edit_distance(&needle_lower, &candidate.to_lowercase());
        let threshold = (needle.len() / 2).max(3);
        if dist <= threshold && dist > 0 {
            if best.is_none() || dist < best.unwrap().1 {
                best = Some((candidate, dist));
            }
        }
    }

    best.map(|(s, _)| s.to_string())
}

// ══════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::*;
    use std::collections::HashMap;

    fn make_field(name: &str, ft: FieldType) -> FieldNode {
        FieldNode {
            name: name.into(),
            field_type: ft,
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
            reference: None,
            doc: None,
            default_value: None,
            min: None,
            max: None,
            min_length: None,
            max_length: None,
            pattern: None,
        }
    }

    fn make_enum_field(name: &str, values: Vec<&str>) -> FieldNode {
        let mut f = make_field(name, FieldType::Enum);
        f.enum_values = Some(values.into_iter().map(String::from).collect());
        f
    }

    fn make_relation_field(name: &str, target: &str) -> FieldNode {
        let mut f = make_field(name, FieldType::Relation);
        f.reference = Some(target.into());
        f
    }

    fn make_entity(name: &str, fields: Vec<FieldNode>) -> AstNode {
        AstNode::Entity(EntityNode {
            name: name.into(),
            fields,
            transitions: vec![],
            effects: vec![],
            shared: false,
            remote_url: None,
            doc: None,
        })
    }

    fn make_page(route: &str, sections: Vec<SectionNode>) -> AstNode {
        AstNode::Page(PageNode {
            route: route.into(),
            page_type: "custom".into(),
            entity: None,
            title: None,
            sections,
            config: HashMap::new(),
            components: vec![],
            requires: None,
            doc: None,
        })
    }

    fn make_section() -> SectionNode {
        SectionNode {
            section_type: "hero".into(),
            title: None,
            subtitle: None,
            config: HashMap::new(),
            items: vec![],
            plans: vec![],
            binding: None,
            actions: vec![],
            visibility: None,
            template: None,
            style_block: None,
            doc: None,
        }
    }

    fn make_binding(entity: &str) -> BindingNode {
        BindingNode {
            entity: entity.into(),
            query: QueryType::All,
            filters: vec![],
            order: None,
            limit: None,
            offset: None,
            group_by: None,
            aggregate: None,
            live: false,
            public: false,
            expand: vec![],
        }
    }

    // ── Test 1: Valid references resolve without errors ──

    #[test]
    fn valid_references_resolve_clean() {
        let nodes = vec![
            make_entity(
                "Product",
                vec![
                    make_field("name", FieldType::String),
                    make_field("price", FieldType::Money),
                ],
            ),
            make_entity(
                "Order",
                vec![
                    make_field("total", FieldType::Money),
                    make_relation_field("product", "Product"),
                ],
            ),
            make_page(
                "/products",
                vec![{
                    let mut s = make_section();
                    s.binding = Some(make_binding("Product"));
                    s
                }],
            ),
        ];

        let (table, errors) = resolve(&nodes);
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
        assert_eq!(table.entities.len(), 2);
        assert_eq!(table.pages.len(), 1);
    }

    // ── Test 2: Missing entity in bind produces error with suggestion ──

    #[test]
    fn missing_entity_in_bind_suggests_closest() {
        let nodes = vec![
            make_entity("Deployment", vec![make_field("name", FieldType::String)]),
            make_page(
                "/deployments",
                vec![{
                    let mut s = make_section();
                    s.binding = Some(make_binding("Deploment")); // typo
                    s
                }],
            ),
        ];

        let (_, errors) = resolve(&nodes);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Deploment"));
        assert_eq!(errors[0].suggestion.as_deref(), Some("Deployment"));
    }

    // ── Test 3: Missing entity in relation produces error ──

    #[test]
    fn missing_entity_in_relation() {
        let nodes = vec![make_entity(
            "Order",
            vec![make_relation_field("customer", "Customer")],
        )];

        let (_, errors) = resolve(&nodes);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Customer"));
        assert!(errors[0].message.contains("relation"));
    }

    // ── Test 4: Missing field in columns produces error ──

    #[test]
    fn missing_field_in_columns() {
        let nodes = vec![
            make_entity(
                "Product",
                vec![
                    make_field("name", FieldType::String),
                    make_field("price", FieldType::Money),
                ],
            ),
            make_page(
                "/products",
                vec![{
                    let mut s = make_section();
                    s.section_type = "table".into();
                    s.binding = Some(make_binding("Product"));
                    s.config
                        .insert("columns".into(), "name, price, stok".into()); // typo: stok
                    s
                }],
            ),
        ];

        let (_, errors) = resolve(&nodes);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("stok"));
        assert!(errors[0].message.contains("Product"));
    }

    // ── Test 5: Transition on non-existent field ──

    #[test]
    fn transition_on_missing_field() {
        let nodes = vec![AstNode::Entity(EntityNode {
            name: "Task".into(),
            fields: vec![make_field("title", FieldType::String)],
            transitions: vec![TransitionNode {
                field: "status".into(),
                rules: vec![],
            }],
            effects: vec![],
            shared: false,
            remote_url: None,
            doc: None,
        })];

        let (_, errors) = resolve(&nodes);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("status"));
        assert!(errors[0].message.contains("Task"));
    }

    // ── Test 6: Transition on non-enum field ──

    #[test]
    fn transition_on_non_enum_field() {
        let nodes = vec![AstNode::Entity(EntityNode {
            name: "Task".into(),
            fields: vec![make_field("status", FieldType::String)],
            transitions: vec![TransitionNode {
                field: "status".into(),
                rules: vec![],
            }],
            effects: vec![],
            shared: false,
            remote_url: None,
            doc: None,
        })];

        let (_, errors) = resolve(&nodes);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("must be an enum"));
    }

    // ── Test 7: Valid transition on enum field ──

    #[test]
    fn valid_transition_on_enum() {
        let nodes = vec![AstNode::Entity(EntityNode {
            name: "Task".into(),
            fields: vec![make_enum_field("status", vec!["open", "closed"])],
            transitions: vec![TransitionNode {
                field: "status".into(),
                rules: vec![TransitionRule {
                    from: "open".into(),
                    to: vec!["closed".into()],
                }],
            }],
            effects: vec![],
            shared: false,
            remote_url: None,
            doc: None,
        })];

        let (_, errors) = resolve(&nodes);
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    // ── Test 8: Dead href in template ──

    #[test]
    fn dead_href_in_template() {
        let nodes = vec![make_page(
            "/home",
            vec![{
                let mut s = make_section();
                s.template = Some(r#"<a href="/nonexistent">link</a>"#.into());
                s
            }],
        )];

        let (_, errors) = resolve(&nodes);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("/nonexistent"));
    }

    // ── Test 9: Valid href to known page ──

    #[test]
    fn valid_href_resolves() {
        let nodes = vec![
            make_page("/about", vec![make_section()]),
            make_page(
                "/home",
                vec![{
                    let mut s = make_section();
                    s.template = Some(r#"<a href="/about">About</a>"#.into());
                    s
                }],
            ),
        ];

        let (_, errors) = resolve(&nodes);
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    // ── Test 10: Symbol table counts are correct ──

    #[test]
    fn symbol_table_counts() {
        let nodes = vec![
            make_entity(
                "User",
                vec![
                    make_field("name", FieldType::String),
                    make_field("email", FieldType::Email),
                ],
            ),
            make_entity(
                "Product",
                vec![
                    make_field("title", FieldType::String),
                    make_field("price", FieldType::Money),
                    make_field("active", FieldType::Boolean),
                ],
            ),
            make_page("/users", vec![]),
            make_page("/products", vec![]),
            make_page("/dashboard", vec![]),
            AstNode::Api(ApiNode {
                prefix: "/api/v1".into(),
                routes: vec![],
                doc: None,
            }),
        ];

        let (table, errors) = resolve(&nodes);
        assert!(errors.is_empty());
        assert_eq!(table.entities.len(), 2);
        assert_eq!(table.entities["User"].fields.len(), 2);
        assert_eq!(table.entities["Product"].fields.len(), 3);
        assert_eq!(table.pages.len(), 3);
        assert_eq!(table.api_prefixes.len(), 1);
        assert!(table.api_prefixes.contains("/api/v1"));
    }

    // ── Test 11: Webhook with unknown entity ──

    #[test]
    fn webhook_unknown_entity() {
        let nodes = vec![
            make_entity("Order", vec![make_field("total", FieldType::Money)]),
            AstNode::Webhook(WebhookNode {
                entity: "Ordr".into(),
                hooks: vec![],
            }),
        ];

        let (_, errors) = resolve(&nodes);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Ordr"));
        assert_eq!(errors[0].suggestion.as_deref(), Some("Order"));
    }

    // ── Test 12: Filter field that doesn't exist in entity ──

    #[test]
    fn filter_on_nonexistent_field() {
        let nodes = vec![
            make_entity(
                "Product",
                vec![
                    make_field("name", FieldType::String),
                    make_field("active", FieldType::Boolean),
                ],
            ),
            make_page(
                "/products",
                vec![{
                    let mut s = make_section();
                    s.binding = Some(BindingNode {
                        entity: "Product".into(),
                        query: QueryType::All,
                        filters: vec![FilterExpr {
                            field: "actve".into(), // typo
                            operator: FilterOp::Eq,
                            value: BindingValue::Bool(true),
                        }],
                        order: None,
                        limit: None,
                        offset: None,
                        group_by: None,
                        aggregate: None,
                        live: false,
                        public: false,
                        expand: vec![],
                    });
                    s
                }],
            ),
        ];

        let (_, errors) = resolve(&nodes);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("actve"));
        assert_eq!(errors[0].suggestion.as_deref(), Some("active"));
    }

    // ── Test 13: Edit distance function ──

    #[test]
    fn edit_distance_basic() {
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("abc", "abc"), 0);
        assert_eq!(edit_distance("Deploment", "Deployment"), 1);
    }

    // ── Test 14: Suggestion with no close match returns None ──

    #[test]
    fn no_suggestion_for_wildly_different() {
        let candidates = vec!["User", "Product", "Order"];
        let result = find_closest("XyzAbcFooBarBaz", &candidates);
        assert!(result.is_none());
    }

    // ── Test 15: Page-level entity reference ──

    #[test]
    fn page_level_entity_reference() {
        let nodes = vec![
            make_entity("User", vec![make_field("name", FieldType::String)]),
            AstNode::Page(PageNode {
                route: "/users".into(),
                page_type: "list".into(),
                entity: Some("Usr".into()), // typo
                title: None,
                sections: vec![],
                config: HashMap::new(),
                components: vec![],
                requires: None,
                doc: None,
            }),
        ];

        let (_, errors) = resolve(&nodes);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Usr"));
        assert_eq!(errors[0].suggestion.as_deref(), Some("User"));
    }

    // ── Test 16: href to kernel route resolves ──

    #[test]
    fn href_to_kernel_route_resolves() {
        let nodes = vec![make_page(
            "/home",
            vec![{
                let mut s = make_section();
                s.template = Some(r#"<a href="/docs">Docs</a> <a href="/graphql">API</a>"#.into());
                s
            }],
        )];

        let (_, errors) = resolve(&nodes);
        assert!(
            errors.is_empty(),
            "Kernel routes should resolve: {:?}",
            errors
        );
    }
}
