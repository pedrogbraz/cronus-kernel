#![allow(dead_code, unused_imports, unused_variables)]
//! Pattern detection rules for identifying HTML section types.
//!
//! Each scoring function checks multiple signals in a DomNode and returns
//! a confidence value between 0.0 and 1.0. Signals are additive (0.1-0.3 each).

use super::dom::{DomNode, find_by_tag, find_by_class, extract_links, extract_buttons, has_class, tw_font_size};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Count direct children with a specific tag.
pub fn count_children_with_tag(node: &DomNode, tag: &str) -> usize {
    node.children.iter().filter(|c| c.tag == tag).count()
}

/// Check if any descendant (recursive) has a class containing `class`.
pub fn has_descendant_class(node: &DomNode, class: &str) -> bool {
    if node.classes.iter().any(|c| c.contains(class)) {
        return true;
    }
    node.children.iter().any(|child| has_descendant_class(child, class))
}

/// Return the largest Tailwind font size (in px) among headings in the subtree.
pub fn max_heading_size(node: &DomNode) -> f32 {
    let heading_tags = ["h1", "h2", "h3", "h4", "h5", "h6"];
    let mut max_size: f32 = 0.0;

    if heading_tags.contains(&node.tag.as_str()) {
        if let Some(sz) = tw_font_size(&node.classes) {
            max_size = max_size.max(sz);
        }
    }

    for child in &node.children {
        max_size = max_size.max(max_heading_size(child));
    }
    max_size
}

/// Count all descendants matching a tag (recursive).
fn count_descendants_with_tag(node: &DomNode, tag: &str) -> usize {
    let mut count = 0;
    for child in &node.children {
        if child.tag == tag {
            count += 1;
        }
        count += count_descendants_with_tag(child, tag);
    }
    count
}

/// Check if text contains a price-like pattern ($XX, R$XX, etc.).
fn has_price_pattern(text: &str) -> bool {
    let patterns = ["$", "R$", "/mo", "/month", "/yr", "/year", "/mes", "/ano"];
    patterns.iter().any(|p| text.contains(p))
}

/// Check if text looks like a number (stat-like).
fn looks_like_stat(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    // Patterns: "99%", "10K+", "1.2M", "500+", "24/7", pure digits
    let first_char = trimmed.chars().next().unwrap();
    first_char.is_ascii_digit()
        || trimmed.ends_with('%')
        || trimmed.ends_with('+')
        || trimmed.ends_with('K')
        || trimmed.ends_with('M')
}

/// Cap a value at 1.0.
fn cap(v: f32) -> f32 {
    v.min(1.0)
}

// ---------------------------------------------------------------------------
// Scoring functions
// ---------------------------------------------------------------------------

/// Detect topbar / navigation bar.
///
/// Signals: tag is nav/header, fixed/sticky class, backdrop class,
/// has brand text + links + button, height ~64px.
pub fn is_topbar(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    // Tag signal
    if node.tag == "nav" {
        score += 0.3;
    } else if node.tag == "header" {
        score += 0.2;
    }

    // Position classes
    if has_class(node, "fixed") || has_class(node, "sticky") {
        score += 0.2;
    }

    // Backdrop / blur (common in modern navbars)
    if has_descendant_class(node, "backdrop") || has_class(node, "backdrop-blur") {
        score += 0.1;
    }

    // Contains links (navigation)
    let links = extract_links(node);
    if links.len() >= 2 {
        score += 0.15;
    }

    // Contains a button (CTA in topbar)
    let buttons = extract_buttons(node);
    if !buttons.is_empty() {
        score += 0.1;
    }

    // Has brand-like text (short text in first child)
    if !node.children.is_empty() {
        let first = &node.children[0];
        let brand_text = first.full_text.trim().to_string();
        if !brand_text.is_empty() && brand_text.len() < 30 {
            score += 0.1;
        }
    }

    // Height hint: h-16, h-14, py-4 etc. (common topbar heights ~64px)
    if has_class(node, "h-16") || has_class(node, "h-14") || has_class(node, "h-[64px]") {
        score += 0.1;
    }

    // Top-level position hint
    if has_class(node, "top-0") {
        score += 0.1;
    }

    cap(score)
}

