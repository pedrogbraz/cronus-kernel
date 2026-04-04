#![allow(dead_code, unused_imports, unused_variables)]
//! DOM parsing and analysis using the `scraper` crate.

use std::collections::HashMap;
use scraper::{Html, Selector, ElementRef, Node};

/// Known Material Symbols class prefixes that mark icon font spans.
const MATERIAL_SYMBOL_CLASSES: &[&str] = &[
    "material-symbols-outlined",
    "material-symbols-rounded",
    "material-symbols-sharp",
    "material-icons",
];

/// Simplified DOM node for analysis
#[derive(Debug, Clone)]
pub struct DomNode {
    pub tag: String,
    pub classes: Vec<String>,
    pub id: Option<String>,
    pub text: String,
    pub full_text: String,
    pub attrs: HashMap<String, String>,
    pub children: Vec<DomNode>,
    pub depth: u32,
}

// ---------------------------------------------------------------------------
// Material-symbols text cleaning
// ---------------------------------------------------------------------------

/// Check if an `ElementRef` has a material-symbols icon class.
fn is_material_symbols_element(el: &ElementRef) -> bool {
    el.value().classes().any(|c| {
        MATERIAL_SYMBOL_CLASSES.iter().any(|ms| c.contains(ms))
    })
}

/// Extract text from an `ElementRef` while skipping children that have a
/// material-symbols class. Returns cleaned, whitespace-collapsed text.
pub fn clean_element_text(el: &ElementRef) -> String {
    let mut parts: Vec<String> = Vec::new();
    for child_node in el.children() {
        match child_node.value() {
            Node::Text(text) => {
                let t = text.text.trim();
                if !t.is_empty() {
                    parts.push(t.to_string());
                }
            }
            Node::Element(_) => {
                if let Some(child_el) = ElementRef::wrap(child_node) {
                    if is_material_symbols_element(&child_el) {
                        continue; // skip icon span entirely
                    }
                    // Recurse into non-icon children
                    let child_text = clean_element_text_recursive(&child_el);
                    if !child_text.is_empty() {
                        parts.push(child_text);
                    }
                }
            }
            _ => {}
        }
    }
    let joined = parts.join(" ");
    collapse_whitespace(&joined)
}

/// Recursively extract text from an ElementRef, skipping material-symbols
/// descendants at any depth.
fn clean_element_text_recursive(el: &ElementRef) -> String {
    let mut parts: Vec<String> = Vec::new();
    for child_node in el.children() {
        match child_node.value() {
            Node::Text(text) => {
                let t = text.text.trim();
                if !t.is_empty() {
                    parts.push(t.to_string());
                }
            }
            Node::Element(_) => {
                if let Some(child_el) = ElementRef::wrap(child_node) {
                    if is_material_symbols_element(&child_el) {
                        continue;
                    }
                    let child_text = clean_element_text_recursive(&child_el);
                    if !child_text.is_empty() {
                        parts.push(child_text);
                    }
                }
            }
            _ => {}
        }
    }
    parts.join(" ")
}

/// Extract text from a `DomNode` while skipping children that have a
/// material-symbols class. Returns cleaned, whitespace-collapsed text.
pub fn clean_node_text(node: &DomNode) -> String {
    // If this node itself is a material-symbols icon, return empty
    if node.classes.iter().any(|c| {
        MATERIAL_SYMBOL_CLASSES.iter().any(|ms| c.contains(ms))
    }) {
        return String::new();
    }

    // full_text already contains all descendant text (collected by
    // clean_element_text which skips material-symbols spans). Using it
    // directly avoids missing text nodes that are siblings of child
    // elements (e.g. "Develop.<br/>Preview.<br/>Ship." where the text
    // nodes after <br/> are not captured by node.text).
    let ft = node.full_text.trim().to_string();
    if !ft.is_empty() {
        return collapse_whitespace(&ft);
    }

    // Fallback: recurse into children (handles nodes where full_text
    // was not populated).
    let mut parts: Vec<String> = Vec::new();
    for child in &node.children {
        if child.classes.iter().any(|c| {
            MATERIAL_SYMBOL_CLASSES.iter().any(|ms| c.contains(ms))
        }) {
            continue;
        }
        let child_text = clean_node_text(child);
        if !child_text.is_empty() {
            parts.push(child_text);
        }
    }

    let joined = parts.join(" ");
    collapse_whitespace(&joined)
}

