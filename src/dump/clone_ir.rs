// CRONUS Clone IR — Deterministic HTML-to-.cronus converter
//
// Unlike the heuristic dump system (detect → classify → emit),
// Clone IR reads the DOM tree structurally and emits exact .cronus
// declarations preserving every value, class, and layout property.
//
// Based on the full-scanner.ts logic from cronus-nexus.

use super::dom::{self, DomNode};
use std::collections::HashMap;

/// Main entry: takes raw HTML string, returns .cronus source
/// Uses the DOM parser blocks (pre-extracted by dom::parse_html).
pub fn clone_html_to_cronus(html: &str) -> String {
    let nodes = dom::parse_html(html);
    let mut out = String::new();

    let theme = detect_theme(html);
    let accent = detect_accent(html);
    let app_name = extract_title(html);
    let font = detect_font(html);

    out.push_str("# Cloned by CRONUS Clone IR\n\n");
    out.push_str(&format!("app \"{}\" {{\n", app_name));
    out.push_str("  stack react + tailwind\n");
    out.push_str("  port 5555\n");
    out.push_str("  database sqlite \"./data.db\"\n");
    out.push_str(&format!("  theme {}\n", theme));
    out.push_str("}\n\n");

    // Style block
    out.push_str("style {\n");
    out.push_str(&format!("  theme {}\n", theme));
    out.push_str(&format!("  accent {}\n", accent));
    out.push_str(&format!("  font \"{}\"\n", font));
    out.push_str("}\n\n");

    out.push_str("page \"/\" type:custom {\n  layout sidebar\n\n");

    // The DOM parser returns pre-extracted blocks (not a full tree).
    // Blocks are: aside, header, divs from inside <main>, footer, etc.
    let mut has_sidebar = false;
    let mut has_topbar = false;

    for node in &nodes {
        let tag = node.tag.to_lowercase();
        match tag.as_str() {
            "aside" => {
                emit_sidebar(node, &mut out);
                has_sidebar = true;
            }
            "header" => {
                emit_topbar(node, &mut out);
                has_topbar = true;
            }
            _ => {
                // For section/div blocks that contain multiple semantic children,
                // decompose into sub-blocks and classify each child
                let children_to_check = if (tag == "section" || tag == "div") && node.children.len() > 1 {
                    // Iterate children instead of the parent
                    node.children.iter().collect::<Vec<_>>()
                } else {
                    vec![node]
                };

                for sub in &children_to_check {
                    if is_page_header_div(sub) {
                        emit_page_header(sub, &mut out);
                    } else if is_kpi_grid(sub) {
                        emit_kpi_grid(sub, &mut out);
                    } else if is_chart_container(sub) {
                        emit_chart(sub, &mut out);
                    } else if is_table_container(sub) {
                        emit_table(sub, &mut out);
                    } else if sub.tag == "footer" && !has_class(sub, "md:hidden") {
                        emit_footer_section(sub, &mut out);
                    }
                }
            }
        }
    }

    out.push_str("}\n");
    out
}

// ─── Sidebar ───────────────────────────────────────────────────────

