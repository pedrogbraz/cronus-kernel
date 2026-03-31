#![allow(dead_code, unused_imports, unused_variables)]
use std::collections::HashMap;
use super::detect::{SectionBlueprint, ItemBlueprint};

// ---------------------------------------------------------------------------
// CronusFile — top-level representation of a .cronus source file
// ---------------------------------------------------------------------------

pub struct CronusFile {
  pub app_name: String,
  pub port: u16,
  pub theme: String,
  pub style_config: HashMap<String, String>,
  pub sections: Vec<SectionBlueprint>,
}

// ---------------------------------------------------------------------------
// GroupedItem — items grouped under a parent card
// ---------------------------------------------------------------------------

/// Represents a titled item with its child items grouped beneath it.
struct GroupedItem<'a> {
  parent: &'a ItemBlueprint,
  children: Vec<&'a ItemBlueprint>,
}

/// Groups consecutive items where non-titled items belong to the preceding
/// titled item. An item is considered "titled" if it has a non-empty title
/// and its item_type is "item", "product", "member", or "stat".
fn group_items<'a>(items: &'a [ItemBlueprint]) -> Vec<GroupedItem<'a>> {
  let titled_types = ["item", "product", "member", "stat", "webhook"];
  let child_types = ["label", "code", "action", "chip", "status", "link"];

  let mut groups: Vec<GroupedItem<'a>> = Vec::new();

  for item in items {
    let is_titled = titled_types.contains(&item.item_type.as_str())
      && !item.title.is_empty();
    let is_child = child_types.contains(&item.item_type.as_str());

    if is_titled {
      groups.push(GroupedItem {
        parent: item,
        children: Vec::new(),
      });
    } else if is_child && !groups.is_empty() {
      // Attach to the last titled item
      groups.last_mut().unwrap().children.push(item);
    } else {
      // Standalone item (no parent to attach to, or non-child type)
      groups.push(GroupedItem {
        parent: item,
        children: Vec::new(),
      });
    }
  }

  groups
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns `level` repetitions of two-space indentation.
fn indent(level: usize) -> String {
  "  ".repeat(level)
}