/// Collapse multiple whitespace characters into a single space.
fn collapse_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------------------
// Wrapper / layout class detection helpers
// ---------------------------------------------------------------------------

/// Tailwind classes that indicate a centering/sizing wrapper with no semantic
/// content of its own. When a `<div>` carries *only* these kinds of classes we
/// should recurse into its children instead of treating it as a block.
fn is_wrapper_class(class: &str) -> bool {
    // max-w-* (max-w-6xl, max-w-screen-xl, …)
    if class.starts_with("max-w-") { return true; }
    // mx-auto centering
    if class == "mx-auto" { return true; }
    // Pure padding/margin utilities (p-*, px-*, py-*, m-*, mt-*, …)
    if class.len() >= 2 {
        let bytes = class.as_bytes();
        if (bytes[0] == b'p' || bytes[0] == b'm')
            && (bytes[1] == b'-' || bytes[1] == b'x' || bytes[1] == b'y'
                || bytes[1] == b't' || bytes[1] == b'b'
                || bytes[1] == b'l' || bytes[1] == b'r')
        {
            return true;
        }
    }
    // w-full, min-h-screen, etc. – sizing-only
    if class.starts_with("w-") || class.starts_with("min-h-") || class.starts_with("h-") {
        return true;
    }
    // container utility
    if class == "container" { return true; }
    // space-y-*, gap-*, – spacing only
    if class.starts_with("space-") || class.starts_with("gap-") { return true; }
    false
}

/// True when a `<div>` carries a Tailwind grid class (`grid grid-cols-*`).
fn is_grid_container(classes: &[String]) -> bool {
    let has_grid = classes.iter().any(|c| c == "grid");
    let has_cols = classes.iter().any(|c| c.starts_with("grid-cols-"));
    has_grid && has_cols
}

/// True when a `<div>` carries a Tailwind flex class and has layout intent.
fn is_flex_container(classes: &[String]) -> bool {
    classes.iter().any(|c| c == "flex")
}

/// True when *all* classes on a node are wrapper/layout-only.
fn is_pure_wrapper(classes: &[String]) -> bool {
    if classes.is_empty() { return true; } // no classes at all → wrapper
    classes.iter().all(|c| is_wrapper_class(c))
}

/// Semantic HTML tags that should always be kept as a single block.
const SEMANTIC_BLOCK_TAGS: &[&str] = &[
    "section", "article", "aside", "table", "nav", "header", "footer", "form",
];