fn emit_sidebar(node: &DomNode, out: &mut String) {
    out.push_str("  section sidebar style:dark {\n");

    // Brand: look for h1/h2 in first div
    for child in &node.children {
        if child.tag == "div" {
            for sub in &child.children {
                if sub.tag == "h1" || sub.tag == "h2" {
                    let brand = clean_text(&sub.full_text);
                    out.push_str(&format!("    brand \"{}\"\n", brand));
                }
                if sub.tag == "p" {
                    let subtitle = clean_text(&sub.full_text);
                    if !subtitle.is_empty() {
                        out.push_str(&format!("    subtitle \"{}\"\n", subtitle));
                    }
                }
            }
        }
    }

    // Nav items: look for <nav> with <a> children
    if let Some(nav) = find_node_by_tag_in(node, "nav") {
        for link in &nav.children {
            if link.tag == "a" {
                let href = link.attrs.get("href").map(|s| s.as_str()).unwrap_or("#");
                let icon = find_material_icon(link).unwrap_or_default();
                let label = clean_text_no_icon(&link.full_text, &icon);
                let is_active = has_class(link, "bg-") && !has_class(link, "hover:");

                if !label.is_empty() {
                    let mut line = format!("    item \"{}\" -> \"{}\"", label, href);
                    if !icon.is_empty() {
                        line.push_str(&format!(" icon:{}", icon));
                    }
                    if is_active {
                        line.push_str(" active");
                    }
                    out.push_str(&line);
                    out.push('\n');
                }
            }
        }
    }

    // Bottom section: mt-auto div with buttons and links
    for child in &node.children {
        if child.tag == "div" && (has_class(child, "mt-auto") || child.classes.iter().any(|c| c.contains("auto"))) {
            // CTA button
            for sub in &child.children {
                if sub.tag == "button" {
                    let text = clean_text(&sub.full_text);
                    if !text.is_empty() {
                        out.push_str(&format!("    action \"{}\" -> \"#\" style:gradient\n", text));
                    }
                }
                if sub.tag == "a" {
                    let href = sub.attrs.get("href").map(|s| s.as_str()).unwrap_or("#");
                    let icon = find_material_icon(sub).unwrap_or_default();
                    let label = clean_text_no_icon(&sub.full_text, &icon);
                    if !label.is_empty() {
                        let mut line = format!("    item \"{}\" -> \"{}\"", label, href);
                        if !icon.is_empty() {
                            line.push_str(&format!(" icon:{}", icon));
                        }
                        line.push_str(" position:bottom");
                        out.push_str(&line);
                        out.push('\n');
                    }
                }
            }
        }
    }

    out.push_str("  }\n\n");
}

// ─── Main Content ──────────────────────────────────────────────────

fn emit_main_content(main: &DomNode, out: &mut String) {
    // Header (topbar)
    if let Some(header) = find_node_by_tag_in(main, "header") {
        emit_topbar(header, out);
    }

    // Sections inside <main>
    for child in &main.children {
        let tag = child.tag.to_lowercase();
        match tag.as_str() {
            "header" => {} // already handled
            "section" => emit_section_block(child, out),
            "footer" => emit_footer_section(child, out),
            _ => {}
        }
    }
}

fn emit_topbar(header: &DomNode, out: &mut String) {
    out.push_str("  section topbar style:dashboard {\n");

    // Brand text
    for child in all_descendants(header) {
        if (child.tag == "span" || child.tag == "h1") && has_any_class(child, &["text-xl", "text-2xl", "font-bold"]) {
            let text = clean_text(&child.full_text);
            if !text.is_empty() && text.len() < 30 {
                out.push_str(&format!("    brand \"{}\"\n", text));
                break;
            }
        }
    }

    // Nav links
    let mut nav_items: Vec<String> = Vec::new();
    if let Some(nav) = find_node_by_tag_in(header, "nav") {
        for link in &nav.children {
            if link.tag == "a" {
                let text = clean_text(&link.full_text);
                if !text.is_empty() {
                    nav_items.push(text);
                }
            }
        }
    }
    if !nav_items.is_empty() {
        out.push_str(&format!("    nav \"{}\"\n", nav_items.join(", ")));
    }

    // Action buttons (notifications, settings icons)
    for child in all_descendants(header) {
        if child.tag == "button" || child.tag == "span" {
            if let Some(icon) = find_material_icon(child) {
                if icon == "notifications" || icon == "settings" {
                    out.push_str(&format!("    action \"{}\" icon:{}\n",
                        capitalize(&icon), icon));
                }
            }
        }
    }

    // Avatar image
    for child in all_descendants(header) {
        if child.tag == "img" {
            if let Some(src) = child.attrs.get("src") {
                out.push_str(&format!("    image \"Avatar\" src:\"{}\"\n", src));
                break;
            }
        }
    }

    out.push_str("  }\n\n");
}

// ─── Section Block ─────────────────────────────────────────────────

