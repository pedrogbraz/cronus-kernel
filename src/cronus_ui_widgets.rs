//! Opt-in cronus-ui widget renderers (all 173 families).
//!
//! Family = first `style` segment (`button+primary+md` -> `button`).
//! Unknown families return None so legacy dispatchers keep working.

use crate::parser::{ComponentItemNode, ComponentNode};

pub const FAMILIES: &[&str] = &[
    "accordion",
    "alert",
    "alert-dialog",
    "animated-button",
    "animated-list",
    "animated-number",
    "app-shell",
    "area-chart",
    "aspect-ratio",
    "aurora-background",
    "autocomplete",
    "avatar",
    "avatar-group",
    "badge",
    "banner",
    "bar-chart",
    "border-beam",
    "breadcrumb",
    "button",
    "button-group",
    "calendar",
    "candlestick-chart",
    "card",
    "card-stack",
    "carousel",
    "chart",
    "checkbox",
    "chip",
    "choropleth-chart",
    "click-spark",
    "code-block",
    "code-tabs",
    "collapsible",
    "color-picker",
    "combobox",
    "command",
    "comparison-slider",
    "composed-chart",
    "confetti",
    "confirmation-dialog",
    "context-menu",
    "copy-button",
    "countdown",
    "credit-card-input",
    "currency-input",
    "data-table",
    "date-picker",
    "date-range-picker",
    "description-list",
    "dialog",
    "dock",
    "dot-pattern",
    "drawer",
    "dropdown-menu",
    "dynamic-island",
    "empty",
    "expandable-tabs",
    "fab",
    "field",
    "file-dropzone",
    "flickering-grid",
    "flip-card",
    "floating-label-input",
    "form",
    "frame",
    "funnel-chart",
    "gauge-chart",
    "glare-hover",
    "glass-card",
    "gradient-border",
    "gradient-text",
    "grid-pattern",
    "heatmap",
    "heatmap-chart",
    "highlighter",
    "hover-card",
    "image-zoom",
    "input",
    "input-group",
    "input-otp",
    "invite-dialog",
    "json-viewer",
    "kanban",
    "kbd",
    "label",
    "light-rays",
    "lightbox",
    "line-chart",
    "live-line-chart",
    "logo-carousel",
    "magnetic",
    "marquee",
    "masonry",
    "menubar",
    "meteors",
    "metric",
    "mode-toggle",
    "morphing-popover",
    "multi-select",
    "navigation-menu",
    "noise",
    "notification-center",
    "number-input",
    "orbit",
    "pagination",
    "particles",
    "password-input",
    "phone-input",
    "pie-chart",
    "pill-nav",
    "popover",
    "profit-loss-chart",
    "progress",
    "progressive-blur",
    "radar-chart",
    "radio-group",
    "rating",
    "resizable",
    "retro-grid",
    "reveal",
    "rich-text-editor",
    "ring-chart",
    "ripple",
    "sankey-chart",
    "scatter-chart",
    "scheduler",
    "scramble-text",
    "scroll-area",
    "scroll-progress",
    "segmented-control",
    "select",
    "separator",
    "sheet",
    "shimmer",
    "shiny-text",
    "sidebar",
    "signature-pad",
    "skeleton",
    "slider",
    "sonner",
    "sparkles-text",
    "sparkline",
    "spinner",
    "spinning-text",
    "split-button",
    "spotlight-card",
    "star-border",
    "status-dot",
    "stepper",
    "sunburst-chart",
    "switch",
    "table",
    "table-of-contents",
    "tabs",
    "tags-input",
    "terminal",
    "text-effect",
    "textarea",
    "tilt-card",
    "time-picker",
    "timeline",
    "toggle",
    "toggle-group",
    "toolbar",
    "tooltip",
    "tree-view",
    "typing-text",
    "usage-meter",
    "video-player",
    "word-rotate",
    "workspace-switcher",
    "toast",
    "motion-presets",
];

/// Families with a dedicated CONTRACT renderer. Interact and stub arms must
/// not run for these (Cronus Audit K13).
pub const PORTED_FAMILIES: &[&str] = &[
    "button",
    "badge",
    "input",
    "label",
    "textarea",
    "checkbox",
    "switch",
    "spinner",
    "separator",
    "kbd",
    "toggle",
    "progress",
    "alert",
    "skeleton",
    "banner",
    "slider",
    "radio-group",
    "chip",
    "avatar",
    "card",
    "empty",
    "select",
    "dialog",
    "tabs",
    "accordion",
    "table",
    "pagination",
    "breadcrumb",
    "tooltip",
    "password-input",
    "number-input",
    "field",
    "input-group",
    "rating",
    "copy-button",
    "fab",
    "toggle-group",
    "metric",
    "avatar-group",
    "button-group",
    "combobox",
    "stepper",
    "input-otp",
    "file-dropzone",
    "popover",
    "hover-card",
    "dropdown-menu",
    "collapsible",
    "mode-toggle",
    "command",
    "menubar",
    "context-menu",
    "drawer",
    "sheet",
    "calendar",
    "date-picker",
    "time-picker",
    "date-range-picker",
    "area-chart",
    "bar-chart",
    "line-chart",
    "sparkline",
    "pie-chart",
    "data-table",
    "sidebar",
    "sonner",
    "navigation-menu",
    "phone-input",
    "currency-input",
];