/// Detect hero section.
///
/// Signals: contains h1 with large text (text-4xl+), CTA buttons,
/// subtitle paragraph, large section padding.
pub fn is_hero(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    // Has h1
    let h1s = find_by_tag(node, "h1");
    if !h1s.is_empty() {
        score += 0.25;

        // h1 with large font
        for h1 in &h1s {
            if let Some(sz) = tw_font_size(&h1.classes) {
                if sz >= 36.0 {
                    score += 0.2;
                }
            }
        }
    }

    // Large heading detected via recursive scan
    let max_hs = max_heading_size(node);
    if max_hs >= 36.0 {
        score += 0.1;
    }

    // CTA buttons — a hero MUST have CTA buttons (rounded-full style)
    let buttons = extract_buttons(node);
    let has_rounded_full_buttons = has_descendant_class(node, "rounded-full")
        && !buttons.is_empty();
    if has_rounded_full_buttons {
        score += 0.15;
    } else if !buttons.is_empty() {
        score += 0.05;
    }
    if buttons.len() >= 2 {
        score += 0.1;
    }

    // Penalize if no CTA buttons at all — h1+subtitle without buttons is page-header, not hero
    if buttons.is_empty() {
        score -= 0.2;
    }

    // Subtitle paragraph
    let paragraphs = find_by_tag(node, "p");
    if !paragraphs.is_empty() {
        score += 0.1;
    }

    // Large padding (hero sections typically have generous vertical padding)
    if has_class(node, "py-20") || has_class(node, "py-24") || has_class(node, "py-32")
        || has_class(node, "pt-20") || has_class(node, "pt-24") || has_class(node, "pt-32")
        || has_class(node, "min-h-screen") || has_class(node, "h-screen")
    {
        score += 0.15;
    }

    // Centered text (common in heroes)
    if has_descendant_class(node, "text-center") || has_class(node, "text-center") {
        score += 0.05;
    }

    cap(score)
}

/// Detect features / card grid section.
///
/// Signals: has grid class, 3+ card-like children each with heading + paragraph,
/// rounded cards, h3 headings, material icons.
pub fn is_features(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    // Grid layout with explicit columns (strong signal)
    if has_descendant_class(node, "grid-cols") {
        score += 0.25;
    } else if has_descendant_class(node, "grid") {
        score += 0.1;
    }

    // Count card-like children: div children that have both a heading and a paragraph
    let card_count = count_card_children(node);
    if card_count >= 3 {
        score += 0.3;
    } else if card_count >= 2 {
        score += 0.15;
    }

    // Multiple h3 headings (typical feature card pattern)
    let h3_count = count_descendants_with_tag(node, "h3");
    if h3_count >= 3 {
        score += 0.2;
    } else if h3_count >= 2 {
        score += 0.1;
    }

    // Has section heading (h2)
    let h2s = find_by_tag(node, "h2");
    if !h2s.is_empty() {
        score += 0.1;
    }

    // Cards have icons (svg, img, or material-symbols-outlined)
    if has_descendant_class(node, "icon")
        || has_descendant_class(node, "material-symbols")
        || count_descendants_with_tag(node, "svg") >= 3
    {
        score += 0.15;
    }

    // Rounded cards (rounded-xl, rounded-2xl, rounded-lg on children)
    if has_descendant_class(node, "rounded-xl") || has_descendant_class(node, "rounded-2xl") {
        score += 0.1;
    }

    // No h1 and no large heading (text-4xl+) — differentiates from hero
    let h1s = find_by_tag(node, "h1");
    if h1s.is_empty() {
        score += 0.1;
    }
    if max_heading_size(node) < 36.0 {
        score += 0.05;
    }

    // Gap classes (grid gap)
    if has_descendant_class(node, "gap-") {
        score += 0.05;
    }

    // Penalize if it looks like a hero (has buttons + h1 = probably hero, not features)
    if !h1s.is_empty() && !extract_buttons(node).is_empty() {
        score -= 0.2;
    }

    cap(score.max(0.0))
}