/// Escapes double-quotes inside a string so it can be safely wrapped in `"…"`.
fn escape_cronus(s: &str) -> String {
  s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Structural config keys that belong on the section opening line.
const STRUCTURAL_KEYS: &[&str] = &["cols", "style", "span", "gap", "layout"];

/// Common Material icon names that may leak into CTA text.
const ICON_NAMES: &[&str] = &[
  "arrow_forward", "arrow_back", "arrow_upward", "arrow_downward",
  "add", "remove", "close", "check", "search", "menu", "home",
  "settings", "delete", "edit", "share", "send", "download",
  "upload", "refresh", "more_vert", "more_horiz", "chevron_right",
  "chevron_left", "expand_more", "expand_less", "open_in_new",
  "content_copy", "visibility", "favorite", "star", "info",
  "warning", "error", "help", "launch", "link", "east", "west",
  "north", "south", "trending_up", "trending_down", "bolt",
  "rocket_launch", "play_arrow", "pause", "stop",
];

/// Cleans text by trimming whitespace, removing trailing icon names,
/// and collapsing multiple spaces/newlines into a single space.
fn clean_text(s: &str) -> String {
  // Replace newlines and collapse whitespace
  let collapsed: String = s
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ");

  // Remove trailing icon names
  let mut result = collapsed.trim().to_string();
  for icon in ICON_NAMES {
    if result.ends_with(icon) {
      result = result[..result.len() - icon.len()].trim().to_string();
    }
  }
  // Also strip leading icon names
  for icon in ICON_NAMES {
    if result.starts_with(icon) {
      result = result[icon.len()..].trim().to_string();
    }
  }
  result
}

/// Formats only structural config keys as inline `key:value` pairs for the section opening line.
/// Keys are sorted for deterministic output.
fn emit_structural_config(config: &HashMap<String, String>) -> String {
  let mut keys: Vec<&String> = config
    .keys()
    .filter(|k| STRUCTURAL_KEYS.contains(&k.as_str()))
    .collect();
  if keys.is_empty() {
    return String::new();
  }
  keys.sort();
  keys
    .iter()
    .map(|k| format!("{}:{}", k, config[*k]))
    .collect::<Vec<_>>()
    .join(" ")
}

/// Formats a `HashMap<String, String>` as inline `key:value` pairs.
/// Keys are sorted for deterministic output. Excludes internal keys prefixed with `_`.
fn emit_config_pairs(config: &HashMap<String, String>) -> String {
  if config.is_empty() {
    return String::new();
  }
  let mut keys: Vec<&String> = config.keys()
    .filter(|k| !k.starts_with('_'))
    .collect();
  keys.sort();
  if keys.is_empty() {
    return String::new();
  }
  keys
    .iter()
    .map(|k| {
      let v = &config[*k];
      if v.contains(':') || v.contains(' ') || v.contains('/') || v.contains('"') {
        format!("{}:\"{}\"", k, escape_cronus(v))
      } else {
        format!("{}:{}", k, v)
      }
    })
    .collect::<Vec<_>>()
    .join(" ")
}

/// Wraps `text` in double-quotes after escaping.
fn quoted(s: &str) -> String {
  format!("\"{}\"", escape_cronus(s))
}

// ---------------------------------------------------------------------------
// Dashboard pattern detection helpers
// ---------------------------------------------------------------------------

/// Detect if items contain progress-bar-like entries (usage meters).
fn has_progress_items(items: &[ItemBlueprint]) -> bool {
  items.iter().any(|i| {
    i.item_type == "meter"
      || i.config.get("_type").map(|t| t == "meter").unwrap_or(false)
      || i.config.contains_key("progress")
      || i.config.contains_key("usage")
      || i.config.contains_key("limit")
  })
}

/// Detect if items look like stat values + labels (billing stats).
fn has_stat_items(items: &[ItemBlueprint]) -> bool {
  items.iter().any(|i| i.item_type == "stat")
}

/// Detect if items are card entries with prices (product grid).
fn has_price_items(items: &[ItemBlueprint]) -> bool {
  items.iter().any(|i| {
    i.item_type == "product"
      || i.config.contains_key("price")
  })
}

/// Detect if items are member-like (name + email + role).
fn has_member_items(items: &[ItemBlueprint]) -> bool {
  items.iter().any(|i| {
    i.item_type == "member"
      || i.config.contains_key("email")
      || i.config.contains_key("role")
  })
}

/// Detect if items are row-based with columns config (activity table).
fn has_table_rows(bp: &SectionBlueprint) -> bool {
  bp.config.contains_key("columns")
    && bp.items.iter().any(|i| i.item_type == "row")
}

/// Detect if items are form fields.
fn has_form_fields(items: &[ItemBlueprint]) -> bool {
  items.iter().filter(|i| {
    i.item_type == "field"
      || i.config.get("_type").map(|t| t == "field").unwrap_or(false)
  }).count() >= 2
}

/// Detect if section looks like a current-plan card (badge + title + rows).
fn is_plan_style(bp: &SectionBlueprint) -> bool {
  bp.config.contains_key("badge")
    && bp.title.is_some()
    && bp.items.iter().any(|i| i.item_type == "row")
}

/// Detect bento/features pattern: many items without a specific _type.
fn is_bento_style(items: &[ItemBlueprint]) -> bool {
  let generic_count = items.iter()
    .filter(|i| i.item_type == "item" && i.description.is_some())
    .count();
  generic_count >= 3
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Generates a complete `.cronus` source file from a `CronusFile`.
pub fn emit_cronus(file: &CronusFile) -> String {
  let mut out = String::with_capacity(4096);

  // Header comment
  out.push_str("# Generated by cronus dump\n");
  out.push('\n');

  // app block
  out.push_str(&format!("app {} {{\n", quoted(&file.app_name)));
  out.push_str("  stack react + tailwind\n");
  out.push_str(&format!("  port {}\n", file.port));
  out.push_str("  database sqlite \"./data.db\"\n");
  out.push_str(&format!("  theme {}\n", file.theme));
  out.push_str("}\n\n");

  // style block
  out.push_str("style {\n");
  out.push_str(&format!("  theme {}\n", file.theme));
  if let Some(accent) = file.style_config.get("accent") {
    out.push_str(&format!("  accent {}\n", accent));
  }
  if let Some(font) = file.style_config.get("font") {
    out.push_str(&format!("  font {}\n", quoted(font)));
  }
  // Emit remaining style keys (sorted, skip already-emitted ones)
  let skip = ["accent", "font", "theme"];
  let mut extra_keys: Vec<&String> = file
    .style_config
    .keys()
    .filter(|k| !skip.contains(&k.as_str()))
    .collect();
  extra_keys.sort();
  for k in extra_keys {
    out.push_str(&format!("  {} {}\n", k, quoted(&file.style_config[k])));
  }
  out.push_str("}\n\n");

  // page block wrapping all sections
  out.push_str("page \"/\" type:custom {\n");
  for bp in &file.sections {
    out.push_str(&emit_section(bp, 1));
    out.push('\n');
  }
  out.push_str("}\n");

  out
}

// ---------------------------------------------------------------------------
// Section emitter — dispatches by section_type with dashboard detection
// ---------------------------------------------------------------------------

fn emit_section(bp: &SectionBlueprint, ind: usize) -> String {
  let mut out = String::new();
  let prefix = indent(ind);

  let section_type = bp.section_type.to_lowercase();

  // Build the section opening line with structural config inlined
  let cfg = emit_structural_config(&bp.config);
  if cfg.is_empty() {
    out.push_str(&format!("{}section {} {{\n", prefix, section_type));
  } else {
    out.push_str(&format!("{}section {} {} {{\n", prefix, section_type, cfg));
  }

  let inner = ind + 1;

  // Route to the appropriate body emitter based on section type,
  // with fallback dashboard pattern detection for generic sections
  match section_type.as_str() {
    "topbar" => emit_topbar_body(bp, inner, &mut out),
    "hero" => emit_hero_body(bp, inner, &mut out),
    "features" | "faq" => emit_features_body(bp, inner, &mut out),
    "stats" | "stat-cards" | "billing-stats" => emit_stats_body(bp, inner, &mut out),
    "testimonial" => emit_testimonial_body(bp, inner, &mut out),
    "pricing" => emit_pricing_body(bp, inner, &mut out),
    "cta" => emit_cta_body(bp, inner, &mut out),
    "footer" => emit_footer_body(bp, inner, &mut out),
    "terminal" => emit_terminal_body(bp, inner, &mut out),
    "sidebar" => emit_sidebar_body(bp, inner, &mut out),
    "page-header" => emit_page_header_body(bp, inner, &mut out),
    "product-grid" => emit_product_grid_body(bp, inner, &mut out),
    "team-list" => emit_team_list_body(bp, inner, &mut out),
    "card" => emit_card_body(bp, inner, &mut out),
    "info-panel" | "status-card" | "promo" | "links" => emit_info_panel_body(bp, inner, &mut out),
    "form" => emit_form_body(bp, inner, &mut out),
    "tabs" => emit_tabs_body(bp, inner, &mut out),
    _ => emit_detected_body(bp, inner, &mut out),
  }

  out.push_str(&format!("{}}}\n", prefix));
  out
}

// ---------------------------------------------------------------------------
// Detected body — tries to detect dashboard patterns before falling back
// ---------------------------------------------------------------------------

fn emit_detected_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  // Try dashboard pattern detection on generic sections
  if is_plan_style(bp) {
    emit_plan_style_body(bp, ind, out);
  } else if has_table_rows(bp) {
    emit_activity_table_body(bp, ind, out);
  } else if has_form_fields(&bp.items) {
    emit_form_body(bp, ind, out);
  } else if has_progress_items(&bp.items) {
    emit_usage_status_body(bp, ind, out);
  } else if has_stat_items(&bp.items) {
    emit_stats_body(bp, ind, out);
  } else if has_price_items(&bp.items) {
    emit_product_grid_body(bp, ind, out);
  } else if has_member_items(&bp.items) {
    emit_team_list_body(bp, ind, out);
  } else if is_bento_style(&bp.items) {
    emit_bento_body(bp, ind, out);
  } else {
    emit_generic_body(bp, ind, out);
  }
}

// ---------------------------------------------------------------------------
// Section body emitters — original types
// ---------------------------------------------------------------------------

fn emit_topbar_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(brand) = bp.config.get("brand") {
    out.push_str(&format!("{}brand {}\n", pre, quoted(brand)));
  } else if let Some(ref t) = bp.title {
    out.push_str(&format!("{}brand {}\n", pre, quoted(t)));
  }

  if let Some(nav) = bp.config.get("nav") {
    out.push_str(&format!("{}nav {}\n", pre, quoted(nav)));
  }

  emit_cta_entries(bp, ind, out);

  for item in &bp.items {
    out.push_str(&emit_item(item, ind));
  }
}

fn emit_hero_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(badge) = bp.config.get("badge") {
    out.push_str(&format!("{}badge {}\n", pre, quoted(badge)));
  }

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }
  if let Some(ref subtitle) = bp.subtitle {
    if !subtitle.is_empty() {
      out.push_str(&format!("{}subtitle {}\n", pre, quoted(subtitle)));
    }
  }

  emit_cta_entries(bp, ind, out);

  for item in &bp.items {
    out.push_str(&emit_item(item, ind));
  }
}