/// Check if a block should be kept together (not split into children).
///
/// Returns true for page-header-like blocks (heading + paragraph, small simple
/// groups) so they are not broken apart by `extract_deep_blocks`.
fn should_keep_together(node: &DomNode) -> bool {
    // Check if any direct child (or grandchild) is a heading
    let has_heading = node.children.iter().any(|c| {
        matches!(c.tag.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
            || c.children.iter().any(|gc| {
                matches!(gc.tag.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
            })
    });
    // Check if any direct child (or grandchild) is a paragraph
    let has_paragraph = node.children.iter().any(|c| {
        c.tag == "p"
            || c.children.iter().any(|gc| gc.tag == "p")
    });
    // Check if any child is a complex container (grid, card, table, section)
    let has_complex = node.children.iter().any(|c| {
        has_class(c, "grid")
            || has_class(c, "ghost-border")
            || has_class(c, "rounded-xl")
            || c.tag == "table"
            || c.tag == "section"
    });

    // Header block: has heading + paragraph but no complex children
    if has_heading && has_paragraph && !has_complex {
        return true;
    }
    // Small block with 3 or fewer children, all simple (no grids/cards)
    if node.children.len() <= 3 && !has_complex {
        return true;
    }
    false
}

/// Recursively extract meaningful blocks from a DOM subtree.
///
/// The logic peels away layout wrappers (centering divs, grid/flex containers)
/// and returns the inner semantic pieces so the detector can classify each one
/// independently.
pub fn extract_deep_blocks(node: &DomNode) -> Vec<DomNode> {
    // 1. Semantic elements → return as-is, they are self-contained blocks.
    if SEMANTIC_BLOCK_TAGS.contains(&node.tag.as_str()) {
        return vec![node.clone()];
    }

    // Only recurse into divs – other tags are kept as-is.
    if node.tag != "div" {
        return vec![node.clone()];
    }

    // 1b. Keep cohesive blocks together (page headers, small simple groups).
    if should_keep_together(node) {
        return vec![node.clone()];
    }

    // 2. Grid container → keep as ONE block when children look homogeneous
    //    (KPI grids, stat cards, feature grids).  Only explode when children
    //    are clearly heterogeneous (mixed section types inside a grid wrapper).
    if is_grid_container(&node.classes) && node.children.len() > 1 {
        // Keep the grid together — the template system will preserve its
        // layout (grid-cols-*) and the detect stage will classify the whole
        // block as kpi-grid / stat-cards / features.
        return vec![node.clone()];
    }

    // 3. Flex container with multiple children → extract each child.
    if is_flex_container(&node.classes) && node.children.len() > 1 {
        let mut out = Vec::new();
        for child in &node.children {
            out.extend(extract_deep_blocks(child));
        }
        return out;
    }

    // 4. Pure wrapper div (max-w-*, mx-auto, only padding/margin) → recurse.
    if is_pure_wrapper(&node.classes) {
        if node.children.is_empty() {
            return vec![node.clone()];
        }
        let mut out = Vec::new();
        for child in &node.children {
            out.extend(extract_deep_blocks(child));
        }
        return out;
    }

    // 5. Single-child div with no real semantic meaning → unwrap.
    if node.children.len() == 1 {
        return extract_deep_blocks(&node.children[0]);
    }

    // 6. Default: keep the node as a block.
    vec![node.clone()]
}

// ---------------------------------------------------------------------------
// Structured text extraction
// ---------------------------------------------------------------------------

/// Extract (tag, text) pairs from a node tree, preserving the relationship
/// between headings, paragraphs, labels, code blocks, etc.
///
/// Walks the tree depth-first and emits an entry every time it hits a
/// "leaf-level" text-bearing element.
pub fn extract_structured_text(node: &DomNode) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    extract_structured_text_inner(node, &mut pairs);
    pairs
}

/// Tags that carry text we want to surface individually.
const TEXT_BEARING_TAGS: &[&str] = &[
    "h1", "h2", "h3", "h4", "h5", "h6",
    "p", "span", "label", "code", "pre", "a",
    "li", "td", "th", "dt", "dd", "blockquote",
    "figcaption", "caption", "legend", "summary",
    "strong", "em", "b", "i", "small", "time",
];

fn extract_structured_text_inner(node: &DomNode, out: &mut Vec<(String, String)>) {
    // Skip material-symbols icons entirely
    if node.classes.iter().any(|c| {
        MATERIAL_SYMBOL_CLASSES.iter().any(|ms| c.contains(ms))
    }) {
        return;
    }

    // If this is a text-bearing tag, emit it and don't recurse further to
    // avoid duplicating content that already appears in parent text.
    if TEXT_BEARING_TAGS.contains(&node.tag.as_str()) {
        let text = clean_node_text(node);
        if !text.is_empty() {
            out.push((node.tag.clone(), text));
            return;
        }
    }

    // Otherwise recurse into children (divs, sections, etc.)
    for child in &node.children {
        extract_structured_text_inner(child, out);
    }
}

// ---------------------------------------------------------------------------
// Form field extraction
// ---------------------------------------------------------------------------

/// A form field descriptor: (field_type, label_text, placeholder_or_value).
///
/// Examples:
/// - `("text", "Public Key", "pk_live_...")`
/// - `("select", "Currency", "USD, BRL, EUR")`
/// - `("button", "", "Save Changes")`
pub fn extract_form_fields(node: &DomNode) -> Vec<(String, String, String)> {
    let mut fields = Vec::new();
    extract_form_fields_inner(node, &mut fields);
    fields
}

fn extract_form_fields_inner(node: &DomNode, out: &mut Vec<(String, String, String)>) {
    match node.tag.as_str() {
        "input" => {
            let field_type = node.attrs.get("type")
                .cloned()
                .unwrap_or_else(|| "text".to_string());
            // Skip hidden inputs
            if field_type == "hidden" {
                return;
            }
            let placeholder = node.attrs.get("placeholder")
                .or(node.attrs.get("value"))
                .cloned()
                .unwrap_or_default();
            // Try to find label from attrs or leave empty (caller can match
            // using sibling label logic below at the block level)
            let label = node.attrs.get("aria-label")
                .cloned()
                .unwrap_or_default();
            out.push((field_type, label, placeholder));
        }
        "textarea" => {
            let placeholder = node.attrs.get("placeholder")
                .cloned()
                .unwrap_or_default();
            let label = node.attrs.get("aria-label")
                .cloned()
                .unwrap_or_default();
            out.push(("textarea".to_string(), label, placeholder));
        }
        "select" => {
            // Collect option texts
            let options: Vec<String> = node.children.iter()
                .filter(|c| c.tag == "option")
                .map(|c| clean_node_text(c))
                .filter(|t| !t.is_empty())
                .collect();
            let label = node.attrs.get("aria-label")
                .cloned()
                .unwrap_or_default();
            out.push(("select".to_string(), label, options.join(", ")));
        }
        "button" => {
            let text = clean_node_text(node);
            let btn_type = node.attrs.get("type")
                .cloned()
                .unwrap_or_else(|| "button".to_string());
            out.push((format!("button[{}]", btn_type), String::new(), text));
        }
        "label" => {
            // When we hit a <label>, look for a child input/select/textarea
            // and pair them together.
            let label_text = {
                // Collect only direct text, not child input text
                let mut parts: Vec<String> = Vec::new();
                for child in &node.children {
                    if matches!(child.tag.as_str(), "input" | "select" | "textarea") {
                        continue;
                    }
                    let t = clean_node_text(child);
                    if !t.is_empty() {
                        parts.push(t);
                    }
                }
                if parts.is_empty() {
                    // Fallback: use node's own direct text
                    node.text.trim().to_string()
                } else {
                    parts.join(" ")
                }
            };

            // Find the paired input inside this label
            let mut found_input = false;
            for child in &node.children {
                match child.tag.as_str() {
                    "input" => {
                        let field_type = child.attrs.get("type")
                            .cloned()
                            .unwrap_or_else(|| "text".to_string());
                        if field_type == "hidden" { continue; }
                        let placeholder = child.attrs.get("placeholder")
                            .or(child.attrs.get("value"))
                            .cloned()
                            .unwrap_or_default();
                        out.push((field_type, label_text.clone(), placeholder));
                        found_input = true;
                    }
                    "select" => {
                        let options: Vec<String> = child.children.iter()
                            .filter(|c| c.tag == "option")
                            .map(|c| clean_node_text(c))
                            .filter(|t| !t.is_empty())
                            .collect();
                        out.push(("select".to_string(), label_text.clone(), options.join(", ")));
                        found_input = true;
                    }
                    "textarea" => {
                        let placeholder = child.attrs.get("placeholder")
                            .cloned()
                            .unwrap_or_default();
                        out.push(("textarea".to_string(), label_text.clone(), placeholder));
                        found_input = true;
                    }
                    _ => {}
                }
            }
            // If label had no child input, still record the label text
            if !found_input && !label_text.is_empty() {
                out.push(("label".to_string(), label_text, String::new()));
            }
            return; // Don't recurse further, we already handled children
        }
        _ => {}
    }

    // Recurse into children for non-terminal tags
    for child in &node.children {
        extract_form_fields_inner(child, out);
    }
}

// ---------------------------------------------------------------------------
// Top-level HTML parser
// ---------------------------------------------------------------------------

/// Parse HTML string into a flat list of semantic top-level blocks.
///
/// Avoids duplicates: sections inside `<main>` are collected as main's
/// direct children instead of being found again at the top level.
///
/// Uses `extract_deep_blocks` on `<main>` content to peel away layout
/// wrappers and expose the real semantic structure.
pub fn parse_html(html: &str) -> Vec<DomNode> {
    let document = Html::parse_document(html);
    let mut blocks = Vec::new();

    // 1) Collect top-level nav, header, aside (top stuff, before main content)
    let top_selectors = [
        "body > nav", "body > header", "body > aside",
    ];
    for sel_str in &top_selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            for el in document.select(&sel) {
                blocks.push(element_to_node(el, 0));
            }
        }
    }

    // 2) For <main>, deeply extract blocks by peeling away layout wrappers.
    //    We iterate main's direct element children and run extract_deep_blocks
    //    on each one so that wrapper divs get unwrapped automatically.
    if let Ok(main_sel) = Selector::parse("main") {
        for main_el in document.select(&main_sel) {
            let mut found_children = false;
            for child in main_el.children().filter_map(|c| ElementRef::wrap(c)) {
                let child_node = element_to_node(child, 0);
                blocks.extend(extract_deep_blocks(&child_node));
                found_children = true;
            }
            // If main has no element children, add main itself
            if !found_children {
                blocks.push(element_to_node(main_el, 0));
            }
        }
    }

    // 3) Collect top-level sections/articles that are NOT inside <main>
    for sel_str in &["body > section", "body > article"] {
        if let Ok(sel) = Selector::parse(sel_str) {
            for el in document.select(&sel) {
                blocks.push(element_to_node(el, 0));
            }
        }
    }

    // 4) Collect footer LAST so it always appears at the end
    if let Ok(sel) = Selector::parse("body > footer") {
        for el in document.select(&sel) {
            blocks.push(element_to_node(el, 0));
        }
    }

    // If no semantic tags found, try to extract from body > div structure
    if blocks.is_empty() {
        if let Ok(sel) = Selector::parse("body > *") {
            for el in document.select(&sel) {
                blocks.push(element_to_node(el, 0));
            }
        }
    }

    blocks
}