/// Detect stats / metrics section.
///
/// Signals: grid 3-4 cols, children are small cards with number + label.
pub fn is_stats(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    // Grid layout
    if has_descendant_class(node, "grid-cols-3") || has_descendant_class(node, "grid-cols-4") {
        score += 0.2;
    } else if has_descendant_class(node, "grid-cols") {
        score += 0.1;
    }

    // Children with stat-like text (numbers)
    let mut stat_children = 0;
    for child in &node.children {
        if looks_like_stat(&child.full_text) || child_has_stat_text(child) {
            stat_children += 1;
        }
    }
    // Also check one level deeper (grid may be nested)
    for child in &node.children {
        for grandchild in &child.children {
            if looks_like_stat(&grandchild.full_text) || child_has_stat_text(grandchild) {
                stat_children += 1;
            }
        }
    }
    if stat_children >= 3 {
        score += 0.3;
    } else if stat_children >= 2 {
        score += 0.15;
    }

    // Short section (stats are typically compact)
    let total_text_len = node.full_text.len();
    if total_text_len < 500 {
        score += 0.1;
    }

    // Has heading (section label)
    if find_by_tag(node, "h2").is_empty() && find_by_tag(node, "h3").is_empty() {
        // Stats sections sometimes have no heading — slight penalty
    } else {
        score += 0.05;
    }

    // Flex or grid with small items
    if has_descendant_class(node, "flex") {
        score += 0.05;
    }

    cap(score)
}

/// Detect CTA (call-to-action) section.
///
/// Signals: centered text, h2, 1-2 buttons, short section.
pub fn is_cta(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    // Has h2 (CTA headline)
    let h2s = find_by_tag(node, "h2");
    if !h2s.is_empty() {
        score += 0.2;
    }

    // Has buttons
    let buttons = extract_buttons(node);
    if buttons.len() == 1 || buttons.len() == 2 {
        score += 0.25;
    }

    // Centered text
    if has_class(node, "text-center") || has_descendant_class(node, "text-center") {
        score += 0.15;
    }

    // Short section (CTA is concise)
    let text_len = node.full_text.len();
    if text_len < 300 {
        score += 0.15;
    }

    // No h1 (distinguishes from hero)
    if find_by_tag(node, "h1").is_empty() {
        score += 0.1;
    }

    // No grid (distinguishes from features/stats)
    if !has_descendant_class(node, "grid-cols") {
        score += 0.1;
    }

    // Padding signals
    if has_class(node, "py-16") || has_class(node, "py-20") || has_class(node, "py-24") {
        score += 0.05;
    }

    cap(score)
}

/// Detect footer section.
///
/// Signals: tag is footer, has links, has copyright text, border-top.
pub fn is_footer(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    // Tag signal
    if node.tag == "footer" {
        score += 0.4;
    }

    // Has links
    let links = extract_links(node);
    if links.len() >= 3 {
        score += 0.2;
    } else if !links.is_empty() {
        score += 0.1;
    }

    // Copyright text
    let lower = node.full_text.to_lowercase();
    if lower.contains("copyright") || lower.contains("\u{00a9}") || lower.contains("all rights reserved") {
        score += 0.2;
    }

    // Border-top class
    if has_class(node, "border-t") || has_class(node, "border-top") {
        score += 0.1;
    }

    // Typical footer classes
    if has_class(node, "footer") || has_descendant_class(node, "footer") {
        score += 0.1;
    }

    // Small text
    if has_descendant_class(node, "text-sm") || has_descendant_class(node, "text-xs") {
        score += 0.05;
    }

    cap(score)
}

