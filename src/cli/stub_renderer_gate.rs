//! Stub + interact gate. A family is `ported` only when the dispatch arm
//! calls a dedicated renderer — not `fx`/`pill`/interact generic HTML.

use crate::cli::audit_codes::{AuditFinding, STUB_RENDERER};
use crate::cronus_ui_widgets::{self, PORTED_FAMILIES};
use crate::parser::AstNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererKind {
    Dedicated(&'static str),
    Interact,
    Stub(&'static str),
    Missing,
}

pub fn dedicated_fn_name(family: &str) -> Option<&'static str> {
    match family {
        "button" => Some("button_from"),
        "badge" => Some("cronus_ui_badge::render"),
        "input" => Some("cronus_ui_input::render"),
        "label" => Some("cronus_ui_label::render"),
        "textarea" => Some("cronus_ui_textarea::render"),
        "checkbox" => Some("cronus_ui_checkbox::render"),
        "switch" => Some("cronus_ui_switch::render"),
        "spinner" => Some("cronus_ui_spinner::render"),
        "separator" => Some("cronus_ui_separator::render"),
        "kbd" => Some("cronus_ui_kbd::render"),
        "toggle" => Some("cronus_ui_toggle::render"),
        "progress" => Some("cronus_ui_progress::render"),
        "alert" => Some("cronus_ui_alert::render"),
        "skeleton" => Some("cronus_ui_skeleton::render"),
        "banner" => Some("cronus_ui_banner::render"),
        "slider" => Some("cronus_ui_slider::render"),
        "radio-group" => Some("cronus_ui_radio_group::render"),
        "chip" => Some("cronus_ui_chip::render"),
        "avatar" => Some("cronus_ui_avatar::render"),
        "card" => Some("cronus_ui_card::render"),
        "empty" => Some("cronus_ui_empty::render"),
        "select" => Some("cronus_ui_select::render"),
        "dialog" => Some("cronus_ui_dialog::render"),
        "tabs" => Some("cronus_ui_tabs::render"),
        "accordion" => Some("cronus_ui_accordion::render"),
        "table" => Some("cronus_ui_table::render"),
        "pagination" => Some("cronus_ui_pagination::render"),
        "breadcrumb" => Some("cronus_ui_breadcrumb::render"),
        "tooltip" => Some("cronus_ui_tooltip::render"),
        "password-input" => Some("cronus_ui_password_input::render"),
        "number-input" => Some("cronus_ui_number_input::render"),
        "field" => Some("cronus_ui_field::render"),
        "input-group" => Some("cronus_ui_input_group::render"),
        "rating" => Some("cronus_ui_rating::render"),
        "copy-button" => Some("cronus_ui_copy_button::render"),
        "fab" => Some("cronus_ui_fab::render"),
        "toggle-group" => Some("cronus_ui_toggle_group::render"),
        "metric" => Some("cronus_ui_metric::render"),
        "avatar-group" => Some("cronus_ui_avatar_group::render"),
        "button-group" => Some("cronus_ui_button_group::render"),
        "combobox" => Some("cronus_ui_combobox::render"),
        "stepper" => Some("cronus_ui_stepper::render"),
        "input-otp" => Some("cronus_ui_input_otp::render"),
        "file-dropzone" => Some("cronus_ui_file_dropzone::render"),
        "popover" => Some("cronus_ui_popover::render"),
        "hover-card" => Some("cronus_ui_hover_card::render"),
        "dropdown-menu" => Some("cronus_ui_dropdown_menu::render"),
        "collapsible" => Some("cronus_ui_collapsible::render"),
        "mode-toggle" => Some("cronus_ui_mode_toggle::render"),
        "command" => Some("cronus_ui_command::render"),
        "menubar" => Some("cronus_ui_menubar::render"),
        "context-menu" => Some("cronus_ui_context_menu::render"),
        "drawer" => Some("cronus_ui_drawer::render"),
        "sheet" => Some("cronus_ui_sheet::render"),
        "calendar" => Some("cronus_ui_calendar::render"),
        "date-picker" => Some("cronus_ui_date_picker::render"),
        "time-picker" => Some("cronus_ui_time_picker::render"),
        "date-range-picker" => Some("cronus_ui_date_range_picker::render"),
        "area-chart" => Some("cronus_ui_area_chart::render"),
        "bar-chart" => Some("cronus_ui_bar_chart::render"),
        "line-chart" => Some("cronus_ui_line_chart::render"),
        "sparkline" => Some("cronus_ui_sparkline::render"),
        "pie-chart" => Some("cronus_ui_pie_chart::render"),
        "data-table" => Some("cronus_ui_data_table::render"),
        "sidebar" => Some("cronus_ui_sidebar::render"),
        "sonner" => Some("cronus_ui_sonner::render"),
        "navigation-menu" => Some("cronus_ui_navigation_menu::render"),
        "radar-chart" => Some("cronus_ui_radar_chart::render"),
        "scatter-chart" => Some("cronus_ui_scatter_chart::render"),
        "ring-chart" => Some("cronus_ui_ring_chart::render"),
        "phone-input" => Some("cronus_ui_phone_input::render"),
        "currency-input" => Some("cronus_ui_currency_input::render"),
        "color-picker" => Some("cronus_ui_color_picker::render"),
        "scroll-area" => Some("cronus_ui_scroll_area::render"),
        "toolbar" => Some("cronus_ui_toolbar::render"),
        "status-dot" => Some("cronus_ui_status_dot::render"),
        "tags-input" => Some("cronus_ui_tags_input::render"),
        "autocomplete" => Some("cronus_ui_autocomplete::render"),
        "multi-select" => Some("cronus_ui_multi_select::render"),
        "credit-card-input" => Some("cronus_ui_credit_card_input::render"),
        "floating-label-input" => Some("cronus_ui_floating_label_input::render"),
        "split-button" => Some("cronus_ui_split_button::render"),
        "pill-nav" => Some("cronus_ui_pill_nav::render"),
        "dock" => Some("cronus_ui_dock::render"),
        "workspace-switcher" => Some("cronus_ui_workspace_switcher::render"),
        "app-shell" => Some("cronus_ui_app_shell::render"),
        "table-of-contents" => Some("cronus_ui_table_of_contents::render"),
        "form" => Some("cronus_ui_form::render"),
        "signature-pad" => Some("cronus_ui_signature_pad::render"),
        "resizable" => Some("cronus_ui_resizable::render"),
        "scheduler" => Some("cronus_ui_scheduler::render"),
        "alert-dialog" => Some("cronus_ui_alert_dialog::render"),
        "lightbox" => Some("cronus_ui_lightbox::render"),
        "notification-center" => Some("cronus_ui_notification_center::render"),
        "segmented-control" => Some("cronus_ui_segmented_control::render"),
        "usage-meter" => Some("cronus_ui_usage_meter::render"),
        "masonry" => Some("cronus_ui_masonry::render"),
        "heatmap" => Some("cronus_ui_heatmap::render"),
        "comparison-slider" => Some("cronus_ui_comparison_slider::render"),
        "code-tabs" => Some("cronus_ui_code_tabs::render"),
        "expandable-tabs" => Some("cronus_ui_expandable_tabs::render"),
        "live-line-chart" => Some("cronus_ui_live_line_chart::render"),
        "sunburst-chart" => Some("cronus_ui_sunburst_chart::render"),
        "choropleth-chart" => Some("cronus_ui_choropleth_chart::render"),
        "profit-loss-chart" => Some("cronus_ui_profit_loss_chart::render"),
        "scroll-progress" => Some("cronus_ui_scroll_progress::render"),
        "rich-text-editor" => Some("cronus_ui_rich_text_editor::render"),
        "confirmation-dialog" => Some("cronus_ui_confirmation_dialog::render"),
        "invite-dialog" => Some("cronus_ui_invite_dialog::render"),
        "shimmer" => Some("cronus_ui_shimmer::render"),
        "reveal" => Some("cronus_ui_reveal::render"),
        "text-shimmer" => Some("cronus_ui_text_shimmer::render"),
        "particles" => Some("cronus_ui_particles::render"),
        "sparkles-text" => Some("cronus_ui_sparkles_text::render"),
        "noise" => Some("cronus_ui_noise::render"),
        "morphing-popover" => Some("cronus_ui_morphing_popover::render"),
        "bouncy-accordion" => Some("cronus_ui_bouncy_accordion::render"),
        "typing-text" => Some("cronus_ui_typing_text::render"),
        _ => None,
    }
}

pub fn renderer_kind(family: &str) -> RendererKind {
    if PORTED_FAMILIES.contains(&family) {
        return match dedicated_fn_name(family) {
            Some(name) => RendererKind::Dedicated(name),
            None => RendererKind::Missing,
        };
    }
    if crate::cronus_ui_interact::covers(family) {
        return RendererKind::Interact;
    }
    match catalog_stub_kind(family) {
        Some(kind) => RendererKind::Stub(kind),
        None => RendererKind::Missing,
    }
}

/// Catalog kind from the generator table (pill/field/overlay/nav/display/chart/fx).
/// Not a proof of port — interact still wins for unported families.
pub fn catalog_stub_kind(family: &str) -> Option<&'static str> {
    match family {
        "button" => Some("button"),
        "badge" | "chip" | "kbd" | "status-dot" | "spinner" | "skeleton" | "separator"
        | "label" | "progress" | "slider" | "rating" | "copy-button" | "fab" | "toggle"
        | "switch" | "checkbox" | "radio-group" | "toggle-group" | "segmented-control" => {
            Some("pill")
        }
        "input" | "textarea" | "password-input" | "number-input" | "phone-input"
        | "currency-input" | "credit-card-input" | "floating-label-input" | "tags-input"
        | "input-otp" | "input-group" | "color-picker" | "file-dropzone" | "select"
        | "combobox" | "autocomplete" | "multi-select" | "date-picker" | "date-range-picker"
        | "time-picker" | "field" | "form" | "signature-pad" | "rich-text-editor" => Some("field"),
        "dialog" | "alert-dialog" | "confirmation-dialog" | "invite-dialog" | "sheet"
        | "drawer" | "popover" | "hover-card" | "tooltip" | "dropdown-menu" | "context-menu"
        | "menubar" | "command" | "lightbox" | "morphing-popover" | "notification-center"
        | "sonner" => Some("overlay"),
        "tabs" | "accordion" | "bouncy-accordion" | "collapsible" | "breadcrumb" | "pagination" | "sidebar"
        | "navigation-menu" | "pill-nav" | "expandable-tabs" | "stepper" | "toolbar"
        | "table-of-contents" | "app-shell" | "workspace-switcher" | "dock" | "mode-toggle"
        | "split-button" | "button-group" => Some("nav"),
        "card" | "card-stack" | "glass-card" | "spotlight-card" | "tilt-card" | "flip-card"
        | "banner" | "empty" | "alert" | "metric" | "description-list" | "table"
        | "data-table" | "avatar" | "avatar-group" | "aspect-ratio" | "frame" | "scroll-area"
        | "resizable" | "calendar" | "timeline" | "kanban" | "scheduler" | "json-viewer"
        | "code-block" | "code-tabs" | "terminal" | "tree-view" | "carousel"
        | "logo-carousel" | "masonry" | "usage-meter" | "video-player" => Some("display"),
        "area-chart" | "bar-chart" | "line-chart" | "pie-chart" | "radar-chart"
        | "composed-chart" | "candlestick-chart" | "choropleth-chart" | "funnel-chart"
        | "gauge-chart" | "heatmap" | "heatmap-chart" | "live-line-chart"
        | "profit-loss-chart" | "ring-chart" | "sankey-chart" | "scatter-chart" | "sparkline"
        | "sunburst-chart" | "chart" => Some("chart"),
        "animated-button" | "animated-list" | "animated-number" | "aurora-background"
        | "border-beam" | "click-spark" | "confetti" | "countdown" | "dot-pattern"
        | "flickering-grid" | "glare-hover" | "gradient-border" | "gradient-text"
        | "grid-pattern" | "highlighter" | "image-zoom" | "light-rays" | "magnetic"
        | "marquee" | "meteors" | "noise" | "orbit" | "particles" | "progressive-blur"
        | "retro-grid" | "reveal" | "ripple" | "scramble-text" | "scroll-progress"
        | "shimmer" | "shiny-text" | "sparkles-text" | "spinning-text" | "star-border"
        | "text-effect" | "text-shimmer" | "typing-text" | "word-rotate" | "comparison-slider"
        | "dynamic-island" | "toast" | "motion-presets" => Some("fx"),
        _ => None,
    }
}