fn emit_features_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  // Skip section-level title/subtitle if they duplicate the first item's title
  let first_item_title = bp.items.first().map(|i| i.title.as_str()).unwrap_or("");
  let title_duplicates_first = bp.title.as_deref().map(|t| t == first_item_title).unwrap_or(false);

  if !title_duplicates_first {
    if let Some(ref title) = bp.title {
      if !title.is_empty() {
        out.push_str(&format!("{}title {}\n", pre, quoted(title)));
      }
    }
    if let Some(ref subtitle) = bp.subtitle {
      if !subtitle.is_empty() {
        out.push_str(&format!("{}subtitle {}\n", pre, quoted(subtitle)));
      }
    }
  }

  // Use grouped items for features (bento-style grouping)
  let groups = group_items(&bp.items);
  for group in &groups {
    out.push_str(&emit_grouped_item(group, ind));
  }
}

fn emit_stats_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }
  if let Some(ref subtitle) = bp.subtitle {
    if !subtitle.is_empty() {
      out.push_str(&format!("{}subtitle {}\n", pre, quoted(subtitle)));
    }
  }

  for item in &bp.items {
    // Emit stat items with value: syntax
    if let Some(value) = item.config.get("value") {
      out.push_str(&format!("{}item {} value:{}\n", pre, quoted(&item.title), quoted(value)));
    } else {
      out.push_str(&emit_item(item, ind));
    }
  }
}

fn emit_testimonial_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }
  if let Some(ref subtitle) = bp.subtitle {
    if !subtitle.is_empty() {
      out.push_str(&format!("{}subtitle {}\n", pre, quoted(subtitle)));
    }
  }

  for item in &bp.items {
    out.push_str(&emit_item(item, ind));
  }
}

fn emit_pricing_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);
  let ipre = indent(ind + 1);

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }
  if let Some(ref subtitle) = bp.subtitle {
    if !subtitle.is_empty() {
      out.push_str(&format!("{}subtitle {}\n", pre, quoted(subtitle)));
    }
  }

  for item in &bp.items {
    if item.item_type == "plan" {
      let price = item.config.get("price").cloned().unwrap_or_default();
      let is_featured = item.config.get("featured").map(|v| v == "true").unwrap_or(false);
      let features_str = item.config.get("_features").cloned().unwrap_or_default();

      // Build plan line: plan "Name" $price [featured]
      let mut line = format!("{}plan {}", pre, quoted(&item.title));
      if !price.is_empty() {
        // Emit price as a Price token (e.g. $49/mês), not a quoted string
        let price_clean = price.replace("R$", "").replace("US$", "").replace("€", "").trim().to_string();
        let parts: Vec<&str> = price_clean.splitn(2, '/').collect();
        let numeric = parts[0].trim().replace(' ', "").replace('.', "").replace(',', ".");
        if numeric.parse::<f64>().is_ok() {
          let interval = parts.get(1).map(|s| s.trim()).unwrap_or(&"");
          if interval.is_empty() {
            line.push_str(&format!(" ${}", numeric));
          } else {
            line.push_str(&format!(" ${}/{}", numeric, interval));
          }
        } else {
          line.push_str(&format!(" {}", quoted(&price)));
        }
      }
      if is_featured {
        line.push_str(" featured");
      }

      if features_str.is_empty() {
        out.push_str(&format!("{}\n", line));
      } else {
        out.push_str(&format!("{} [\n", line));
        let features: Vec<&str> = features_str.split("||").collect();
        for (i, feature) in features.iter().enumerate() {
          if i < features.len() - 1 {
            out.push_str(&format!("{}{},\n", ipre, quoted(feature)));
          } else {
            out.push_str(&format!("{}{}\n", ipre, quoted(feature)));
          }
        }
        out.push_str(&format!("{}]\n", pre));
      }
    } else {
      out.push_str(&emit_item(item, ind));
    }
  }
}

fn emit_cta_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }
  if let Some(ref subtitle) = bp.subtitle {
    if !subtitle.is_empty() {
      out.push_str(&format!("{}subtitle {}\n", pre, quoted(subtitle)));
    }
  }

  emit_cta_entries(bp, ind, out);

  for item in &bp.items {
    out.push_str(&emit_item(item, ind));
  }
}

fn emit_footer_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(copyright) = bp.config.get("copyright") {
    out.push_str(&format!("{}copyright {}\n", pre, quoted(copyright)));
  }
  if let Some(nav) = bp.config.get("nav") {
    out.push_str(&format!("{}nav {}\n", pre, quoted(nav)));
  }

  for item in &bp.items {
    out.push_str(&emit_item(item, ind));
  }
}

// ---------------------------------------------------------------------------
// NEW section body emitters — dashboard types
// ---------------------------------------------------------------------------

fn emit_terminal_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }

  for item in &bp.items {
    out.push_str(&emit_item(item, ind));
  }
}

/// Emit sidebar as a component block with proper nav-link routing.
fn emit_sidebar_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  // Brand
  if let Some(brand) = bp.config.get("brand") {
    out.push_str(&format!("{}brand {}\n", pre, quoted(brand)));
  }

  // Subtitle (e.g., "Enterprise")
  if let Some(subtitle) = bp.config.get("subtitle") {
    out.push_str(&format!("{}subtitle {}\n", pre, quoted(subtitle)));
  } else if let Some(ref sub) = bp.subtitle {
    if !sub.is_empty() {
      out.push_str(&format!("{}subtitle {}\n", pre, quoted(sub)));
    }
  }

  // Nav links with arrow syntax: item "Label" -> "/path" icon:X active:true position:bottom
  for item in &bp.items {
    match item.item_type.as_str() {
      "nav-link" => {
        let href = item.config.get("href").map(|s| s.as_str()).unwrap_or("#");
        let icon = item.config.get("icon");
        let mut line = format!("{}item {} -> {}", pre, quoted(&item.title), quoted(href));
        if let Some(ic) = icon {
          line.push_str(&format!(" icon:{}", ic));
        }
        if item.config.get("active").map(|v| v == "true").unwrap_or(false) {
          line.push_str(" active:true");
        }
        if item.config.get("position").map(|v| v == "bottom").unwrap_or(false) {
          line.push_str(" position:bottom");
        }
        line.push('\n');
        out.push_str(&line);
      }
      "action" => {
        out.push_str(&emit_action_line(item, ind));
      }
      _ => {
        out.push_str(&emit_item(item, ind));
      }
    }
  }
}

