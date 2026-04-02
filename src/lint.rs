#![allow(dead_code)]
//! CRONUS Lint Engine — Zero Hardcode Enforcement
//!
//! 7 rules that prevent hardcoded data, dead UI, and broken SPA contracts.
//! Runs at parse time (<5ms) using regex on template strings.

use crate::parser::{AstNode, ApiNode, EntityNode, HttpMethod, PageNode, SectionNode};

#[derive(Debug, Clone)]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct LintResult {
    pub rule: &'static str,
    pub severity: Severity,
    pub message: String,
    pub fix: String,
    pub section: String,
    pub page: String,
}

impl std::fmt::Display for LintResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let icon = match self.severity {
            Severity::Warning => "\x1b[33m⚠\x1b[0m",
            Severity::Error => "\x1b[31m✗\x1b[0m",
        };
        let sev = match self.severity {
            Severity::Warning => "warning",
            Severity::Error => "error",
        };
        write!(f, "  {} \x1b[1m{}\x1b[0m ({}): {}\n    → {}",
            icon, self.rule, self.section, self.message, self.fix)
    }
}

/// Run all lint rules against the parsed AST.
/// In strict mode, warnings are promoted to errors.
pub fn lint_ast(nodes: &[AstNode], strict: bool) -> Vec<LintResult> {
    let mut results = Vec::new();

    // Collect pages, entities, APIs, and all sections (including from Define blocks)
    let mut pages: Vec<&PageNode> = Vec::new();
    let mut all_routes: Vec<String> = Vec::new();
    let mut entities: Vec<&EntityNode> = Vec::new();
    let mut apis: Vec<&ApiNode> = Vec::new();
    let mut define_sections: Vec<(&str, &SectionNode)> = Vec::new();

    for node in nodes {
        match node {
            AstNode::Page(p) => {
                pages.push(p);
                all_routes.push(p.route.clone());
            }
            AstNode::Entity(e) => {
                entities.push(e);
            }
            AstNode::Api(a) => {
                apis.push(a);
            }
            AstNode::Define(d) => {
                for s in &d.sections {
                    define_sections.push((&d.name, s));
                }
            }
            _ => {}
        }
    }

    // Collect sensitive field names from all entities
    let sensitive_fields: Vec<String> = entities.iter()
        .flat_map(|e| e.fields.iter())
        .filter(|f| f.sensitive)
        .map(|f| f.name.clone())
        .collect();

    // Kernel routes that always exist
    let kernel_routes = vec![
        "/docs".to_string(),
        "/docs/design".to_string(),
        "/graphql".to_string(),
    ];

    // Run rules on each page's sections
    for page in &pages {
        for section in &page.sections {
            let sec_name = format!("{} ({})", section.section_type, page.route);
            results.extend(rule_no_dead_text(section, &sec_name, &page.route));
            results.extend(rule_no_dead_links(section, &sec_name, &page.route, &all_routes, &kernel_routes));
            results.extend(rule_no_dead_ui(section, &sec_name, &page.route));
            results.extend(rule_no_fake_state(section, &sec_name, &page.route));
            results.extend(rule_no_orphan_reload(section, &sec_name, &page.route));
            results.extend(rule_no_hardcode_user(section, &sec_name, &page.route));
            results.extend(rule_no_sensitive_render(section, &sec_name, &page.route, &sensitive_fields));
        }
        // Rule 7: bind-or-empty (page-level)
        results.extend(rule_bind_or_empty(page));
    }

    // Run rules on define sections too
    for (def_name, section) in &define_sections {
        let sec_name = format!("{} (define:{})", section.section_type, def_name);
        results.extend(rule_no_dead_text(section, &sec_name, "define"));
        results.extend(rule_no_dead_links(section, &sec_name, "define", &all_routes, &kernel_routes));
        results.extend(rule_no_dead_ui(section, &sec_name, "define"));
        results.extend(rule_no_fake_state(section, &sec_name, "define"));
        results.extend(rule_no_orphan_reload(section, &sec_name, "define"));
        results.extend(rule_no_hardcode_user(section, &sec_name, "define"));
        results.extend(rule_no_sensitive_render(section, &sec_name, "define", &sensitive_fields));
    }

    // Rule C012: form-submit-handler — Form sections must have submit handler
    for page in &pages {
        for section in &page.sections {
            let sec_name = format!("{} ({})", section.section_type, page.route);
            results.extend(rule_form_submit_handler(section, &sec_name, &page.route));
        }
    }
    for (def_name, section) in &define_sections {
        let sec_name = format!("{} (define:{})", section.section_type, def_name);
        results.extend(rule_form_submit_handler(section, &sec_name, "define"));
    }

    // Rule C030: shared-entity-auth — Shared entity mutations require auth
    results.extend(rule_shared_entity_auth(&entities, &apis));

    // Rule C031: no-sensitive-select — Sensitive fields excluded from table columns and bind refs
    for page in &pages {
        for section in &page.sections {
            let sec_name = format!("{} ({})", section.section_type, page.route);
            results.extend(rule_no_sensitive_select(section, &sec_name, &page.route, &sensitive_fields));
        }
    }
    for (def_name, section) in &define_sections {
        let sec_name = format!("{} (define:{})", section.section_type, def_name);
        results.extend(rule_no_sensitive_select(section, &sec_name, "define", &sensitive_fields));
    }

    // Promote warnings to errors in strict mode
    if strict {
        for r in &mut results {
            if matches!(r.severity, Severity::Warning) {
                r.severity = Severity::Error;
            }
        }
    }

    results
}

/// Validate a raw template HTML string (for AI code generation).
/// Returns violations without needing a full AST.
pub fn lint_template(html: &str) -> Vec<LintResult> {
    let mut results = Vec::new();
    let section_name = "template".to_string();
    let page = "generated".to_string();

    // Check dead metrics
    for finding in find_dead_metrics_in_html(html) {
        results.push(LintResult {
            rule: "no-dead-text",
            severity: Severity::Warning,
            message: format!("\"{}\" looks like hardcoded metric", finding),
            fix: "Populate via JS fetch or bind to an entity field".into(),
            section: section_name.clone(),
            page: page.clone(),
        });
    }

    // Check orphan reload
    if has_orphan_reload(html) {
        results.push(LintResult {
            rule: "no-orphan-reload",
            severity: Severity::Error,
            message: "location.reload() found — breaks SPA contract".into(),
            fix: "Use CRONUS.reload() for soft refresh".into(),
            section: section_name.clone(),
            page: page.clone(),
        });
    }

    // Check dead links
    for href in find_dead_hrefs(html) {
        results.push(LintResult {
            rule: "no-dead-links",
            severity: Severity::Error,
            message: format!("href=\"{}\" is a dead link", href),
            fix: "Point to a real page route or remove the element".into(),
            section: section_name.clone(),
            page: page.clone(),
        });
    }

    results
}