pub fn looks_like_interact_generic(html: &str) -> bool {
    html.contains("data-slot=\"input-control\"")
        || html.contains("<label data-slot=\"input\"")
        || html.contains("height:2.5rem;padding:0 0.75rem;border-radius:0.5rem;border:1px solid var(--cronus-border);background:var(--cronus-surface-inset")
        || html.contains("data-slot=\"switch-control\"")
        || html.contains("<label data-slot=\"switch\"")
        || (html.contains("data-slot=\"spinner\"") && html.contains("border-top-color"))
        || html.contains("<hr data-slot=\"separator\"")
        || (html.contains("data-slot=\"alert\"") && !html.contains("data-slot=\"alert-title\""))
        || html.contains("padding:0.85rem 1rem;display:flex;flex-direction:column;gap:0.25rem;font-size:0.875rem;")
        || html.contains("height:0.9rem;width:8rem;")
        || (html.contains("data-slot=\"banner\"") && !html.contains("data-slot=\"banner-title\""))
        || html.contains("data-slot=\"slider-control\"")
        || html.contains("<label data-slot=\"slider\"")
        || html.contains("<input type=\"radio\"")
        || (html.contains("data-slot=\"chip\"") && html.contains("padding:0.15rem 0.55rem"))
        || html.contains("<form data-slot=\"field\"")
        || (html.contains("data-slot=\"field\"") && !html.contains("data-slot=\"field-label\""))
        || html.contains("data-slot=\"input-group-control\"")
        || html.contains("<label data-slot=\"input-group\"")
        || (html.contains("data-slot=\"rating\"") && html.contains("role=\"radiogroup\""))
        || html.contains("navigator.clipboard.writeText")
        || (html.contains("data-slot=\"copy-button\"") && html.contains("style="))
        || (html.contains("data-slot=\"fab\"") && html.contains("data-slot=\"button\""))
        || (html.contains("data-slot=\"toggle-group\"") && html.contains("role=\"radiogroup\""))
        || html.contains("<section data-slot=\"metric\"")
        || (html.contains("data-slot=\"metric\"") && html.contains("{ value }"))
        || (html.contains("data-slot=\"avatar-group\"")
            && html.contains("width:2.25rem;height:2.25rem;border-radius:999px"))
        || (html.contains("data-slot=\"button-group\"") && html.contains("display:inline-flex;gap:0.25rem"))
        || html.contains("data-slot=\"combobox-control\"")
        || html.contains("<label data-slot=\"combobox\"")
        || (html.contains("data-slot=\"combobox") && html.contains("<select"))
        || (html.contains("data-slot=\"stepper\"") && html.contains("<ol"))
        || html.contains("<nav data-slot=\"stepper\"")
        || (html.contains("data-slot=\"stepper\"") && html.contains("style="))
        || html.contains("<fieldset data-slot=\"input-otp\"")
        || html.contains("data-slot=\"input-otp-control\"")
        || (html.contains("data-slot=\"input-otp\"") && html.contains("style="))
        || html.contains("width:2.5rem;text-align:center;font-variant-numeric:tabular-nums")
        || html.contains("data-slot=\"file-dropzone-control\"")
        || (html.contains("data-slot=\"file-dropzone\"") && html.contains("style="))
        || html.contains("<details data-slot=\"popover\"")
        || html.contains("<details data-slot=\"hover-card\"")
        || html.contains("<details data-slot=\"morphing-popover\"")
        || (html.contains("data-slot=\"morphing-popover\"") && html.contains("<details"))
        || (html.contains("data-slot=\"bouncy-accordion\"")
            && (html.contains("<details")
                || !html.contains("data-slot=\"bouncy-accordion-trigger\"")))
        || (html.contains("data-slot=\"dropdown-menu\"") && html.contains("<details"))
        || html.contains("position:absolute;z-index:20;margin-top:0.35rem")
        || (html.contains("data-slot=\"collapsible\"") && html.contains("<details"))
        || (html.contains("data-slot=\"collapsible\"")
            && !html.contains("data-slot=\"collapsible-content\""))
        || (html.contains("data-slot=\"mode-toggle\"") && html.contains("classList.toggle('dark')"))
        || (html.contains("data-slot=\"mode-toggle\"") && html.contains("style="))
        || html.contains("<details data-slot=\"command\"")
        || (html.contains("data-slot=\"command\"") && html.contains("<details"))
        || html.contains("<details data-slot=\"menubar\"")
        || (html.contains("data-slot=\"menubar\"") && html.contains("<details"))
        || html.contains("<details data-slot=\"context-menu\"")
        || (html.contains("data-slot=\"context-menu\"") && html.contains("<details"))
        || (html.contains("data-slot=\"drawer")
            && (html.contains("<dialog") || html.contains("showModal()")))
        || (html.contains("data-slot=\"sheet")
            && (html.contains("<dialog") || html.contains("showModal()")))
        || (html.contains("data-slot=\"calendar\"") && html.contains("style="))
        || html.contains("grid-template-columns:repeat(7,1fr)")
        || html.contains("data-slot=\"date-picker-control\"")
        || html.contains("<label data-slot=\"date-picker\"")
        || html.contains("<input type=\"date\"")
        || html.contains("data-slot=\"time-picker-control\"")
        || html.contains("<label data-slot=\"time-picker\"")
        || html.contains("<input type=\"time\"")
        || html.contains("<label data-slot=\"date-range-picker\"")
        || html.contains("data-slot=\"date-range-picker-control\"")
        || html.contains("<figure data-slot=\"sparkline\"")
        || (html.contains("data-slot=\"sparkline\"") && html.contains("<figcaption"))
        || html.contains("<figure data-slot=\"pie-chart\"")
        || (html.contains("data-slot=\"pie-chart\"") && html.contains("<figcaption"))
        || html.contains("<figure data-slot=\"radar-chart\"")
        || (html.contains("data-slot=\"radar-chart\"") && html.contains("<figcaption"))
        || html.contains("<figure data-slot=\"scatter-chart\"")
        || (html.contains("data-slot=\"scatter-chart\"") && html.contains("<figcaption"))
        || html.contains("<figure data-slot=\"ring-chart\"")
        || (html.contains("data-slot=\"ring-chart\"") && html.contains("<figcaption"))
        || (html.contains("data-slot=\"data-table\"")
            && html.contains("text-align:left;padding:0.5rem 0.75rem"))
        || (html.contains("data-slot=\"data-table\"") && html.contains("style="))
        || html.contains("<nav data-slot=\"sidebar\"")
        || (html.contains("data-slot=\"sidebar\"") && !html.contains("data-slot=\"sidebar-content\""))
        || (html.contains("data-slot=\"sidebar\"") && !html.contains("data-slot=\"sidebar-menu\""))
        || (html.contains("data-slot=\"sidebar\"") && html.contains("flex-wrap:wrap"))
        || html.contains("<details data-slot=\"sonner\"")
        || (html.contains("data-slot=\"sonner\"") && html.contains("<details"))
        || (html.contains("data-slot=\"sonner\"") && !html.contains("data-slot=\"toaster\""))
        || (html.contains("data-slot=\"sonner\"") && !html.contains("data-slot=\"toast\""))
        || (html.contains("data-slot=\"navigation-menu\"")
            && !html.contains("data-slot=\"navigation-menu-list\""))
        || (html.contains("data-slot=\"navigation-menu\"") && html.contains("flex-wrap:wrap"))
        || html.contains("data-slot=\"phone-input-control\"")
        || html.contains("<label data-slot=\"phone-input\"")
        || html.contains("data-slot=\"currency-input-control\"")
        || html.contains("<label data-slot=\"currency-input\"")
        || html.contains("data-slot=\"color-picker-control\"")
        || html.contains("<label data-slot=\"color-picker\"")
        || html.contains("<input type=\"color\"")
        || (html.contains("data-slot=\"scroll-area\"") && html.contains("<section"))
        || (html.contains("data-slot=\"scroll-area\"") && html.contains("max-height:12rem;overflow:auto"))
        || (html.contains("data-slot=\"scroll-area\"")
            && !html.contains("data-slot=\"scroll-area-viewport\""))
        || (html.contains("data-slot=\"toolbar\"") && html.contains("<nav"))
        || (html.contains("data-slot=\"toolbar\"")
            && !html.contains("data-slot=\"toolbar-button\""))
        || (html.contains("data-slot=\"toolbar\"") && html.contains("flex-wrap:wrap"))
        || (html.contains("data-slot=\"status-dot\"")
            && (html.contains("padding:0.15rem 0.55rem")
                || !html.contains("data-slot=\"status-dot-indicator\"")
                || !html.contains("data-slot=\"status-dot-label\"")))
        || html.contains("data-slot=\"tags-input-control\"")
        || html.contains("<label data-slot=\"tags-input\"")
        || (html.contains("data-slot=\"tags-input") && html.contains("<select"))
        || html.contains("data-slot=\"autocomplete-control\"")
        || html.contains("<label data-slot=\"autocomplete\"")
        || (html.contains("data-slot=\"autocomplete") && html.contains("<select"))
        || html.contains("data-slot=\"multi-select-control\"")
        || html.contains("<label data-slot=\"multi-select\"")
        || (html.contains("data-slot=\"multi-select") && html.contains("<select"))
        || html.contains("data-slot=\"credit-card-input-control\"")
        || html.contains("<label data-slot=\"credit-card-input\"")
        || html.contains("data-slot=\"floating-label-input-control\"")
        || html.contains("<label data-slot=\"floating-label-input\" style=")
        || (html.contains("data-slot=\"split-button\"")
            && html.contains("display:inline-flex;gap:0.25rem"))
        || (html.contains("data-slot=\"split-button\"") && html.contains("style="))
        || (html.contains("data-slot=\"split-button\"") && !html.contains("role=\"group\""))
        || (html.contains("data-slot=\"pill-nav\"") && !html.contains("data-slot=\"pill-nav-item\""))
        || (html.contains("data-slot=\"pill-nav\"") && html.contains("flex-wrap:wrap"))
        || (html.contains("data-slot=\"dock\"") && !html.contains("data-slot=\"dock-item\""))
        || (html.contains("data-slot=\"dock\"") && html.contains("flex-wrap:wrap"))
        || (html.contains("data-slot=\"workspace-switcher\"") && html.contains("<nav"))
        || (html.contains("data-slot=\"workspace-switcher\"")
            && !html.contains("data-slot=\"workspace-switcher-content\""))
        || (html.contains("data-slot=\"workspace-switcher\"") && html.contains("<details"))
        || (html.contains("data-slot=\"workspace-switcher\"") && html.contains("flex-wrap:wrap"))
        || html.contains("<nav data-slot=\"app-shell\"")
        || (html.contains("data-slot=\"app-shell\"")
            && (!html.contains("data-slot=\"app-shell-header\"")
                || !html.contains("data-slot=\"app-shell-body\"")
                || !html.contains("data-slot=\"app-shell-content\"")))
        || (html.contains("data-slot=\"app-shell\"") && html.contains("flex-wrap:wrap"))
        || (html.contains("data-slot=\"table-of-contents\"")
            && (!html.contains("data-slot=\"table-of-contents-list\"")
                || !html.contains("data-slot=\"table-of-contents-link\"")))
        || (html.contains("data-slot=\"table-of-contents\"") && html.contains("flex-wrap:wrap"))
        || (html.contains("data-slot=\"form\"") && html.contains("style="))
        || (html.contains("data-slot=\"form\"") && html.contains(">Submit</button>"))
        || (html.contains("data-slot=\"form\"") && !html.contains("data-slot=\"form-item\""))
        || (html.contains("data-slot=\"form\"") && !html.contains("data-slot=\"form-label\""))
        || (html.contains("data-slot=\"form\"") && html.contains("v-submit="))
        || (html.contains("data-slot=\"signature-pad\"")
            && !html.contains("data-slot=\"signature-pad-canvas\""))
        || (html.contains("data-slot=\"signature-pad\"") && html.contains("style="))
        || (html.contains("data-slot=\"resizable\"") && html.contains("<section"))
        || (html.contains("data-slot=\"resizable\"")
            && (!html.contains("data-slot=\"resizable-panel-group\"")
                || !html.contains("data-slot=\"resizable-handle\"")))
        || (html.contains("data-slot=\"resizable\"") && html.contains("max-height:12rem;overflow:auto"))
        || (html.contains("data-slot=\"scheduler\"")
            && (!html.contains("data-slot=\"scheduler-title\"")
                || !html.contains("data-slot=\"scheduler-grid\"")))
        || (html.contains("data-slot=\"scheduler\"")
            && html.contains("style=")
            && html.contains("grid-template-columns:repeat(7,1fr)"))
        || (html.contains("data-slot=\"alert-dialog")
            && (html.contains("<dialog") || html.contains("showModal()")))
        || (html.contains("data-slot=\"lightbox")
            && (html.contains("<dialog") || html.contains("showModal()")))
        || html.contains("<details data-slot=\"notification-center\"")
        || (html.contains("data-slot=\"notification-center\"") && html.contains("<details"))
        || (html.contains("data-slot=\"notification-center\"")
            && !html.contains("data-slot=\"notification-row\""))
        || (html.contains("data-slot=\"segmented-control\"") && html.contains("<input type=\"radio\""))
        || (html.contains("data-slot=\"segmented-control\"") && html.contains("role=\"radiogroup\""))
        || (html.contains("data-slot=\"segmented-control\"")
            && !html.contains("data-slot=\"segmented-control-item\""))
        || (html.contains("data-slot=\"usage-meter\"") && html.contains("<progress"))
        || (html.contains("data-slot=\"usage-meter\"")
            && !html.contains("data-slot=\"usage-meter-fill\""))
        || (html.contains("data-slot=\"masonry\"") && html.contains("<section"))
        || (html.contains("data-slot=\"masonry\"") && html.contains("max-height:12rem;overflow:auto"))
        || (html.contains("data-slot=\"masonry\"") && !html.contains("data-slot=\"masonry-cell\""))
        || html.contains("<figure data-slot=\"heatmap\"")
        || (html.contains("data-slot=\"heatmap\"") && html.contains("<figcaption"))
        || (html.contains("data-slot=\"heatmap\"") && !html.contains("data-slot=\"heatmap-day\""))
        || (html.contains("data-slot=\"comparison-slider\"")
            && (!html.contains("data-slot=\"comparison-before\"")
                || !html.contains("data-slot=\"comparison-after\"")
                || html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")))
        || (html.contains("data-slot=\"code-tabs\"") && !html.contains("data-slot=\"code-tabs-pre\""))
        || (html.contains("data-slot=\"code-tabs\"")
            && html.contains("role=\"tablist\"")
            && !html.contains("data-slot=\"code-tabs-list\""))
        || (html.contains("data-slot=\"expandable-tabs\"")
            && !html.contains("data-slot=\"expandable-tabs-item\""))
        || (html.contains("data-slot=\"expandable-tabs\"") && html.contains("role=\"tabpanel\""))
        || (html.contains("data-slot=\"expandable-tabs\"") && html.contains("<div role=\"tablist\""))
        || html.contains("<figure data-slot=\"live-line-chart\"")
        || (html.contains("data-slot=\"live-line-chart\"") && html.contains("<figcaption"))
        || html.contains("<figure data-slot=\"sunburst-chart\"")
        || (html.contains("data-slot=\"sunburst-chart\"") && html.contains("<figcaption"))
        || html.contains("<figure data-slot=\"choropleth-chart\"")
        || (html.contains("data-slot=\"choropleth-chart\"") && html.contains("<figcaption"))
        || html.contains("<figure data-slot=\"profit-loss-chart\"")
        || (html.contains("data-slot=\"profit-loss-chart\"") && html.contains("<figcaption"))
        || (html.contains("data-slot=\"scroll-progress\"") && html.contains("<progress"))
        || (html.contains("data-slot=\"scroll-progress\"")
            && html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
            && !html.contains("data-slot=\"scroll-progress-fill\"")
            && !html.contains("data-slot=\"scroll-progress-ring\""))
        || html.contains("data-slot=\"rich-text-editor-control\"")
        || html.contains("<label data-slot=\"rich-text-editor\"")
        || (html.contains("data-slot=\"rich-text-editor\"") && html.contains("<textarea"))
        || (html.contains("data-slot=\"confirmation-dialog")
            && (html.contains("<dialog") || html.contains("showModal()")))
        || (html.contains("data-slot=\"invite-dialog")
            && (html.contains("<dialog") || html.contains("showModal()")))
        || html.contains("data-slot=\"invite-dialog-control\"")
        || html.contains("<label data-slot=\"invite-dialog\"")
        || (html.contains("data-slot=\"shimmer\"")
            && (html.contains("<span")
                || html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")))
        || (html.contains("data-slot=\"reveal\"")
            && html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden"))
        || (html.contains("data-slot=\"text-shimmer\"")
            && html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden"))
        || (html.contains("data-slot=\"particles\"")
            && (html.contains("<canvas")
                || html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")))
        || (html.contains("data-slot=\"sparkles-text\"")
            && html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden"))
        || (html.contains("data-slot=\"noise\"")
        || (html.contains("data-slot=\"typing-text\"")
            && html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden"))
}

pub fn looks_like_stub_fingerprint(html: &str) -> Option<&'static str> {
    if html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden") {
        return Some("fx");
    }
    if html.contains("<figure data-slot") && html.contains("<figcaption") {
        return Some("chart");
    }
    if html.contains("display:inline-flex;align-items:center;gap:0.35rem;padding:0.15rem 0.55rem;font-size:0.75rem;font-weight:500;") {
        return Some("pill");
    }
    if html.contains("<input data-slot=") && html.contains("-control\"") {
        return Some("field");
    }
    if html.contains("role=\"dialog\"") && html.contains("max-width:28rem;") {
        return Some("overlay");
    }
    if html.contains("<nav data-slot") && html.contains("flex-wrap:wrap;gap:0.25rem") {
        return Some("nav");
    }
    if html.contains("<section data-slot") && html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem;") {
        return Some("display");
    }
    None
}

pub fn check_family(family: &str) -> Result<(), AuditFinding> {
    match renderer_kind(family) {
        RendererKind::Dedicated(_) => Ok(()),
        RendererKind::Stub(kind) => Err(AuditFinding::fail(
            "logic",
            STUB_RENDERER,
            format!("family {family} is stub renderer {kind}"),
        )),
        RendererKind::Interact => Err(AuditFinding::fail(
            "logic",
            STUB_RENDERER,
            format!("family {family} is interact generic, not a dedicated port"),
        )),
        RendererKind::Missing => Err(AuditFinding::fail(
            "logic",
            STUB_RENDERER,
            format!("family {family} has no renderer"),
        )),
    }
}

pub fn check_ast(nodes: &[AstNode]) -> Vec<AuditFinding> {
    let mut out = Vec::new();
    for node in nodes {
        if let AstNode::Component(comp) = node {
            if let Some(family) = cronus_ui_widgets::family_of(comp) {
                if let Err(mut f) = check_family(family) {
                    f.family = Some(family.to_string());
                    out.push(f);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_widgets::{render, FAMILIES};

    fn stub(family: &str) -> crate::parser::ComponentNode {
        crate::cronus_ui_widgets::test_stub(family)
    }

    #[test]
    fn button_is_dedicated() {
        assert_eq!(renderer_kind("button"), RendererKind::Dedicated("button_from"));
        assert!(check_family("button").is_ok());
        let html = render(&stub("button")).unwrap();
        assert!(html.contains("data-slot=\"button\""));
        assert!(looks_like_stub_fingerprint(&html).is_none());
        assert!(!looks_like_interact_generic(&html));
    }

    #[test]
    fn sankey_chart_is_stub() {
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
        let err = check_family("sankey-chart").unwrap_err();
        assert_eq!(err.code, STUB_RENDERER);
        let html = render(&stub("sankey-chart")).unwrap();
        assert_eq!(looks_like_stub_fingerprint(&html), Some("chart"));
    }

    #[test]
    fn meteors_is_stub_fx() {
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert!(check_family("meteors").is_err());
        let html = render(&stub("meteors")).unwrap();
        assert_eq!(looks_like_stub_fingerprint(&html), Some("fx"));
    }

    #[test]
    fn every_family_is_classified() {
        for family in FAMILIES {
            assert!(
                !matches!(renderer_kind(family), RendererKind::Missing),
                "{family} missing RendererKind"
            );
        }
    }

    #[test]
    fn ported_families_are_dedicated_and_skip_interact() {
        for family in PORTED_FAMILIES {
            assert!(
                matches!(renderer_kind(family), RendererKind::Dedicated(_)),
                "{family}"
            );
            let html = render(&stub(family)).expect(family);
            if let Some(ih) = crate::cronus_ui_interact::render(family, &stub(family)) {
                assert_ne!(html, ih, "{family} still equals interact HTML");
            }
            assert!(
                !looks_like_interact_generic(&html),
                "{family} looks like interact: {html}"
            );
            assert!(
                looks_like_stub_fingerprint(&html).is_none(),
                "{family} looks like stub: {html}"
            );
        }
    }

    #[test]
    fn no_new_stub_families() {
        assert_eq!(FAMILIES.len(), 175);
        for family in PORTED_FAMILIES {
            assert!(FAMILIES.contains(family));
            assert!(cronus_ui_widgets::dedicated_render(family, &stub(family)).is_some());
        }
    }

    #[test]
    fn dedicated_module_no_sidecar_assets() {
        let files = [
            "src/cronus_ui.rs",
            "src/cronus_ui_widgets.rs",
            "src/cronus_ui_alert.rs",
            "src/cronus_ui_app_shell.rs",
            "src/cronus_ui_alert_dialog.rs",
            "src/cronus_ui_autocomplete.rs",
            "src/cronus_ui_badge.rs",
            "src/cronus_ui_banner.rs",
            "src/cronus_ui_checkbox.rs",
            "src/cronus_ui_copy_button.rs",
            "src/cronus_ui_credit_card_input.rs",
            "src/cronus_ui_input.rs",
            "src/cronus_ui_label.rs",
            "src/cronus_ui_lightbox.rs",
            "src/cronus_ui_metric.rs",
            "src/cronus_ui_mode_toggle.rs",
            "src/cronus_ui_multi_select.rs",
            "src/cronus_ui_textarea.rs",
            "src/cronus_ui_switch.rs",
            "src/cronus_ui_spinner.rs",
            "src/cronus_ui_split_button.rs",
            "src/cronus_ui_separator.rs",
            "src/cronus_ui_sheet.rs",
            "src/cronus_ui_sidebar.rs",
            "src/cronus_ui_signature_pad.rs",
            "src/cronus_ui_sonner.rs",
            "src/cronus_ui_skeleton.rs",
            "src/cronus_ui_shimmer.rs",
            "src/cronus_ui_reveal.rs",
            "src/cronus_ui_text_shimmer.rs",
            "src/cronus_ui_particles.rs",
            "src/cronus_ui_sparkles_text.rs",
            "src/cronus_ui_noise.rs",
            "src/cronus_ui_typing_text.rs",
            "src/cronus_ui_kbd.rs",
            "src/cronus_ui_toggle.rs",
            "src/cronus_ui_toggle_group.rs",
            "src/cronus_ui_usage_meter.rs",
            "src/cronus_ui_toolbar.rs",
            "src/cronus_ui_progress.rs",
            "src/cronus_ui_profit_loss_chart.rs",
            "src/cronus_ui_slider.rs",
            "src/cronus_ui_radio_group.rs",
            "src/cronus_ui_rating.rs",
            "src/cronus_ui_resizable.rs",
            "src/cronus_ui_rich_text_editor.rs",
            "src/cronus_ui_scroll_area.rs",
            "src/cronus_ui_scroll_progress.rs",
            "src/cronus_ui_chip.rs",
            "src/cronus_ui_choropleth_chart.rs",
            "src/cronus_ui_code_tabs.rs",
            "src/cronus_ui_collapsible.rs",
            "src/cronus_ui_avatar.rs",
            "src/cronus_ui_avatar_group.rs",
            "src/cronus_ui_card.rs",
            "src/cronus_ui_empty.rs",
            "src/cronus_ui_expandable_tabs.rs",
            "src/cronus_ui_field.rs",
            "src/cronus_ui_file_dropzone.rs",
            "src/cronus_ui_floating_label_input.rs",
            "src/cronus_ui_form.rs",
            "src/cronus_ui_input_group.rs",
            "src/cronus_ui_input_otp.rs",
            "src/cronus_ui_invite_dialog.rs",
            "src/cronus_ui_fab.rs",
            "src/cronus_ui_heatmap.rs",
            "src/cronus_ui_hover_card.rs",
            "src/cronus_ui_select.rs",
            "src/cronus_ui_segmented_control.rs",
            "src/cronus_ui_dialog.rs",
            "src/cronus_ui_dock.rs",
            "src/cronus_ui_drawer.rs",
            "src/cronus_ui_dropdown_menu.rs",
            "src/cronus_ui_tabs.rs",
            "src/cronus_ui_accordion.rs",
            "src/cronus_ui_bouncy_accordion.rs",
            "src/cronus_ui_table.rs",
            "src/cronus_ui_table_of_contents.rs",
            "src/cronus_ui_tags_input.rs",
            "src/cronus_ui_pagination.rs",
            "src/cronus_ui_popover.rs",
            "src/cronus_ui_morphing_popover.rs",
            "src/cronus_ui_breadcrumb.rs",
            "src/cronus_ui_button_group.rs",
            "src/cronus_ui_calendar.rs",
            "src/cronus_ui_combobox.rs",
            "src/cronus_ui_command.rs",
            "src/cronus_ui_masonry.rs",
            "src/cronus_ui_comparison_slider.rs",
            "src/cronus_ui_confirmation_dialog.rs",
            "src/cronus_ui_menubar.rs",
            "src/cronus_ui_navigation_menu.rs",
            "src/cronus_ui_notification_center.rs",
            "src/cronus_ui_context_menu.rs",
            "src/cronus_ui_status_dot.rs",
            "src/cronus_ui_sunburst_chart.rs",
            "src/cronus_ui_stepper.rs",
            "src/cronus_ui_tooltip.rs",
            "src/cronus_ui_password_input.rs",
            "src/cronus_ui_phone_input.rs",
            "src/cronus_ui_currency_input.rs",
            "src/cronus_ui_color_picker.rs",
            "src/cronus_ui_number_input.rs",
            "src/cronus_ui_date_picker.rs",
            "src/cronus_ui_time_picker.rs",
            "src/cronus_ui_date_range_picker.rs",
            "src/cronus_ui_area_chart.rs",
            "src/cronus_ui_bar_chart.rs",
            "src/cronus_ui_line_chart.rs",
            "src/cronus_ui_live_line_chart.rs",
            "src/cronus_ui_sparkline.rs",
            "src/cronus_ui_pie_chart.rs",
            "src/cronus_ui_pill_nav.rs",
            "src/cronus_ui_radar_chart.rs",
            "src/cronus_ui_scatter_chart.rs",
            "src/cronus_ui_scheduler.rs",
            "src/cronus_ui_ring_chart.rs",
            "src/cronus_ui_data_table.rs",
            "src/cronus_ui_workspace_switcher.rs",
            "src/cronus_ui_kit.rs",
            "src/cli/audit_http.rs",
            "src/ui/audit_layout.rs",
        ];
        for file in files {
            let src = std::fs::read_to_string(file).unwrap_or_else(|_| panic!("read {file}"));
            for (i, line) in src.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }
                if trimmed.contains("include_str!")
                    && !trimmed.contains("cronus_ui_tokens.css")
                    && !trimmed.contains(".cronus")
                {
                    panic!("{file}:{} sidecar include_str: {trimmed}", i + 1);
                }
                if (trimmed.contains("fs::read(")
                    || trimmed.contains("fs::read_to_string")
                    || trimmed.contains("std::fs::read_to_string"))
                    && !trimmed.contains("cronus_ui_tokens")
                    && !file.contains("audit_http")
                    && !file.contains("source_language")
                {
                    // audit_http must never read disk; this file is checked below.
                    if file.ends_with("audit_http.rs") {
                        panic!("{file}:{} must not read disk: {trimmed}", i + 1);
                    }
                }
            }
        }
        let http = std::fs::read_to_string("src/cli/audit_http.rs").unwrap();
        assert!(!http.contains("fs::read"), "audit_http must never read disk");
        assert!(
            !http.contains("read_to_string"),
            "audit_http must never read disk"
        );
    }
}