/// Emit page-header with title, subtitle, and action buttons.
fn emit_page_header_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }
  if let Some(ref subtitle) = bp.subtitle {
    if !subtitle.is_empty() {
      out.push_str(&format!("{}subtitle {}\n", pre, quoted(subtitle)));
    }
  }

  // Actions with icon inline
  for item in &bp.items {
    out.push_str(&emit_item(item, ind));
  }
}

/// Emit product-grid with proper product blocks including price and status.
fn emit_product_grid_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }

  for item in &bp.items {
    match item.item_type.as_str() {
      "product" => emit_product_block(item, ind, out),
      _ => out.push_str(&emit_item(item, ind)),
    }
  }
}

/// Emit a single product as a block with price, status, description.
fn emit_product_block(item: &ItemBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);
  let ipre = indent(ind + 1);

  // Gather inline config (exclude inner-only keys)
  let inner_keys = ["price", "price_interval", "status", "description", "_type"];
  let mut inline_parts: Vec<String> = Vec::new();
  let mut keys: Vec<&String> = item.config.keys()
    .filter(|k| !inner_keys.contains(&k.as_str()))
    .collect();
  keys.sort();
  for k in &keys {
    inline_parts.push(format!("{}:{}", k, item.config[*k]));
  }
  let cfg_str = inline_parts.join(" ");

  let has_inner = item.description.is_some()
    || item.config.contains_key("price")
    || item.config.contains_key("status");

  if has_inner {
    if cfg_str.is_empty() {
      out.push_str(&format!("{}item {} {{\n", pre, quoted(&item.title)));
    } else {
      out.push_str(&format!("{}item {} {} {{\n", pre, quoted(&item.title), cfg_str));
    }

    if let Some(ref desc) = item.description {
      if !desc.is_empty() {
        out.push_str(&format!("{}{}\n", ipre, quoted(desc)));
      }
    }
    if let Some(price) = item.config.get("price") {
      let mut line = format!("{}price {}", ipre, quoted(price));
      if let Some(interval) = item.config.get("price_interval") {
        line.push_str(&format!(" interval:{}", quoted(interval)));
      }
      line.push('\n');
      out.push_str(&line);
    }
    if let Some(status) = item.config.get("status") {
      out.push_str(&format!("{}status {}\n", ipre, quoted(status)));
    }

    out.push_str(&format!("{}}}\n", pre));
  } else {
    if cfg_str.is_empty() {
      out.push_str(&format!("{}item {}\n", pre, quoted(&item.title)));
    } else {
      out.push_str(&format!("{}item {} {}\n", pre, quoted(&item.title), cfg_str));
    }
  }
}

/// Emit team-list with member blocks including email, role, avatar.
fn emit_team_list_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }

  for item in &bp.items {
    match item.item_type.as_str() {
      "member" => emit_member_block(item, ind, out),
      _ => out.push_str(&emit_item(item, ind)),
    }
  }
}

/// Emit a single member block with email, role, avatar.
fn emit_member_block(item: &ItemBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);
  let ipre = indent(ind + 1);

  let has_detail = item.config.contains_key("email")
    || item.config.contains_key("role")
    || item.config.contains_key("avatar_src");

  if has_detail {
    out.push_str(&format!("{}member {} {{\n", pre, quoted(&item.title)));

    if let Some(email) = item.config.get("email") {
      out.push_str(&format!("{}email {}\n", ipre, quoted(email)));
    }
    if let Some(role) = item.config.get("role") {
      out.push_str(&format!("{}role {}\n", ipre, quoted(role)));
    }
    if let Some(avatar) = item.config.get("avatar_src") {
      let alt = item.config.get("avatar_alt").map(|s| s.as_str()).unwrap_or("");
      out.push_str(&format!("{}avatar {} src:{}\n", ipre, quoted(alt), quoted(avatar)));
    }

    out.push_str(&format!("{}}}\n", pre));
  } else {
    out.push_str(&format!("{}member {}\n", pre, quoted(&item.title)));
  }
}

/// Emit card body with grouped items (label + code + action under titled parent).
fn emit_card_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }
  if let Some(ref subtitle) = bp.subtitle {
    if !subtitle.is_empty() {
      out.push_str(&format!("{}subtitle {}\n", pre, quoted(subtitle)));
    }
  }

  // Group child items under their parent card items
  let groups = group_items(&bp.items);
  for group in &groups {
    out.push_str(&emit_grouped_item(group, ind));
  }
}

/// Emit info-panel / status-card / promo / links body.
fn emit_info_panel_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  // Panel type annotation
  if let Some(panel_type) = bp.config.get("panel_type") {
    if panel_type != &bp.section_type {
      out.push_str(&format!("{}# panel_type: {}\n", pre, panel_type));
    }
  }

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }
  if let Some(ref subtitle) = bp.subtitle {
    if !subtitle.is_empty() {
      out.push_str(&format!("{}subtitle {}\n", pre, quoted(subtitle)));
    }
  }

  for item in &bp.items {
    match item.item_type.as_str() {
      "link" => {
        let href = item.config.get("href").map(|s| s.as_str()).unwrap_or("#");
        let icon = item.config.get("icon");
        let mut line = format!("{}item {} -> {}", pre, quoted(&item.title), quoted(href));
        if let Some(ic) = icon {
          line.push_str(&format!(" icon:{}", ic));
        }
        line.push('\n');
        out.push_str(&line);
      }
      "status" => {
        let status = item.config.get("status").map(|s| s.as_str()).unwrap_or("unknown");
        out.push_str(&format!("{}status {} state:{}\n", pre, quoted(&item.title), status));
      }
      _ => out.push_str(&emit_item(item, ind)),
    }
  }
}

// ---------------------------------------------------------------------------
// Dashboard pattern body emitters
// ---------------------------------------------------------------------------

/// Emit current-plan style: badge + title + rows.
fn emit_plan_style_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(badge) = bp.config.get("badge") {
    out.push_str(&format!("{}badge {}\n", pre, quoted(badge)));
  }

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }
  if let Some(ref subtitle) = bp.subtitle {
    if !subtitle.is_empty() {
      out.push_str(&format!("{}subtitle {}\n", pre, quoted(subtitle)));
    }
  }

  emit_cta_entries(bp, ind, out);

  for item in &bp.items {
    out.push_str(&emit_item(item, ind));
  }
}

/// Emit activity-table style: columns config + row items.
fn emit_activity_table_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }

  // Emit columns definition
  if let Some(columns) = bp.config.get("columns") {
    out.push_str(&format!("{}columns {}\n", pre, quoted(columns)));
  }

  for item in &bp.items {
    out.push_str(&emit_item(item, ind));
  }
}