// ══════════════════════════════════════════════════
// RULE 1: no-dead-text — Detect Hardcoded Metrics
// ══════════════════════════════════════════════════

fn rule_no_dead_text(section: &SectionNode, sec_name: &str, page: &str) -> Vec<LintResult> {
    let template = match &section.template {
        Some(t) => t,
        None => return vec![],
    };

    let mut results = Vec::new();
    for finding in find_dead_metrics_in_html(template) {
        results.push(LintResult {
            rule: "no-dead-text",
            severity: Severity::Warning,
            message: format!("\"{}\" looks like hardcoded metric", finding),
            fix: "Use <span id=\"...\"> populated via JS fetch, or bind to entity".into(),
            section: sec_name.into(),
            page: page.into(),
        });
    }
    results
}

/// Extract metric-like strings from HTML, skipping safe contexts.
fn find_dead_metrics_in_html(html: &str) -> Vec<String> {
    let mut findings = Vec::new();

    // Remove <script>...</script> blocks (JS is OK)
    let without_scripts = remove_tag_content(html, "script");
    // Remove <style>...</style> blocks (CSS is OK)
    let clean = remove_tag_content(&without_scripts, "style");

    // Strip HTML tags to get text nodes, but preserve attribute context
    let text_only = strip_tags_preserve_attrs(&clean);

    // Metric patterns
    let patterns: &[(&str, &str)] = &[
        (r"\b\d+(\.\d+)?%", "percentage"),           // 99.99%, 12%
        (r"\b\d+(\.\d+)?(ms|s|m|h)\b", "duration"),  // 14ms, 2.5s
        (r"\$[\d,.]+", "currency"),                    // $142,804
        (r"\b\d+(\.\d+)?(K|M|B|T|GB|MB|TB)\b", "magnitude"), // 1.2M, 4.5GB
        (r"\bv\d+\.\d+(\.\d+)?(-\w+)?", "version"),  // v2.4.0-stable
    ];

    for (pattern, _kind) in patterns {
        // Simple regex-like matching (no regex crate — manual scan)
        for word in text_only.split_whitespace() {
            let w = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '%' && c != '$' && c != '-');
            if matches_metric_pattern(w, pattern) {
                // Skip if it's inside a placeholder attribute
                if is_in_placeholder(html, w) { continue; }
                // Skip if the containing element has an id="" (will be populated by JS)
                if is_in_id_element(html, w) { continue; }
                // Skip common false positives
                if w == "0" || w == "0%" || w.len() < 2 { continue; }
                findings.push(w.to_string());
            }
        }
    }

    findings.dedup();
    findings
}

fn matches_metric_pattern(word: &str, _pattern: &str) -> bool {
    let w = word;
    // Check each pattern type manually (fast, no regex dependency)
    if w.ends_with('%') && w[..w.len()-1].parse::<f64>().is_ok() && w.len() > 2 {
        return true;
    }
    if (w.ends_with("ms") || w.ends_with("s") || w.ends_with("m") || w.ends_with("h"))
        && w.len() > 2
        && w[..w.len()-2].trim_end_matches(|c: char| c == 's' || c == 'm' || c == 'h')
            .parse::<f64>().is_ok()
    {
        // Avoid matching CSS values and common words
        let num_part = w.trim_end_matches(|c: char| c.is_alphabetic());
        if num_part.parse::<f64>().is_ok() && num_part.len() >= 1 {
            return true;
        }
    }
    if w.starts_with('$') && w.len() > 1 && w[1..].replace(',', "").parse::<f64>().is_ok() {
        return true;
    }
    if (w.ends_with('K') || w.ends_with('M') || w.ends_with('B') || w.ends_with('T')
        || w.ends_with("GB") || w.ends_with("MB") || w.ends_with("TB"))
        && w.len() > 1
    {
        let num_part = w.trim_end_matches(|c: char| c.is_alphabetic());
        if num_part.parse::<f64>().is_ok() {
            return true;
        }
    }
    if w.starts_with('v') && w.len() > 3 && w.chars().nth(1).map(|c| c.is_ascii_digit()).unwrap_or(false) {
        if w.contains('.') { return true; }
    }
    false
}

// ══════════════════════════════════════════════════
// RULE 2: no-dead-links — Every Link Must Navigate
// ══════════════════════════════════════════════════

fn rule_no_dead_links(section: &SectionNode, sec_name: &str, page: &str, routes: &[String], kernel_routes: &[String]) -> Vec<LintResult> {
    let template = match &section.template {
        Some(t) => t,
        None => return vec![],
    };

    let mut results = Vec::new();
    for href in find_dead_hrefs(template) {
        results.push(LintResult {
            rule: "no-dead-links",
            severity: Severity::Error, // Always error
            message: format!("href=\"{}\" is a dead link", href),
            fix: "Point to a real page route or remove the element".into(),
            section: sec_name.into(),
            page: page.into(),
        });
    }

    // Check internal links point to existing routes
    for href in extract_hrefs(template) {
        if href.starts_with('/') && href != "#" {
            // /api/* paths are always valid (kernel API routes)
            if href.starts_with("/api/") || href == "/api" {
                continue;
            }
            if !routes.contains(&href) && !kernel_routes.contains(&href) {
                let suggestion = find_closest_route(&href, routes, kernel_routes);
                let fix = if let Some(ref s) = suggestion {
                    format!("Did you mean \"{}\"? Or create a page with route \"{}\"", s, href)
                } else {
                    format!("Create a page with route \"{}\" or remove the link", href)
                };
                results.push(LintResult {
                    rule: "no-dead-links",
                    severity: Severity::Warning,
                    message: format!("href=\"{}\" points to unknown route", href),
                    fix,
                    section: sec_name.into(),
                    page: page.into(),
                });
            }
        }
    }

    results
}

fn find_dead_hrefs(html: &str) -> Vec<String> {
    let mut dead = Vec::new();
    // Find href="#" (excluding data-scroll and anchor-to-section)
    let mut pos = 0;
    while let Some(idx) = html[pos..].find("href=") {
        let start = pos + idx + 5;
        if start >= html.len() { break; }
        let quote = html.as_bytes().get(start).copied().unwrap_or(b'"');
        if quote == b'"' || quote == b'\'' {
            let end = html[start+1..].find(quote as char).map(|i| start + 1 + i).unwrap_or(html.len());
            let href = &html[start+1..end];
            if href == "#" {
                dead.push("#".into());
            }
        }
        pos = start + 1;
    }
    dead.dedup();
    dead
}