/// Extract classes from `<html>` and `<body>` elements.
pub fn detect_body_classes(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let mut classes = Vec::new();

    for tag in &["html", "body"] {
        if let Ok(sel) = Selector::parse(tag) {
            if let Some(el) = document.select(&sel).next() {
                classes.extend(el.value().classes().map(|c| c.to_string()));
            }
        }
    }

    classes
}

/// Convert a scraper ElementRef into our DomNode recursively
fn element_to_node(el: ElementRef, depth: u32) -> DomNode {
    let tag = el.value().name().to_string();
    let classes: Vec<String> = el.value().classes().map(|c| c.to_string()).collect();
    let id = el.value().id().map(|s| s.to_string());

    // Check if this element itself is a material-symbols icon
    let is_icon = classes.iter().any(|c| {
        MATERIAL_SYMBOL_CLASSES.iter().any(|ms| c.contains(ms))
    });

    // Direct text: first text node only, cleaned of material-symbols children
    let text: String = if is_icon {
        // For icon spans, keep the raw icon name as text (used by extract_material_icon)
        el.text().next().map(|s| s.trim().to_string()).unwrap_or_default()
    } else {
        // Clean: skip material-symbols children, only direct text nodes
        let mut direct_parts: Vec<String> = Vec::new();
        for child_node in el.children() {
            match child_node.value() {
                Node::Text(t) => {
                    let trimmed = t.text.trim();
                    if !trimmed.is_empty() {
                        direct_parts.push(trimmed.to_string());
                        break; // only first direct text, matching old behavior
                    }
                }
                _ => {}
            }
        }
        direct_parts.join("").trim().to_string()
    };

    // Full text: all descendants but skip material-symbols spans
    let full_text: String = if is_icon {
        el.text().collect::<Vec<_>>().join(" ").trim().to_string()
    } else {
        clean_element_text(&el)
    };

    // Attributes
    let mut attrs = HashMap::new();
    for attr in el.value().attrs() {
        attrs.insert(attr.0.to_string(), attr.1.to_string());
    }

    // Children — include both element children AND text nodes so that
    // dom_to_html() can reconstruct the original HTML faithfully.
    let children: Vec<DomNode> = el.children()
        .filter_map(|child| {
            if let Some(child_el) = ElementRef::wrap(child) {
                Some(element_to_node(child_el, depth + 1))
            } else if let Node::Text(t) = child.value() {
                let text_content = t.text.to_string();
                if text_content.trim().is_empty() {
                    return None; // skip whitespace-only text nodes
                }
                Some(DomNode {
                    tag: "_text".to_string(),
                    classes: Vec::new(),
                    id: None,
                    text: text_content.clone(),
                    full_text: text_content,
                    attrs: HashMap::new(),
                    children: Vec::new(),
                    depth: depth + 1,
                })
            } else {
                None
            }
        })
        .collect();

    DomNode { tag, classes, id, text, full_text, attrs, children, depth }
}