/// Detect terminal / code block section.
///
/// Signals: font-mono/monospace class, dark background, traffic-light dots,
/// code-like content.
pub fn is_terminal(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    // Monospace font
    if has_descendant_class(node, "font-mono") || has_descendant_class(node, "monospace") {
        score += 0.25;
    }

    // Dark background
    if has_descendant_class(node, "bg-black")
        || has_descendant_class(node, "bg-gray-900")
        || has_descendant_class(node, "bg-neutral-900")
        || has_descendant_class(node, "bg-zinc-900")
        || has_descendant_class(node, "bg-slate-900")
    {
        score += 0.2;
    }

    // Has <pre> or <code> tags
    if count_descendants_with_tag(node, "pre") > 0 || count_descendants_with_tag(node, "code") > 0 {
        score += 0.2;
    }

    // Traffic-light dots (red/yellow/green circles in terminal header)
    if has_descendant_class(node, "bg-red-") || has_descendant_class(node, "rounded-full") {
        let has_green = has_descendant_class(node, "bg-green-");
        let has_yellow = has_descendant_class(node, "bg-yellow-");
        if has_green || has_yellow {
            score += 0.2;
        }
    }

    // Code-like content: $ prefix, arrows, common CLI patterns
    let text = &node.full_text;
    if text.contains("$ ") || text.contains(">>> ") || text.contains("~/") {
        score += 0.15;
    }

    // Rounded container (terminal window look)
    if has_descendant_class(node, "rounded-lg") || has_descendant_class(node, "rounded-xl") {
        score += 0.05;
    }

    cap(score)
}

/// Detect table / data table section.
///
/// Signals: has <table> tag, or structured rows with consistent column count.
pub fn is_table(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    // Direct <table> tag
    if node.tag == "table" {
        score += 0.5;
    }

    // Descendant <table>
    if count_descendants_with_tag(node, "table") > 0 {
        score += 0.4;
    }

    // Has thead / tbody
    if count_descendants_with_tag(node, "thead") > 0 {
        score += 0.15;
    }
    if count_descendants_with_tag(node, "tbody") > 0 {
        score += 0.1;
    }

    // Has <tr> rows
    let row_count = count_descendants_with_tag(node, "tr");
    if row_count >= 3 {
        score += 0.2;
    } else if row_count >= 1 {
        score += 0.1;
    }

    // Has <th> or <td>
    if count_descendants_with_tag(node, "th") > 0 || count_descendants_with_tag(node, "td") > 0 {
        score += 0.1;
    }

    // CSS table classes (for div-based tables)
    if has_descendant_class(node, "table") && node.tag != "table" {
        score += 0.15;
    }

    cap(score)
}

/// Detect pricing section.
///
/// Signals: plan cards with price patterns ($XX), feature lists, badge/tier labels.
pub fn is_pricing(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    // Price patterns in text
    let text = &node.full_text;
    if has_price_pattern(text) {
        score += 0.3;
    }

    // Multiple price mentions (multiple plans)
    let price_mentions = text.matches('$').count() + text.matches("R$").count();
    if price_mentions >= 2 {
        score += 0.15;
    }

    // Grid layout (plan columns)
    if has_descendant_class(node, "grid-cols") {
        score += 0.1;
    }

    // Feature lists (ul/li inside cards)
    let list_count = count_descendants_with_tag(node, "ul");
    if list_count >= 2 {
        score += 0.15;
    }

    // Has heading (section label like "Pricing")
    let h2s = find_by_tag(node, "h2");
    for h2 in &h2s {
        let lower = h2.full_text.to_lowercase();
        if lower.contains("pricing") || lower.contains("plans") || lower.contains("preco")
            || lower.contains("plano")
        {
            score += 0.15;
        }
    }

    // Buttons per card (subscribe/buy CTA)
    let buttons = extract_buttons(node);
    if buttons.len() >= 2 {
        score += 0.1;
    }

    // Badge-like classes (popular, recommended)
    if has_descendant_class(node, "badge") || has_descendant_class(node, "popular") {
        score += 0.05;
    }

    cap(score)
}