pub fn family_of(comp: &ComponentNode) -> Option<&str> {
    let style = comp.style.as_deref().unwrap_or("");
    let family = style.split('+').next().unwrap_or("").trim();
    if family.is_empty() {
        None
    } else {
        Some(family)
    }
}

pub fn dedicated_render(family: &str, comp: &ComponentNode) -> Option<String> {
    match family {
        "button" => Some(button_from(comp)),
        "badge" => Some(crate::cronus_ui_badge::render(comp)),
        "input" => Some(crate::cronus_ui_input::render(comp)),
        "label" => Some(crate::cronus_ui_label::render(comp)),
        "textarea" => Some(crate::cronus_ui_textarea::render(comp)),
        "checkbox" => Some(crate::cronus_ui_checkbox::render(comp)),
        "switch" => Some(crate::cronus_ui_switch::render(comp)),
        "spinner" => Some(crate::cronus_ui_spinner::render(comp)),
        "separator" => Some(crate::cronus_ui_separator::render(comp)),
        "kbd" => Some(crate::cronus_ui_kbd::render(comp)),
        "toggle" => Some(crate::cronus_ui_toggle::render(comp)),
        "progress" => Some(crate::cronus_ui_progress::render(comp)),
        "alert" => Some(crate::cronus_ui_alert::render(comp)),
        "skeleton" => Some(crate::cronus_ui_skeleton::render(comp)),
        "banner" => Some(crate::cronus_ui_banner::render(comp)),
        "slider" => Some(crate::cronus_ui_slider::render(comp)),
        "radio-group" => Some(crate::cronus_ui_radio_group::render(comp)),
        "chip" => Some(crate::cronus_ui_chip::render(comp)),
        "avatar" => Some(crate::cronus_ui_avatar::render(comp)),
        "card" => Some(crate::cronus_ui_card::render(comp)),
        "empty" => Some(crate::cronus_ui_empty::render(comp)),
        "select" => Some(crate::cronus_ui_select::render(comp)),
        "dialog" => Some(crate::cronus_ui_dialog::render(comp)),
        "tabs" => Some(crate::cronus_ui_tabs::render(comp)),
        "accordion" => Some(crate::cronus_ui_accordion::render(comp)),
        "table" => Some(crate::cronus_ui_table::render(comp)),
        "pagination" => Some(crate::cronus_ui_pagination::render(comp)),
        "breadcrumb" => Some(crate::cronus_ui_breadcrumb::render(comp)),
        "tooltip" => Some(crate::cronus_ui_tooltip::render(comp)),
        "password-input" => Some(crate::cronus_ui_password_input::render(comp)),
        "number-input" => Some(crate::cronus_ui_number_input::render(comp)),
        "field" => Some(crate::cronus_ui_field::render(comp)),
        "input-group" => Some(crate::cronus_ui_input_group::render(comp)),
        "rating" => Some(crate::cronus_ui_rating::render(comp)),
        "copy-button" => Some(crate::cronus_ui_copy_button::render(comp)),
        "fab" => Some(crate::cronus_ui_fab::render(comp)),
        "toggle-group" => Some(crate::cronus_ui_toggle_group::render(comp)),
        "metric" => Some(crate::cronus_ui_metric::render(comp)),
        "avatar-group" => Some(crate::cronus_ui_avatar_group::render(comp)),
        "button-group" => Some(crate::cronus_ui_button_group::render(comp)),
        "combobox" => Some(crate::cronus_ui_combobox::render(comp)),
        "stepper" => Some(crate::cronus_ui_stepper::render(comp)),
        "input-otp" => Some(crate::cronus_ui_input_otp::render(comp)),
        "file-dropzone" => Some(crate::cronus_ui_file_dropzone::render(comp)),
        "popover" => Some(crate::cronus_ui_popover::render(comp)),
        "hover-card" => Some(crate::cronus_ui_hover_card::render(comp)),
        "dropdown-menu" => Some(crate::cronus_ui_dropdown_menu::render(comp)),
        "collapsible" => Some(crate::cronus_ui_collapsible::render(comp)),
        "mode-toggle" => Some(crate::cronus_ui_mode_toggle::render(comp)),
        "command" => Some(crate::cronus_ui_command::render(comp)),
        "menubar" => Some(crate::cronus_ui_menubar::render(comp)),
        "context-menu" => Some(crate::cronus_ui_context_menu::render(comp)),
        "drawer" => Some(crate::cronus_ui_drawer::render(comp)),
        "sheet" => Some(crate::cronus_ui_sheet::render(comp)),
        "calendar" => Some(crate::cronus_ui_calendar::render(comp)),
        "date-picker" => Some(crate::cronus_ui_date_picker::render(comp)),
        "time-picker" => Some(crate::cronus_ui_time_picker::render(comp)),
        "date-range-picker" => Some(crate::cronus_ui_date_range_picker::render(comp)),
        "area-chart" => Some(crate::cronus_ui_area_chart::render(comp)),
        "bar-chart" => Some(crate::cronus_ui_bar_chart::render(comp)),
        "line-chart" => Some(crate::cronus_ui_line_chart::render(comp)),
        "sparkline" => Some(crate::cronus_ui_sparkline::render(comp)),
        "pie-chart" => Some(crate::cronus_ui_pie_chart::render(comp)),
        "data-table" => Some(crate::cronus_ui_data_table::render(comp)),
        "sidebar" => Some(crate::cronus_ui_sidebar::render(comp)),
        "sonner" => Some(crate::cronus_ui_sonner::render(comp)),
        "navigation-menu" => Some(crate::cronus_ui_navigation_menu::render(comp)),
        "phone-input" => Some(crate::cronus_ui_phone_input::render(comp)),
        "currency-input" => Some(crate::cronus_ui_currency_input::render(comp)),
        _ => None,
    }
}