/// Detect theme from DOM analysis (light or dark).
///
/// Checks html/body classes first via the raw HTML, then falls back to
/// semantic block analysis. Also looks for Tailwind config in `<script>`.
pub fn detect_theme_with_html(html: &str, nodes: &[DomNode]) -> &'static str {
    // 1) Check <html> and <body> classes first — most reliable signal
    let body_classes = detect_body_classes(html);
    let body_str = body_classes.join(" ");

    if body_str.contains("dark") {
        return "dark";
    }
    if body_str.contains("light") {
        return "light";
    }
    // Body-level Tailwind background utilities
    if body_str.contains("bg-black") || body_str.contains("bg-gray-900") || body_str.contains("bg-neutral-950") {
        return "dark";
    }
    if body_str.contains("bg-white") || body_str.contains("bg-background") || body_str.contains("bg-gray-50") {
        return "light";
    }

    // 2) Check Tailwind config in <script> for background color definitions
    let document = Html::parse_document(html);
    if let Ok(sel) = Selector::parse("script") {
        for el in document.select(&sel) {
            let script_text: String = el.text().collect();
            // Light background definitions
            if script_text.contains("\"background\"") || script_text.contains("'background'") {
                if script_text.contains("#f9f9f9")
                    || script_text.contains("#fff")
                    || script_text.contains("#ffffff")
                    || script_text.contains("#fafafa")
                {
                    return "light";
                }
                if script_text.contains("#000")
                    || script_text.contains("#0a0a0a")
                    || script_text.contains("#09090b")
                    || script_text.contains("#111")
                {
                    return "dark";
                }
            }
        }
    }

    // 3) Fall back to semantic block analysis
    detect_theme(nodes)
}