fn emit_section_block(section: &DomNode, out: &mut String) {
    // Analyze children to determine section type
    let children = &section.children;

    for child in children {
        let tag = child.tag.to_lowercase();
        match tag.as_str() {
            "div" => {
                // Check what this div contains
                if is_page_header_div(child) {
                    emit_page_header(child, out);
                } else if is_kpi_grid(child) {
                    emit_kpi_grid(child, out);
                } else if is_chart_container(child) {
                    emit_chart(child, out);
                } else if is_table_container(child) {
                    emit_table(child, out);
                }
            }
            _ => {}
        }
    }
}

fn is_page_header_div(node: &DomNode) -> bool {
    // Has h2 with large text and optional eyebrow span
    node.children.iter().any(|c| c.tag == "h2" || c.tag == "h1") &&
    node.children.iter().any(|c| c.tag == "span" || c.tag == "p") &&
    !node.children.iter().any(|c| has_class(c, "grid"))
}

fn emit_page_header(node: &DomNode, out: &mut String) {
    out.push_str("  section page-header style:dark {\n");

    for child in &node.children {
        if child.tag == "span" && has_any_class(child, &["uppercase", "text-primary"]) {
            let text = clean_text(&child.full_text);
            if !text.is_empty() {
                out.push_str(&format!("    eyebrow \"{}\"\n", text));
            }
        }
        if child.tag == "h2" || child.tag == "h1" {
            let text = clean_text(&child.full_text);
            out.push_str(&format!("    title \"{}\"\n", text));
        }
        if child.tag == "p" {
            let text = clean_text(&child.full_text);
            if !text.is_empty() {
                out.push_str(&format!("    subtitle \"{}\"\n", text));
            }
        }
    }

    out.push_str("  }\n\n");
}

fn is_kpi_grid(node: &DomNode) -> bool {
    has_class(node, "grid") && node.children.iter().any(|c| {
        all_descendants(c).iter().any(|d| {
            has_any_class(d, &["text-6xl", "text-5xl", "text-4xl", "text-3xl"]) &&
            d.full_text.trim().len() > 0
        })
    })
}

fn emit_kpi_grid(node: &DomNode, out: &mut String) {
    // Detect grid columns
    let cols = if has_class(node, "grid-cols-4") || has_class(node, "md:grid-cols-4") { 4 }
    else if has_class(node, "grid-cols-3") || has_class(node, "md:grid-cols-3") { 3 }
    else { 4 };

    out.push_str(&format!("  section kpi cols:{} {{\n", cols));

    for child in &node.children {
        // Detect col-span
        let span = if has_class(child, "col-span-2") || has_class(child, "md:col-span-2") { 2 }
        else { 1 };

        // Find label (small uppercase text)
        let label = find_first_matching(child, |n| {
            (has_class(n, "uppercase") || has_class(n, "tracking-widest")) &&
            n.full_text.trim().len() > 2 && n.full_text.trim().len() < 50
        }).map(|n| clean_text(&n.full_text)).unwrap_or_default();

        // Find value (large number text)
        let value = find_first_matching(child, |n| {
            has_any_class(n, &["text-6xl", "text-5xl", "text-4xl", "text-3xl"]) &&
            n.full_text.trim().len() > 0
        }).map(|n| clean_text(&n.full_text)).unwrap_or_default();

        // Find badge (percentage pill)
        let badge = find_first_matching(child, |n| {
            let text = n.full_text.trim();
            (text.starts_with('+') || text.starts_with('-')) && text.contains('%')
        }).map(|n| clean_text(&n.full_text)).unwrap_or_default();

        // Find icon
        let icon = find_material_icon_in_tree(child).unwrap_or_default();

        // Find subtitle (small description text)
        // Allow % in subtitle if it's a longer phrase (not just a badge like "+12.4%")
        let subtitle = find_first_matching(child, |n| {
            has_any_class(n, &["text-xs", "text-sm"]) &&
            !has_class(n, "uppercase") && !has_class(n, "tracking-widest") &&
            n.full_text.trim().len() > 5 && n.full_text.trim().len() < 80 &&
            !(n.full_text.trim().starts_with('+') || n.full_text.trim().starts_with('-'))
        }).map(|n| clean_text(&n.full_text)).unwrap_or_default();

        if label.is_empty() && value.is_empty() { continue; }

        let mut line = format!("    item \"{}\"", label);
        if !icon.is_empty() { line.push_str(&format!(" icon:{}", icon)); }
        if span > 1 { line.push_str(&format!(" span:{}", span)); }
        out.push_str(&line);
        out.push_str(" {\n");

        if !value.is_empty() {
            out.push_str(&format!("      value \"{}\"\n", value));
        }
        if !badge.is_empty() {
            out.push_str(&format!("      badge \"{}\"\n", badge));
        }
        if !subtitle.is_empty() {
            out.push_str(&format!("      subtitle \"{}\"\n", subtitle));
        }

        out.push_str("    }\n");
    }

    out.push_str("  }\n\n");
}

