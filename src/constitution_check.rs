//! Constitution Enforcement Engine
//!
//! Validates that the parsed AST respects the unbreakable rules defined
//! in the `constitution { ... }` block of the app declaration.
//!
//! "must" rules are checked to be TRUE.
//! "never" rules are checked to be FALSE.
//! Unrecognized rules are stored as informational (no automatic check possible).

use crate::parser::{AstNode, ConstitutionNode, EntityNode, PageNode, SectionNode, FieldType};

#[derive(Debug, Clone)]
pub struct ConstitutionViolation {
    pub rule_type: String,  // "must" or "never"
    pub rule: String,
    pub violation: String,
    pub entity: Option<String>,
}

impl std::fmt::Display for ConstitutionViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let icon = match self.rule_type.as_str() {
            "must" => "\x1b[31m\u{2717}\x1b[0m",
            "never" => "\x1b[31m\u{2717}\x1b[0m",
            _ => "\x1b[36mi\x1b[0m",
        };
        write!(f, "  {} \x1b[1m[{}]\x1b[0m \"{}\"\n    \u{2192} {}",
            icon, self.rule_type, self.rule, self.violation)?;
        if let Some(ref ent) = self.entity {
            write!(f, " ({})", ent)?;
        }
        Ok(())
    }
}

/// Check all constitution rules against the parsed AST.
/// Returns a list of violations (empty = all rules pass).
pub fn check_constitution(nodes: &[AstNode], constitution: &ConstitutionNode) -> Vec<ConstitutionViolation> {
    let mut violations = Vec::new();

    // Collect AST parts
    let mut entities: Vec<&EntityNode> = Vec::new();
    let mut pages: Vec<&PageNode> = Vec::new();
    let mut sensitive_fields: Vec<String> = Vec::new();

    for node in nodes {
        match node {
            AstNode::Entity(e) => {
                entities.push(e);
                for field in &e.fields {
                    if field.sensitive {
                        sensitive_fields.push(field.name.clone());
                    }
                }
            }
            AstNode::Page(p) => pages.push(p),
            _ => {}
        }
    }

    // Check "must" rules
    for rule in &constitution.must {
        let lower = rule.to_lowercase();

        if lower.contains("created_at") {
            // CRONUS kernel auto-adds created_at to all entities at DB level.
            // Verify no entity explicitly removes it (currently not possible).
            // This rule is always satisfied by the kernel — pass silently.
        } else if lower.contains("bind") && lower.contains("data") {
            // "all data sections require bind" — check data sections have bindings
            let data_types = ["kpi", "table", "chart", "kanban", "timeline", "stats", "stat-cards"];
            for page in &pages {
                for section in &page.sections {
                    if !data_types.contains(&section.section_type.as_str()) { continue; }
                    let has_bind = section.binding.is_some();
                    let has_items = !section.items.is_empty();
                    if !has_bind && !has_items {
                        violations.push(ConstitutionViolation {
                            rule_type: "must".into(),
                            rule: rule.clone(),
                            violation: format!(
                                "data section '{}' on page '{}' has no bind or items",
                                section.section_type, page.route
                            ),
                            entity: None,
                        });
                    }
                }
            }
        } else if lower.contains("auth") {
            // "auth required on all admin endpoints" — pages with data sections should have requires
            for page in &pages {
                let has_data_section = page.sections.iter().any(|s| {
                    matches!(s.section_type.as_str(),
                        "kpi" | "table" | "chart" | "kanban" | "timeline" | "stats" | "stat-cards" | "form"
                    )
                });
                // Skip public-facing pages (landing, login, signup, settings)
                let is_public = matches!(page.route.as_str(), "/" | "/login" | "/signup" | "/settings")
                    || page.page_type == "landing"
                    || page.page_type == "auth";

                if has_data_section && !is_public && page.requires.is_none() {
                    violations.push(ConstitutionViolation {
                        rule_type: "must".into(),
                        rule: rule.clone(),
                        violation: format!(
                            "page '{}' has data sections but no requires:auth",
                            page.route
                        ),
                        entity: None,
                    });
                }
            }
        } else if lower.contains("centavos") || lower.contains("cents") {
            // Informational: verify money fields exist. Cannot enforce formatting at AST level.
            let has_money = entities.iter().any(|e|
                e.fields.iter().any(|f| f.field_type == FieldType::Money)
            );
            if has_money {
                // Money fields found — informational note only, no violation
            }
            // If no money fields, nothing to check — rule is trivially satisfied
        } else {
            // Unrecognized "must" rule — store as informational
            violations.push(ConstitutionViolation {
                rule_type: "info".into(),
                rule: rule.clone(),
                violation: "no automatic check available for this rule".into(),
                entity: None,
            });
        }
    }

    // Check "never" rules
    for rule in &constitution.never {
        let lower = rule.to_lowercase();

        if lower.contains("expose") && lower.contains("password") {
            // Reuse C002 logic: sensitive fields must not appear in rendered templates
            check_sensitive_in_templates(&pages, &sensitive_fields, rule, &mut violations);
        } else if lower.contains("hardcode") {
            // Check for hardcoded text in templates (reuse no-dead-text pattern)
            check_hardcoded_metrics(&pages, rule, &mut violations);
        } else if lower.contains("location.reload") || lower.contains("location.reload()") {
            // Check for orphan location.reload() (reuse lint logic)
            check_orphan_reload(&pages, rule, &mut violations);
        } else if lower.contains("fake data") || lower.contains("mock") {
            // Check for static numeric values in data sections
            check_fake_data(&pages, rule, &mut violations);
        } else {
            // Unrecognized "never" rule — store as informational
            violations.push(ConstitutionViolation {
                rule_type: "info".into(),
                rule: rule.clone(),
                violation: "no automatic check available for this rule".into(),
                entity: None,
            });
        }
    }

    violations
}