pub fn render(comp: &ComponentNode) -> Option<String> {
    let family = family_of(comp)?;
    if PORTED_FAMILIES.contains(&family) {
        return dedicated_render(family, comp);
    }
    if let Some(html) = crate::cronus_ui_interact::render(family, comp) {
        return Some(html);
    }
    let html = match family {
        "accordion" => nav("accordion", comp),
        "alert" => display("alert", comp),
        "alert-dialog" => overlay("alert-dialog", comp),
        "animated-button" => fx("animated-button", comp),
        "animated-list" => fx("animated-list", comp),
        "animated-number" => fx("animated-number", comp),
        "app-shell" => nav("app-shell", comp),
        "area-chart" => chart("area-chart", comp),
        "aspect-ratio" => display("aspect-ratio", comp),
        "aurora-background" => fx("aurora-background", comp),
        "autocomplete" => field("autocomplete", comp),
        "avatar" => display("avatar", comp),
        "avatar-group" => display("avatar-group", comp),
        "badge" => pill("badge", comp),
        "banner" => display("banner", comp),
        "bar-chart" => chart("bar-chart", comp),
        "border-beam" => fx("border-beam", comp),
        "breadcrumb" => nav("breadcrumb", comp),
        "button" => button_from(comp),
        "button-group" => nav("button-group", comp),
        "calendar" => display("calendar", comp),
        "candlestick-chart" => chart("candlestick-chart", comp),
        "card" => display("card", comp),
        "card-stack" => display("card-stack", comp),
        "carousel" => display("carousel", comp),
        "chart" => chart("chart", comp),
        "checkbox" => pill("checkbox", comp),
        "chip" => pill("chip", comp),
        "choropleth-chart" => chart("choropleth-chart", comp),
        "click-spark" => fx("click-spark", comp),
        "code-block" => display("code-block", comp),
        "code-tabs" => display("code-tabs", comp),
        "collapsible" => nav("collapsible", comp),
        "color-picker" => field("color-picker", comp),
        "combobox" => field("combobox", comp),
        "command" => overlay("command", comp),
        "comparison-slider" => fx("comparison-slider", comp),
        "composed-chart" => chart("composed-chart", comp),
        "confetti" => fx("confetti", comp),
        "confirmation-dialog" => overlay("confirmation-dialog", comp),
        "context-menu" => overlay("context-menu", comp),
        "copy-button" => pill("copy-button", comp),
        "countdown" => fx("countdown", comp),
        "credit-card-input" => field("credit-card-input", comp),
        "currency-input" => field("currency-input", comp),
        "data-table" => display("data-table", comp),
        "date-picker" => field("date-picker", comp),
        "date-range-picker" => field("date-range-picker", comp),
        "description-list" => display("description-list", comp),
        "dialog" => overlay("dialog", comp),
        "dock" => nav("dock", comp),
        "dot-pattern" => fx("dot-pattern", comp),
        "drawer" => overlay("drawer", comp),
        "dropdown-menu" => overlay("dropdown-menu", comp),
        "dynamic-island" => fx("dynamic-island", comp),
        "empty" => display("empty", comp),
        "expandable-tabs" => nav("expandable-tabs", comp),
        "fab" => pill("fab", comp),
        "field" => field("field", comp),
        "file-dropzone" => field("file-dropzone", comp),
        "flickering-grid" => fx("flickering-grid", comp),
        "flip-card" => display("flip-card", comp),
        "floating-label-input" => field("floating-label-input", comp),
        "form" => field("form", comp),
        "frame" => display("frame", comp),
        "funnel-chart" => chart("funnel-chart", comp),
        "gauge-chart" => chart("gauge-chart", comp),
        "glare-hover" => fx("glare-hover", comp),
        "glass-card" => display("glass-card", comp),
        "gradient-border" => fx("gradient-border", comp),
        "gradient-text" => fx("gradient-text", comp),
        "grid-pattern" => fx("grid-pattern", comp),
        "heatmap" => chart("heatmap", comp),
        "heatmap-chart" => chart("heatmap-chart", comp),
        "highlighter" => fx("highlighter", comp),
        "hover-card" => overlay("hover-card", comp),
        "image-zoom" => fx("image-zoom", comp),
        "input" => field("input", comp),
        "input-group" => field("input-group", comp),
        "input-otp" => field("input-otp", comp),
        "invite-dialog" => overlay("invite-dialog", comp),
        "json-viewer" => display("json-viewer", comp),
        "kanban" => display("kanban", comp),
        "kbd" => pill("kbd", comp),
        "label" => pill("label", comp),
        "light-rays" => fx("light-rays", comp),
        "lightbox" => overlay("lightbox", comp),
        "line-chart" => chart("line-chart", comp),
        "live-line-chart" => chart("live-line-chart", comp),
        "logo-carousel" => display("logo-carousel", comp),
        "magnetic" => fx("magnetic", comp),
        "marquee" => fx("marquee", comp),
        "masonry" => display("masonry", comp),
        "menubar" => overlay("menubar", comp),
        "meteors" => fx("meteors", comp),
        "metric" => display("metric", comp),
        "mode-toggle" => nav("mode-toggle", comp),
        "morphing-popover" => overlay("morphing-popover", comp),
        "multi-select" => field("multi-select", comp),
        "navigation-menu" => nav("navigation-menu", comp),
        "noise" => fx("noise", comp),
        "notification-center" => overlay("notification-center", comp),
        "number-input" => field("number-input", comp),
        "orbit" => fx("orbit", comp),
        "pagination" => nav("pagination", comp),
        "particles" => fx("particles", comp),
        "password-input" => field("password-input", comp),
        "phone-input" => field("phone-input", comp),
        "pie-chart" => chart("pie-chart", comp),
        "pill-nav" => nav("pill-nav", comp),
        "popover" => overlay("popover", comp),
        "profit-loss-chart" => chart("profit-loss-chart", comp),
        "progress" => pill("progress", comp),
        "progressive-blur" => fx("progressive-blur", comp),
        "radar-chart" => chart("radar-chart", comp),
        "radio-group" => pill("radio-group", comp),
        "rating" => pill("rating", comp),
        "resizable" => display("resizable", comp),
        "retro-grid" => fx("retro-grid", comp),
        "reveal" => fx("reveal", comp),
        "rich-text-editor" => field("rich-text-editor", comp),
        "ring-chart" => chart("ring-chart", comp),
        "ripple" => fx("ripple", comp),
        "sankey-chart" => chart("sankey-chart", comp),
        "scatter-chart" => chart("scatter-chart", comp),
        "scheduler" => display("scheduler", comp),
        "scramble-text" => fx("scramble-text", comp),
        "scroll-area" => display("scroll-area", comp),
        "scroll-progress" => fx("scroll-progress", comp),
        "segmented-control" => pill("segmented-control", comp),
        "select" => field("select", comp),
        "separator" => pill("separator", comp),
        "sheet" => overlay("sheet", comp),
        "shimmer" => fx("shimmer", comp),
        "shiny-text" => fx("shiny-text", comp),
        "sidebar" => nav("sidebar", comp),
        "signature-pad" => field("signature-pad", comp),
        "skeleton" => pill("skeleton", comp),
        "slider" => pill("slider", comp),
        "sonner" => overlay("sonner", comp),
        "sparkles-text" => fx("sparkles-text", comp),
        "sparkline" => chart("sparkline", comp),
        "spinner" => pill("spinner", comp),
        "spinning-text" => fx("spinning-text", comp),
        "split-button" => nav("split-button", comp),
        "spotlight-card" => display("spotlight-card", comp),
        "star-border" => fx("star-border", comp),
        "status-dot" => pill("status-dot", comp),
        "stepper" => nav("stepper", comp),
        "sunburst-chart" => chart("sunburst-chart", comp),
        "switch" => pill("switch", comp),
        "table" => display("table", comp),
        "table-of-contents" => nav("table-of-contents", comp),
        "tabs" => nav("tabs", comp),
        "tags-input" => field("tags-input", comp),
        "terminal" => display("terminal", comp),
        "text-effect" => fx("text-effect", comp),
        "textarea" => field("textarea", comp),
        "tilt-card" => display("tilt-card", comp),
        "time-picker" => field("time-picker", comp),
        "timeline" => display("timeline", comp),
        "toggle" => pill("toggle", comp),
        "toggle-group" => pill("toggle-group", comp),
        "toolbar" => nav("toolbar", comp),
        "tooltip" => overlay("tooltip", comp),
        "tree-view" => display("tree-view", comp),
        "typing-text" => fx("typing-text", comp),
        "usage-meter" => display("usage-meter", comp),
        "video-player" => display("video-player", comp),
        "word-rotate" => fx("word-rotate", comp),
        "workspace-switcher" => nav("workspace-switcher", comp),
        "toast" => fx("toast", comp),
        "motion-presets" => fx("motion-presets", comp),
        _ => return None,
    };
    Some(html)
}

