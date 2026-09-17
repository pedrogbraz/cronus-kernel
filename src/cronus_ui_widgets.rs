//! Opt-in cronus-ui widget renderers (all 188 families).
//!
//! Family = first `style` segment (`button+primary+md` -> `button`).
//! Unknown families return None so legacy dispatchers keep working.

use crate::cronus_ui_kit::{esc, flag_any};
use crate::parser::{ComponentItemNode, ComponentNode};

type RenderFn = fn(&ComponentNode) -> String;
type StubFn = fn(&str, &ComponentNode) -> String;

/// How a family renders. The single source for [`FAMILIES`],
/// [`PORTED_FAMILIES`] and `cli::stub_renderer_gate`.
#[derive(Clone, Copy)]
pub enum Renderer {
    /// Dedicated CONTRACT renderer: (function path reported by audits, function).
    Dedicated(&'static str, RenderFn),
    /// Intentional placeholder: (catalog kind, generic renderer).
    Stub(&'static str, StubFn),
}

macro_rules! dedicated {
    ($family:literal, $module:ident) => {
        (
            $family,
            Renderer::Dedicated(
                concat!(stringify!($module), "::render"),
                crate::$module::render,
            ),
        )
    };
}

/// Every registered family, in registry order.
pub const FAMILY_TABLE: &[(&str, Renderer)] = &[
    dedicated!("accordion", cronus_ui_accordion),
    dedicated!("alert", cronus_ui_alert),
    dedicated!("alert-dialog", cronus_ui_alert_dialog),
    dedicated!("animated-button", cronus_ui_animated_button),
    dedicated!("animated-checkbox", cronus_ui_animated_checkbox),
    dedicated!("animated-list", cronus_ui_animated_list),
    dedicated!("animated-number", cronus_ui_animated_number),
    dedicated!("app-shell", cronus_ui_app_shell),
    dedicated!("area-chart", cronus_ui_area_chart),
    dedicated!("aspect-ratio", cronus_ui_aspect_ratio),
    dedicated!("aurora-background", cronus_ui_aurora_background),
    dedicated!("author-tooltip", cronus_ui_author_tooltip),
    dedicated!("autocomplete", cronus_ui_autocomplete),
    dedicated!("avatar", cronus_ui_avatar),
    dedicated!("avatar-group", cronus_ui_avatar_group),
    dedicated!("badge", cronus_ui_badge),
    dedicated!("banner", cronus_ui_banner),
    dedicated!("bar-chart", cronus_ui_bar_chart),
    dedicated!("border-beam", cronus_ui_border_beam),
    dedicated!("bouncy-accordion", cronus_ui_bouncy_accordion),
    dedicated!("breadcrumb", cronus_ui_breadcrumb),
    ("button", Renderer::Dedicated("button_from", button_from)),
    dedicated!("button-group", cronus_ui_button_group),
    dedicated!("calendar", cronus_ui_calendar),
    dedicated!("candlestick-chart", cronus_ui_candlestick_chart),
    dedicated!("card", cronus_ui_card),
    dedicated!("card-stack", cronus_ui_card_stack),
    dedicated!("carousel", cronus_ui_carousel),
    dedicated!("chart", cronus_ui_chart),
    dedicated!("checkbox", cronus_ui_checkbox),
    dedicated!("chip", cronus_ui_chip),
    dedicated!("choropleth-chart", cronus_ui_choropleth_chart),
    dedicated!("click-spark", cronus_ui_click_spark),
    dedicated!("code-block", cronus_ui_code_block),
    dedicated!("code-tabs", cronus_ui_code_tabs),
    dedicated!("collapsible", cronus_ui_collapsible),
    dedicated!("color-picker", cronus_ui_color_picker),
    dedicated!("combobox", cronus_ui_combobox),
    dedicated!("command", cronus_ui_command),
    dedicated!("comparison-slider", cronus_ui_comparison_slider),
    dedicated!(
        "component-preview-tooltip",
        cronus_ui_component_preview_tooltip
    ),
    dedicated!("composed-chart", cronus_ui_composed_chart),
    dedicated!("confetti", cronus_ui_confetti),
    dedicated!("confirmation-dialog", cronus_ui_confirmation_dialog),
    dedicated!("context-menu", cronus_ui_context_menu),
    dedicated!("copy-button", cronus_ui_copy_button),
    dedicated!("countdown", cronus_ui_countdown),
    dedicated!("credit-card-input", cronus_ui_credit_card_input),
    dedicated!("currency-input", cronus_ui_currency_input),
    dedicated!("data-table", cronus_ui_data_table),
    dedicated!("date-picker", cronus_ui_date_picker),
    dedicated!("date-range-picker", cronus_ui_date_range_picker),
    dedicated!("description-list", cronus_ui_description_list),
    dedicated!("dialog", cronus_ui_dialog),
    dedicated!("dock", cronus_ui_dock),
    dedicated!("dot-pattern", cronus_ui_dot_pattern),
    dedicated!("drawer", cronus_ui_drawer),
    dedicated!("dropdown-menu", cronus_ui_dropdown_menu),
    dedicated!("dynamic-island", cronus_ui_dynamic_island),
    dedicated!("empty", cronus_ui_empty),
    dedicated!("expandable-tabs", cronus_ui_expandable_tabs),
    dedicated!("explore-nav", cronus_ui_explore_nav),
    dedicated!("fab", cronus_ui_fab),
    dedicated!("family-wallet", cronus_ui_family_wallet),
    dedicated!("field", cronus_ui_field),
    dedicated!("file-dropzone", cronus_ui_file_dropzone),
    dedicated!("flickering-grid", cronus_ui_flickering_grid),
    dedicated!("flip-card", cronus_ui_flip_card),
    dedicated!("floating-label-input", cronus_ui_floating_label_input),
    dedicated!("form", cronus_ui_form),
    dedicated!("frame", cronus_ui_frame),
    dedicated!("funnel-chart", cronus_ui_funnel_chart),
    dedicated!("gauge-chart", cronus_ui_gauge_chart),
    dedicated!("glare-hover", cronus_ui_glare_hover),
    dedicated!("glass-card", cronus_ui_glass_card),
    dedicated!("globe-3d", cronus_ui_globe_3d),
    dedicated!("globe-wireframe", cronus_ui_globe_wireframe),
    dedicated!("goal-card", cronus_ui_goal_card),
    dedicated!("gradient-border", cronus_ui_gradient_border),
    dedicated!("gradient-text", cronus_ui_gradient_text),
    dedicated!("grid-pattern", cronus_ui_grid_pattern),
    dedicated!("heatmap", cronus_ui_heatmap),
    dedicated!("heatmap-chart", cronus_ui_heatmap_chart),
    dedicated!("highlighter", cronus_ui_highlighter),
    dedicated!("hover-card", cronus_ui_hover_card),
    dedicated!("image-zoom", cronus_ui_image_zoom),
    dedicated!("images-badge", cronus_ui_images_badge),
    dedicated!("input", cronus_ui_input),
    dedicated!("input-group", cronus_ui_input_group),
    dedicated!("input-otp", cronus_ui_input_otp),
    dedicated!("invite-dialog", cronus_ui_invite_dialog),
    dedicated!("json-viewer", cronus_ui_json_viewer),
    dedicated!("kanban", cronus_ui_kanban),
    dedicated!("kbd", cronus_ui_kbd),
    dedicated!("label", cronus_ui_label),
    dedicated!("light-rays", cronus_ui_light_rays),
    dedicated!("lightbox", cronus_ui_lightbox),
    dedicated!("line-chart", cronus_ui_line_chart),
    dedicated!("link-preview", cronus_ui_link_preview),
    dedicated!("live-line-chart", cronus_ui_live_line_chart),
    dedicated!("loader", cronus_ui_loader),
    dedicated!("logo-carousel", cronus_ui_logo_carousel),
    dedicated!("magnetic", cronus_ui_magnetic),
    dedicated!("marquee", cronus_ui_marquee),
    dedicated!("masonry", cronus_ui_masonry),
    dedicated!("menubar", cronus_ui_menubar),
    dedicated!("meteors", cronus_ui_meteors),
    dedicated!("metric", cronus_ui_metric),
    dedicated!("mode-toggle", cronus_ui_mode_toggle),
    dedicated!("morphing-popover", cronus_ui_morphing_popover),
    dedicated!("multi-select", cronus_ui_multi_select),
    dedicated!("navigation-menu", cronus_ui_navigation_menu),
    dedicated!("noise", cronus_ui_noise),
    dedicated!("notification-center", cronus_ui_notification_center),
    dedicated!("number-flow", cronus_ui_number_flow),
    dedicated!("number-input", cronus_ui_number_input),
    dedicated!("orbit", cronus_ui_orbit),
    dedicated!("pagination", cronus_ui_pagination),
    dedicated!("particles", cronus_ui_particles),
    dedicated!("password-input", cronus_ui_password_input),
    dedicated!("phone-input", cronus_ui_phone_input),
    dedicated!("pie-chart", cronus_ui_pie_chart),
    dedicated!("pill-nav", cronus_ui_pill_nav),
    dedicated!("popover", cronus_ui_popover),
    dedicated!("profit-loss-chart", cronus_ui_profit_loss_chart),
    dedicated!("progress", cronus_ui_progress),
    dedicated!("progressive-blur", cronus_ui_progressive_blur),
    dedicated!("radar-chart", cronus_ui_radar_chart),
    dedicated!("radio-group", cronus_ui_radio_group),
    dedicated!("rating", cronus_ui_rating),
    dedicated!("receive-button", cronus_ui_receive_button),
    dedicated!("resizable", cronus_ui_resizable),
    dedicated!("retro-grid", cronus_ui_retro_grid),
    dedicated!("reveal", cronus_ui_reveal),
    dedicated!("rich-text-editor", cronus_ui_rich_text_editor),
    dedicated!("ring-chart", cronus_ui_ring_chart),
    dedicated!("ripple", cronus_ui_ripple),
    dedicated!("sankey-chart", cronus_ui_sankey_chart),
    dedicated!("scatter-chart", cronus_ui_scatter_chart),
    dedicated!("scheduler", cronus_ui_scheduler),
    dedicated!("scramble-text", cronus_ui_scramble_text),
    dedicated!("scroll-area", cronus_ui_scroll_area),
    dedicated!("scroll-nav", cronus_ui_scroll_nav),
    dedicated!("scroll-progress", cronus_ui_scroll_progress),
    dedicated!("segmented-control", cronus_ui_segmented_control),
    dedicated!("select", cronus_ui_select),
    dedicated!("separator", cronus_ui_separator),
    dedicated!("sheet", cronus_ui_sheet),
    dedicated!("shimmer", cronus_ui_shimmer),
    dedicated!("shiny-text", cronus_ui_shiny_text),
    dedicated!("sidebar", cronus_ui_sidebar),
    dedicated!("signature-pad", cronus_ui_signature_pad),
    dedicated!("skeleton", cronus_ui_skeleton),
    dedicated!("slide-up-text", cronus_ui_slide_up_text),
    dedicated!("slider", cronus_ui_slider),
    dedicated!("sonner", cronus_ui_sonner),
    dedicated!("sparkles-text", cronus_ui_sparkles_text),
    dedicated!("sparkline", cronus_ui_sparkline),
    dedicated!("spinner", cronus_ui_spinner),
    dedicated!("spinning-text", cronus_ui_spinning_text),
    dedicated!("split-button", cronus_ui_split_button),
    dedicated!("spotlight-card", cronus_ui_spotlight_card),
    dedicated!("star-border", cronus_ui_star_border),
    dedicated!("status-dot", cronus_ui_status_dot),
    dedicated!("stepper", cronus_ui_stepper),
    dedicated!("sunburst-chart", cronus_ui_sunburst_chart),
    dedicated!("switch", cronus_ui_switch),
    dedicated!("table", cronus_ui_table),
    dedicated!("table-of-contents", cronus_ui_table_of_contents),
    dedicated!("tabs", cronus_ui_tabs),
    dedicated!("tags-input", cronus_ui_tags_input),
    dedicated!("terminal", cronus_ui_terminal),
    dedicated!("text-effect", cronus_ui_text_effect),
    dedicated!("text-shimmer", cronus_ui_text_shimmer),
    dedicated!("textarea", cronus_ui_textarea),
    dedicated!("tilt-card", cronus_ui_tilt_card),
    dedicated!("time-picker", cronus_ui_time_picker),
    dedicated!("timeline", cronus_ui_timeline),
    dedicated!("todo-item", cronus_ui_todo_item),
    dedicated!("toggle", cronus_ui_toggle),
    dedicated!("toggle-group", cronus_ui_toggle_group),
    dedicated!("token-swap", cronus_ui_token_swap),
    dedicated!("toolbar", cronus_ui_toolbar),
    dedicated!("tooltip", cronus_ui_tooltip),
    dedicated!("tree-view", cronus_ui_tree_view),
    dedicated!("typing-text", cronus_ui_typing_text),
    dedicated!("usage-meter", cronus_ui_usage_meter),
    dedicated!("video-player", cronus_ui_video_player),
    dedicated!("word-rotate", cronus_ui_word_rotate),
    dedicated!("workspace-switcher", cronus_ui_workspace_switcher),
    dedicated!("toast", cronus_ui_toast),
    dedicated!("motion-presets", cronus_ui_motion_presets),
    // Sprint 5 C2 — AI suite (alphabetical).
    dedicated!("conversation", cronus_ui_conversation),
    dedicated!("inline-citation", cronus_ui_inline_citation),
    dedicated!("message", cronus_ui_message),
    dedicated!("prompt-input", cronus_ui_prompt_input),
    dedicated!("reasoning", cronus_ui_reasoning),
    dedicated!("sources", cronus_ui_sources),
    dedicated!("suggestion", cronus_ui_suggestion),
    dedicated!("tool", cronus_ui_tool),
];

const FAMILY_COUNT: usize = FAMILY_TABLE.len();
const PORTED_COUNT: usize = count_dedicated();

const fn count_dedicated() -> usize {
    let mut n = 0;
    let mut i = 0;
    while i < FAMILY_COUNT {
        if matches!(FAMILY_TABLE[i].1, Renderer::Dedicated(..)) {
            n += 1;
        }
        i += 1;
    }
    n
}

const fn family_names() -> [&'static str; FAMILY_COUNT] {
    let mut out = [""; FAMILY_COUNT];
    let mut i = 0;
    while i < FAMILY_COUNT {
        out[i] = FAMILY_TABLE[i].0;
        i += 1;
    }
    out
}

const fn ported_names() -> [&'static str; PORTED_COUNT] {
    let mut out = [""; PORTED_COUNT];
    let mut n = 0;
    let mut i = 0;
    while i < FAMILY_COUNT {
        if matches!(FAMILY_TABLE[i].1, Renderer::Dedicated(..)) {
            out[n] = FAMILY_TABLE[i].0;
            n += 1;
        }
        i += 1;
    }
    out
}

pub const FAMILIES: &[&str] = &family_names();

/// Families with a dedicated CONTRACT renderer. Stub renderers must not run
/// for these (Cronus Audit K13).
pub const PORTED_FAMILIES: &[&str] = &ported_names();

pub fn renderer_of(family: &str) -> Option<Renderer> {
    FAMILY_TABLE
        .iter()
        .find(|(name, _)| *name == family)
        .map(|(_, renderer)| *renderer)
}

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
    match renderer_of(family)? {
        Renderer::Dedicated(_, render) => {
            note_css_usage(family);
            Some(render(comp))
        }
        Renderer::Stub(..) => None,
    }
}

pub fn render(comp: &ComponentNode) -> Option<String> {
    let family = family_of(comp)?;
    let renderer = renderer_of(family)?;
    note_css_usage(family);
    Some(match renderer {
        Renderer::Dedicated(_, render) => render(comp),
        Renderer::Stub(_, render) => render(family, comp),
    })
}

/// Per-page CSS registry hook: the layout ships this family's stylesheet
/// (`cronus_ui_css::note_family`). Kept out of the table logic above.
fn note_css_usage(family: &str) {
    crate::cronus_ui_css::note_family(family);
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

fn variant(comp: &ComponentNode) -> String {
    comp.props.get("variant").cloned().unwrap_or_else(|| {
        let style = comp.style.as_deref().unwrap_or("");
        for v in [
            "destructive",
            "danger",
            "secondary",
            "outline",
            "ghost",
            "link",
            "primary",
        ] {
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

fn href(comp: &ComponentNode) -> Option<&str> {
    comp.items.iter().find_map(|i| i.link.as_deref())
}

/// `label` is the text; `icon:"download"` / `icon-end:"arrow-right"` put a
/// lucide glyph before / after it (React children order); `loading:true`
/// puts a `sm` Spinner first, like the docs' `<Spinner size="sm" aria-hidden />`.
/// Icon-only sizes (`icon`, `icon-sm`) drop the text and label the control.
fn button_from(comp: &ComponentNode) -> String {
    let size = size(comp);
    let text = item(comp, "label").unwrap_or(comp.name.as_str());
    let icon_only = size.starts_with("icon");
    let mut inner = String::new();
    if flag_any(comp, "loading") {
        inner.push_str(&crate::cronus_ui_spinner::glyph("sm", true));
    }
    if let Some(icon) = crate::cronus_ui_kit::attr_nonempty(comp, "icon") {
        inner.push_str(&crate::cronus_ui_icons::svg_or_empty(icon));
    }
    if !icon_only {
        inner.push_str(&esc(text));
    }
    if let Some(icon) = crate::cronus_ui_kit::attr_nonempty(comp, "icon-end") {
        inner.push_str(&crate::cronus_ui_icons::svg_or_empty(icon));
    }
    let aria = crate::cronus_ui_kit::attr_nonempty(comp, "aria-label").or(if icon_only {
        Some(text)
    } else {
        None
    });
    crate::cronus_ui::button_html(
        &inner,
        &variant(comp),
        &size,
        href(comp),
        flag_any(comp, "disabled"),
        aria,
    )
}

const BASE: &str =
    "color:var(--cronus-fg);font-family:var(--cronus-font-sans,inherit);box-sizing:border-box;";
const SURF: &str = "background:var(--cronus-surface-raised);border:1px solid var(--cronus-border);border-radius:var(--cronus-radius,14px);";

fn chart(family: &str, comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let rows = crate::cronus_ui_data::rows();
    let bars = if rows.len() >= 2 {
        let vals: Vec<f64> = rows
            .iter()
            .filter_map(|r| {
                r.get("value").or_else(|| r.get("amount")).and_then(|v| {
                    v.as_f64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })
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
        format!(
            "<div style=\"display:flex;align-items:flex-end;gap:4px;height:80px;\">{bars}</div>"
        )
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
        span: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_180_unique_families() {
        assert_eq!(FAMILIES.len(), 188);
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
                || html.contains(&format!("data-slot=\"{family}-panel-group\""))
                || html.contains("data-slot=\"button\"")
                // wave1t: React <Toaster /> roots at data-slot="toaster" (no sonner slot).
                || (*family == "sonner" && html.contains("data-slot=\"toaster\""));
            assert!(slot_ok, "{family} missing data-slot: {html}");
            assert!(!html.contains("zinc-"), "{family} used zinc palette");
            assert!(!html.contains("amber-500"), "{family} used amber palette");
            assert!(
                !html.contains("bg-neutral-"),
                "{family} used neutral palette scale"
            );
        }
    }

    #[test]
    fn family_table_derives_lists() {
        assert_eq!(FAMILIES.len(), FAMILY_TABLE.len());
        let stubs: Vec<&str> = FAMILY_TABLE
            .iter()
            .filter(|(_, r)| matches!(r, Renderer::Stub(..)))
            .map(|(f, _)| *f)
            .collect();
        assert!(stubs.is_empty(), "unexpected stubs: {stubs:?}");
        assert_eq!(PORTED_FAMILIES.len(), FAMILIES.len() - stubs.len());
        for family in FAMILIES {
            assert_eq!(
                PORTED_FAMILIES.contains(family),
                !stubs.contains(family),
                "{family}"
            );
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
            ("select", "role=\"combobox\""),
            ("dialog", "role=\"dialog\""),
            ("accordion", "type=\"radio\""),
            ("tabs", "type=\"radio\""),
            ("table", "<table"),
            ("progress", "role=\"progressbar\""),
            ("slider", "role=\"slider\""),
            ("radio-group", "role=\"radiogroup\""),
        ];
        for (family, needle) in cases {
            let html = render(&stub(family)).expect(family);
            assert!(html.contains(needle), "{family} missing {needle}: {html}");
            assert!(!html.contains("{ value }"), "{family} leaked voodoo interp");
            assert!(
                !html.contains("v-data="),
                "{family} leaked v-data without opt-in"
            );
            assert!(
                !html.contains("v-model="),
                "{family} leaked v-model without opt-in"
            );
            assert!(!html.contains("zinc-"), "{family} used zinc palette");
        }
    }

    #[test]
    fn voodoo_attrs_only_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            // Dedicated renderers are zero-JS even with the runtime on.
            assert!(!render(&stub("select")).unwrap().contains("v-data="));
            let meter = render(&stub("scroll-progress")).unwrap();
            assert!(!meter.contains("{ value }"), "{meter}");
            assert!(!meter.contains("v-data="), "{meter}");
            assert!(
                meter.contains("data-slot=\"scroll-progress-fill\""),
                "{meter}"
            );
            let usage = render(&stub("usage-meter")).unwrap();
            assert!(!usage.contains("{ value }"), "{usage}");
            assert!(!usage.contains("v-data="), "{usage}");
            assert!(usage.contains("data-slot=\"usage-meter-fill\""), "{usage}");
            let tabs = render(&stub("tabs")).unwrap();
            assert!(
                tabs.contains("type=\"radio\""),
                "tabs switch with native radios"
            );
            assert!(!tabs.contains("onclick="), "tabs are zero-JS");
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
    fn form_skips_vsubmit_even_when_voodoo_and_entity() {
        use crate::binding::ResolvedData;
        crate::voodoo::with_enabled(true, || {
            crate::cronus_ui_data::with_binding("Lead", &ResolvedData::None, || {
                let html = render(&stub("form")).unwrap();
                assert!(!html.contains("v-submit="), "{html}");
                assert!(!html.contains("v-method="), "{html}");
                assert!(html.contains("data-slot=\"form-item\""), "{html}");
                assert!(html.contains("<label data-slot=\"label\""), "{html}");
            });
        });
        let off = render(&stub("form")).unwrap();
        assert!(!off.contains("v-submit="), "{off}");
        assert!(off.contains("data-slot=\"form-item\""), "{off}");
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
        assert!(tabs.contains("role=\"radiogroup\""));
        assert!(tabs.contains("aria-label=\"Demo\""));
        let dlg = render(&stub("dialog")).unwrap();
        assert!(dlg.contains("role=\"dialog\""));
        assert!(!dlg.contains("<dialog"));
        let toast = render(&stub("toast")).unwrap();
        assert!(toast.contains("aria-live"));
    }

    #[test]
    fn ported_family_skips_interact() {
        for family in PORTED_FAMILIES {
            let html = render(&stub(family)).expect(family);
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
        let comp = nodes
            .iter()
            .find_map(|n| match n {
                crate::parser::AstNode::Component(c) => Some(c),
                _ => None,
            })
            .expect("component");
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
                    RendererKind::Dedicated(_) => {}
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
        assert!(html.contains("data-slot=\"catalog\""), "{html}");
        assert!(html.contains("data-slot=\"catalog-specimen\""), "{html}");
        assert!(html.contains("id=\"buttons\""));
        assert!(html.contains("id=\"forms\""));
        assert!(html.contains("Save"), "{html}");
        assert!(
            html.contains("Email") || html.contains("you@cooud.app"),
            "{html}"
        );
        for family in [
            "button",
            "input",
            "dialog-content",
            "tabs",
            "select-trigger",
        ] {
            assert!(
                html.contains(&format!("data-slot=\"{family}\"")),
                "missing {family}"
            );
        }
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"catalog\"]"));
        assert!(
            css.contains("--cronus-")
                || crate::cronus_ui::token_css("aurora", "dark").contains("--cronus-")
        );
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
        // Closed Radix trigger: the field label is the placeholder text; the
        // `item` options never become the value or the accessible name. They
        // only exist as radios inside the closed native popover.
        let html = render(&catalog_component("Plan")).expect("select");
        let (trigger, popup) = html.split_once("</button>").expect("trigger");
        assert!(
            trigger.contains("aria-label=\"Plan\"") && trigger.ends_with(">Plan</span><svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m6 9 6 6 6-6\"></path></svg>"),
            "Plan must be the trigger placeholder: {html}"
        );
        assert!(trigger.contains("data-slot=\"select-trigger\""), "{html}");
        assert!(popup.contains("data-slot=\"select-content\""), "{html}");
        assert!(
            popup.contains("value=\"Free\"") && popup.contains("value=\"Pro\""),
            "{html}"
        );
        assert!(!html.contains("value=\"Plan\""), "{html}");
    }

    #[test]
    fn catalog_radio_items_are_items_not_the_field_label() {
        let html = render(&catalog_component("PlanRadio")).expect("radio-group");
        assert!(
            html.contains("role=\"radiogroup\"")
                && html.contains("data-slot=\"radio-group\" aria-label=\"Plan\""),
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