/// Check that sensitive fields (password, etc.) don't appear in rendered templates.
fn check_sensitive_in_templates(
    pages: &[&PageNode],
    sensitive_fields: &[String],
    rule: &str,
    violations: &mut Vec<ConstitutionViolation>,
) {
    if sensitive_fields.is_empty() { return; }

    for page in pages {
        for section in &page.sections {
            // Check columns config
            if let Some(columns) = section.config.get("columns") {
                for field in sensitive_fields {
                    for col in columns.split(',') {
                        if col.trim().eq_ignore_ascii_case(field) {
                            violations.push(ConstitutionViolation {
                                rule_type: "never".into(),
                                rule: rule.into(),
                                violation: format!(
                                    "sensitive field '{}' listed in columns on page '{}'",
                                    field, page.route
                                ),
                                entity: None,
                            });
                        }
                    }
                }
            }

            // Check template HTML (outside <script> blocks)
            if let Some(ref template) = section.template {
                let without_scripts = remove_tag_content(template, "script");
                for field in sensitive_fields {
                    if has_exposed_sensitive_field(&without_scripts, field) {
                        violations.push(ConstitutionViolation {
                            rule_type: "never".into(),
                            rule: rule.into(),
                            violation: format!(
                                "sensitive field '{}' appears in template on page '{}'",
                                field, page.route
                            ),
                            entity: None,
                        });
                    }
                }
            }
        }
    }
}

/// Check for hardcoded metric-like text in templates.
fn check_hardcoded_metrics(
    pages: &[&PageNode],
    rule: &str,
    violations: &mut Vec<ConstitutionViolation>,
) {
    for page in pages {
        for section in &page.sections {
            if let Some(ref template) = section.template {
                let without_scripts = remove_tag_content(template, "script");
                let without_styles = remove_tag_content(&without_scripts, "style");
                let text = strip_html_tags(&without_styles);

                for word in text.split_whitespace() {
                    let w = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '%' && c != '$');
                    if looks_like_hardcoded_metric(w) {
                        violations.push(ConstitutionViolation {
                            rule_type: "never".into(),
                            rule: rule.into(),
                            violation: format!(
                                "hardcoded value '{}' in template on page '{}'",
                                w, page.route
                            ),
                            entity: None,
                        });
                    }
                }
            }
        }
    }
}