fn extract_hrefs(html: &str) -> Vec<String> {
    let mut hrefs = Vec::new();
    let mut pos = 0;
    while let Some(idx) = html[pos..].find("href=") {
        let start = pos + idx + 5;
        if start >= html.len() { break; }
        let quote = html.as_bytes().get(start).copied().unwrap_or(b'"');
        if quote == b'"' || quote == b'\'' {
            let end = html[start+1..].find(quote as char).map(|i| start + 1 + i).unwrap_or(html.len());
            let href = &html[start+1..end];
            if !href.is_empty() && href != "#" {
                hrefs.push(href.to_string());
            }
        }
        pos = start + 1;
    }
    hrefs
}

// ══════════════════════════════════════════════════
// RULE 3: no-dead-ui — Interactive Elements Need Behavior
// ══════════════════════════════════════════════════

fn rule_no_dead_ui(section: &SectionNode, sec_name: &str, page: &str) -> Vec<LintResult> {
    let template = match &section.template {
        Some(t) => t,
        None => return vec![],
    };

    let mut results = Vec::new();

    // Find <button> without onclick, type="submit", or data-* action attributes
    let mut pos = 0;
    while let Some(idx) = template[pos..].find("<button") {
        let tag_start = pos + idx;
        let tag_end = template[tag_start..].find('>').map(|i| tag_start + i).unwrap_or(template.len());
        let tag = &template[tag_start..tag_end+1];

        let has_action = tag.contains("onclick") || tag.contains("type=\"submit\"")
            || tag.contains("type='submit'") || tag.contains("data-cronus")
            || tag.contains("data-modal") || tag.contains("data-delete")
            || tag.contains("data-scroll");

        if !has_action {
            // Extract button text
            let text_end = template[tag_end+1..].find("</button").unwrap_or(20);
            let btn_text = &template[tag_end+1..tag_end+1+text_end.min(40)];
            let btn_text = strip_html_tags(btn_text).trim().to_string();
            if !btn_text.is_empty() {
                results.push(LintResult {
                    rule: "no-dead-ui",
                    severity: Severity::Warning,
                    message: format!("<button> \"{}\" has no action handler", btn_text),
                    fix: "Add onclick, data-cronus-action, or type=\"submit\" (inside form)".into(),
                    section: sec_name.into(),
                    page: page.into(),
                });
            }
        }
        pos = tag_end + 1;
    }

    results
}

// ══════════════════════════════════════════════════
// RULE 4: no-fake-state — Static Text Implying Live State
// ══════════════════════════════════════════════════

fn rule_no_fake_state(section: &SectionNode, sec_name: &str, page: &str) -> Vec<LintResult> {
    let template = match &section.template {
        Some(t) => t,
        None => return vec![],
    };

    let mut results = Vec::new();
    let without_scripts = remove_tag_content(template, "script");

    let state_keywords = ["Loading", "Connecting", "Syncing"];

    for keyword in &state_keywords {
        if without_scripts.contains(keyword) {
            // Check if it's inside an id="" element (will be replaced by JS) → OK
            if is_in_id_element(template, keyword) { continue; }

            results.push(LintResult {
                rule: "no-fake-state",
                severity: Severity::Warning,
                message: format!("\"{}\" in static HTML implies live state", keyword),
                fix: "Place inside an element with id=\"...\" that JS will populate, or use a skeleton loader".into(),
                section: sec_name.into(),
                page: page.into(),
            });
        }
    }

    results
}

// ══════════════════════════════════════════════════
// RULE 5: no-orphan-reload — Ban location.reload()
// ══════════════════════════════════════════════════

fn rule_no_orphan_reload(section: &SectionNode, sec_name: &str, page: &str) -> Vec<LintResult> {
    let template = match &section.template {
        Some(t) => t,
        None => return vec![],
    };

    let mut results = Vec::new();
    if has_orphan_reload(template) {
        results.push(LintResult {
            rule: "no-orphan-reload",
            severity: Severity::Error, // Always error
            message: "location.reload() found in template — breaks SPA contract".into(),
            fix: "Use if(window.CRONUS&&window.CRONUS.reload)window.CRONUS.reload() instead".into(),
            section: sec_name.into(),
            page: page.into(),
        });
    }
    results
}

fn has_orphan_reload(html: &str) -> bool {
    // Find location.reload() NOT preceded by CRONUS.reload
    let needle = "location.reload()";
    let mut pos = 0;
    while let Some(idx) = html[pos..].find(needle) {
        let abs = pos + idx;
        // Check if this is inside a CRONUS.reload() fallback pattern → OK
        let before = if abs > 30 { &html[abs-30..abs] } else { &html[..abs] };
        if before.contains("CRONUS.reload") || before.contains("CRONUS&&") {
            pos = abs + needle.len();
            continue;
        }
        return true;
    }
    false
}

// ══════════════════════════════════════════════════
// RULE 6: no-hardcode-user — User Data Must Come From Auth
// ══════════════════════════════════════════════════

fn rule_no_hardcode_user(section: &SectionNode, sec_name: &str, page: &str) -> Vec<LintResult> {
    let template = match &section.template {
        Some(t) => t,
        None => return vec![],
    };

    let mut results = Vec::new();
    let without_scripts = remove_tag_content(template, "script");
    let text = strip_html_tags(&without_scripts);

    // Hardcoded role/admin names in visible text
    let user_patterns = ["System Admin", "Administrator", "Super Admin", "Root User"];
    for pattern in &user_patterns {
        if text.contains(pattern) {
            results.push(LintResult {
                rule: "no-hardcode-user",
                severity: Severity::Warning,
                message: format!("\"{}\" is hardcoded user/role text", pattern),
                fix: "Read from localStorage.getItem('user') or fetch /api/auth/me".into(),
                section: sec_name.into(),
                page: page.into(),
            });
        }
    }

    results
}

// ══════════════════════════════════════════════════
// RULE 7: bind-or-empty — Data Sections Must Bind
// ══════════════════════════════════════════════════

fn rule_bind_or_empty(page: &PageNode) -> Vec<LintResult> {
    let data_types = ["kpi", "table", "chart", "kanban", "timeline", "stats", "stat-cards"];
    let mut results = Vec::new();

    for section in &page.sections {
        if !data_types.contains(&section.section_type.as_str()) { continue; }
        let has_bind = section.binding.is_some();
        let has_items = !section.items.is_empty();
        let has_template = section.template.is_some();

        if !has_bind && !has_items && !has_template {
            results.push(LintResult {
                rule: "bind-or-empty",
                severity: Severity::Warning,
                message: format!("section {} has no data source (no bind, no items, no template)", section.section_type),
                fix: format!("Add: bind EntityName {{ query all }} or define static items"),
                section: format!("{} ({})", section.section_type, page.route),
                page: page.route.clone(),
            });
        }
    }

    results
}

