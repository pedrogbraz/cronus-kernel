//! Canonical `.cronus` grammar facts for AI-facing context.
//!
//! `cronus context --for-claude` prints the grammar summary and the "never do"
//! list embedded from `llms-full.txt`, plus the section and field types below.
//! The tests keep every copy in sync with the code: built-in sections against
//! the dispatcher match in `src/ui/mod.rs`, families against
//! `cronus_ui_widgets::FAMILIES`, aliases against
//! `ContractRegistry::resolve_alias`, field types against `FieldType`, and the
//! marked blocks of `llms-full.txt` against all of them.

/// The full LLM reference shipped at the kernel root.
pub const LLMS_FULL: &str = include_str!("../../llms-full.txt");

/// Canonical field type keywords accepted by `FieldType::from_keyword` (relations use `->`).
pub const FIELD_TYPES: &[&str] = &[
    "string",
    "text",
    "email",
    "url",
    "file",
    "slug",
    "phone",
    "number",
    "money",
    "percentage",
    "boolean",
    "date",
    "datetime",
    "ulid",
    "json",
    "enum",
    "ip",
];

/// Section types with an explicit arm in `ui::render_section_inner`, in
/// dispatcher order.
pub const BUILTIN_SECTION_TYPES: &[&str] = &[
    "hero",
    "features",
    "pricing",
    "cta",
    "faq",
    "stats",
    "trusted",
    "topbar",
    "checkout",
    "testimonial",
    "footer",
    "page-header",
    "stat-cards",
    "product-grid",
    "promo",
    "info-bar",
    "bento",
    "features-split",
    "team-list",
    "status-card",
    "policies",
    "activity-table",
    "edge",
    "sidebar",
    "form",
    "card",
    "live-keys",
    "test-keys",
    "webhooks",
    "links",
    "quick-links",
    "tabs",
    "accordion",
    "breadcrumb",
    "alert",
    "chart",
    "modal",
    "sheet",
    "skeleton",
    "loading",
    "empty",
    "error",
    "not-found",
    "404",
    "kpi",
    "timeline",
    "progress",
    "command",
    "table",
    "pagination",
    "filters",
    "dropdown",
    "toast",
    "notifications",
    "kanban",
    "dark-mode",
    "layout",
];

/// Section names that `ContractRegistry::resolve_alias` rewrites.
pub const SECTION_ALIASES: &[&str] = &[
    "stats",
    "stat-cards",
    "status-card",
    "activity-table",
    "team-list",
    "policies",
    "live-keys",
    "test-keys",
    "webhooks",
    "quick-links",
    "promo",
    "info-bar",
    "edge",
    "bento",
    "features-split",
    "product-grid",
];

/// cronus-ui families rendered by the dispatcher fallback and by components.
pub fn family_section_types() -> &'static [&'static str] {
    crate::cronus_ui_widgets::FAMILIES
}

/// Text between `<!-- begin:name -->` and `<!-- end:name -->`, trimmed.
pub fn marked_block<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    let begin = format!("<!-- begin:{name} -->");
    let end = format!("<!-- end:{name} -->");
    let start = text.find(&begin)? + begin.len();
    let stop = text[start..].find(&end)? + start;
    Some(text[start..stop].trim())
}

/// Canonical grammar summary (the `grammar` block of `llms-full.txt`).
pub fn grammar_summary() -> &'static str {
    marked_block(LLMS_FULL, "grammar").unwrap_or("")
}

/// "Never do" list (the `never` block of `llms-full.txt`).
pub fn never_list() -> &'static str {
    marked_block(LLMS_FULL, "never").unwrap_or("")
}