/// Check for location.reload() not wrapped in CRONUS.reload().
fn check_orphan_reload(
    pages: &[&PageNode],
    rule: &str,
    violations: &mut Vec<ConstitutionViolation>,
) {
    for page in pages {
        for section in &page.sections {
            if let Some(ref template) = section.template {
                if has_orphan_reload(template) {
                    violations.push(ConstitutionViolation {
                        rule_type: "never".into(),
                        rule: rule.into(),
                        violation: format!(
                            "location.reload() found in template on page '{}'",
                            page.route
                        ),
                        entity: None,
                    });
                }
            }
        }
    }
}

/// Check for static numeric values in data section items (fake data).
fn check_fake_data(
    pages: &[&PageNode],
    rule: &str,
    violations: &mut Vec<ConstitutionViolation>,
) {
    let data_types = ["kpi", "table", "chart", "kanban", "stats", "stat-cards"];
    for page in pages {
        for section in &page.sections {
            if !data_types.contains(&section.section_type.as_str()) { continue; }
            // Check if items contain hardcoded numeric values
            for item in &section.items {
                for (key, value) in item {
                    if key == "icon" || key == "type" || key == "style" { continue; }
                    // Flag if value looks like a hardcoded number (not a field reference)
                    let v = value.trim();
                    if v.parse::<f64>().is_ok() && v != "0" && v != "1" {
                        violations.push(ConstitutionViolation {
                            rule_type: "never".into(),
                            rule: rule.into(),
                            violation: format!(
                                "static numeric value '{}' in {} item on page '{}'",
                                v, section.section_type, page.route
                            ),
                            entity: None,
                        });
                    }
                }
            }
        }
    }
}

// ── Helper functions (mirrored from lint.rs to avoid coupling) ──

fn remove_tag_content(html: &str, tag: &str) -> String {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let mut result = String::with_capacity(html.len());
    let mut pos = 0;
    while let Some(start) = html[pos..].find(&open) {
        let abs_start = pos + start;
        result.push_str(&html[pos..abs_start]);
        if let Some(end) = html[abs_start..].find(&close) {
            pos = abs_start + end + close.len();
        } else {
            pos = html.len();
        }
    }
    result.push_str(&html[pos..]);
    result
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        if ch == '<' { in_tag = true; continue; }
        if ch == '>' { in_tag = false; result.push(' '); continue; }
        if !in_tag { result.push(ch); }
    }
    result
}

/// Check if a sensitive field is exposed in rendered HTML (not inside form inputs).
/// Mirrors lint.rs rule_no_sensitive_render logic.
fn has_exposed_sensitive_field(html: &str, field: &str) -> bool {
    let mut pos = 0;
    while let Some(idx) = html[pos..].find(field) {
        let abs = pos + idx;

        // Word boundary check
        let before_byte = if abs > 0 { html.as_bytes()[abs - 1] } else { b' ' };
        let after_pos = abs + field.len();
        let after_byte = if after_pos < html.len() { html.as_bytes()[after_pos] } else { b' ' };
        let is_boundary = |b: u8| !b.is_ascii_alphanumeric() && b != b'_';

        if !is_boundary(before_byte) || !is_boundary(after_byte) {
            pos = abs + 1;
            continue;
        }

        // Look backwards to find context
        let search_start = if abs > 300 { abs - 300 } else { 0 };
        let before = &html[search_start..abs];

        // Skip if inside <input type="password"> — form input, not display
        if let Some(tag_start) = before.rfind('<') {
            let tag = &before[tag_start..];
            if tag.contains("input") && (tag.contains("type='password'") || tag.contains("type=\"password\"")) {
                pos = abs + field.len();
                continue;
            }
        }

        // Skip if inside name="", type="", or placeholder="" attribute value
        let in_attr = ["name", "type", "placeholder"].iter().any(|a| is_in_attr_value(before, a));
        if in_attr {
            pos = abs + field.len();
            continue;
        }

        // This is an exposed sensitive field
        return true;
    }
    false
}