// ══════════════════════════════════════════════════
// RULE C002: no-sensitive-render — Sensitive Fields Never in Rendered Output
// ══════════════════════════════════════════════════

fn rule_no_sensitive_render(section: &SectionNode, sec_name: &str, page: &str, sensitive_fields: &[String]) -> Vec<LintResult> {
    if sensitive_fields.is_empty() {
        return vec![];
    }

    let template = match &section.template {
        Some(t) => t,
        None => return vec![],
    };

    let mut results = Vec::new();

    // Also check section config (e.g., columns: "name, email, password")
    if let Some(columns) = section.config.get("columns") {
        for field in sensitive_fields {
            // Check if the sensitive field name appears as a column name
            for col in columns.split(',') {
                let col = col.trim();
                if col == field {
                    results.push(LintResult {
                        rule: "no-sensitive-render",
                        severity: Severity::Error,
                        message: format!("sensitive field \"{}\" listed in columns config", field),
                        fix: format!("Remove \"{}\" from columns — sensitive fields must never be displayed", field),
                        section: sec_name.into(),
                        page: page.into(),
                    });
                }
            }
        }
    }

    // Strip <script> blocks — JS logic referencing sensitive fields is OK
    let without_scripts = remove_tag_content(template, "script");

    for field in sensitive_fields {
        // Find all occurrences of the field name in the HTML (outside scripts)
        let mut pos = 0;
        while let Some(idx) = without_scripts[pos..].find(field.as_str()) {
            let abs = pos + idx;

            // Check word boundaries: the match must be the whole word/identifier
            let before_char = if abs > 0 { without_scripts.as_bytes()[abs - 1] } else { b' ' };
            let after_pos = abs + field.len();
            let after_char = if after_pos < without_scripts.len() { without_scripts.as_bytes()[after_pos] } else { b' ' };

            let is_word_boundary = |b: u8| -> bool {
                !b.is_ascii_alphanumeric() && b != b'_'
            };

            if !is_word_boundary(before_char) || !is_word_boundary(after_char) {
                pos = abs + 1;
                continue;
            }

            // Find the context: look backwards for the nearest tag
            let search_start = if abs > 300 { abs - 300 } else { 0 };
            let before = &without_scripts[search_start..abs];

            // Skip if inside type="password" input — this is an input, not display
            if is_in_password_input(before) {
                pos = abs + field.len();
                continue;
            }

            // Skip if inside a placeholder attribute value
            if is_in_attribute_value(before, "placeholder") {
                pos = abs + field.len();
                continue;
            }

            // Skip if it's the value of a name="" or type="" attribute (form input context)
            if is_in_attribute_value(before, "name") || is_in_attribute_value(before, "type") {
                pos = abs + field.len();
                continue;
            }

            // This is a sensitive field rendered in visible HTML — ERROR
            results.push(LintResult {
                rule: "no-sensitive-render",
                severity: Severity::Error,
                message: format!("sensitive field \"{}\" appears in rendered output", field),
                fix: format!("Remove \"{}\" from the template — sensitive fields must never be displayed to users", field),
                section: sec_name.into(),
                page: page.into(),
            });

            // Only report once per field per section
            break;
        }
    }

    results
}

/// Check if position is inside an <input type="password"> tag
fn is_in_password_input(before: &str) -> bool {
    // Find the last opening tag before the match
    if let Some(tag_start) = before.rfind('<') {
        let tag = &before[tag_start..];
        // Check it's an input with type="password"
        if tag.contains("input") && (tag.contains("type='password'") || tag.contains("type=\"password\"")) {
            return true;
        }
    }
    false
}

/// Check if position is inside a specific attribute's value
fn is_in_attribute_value(before: &str, attr_name: &str) -> bool {
    // Find the last occurrence of `attr="...` without a closing quote
    let patterns = [
        format!("{}=\"", attr_name),
        format!("{}='", attr_name),
    ];
    for pat in &patterns {
        if let Some(idx) = before.rfind(pat.as_str()) {
            let after_attr = idx + pat.len();
            let quote = pat.as_bytes()[pat.len() - 1];
            // Check if there's no closing quote between the attr start and our position
            if !before[after_attr..].contains(quote as char) {
                return true;
            }
        }
    }
    false
}

// ══════════════════════════════════════════════════
// RULE C012: form-submit-handler — Form Sections Must Have Submit
// ══════════════════════════════════════════════════

fn rule_form_submit_handler(section: &SectionNode, sec_name: &str, page: &str) -> Vec<LintResult> {
    if section.section_type != "form" {
        return vec![];
    }

    // Check 1: Does the actions list contain a "submit" event?
    let has_submit_action = section.actions.iter().any(|a| a.event == "submit");

    // Check 2: Does the template contain submit-related patterns?
    let has_submit_in_template = if let Some(ref template) = section.template {
        template.contains("type=\"submit\"")
            || template.contains("type='submit'")
            || template.contains("onsubmit")
            || template.contains("data-cronus-form")
    } else {
        false
    };

    if !has_submit_action && !has_submit_in_template {
        return vec![LintResult {
            rule: "form-submit-handler",
            severity: Severity::Warning,
            message: "form section has no submit handler".into(),
            fix: "Add an action { on submit ... } block, or include type=\"submit\" / data-cronus-form in template".into(),
            section: sec_name.into(),
            page: page.into(),
        }];
    }

    vec![]
}

// ══════════════════════════════════════════════════
// RULE C030: shared-entity-auth — Shared Entity Mutations Require Auth
// ══════════════════════════════════════════════════