/// Detect bento grid section.
///
/// Signals: grid with mixed col-span items, cards of different sizes.
pub fn is_bento(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    // Grid layout
    if has_descendant_class(node, "grid") {
        score += 0.15;
    }

    // Mixed col-span classes (key bento indicator)
    let has_col_span_1 = has_descendant_class(node, "col-span-1");
    let has_col_span_2 = has_descendant_class(node, "col-span-2");
    let has_col_span_3 = has_descendant_class(node, "col-span-3");
    let has_row_span = has_descendant_class(node, "row-span-");

    let span_variety = [has_col_span_1, has_col_span_2, has_col_span_3, has_row_span]
        .iter()
        .filter(|&&v| v)
        .count();

    if span_variety >= 2 {
        score += 0.35;
    } else if has_col_span_2 || has_col_span_3 {
        score += 0.2;
    }

    // Grid with 3+ columns
    if has_descendant_class(node, "grid-cols-3") || has_descendant_class(node, "grid-cols-4") {
        score += 0.1;
    }

    // Multiple card-like children
    let card_children = node.children.iter()
        .filter(|c| c.tag == "div" && !c.classes.is_empty())
        .count();
    if card_children >= 4 {
        score += 0.15;
    }

    // Cards have varied content (not uniform like features)
    if has_descendant_class(node, "rounded") {
        score += 0.05;
    }

    // Gap class
    if has_descendant_class(node, "gap-") {
        score += 0.05;
    }

    cap(score)
}

// ---------------------------------------------------------------------------
// Internal helpers for scoring
// ---------------------------------------------------------------------------

/// Check if a child node has stat-like text (number in heading or bold).
fn child_has_stat_text(node: &DomNode) -> bool {
    for child in &node.children {
        let heading_tags = ["h1", "h2", "h3", "h4", "h5", "h6", "strong", "b", "span"];
        if heading_tags.contains(&child.tag.as_str()) && looks_like_stat(child.full_text.trim()) {
            return true;
        }
        if child_has_stat_text(child) {
            return true;
        }
    }
    false
}

/// Count children that look like feature cards (have a heading + paragraph).
fn count_card_children(node: &DomNode) -> usize {
    // Check direct children first
    let direct = node.children.iter().filter(|c| is_card_like(c)).count();
    if direct >= 2 {
        return direct;
    }

    // Check one level deeper (grid wrapper pattern)
    for child in &node.children {
        let nested = child.children.iter().filter(|c| is_card_like(c)).count();
        if nested >= 2 {
            return nested;
        }
    }

    direct
}

/// Check if a node looks like a card (has heading + paragraph or description,
/// or is a rounded div with heading content).
fn is_card_like(node: &DomNode) -> bool {
    let heading_tags = ["h2", "h3", "h4", "h5", "h6"];
    let has_heading = node.children.iter().any(|c| heading_tags.contains(&c.tag.as_str()))
        || find_by_tag(node, "h3").len() + find_by_tag(node, "h4").len() > 0;
    let has_paragraph = find_by_tag(node, "p").len() > 0;

    // Classic card: heading + paragraph
    if has_heading && has_paragraph {
        return true;
    }

    // Rounded card with heading (some cards don't have a <p>, just <h3> + loose text)
    let is_rounded = has_class(node, "rounded-xl")
        || has_class(node, "rounded-2xl")
        || has_class(node, "rounded-lg");
    if is_rounded && has_heading {
        return true;
    }

    // Card with padding and heading
    let has_padding = node.classes.iter().any(|c| c.starts_with("p-") || c.starts_with("px-") || c.starts_with("py-"));
    if has_padding && has_heading {
        return true;
    }

    false
}

// ---------------------------------------------------------------------------
// Dashboard-specific scoring functions
// ---------------------------------------------------------------------------

/// Detect sidebar / side navigation.
///
/// Signals: aside tag (0.3), h-screen class (0.2), fixed class (0.2),
/// has 3+ links (0.2), has icons (0.1).
pub fn is_sidebar(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    // Tag signal
    if node.tag == "aside" {
        score += 0.3;
    }

    // Full-height layout
    if has_class(node, "h-screen") {
        score += 0.2;
    }

    // Fixed positioning
    if has_class(node, "fixed") {
        score += 0.2;
    }

    // Navigation links
    let links = extract_links(node);
    if links.len() >= 3 {
        score += 0.2;
    }

    // Has icons (material symbols or svg)
    if has_descendant_class(node, "material-symbols")
        || count_descendants_with_tag(node, "svg") >= 2
    {
        score += 0.1;
    }

    cap(score)
}

