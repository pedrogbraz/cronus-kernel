"""Emit src/cronus_ui_widgets.rs — one opt-in renderer per cronus-ui family.

Do not add a new family as a stub (pill/field/overlay/nav/display/chart/fx).
A family is only ported when dedicated_render calls a named module/function
and PORTED_FAMILIES lists it so interact is skipped.
"""
from pathlib import Path
from string import Template

FAMILIES = [
    "accordion", "alert", "alert-dialog", "animated-button", "animated-list",
    "animated-number", "app-shell", "area-chart", "aspect-ratio", "aurora-background",
    "autocomplete", "avatar", "avatar-group", "badge", "banner", "bar-chart",
    "border-beam", "bouncy-accordion", "breadcrumb", "button", "button-group", "calendar",
    "candlestick-chart", "card", "card-stack", "carousel", "chart", "checkbox",
    "chip", "choropleth-chart", "click-spark", "code-block", "code-tabs",
    "collapsible", "color-picker", "combobox", "command", "comparison-slider",
    "composed-chart", "confetti", "confirmation-dialog", "context-menu",
    "copy-button", "countdown", "credit-card-input", "currency-input",
    "data-table", "date-picker", "date-range-picker", "description-list",
    "dialog", "dock", "dot-pattern", "drawer", "dropdown-menu", "dynamic-island",
    "empty", "expandable-tabs", "fab", "field", "file-dropzone", "flickering-grid",
    "flip-card", "floating-label-input", "form", "frame", "funnel-chart",
    "gauge-chart", "glare-hover", "glass-card", "gradient-border", "gradient-text",
    "grid-pattern", "heatmap", "heatmap-chart", "highlighter", "hover-card",
    "image-zoom", "input", "input-group", "input-otp", "invite-dialog",
    "json-viewer", "kanban", "kbd", "label", "light-rays", "lightbox",
    "line-chart", "live-line-chart", "logo-carousel", "magnetic", "marquee",
    "masonry", "menubar", "meteors", "metric", "mode-toggle", "morphing-popover",
    "multi-select", "navigation-menu", "noise", "notification-center",
    "number-input", "orbit", "pagination", "particles", "password-input",
    "phone-input", "pie-chart", "pill-nav", "popover", "profit-loss-chart",
    "progress", "progressive-blur", "radar-chart", "radio-group", "rating",
    "resizable", "retro-grid", "reveal", "rich-text-editor", "ring-chart",
    "ripple", "sankey-chart", "scatter-chart", "scheduler", "scramble-text",
    "scroll-area", "scroll-progress", "segmented-control", "select", "separator",
    "sheet", "shimmer", "shiny-text", "sidebar", "signature-pad", "skeleton",
    "slider", "sonner", "sparkles-text", "sparkline", "spinner", "spinning-text",
    "split-button", "spotlight-card", "star-border", "status-dot", "stepper",
    "sunburst-chart", "switch", "table", "table-of-contents", "tabs",
    "tags-input", "terminal", "text-effect", "text-shimmer", "textarea", "tilt-card",
    "time-picker", "timeline", "toggle", "toggle-group", "toolbar", "tooltip",
    "tree-view", "typing-text", "usage-meter", "video-player", "word-rotate",
    "workspace-switcher",
    "toast",
    "motion-presets",
]

KIND: dict[str, str] = {}


def add(kind: str, *names: str) -> None:
    for n in names:
        KIND[n] = kind