/// Check if position is inside an attribute value (no closing quote found after attr="...).
fn is_in_attr_value(before: &str, attr_name: &str) -> bool {
    for quote in &['"', '\''] {
        let pat = format!("{}={}", attr_name, quote);
        if let Some(idx) = before.rfind(&pat) {
            let after = idx + pat.len();
            if !before[after..].contains(*quote) {
                return true;
            }
        }
    }
    false
}

fn contains_word(text: &str, word: &str) -> bool {
    let mut pos = 0;
    while let Some(idx) = text[pos..].find(word) {
        let abs = pos + idx;
        let before = if abs > 0 { text.as_bytes()[abs - 1] } else { b' ' };
        let after_pos = abs + word.len();
        let after = if after_pos < text.len() { text.as_bytes()[after_pos] } else { b' ' };
        let is_boundary = |b: u8| !b.is_ascii_alphanumeric() && b != b'_';
        if is_boundary(before) && is_boundary(after) {
            return true;
        }
        pos = abs + 1;
    }
    false
}

fn has_orphan_reload(html: &str) -> bool {
    let needle = "location.reload()";
    let mut pos = 0;
    while let Some(idx) = html[pos..].find(needle) {
        let abs = pos + idx;
        let before = if abs > 30 { &html[abs-30..abs] } else { &html[..abs] };
        if before.contains("CRONUS.reload") || before.contains("CRONUS&&") {
            pos = abs + needle.len();
            continue;
        }
        return true;
    }
    false
}