/// Emit form section: field items with labels.
fn emit_form_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }
  if let Some(ref subtitle) = bp.subtitle {
    if !subtitle.is_empty() {
      out.push_str(&format!("{}subtitle {}\n", pre, quoted(subtitle)));
    }
  }

  // Group fields under parent items
  let groups = group_items(&bp.items);
  for group in &groups {
    out.push_str(&emit_grouped_item(group, ind));
  }
}

/// Emit tabs section: list of tab labels.
fn emit_tabs_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }

  for item in &bp.items {
    let active = item.config.get("active").map(|v| v == "true").unwrap_or(false);
    if active {
      out.push_str(&format!("{}item {} active:true\n", pre, quoted(&item.title)));
    } else {
      out.push_str(&format!("{}item {}\n", pre, quoted(&item.title)));
    }
  }
}

/// Emit usage-status style with progress/meter items.
fn emit_usage_status_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }

  for item in &bp.items {
    if item.item_type == "meter" || item.config.contains_key("progress") {
      emit_meter_item(item, ind, out);
    } else {
      out.push_str(&emit_item(item, ind));
    }
  }
}

/// Emit a meter/progress item.
fn emit_meter_item(item: &ItemBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);
  let ipre = indent(ind + 1);

  let has_detail = item.config.contains_key("progress")
    || item.config.contains_key("usage")
    || item.config.contains_key("limit")
    || item.description.is_some();

  if has_detail {
    out.push_str(&format!("{}meter {} {{\n", pre, quoted(&item.title)));

    if let Some(ref desc) = item.description {
      if !desc.is_empty() {
        out.push_str(&format!("{}{}\n", ipre, quoted(desc)));
      }
    }
    if let Some(progress) = item.config.get("progress") {
      out.push_str(&format!("{}progress {}\n", ipre, progress));
    }
    if let Some(usage) = item.config.get("usage") {
      out.push_str(&format!("{}usage {}\n", ipre, quoted(usage)));
    }
    if let Some(limit) = item.config.get("limit") {
      out.push_str(&format!("{}limit {}\n", ipre, quoted(limit)));
    }

    out.push_str(&format!("{}}}\n", pre));
  } else {
    out.push_str(&format!("{}meter {}\n", pre, quoted(&item.title)));
  }
}

/// Emit bento/features-style body with grouped items.
fn emit_bento_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }
  if let Some(ref subtitle) = bp.subtitle {
    if !subtitle.is_empty() {
      out.push_str(&format!("{}subtitle {}\n", pre, quoted(subtitle)));
    }
  }

  // Group items for bento layout
  let groups = group_items(&bp.items);
  for group in &groups {
    out.push_str(&emit_grouped_item(group, ind));
  }
}

fn emit_generic_body(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(ref title) = bp.title {
    if !title.is_empty() {
      out.push_str(&format!("{}title {}\n", pre, quoted(title)));
    }
  }
  if let Some(ref subtitle) = bp.subtitle {
    if !subtitle.is_empty() {
      out.push_str(&format!("{}subtitle {}\n", pre, quoted(subtitle)));
    }
  }

  // Emit any well-known config fields the section might carry
  let known_section_fields = [
    "brand", "nav", "badge", "copyright", "footnote", "icon", "columns",
  ];
  for field in &known_section_fields {
    if let Some(val) = bp.config.get(*field) {
      out.push_str(&format!("{}{} {}\n", pre, field, quoted(val)));
    }
  }

  emit_cta_entries(bp, ind, out);

  // Use grouped items for richer output
  let groups = group_items(&bp.items);
  for group in &groups {
    out.push_str(&emit_grouped_item(group, ind));
  }
}

// ---------------------------------------------------------------------------
// Grouped item emitter
// ---------------------------------------------------------------------------

/// Emit a grouped item: parent with children nested inside a block.
fn emit_grouped_item(group: &GroupedItem, ind: usize) -> String {
  if group.children.is_empty() {
    // No children — emit normally
    return emit_item(group.parent, ind);
  }

  // Has children — emit as a block
  let pre = indent(ind);
  let ipre = indent(ind + 1);
  let mut out = String::new();

  // Inline config for parent (exclude inner-only keys)
  let inner_keys = ["price", "action", "meta", "action_icon", "price_interval", "href", "_type"];
  let mut inline_parts: Vec<String> = Vec::new();
  let mut keys: Vec<&String> = group.parent.config.keys()
    .filter(|k| !inner_keys.contains(&k.as_str()))
    .collect();
  keys.sort();
  for k in &keys {
    inline_parts.push(format!("{}:{}", k, group.parent.config[*k]));
  }
  let cfg_str = inline_parts.join(" ");

  let keyword = match group.parent.item_type.as_str() {
    "product" => "item",
    "member" => "member",
    "stat" => "metric",
    "webhook" => "webhook",
    _ => "item",
  };

  if cfg_str.is_empty() {
    out.push_str(&format!("{}{} {} {{\n", pre, keyword, quoted(&group.parent.title)));
  } else {
    out.push_str(&format!("{}{} {} {} {{\n", pre, keyword, quoted(&group.parent.title), cfg_str));
  }

  // Parent description
  if let Some(ref desc) = group.parent.description {
    if !desc.is_empty() {
      out.push_str(&format!("{}{}\n", ipre, quoted(desc)));
    }
  }

  // Parent inner config (price, action, meta)
  emit_item_inner_config(group.parent, ind + 1, &mut out);

  // Children
  for child in &group.children {
    out.push_str(&emit_item(child, ind + 1));
  }

  out.push_str(&format!("{}}}\n", pre));
  out
}

// ---------------------------------------------------------------------------
// CTA helper — emits cta lines from config pairs
// ---------------------------------------------------------------------------

fn emit_cta_entries(bp: &SectionBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  // Primary CTA
  if let Some(text) = bp.config.get("cta_text") {
    let clean = clean_text(text);
    let link = bp.config.get("cta_link").map(|s| s.as_str()).unwrap_or("#");
    out.push_str(&format!(
      "{}cta {} -> {} primary\n",
      pre,
      quoted(&clean),
      quoted(link)
    ));
  }

  // Secondary CTA
  if let Some(text) = bp.config.get("cta2_text") {
    let clean = clean_text(text);
    let link = bp.config.get("cta2_link").map(|s| s.as_str()).unwrap_or("#");
    out.push_str(&format!(
      "{}cta {} -> {} secondary\n",
      pre,
      quoted(&clean),
      quoted(link)
    ));
  }

  // Additional CTAs (cta3_text, cta4_text, ...)
  let mut n = 3;
  loop {
    let key = format!("cta{}_text", n);
    if let Some(text) = bp.config.get(&key) {
      let clean = clean_text(text);
      let link_key = format!("cta{}_link", n);
      let link = bp.config.get(&link_key).map(|s| s.as_str()).unwrap_or("#");
      out.push_str(&format!(
        "{}cta {} -> {} secondary\n",
        pre,
        quoted(&clean),
        quoted(link)
      ));
      n += 1;
    } else {
      break;
    }
  }
}