fn is_chart_container(node: &DomNode) -> bool {
    // Chart: has gradient/clip-path, axis labels, period buttons
    let has_gradient = all_descendants(node).iter().any(|d|
        d.classes.iter().any(|c| c.contains("gradient")) ||
        d.attrs.get("style").map(|s| s.contains("clip-path")).unwrap_or(false)
    );
    let has_title = node.children.iter().any(|c| {
        find_node_by_tag_in(c, "h3").is_some() || find_node_by_tag_in(c, "h2").is_some()
    });
    has_gradient && has_title
}

fn emit_chart(node: &DomNode, out: &mut String) {
    out.push_str("  section chart style:dark {\n");

    // Title
    for desc in all_descendants(node) {
        if desc.tag == "h3" || desc.tag == "h2" {
            let text = clean_text(&desc.full_text);
            if !text.is_empty() {
                out.push_str(&format!("    title \"{}\"\n", text));
                break;
            }
        }
    }

    // Subtitle
    for desc in all_descendants(node) {
        if desc.tag == "p" && has_any_class(desc, &["text-xs", "text-sm"]) {
            let text = clean_text(&desc.full_text);
            if !text.is_empty() && text.len() < 60 {
                out.push_str(&format!("    subtitle \"{}\"\n", text));
                break;
            }
        }
    }

    out.push_str("    type area\n");

    // Period buttons
    let mut periods: Vec<String> = Vec::new();
    for desc in all_descendants(node) {
        if desc.tag == "button" {
            let text = clean_text(&desc.full_text);
            if !text.is_empty() && text.len() < 15 {
                periods.push(text);
            }
        }
    }
    if !periods.is_empty() {
        out.push_str(&format!("    periods \"{}\"\n", periods.join(", ")));
    }

    // Y-axis labels
    let mut y_labels: Vec<String> = Vec::new();
    for desc in all_descendants(node) {
        if desc.tag == "span" {
            let text = desc.full_text.trim().to_string();
            if text.ends_with('M') || text == "0.0" || text.ends_with('K') {
                y_labels.push(text);
            }
        }
    }
    if !y_labels.is_empty() {
        out.push_str(&format!("    y_axis \"{}\"\n", y_labels.join(", ")));
    }

    // X-axis labels
    let mut x_labels: Vec<String> = Vec::new();
    for desc in all_descendants(node) {
        if desc.tag == "span" {
            let text = desc.full_text.trim().to_string();
            // Date patterns: Oct 01, Jan 15, etc.
            if text.len() >= 4 && text.len() <= 8 &&
               text.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) &&
               text.contains(' ') {
                let parts: Vec<&str> = text.split_whitespace().collect();
                if parts.len() == 2 && parts[1].chars().all(|c| c.is_ascii_digit()) {
                    x_labels.push(text);
                }
            }
        }
    }
    if !x_labels.is_empty() {
        out.push_str(&format!("    x_axis \"{}\"\n", x_labels.join(", ")));
    }

    out.push_str("  }\n\n");
}

fn is_table_container(node: &DomNode) -> bool {
    find_node_by_tag_in(node, "table").is_some() ||
    find_node_by_tag_in(node, "thead").is_some()
}