/// Detect page header (title + action button row).
///
/// Signals: has h1 (0.3), has button sibling (0.2), flex justify-between (0.1),
/// text-4xl class on heading (0.1).
pub fn is_page_header(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    // Has h1 or h2 with text-4xl (dashboard pages often use h2 for page title)
    let h1s = find_by_tag(node, "h1");
    let h2s = find_by_tag(node, "h2");
    if !h1s.is_empty() {
        score += 0.3;
    } else {
        // Check h2 with large font (acts as page title in dashboards)
        for h2 in &h2s {
            if has_class(h2, "text-4xl") || has_class(h2, "text-3xl") {
                score += 0.3;
                break;
            }
        }
    }

    // Has subtitle paragraph
    let paragraphs = find_by_tag(node, "p");
    if !paragraphs.is_empty() {
        score += 0.1;
    }

    // Does NOT have rounded-full CTA buttons (distinguishes from hero)
    let buttons = extract_buttons(node);
    let has_rounded_full_btns = has_descendant_class(node, "rounded-full")
        && !buttons.is_empty();
    if !has_rounded_full_btns {
        score += 0.2;
    }

    // Has badge span (uppercase small text like "DEVELOPER")
    if has_descendant_class(node, "uppercase") && has_descendant_class(node, "tracking-wider") {
        score += 0.1;
    } else if has_descendant_class(node, "uppercase") && has_descendant_class(node, "tracking-widest") {
        score += 0.1;
    }

    // Flex justify-between layout (header pattern)
    if has_class(node, "justify-between") || has_descendant_class(node, "justify-between") {
        score += 0.05;
    }

    // Short section (page headers are compact, no grids/cards)
    let total_text_len = node.full_text.len();
    if total_text_len < 300 {
        score += 0.1;
    }

    // Penalize if it has grid or many cards (that's features/bento, not a header)
    if has_descendant_class(node, "grid-cols") {
        score -= 0.3;
    }

    cap(score.max(0.0))
}

/// Detect stat cards row (small metric cards in a grid).
///
/// Signals: grid with 3 small cards (0.3), each card has a number and label (0.3),
/// cards have "uppercase tracking-widest" labels (0.2).
pub fn is_stat_cards(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    // Grid layout with 3 columns
    if has_descendant_class(node, "grid-cols-3") || has_class(node, "grid-cols-3") {
        score += 0.3;
    } else if has_descendant_class(node, "grid-cols") || has_class(node, "grid") {
        score += 0.1;
    }

    // Count children that look like stat cards (number + label)
    let mut stat_card_count = 0;
    let children_to_check: Vec<&DomNode> = if node.children.len() >= 2 {
        node.children.iter().collect()
    } else {
        // Check one level deeper (wrapper div)
        node.children.iter().flat_map(|c| c.children.iter()).collect()
    };

    for child in &children_to_check {
        let has_number = child_has_stat_text(child) || {
            // Look for large bold text (text-2xl, text-3xl, font-bold)
            has_descendant_class(child, "text-2xl")
                || has_descendant_class(child, "text-3xl")
                || has_descendant_class(child, "font-bold")
        };
        let has_label = has_descendant_class(child, "uppercase")
            || has_descendant_class(child, "tracking-widest")
            || has_descendant_class(child, "text-xs")
            || has_descendant_class(child, "text-sm");
        if has_number && has_label {
            stat_card_count += 1;
        }
    }
    if stat_card_count >= 3 {
        score += 0.3;
    } else if stat_card_count >= 2 {
        score += 0.15;
    }

    // Uppercase tracking-widest labels (stat label pattern)
    if has_descendant_class(node, "uppercase") && has_descendant_class(node, "tracking-widest") {
        score += 0.2;
    }

    // Compact section (stat cards are short)
    if node.full_text.len() < 400 {
        score += 0.1;
    }

    // Penalize if has h1 (that's a page-header, not stat-cards)
    if !find_by_tag(node, "h1").is_empty() {
        score -= 0.3;
    }

    cap(score.max(0.0))
}