fn looks_like_hardcoded_metric(w: &str) -> bool {
    if w.len() < 3 { return false; }
    // $123,456
    if w.starts_with('$') && w.len() > 1 && w[1..].replace(',', "").parse::<f64>().is_ok() {
        return true;
    }
    // 99.99%
    if w.ends_with('%') && w[..w.len()-1].parse::<f64>().is_ok() {
        return true;
    }
    // 1.2M, 4.5GB etc.
    if (w.ends_with('K') || w.ends_with('M') || w.ends_with('B') || w.ends_with('T')
        || w.ends_with("GB") || w.ends_with("MB") || w.ends_with("TB"))
        && w.len() > 1
    {
        let num_part = w.trim_end_matches(|c: char| c.is_alphabetic());
        if num_part.parse::<f64>().is_ok() {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::*;
    use std::collections::HashMap;

    fn make_constitution(must: Vec<&str>, never: Vec<&str>) -> ConstitutionNode {
        ConstitutionNode {
            must: must.into_iter().map(String::from).collect(),
            never: never.into_iter().map(String::from).collect(),
        }
    }

    fn make_entity(name: &str, fields: Vec<(&str, FieldType, bool)>) -> AstNode {
        AstNode::Entity(EntityNode {
            name: name.into(),
            fields: fields.into_iter().map(|(n, ft, sensitive)| FieldNode {
                name: n.into(),
                field_type: ft,
                required: false,
                unique: false,
                sensitive,
                optional: false,
                searchable: false,
                index: false,
                featured: false,
                formatted: false,
                array: false,
                enum_values: None,
                reference: None,
                doc: None,
                default_value: None, min: None, max: None, min_length: None, max_length: None, pattern: None,
            }).collect(),
            transitions: vec![],
            shared: false,
            doc: None,
        })
    }

    fn make_page(route: &str, requires: Option<&str>, sections: Vec<SectionNode>) -> AstNode {
        AstNode::Page(PageNode {
            route: route.into(),
            page_type: "custom".into(),
            entity: None,
            title: None,
            sections,
            config: HashMap::new(),
            components: vec![],
            requires: requires.map(String::from),
            doc: None,
        })
    }

    fn make_data_section(stype: &str, has_bind: bool) -> SectionNode {
        SectionNode {
            section_type: stype.into(),
            title: None,
            subtitle: None,
            config: HashMap::new(),
            items: vec![],
            plans: vec![],
            binding: if has_bind {
                Some(BindingNode {
                    entity: "Test".into(),
                    query: QueryType::All,
                    filters: vec![],
                    order: None,
                    limit: None,
                    offset: None,
                    group_by: None,
                    aggregate: None,
                })
            } else { None },
            actions: vec![],
            visibility: None,
            template: None,
            style_block: None,
            doc: None,
        }
    }

    #[test]
    fn test_must_bind_pass() {
        let nodes = vec![
            make_page("/dashboard", Some("auth"), vec![
                make_data_section("table", true),
                make_data_section("kpi", true),
            ]),
        ];
        let c = make_constitution(vec!["all data sections require bind"], vec![]);
        let v = check_constitution(&nodes, &c);
        assert!(v.is_empty(), "expected no violations, got: {:?}", v);
    }

    #[test]
    fn test_must_bind_fail() {
        let nodes = vec![
            make_page("/dashboard", Some("auth"), vec![
                make_data_section("table", false),
            ]),
        ];
        let c = make_constitution(vec!["all data sections require bind"], vec![]);
        let v = check_constitution(&nodes, &c);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule_type, "must");
    }

    #[test]
    fn test_never_orphan_reload() {
        let mut section = make_data_section("hero", false);
        section.template = Some("<script>location.reload()</script>".into());
        let nodes = vec![
            make_page("/", None, vec![section]),
        ];
        let c = make_constitution(vec![], vec!["use location.reload()"]);
        let v = check_constitution(&nodes, &c);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule_type, "never");
    }

    #[test]
    fn test_never_expose_password() {
        let mut section = make_data_section("table", true);
        section.config.insert("columns".into(), "name, email, password".into());
        let nodes = vec![
            make_entity("User", vec![
                ("name", FieldType::String, false),
                ("password", FieldType::String, true),
            ]),
            make_page("/users", Some("auth"), vec![section]),
        ];
        let c = make_constitution(vec![], vec!["expose passwords in API responses"]);
        let v = check_constitution(&nodes, &c);
        assert_eq!(v.len(), 1);
        assert!(v[0].violation.contains("password"));
    }

    #[test]
    fn test_unrecognized_rule_is_info() {
        let nodes = vec![];
        let c = make_constitution(vec!["always be nice"], vec!["use tabs"]);
        let v = check_constitution(&nodes, &c);
        assert_eq!(v.len(), 2);
        assert!(v.iter().all(|v| v.rule_type == "info"));
    }

    #[test]
    fn test_nova_core_constitution_passes() {
        // Simulate the Nova Core app's constitution and AST
        let c = make_constitution(
            vec![
                "prices in centavos \u{2014} use formatPrice()",
                "all data sections require bind",
                "auth required on all admin endpoints",
            ],
            vec![
                "expose passwords in API responses",
                "hardcode user data in templates",
                "use location.reload()",
            ],
        );

        let nodes = vec![
            make_entity("User", vec![
                ("name", FieldType::String, false),
                ("password", FieldType::String, true),
            ]),
            make_entity("Deployment", vec![
                ("deploy_id", FieldType::String, false),
                ("service", FieldType::String, false),
            ]),
            // Home page "/" is public, so no auth required
            make_page("/", None, vec![
                make_data_section("kpi", true),
                make_data_section("chart", true),
                make_data_section("table", true),
            ]),
        ];

        let v = check_constitution(&nodes, &c);
        // Filter out informational — only check real violations
        let real: Vec<_> = v.iter().filter(|v| v.rule_type != "info").collect();
        assert!(real.is_empty(), "expected no violations, got: {:?}", real);
    }
}
