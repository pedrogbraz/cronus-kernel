//! Stub renderer gate. A family is `ported` only when its
//! `cronus_ui_widgets::FAMILY_TABLE` entry is a dedicated renderer, not a stub.

use crate::cli::audit_codes::{AuditFinding, STUB_RENDERER};
use crate::cronus_ui_widgets::{self, Renderer};
use crate::parser::AstNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererKind {
    Dedicated(&'static str),
    Stub(&'static str),
    Missing,
}

pub fn dedicated_fn_name(family: &str) -> Option<&'static str> {
    match cronus_ui_widgets::renderer_of(family)? {
        Renderer::Dedicated(name, _) => Some(name),
        Renderer::Stub(..) => None,
    }
}

pub fn renderer_kind(family: &str) -> RendererKind {
    match cronus_ui_widgets::renderer_of(family) {
        Some(Renderer::Dedicated(name, _)) => RendererKind::Dedicated(name),
        Some(Renderer::Stub(kind, _)) => RendererKind::Stub(kind),
        None => RendererKind::Missing,
    }
}

/// Fingerprints of the retired generic renderers (inline-style stubs and the
/// old interact fallback). Dedicated output must never match.
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
        // Interact radios are inline-styled; zero-JS radio labels (tabs) are not.
        || (html.contains("<input type=\"radio\"") && html.contains("style="))
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
        || (html.contains("data-slot=\"stepper\"")
            && html.contains("<ol")
            && !html.contains("data-slot=\"stepper-list\""))
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
        // The retired renderer was a bare `<details data-slot="bouncy-accordion">`;
        // the dedicated one keeps React's slots around native `<details name>`.
        || html.contains("<details data-slot=\"bouncy-accordion\"")
        || (html.contains("data-slot=\"bouncy-accordion\"")
            && !html.contains("data-slot=\"bouncy-accordion-trigger\""))
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
            && !html.contains("tabindex=\"0\"")
            && !html.contains("data-slot=\"sidebar-content\""))
        || (html.contains("data-slot=\"toolbar\"") && html.contains("<nav"))
        || (html.contains("data-slot=\"toolbar\"")
            && !html.contains("data-slot=\"toolbar-button\""))
        || (html.contains("data-slot=\"toolbar\"") && html.contains("flex-wrap:wrap"))
        || (html.contains("data-slot=\"status-dot\"")
            && (html.contains("padding:0.15rem 0.55rem")
                || !html.contains("data-slot=\"status-dot-indicator\"")
                || !(html.contains("data-slot=\"status-dot-label\"")
                    || html.contains("data-slot=\"status-dot-sr-label\""))))
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
            && !html.contains("data-slot=\"avatar-fallback\""))
        || (html.contains("data-slot=\"workspace-switcher\"") && html.contains("<details"))
        || (html.contains("data-slot=\"workspace-switcher\"") && html.contains("flex-wrap:wrap"))
        || html.contains("<nav data-slot=\"app-shell\"")
        || (html.contains("data-slot=\"app-shell-content\"")
            && (!html.contains("data-slot=\"app-shell-header\"")
                || !html.contains("data-slot=\"app-shell-body\"")
                || !html.contains("data-slot=\"sidebar-wrapper\"")))
        || (html.contains("data-slot=\"app-shell\"") && html.contains("flex-wrap:wrap"))
        || (html.contains("data-slot=\"table-of-contents\"")
            && (!html.contains("data-slot=\"table-of-contents-list\"")
                || !html.contains("data-slot=\"table-of-contents-link\"")))
        || (html.contains("data-slot=\"table-of-contents\"") && html.contains("flex-wrap:wrap"))
        || (html.contains("data-slot=\"form\"") && html.contains("style="))
        || (html.contains("data-slot=\"form\"") && html.contains(">Submit</button>"))
        || (html.contains("data-slot=\"form\"") && !html.contains("data-slot=\"form-item\""))
        || (html.contains("data-slot=\"form\"")
            && !html.contains("data-slot=\"form-label\"")
            && !html.contains("<label data-slot=\"label\""))
        || (html.contains("data-slot=\"form\"") && html.contains("v-submit="))
        || (html.contains("data-slot=\"signature-pad\"")
            && !html.contains("data-slot=\"signature-pad-canvas\""))
        || (html.contains("data-slot=\"signature-pad\"") && html.contains("style="))
        || (html.contains("data-slot=\"resizable\"") && html.contains("<section"))
        || (html.contains("data-slot=\"resizable-panel-group\"")
            && !html.contains("data-slot=\"resizable-handle\""))
        || (html.contains("data-slot=\"resizable\"") && html.contains("max-height:12rem;overflow:auto"))
        || (html.contains("data-slot=\"scheduler\"")
            && (!html.contains("data-slot=\"scheduler-title\"")
                || !html.contains("data-slot=\"scheduler-grid\"")))
        || (html.contains("data-slot=\"scheduler\"")
            && html.contains("style=")
            && html.contains("grid-template-columns:repeat(7,1fr)"))
        || (html.contains("data-slot=\"alert-dialog")
            && (html.contains("<dialog data-slot=") || html.contains("showModal()")))
        || (html.contains("data-slot=\"lightbox")
            && (html.contains("<dialog") || html.contains("showModal()")))
        || html.contains("<details data-slot=\"notification-center\"")
        || (html.contains("data-slot=\"notification-center\"") && html.contains("<details"))
        || (html.contains("data-slot=\"notification-center\"")
            && !html.contains("data-slot=\"notification-row\"")
            && !html.contains("data-slot=\"notification-empty\""))
        // Interact radios are inline-styled; zero-JS radio labels are not (as for tabs).
        || (html.contains("data-slot=\"segmented-control\"")
            && html.contains("<input type=\"radio\"")
            && html.contains("style="))
        || (html.contains("data-slot=\"segmented-control\"")
            && !html.contains("data-slot=\"segmented-control-item\""))
        || (html.contains("data-slot=\"usage-meter\"") && html.contains("<progress"))
        || (html.contains("data-slot=\"usage-meter\"")
            && !html.contains("data-slot=\"usage-meter-fill\""))
        || (html.contains("data-slot=\"masonry\"") && html.contains("<section"))
        || (html.contains("data-slot=\"masonry\"") && html.contains("max-height:12rem;overflow:auto"))
        || (html.contains("data-slot=\"masonry\"") && html.contains("style="))
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
        || html.contains("<figure data-slot=\"gauge-chart\"")
        || (html.contains("data-slot=\"gauge-chart\"") && html.contains("<figcaption"))
        || html.contains("<figure data-slot=\"funnel-chart\"")
        || (html.contains("data-slot=\"funnel-chart\"") && html.contains("<figcaption"))
        || html.contains("<figure data-slot=\"candlestick-chart\"")
        || (html.contains("data-slot=\"candlestick-chart\"") && html.contains("<figcaption"))
        || (html.contains("data-slot=\"scroll-progress\"") && html.contains("<progress"))
        || (html.contains("data-slot=\"scroll-progress\"")
            && html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
            && !html.contains("data-slot=\"scroll-progress-fill\"")
            && !html.contains("data-slot=\"scroll-progress-ring\""))
        || html.contains("data-slot=\"rich-text-editor-control\"")
        || html.contains("<label data-slot=\"rich-text-editor\"")
        || (html.contains("data-slot=\"rich-text-editor\"") && html.contains("<textarea"))
        || (html.contains("data-slot=\"confirmation-dialog")
            && (html.contains("<dialog data-slot=") || html.contains("showModal()")))
        || (html.contains("data-slot=\"invite-dialog")
            && (html.contains("<dialog data-slot=") || html.contains("showModal()")))
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
            && html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden"))
        || (html.contains("data-slot=\"typing-text\"")
            && html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden"))
        || (html.contains("data-slot=\"word-rotate\"")
            && html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden"))
        || html.contains("<section data-slot=\"timeline\"")
        || (html.contains("data-slot=\"timeline\"")
            && (!html.contains("data-slot=\"timeline-item\"")
                || !html.contains("data-slot=\"timeline-content\"")))
        || (html.contains("data-slot=\"timeline\"") && html.contains("style="))
        || html.contains("<ul data-slot=\"tree-view\"")
        || html.contains("<section data-slot=\"tree-view\"")
        || (html.contains("data-slot=\"tree-view\"")
            && (!html.contains("data-slot=\"tree-view-item\"")
                || !html.contains("data-slot=\"tree-view-item-trigger\"")))
        || (html.contains("data-slot=\"tree-view\"") && html.contains("style="))
        || html.contains("<section data-slot=\"tilt-card\"")
        || (html.contains("data-slot=\"tilt-card\"")
            && html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem;"))
        || (html.contains("data-slot=\"star-border\"")
            && html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden"))
        || html.contains("<section data-slot=\"glass-card\"")
        || (html.contains("data-slot=\"glass-card\"")
            && html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem"))
        || html.contains("<pre data-slot=\"terminal\"")
        || html.contains("<section data-slot=\"terminal\"")
        || (html.contains("data-slot=\"terminal\"")
            && (!html.contains("data-slot=\"terminal-screen\"")
                || !html.contains("data-slot=\"terminal-prompt\"")))
        || (html.contains("data-slot=\"terminal\"") && html.contains("style="))
        || html.contains("<video data-slot=\"video-player\"")
        || html.contains("<section data-slot=\"video-player\"")
        || (html.contains("data-slot=\"video-player\"")
            && (!html.contains("data-slot=\"video-player-video\"")
                || !html.contains("data-slot=\"video-player-controls\"")
                || !html.contains("<video")
                || !html.contains("data-slot=\"video-player-play\"")))
        || (html.contains("data-slot=\"video-player\"") && html.contains("style="))
        || (html.contains("data-slot=\"text-effect\"")
            && html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden"))
        || html.contains("<section data-slot=\"spotlight-card\"")
        || (html.contains("data-slot=\"spotlight-card\"") && html.contains("style="))
        || (html.contains("data-slot=\"spotlight-card\"")
            && html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem"))
        || (html.contains("data-slot=\"animated-list\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("<span")
                || html.contains("style=")
                || !html.contains("data-slot=\"animated-list-item\"")))
        || (html.contains("data-slot=\"toast\"")
            && html.contains("style=")
            && !html.contains("data-slot=\"sonner\"")
            && !html.contains("data-slot=\"toaster\""))
        || html.contains("<section data-slot=\"carousel\"")
        || (html.contains("data-slot=\"carousel\"")
            && (!html.contains("data-slot=\"carousel-content\"")
                || !html.contains("data-slot=\"carousel-item\"")))
        || (html.contains("data-slot=\"carousel\"") && html.contains("style="))
        || html.contains("<section data-slot=\"code-block\"")
        || (html.contains("data-slot=\"code-block\"") && html.contains("style="))
        || (html.contains("<pre data-slot=\"code-block\"") && !html.contains("<code"))
        || html.contains("<section data-slot=\"description-list\"")
        || (html.contains("data-slot=\"description-list\"")
            && (!html.contains("data-slot=\"description-item\"")
                || !html.contains("data-slot=\"description-term\"")
                || !html.contains("data-slot=\"description-details\"")))
        || (html.contains("data-slot=\"description-list\"") && html.contains("style="))
        || html.contains("<section data-slot=\"kanban\"")
        || (html.contains("data-slot=\"kanban\"")
            && (!html.contains("data-slot=\"kanban-column\"")
                || !html.contains("data-slot=\"kanban-card\"")))
        || (html.contains("data-slot=\"kanban\"") && html.contains("style="))
        || html.contains("<pre data-slot=\"json-viewer\"")
        || html.contains("<section data-slot=\"json-viewer\"")
        || (html.contains("data-slot=\"json-viewer\"")
            && (!html.contains("data-slot=\"json-viewer-row\"")
                || !html.contains("data-slot=\"json-viewer-key\"")
                || !html.contains("data-slot=\"json-viewer-value\"")))
        || (html.contains("data-slot=\"json-viewer\"") && html.contains("style="))
        || (html.contains("data-slot=\"animated-number\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("<div")
                || html.contains("style=")
                || html.contains("setInterval")
                || html.contains("requestAnimationFrame")))
        || (html.contains("data-slot=\"marquee\"")
            && !html.contains("data-slot=\"marquee-group\""))
        || (html.contains("data-slot=\"gradient-text\"")
            && html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden"))
        || (html.contains("data-slot=\"shiny-text\"")
            && html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden"))
        || html.contains("<section data-slot=\"aspect-ratio\"")
        || (html.contains("data-slot=\"aspect-ratio\"") && html.contains("style="))
        || html.contains("<section data-slot=\"frame\"")
        || (html.contains("data-slot=\"frame\"")
            && (!html.contains("data-slot=\"frame-chrome\"")
                || !html.contains("data-slot=\"frame-content\"")))
        || (html.contains("data-slot=\"frame\"") && html.contains("style="))
        || html.contains("<section data-slot=\"flip-card\"")
        || (html.contains("data-slot=\"flip-card\"")
            && (!html.contains("data-slot=\"flip-card-front\"")
                || !html.contains("data-slot=\"flip-card-back\"")))
        || (html.contains("data-slot=\"flip-card\"") && html.contains("style="))
        || (html.contains("data-slot=\"countdown\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || !html.contains("data-slot=\"countdown-unit\"")
                || !html.contains("data-slot=\"countdown-value\"")
                || !html.contains("data-slot=\"countdown-label\"")
                || html.contains("setInterval")
                || html.contains("setTimeout")))
        || (html.contains("data-slot=\"animated-button\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("<div")
                || html.contains("<span")
                || !html.contains("<button")))
        || (html.contains("data-slot=\"card-stack\"")
            && (!html.contains("data-slot=\"card-stack-item\"") || html.contains("style=")))
        || html.contains("<figure data-slot=\"gauge-chart\"")
        || (html.contains("data-slot=\"gauge-chart\"") && html.contains("<figcaption"))
        || html.contains("<figure data-slot=\"funnel-chart\"")
        || (html.contains("data-slot=\"funnel-chart\"") && html.contains("<figcaption"))
        || html.contains("<figure data-slot=\"candlestick-chart\"")
        || (html.contains("data-slot=\"candlestick-chart\"") && html.contains("<figcaption"))
        || (html.contains("data-slot=\"logo-carousel\"")
            && (html.contains("<div data-slot=\"logo-carousel\"")
                || html.contains("style=")
                || !html.contains("data-slot=\"logo-carousel-item\"")
                || !html.contains("<ul")))
        || (html.contains("data-slot=\"dynamic-island\"")
            && (!html.contains("data-slot=\"dynamic-island-shell\"")
                || !html.contains("data-slot=\"dynamic-island-trigger\"")
                || html.contains("style=")))
        || (html.contains("data-slot=\"image-zoom\"")
            && (!html.contains("<button")
                || !html.contains("data-slot=\"image-zoom-content\"")
                || html.contains("style=")))
        || (html.contains("data-slot=\"aurora-background\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || !html.contains("<div aria-hidden=\"true\"><div></div><div></div><div></div></div>")
                || html.contains("<canvas")
                || html.contains("<script")
                || html.contains("setInterval")))
        || (html.contains("data-slot=\"border-beam\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<style")
                || !html.contains("<div data-slot=\"border-beam\"")
                || html.contains("<canvas")
                || html.contains("<script")
                || html.contains("setInterval")))
        || (html.contains("data-slot=\"confetti\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<canvas")
                || html.contains("<script")
                || html.contains("setInterval")
                || html.contains("requestAnimationFrame")
                || !html.contains("<div data-slot=\"confetti\"")))
        || html.contains("<figure data-slot=\"composed-chart\"")
        || (html.contains("data-slot=\"composed-chart\"") && html.contains("<figcaption"))
        || html.contains("<figure data-slot=\"heatmap-chart\"")
        || (html.contains("data-slot=\"heatmap-chart\"") && html.contains("<figcaption"))
        || html.contains("<figure data-slot=\"chart\"")
        || (html.contains("data-slot=\"chart\"") && html.contains("<figcaption"))
        || (html.contains("data-slot=\"click-spark\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<script")
                || html.contains("onclick=")
                || html.contains("requestAnimationFrame")
                || !html.contains("<div data-slot=\"click-spark\"")))
        || (html.contains("data-slot=\"glare-hover\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<script")
                || html.contains("onclick=")
                || html.contains("requestAnimationFrame")
                || !html.contains("<div data-slot=\"glare-hover\"")))
        || (html.contains("data-slot=\"magnetic\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<script")
                || html.contains("onclick=")
                || html.contains("requestAnimationFrame")
                || html.contains("data-magnetic-active")
                || !html.contains("data-slot=\"magnetic-target\"")))
        || (html.contains("data-slot=\"dot-pattern\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<style")
                || !html.contains("<div data-slot=\"dot-pattern\"")
                || html.contains("<script")
                || html.contains("<canvas")))
        || (html.contains("data-slot=\"flickering-grid\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<style")
                || !html.contains("<div data-slot=\"flickering-grid\"")
                || html.contains("<script")
                || html.contains("<canvas")))
        || (html.contains("data-slot=\"grid-pattern\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<style")
                || !html.contains("<div data-slot=\"grid-pattern\"")
                || html.contains("<script")
                || html.contains("<canvas")))
        || (html.contains("data-slot=\"highlighter\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<style")
                || html.contains("<script")
                || html.contains("setTimeout")
                || !html.contains("<span data-slot=\"highlighter\"><span aria-hidden=\"true\"></span>")))
        || (html.contains("data-slot=\"scramble-text\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<style")
                || html.contains("<script")
                || html.contains("setTimeout")
                || !html.contains("<span aria-hidden=\"true\">")))
        || (html.contains("data-slot=\"spinning-text\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<style")
                || html.contains("<script")
                || html.contains("setTimeout")
                || !html.contains("<div aria-hidden=\"true\">")
                || !html.contains("data-angle=\"")))
        || (html.contains("data-slot=\"gradient-border\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<style")
                || !html.contains("<div data-slot=\"gradient-border\"")
                || html.contains("<canvas")
                || html.contains("<script")))
        || (html.contains("data-slot=\"light-rays\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<style")
                || !html.contains("<div aria-hidden=\"true\"><div></div></div>")
                || html.contains("<canvas")
                || html.contains("<script")))
        || (html.contains("data-slot=\"orbit\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<style")
                || !html.contains("data-slot=\"orbit-ring\"")
                || !html.contains("data-slot=\"orbit-positioner\"")
                || !html.contains("data-slot=\"orbit-item\"")
                || html.contains("<canvas")
                || html.contains("<script")))
        || (html.contains("data-slot=\"progressive-blur\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<style")
                || html.contains("<script")
                || !html.contains("data-progressive-blur-host=\"true\"")
                || !html.contains("aria-hidden=\"true\"")
                || html.contains("<canvas")))
        || (html.contains("data-slot=\"retro-grid\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<style")
                || html.contains("<script")
                || !html.contains("<div aria-hidden=\"true\"><div><div></div></div></div>")
                || html.contains("<canvas")))
        || (html.contains("data-slot=\"ripple\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<style")
                || html.contains("<script")
                || !html.contains("<div aria-hidden=\"true\"><span></span>")
                || html.contains("<canvas")))
        || (html.contains("data-slot=\"motion-presets\"")
            && (html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden")
                || html.contains("style=")
                || html.contains("<style")
                || html.contains("<script")
                || !html.contains("data-slot=\"motion-preset\"")))
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
    if html.contains("<section data-slot")
        && html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem;")
    {
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
    use crate::cronus_ui_widgets::{render, FAMILIES, PORTED_FAMILIES};

    fn stub(family: &str) -> crate::parser::ComponentNode {
        crate::cronus_ui_widgets::test_stub(family)
    }

    #[test]
    fn button_is_dedicated() {
        assert_eq!(
            renderer_kind("button"),
            RendererKind::Dedicated("button_from")
        );
        assert!(check_family("button").is_ok());
        let html = render(&stub("button")).unwrap();
        assert!(html.contains("data-slot=\"button\""));
        assert!(looks_like_stub_fingerprint(&html).is_none());
        assert!(!looks_like_interact_generic(&html));
    }

    /// wave1s: form emits React's `label` / `input` slots (no form-label).
    /// Gate accepts it; a bare form without label slots is still caught.
    #[test]
    fn form_react_label_slots_pass_gate_bare_form_does_not() {
        let html = render(&stub("form")).unwrap();
        assert!(html.contains("<label data-slot=\"label\""), "{html}");
        assert!(!html.contains("data-slot=\"form-label\""), "{html}");
        assert!(!looks_like_interact_generic(&html), "{html}");
        assert!(looks_like_stub_fingerprint(&html).is_none(), "{html}");
        assert!(check_family("form").is_ok());
        let bare = "<form data-slot=\"form\"><div data-slot=\"form-item\"><input /></div></form>";
        assert!(looks_like_interact_generic(bare));
    }

    #[test]
    fn sankey_chart_is_dedicated() {
        assert_eq!(
            renderer_kind("sankey-chart"),
            RendererKind::Dedicated("cronus_ui_sankey_chart::render")
        );
        assert!(check_family("sankey-chart").is_ok());
        let html = render(&stub("sankey-chart")).unwrap();
        assert!(looks_like_stub_fingerprint(&html).is_none(), "{html}");
    }

    #[test]
    fn meteors_is_dedicated() {
        assert_eq!(
            renderer_kind("meteors"),
            RendererKind::Dedicated("cronus_ui_meteors::render")
        );
        assert!(check_family("meteors").is_ok());
        let html = render(&stub("meteors")).unwrap();
        assert!(looks_like_stub_fingerprint(&html).is_none(), "{html}");
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
    fn radio_fingerprint_is_inline_styling_not_any_radio() {
        let styled = "<label><input type=\"radio\" style=\"margin:0\" /></label>";
        assert!(looks_like_interact_generic(styled));
        let tabs = render(&stub("tabs")).unwrap();
        assert!(tabs.contains("<input type=\"radio\""), "{tabs}");
        assert!(!looks_like_interact_generic(&tabs), "{tabs}");
    }

    #[test]
    fn ported_families_are_dedicated_not_stub() {
        for family in PORTED_FAMILIES {
            assert!(
                matches!(renderer_kind(family), RendererKind::Dedicated(_)),
                "{family}"
            );
            let html = render(&stub(family)).expect(family);
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
        assert_eq!(FAMILIES.len(), crate::cronus_ui_widgets::FAMILY_TABLE.len());
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
            "src/cronus_ui_text_effect.rs",
            "src/cronus_ui_particles.rs",
            "src/cronus_ui_sparkles_text.rs",
            "src/cronus_ui_noise.rs",
            "src/cronus_ui_typing_text.rs",
            "src/cronus_ui_word_rotate.rs",
            "src/cronus_ui_timeline.rs",
            "src/cronus_ui_tree_view.rs",
            "src/cronus_ui_tilt_card.rs",
            "src/cronus_ui_star_border.rs",
            "src/cronus_ui_glare_hover.rs",
            "src/cronus_ui_glass_card.rs",
            "src/cronus_ui_terminal.rs",
            "src/cronus_ui_video_player.rs",
            "src/cronus_ui_spotlight_card.rs",
            "src/cronus_ui_animated_list.rs",
            "src/cronus_ui_toast.rs",
            "src/cronus_ui_carousel.rs",
            "src/cronus_ui_code_block.rs",
            "src/cronus_ui_description_list.rs",
            "src/cronus_ui_kanban.rs",
            "src/cronus_ui_json_viewer.rs",
            "src/cronus_ui_animated_number.rs",
            "src/cronus_ui_marquee.rs",
            "src/cronus_ui_gradient_text.rs",
            "src/cronus_ui_shiny_text.rs",
            "src/cronus_ui_aspect_ratio.rs",
            "src/cronus_ui_frame.rs",
            "src/cronus_ui_flip_card.rs",
            "src/cronus_ui_countdown.rs",
            "src/cronus_ui_animated_button.rs",
            "src/cronus_ui_card_stack.rs",
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
            "src/cronus_ui_click_spark.rs",
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
            "src/cronus_ui_funnel_chart.rs",
            "src/cronus_ui_gauge_chart.rs",
            "src/cronus_ui_input_group.rs",
            "src/cronus_ui_input_otp.rs",
            "src/cronus_ui_invite_dialog.rs",
            "src/cronus_ui_fab.rs",
            "src/cronus_ui_heatmap.rs",
            "src/cronus_ui_hover_card.rs",
            "src/cronus_ui_image_zoom.rs",
            "src/cronus_ui_select.rs",
            "src/cronus_ui_segmented_control.rs",
            "src/cronus_ui_dialog.rs",
            "src/cronus_ui_dock.rs",
            "src/cronus_ui_drawer.rs",
            "src/cronus_ui_dropdown_menu.rs",
            "src/cronus_ui_dynamic_island.rs",
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
            "src/cronus_ui_candlestick_chart.rs",
            "src/cronus_ui_combobox.rs",
            "src/cronus_ui_command.rs",
            "src/cronus_ui_magnetic.rs",
            "src/cronus_ui_dot_pattern.rs",
            "src/cronus_ui_flickering_grid.rs",
            "src/cronus_ui_grid_pattern.rs",
            "src/cronus_ui_highlighter.rs",
            "src/cronus_ui_scramble_text.rs",
            "src/cronus_ui_spinning_text.rs",
            "src/cronus_ui_gradient_border.rs",
            "src/cronus_ui_light_rays.rs",
            "src/cronus_ui_orbit.rs",
            "src/cronus_ui_animated_checkbox.rs",
            "src/cronus_ui_loader.rs",
            "src/cronus_ui_number_flow.rs",
            "src/cronus_ui_scroll_nav.rs",
            "src/cronus_ui_slide_up_text.rs",
            "src/cronus_ui_progressive_blur.rs",
            "src/cronus_ui_retro_grid.rs",
            "src/cronus_ui_ripple.rs",
            "src/cronus_ui_motion_presets.rs",
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
            "src/cronus_ui_logo_carousel.rs",
            "src/cronus_ui_aurora_background.rs",
            "src/cronus_ui_border_beam.rs",
            "src/cronus_ui_confetti.rs",
            "src/cronus_ui_composed_chart.rs",
            "src/cronus_ui_heatmap_chart.rs",
            "src/cronus_ui_chart.rs",
            "src/cronus_ui_sparkline.rs",
            "src/cronus_ui_pie_chart.rs",
            "src/cronus_ui_pill_nav.rs",
            "src/cronus_ui_radar_chart.rs",
            "src/cronus_ui_scatter_chart.rs",
            "src/cronus_ui_scheduler.rs",
            "src/cronus_ui_ring_chart.rs",
            "src/cronus_ui_data_table.rs",
            "src/cronus_ui_workspace_switcher.rs",
            // Sprint 5 C2 — AI suite (alphabetical).
            "src/cronus_ui_conversation.rs",
            "src/cronus_ui_inline_citation.rs",
            "src/cronus_ui_message.rs",
            "src/cronus_ui_prompt_input.rs",
            "src/cronus_ui_reasoning.rs",
            "src/cronus_ui_sources.rs",
            "src/cronus_ui_suggestion.rs",
            "src/cronus_ui_tool.rs",
            "src/cronus_ui_kit.rs",
            "src/cli/audit_http.rs",
            "src/ui/audit_layout.rs",
        ];
        for file in files {
            let src = std::fs::read_to_string(file).unwrap_or_else(|_| panic!("read {file}"));
            // Only the render path is gated: a `#[cfg(test)]` module may read
            // its family stylesheet to assert chrome.
            let src = src.split("#[cfg(test)]").next().unwrap_or("");
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
        assert!(
            !http.contains("fs::read"),
            "audit_http must never read disk"
        );
        assert!(
            !http.contains("read_to_string"),
            "audit_http must never read disk"
        );
    }
}