fn label_of(comp: &ComponentNode) -> String {
    for kind in ["label", "title", "text", "value"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() {
                return esc(t);
            }
        }
    }
    esc(&comp.name)
}

fn item<'a>(comp: &'a ComponentNode, kind: &str) -> Option<&'a str> {
    comp.items
        .iter()
        .find(|i| i.item_type == kind)
        .map(|i| i.text.as_str())
}

fn texts(comp: &ComponentNode) -> Vec<String> {
    let mut out: Vec<String> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if out.is_empty() {
        out.push(label_of(comp));
    }
    out
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn variant(comp: &ComponentNode) -> String {
    comp.props.get("variant").cloned().unwrap_or_else(|| {
        let style = comp.style.as_deref().unwrap_or("");
        for v in ["destructive", "danger", "secondary", "outline", "ghost", "link", "primary"] {
            if style.contains(v) {
                return v.to_string();
            }
        }
        "primary".into()
    })
}

fn size(comp: &ComponentNode) -> String {
    comp.props.get("size").cloned().unwrap_or_else(|| {
        let style = comp.style.as_deref().unwrap_or("");
        if style.contains("icon-sm") {
            "icon-sm".into()
        } else if style.contains("icon") {
            "icon".into()
        } else if style.contains("lg") {
            "lg".into()
        } else if style.contains("sm") {
            "sm".into()
        } else {
            "md".into()
        }
    })
}

fn disabled(comp: &ComponentNode) -> bool {
    if comp.props.get("disabled").map(|s| s == "true").unwrap_or(false) {
        return true;
    }
    comp.items
        .iter()
        .any(|i| i.config.get("disabled").map(|s| s == "true").unwrap_or(false))
}

fn href(comp: &ComponentNode) -> Option<&str> {
    comp.items.iter().find_map(|i| i.link.as_deref())
}

fn button_from(comp: &ComponentNode) -> String {
    let raw = item(comp, "label").unwrap_or(comp.name.as_str());
    crate::cronus_ui::button_ex(raw, &variant(comp), &size(comp), href(comp), disabled(comp))
}

const BASE: &str = "color:var(--cronus-fg);font-family:var(--cronus-font-sans,inherit);box-sizing:border-box;";
const SURF: &str = "background:var(--cronus-surface-raised);border:1px solid var(--cronus-border);border-radius:var(--cronus-radius,14px);";

fn pill(family: &str, comp: &ComponentNode) -> String {
    let label = label_of(comp);
    format!(
        "<span data-slot=\"{family}\" style=\"{BASE}{SURF}display:inline-flex;align-items:center;gap:0.35rem;padding:0.15rem 0.55rem;font-size:0.75rem;font-weight:500;\">{label}</span>"
    )
}

fn field(family: &str, comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let ph = esc(item(comp, "text").unwrap_or(""));
    format!(
        "<label data-slot=\"{family}\" style=\"{BASE}display:flex;flex-direction:column;gap:0.35rem;font-size:0.875rem;\"><span style=\"color:var(--cronus-fg-secondary);\">{label}</span><input data-slot=\"{family}-control\" placeholder=\"{ph}\" style=\"height:2.5rem;padding:0 0.75rem;border-radius:0.5rem;border:1px solid var(--cronus-border);background:var(--cronus-surface-inset);color:var(--cronus-fg);outline:none;\" /></label>"
    )
}

fn overlay(family: &str, comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let body = texts(comp)
        .into_iter()
        .map(|t| format!("<p style=\"margin:0;color:var(--cronus-fg-secondary);font-size:0.875rem;\">{t}</p>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"{family}\" role=\"dialog\" style=\"{BASE}{SURF}padding:1.25rem;display:flex;flex-direction:column;gap:0.75rem;max-width:28rem;\"><div style=\"font-weight:500;\">{title}</div>{body}</div>"
    )
}

fn nav(family: &str, comp: &ComponentNode) -> String {
    let links = texts(comp)
        .iter()
        .map(|t| format!("<a href=\"#\" style=\"color:var(--cronus-fg-secondary);text-decoration:none;padding:0.35rem 0.6rem;border-radius:0.4rem;\">{t}</a>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<nav data-slot=\"{family}\" style=\"{BASE}display:flex;flex-wrap:wrap;gap:0.25rem;align-items:center;\">{links}</nav>"
    )
}

fn display(family: &str, comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let rest = texts(comp)
        .into_iter()
        .skip(1)
        .map(|t| format!("<div style=\"color:var(--cronus-fg-secondary);font-size:0.875rem;\">{t}</div>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<section data-slot=\"{family}\" style=\"{BASE}{SURF}padding:1rem;display:flex;flex-direction:column;gap:0.5rem;\"><div style=\"font-weight:500;\">{title}</div>{rest}</section>"
    )
}

fn chart(family: &str, comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let rows = crate::cronus_ui_data::rows();
    let bars = if rows.len() >= 2 {
        let vals: Vec<f64> = rows
            .iter()
            .filter_map(|r| {
                r.get("value")
                    .or_else(|| r.get("amount"))
                    .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            })
            .collect();
        let max = vals.iter().cloned().fold(1.0_f64, f64::max);
        vals.iter()
            .map(|v| {
                let h = ((*v / max) * 36.0).max(2.0);
                format!(
                    "<div style=\"flex:1;height:{h}px;background:var(--cronus-primary);border-radius:2px 2px 0 0;\"></div>"
                )
            })
            .collect::<Vec<_>>()
            .join("")
    } else {
        String::new()
    };
    let plot = if bars.is_empty() {
        r#"<svg viewBox="0 0 120 40" width="100%" height="80" aria-hidden="true"><polyline fill="none" stroke="var(--cronus-primary)" stroke-width="2" points="0,30 20,22 40,26 60,12 80,16 100,8 120,14" /></svg>"#.to_string()
    } else {
        format!("<div style=\"display:flex;align-items:flex-end;gap:4px;height:80px;\">{bars}</div>")
    };
    format!(
        "<figure data-slot=\"{family}\" style=\"{BASE}{SURF}padding:1rem;\"><figcaption style=\"margin-bottom:0.5rem;font-size:0.875rem;color:var(--cronus-fg-secondary);\">{title}</figcaption>{plot}</figure>"
    )
}

fn fx(family: &str, comp: &ComponentNode) -> String {
    let title = label_of(comp);
    format!(
        "<div data-slot=\"{family}\" style=\"{BASE}{SURF}padding:0.75rem 1rem;position:relative;overflow:hidden;\"><span>{title}</span></div>"
    )
}

pub(crate) fn test_stub(family: &str) -> ComponentNode {
    stub(family)
}

fn stub(family: &str) -> ComponentNode {
    ComponentNode {
        name: family.to_string(),
        layout: Some("stack".into()),
        style: Some(format!("{family}+primary")),
        items: vec![ComponentItemNode {
            item_type: "label".into(),
            text: "Demo".into(),
            link: None,
            tone: None,
            config: Default::default(),
        }],
        props: Default::default(),
        params: vec![],
        template: None,
        sections: vec![],
        state: vec![],
        tests: vec![],
        binding: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_173_unique_families() {
        assert_eq!(FAMILIES.len(), 173);
        let mut s = std::collections::BTreeSet::new();
        for f in FAMILIES {
            assert!(s.insert(*f), "duplicate {f}");
        }
    }

    #[test]
    fn every_family_renders_slot_without_palette_scales() {
        for family in FAMILIES {
            let html = render(&stub(family)).expect(family);
            let slot_ok = html.contains(&format!("data-slot=\"{family}\""))
                || html.contains(&format!("data-slot=\"{family}-content\""))
                || html.contains(&format!("data-slot=\"{family}-trigger\""))
                || html.contains("data-slot=\"button\"");
            assert!(slot_ok, "{family} missing data-slot: {html}");
            assert!(!html.contains("zinc-"), "{family} used zinc palette");
            assert!(!html.contains("amber-500"), "{family} used amber palette");
            assert!(!html.contains("bg-neutral-"), "{family} used neutral palette scale");
        }
    }

    #[test]
    fn unknown_family_is_none() {
        let mut c = stub("nope-widget");
        c.style = Some("nope-widget".into());
        assert!(render(&c).is_none());
    }

    #[test]
    fn legacy_primary_style_is_not_hijacked() {
        let mut c = stub("button");
        c.style = Some("primary".into());
        assert!(render(&c).is_none());
    }

    #[test]
    fn interactive_families_emit_real_controls() {
        let cases = [
            ("checkbox", "role=\"checkbox\""),
            ("switch", "role=\"switch\""),
            ("input", "<input"),
            ("textarea", "<textarea"),
            ("select", "<select"),
            ("dialog", "<dialog"),
            ("accordion", "<details"),
            ("tabs", "role=\"tablist\""),
            ("table", "<table"),
            ("progress", "role=\"progressbar\""),
            ("slider", "role=\"slider\""),
            ("radio-group", "role=\"radiogroup\""),
        ];
        for (family, needle) in cases {
            let html = render(&stub(family)).expect(family);
            assert!(html.contains(needle), "{family} missing {needle}: {html}");
            assert!(!html.contains("{ value }"), "{family} leaked voodoo interp");
            assert!(!html.contains("v-data="), "{family} leaked v-data without opt-in");
            assert!(!html.contains("v-model="), "{family} leaked v-model without opt-in");
            assert!(!html.contains("zinc-"), "{family} used zinc palette");
        }
    }

    #[test]
    fn voodoo_attrs_only_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("select")).unwrap();
            assert!(html.contains("v-data="));
            assert!(html.contains("v-model="));
            let meter = render(&stub("usage-meter")).unwrap();
            assert!(meter.contains("{ value }") || meter.contains("v-data="));
            let tabs = render(&stub("tabs")).unwrap();
            assert!(tabs.contains("role=\"tablist\""));
            assert!(tabs.contains("onclick="), "tabs stay native; v-show + hidden deadlock");
            assert!(!tabs.contains("v-show="));
            let checkbox = render(&stub("checkbox")).unwrap();
            assert!(!checkbox.contains("v-data="), "{checkbox}");
            assert!(!checkbox.contains("v-model="), "{checkbox}");
        });
        let off = render(&stub("select")).unwrap();
        assert!(!off.contains("v-data="));
        assert!(!off.contains("{ value }"));
    }

    #[test]
    fn voodoo_off_does_not_inject_script() {
        let page = crate::ui::render_layout("Legacy", &[], "blue", "<p>ok</p>");
        assert!(!page.contains("voodoojs"));
        assert!(!page.contains("data-cronus-runtime"));
    }

    #[test]
    fn metric_reads_bound_count() {
        use crate::binding::ResolvedData;
        crate::cronus_ui_data::with_binding("Lead", &ResolvedData::Count(12), || {
            let html = render(&stub("metric")).unwrap();
            assert!(html.contains("12"), "{html}");
            assert!(html.contains("data-slot=\"metric\""));
        });
    }

    #[test]
    fn form_gets_vsubmit_when_voodoo_and_entity() {
        use crate::binding::ResolvedData;
        crate::voodoo::with_enabled(true, || {
            crate::cronus_ui_data::with_binding("Lead", &ResolvedData::None, || {
                let html = render(&stub("form")).unwrap();
                assert!(html.contains("v-submit=\"/api/lead\""), "{html}");
                assert!(html.contains("v-method=\"POST\""));
            });
        });
        let off = render(&stub("form")).unwrap();
        assert!(!off.contains("v-submit="), "{off}");
    }

    #[test]
    fn interactive_controls_are_labelled() {
        for family in ["checkbox", "switch", "textarea", "select"] {
            let html = render(&stub(family)).unwrap();
            assert!(
                html.contains("<label") || html.contains("aria-label"),
                "{family} has no label: {html}"
            );
        }
        let tabs = render(&stub("tabs")).unwrap();
        assert!(tabs.contains("role=\"tablist\""));
        let dlg = render(&stub("dialog")).unwrap();
        assert!(dlg.contains("<dialog"));
        let toast = render(&stub("toast")).unwrap();
        assert!(toast.contains("aria-live"));
    }

    #[test]
    fn ported_family_skips_interact() {
        for family in PORTED_FAMILIES {
            let html = render(&stub(family)).expect(family);
            if let Some(ih) = crate::cronus_ui_interact::render(family, &stub(family)) {
                assert_ne!(html, ih, "{family} still equals interact HTML");
            }
            assert!(
                !html.contains("data-slot=\"input-control\""),
                "{family} used interact input-control"
            );
        }
    }

    #[test]
    fn ported_family_does_not_use_interact_generic() {
        let html = render(&stub("button")).unwrap();
        assert!(html.contains("data-slot=\"button\""));
        assert!(!html.contains("<label data-slot"));
        assert!(!crate::cli::stub_renderer_gate::looks_like_interact_generic(&html));
    }

    #[test]
    fn disabled_colon_pair_is_a_prop() {
        let src = r#"
app "x" { port 1 }
component B layout:inline style:button+primary+md {
  label "Save"
  disabled:true
  invalid:true
}
"#;
        let nodes = crate::parser::parse(src).expect("parse");
        let comp = nodes
            .iter()
            .find_map(|n| match n {
                crate::parser::AstNode::Component(c) => Some(c),
                _ => None,
            })
            .expect("component");
        let html = render(comp).unwrap();
        assert!(
            html.contains(" disabled"),
            "props={:?} items={:?} html={html}",
            comp.props,
            comp.items
        );
    }

    #[test]
    fn parser_bind_on_component() {
        let src = r#"
app "X" { port 1 }
component Revenue layout:stack style:metric {
  label "Revenue"
  bind Order { query count }
}
"#;
        let nodes = crate::parser::parse(src).expect("parse");
        let comp = nodes.iter().find_map(|n| match n {
            crate::parser::AstNode::Component(c) => Some(c),
            _ => None,
        }).expect("component");
        let b = comp.binding.as_ref().expect("binding");
        assert_eq!(b.entity, "Order");
    }

    #[test]
    fn catalog_source_is_declaration_only() {
        let src = include_str!("../demos/cronus-ui-catalog/app.cronus");
        assert!(!src.contains("<div"));
        assert!(!src.contains("className"));
        assert!(!src.contains("jsx"));
        assert!(!src.contains("template \""));
        assert!(!src.contains("style_block"));
        assert!(!src.contains(".tsx"));
        assert!(!src.contains("eq:\""));
        assert!(!src.contains("query sum"));
        assert!(!src.contains("zinc-"));
        assert!(!src.contains("amber-"));
    }

    #[test]
    fn catalog_families_are_native_not_stub() {
        use crate::cli::stub_renderer_gate::{renderer_kind, RendererKind};
        let src = include_str!("../demos/cronus-ui-catalog/app.cronus");
        let nodes = crate::parser::parse(src).expect("parse catalog");
        let mut families = Vec::new();
        for node in &nodes {
            if let crate::parser::AstNode::Component(comp) = node {
                let family = family_of(comp).expect(&comp.name);
                families.push(family.to_string());
                match renderer_kind(family) {
                    RendererKind::Dedicated(_) | RendererKind::Interact => {}
                    other => panic!("{family} on catalog is {other:?}, expected native"),
                }
                let html = render(comp).expect(family);
                assert!(
                    html.contains("data-slot") || html.contains("--cronus-"),
                    "{family} missing slot/tokens: {html}"
                );
                assert!(!html.contains("zinc-"), "{family}");
                assert!(!html.contains("amber-500"), "{family}");
                assert!(!html.contains("bg-neutral-"), "{family}");
                if matches!(renderer_kind(family), RendererKind::Stub(_)) {
                    panic!("{family} stub");
                }
            }
        }
        assert!(
            families.len() >= 20,
            "catalog too small: {}",
            families.len()
        );
        let has_app = nodes
            .iter()
            .any(|n| matches!(n, crate::parser::AstNode::App(_)));
        let has_page = nodes
            .iter()
            .any(|n| matches!(n, crate::parser::AstNode::Page(_)));
        assert!(has_app && has_page);
    }

    #[test]
    fn catalog_kit_html_contains_widget_labels_and_tokens() {
        let src = include_str!("../demos/cronus-ui-catalog/app.cronus");
        let nodes = crate::parser::parse(src).expect("parse catalog");
        let comps: Vec<_> = nodes
            .iter()
            .filter_map(|n| match n {
                crate::parser::AstNode::Component(c) => Some(c.clone()),
                _ => None,
            })
            .collect();
        let html = crate::ui::render_components_page(&comps);
        assert!(html.contains("Save"), "{html}");
        assert!(html.contains("Email") || html.contains("you@cooud.app"), "{html}");
        for family in ["button", "input", "dialog", "tabs", "select"] {
            assert!(
                html.contains(&format!("data-slot=\"{family}\"")),
                "missing {family}"
            );
        }
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("--cronus-") || crate::cronus_ui::token_css("aurora", "dark").contains("--cronus-"));
        assert!(!html.contains("zinc-"));
    }

    fn catalog_component(name: &str) -> crate::parser::ComponentNode {
        let src = include_str!("../demos/cronus-ui-catalog/app.cronus");
        let nodes = crate::parser::parse(src).expect("parse catalog");
        nodes
            .into_iter()
            .find_map(|n| match n {
                crate::parser::AstNode::Component(c) if c.name == name => Some(c),
                _ => None,
            })
            .unwrap_or_else(|| panic!("catalog missing component {name}"))
    }

    #[test]
    fn catalog_select_options_are_items_not_the_field_label() {
        let html = render(&catalog_component("Plan")).expect("select");
        assert!(
            html.contains("data-slot=\"label\">Plan</span>"),
            "Plan must be the field label: {html}"
        );
        assert!(html.contains("<option value=\"Free\">Free</option>"), "{html}");
        assert!(html.contains("<option value=\"Pro\">Pro</option>"), "{html}");
        assert!(
            !html.contains("<option value=\"Plan\">"),
            "Plan must not be an option: {html}"
        );
        assert_eq!(
            html.matches("<option ").count(),
            2,
            "expected Free/Pro only: {html}"
        );
    }

    #[test]
    fn catalog_radio_items_are_items_not_the_field_label() {
        let html = render(&catalog_component("PlanRadio")).expect("radio-group");
        assert!(
            html.contains("role=\"radiogroup\" aria-label=\"Plan\""),
            "Plan must name the group: {html}"
        );
        assert_eq!(
            html.matches("role=\"radio\"").count(),
            2,
            "expected Free/Pro only: {html}"
        );
        assert!(html.contains("aria-label=\"Free\""), "{html}");
        assert!(html.contains("aria-label=\"Pro\""), "{html}");
        assert!(
            !html.contains("role=\"radio\"") || !html.contains("aria-label=\"Plan\"></button>"),
            "Plan must not be a radio: {html}"
        );
        assert!(
            !html.contains("data-slot=\"radio-group-item\" role=\"radio\"")
                || html
                    .split("data-slot=\"radio-group-item\"")
                    .skip(1)
                    .all(|chunk| !chunk.contains("aria-label=\"Plan\"")),
            "Plan leaked as a radio item: {html}"
        );
    }
}