fn rule_shared_entity_auth(entities: &[&EntityNode], apis: &[&ApiNode]) -> Vec<LintResult> {
    let mut results = Vec::new();

    // Collect names of shared entities
    let shared_names: Vec<&str> = entities.iter()
        .filter(|e| e.shared)
        .map(|e| e.name.as_str())
        .collect();

    if shared_names.is_empty() {
        return results;
    }

    for api in apis {
        // Check if this API's prefix references a shared entity
        // API prefixes are typically like "/products", "/posts" — match against entity name (lowercase)
        let prefix_lower = api.prefix.to_lowercase();
        let matches_shared = shared_names.iter().any(|name| {
            let name_lower = name.to_lowercase();
            // Match: prefix contains entity name (e.g., "/products" contains "product")
            // Or prefix is the plural/singular of the entity name
            prefix_lower.contains(&name_lower)
                || prefix_lower.contains(&format!("{}s", name_lower))
                || name_lower.contains(&prefix_lower.trim_start_matches('/').replace('/', ""))
        });

        if !matches_shared {
            continue;
        }

        for route in &api.routes {
            // Only check mutation methods (POST, PATCH, PUT, DELETE)
            match route.method {
                HttpMethod::GET => continue,
                _ => {}
            }

            // Check if auth is missing or public
            let auth_lower = route.auth.to_lowercase();
            if auth_lower == "public" || auth_lower.is_empty() || auth_lower == "none" {
                results.push(LintResult {
                    rule: "shared-entity-auth",
                    severity: Severity::Error,
                    message: format!(
                        "{} {} on shared entity API \"{}\" has no auth (auth: \"{}\")",
                        match route.method {
                            HttpMethod::POST => "POST",
                            HttpMethod::PATCH => "PATCH",
                            HttpMethod::PUT => "PUT",
                            HttpMethod::DELETE => "DELETE",
                            HttpMethod::GET => "GET",
                        },
                        route.path,
                        api.prefix,
                        route.auth
                    ),
                    fix: "Shared entities must require authentication for mutations — set auth: \"token\" or auth: \"role(admin)\"".into(),
                    section: format!("api {}", api.prefix),
                    page: "api".into(),
                });
            }
        }
    }

    results
}

// ══════════════════════════════════════════════════
// RULE C031: no-sensitive-select — Sensitive Fields Not in Table Columns or Bind Refs
// ══════════════════════════════════════════════════

fn rule_no_sensitive_select(section: &SectionNode, sec_name: &str, page: &str, sensitive_fields: &[String]) -> Vec<LintResult> {
    if sensitive_fields.is_empty() {
        return vec![];
    }

    let mut results = Vec::new();

    // Check table section columns config (extends C002 coverage)
    if section.section_type == "table" {
        if let Some(columns) = section.config.get("columns") {
            for field in sensitive_fields {
                for col in columns.split(',') {
                    let col = col.trim();
                    if col == field {
                        results.push(LintResult {
                            rule: "no-sensitive-select",
                            severity: Severity::Error,
                            message: format!("sensitive field \"{}\" in table columns — would be included in SELECT", field),
                            fix: format!("Remove \"{}\" from columns config — sensitive fields must be excluded from queries", field),
                            section: sec_name.into(),
                            page: page.into(),
                        });
                    }
                }
            }
        }
    }

    // Check bind sections that explicitly reference sensitive fields
    if let Some(ref binding) = section.binding {
        // Check filters that reference sensitive fields
        for filter in &binding.filters {
            let filter_field = &filter.field;
            if sensitive_fields.contains(filter_field) {
                results.push(LintResult {
                    rule: "no-sensitive-select",
                    severity: Severity::Error,
                    message: format!("bind filter references sensitive field \"{}\"", filter_field),
                    fix: format!("Do not use sensitive field \"{}\" in bind filters — handle in server-side logic", filter_field),
                    section: sec_name.into(),
                    page: page.into(),
                });
            }
        }

        // Check order_by referencing sensitive fields
        if let Some(ref order) = binding.order {
            if sensitive_fields.contains(&order.field) {
                results.push(LintResult {
                    rule: "no-sensitive-select",
                    severity: Severity::Error,
                    message: format!("bind order references sensitive field \"{}\"", order.field),
                    fix: format!("Do not order by sensitive field \"{}\"", order.field),
                    section: sec_name.into(),
                    page: page.into(),
                });
            }
        }

        // Check group_by referencing sensitive fields
        if let Some(ref group) = binding.group_by {
            if sensitive_fields.contains(&group.field) {
                results.push(LintResult {
                    rule: "no-sensitive-select",
                    severity: Severity::Error,
                    message: format!("bind group_by references sensitive field \"{}\"", group.field),
                    fix: format!("Do not group by sensitive field \"{}\"", group.field),
                    section: sec_name.into(),
                    page: page.into(),
                });
            }
        }
    }

    results
}

// ══════════════════════════════════════════════════
// RULE C010: Enhanced dead-link route validation helpers
// ══════════════════════════════════════════════════

/// Find the closest matching route using simple string similarity (Levenshtein-like)
fn find_closest_route(href: &str, routes: &[String], kernel_routes: &[String]) -> Option<String> {
    let all: Vec<&String> = routes.iter().chain(kernel_routes.iter()).collect();
    if all.is_empty() {
        return None;
    }

    let mut best: Option<(&String, usize)> = None;
    for route in &all {
        let dist = simple_edit_distance(href, route);
        // Only suggest if reasonably close (distance <= 3, or < half the length)
        let threshold = (href.len() / 2).max(3);
        if dist <= threshold {
            if best.is_none() || dist < best.unwrap().1 {
                best = Some((route, dist));
            }
        }
    }

    best.map(|(r, _)| r.to_string())
}

/// Simple edit distance (Levenshtein) — no dependencies needed
fn simple_edit_distance(a: &str, b: &str) -> usize {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let m = a_bytes.len();
    let n = b_bytes.len();

    let mut prev = (0..=n).collect::<Vec<_>>();
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_bytes[i - 1] == b_bytes[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

// ══════════════════════════════════════════════════
// HELPERS
// ══════════════════════════════════════════════════

/// Remove content between <tag>...</tag> (e.g., strip scripts)
fn remove_tag_content(html: &str, tag: &str) -> String {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let mut result = String::with_capacity(html.len());
    let mut pos = 0;

    while pos < html.len() {
        if let Some(start) = html[pos..].find(&open) {
            result.push_str(&html[pos..pos+start]);
            if let Some(end) = html[pos+start..].find(&close) {
                pos = pos + start + end + close.len();
            } else {
                break;
            }
        } else {
            result.push_str(&html[pos..]);
            break;
        }
    }
    result
}

/// Strip HTML tags, returning only text content
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        if c == '<' { in_tag = true; continue; }
        if c == '>' { in_tag = false; result.push(' '); continue; }
        if !in_tag { result.push(c); }
    }
    result
}

/// Strip tags but keep attribute values for context
fn strip_tags_preserve_attrs(html: &str) -> String {
    // For metric detection, we want to see text outside of placeholder/id attrs
    let without_attrs = remove_attribute_values(html, "placeholder");
    let without_ids = remove_attribute_values(&without_attrs, "id");
    strip_html_tags(&without_ids)
}