/// Detect product grid (cards with price, badges, icons).
///
/// Signals: grid of cards with price text ($XX.XX) (0.3),
/// badges (Active/Draft) (0.2), icon boxes (0.1).
pub fn is_product_grid(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    // Grid layout
    if has_descendant_class(node, "grid-cols") || has_class(node, "grid-cols-3")
        || has_class(node, "grid-cols-2")
    {
        score += 0.15;
    }

    // Price patterns in text ($XX.XX, /month, /mo)
    let text = &node.full_text;
    let price_count = text.matches('$').count();
    if price_count >= 2 {
        score += 0.3;
    } else if price_count >= 1 {
        score += 0.15;
    }

    // Status badges (Active, Draft, etc.)
    let lower = text.to_lowercase();
    let has_active = lower.contains("active");
    let has_draft = lower.contains("draft");
    if has_active || has_draft {
        score += 0.2;
    }

    // Icon boxes (material symbols in cards)
    if has_descendant_class(node, "material-symbols") {
        score += 0.1;
    }

    // Multiple card children with h3 headings (product names)
    let h3_count = count_descendants_with_tag(node, "h3");
    if h3_count >= 2 {
        score += 0.15;
    }

    // Penalize if it looks like stat-cards (uppercase tracking-widest + no prices)
    if has_descendant_class(node, "uppercase") && has_descendant_class(node, "tracking-widest")
        && price_count == 0
    {
        score -= 0.2;
    }

    // Penalize if has h1 (page-header)
    if !find_by_tag(node, "h1").is_empty() {
        score -= 0.2;
    }

    cap(score.max(0.0))
}

/// Detect team / member list.
///
/// Signals: has avatar images (0.2), has role text (Admin/Developer/Viewer) (0.3),
/// has email-like text (0.2).
pub fn is_team_list(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    let text = &node.full_text;
    let lower = text.to_lowercase();

    // Role text patterns
    let role_keywords = ["admin", "developer", "viewer", "editor", "owner", "member"];
    let mut role_hits = 0;
    for keyword in &role_keywords {
        if lower.contains(keyword) {
            role_hits += 1;
        }
    }
    if role_hits >= 2 {
        score += 0.3;
    } else if role_hits >= 1 {
        score += 0.15;
    }

    // Avatar images (rounded-full images or placeholders)
    let img_count = count_descendants_with_tag(node, "img");
    if img_count >= 2 {
        score += 0.2;
    }
    // Also check for rounded-full divs (avatar placeholders)
    if has_descendant_class(node, "rounded-full") {
        score += 0.05;
    }

    // Email-like text (contains @)
    let email_count = text.matches('@').count();
    if email_count >= 2 {
        score += 0.2;
    } else if email_count >= 1 {
        score += 0.1;
    }

    // Has heading (like "Team Members")
    if !find_by_tag(node, "h2").is_empty() {
        score += 0.1;
    }

    // Repeated row structure (flex items-center justify-between)
    if has_descendant_class(node, "justify-between") {
        score += 0.05;
    }

    // Penalize if has h1 (page-header) or grid-cols (product-grid)
    if !find_by_tag(node, "h1").is_empty() {
        score -= 0.2;
    }

    cap(score.max(0.0))
}

// ---------------------------------------------------------------------------
// Dashboard-specific: content card
// ---------------------------------------------------------------------------