add("button", "button")
add(
    "pill",
    "badge", "chip", "kbd", "status-dot", "spinner", "skeleton", "separator",
    "label", "progress", "slider", "rating", "copy-button", "fab", "toggle",
    "switch", "checkbox", "radio-group", "toggle-group", "segmented-control",
)
add(
    "field",
    "input", "textarea", "password-input", "number-input", "phone-input",
    "currency-input", "credit-card-input", "floating-label-input", "tags-input",
    "input-otp", "input-group", "color-picker", "file-dropzone", "select",
    "combobox", "autocomplete", "multi-select", "date-picker", "date-range-picker",
    "time-picker", "field", "form", "signature-pad", "rich-text-editor",
)
add(
    "overlay",
    "dialog", "alert-dialog", "confirmation-dialog", "invite-dialog", "sheet",
    "drawer", "popover", "hover-card", "tooltip", "dropdown-menu", "context-menu",
    "menubar", "command", "lightbox", "morphing-popover", "notification-center",
    "sonner",
)
add(
    "nav",
    "tabs", "accordion", "bouncy-accordion", "collapsible", "breadcrumb", "pagination", "sidebar",
    "navigation-menu", "pill-nav", "expandable-tabs", "stepper", "toolbar",
    "table-of-contents", "app-shell", "workspace-switcher", "dock",
    "mode-toggle", "split-button", "button-group",
)
add(
    "display",
    "card", "card-stack", "glass-card", "spotlight-card", "tilt-card", "flip-card",
    "banner", "empty", "alert", "metric", "description-list", "table",
    "data-table", "avatar", "avatar-group", "aspect-ratio", "frame",
    "scroll-area", "resizable", "calendar", "timeline", "kanban", "scheduler",
    "json-viewer", "code-block", "code-tabs", "terminal", "tree-view",
    "carousel", "logo-carousel", "masonry", "usage-meter", "video-player",
)
add(
    "chart",
    "area-chart", "bar-chart", "line-chart", "pie-chart", "radar-chart",
    "composed-chart", "candlestick-chart", "choropleth-chart", "funnel-chart",
    "gauge-chart", "heatmap", "heatmap-chart", "live-line-chart",
    "profit-loss-chart", "ring-chart", "sankey-chart", "scatter-chart",
    "sparkline", "sunburst-chart", "chart",
)
add(
    "fx",
    "animated-button", "animated-list", "animated-number", "aurora-background",
    "border-beam", "click-spark", "confetti", "countdown", "dot-pattern",
    "flickering-grid", "glare-hover", "gradient-border", "gradient-text",
    "grid-pattern", "highlighter", "image-zoom", "light-rays", "magnetic",
    "marquee", "meteors", "noise", "orbit", "particles", "progressive-blur",
    "retro-grid", "reveal", "ripple", "scramble-text", "scroll-progress",
    "shimmer", "shiny-text", "sparkles-text", "spinning-text", "star-border",
    "text-effect", "text-shimmer", "typing-text", "word-rotate", "comparison-slider",
    "dynamic-island", "toast", "motion-presets",
)

assert len(FAMILIES) == 175
assert set(FAMILIES) == set(KIND)

arms = []
for n in FAMILIES:
    kind = KIND[n]
    if kind == "button":
        arms.append('        "button" => button_from(comp),')
    else:
        arms.append(f'        "{n}" => {kind}("{n}", comp),')

fam_list = ",\n    ".join(f'"{n}"' for n in FAMILIES)