/// Detect theme from semantic block classes only (legacy entry point).
pub fn detect_theme(nodes: &[DomNode]) -> &'static str {
    for node in nodes {
        let all_classes = collect_all_classes(node);
        let class_str = all_classes.join(" ");

        if class_str.contains("dark") { return "dark"; }

        // Check background colors in classes
        if class_str.contains("bg-black") || class_str.contains("bg-gray-900") || class_str.contains("bg-neutral-950") {
            return "dark";
        }
        if class_str.contains("bg-white") || class_str.contains("bg-background") || class_str.contains("bg-gray-50") {
            return "light";
        }

        // Check inline styles
        if let Some(style) = node.attrs.get("style") {
            if style.contains("background:#000") || style.contains("background:black") || style.contains("background: #000") {
                return "dark";
            }
        }
    }
    "light"
}

/// Extract page title from HTML.
///
/// Priority: `<title>` tag -> brand text from first `<nav>` -> first `<h1>`.
/// Returns "App" only as last resort.
pub fn extract_title(html: &str) -> String {
    let doc = Html::parse_document(html);

    // 1) Try <title> tag
    if let Ok(sel) = Selector::parse("title") {
        if let Some(el) = doc.select(&sel).next() {
            let title = el.text().collect::<String>().trim().to_string();
            if !title.is_empty() {
                return title;
            }
        }
    }

    // 2) Try brand text from the first <nav> — look for bold/brand element first
    if let Ok(sel) = Selector::parse("nav") {
        if let Some(nav) = doc.select(&sel).next() {
            // Look for brand: elements with font-bold, font-semibold, or tracking-tighter class
            // These are typically the brand name, not nav links
            // SKIP elements that ARE material-symbols icons
            let brand_selectors = [
                "[class*=\"font-bold\"]",
                "[class*=\"tracking-tighter\"]",
                "[class*=\"font-semibold\"]",
            ];
            for brand_sel_str in &brand_selectors {
                if let Ok(brand_sel) = Selector::parse(brand_sel_str) {
                    for el in nav.select(&brand_sel) {
                        // Skip material-symbols icon spans
                        if is_material_symbols_element(&el) {
                            continue;
                        }
                        let text = clean_element_text(&el);
                        if !text.is_empty() && text.len() < 20 {
                            return text;
                        }
                    }
                }
            }
            // Fallback: first <a> in nav
            if let Ok(a_sel) = Selector::parse("a") {
                if let Some(brand) = nav.select(&a_sel).next() {
                    let text = clean_element_text(&brand);
                    if !text.is_empty() {
                        return text;
                    }
                }
            }
            // Or first span/div with text
            for child in nav.children().filter_map(|c| ElementRef::wrap(c)) {
                if is_material_symbols_element(&child) {
                    continue;
                }
                let text = clean_element_text(&child);
                if !text.is_empty() {
                    return text;
                }
            }
        }
    }

    // 3) Try first <h1>
    if let Ok(sel) = Selector::parse("h1") {
        if let Some(el) = doc.select(&sel).next() {
            let text = el.text().collect::<String>().trim().to_string();
            if !text.is_empty() {
                return text;
            }
        }
    }

    "App".to_string()
}