/// Every `code span` in a block, in order.
pub fn code_spans(block: &str) -> Vec<String> {
    block
        .split('`')
        .skip(1)
        .step_by(2)
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{self, FieldType};
    use std::collections::BTreeSet;

    /// String patterns of the top-level arms of the section dispatcher match.
    fn dispatcher_arms() -> Vec<String> {
        let src = include_str!("../ui/mod.rs");
        let start = src
            .find("let section_html = match resolved_type {")
            .expect("dispatcher match not found in src/ui/mod.rs");
        let body = &src[start..];
        let stop = body
            .find("\n        _ => {")
            .expect("dispatcher default arm not found");
        let mut names = Vec::new();
        for line in body[..stop].lines().skip(1) {
            // Top-level arms sit at 8 spaces; nested matches are deeper.
            if !line.starts_with("        \"") {
                continue;
            }
            let pattern = match line.find("=>") {
                Some(i) => &line[..i],
                None => continue,
            };
            for part in pattern.split('|') {
                let name = part.trim().trim_matches('"');
                if !name.is_empty() {
                    names.push(name.to_string());
                }
            }
        }
        names
    }

    fn set<I: IntoIterator<Item = S>, S: Into<String>>(items: I) -> BTreeSet<String> {
        items.into_iter().map(Into::into).collect()
    }

    fn diff(label: &str, doc: &BTreeSet<String>, code: &BTreeSet<String>) -> Option<String> {
        let missing: Vec<_> = code.difference(doc).collect();
        let extra: Vec<_> = doc.difference(code).collect();
        if missing.is_empty() && extra.is_empty() {
            None
        } else {
            Some(format!(
                "{label}: missing from doc {missing:?}, not in code {extra:?}"
            ))
        }
    }

    fn keyword(ft: &FieldType) -> Option<&'static str> {
        // Exhaustive on purpose: a new FieldType variant fails to compile here
        // until FIELD_TYPES and llms-full.txt are updated.
        match ft {
            FieldType::String => Some("string"),
            FieldType::Text => Some("text"),
            FieldType::Email => Some("email"),
            FieldType::Url => Some("url"),
            FieldType::File => Some("file"),
            FieldType::Slug => Some("slug"),
            FieldType::Phone => Some("phone"),
            FieldType::Number => Some("number"),
            FieldType::Money => Some("money"),
            FieldType::Percentage => Some("percentage"),
            FieldType::Boolean => Some("boolean"),
            FieldType::Date => Some("date"),
            FieldType::DateTime => Some("datetime"),
            FieldType::Ulid => Some("ulid"),
            FieldType::Json => Some("json"),
            FieldType::Enum => Some("enum"),
            FieldType::Ip => Some("ip"),
            FieldType::Relation => None,
        }
    }

    #[test]
    fn context_grammar_builtin_sections_match_dispatcher() {
        let arms = dispatcher_arms();
        assert_eq!(
            arms,
            BUILTIN_SECTION_TYPES
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            "BUILTIN_SECTION_TYPES drifted from src/ui/mod.rs render_section_inner"
        );
    }

    #[test]
    fn context_grammar_aliases_match_registry() {
        for alias in SECTION_ALIASES {
            let canonical = crate::contracts::ContractRegistry::resolve_alias(alias)
                .unwrap_or_else(|| panic!("{alias} is not an alias any more"));
            assert!(
                BUILTIN_SECTION_TYPES.contains(&canonical),
                "{alias} resolves to {canonical}, which has no dispatcher arm"
            );
        }
        // Every dispatcher name that resolves elsewhere must be listed.
        for name in BUILTIN_SECTION_TYPES {
            if let Some(c) = crate::contracts::ContractRegistry::resolve_alias(name) {
                assert!(
                    SECTION_ALIASES.contains(name),
                    "{name} -> {c} missing from SECTION_ALIASES"
                );
            }
        }
    }

    #[test]
    fn context_grammar_field_types_match_enum() {
        for name in FIELD_TYPES {
            assert_eq!(
                FieldType::from_keyword(name).as_ref().and_then(keyword),
                Some(*name),
                "{name} is not parsed as its own FieldType"
            );
        }
        let variants = [
            FieldType::String,
            FieldType::Text,
            FieldType::Email,
            FieldType::Url,
            FieldType::File,
            FieldType::Slug,
            FieldType::Phone,
            FieldType::Number,
            FieldType::Money,
            FieldType::Percentage,
            FieldType::Boolean,
            FieldType::Date,
            FieldType::DateTime,
            FieldType::Ulid,
            FieldType::Json,
            FieldType::Enum,
            FieldType::Ip,
            FieldType::Relation,
        ];
        let keywords: Vec<_> = variants.iter().filter_map(keyword).collect();
        assert_eq!(keywords, FIELD_TYPES.to_vec());
    }

    #[test]
    fn context_grammar_llms_full_lists_match_code() {
        let mut problems = Vec::new();
        let block = |name: &str| {
            marked_block(LLMS_FULL, name)
                .unwrap_or_else(|| panic!("llms-full.txt has no {name} block"))
        };

        let doc = set(code_spans(block("builtin-sections")));
        let code = set(dispatcher_arms());
        problems.extend(diff("builtin-sections", &doc, &code));

        let doc = set(code_spans(block("family-sections")));
        let code = set(family_section_types().iter().copied());
        problems.extend(diff("family-sections", &doc, &code));

        let doc = set(code_spans(block("field-types")));
        let code = set(FIELD_TYPES.iter().copied());
        problems.extend(diff("field-types", &doc, &code));

        let aliases_block = block("section-aliases");
        let doc = set(code_spans(aliases_block));
        let code = set(SECTION_ALIASES.iter().copied());
        problems.extend(diff("section-aliases", &doc, &code));
        for entry in aliases_block.split(',') {
            let spans = code_spans(entry);
            let (Some(alias), Some(target)) = (spans.first(), entry.split('→').nth(1)) else {
                continue;
            };
            let target = target.trim();
            let resolved = crate::contracts::ContractRegistry::resolve_alias(alias);
            if resolved != Some(target) {
                problems.push(format!(
                    "alias {alias}: doc says {target}, code says {resolved:?}"
                ));
            }
        }

        assert!(
            problems.is_empty(),
            "llms-full.txt drifted from the code:\n  {}",
            problems.join("\n  ")
        );
    }

    #[test]
    fn context_grammar_llms_full_example_builds_clean() {
        let block = marked_block(LLMS_FULL, "example").expect("example block");
        let source = block
            .trim_start_matches("```cronus")
            .trim_end_matches("```")
            .trim();
        let nodes = parser::parse(source).expect("llms-full.txt example must parse");
        let (e, p, r) = parser::stats(&nodes);
        let result = crate::cli::build::build_ai_error_json(&nodes, "llms-full.txt", e, p, r);
        assert_eq!(
            result["valid"],
            serde_json::json!(true),
            "llms-full.txt example has build errors: {}",
            result["errors"]
        );
    }

    #[test]
    fn context_grammar_blocks_present() {
        assert!(grammar_summary().contains("### entity"));
        assert!(never_list().contains("Never"));
        assert!(!grammar_summary().contains("formatPrice"));
    }
}