/// Detect content card (API keys, forms, settings panels).
///
/// Signals: has h2/h3 but NOT h1 (0.2), has input/code elements (0.3),
/// rounded border container (0.2), does NOT have grid-cols (0.1).
pub fn is_content_card(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    // Has h2/h3 heading but NOT h1
    let h1s = find_by_tag(node, "h1");
    let h2s = find_by_tag(node, "h2");
    let h3s = find_by_tag(node, "h3");
    if h1s.is_empty() && (!h2s.is_empty() || !h3s.is_empty()) {
        score += 0.2;
    }

    // Has input, code, or font-mono elements (form-like content)
    if count_descendants_with_tag(node, "input") > 0
        || count_descendants_with_tag(node, "code") > 0
        || has_descendant_class(node, "font-mono")
    {
        score += 0.3;
    }

    // Rounded border container (card look)
    let is_rounded = has_class(node, "rounded-xl") || has_class(node, "rounded-lg")
        || has_class(node, "rounded-2xl");
    let has_border = has_class(node, "border") || has_class(node, "shadow");
    if is_rounded || has_border {
        score += 0.2;
    }

    // Does NOT have grid-cols (not a grid of cards)
    if !has_descendant_class(node, "grid-cols") {
        score += 0.1;
    }

    // Has label elements or uppercase tracking-widest text (form labels)
    if count_descendants_with_tag(node, "label") > 0
        || (has_descendant_class(node, "uppercase") && has_descendant_class(node, "tracking-widest"))
    {
        score += 0.1;
    }

    // Penalize if has h1 (that's a page-header)
    if !h1s.is_empty() {
        score -= 0.3;
    }

    cap(score.max(0.0))
}

// ---------------------------------------------------------------------------
// Dashboard-specific: info panel
// ---------------------------------------------------------------------------

/// Detect info/promo panel (dark promo cards, quick-links, status indicators).
///
/// Signals: dark bg (promo), link list with external icons (links),
/// status dot (status-card).
pub fn is_info_panel(node: &DomNode) -> f32 {
    let mut score: f32 = 0.0;

    // Dark background (promo panel)
    if has_class(node, "bg-primary") || has_class(node, "bg-black")
        || has_class(node, "bg-gray-900") || has_class(node, "bg-zinc-900")
    {
        score += 0.3;
    }

    // Link list with external icons (quick-links)
    let links = extract_links(node);
    if links.len() >= 3 {
        score += 0.2;
    }
    if has_descendant_class(node, "open_in_new") || count_descendants_with_tag(node, "ul") > 0 {
        score += 0.1;
    }

    // Status indicator (green dot)
    if has_descendant_class(node, "bg-green-500") || has_descendant_class(node, "bg-green-400") {
        let text = &node.full_text.to_lowercase();
        if text.contains("operational") || text.contains("status") || text.contains("health") {
            score += 0.3;
        }
    }

    // Has h4 heading (info panels use h4, not h1/h2)
    if !find_by_tag(node, "h4").is_empty() {
        score += 0.1;
    }

    // Rounded container
    if has_class(node, "rounded-xl") || has_class(node, "rounded-lg") {
        score += 0.1;
    }

    // Compact section
    if node.full_text.len() < 500 {
        score += 0.05;
    }

    // Penalize if has h1 or grid-cols (not an info panel)
    if !find_by_tag(node, "h1").is_empty() || has_descendant_class(node, "grid-cols") {
        score -= 0.3;
    }

    cap(score.max(0.0))
}

// ---------------------------------------------------------------------------
// Classifier
// ---------------------------------------------------------------------------

/// Run all detectors against a node and return the best-matching section type
/// name and its confidence score.
///
/// Returns `("generic", 0.0)` if no detector reaches a meaningful threshold.
pub fn classify_node(node: &DomNode) -> (&'static str, f32) {
    let detectors: Vec<(&'static str, fn(&DomNode) -> f32)> = vec![
        ("topbar",       is_topbar),
        ("hero",         is_hero),
        ("features",     is_features),
        ("stats",        is_stats),
        ("cta",          is_cta),
        ("footer",       is_footer),
        ("terminal",     is_terminal),
        ("table",        is_table),
        ("pricing",      is_pricing),
        ("bento",        is_bento),
        ("sidebar",      is_sidebar),
        ("page-header",  is_page_header),
        ("stat-cards",   is_stat_cards),
        ("product-grid", is_product_grid),
        ("team-list",    is_team_list),
        ("card",         is_content_card),
        ("info-panel",   is_info_panel),
    ];

    let mut best_name: &'static str = "generic";
    let mut best_score: f32 = 0.0;

    for (name, detector) in &detectors {
        let score = detector(node);
        if score > best_score {
            best_score = score;
            best_name = name;
        }
    }

    // Require a minimum confidence to avoid false positives
    if best_score < 0.25 {
        return ("generic", best_score);
    }

    (best_name, best_score)
}