// ---------------------------------------------------------------------------
// Item emitter — dispatches by item_type
// ---------------------------------------------------------------------------

fn emit_item(item: &ItemBlueprint, ind: usize) -> String {
  let item_type = item.item_type.to_lowercase();
  match item_type.as_str() {
    "item" => emit_item_block(item, ind),
    "line" => emit_simple_line("line", item, ind),
    "output" => emit_output_line(item, ind),
    "success" => emit_simple_line("success", item, ind),
    "prompt" => emit_prompt_line(item, ind),
    "chip" => emit_chip_line(item, ind),
    "code" => emit_simple_line("code", item, ind),
    "image" => emit_image_line(item, ind),
    "label" => emit_label_line(item, ind),
    "row" => emit_row_block(item, ind),
    "policy" => emit_policy_line(item, ind),
    "action" => emit_action_line(item, ind),
    "metric" => emit_metric_block(item, ind),
    "nav-link" => emit_nav_link(item, ind),
    "stat" => emit_stat_item(item, ind),
    "testimonial" => emit_testimonial_item(item, ind),
    "plan" => emit_plan_item(item, ind),
    "product" => emit_product_item(item, ind),
    "member" => emit_member_item(item, ind),
    "webhook" => emit_webhook_item(item, ind),
    "link" => emit_link_item(item, ind),
    "status" => emit_status_item(item, ind),
    "meter" | "field" => emit_meter_or_field(item, ind),
    _ => emit_item_block(item, ind), // fallback: treat as generic item
  }
}

// -- item "Title" -> "href"                     (link item)
// -- item "Title" icon:X { "description" ... }  (block item)
// -- item "Title"                                (simple item)
fn emit_item_block(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let mut out = String::new();

  // If item has href, emit as link item: item "Title" -> "href"
  if let Some(href) = item.config.get("href") {
    let icon = item.config.get("icon");
    let mut line = format!("{}item {} -> {}", pre, quoted(&item.title), quoted(href));
    if let Some(ic) = icon {
      line.push_str(&format!(" icon:{}", ic));
    }
    line.push('\n');
    out.push_str(&line);
    return out;
  }

  // Config pairs excluding inner-only keys, href, and internal keys
  let inner_keys = ["price", "action", "meta", "action_icon", "price_interval", "href", "_type"];
  let mut inline_cfg: Vec<String> = Vec::new();
  let mut keys: Vec<&String> = item.config.keys()
    .filter(|k| !inner_keys.contains(&k.as_str()))
    .collect();
  keys.sort();
  for k in &keys {
    inline_cfg.push(format!("{}:{}", k, item.config[*k]));
  }
  let cfg_str = inline_cfg.join(" ");

  let has_desc = item.description.as_ref().map(|d| !d.is_empty()).unwrap_or(false);
  let has_inner = has_inner_config(item);
  let has_children = has_desc || has_inner;

  if has_children {
    if cfg_str.is_empty() {
      out.push_str(&format!("{}item {} {{\n", pre, quoted(&item.title)));
    } else {
      out.push_str(&format!("{}item {} {} {{\n", pre, quoted(&item.title), cfg_str));
    }
    let inner = ind + 1;
    let ipre = indent(inner);

    if let Some(ref desc) = item.description {
      if !desc.is_empty() {
        out.push_str(&format!("{}{}\n", ipre, quoted(desc)));
      }
    }

    // Emit inner config keys that represent child elements (price, action, meta)
    emit_item_inner_config(item, inner, &mut out);

    out.push_str(&format!("{}}}\n", pre));
  } else {
    // Single-line item
    if cfg_str.is_empty() {
      out.push_str(&format!("{}item {}\n", pre, quoted(&item.title)));
    } else {
      out.push_str(&format!("{}item {} {}\n", pre, quoted(&item.title), cfg_str));
    }
  }

  out
}

/// Checks if an item has inner config keys (price, action, meta) that need a block.
fn has_inner_config(item: &ItemBlueprint) -> bool {
  let inner_keys = ["price", "action", "meta", "action_icon", "price_interval"];
  item.config.keys().any(|k| inner_keys.contains(&k.as_str()))
}

/// Emits inner config entries (price, action, meta) inside an item block.
fn emit_item_inner_config(item: &ItemBlueprint, ind: usize, out: &mut String) {
  let pre = indent(ind);

  if let Some(price) = item.config.get("price") {
    let mut line = format!("{}price {}", pre, quoted(price));
    if let Some(interval) = item.config.get("price_interval") {
      line.push_str(&format!(" interval:{}", quoted(interval)));
    }
    line.push('\n');
    out.push_str(&line);
  }

  if let Some(action) = item.config.get("action") {
    let mut line = format!("{}action {}", pre, quoted(action));
    if let Some(icon) = item.config.get("action_icon") {
      line.push_str(&format!(" icon:{}", icon));
    }
    line.push('\n');
    out.push_str(&line);
  }

  if let Some(meta) = item.config.get("meta") {
    out.push_str(&format!("{}meta {}\n", pre, quoted(meta)));
  }
}

// -- line "$ command"
fn emit_simple_line(keyword: &str, item: &ItemBlueprint, ind: usize) -> String {
  format!("{}{} {}\n", indent(ind), keyword, quoted(&item.title))
}

// -- output "text" color:blue
fn emit_output_line(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let cfg = emit_config_pairs(&item.config);
  if cfg.is_empty() {
    format!("{}output {}\n", pre, quoted(&item.title))
  } else {
    format!("{}output {} {}\n", pre, quoted(&item.title), cfg)
  }
}

// -- prompt "question" answer:y
fn emit_prompt_line(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let cfg = emit_config_pairs(&item.config);
  if cfg.is_empty() {
    format!("{}prompt {}\n", pre, quoted(&item.title))
  } else {
    format!("{}prompt {} {}\n", pre, quoted(&item.title), cfg)
  }
}

// -- chip "text" icon:X
fn emit_chip_line(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let cfg = emit_config_pairs(&item.config);
  if cfg.is_empty() {
    format!("{}chip {}\n", pre, quoted(&item.title))
  } else {
    format!("{}chip {} {}\n", pre, quoted(&item.title), cfg)
  }
}