/// Remove values of a specific attribute from HTML
fn remove_attribute_values(html: &str, attr: &str) -> String {
    let pattern = format!("{}=", attr);
    let mut result = html.to_string();
    let mut search_from = 0;
    while let Some(idx) = result[search_from..].find(&pattern) {
        let abs = search_from + idx;
        let after = abs + pattern.len();
        if after >= result.len() { break; }
        let quote = result.as_bytes()[after];
        if quote == b'"' || quote == b'\'' {
            if let Some(end) = result[after+1..].find(quote as char) {
                let remove_start = after + 1;
                let remove_end = after + 1 + end;
                result.replace_range(remove_start..remove_end, "");
                search_from = remove_start + 1;
            } else {
                search_from = after + 1;
            }
        } else {
            search_from = after + 1;
        }
    }
    result
}

/// Check if a text appears inside a placeholder="" attribute
fn is_in_placeholder(html: &str, text: &str) -> bool {
    if let Some(idx) = html.find(text) {
        // Look backwards for placeholder="
        let before = if idx > 50 { &html[idx-50..idx] } else { &html[..idx] };
        before.contains("placeholder=")
    } else {
        false
    }
}

/// Check if a text appears inside an element with id="" (will be JS-populated)
fn is_in_id_element(html: &str, text: &str) -> bool {
    if let Some(idx) = html.find(text) {
        // Look backwards for id="something" in the containing element
        let search_start = if idx > 200 { idx - 200 } else { 0 };
        let before = &html[search_start..idx];
        // Find the last opening tag before this text
        if let Some(tag_start) = before.rfind('<') {
            let tag = &before[tag_start..];
            return tag.contains("id=");
        }
    }
    false
}

