// hardcode_lint.rs — Detects hardcoded content in rendered HTML
//
// Compares visible text in rendered HTML against .cronus section data.
// Any text that doesn't trace back to section title/subtitle/config/items
// is flagged as a hardcoded string.

use crate::parser::{PageNode, SectionNode, StyleNode, EntityNode};
use std::collections::HashSet;

/// A single hardcode finding
#[derive(Debug)]
pub struct HardcodeFinding {
    pub page: String,
    pub text: String,
    pub severity: &'static str, // "warning" or "error"
}

/// Collect all user-facing strings from a section's DSL data
fn collect_section_strings(section: &SectionNode) -> HashSet<String> {
    let mut strings = HashSet::new();

    // Title and subtitle
    if let Some(ref t) = section.title {
        strings.insert(t.clone());
    }
    if let Some(ref s) = section.subtitle {
        strings.insert(s.clone());
    }

    // All config values
    for (_, v) in &section.config {
        if !v.is_empty() && v.len() > 1 {
            strings.insert(v.clone());
            // Also add comma-separated parts (for nav "Inventory, Analytics, ...")
            for part in v.split(',').map(|s| s.trim()) {
                if part.len() > 1 {
                    strings.insert(part.to_string());
                }
            }
        }
    }

    // All item values
    for item in &section.items {
        for (_, v) in item {
            if !v.is_empty() && v.len() > 1 && v != "item" && v != "action" && v != "image" && v != "row" && v != "true" && v != "false" {
                strings.insert(v.clone());
            }
        }
    }

    // Plan names/features
    for plan in &section.plans {
        strings.insert(plan.name.clone());
        strings.insert(plan.price.clone());
        for f in &plan.features {
            strings.insert(f.clone());
        }
    }

    strings
}

/// Collect all DSL strings from all sections of a page
fn collect_page_strings(page: &PageNode) -> HashSet<String> {
    let mut strings = HashSet::new();

    // Page title
    if let Some(ref t) = page.title {
        strings.insert(t.clone());
    }

    // Page config
    for (_, v) in &page.config {
        if !v.is_empty() && v.len() > 1 {
            strings.insert(v.clone());
        }
    }

    // All sections
    for section in &page.sections {
        strings.extend(collect_section_strings(section));
    }

    strings
}

/// Extract visible text from rendered HTML
fn extract_visible_text(html: &str) -> Vec<String> {
    let mut texts = Vec::new();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut current = String::new();

    let chars: Vec<char> = html.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        if ch == '<' {
            // Flush current text
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() && !in_script && !in_style {
                texts.push(trimmed);
            }
            current.clear();

            // Check for script/style open/close
            let rest: String = chars[i..std::cmp::min(i + 20, len)].iter().collect();
            let rest_lower = rest.to_lowercase();
            if rest_lower.starts_with("<script") {
                in_script = true;
            } else if rest_lower.starts_with("</script") {
                in_script = false;
            } else if rest_lower.starts_with("<style") {
                in_style = true;
            } else if rest_lower.starts_with("</style") {
                in_style = false;
            }
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            current.push(ch);
        }

        i += 1;
    }

    // Flush remaining
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() && !in_script && !in_style {
        texts.push(trimmed);
    }

    texts
}

/// Strings that are structural / not content (icons, CSS class names, etc.)
fn is_structural(text: &str) -> bool {
    let trimmed = text.trim();

    // Too short to be meaningful content
    if trimmed.len() < 3 {
        return true;
    }

    // HTML entities (&#x2026; etc.)
    if trimmed.starts_with("&#") || trimmed.starts_with("&amp;") || trimmed.starts_with("&lt;") || trimmed.starts_with("&gt;") {
        return true;
    }

    // Material icon names (lowercase with underscores)
    if trimmed.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
        return true;
    }

    // Pure numbers / CSS values
    if trimmed.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '%' || c == 'p' || c == 'x' || c == 'e' || c == 'm') {
        return true;
    }

    // Common HTML/CSS structural words
    let structural_words = [
        "none", "auto", "inherit", "block", "flex", "grid", "true", "false",
        "item", "action", "image", "row", "text", "email", "password",
    ];
    if structural_words.contains(&trimmed.to_lowercase().as_str()) {
        return true;
    }

    false
}

/// Run the hardcode lint on a rendered page
pub fn lint_page(
    page: &PageNode,
    rendered_html: &str,
) -> Vec<HardcodeFinding> {
    let dsl_strings = collect_page_strings(page);
    let visible_texts = extract_visible_text(rendered_html);

    let mut findings = Vec::new();

    for text in &visible_texts {
        if is_structural(text) {
            continue;
        }

        // Check if this text (or a substring of it) exists in DSL strings
        let found = dsl_strings.iter().any(|dsl| {
            // Exact match
            dsl == text
            // DSL contains this text
            || dsl.contains(text.as_str())
            // This text contains a DSL string (e.g., combined text in a single element)
            || text.contains(dsl.as_str())
            // Partial match for multi-word text
            || text.split_whitespace().all(|word| {
                word.len() < 3 || dsl_strings.iter().any(|d| d.contains(word))
            })
        });

        if !found {
            findings.push(HardcodeFinding {
                page: page.route.clone(),
                text: text.clone(),
                severity: "warning",
            });
        }
    }

    findings
}

/// Run lint on all pages — called from cmd_build --strict
pub fn lint_all_pages(
    pages: &[PageNode],
    entities: &[EntityNode],
    style: Option<&StyleNode>,
) -> Vec<HardcodeFinding> {
    let accent = style.and_then(|s| s.accent.as_deref()).unwrap_or("#3b82f6");
    let theme = style.and_then(|s| s.theme.as_deref()).unwrap_or("dark");
    let empty_params = std::collections::HashMap::new();

    // Disable strict mode during lint rendering so we get actual output, not error messages
    let was_strict = crate::STRICT_MODE.swap(false, std::sync::atomic::Ordering::Relaxed);

    let mut all_findings = Vec::new();

    for page in pages {
        // Render the page body
        let body = crate::ui::render_page(page, entities, accent, theme, None, &empty_params, "");

        // Run lint
        let findings = lint_page(page, &body);
        all_findings.extend(findings);
    }

    // Restore strict mode
    crate::STRICT_MODE.store(was_strict, std::sync::atomic::Ordering::Relaxed);

    all_findings
}