tpl = Template(r'''//! Opt-in cronus-ui widget renderers (all $n families).
//!
//! Family = first `style` segment (`button+primary+md` -> `button`).
//! Unknown families return None so legacy dispatchers keep working.

use crate::parser::{ComponentItemNode, ComponentNode};

pub const FAMILIES: &[&str] = &[
    $fam_list,
];

/// Families with a dedicated CONTRACT renderer. Interact and stub arms must
/// not run for these (Cronus Audit K13). Do not add a family here unless
/// `dedicated_render` has a named arm.
pub const PORTED_FAMILIES: &[&str] = &[
    "button", "badge", "input", "label", "textarea", "checkbox",
    "switch", "spinner", "separator", "kbd", "toggle", "progress",
    "alert", "skeleton", "banner", "slider", "radio-group", "chip",
    "avatar", "card", "empty",
    "select", "dialog", "tabs", "accordion", "table", "pagination",
    "breadcrumb", "tooltip", "password-input", "number-input",
    "field", "input-group", "rating",
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
    "radar-chart",
    "scatter-chart",
    "ring-chart",
    "phone-input",
    "currency-input",
    "color-picker",
    "scroll-area",
    "toolbar",
    "status-dot",
    "tags-input",
    "autocomplete",
    "multi-select",
    "credit-card-input",
    "floating-label-input",
    "split-button",
    "pill-nav",
    "dock",
    "workspace-switcher",
    "app-shell",
    "table-of-contents",
    "form",
    "signature-pad",
    "resizable",
    "scheduler",
    "alert-dialog",
    "lightbox",
    "notification-center",
    "segmented-control",
    "usage-meter",
    "masonry",
    "heatmap",
    "comparison-slider",
    "code-tabs",
    "expandable-tabs",
    "live-line-chart",
    "sunburst-chart",
    "choropleth-chart",
    "profit-loss-chart",
    "scroll-progress",
    "rich-text-editor",
    "confirmation-dialog",
    "invite-dialog",
    "shimmer",
    "reveal",
    "text-shimmer",
    "particles",
    "sparkles-text",
    "noise",
    "morphing-popover",
    "bouncy-accordion",
    "typing-text",
    "word-rotate",
    "timeline",
    "tree-view",
    "spotlight-card",
    "animated-list",
    "toast",
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
        "radar-chart" => Some(crate::cronus_ui_radar_chart::render(comp)),
        "scatter-chart" => Some(crate::cronus_ui_scatter_chart::render(comp)),
        "ring-chart" => Some(crate::cronus_ui_ring_chart::render(comp)),
        "phone-input" => Some(crate::cronus_ui_phone_input::render(comp)),
        "currency-input" => Some(crate::cronus_ui_currency_input::render(comp)),
        "color-picker" => Some(crate::cronus_ui_color_picker::render(comp)),
        "scroll-area" => Some(crate::cronus_ui_scroll_area::render(comp)),
        "toolbar" => Some(crate::cronus_ui_toolbar::render(comp)),
        "status-dot" => Some(crate::cronus_ui_status_dot::render(comp)),
        "tags-input" => Some(crate::cronus_ui_tags_input::render(comp)),
        "autocomplete" => Some(crate::cronus_ui_autocomplete::render(comp)),
        "multi-select" => Some(crate::cronus_ui_multi_select::render(comp)),
        "credit-card-input" => Some(crate::cronus_ui_credit_card_input::render(comp)),
        "floating-label-input" => Some(crate::cronus_ui_floating_label_input::render(comp)),
        "split-button" => Some(crate::cronus_ui_split_button::render(comp)),
        "pill-nav" => Some(crate::cronus_ui_pill_nav::render(comp)),
        "dock" => Some(crate::cronus_ui_dock::render(comp)),
        "workspace-switcher" => Some(crate::cronus_ui_workspace_switcher::render(comp)),
        "app-shell" => Some(crate::cronus_ui_app_shell::render(comp)),
        "table-of-contents" => Some(crate::cronus_ui_table_of_contents::render(comp)),
        "form" => Some(crate::cronus_ui_form::render(comp)),
        "signature-pad" => Some(crate::cronus_ui_signature_pad::render(comp)),
        "resizable" => Some(crate::cronus_ui_resizable::render(comp)),
        "scheduler" => Some(crate::cronus_ui_scheduler::render(comp)),
        "alert-dialog" => Some(crate::cronus_ui_alert_dialog::render(comp)),
        "lightbox" => Some(crate::cronus_ui_lightbox::render(comp)),
        "notification-center" => Some(crate::cronus_ui_notification_center::render(comp)),
        "segmented-control" => Some(crate::cronus_ui_segmented_control::render(comp)),
        "usage-meter" => Some(crate::cronus_ui_usage_meter::render(comp)),
        "masonry" => Some(crate::cronus_ui_masonry::render(comp)),
        "heatmap" => Some(crate::cronus_ui_heatmap::render(comp)),
        "comparison-slider" => Some(crate::cronus_ui_comparison_slider::render(comp)),
        "code-tabs" => Some(crate::cronus_ui_code_tabs::render(comp)),
        "expandable-tabs" => Some(crate::cronus_ui_expandable_tabs::render(comp)),
        "live-line-chart" => Some(crate::cronus_ui_live_line_chart::render(comp)),
        "sunburst-chart" => Some(crate::cronus_ui_sunburst_chart::render(comp)),
        "choropleth-chart" => Some(crate::cronus_ui_choropleth_chart::render(comp)),
        "profit-loss-chart" => Some(crate::cronus_ui_profit_loss_chart::render(comp)),
        "scroll-progress" => Some(crate::cronus_ui_scroll_progress::render(comp)),
        "rich-text-editor" => Some(crate::cronus_ui_rich_text_editor::render(comp)),
        "confirmation-dialog" => Some(crate::cronus_ui_confirmation_dialog::render(comp)),
        "invite-dialog" => Some(crate::cronus_ui_invite_dialog::render(comp)),
        "shimmer" => Some(crate::cronus_ui_shimmer::render(comp)),
        "reveal" => Some(crate::cronus_ui_reveal::render(comp)),
        "text-shimmer" => Some(crate::cronus_ui_text_shimmer::render(comp)),
        "particles" => Some(crate::cronus_ui_particles::render(comp)),
        "sparkles-text" => Some(crate::cronus_ui_sparkles_text::render(comp)),
        "noise" => Some(crate::cronus_ui_noise::render(comp)),
        "morphing-popover" => Some(crate::cronus_ui_morphing_popover::render(comp)),
        "bouncy-accordion" => Some(crate::cronus_ui_bouncy_accordion::render(comp)),
        "typing-text" => Some(crate::cronus_ui_typing_text::render(comp)),
        "word-rotate" => Some(crate::cronus_ui_word_rotate::render(comp)),
        "timeline" => Some(crate::cronus_ui_timeline::render(comp)),
        "tree-view" => Some(crate::cronus_ui_tree_view::render(comp)),
        "spotlight-card" => Some(crate::cronus_ui_spotlight_card::render(comp)),
        "animated-list" => Some(crate::cronus_ui_animated_list::render(comp)),
        "toast" => Some(crate::cronus_ui_toast::render(comp)),
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
$arms
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
    comp.props.get("disabled").map(|s| s == "true").unwrap_or(false)
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
    format!(
        "<figure data-slot=\"{family}\" style=\"{BASE}{SURF}padding:1rem;\"><figcaption style=\"margin-bottom:0.5rem;font-size:0.875rem;color:var(--cronus-fg-secondary);\">{title}</figcaption><svg viewBox=\"0 0 120 40\" width=\"100%\" height=\"80\" aria-hidden=\"true\"><polyline fill=\"none\" stroke=\"var(--cronus-primary)\" stroke-width=\"2\" points=\"0,30 20,22 40,26 60,12 80,16 100,8 120,14\" /></svg></figure>"
    )
}

fn fx(family: &str, comp: &ComponentNode) -> String {
    let title = label_of(comp);
    format!(
        "<div data-slot=\"{family}\" style=\"{BASE}{SURF}padding:0.75rem 1rem;position:relative;overflow:hidden;\"><span>{title}</span></div>"
    )
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_175_unique_families() {
        assert_eq!(FAMILIES.len(), 175);
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
}
''')

out = Path(__file__).resolve().parents[1] / "src" / "cronus_ui_widgets.rs"
out.write_text(
    tpl.substitute(n=len(FAMILIES), fam_list=fam_list, arms="\n".join(arms)),
    encoding="utf-8",
)
print("wrote", out, "families", len(FAMILIES))