// ══════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::parser::{ActionBlock, ActionInstruction, ApiNode, EntityNode, FieldNode, FieldType, FilterExpr, FilterOp, BindingValue, BindingNode, GroupByExpr, HttpMethod, OrderExpr, OrderDirection, QueryType, RouteNode};

    fn make_section(template: &str) -> SectionNode {
        SectionNode {
            section_type: "table".into(),
            title: None,
            subtitle: None,
            config: HashMap::new(),
            items: vec![],
            plans: vec![],
            binding: None,
            actions: vec![],
            visibility: None,
            template: Some(template.to_string()),
            style_block: None,
            doc: None,
        }
    }

    fn make_section_with_columns(template: &str, columns: &str) -> SectionNode {
        let mut s = make_section(template);
        s.config.insert("columns".into(), columns.into());
        s
    }

    fn sensitive_entity() -> EntityNode {
        EntityNode {
            name: "User".into(),
            fields: vec![
                FieldNode {
                    name: "name".into(),
                    field_type: FieldType::String,
                    required: true, unique: false, sensitive: false,
                    optional: false, searchable: false, index: false,
                    featured: false, formatted: false, array: false,
                    enum_values: None, reference: None, doc: None,
                },
                FieldNode {
                    name: "password".into(),
                    field_type: FieldType::String,
                    required: false, unique: false, sensitive: true,
                    optional: false, searchable: false, index: false,
                    featured: false, formatted: false, array: false,
                    enum_values: None, reference: None, doc: None,
                },
                FieldNode {
                    name: "secret_key".into(),
                    field_type: FieldType::String,
                    required: false, unique: false, sensitive: true,
                    optional: false, searchable: false, index: false,
                    featured: false, formatted: false, array: false,
                    enum_values: None, reference: None, doc: None,
                },
            ],
            shared: false,
            doc: None,
        }
    }

    // ── C002: no-sensitive-render ──

    #[test]
    fn c002_catches_sensitive_field_in_table_column() {
        let section = make_section_with_columns("<table></table>", "name, email, password");
        let fields = vec!["password".to_string()];
        let results = rule_no_sensitive_render(&section, "table (/users)", "/users", &fields);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule, "no-sensitive-render");
        assert!(matches!(results[0].severity, Severity::Error));
        assert!(results[0].message.contains("password"));
    }

    #[test]
    fn c002_catches_sensitive_field_in_visible_html() {
        let html = "<div class='user-card'><span>password</span></div>";
        let section = make_section(html);
        let fields = vec!["password".to_string()];
        let results = rule_no_sensitive_render(&section, "hero (/users)", "/users", &fields);
        assert_eq!(results.len(), 1);
        assert!(results[0].message.contains("password"));
    }

    #[test]
    fn c002_allows_password_in_type_password_input() {
        let html = "<input type='password' name='password' placeholder='Enter password'>";
        let section = make_section(html);
        let fields = vec!["password".to_string()];
        let results = rule_no_sensitive_render(&section, "form (/login)", "/login", &fields);
        assert!(results.is_empty(), "type=password input should not trigger: {:?}", results);
    }

    #[test]
    fn c002_allows_password_in_script() {
        let html = "<script>var data = {password: form.password.value};</script>";
        let section = make_section(html);
        let fields = vec!["password".to_string()];
        let results = rule_no_sensitive_render(&section, "hero (/login)", "/login", &fields);
        assert!(results.is_empty(), "script content should not trigger: {:?}", results);
    }

    #[test]
    fn c002_allows_password_in_placeholder() {
        let html = "<input type='text' placeholder='password'>";
        let section = make_section(html);
        let fields = vec!["password".to_string()];
        let results = rule_no_sensitive_render(&section, "form (/reset)", "/reset", &fields);
        assert!(results.is_empty(), "placeholder should not trigger: {:?}", results);
    }

    #[test]
    fn c002_no_false_positive_on_partial_match() {
        let html = "<div>passwords policy</div>";
        let section = make_section(html);
        let fields = vec!["password".to_string()];
        let results = rule_no_sensitive_render(&section, "hero (/)", "/", &fields);
        assert!(results.is_empty(), "partial word match should not trigger: {:?}", results);
    }

    #[test]
    fn c002_allows_name_attribute_value() {
        let html = "<input type='text' name='password'>";
        let section = make_section(html);
        let fields = vec!["password".to_string()];
        let results = rule_no_sensitive_render(&section, "form (/)", "/", &fields);
        assert!(results.is_empty(), "name attribute value should not trigger: {:?}", results);
    }

    #[test]
    fn c002_no_results_when_no_sensitive_fields() {
        let html = "<div>password visible</div>";
        let section = make_section(html);
        let fields: Vec<String> = vec![];
        let results = rule_no_sensitive_render(&section, "hero (/)", "/", &fields);
        assert!(results.is_empty());
    }

    // ── C010: enhanced dead-link route validation ──

    #[test]
    fn c010_allows_api_routes() {
        let section = make_section("<a href='/api/auth/login'>Login</a>");
        let routes = vec!["/".to_string()];
        let kernel = vec!["/docs".to_string()];
        let results = rule_no_dead_links(&section, "hero (/)", "/", &routes, &kernel);
        assert!(results.is_empty(), "API routes should be allowed: {:?}", results);
    }

    #[test]
    fn c010_suggests_similar_route() {
        let section = make_section("<a href='/settigns'>Settings</a>");
        let routes = vec!["/settings".to_string(), "/dashboard".to_string()];
        let kernel = vec!["/docs".to_string()];
        let results = rule_no_dead_links(&section, "nav (/)", "/", &routes, &kernel);
        assert_eq!(results.len(), 1);
        assert!(results[0].fix.contains("/settings"), "Should suggest /settings: {}", results[0].fix);
    }

    #[test]
    fn c010_warns_unknown_internal_route() {
        let section = make_section("<a href='/nonexistent'>Link</a>");
        let routes = vec!["/".to_string(), "/about".to_string()];
        let kernel = vec!["/docs".to_string()];
        let results = rule_no_dead_links(&section, "hero (/)", "/", &routes, &kernel);
        assert_eq!(results.len(), 1);
        assert!(results[0].message.contains("/nonexistent"));
    }

    #[test]
    fn c010_allows_kernel_routes() {
        let section = make_section("<a href='/docs'>Docs</a><a href='/graphql'>API</a>");
        let routes = vec!["/".to_string()];
        let kernel = vec!["/docs".to_string(), "/graphql".to_string()];
        let results = rule_no_dead_links(&section, "hero (/)", "/", &routes, &kernel);
        assert!(results.is_empty(), "Kernel routes should be valid: {:?}", results);
    }

    #[test]
    fn c010_allows_declared_page_routes() {
        let section = make_section("<a href='/about'>About</a>");
        let routes = vec!["/".to_string(), "/about".to_string()];
        let kernel = vec![];
        let results = rule_no_dead_links(&section, "hero (/)", "/", &routes, &kernel);
        assert!(results.is_empty());
    }

    // ── edit distance ──

    #[test]
    fn edit_distance_basic() {
        assert_eq!(simple_edit_distance("kitten", "sitting"), 3);
        assert_eq!(simple_edit_distance("", "abc"), 3);
        assert_eq!(simple_edit_distance("abc", "abc"), 0);
        assert_eq!(simple_edit_distance("/settings", "/settigns"), 2);
    }

    // ── Integration: lint_ast with sensitive fields ──

    #[test]
    fn c002_integration_catches_via_lint_ast() {
        let entity = AstNode::Entity(sensitive_entity());
        let page = AstNode::Page(PageNode {
            route: "/admin/users".into(),
            page_type: "custom".into(),
            entity: None,
            title: None,
            sections: vec![make_section_with_columns("<table></table>", "name, email, password")],
            config: HashMap::new(),
            components: vec![],
            requires: None,
            doc: None,
        });
        let results = lint_ast(&[entity, page], false);
        let sensitive_results: Vec<_> = results.iter().filter(|r| r.rule == "no-sensitive-render").collect();
        assert!(!sensitive_results.is_empty(), "lint_ast should catch sensitive field in columns");
    }

    // ── C012: form-submit-handler ──

    fn make_form_section(template: Option<&str>, actions: Vec<ActionBlock>) -> SectionNode {
        SectionNode {
            section_type: "form".into(),
            title: None,
            subtitle: None,
            config: HashMap::new(),
            items: vec![],
            plans: vec![],
            binding: None,
            actions,
            visibility: None,
            template: template.map(|t| t.to_string()),
            style_block: None,
            doc: None,
        }
    }

    #[test]
    fn c012_catches_form_without_submit() {
        let section = make_form_section(Some("<form><input type='text'></form>"), vec![]);
        let results = rule_form_submit_handler(&section, "form (/contact)", "/contact");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule, "form-submit-handler");
        assert!(matches!(results[0].severity, Severity::Warning));
    }

    #[test]
    fn c012_allows_form_with_submit_action() {
        let action = ActionBlock {
            event: "submit".into(),
            confirm: None,
            instructions: vec![],
        };
        let section = make_form_section(Some("<form></form>"), vec![action]);
        let results = rule_form_submit_handler(&section, "form (/contact)", "/contact");
        assert!(results.is_empty(), "form with submit action should pass: {:?}", results);
    }

    #[test]
    fn c012_allows_form_with_submit_button_in_template() {
        let section = make_form_section(
            Some("<form><button type=\"submit\">Send</button></form>"),
            vec![],
        );
        let results = rule_form_submit_handler(&section, "form (/contact)", "/contact");
        assert!(results.is_empty(), "form with type=submit button should pass: {:?}", results);
    }

    #[test]
    fn c012_allows_form_with_onsubmit() {
        let section = make_form_section(
            Some("<form onsubmit=\"handleSubmit()\"></form>"),
            vec![],
        );
        let results = rule_form_submit_handler(&section, "form (/contact)", "/contact");
        assert!(results.is_empty(), "form with onsubmit should pass: {:?}", results);
    }

    #[test]
    fn c012_allows_form_with_data_cronus_form() {
        let section = make_form_section(
            Some("<form data-cronus-form></form>"),
            vec![],
        );
        let results = rule_form_submit_handler(&section, "form (/contact)", "/contact");
        assert!(results.is_empty(), "form with data-cronus-form should pass: {:?}", results);
    }

    #[test]
    fn c012_ignores_non_form_sections() {
        let mut section = make_section("<div>not a form</div>");
        section.section_type = "hero".into();
        let results = rule_form_submit_handler(&section, "hero (/)", "/");
        assert!(results.is_empty(), "non-form section should be ignored");
    }

    #[test]
    fn c012_strict_promotes_to_error() {
        let section = make_form_section(Some("<form><input></form>"), vec![]);
        let page = AstNode::Page(PageNode {
            route: "/contact".into(),
            page_type: "custom".into(),
            entity: None,
            title: None,
            sections: vec![section],
            config: HashMap::new(),
            components: vec![],
            requires: None,
            doc: None,
        });
        let results = lint_ast(&[page], true);
        let form_results: Vec<_> = results.iter().filter(|r| r.rule == "form-submit-handler").collect();
        assert!(!form_results.is_empty(), "should catch form without submit");
        assert!(matches!(form_results[0].severity, Severity::Error), "strict mode should promote to Error");
    }

    // ── C030: shared-entity-auth ──

    fn make_shared_entity(name: &str) -> EntityNode {
        EntityNode {
            name: name.into(),
            fields: vec![],
            shared: true,
            doc: None,
        }
    }

    fn make_api(prefix: &str, routes: Vec<RouteNode>) -> ApiNode {
        ApiNode {
            prefix: prefix.into(),
            routes,
            doc: None,
        }
    }

    fn make_route(name: &str, method: HttpMethod, path: &str, auth: &str) -> RouteNode {
        RouteNode {
            name: name.into(),
            method,
            path: path.into(),
            auth: auth.into(),
            roles: vec![],
            doc: None,
        }
    }

    #[test]
    fn c030_catches_public_post_on_shared_entity() {
        let entity = make_shared_entity("Product");
        let api = make_api("/products", vec![
            make_route("create", HttpMethod::POST, "/", "public"),
        ]);
        let results = rule_shared_entity_auth(&[&entity], &[&api]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule, "shared-entity-auth");
        assert!(matches!(results[0].severity, Severity::Error));
    }

    #[test]
    fn c030_catches_empty_auth_on_delete() {
        let entity = make_shared_entity("Product");
        let api = make_api("/products", vec![
            make_route("delete", HttpMethod::DELETE, "/:id", ""),
        ]);
        let results = rule_shared_entity_auth(&[&entity], &[&api]);
        assert_eq!(results.len(), 1);
        assert!(results[0].message.contains("DELETE"));
    }

    #[test]
    fn c030_allows_public_get_on_shared_entity() {
        let entity = make_shared_entity("Product");
        let api = make_api("/products", vec![
            make_route("list", HttpMethod::GET, "/", "public"),
        ]);
        let results = rule_shared_entity_auth(&[&entity], &[&api]);
        assert!(results.is_empty(), "GET on shared entity can be public: {:?}", results);
    }

    #[test]
    fn c030_allows_authed_post_on_shared_entity() {
        let entity = make_shared_entity("Product");
        let api = make_api("/products", vec![
            make_route("create", HttpMethod::POST, "/", "token"),
        ]);
        let results = rule_shared_entity_auth(&[&entity], &[&api]);
        assert!(results.is_empty(), "authed POST should pass: {:?}", results);
    }

    #[test]
    fn c030_ignores_non_shared_entity() {
        let entity = EntityNode {
            name: "Draft".into(),
            fields: vec![],
            shared: false,
            doc: None,
        };
        let api = make_api("/drafts", vec![
            make_route("create", HttpMethod::POST, "/", "public"),
        ]);
        let results = rule_shared_entity_auth(&[&entity], &[&api]);
        assert!(results.is_empty(), "non-shared entity public POST is OK: {:?}", results);
    }

    #[test]
    fn c030_catches_multiple_violations() {
        let entity = make_shared_entity("Product");
        let api = make_api("/products", vec![
            make_route("create", HttpMethod::POST, "/", "public"),
            make_route("update", HttpMethod::PATCH, "/:id", "none"),
            make_route("list", HttpMethod::GET, "/", "public"),
            make_route("delete", HttpMethod::DELETE, "/:id", ""),
        ]);
        let results = rule_shared_entity_auth(&[&entity], &[&api]);
        assert_eq!(results.len(), 3, "should catch POST, PATCH, DELETE but not GET: {:?}", results);
    }

    // ── C031: no-sensitive-select ──

    #[test]
    fn c031_catches_sensitive_field_in_table_columns() {
        let mut section = make_section("<table></table>");
        section.section_type = "table".into();
        section.config.insert("columns".into(), "name, email, password".into());
        let fields = vec!["password".to_string()];
        let results = rule_no_sensitive_select(&section, "table (/users)", "/users", &fields);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule, "no-sensitive-select");
        assert!(results[0].message.contains("password"));
    }

    #[test]
    fn c031_allows_non_sensitive_columns() {
        let mut section = make_section("<table></table>");
        section.section_type = "table".into();
        section.config.insert("columns".into(), "name, email, role".into());
        let fields = vec!["password".to_string()];
        let results = rule_no_sensitive_select(&section, "table (/users)", "/users", &fields);
        assert!(results.is_empty());
    }

    #[test]
    fn c031_catches_sensitive_field_in_bind_filter() {
        let mut section = make_section("<div></div>");
        section.binding = Some(BindingNode {
            entity: "User".into(),
            query: QueryType::All,
            filters: vec![FilterExpr {
                field: "password".into(),
                operator: FilterOp::Eq,
                value: BindingValue::Str("test".into()),
            }],
            order: None,
            limit: None,
            offset: None,
            group_by: None,
            aggregate: None,
        });
        let fields = vec!["password".to_string()];
        let results = rule_no_sensitive_select(&section, "table (/users)", "/users", &fields);
        assert_eq!(results.len(), 1);
        assert!(results[0].message.contains("filter"));
    }

    #[test]
    fn c031_catches_sensitive_field_in_order_by() {
        let mut section = make_section("<div></div>");
        section.binding = Some(BindingNode {
            entity: "User".into(),
            query: QueryType::All,
            filters: vec![],
            order: Some(OrderExpr {
                field: "secret_key".into(),
                direction: OrderDirection::Asc,
            }),
            limit: None,
            offset: None,
            group_by: None,
            aggregate: None,
        });
        let fields = vec!["secret_key".to_string()];
        let results = rule_no_sensitive_select(&section, "table (/users)", "/users", &fields);
        assert_eq!(results.len(), 1);
        assert!(results[0].message.contains("order"));
    }

    #[test]
    fn c031_catches_sensitive_field_in_group_by() {
        let mut section = make_section("<div></div>");
        section.binding = Some(BindingNode {
            entity: "User".into(),
            query: QueryType::All,
            filters: vec![],
            order: None,
            limit: None,
            offset: None,
            group_by: Some(GroupByExpr {
                field: "password".into(),
                interval: None,
            }),
            aggregate: None,
        });
        let fields = vec!["password".to_string()];
        let results = rule_no_sensitive_select(&section, "table (/users)", "/users", &fields);
        assert_eq!(results.len(), 1);
        assert!(results[0].message.contains("group_by"));
    }

    #[test]
    fn c031_no_results_when_no_sensitive_fields() {
        let mut section = make_section("<table></table>");
        section.section_type = "table".into();
        section.config.insert("columns".into(), "name, email".into());
        let fields: Vec<String> = vec![];
        let results = rule_no_sensitive_select(&section, "table (/users)", "/users", &fields);
        assert!(results.is_empty());
    }

    #[test]
    fn c031_only_checks_table_sections_for_columns() {
        // hero section with columns config should NOT trigger C031 (not a table)
        let mut section = make_section("<div></div>");
        section.section_type = "hero".into();
        section.config.insert("columns".into(), "name, password".into());
        let fields = vec!["password".to_string()];
        let results = rule_no_sensitive_select(&section, "hero (/)", "/", &fields);
        assert!(results.is_empty(), "non-table section columns should not trigger C031");
    }
}