// -- image "alt" src:"url"
fn emit_image_line(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let cfg = emit_config_pairs(&item.config);
  if cfg.is_empty() {
    format!("{}image {}\n", pre, quoted(&item.title))
  } else {
    format!("{}image {} {}\n", pre, quoted(&item.title), cfg)
  }
}

// -- label "TEXT" style:mono
fn emit_label_line(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let cfg = emit_config_pairs(&item.config);
  if cfg.is_empty() {
    format!("{}label {}\n", pre, quoted(&item.title))
  } else {
    format!("{}label {} {}\n", pre, quoted(&item.title), cfg)
  }
}

// -- row "Label" { key "value" }
fn emit_row_block(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let mut out = format!("{}row {} {{\n", pre, quoted(&item.title));
  let inner = ind + 1;
  let ipre = indent(inner);

  // Emit each config pair as `key "value"` inside the row (skip internal keys)
  let mut keys: Vec<&String> = item.config.keys()
    .filter(|k| !k.starts_with('_'))
    .collect();
  keys.sort();
  for k in keys {
    out.push_str(&format!("{}{} {}\n", ipre, k, quoted(&item.config[k])));
  }

  if let Some(ref desc) = item.description {
    if !desc.is_empty() {
      out.push_str(&format!("{}{}\n", ipre, quoted(desc)));
    }
  }

  out.push_str(&format!("{}}}\n", pre));
  out
}

// -- policy "Name" toggle:on
fn emit_policy_line(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let cfg = emit_config_pairs(&item.config);
  if cfg.is_empty() {
    format!("{}policy {}\n", pre, quoted(&item.title))
  } else {
    format!("{}policy {} {}\n", pre, quoted(&item.title), cfg)
  }
}

// -- action "Text" -> "/link" icon:add
fn emit_action_line(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let link = item.config.get("link").map(|s| s.as_str()).unwrap_or("#");

  // Collect config pairs excluding "link" and internal keys
  let mut extra: HashMap<&String, &String> = item
    .config
    .iter()
    .filter(|(k, _)| k.as_str() != "link" && !k.starts_with('_'))
    .collect();

  let mut line = format!("{}action {} -> {}", pre, quoted(&item.title), quoted(link));

  // Append variant if present
  if let Some(variant) = extra.remove(&"variant".to_string()) {
    line.push_str(&format!(" {}", variant));
  }

  // Append remaining config pairs
  let mut ekeys: Vec<&&String> = extra.keys().collect();
  ekeys.sort();
  for k in ekeys {
    line.push_str(&format!(" {}:{}", k, extra[k]));
  }

  line.push('\n');
  line
}

// -- metric "Label" { "Value" }
fn emit_metric_block(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let inner = ind + 1;
  let ipre = indent(inner);

  let mut out = format!("{}metric {} {{\n", pre, quoted(&item.title));

  if let Some(ref desc) = item.description {
    if !desc.is_empty() {
      out.push_str(&format!("{}{}\n", ipre, quoted(desc)));
    }
  }

  // Emit config inside metric block (skip internal keys)
  let mut keys: Vec<&String> = item.config.keys()
    .filter(|k| !k.starts_with('_'))
    .collect();
  keys.sort();
  for k in keys {
    out.push_str(&format!("{}{} {}\n", ipre, k, quoted(&item.config[k])));
  }

  out.push_str(&format!("{}}}\n", pre));
  out
}

// ---------------------------------------------------------------------------
// NEW item emitters — dashboard-specific types
// ---------------------------------------------------------------------------

// -- nav-link: item "Label" -> "/path" icon:X active:true position:bottom
fn emit_nav_link(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let href = item.config.get("href").map(|s| s.as_str()).unwrap_or("#");
  let icon = item.config.get("icon");
  let mut line = format!("{}item {} -> {}", pre, quoted(&item.title), quoted(href));
  if let Some(ic) = icon {
    line.push_str(&format!(" icon:{}", ic));
  }
  if item.config.get("active").map(|v| v == "true").unwrap_or(false) {
    line.push_str(" active:true");
  }
  if item.config.get("position").map(|v| v == "bottom").unwrap_or(false) {
    line.push_str(" position:bottom");
  }
  line.push('\n');
  line
}

// -- stat "LABEL" { "VALUE" }
fn emit_stat_item(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let ipre = indent(ind + 1);

  if let Some(ref value) = item.description {
    if !value.is_empty() {
      let mut out = format!("{}metric {} {{\n", pre, quoted(&item.title));
      out.push_str(&format!("{}{}\n", ipre, quoted(value)));
      // Emit extra config
      let mut keys: Vec<&String> = item.config.keys()
        .filter(|k| !k.starts_with('_'))
        .collect();
      keys.sort();
      for k in keys {
        out.push_str(&format!("{}{} {}\n", ipre, k, quoted(&item.config[k])));
      }
      out.push_str(&format!("{}}}\n", pre));
      return out;
    }
  }

  let cfg = emit_config_pairs(&item.config);
  if cfg.is_empty() {
    format!("{}metric {}\n", pre, quoted(&item.title))
  } else {
    format!("{}metric {} {}\n", pre, quoted(&item.title), cfg)
  }
}

// -- item "Author Name" role:"Title, Company" { "quote text" }
fn emit_testimonial_item(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let ipre = indent(ind + 1);

  let role_str = item.config.get("role").cloned().unwrap_or_default();

  if let Some(ref desc) = item.description {
    let mut line = format!("{}item {}", pre, quoted(&item.title));
    if !role_str.is_empty() {
      line.push_str(&format!(" role:{}", quoted(&role_str)));
    }
    line.push_str(" {\n");
    let mut out = line;
    out.push_str(&format!("{}{}\n", ipre, quoted(desc)));
    out.push_str(&format!("{}}}\n", pre));
    out
  } else {
    let cfg = emit_config_pairs(&item.config);
    if cfg.is_empty() {
      format!("{}item {}\n", pre, quoted(&item.title))
    } else {
      format!("{}item {} {}\n", pre, quoted(&item.title), cfg)
    }
  }
}