/// Collect all classes from a node and its descendants
fn collect_all_classes(node: &DomNode) -> Vec<String> {
    let mut classes = node.classes.clone();
    for child in &node.children {
        classes.extend(collect_all_classes(child));
    }
    classes
}

/// Find all elements matching a tag within a node tree
pub fn find_by_tag<'a>(node: &'a DomNode, tag: &str) -> Vec<&'a DomNode> {
    let mut results = Vec::new();
    if node.tag == tag {
        results.push(node);
    }
    for child in &node.children {
        results.extend(find_by_tag(child, tag));
    }
    results
}

/// Find all elements with a specific class
pub fn find_by_class<'a>(node: &'a DomNode, class: &str) -> Vec<&'a DomNode> {
    let mut results = Vec::new();
    if node.classes.iter().any(|c| c.contains(class)) {
        results.push(node);
    }
    for child in &node.children {
        results.extend(find_by_class(child, class));
    }
    results
}

/// Extract all <a> links from a node, with material-symbols text filtered out.
pub fn extract_links(node: &DomNode) -> Vec<(String, String)> {
    let mut links = Vec::new();
    if node.tag == "a" {
        let href = node.attrs.get("href").cloned().unwrap_or_default();
        let text = clean_node_text(node);
        if !text.is_empty() {
            links.push((text, href));
        }
    }
    for child in &node.children {
        links.extend(extract_links(child));
    }
    links
}

/// Extract all images from a node
pub fn extract_images(node: &DomNode) -> Vec<(String, String)> {
    let mut images = Vec::new();
    if node.tag == "img" {
        let alt = node.attrs.get("alt").or(node.attrs.get("data-alt")).cloned().unwrap_or_default();
        let src = node.attrs.get("src").cloned().unwrap_or_default();
        images.push((alt, src));
    }
    for child in &node.children {
        images.extend(extract_images(child));
    }
    images
}

/// Extract all button texts from a node, with material-symbols text filtered out.
pub fn extract_buttons(node: &DomNode) -> Vec<String> {
    let mut buttons = Vec::new();
    if node.tag == "button" || (node.tag == "a" && node.classes.iter().any(|c| c.contains("btn") || c.contains("rounded-full"))) {
        let text = clean_node_text(node);
        if !text.is_empty() {
            buttons.push(text);
        }
    }
    for child in &node.children {
        buttons.extend(extract_buttons(child));
    }
    buttons
}

/// Find the first heading (h1-h6) text in a node tree, cleaned of icon text.
pub fn find_heading(node: &DomNode) -> Option<String> {
    let heading_tags = ["h1", "h2", "h3", "h4", "h5", "h6"];
    if heading_tags.contains(&node.tag.as_str()) {
        let text = clean_node_text(node);
        if !text.is_empty() {
            return Some(text);
        }
    }
    for child in &node.children {
        if let Some(h) = find_heading(child) {
            return Some(h);
        }
    }
    None
}

/// Find first <p> text in a node tree, cleaned of icon text.
pub fn find_paragraph(node: &DomNode) -> Option<String> {
    if node.tag == "p" {
        let text = clean_node_text(node);
        if !text.is_empty() {
            return Some(text);
        }
    }
    for child in &node.children {
        if let Some(p) = find_paragraph(child) {
            return Some(p);
        }
    }
    None
}

/// Check if node has Tailwind class matching a pattern
pub fn has_class(node: &DomNode, pattern: &str) -> bool {
    node.classes.iter().any(|c| c.contains(pattern))
}

/// Get Tailwind font size in approximate px
pub fn tw_font_size(classes: &[String]) -> Option<f32> {
    for c in classes {
        match c.as_str() {
            "text-xs" => return Some(12.0),
            "text-sm" => return Some(14.0),
            "text-base" => return Some(16.0),
            "text-lg" => return Some(18.0),
            "text-xl" => return Some(20.0),
            "text-2xl" => return Some(24.0),
            "text-3xl" => return Some(30.0),
            "text-4xl" => return Some(36.0),
            "text-5xl" => return Some(48.0),
            "text-6xl" => return Some(60.0),
            "text-7xl" => return Some(72.0),
            "text-8xl" => return Some(96.0),
            _ => {}
        }
    }
    None
}