fn emit_table(node: &DomNode, out: &mut String) {
    out.push_str("  section table style:dark {\n");

    // Title
    for desc in all_descendants(node) {
        if (desc.tag == "h3" || desc.tag == "h2") && !desc.full_text.trim().is_empty() {
            out.push_str(&format!("    title \"{}\"\n", clean_text(&desc.full_text)));
            break;
        }
    }

    // Search input
    for desc in all_descendants(node) {
        if desc.tag == "input" {
            if let Some(placeholder) = desc.attrs.get("placeholder") {
                out.push_str(&format!("    search \"{}\"\n", placeholder));
                break;
            }
        }
    }

    // Column headers from <thead>
    let table = find_node_by_tag_in(node, "table");
    if let Some(table) = table {
        if let Some(thead) = find_node_by_tag_in(table, "thead") {
            let mut columns: Vec<String> = Vec::new();
            for tr in &thead.children {
                if tr.tag == "tr" {
                    for th in &tr.children {
                        if th.tag == "th" {
                            let text = clean_text(&th.full_text);
                            if !text.is_empty() {
                                columns.push(text);
                            }
                        }
                    }
                }
            }
            if !columns.is_empty() {
                out.push_str(&format!("    columns \"{}\"\n", columns.join(", ")));
            }

            // Data rows from <tbody>
            if let Some(tbody) = find_node_by_tag_in(table, "tbody") {
                for tr in &tbody.children {
                    if tr.tag != "tr" { continue; }
                    let mut cells: Vec<String> = Vec::new();
                    for td in &tr.children {
                        if td.tag == "td" {
                            let text = clean_text(&td.full_text);
                            cells.push(text);
                        }
                    }
                    if cells.is_empty() { continue; }

                    // First cell is the row key
                    let key = cells.first().map(|s| s.as_str()).unwrap_or("");
                    out.push_str(&format!("    row \"{}\" {{", key));

                    // Emit remaining cells as key:value pairs
                    for (i, cell) in cells.iter().enumerate().skip(1) {
                        if i < columns.len() && !cell.is_empty() {
                            let col = columns[i].to_lowercase().replace(' ', "_");
                            out.push_str(&format!(" {}:\"{}\"", col, cell));
                        }
                    }
                    out.push_str(" }\n");
                }
            }
        }
    }

    // Footer link (View All)
    for desc in all_descendants(node) {
        if desc.tag == "button" || desc.tag == "a" {
            let text = clean_text(&desc.full_text);
            if text.to_lowercase().contains("view all") || text.to_lowercase().contains("ver todos") {
                out.push_str(&format!("    footer_link \"{}\"\n", text));
                break;
            }
        }
    }

    out.push_str("  }\n\n");
}

fn emit_footer_section(node: &DomNode, out: &mut String) {
    // Mobile footer — skip for desktop dashboards
    if has_class(node, "md:hidden") {
        return; // Don't emit mobile-only footer
    }
    // Generic footer
    out.push_str("  section footer style:dark {\n");
    for child in all_descendants(node) {
        if child.tag == "a" {
            let text = clean_text(&child.full_text);
            let icon = find_material_icon(child).unwrap_or_default();
            if !text.is_empty() {
                let mut line = format!("    item \"{}\" -> \"#\"", text);
                if !icon.is_empty() {
                    line.push_str(&format!(" icon:{}", icon));
                }
                out.push_str(&line);
                out.push('\n');
            }
        }
    }
    out.push_str("  }\n\n");
}

// ─── Helpers ───────────────────────────────────────────────────────

/// Collect all semantically meaningful nodes (aside, main, section, header, footer, nav, table)
/// from anywhere in the tree. This handles cases where the DOM parser nests things differently.
fn collect_semantic_nodes<'a>(root: &'a DomNode) -> Vec<&'a DomNode> {
    let semantic_tags = ["aside", "main", "footer", "header"];
    let mut result = Vec::new();
    for child in &root.children {
        let tag = child.tag.to_lowercase();
        if semantic_tags.contains(&tag.as_str()) {
            result.push(child);
        } else {
            // Recurse into divs that might wrap semantic content
            result.extend(collect_semantic_nodes(child));
        }
    }
    result
}