// -- plan "Name" "R$ 99/mês" featured [ "feature1", "feature2" ]
fn emit_plan_item(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let ipre = indent(ind + 1);

  let price = item.config.get("price").cloned().unwrap_or_default();
  let is_featured = item.config.get("featured").map(|v| v == "true").unwrap_or(false);
  let features_str = item.config.get("_features").cloned().unwrap_or_default();

  let mut line = format!("{}plan {}", pre, quoted(&item.title));
  if !price.is_empty() {
    // Emit price as a Price token (e.g. $49/mês), not a quoted string.
    // The parser expects Price token syntax: $<number>[/<interval>]
    // Try to extract numeric value and emit as Price; fallback to quoted.
    let price_clean = price.replace("R$", "").replace("US$", "").replace("€", "").trim().to_string();
    let parts: Vec<&str> = price_clean.splitn(2, '/').collect();
    let numeric = parts[0].trim().replace(' ', "").replace('.', "").replace(',', ".");
    if numeric.parse::<f64>().is_ok() {
      let interval = parts.get(1).map(|s| s.trim()).unwrap_or("");
      if interval.is_empty() {
        line.push_str(&format!(" ${}", numeric));
      } else {
        line.push_str(&format!(" ${}/{}", numeric, interval));
      }
    } else {
      line.push_str(&format!(" {}", quoted(&price)));
    }
  }
  if is_featured {
    line.push_str(" featured");
  }

  if features_str.is_empty() {
    format!("{}\n", line)
  } else {
    let mut out = format!("{} [\n", line);
    let features: Vec<&str> = features_str.split("||").collect();
    for (i, feature) in features.iter().enumerate() {
      if i < features.len() - 1 {
        out.push_str(&format!("{}{},\n", ipre, quoted(feature)));
      } else {
        out.push_str(&format!("{}{}\n", ipre, quoted(feature)));
      }
    }
    out.push_str(&format!("{}]\n", pre));
    out
  }
}

// -- product "Name" { price "$29" status "Active" }
fn emit_product_item(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let ipre = indent(ind + 1);

  let has_detail = item.description.is_some()
    || item.config.contains_key("price")
    || item.config.contains_key("status");

  if has_detail {
    // Inline config (exclude detail keys)
    let detail_keys = ["price", "price_interval", "status", "_type"];
    let mut inline: Vec<String> = Vec::new();
    let mut keys: Vec<&String> = item.config.keys()
      .filter(|k| !detail_keys.contains(&k.as_str()))
      .collect();
    keys.sort();
    for k in &keys {
      inline.push(format!("{}:{}", k, item.config[*k]));
    }
    let cfg_str = inline.join(" ");

    let mut out = if cfg_str.is_empty() {
      format!("{}item {} {{\n", pre, quoted(&item.title))
    } else {
      format!("{}item {} {} {{\n", pre, quoted(&item.title), cfg_str)
    };

    if let Some(ref desc) = item.description {
      if !desc.is_empty() {
        out.push_str(&format!("{}{}\n", ipre, quoted(desc)));
      }
    }
    if let Some(price) = item.config.get("price") {
      let mut line = format!("{}price {}", ipre, quoted(price));
      if let Some(interval) = item.config.get("price_interval") {
        line.push_str(&format!(" interval:{}", quoted(interval)));
      }
      line.push('\n');
      out.push_str(&line);
    }
    if let Some(status) = item.config.get("status") {
      out.push_str(&format!("{}status {}\n", ipre, quoted(status)));
    }

    out.push_str(&format!("{}}}\n", pre));
    out
  } else {
    let cfg = emit_config_pairs(&item.config);
    if cfg.is_empty() {
      format!("{}item {}\n", pre, quoted(&item.title))
    } else {
      format!("{}item {} {}\n", pre, quoted(&item.title), cfg)
    }
  }
}

// -- member "Name" { email "x@y" role "Admin" }
fn emit_member_item(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let ipre = indent(ind + 1);

  let has_detail = item.config.contains_key("email")
    || item.config.contains_key("role")
    || item.config.contains_key("avatar_src");

  if has_detail {
    let mut out = format!("{}member {} {{\n", pre, quoted(&item.title));

    if let Some(email) = item.config.get("email") {
      out.push_str(&format!("{}email {}\n", ipre, quoted(email)));
    }
    if let Some(role) = item.config.get("role") {
      out.push_str(&format!("{}role {}\n", ipre, quoted(role)));
    }
    if let Some(avatar) = item.config.get("avatar_src") {
      let alt = item.config.get("avatar_alt").map(|s| s.as_str()).unwrap_or("");
      out.push_str(&format!("{}avatar {} src:{}\n", ipre, quoted(alt), quoted(avatar)));
    }

    out.push_str(&format!("{}}}\n", pre));
    out
  } else {
    format!("{}member {}\n", pre, quoted(&item.title))
  }
}

// -- webhook "https://..." { status "Active" "payment.created, ..." }
fn emit_webhook_item(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let ipre = indent(ind + 1);

  let has_detail = item.config.contains_key("status") || item.description.is_some();

  if has_detail {
    let mut out = format!("{}webhook {} {{\n", pre, quoted(&item.title));

    if let Some(status) = item.config.get("status") {
      out.push_str(&format!("{}status {}\n", ipre, quoted(status)));
    }
    if let Some(ref desc) = item.description {
      if !desc.is_empty() {
        out.push_str(&format!("{}{}\n", ipre, quoted(desc)));
      }
    }

    out.push_str(&format!("{}}}\n", pre));
    out
  } else {
    format!("{}webhook {}\n", pre, quoted(&item.title))
  }
}

// -- item "Text" -> "/link" icon:X  (for link-type items)
fn emit_link_item(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let href = item.config.get("href").map(|s| s.as_str()).unwrap_or("#");
  let icon = item.config.get("icon");
  let mut line = format!("{}item {} -> {}", pre, quoted(&item.title), quoted(href));
  if let Some(ic) = icon {
    line.push_str(&format!(" icon:{}", ic));
  }
  line.push('\n');
  line
}

// -- status "All systems operational" state:operational
fn emit_status_item(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let cfg = emit_config_pairs(&item.config);
  if cfg.is_empty() {
    format!("{}status {}\n", pre, quoted(&item.title))
  } else {
    format!("{}status {} {}\n", pre, quoted(&item.title), cfg)
  }
}

// -- meter "API Calls" { progress 75 usage "7,500" limit "10,000" }
// -- field "Email" { ... }
fn emit_meter_or_field(item: &ItemBlueprint, ind: usize) -> String {
  let pre = indent(ind);
  let ipre = indent(ind + 1);
  let keyword = &item.item_type;

  let has_detail = !item.config.is_empty() || item.description.is_some();

  if has_detail {
    let mut out = format!("{}{} {} {{\n", pre, keyword, quoted(&item.title));

    if let Some(ref desc) = item.description {
      if !desc.is_empty() {
        out.push_str(&format!("{}{}\n", ipre, quoted(desc)));
      }
    }

    let mut keys: Vec<&String> = item.config.keys()
      .filter(|k| !k.starts_with('_'))
      .collect();
    keys.sort();
    for k in keys {
      out.push_str(&format!("{}{} {}\n", ipre, k, quoted(&item.config[k])));
    }

    out.push_str(&format!("{}}}\n", pre));
    out
  } else {
    format!("{}{} {}\n", pre, keyword, quoted(&item.title))
  }
}