fn detect_theme(html: &str) -> &str {
    if html.contains("class=\"dark\"") || html.contains("bg-background") { "dark" } else { "light" }
}

fn detect_accent(html: &str) -> &str {
    if html.contains("#adc6ff") { "blue" }
    else if html.contains("#10b981") { "emerald" }
    else if html.contains("#f59e0b") { "amber" }
    else { "blue" }
}

fn extract_title(html: &str) -> String {
    if let Some(start) = html.find("<title>") {
        let after = &html[start + 7..];
        if let Some(end) = after.find("</title>") {
            let title = after[..end].trim();
            if !title.is_empty() { return title.to_string(); }
        }
    }
    // Fallback: look for first <h1> text
    if let Some(start) = html.find("<h1") {
        if let Some(tag_end) = html[start..].find('>') {
            let after = &html[start + tag_end + 1..];
            if let Some(end) = after.find("</h1>") {
                let text = after[..end].trim();
                // Strip inner tags
                let clean: String = text.chars().fold((String::new(), false), |(mut s, in_tag), c| {
                    if c == '<' { (s, true) }
                    else if c == '>' { (s, false) }
                    else if !in_tag { s.push(c); (s, false) }
                    else { (s, true) }
                }).0.trim().to_string();
                if !clean.is_empty() { return clean; }
            }
        }
    }
    "Dashboard".to_string()
}

fn detect_font(html: &str) -> &str {
    if html.contains("Inter") { "Inter" }
    else if html.contains("Space Grotesk") { "Space Grotesk" }
    else { "Inter" }
}

fn has_class(node: &DomNode, class: &str) -> bool {
    node.classes.iter().any(|c| c.contains(class))
}

fn has_any_class(node: &DomNode, classes: &[&str]) -> bool {
    classes.iter().any(|cls| has_class(node, cls))
}

fn find_node_by_tag<'a>(nodes: &'a [DomNode], tag: &str) -> Option<&'a DomNode> {
    for node in nodes {
        if node.tag.eq_ignore_ascii_case(tag) { return Some(node); }
        if let Some(found) = find_node_by_tag(&node.children, tag) { return Some(found); }
    }
    None
}

fn find_node_by_tag_in<'a>(parent: &'a DomNode, tag: &str) -> Option<&'a DomNode> {
    find_node_by_tag(&parent.children, tag)
}

fn find_material_icon(node: &DomNode) -> Option<String> {
    for child in &node.children {
        if child.tag == "span" && child.classes.iter().any(|c| c.contains("material")) {
            let text = child.full_text.trim().to_string();
            if !text.is_empty() && text.len() < 30 {
                return Some(text);
            }
        }
        if let Some(icon) = find_material_icon(child) { return Some(icon); }
    }
    None
}

fn find_material_icon_in_tree(node: &DomNode) -> Option<String> {
    for desc in all_descendants(node) {
        if desc.classes.iter().any(|c| c.contains("material")) {
            let text = desc.full_text.trim().to_string();
            if !text.is_empty() && text.len() < 30 {
                return Some(text);
            }
        }
    }
    None
}

fn all_descendants(node: &DomNode) -> Vec<&DomNode> {
    let mut result = Vec::new();
    for child in &node.children {
        result.push(child);
        result.extend(all_descendants(child));
    }
    result
}

fn find_first_matching<'a, F>(node: &'a DomNode, pred: F) -> Option<&'a DomNode>
where F: Fn(&DomNode) -> bool + Copy {
    for desc in all_descendants(node) {
        if pred(desc) { return Some(desc); }
    }
    None
}

fn clean_text(text: &str) -> String {
    text.trim()
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

fn clean_text_no_icon(text: &str, icon: &str) -> String {
    let cleaned = clean_text(text);
    if !icon.is_empty() {
        cleaned.replace(icon, "").trim().to_string()
    } else {
        cleaned
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}
