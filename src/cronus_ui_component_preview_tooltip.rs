//! Dedicated ComponentPreviewTooltip renderer. React wraps a trigger
//! (`TooltipTrigger asChild`, an outline `Button` in the docs) and a
//! `TooltipContent` (`componentPreviewTooltipVariants`: rounded-3xl,
//! surface-floating, shadow-xl, 300×200, `p-3`) whose inset panel
//! (`rounded-2xl bg-surface-inset`) shows the preview scaled by 0.8.
//! The kernel keeps that tree inside one anchoring
//! `<span data-slot="component-preview-tooltip">`:
//! `<button data-slot="button" data-variant="outline" data-state="closed">`
//! then `<div data-slot="tooltip-content" role="tooltip" data-side aria-label=
//! "Component preview: <name>">` > inset panel > scaled frame > centred
//! preview card. The preview is a `w-56` raised card built from the `.cronus`:
//! `title "…"`, an optional `text "…"` caption and an optional `progress:75`
//! bar (`[data-slot="progress"]`, the docs' `h-2` primary bar). Without any
//! preview the panel shows `Preview not found` like React's fallback.
//!
//! Zero JS: `:hover` / `:focus-within` on the wrapper reveals the content with
//! React's `cronus-pop-in` entrance; `side:right|left|top|bottom` picks the
//! placement (`sideOffset` 5). `width` / `height` / `scale` props are not
//! read — the CSS carries React's defaults (300×200, 0.8).
//!
//! Inputs: `label "Hover me: Goal Card"` (trigger), `name:"goal-card"`
//! (accessible name), `side:`, `title`, `text`, `progress:`.

use crate::cronus_ui_kit::{attr_nonempty, choice, esc, item, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let trigger = label_of(comp);
    let side = choice(comp, "side", &["top", "right", "bottom", "left"]).unwrap_or("right");
    let name = attr_nonempty(comp, "name")
        .or_else(|| attr_nonempty(comp, "component"))
        .map(|n| format!("Component preview: {}", esc(n)))
        .unwrap_or_else(|| "Component preview".to_string());
    let title = item(comp, "title").filter(|t| !t.is_empty()).map(esc);
    let caption = item(comp, "text").filter(|t| !t.is_empty()).map(esc);
    let progress = attr_nonempty(comp, "progress")
        .and_then(|p| p.trim().trim_end_matches('%').parse::<f64>().ok())
        .map(|p| p.clamp(0.0, 100.0).round() as u32);
    let content_id = crate::cronus_ui_kit::instance_id(comp, "component-preview-tooltip");
    let panel = if title.is_none() && caption.is_none() && progress.is_none() {
        "<div data-failed=\"true\">Preview not found</div>".to_string()
    } else {
        let mut card = String::new();
        if let Some(t) = &title {
            card.push_str(&format!("<p>{t}</p>"));
        }
        if let Some(c) = &caption {
            card.push_str(&format!("<p>{c}</p>"));
        }
        if let Some(p) = progress {
            let state = if p >= 100 { "complete" } else { "loading" };
            card.push_str(&format!(
                "<div data-slot=\"progress\" role=\"progressbar\" aria-valuenow=\"{p}\" aria-valuemin=\"0\" aria-valuemax=\"100\" aria-valuetext=\"{p}%\" data-state=\"{state}\" data-value=\"{p}\" data-max=\"100\"><div data-state=\"{state}\" data-value=\"{p}\" data-max=\"100\"></div></div>"
            ));
        }
        format!("<div><div><div class=\"preview-card\">{card}</div></div></div>")
    };
    format!(
        "<span data-slot=\"component-preview-tooltip\"><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-state=\"closed\" aria-describedby=\"{content_id}\">{trigger}</button><div id=\"{content_id}\" data-slot=\"tooltip-content\" role=\"tooltip\" data-side=\"{side}\" aria-label=\"{name}\"><div>{panel}</div></div></span>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
    use crate::parser::ComponentItemNode;

    fn preview(label: &str) -> ComponentNode {
        reset_instance_ids();
        stub("component-preview-tooltip", label)
    }

    fn push(c: &mut ComponentNode, kind: &str, text: &str) {
        c.items.push(ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        });
    }

    fn reject_js(html: &str) {
        for bad in [
            "<script", "style=", "onclick", "onmouse", "<canvas", "popover",
        ] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn docs_goal_card_preview_dom() {
        let mut c = preview("Hover me: Goal Card");
        c.props.insert("name".into(), "goal-card".into());
        c.props.insert("progress".into(), "75".into());
        push(&mut c, "title", "Launch MVP");
        let html = render(&c);
        assert!(html.starts_with("<span data-slot=\"component-preview-tooltip\"><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-state=\"closed\" aria-describedby=\"cui-component-preview-tooltip-component-preview-tooltip\">Hover me: Goal Card</button><div id=\"cui-component-preview-tooltip-component-preview-tooltip\" data-slot=\"tooltip-content\" role=\"tooltip\" data-side=\"right\" aria-label=\"Component preview: goal-card\"><div><div><div><div class=\"preview-card\"><p>Launch MVP</p><div data-slot=\"progress\" role=\"progressbar\" aria-valuenow=\"75\""));
        assert!(html.ends_with("</div></div></div></div></div></div></span>"));
        reject_js(&html);
    }

    #[test]
    fn docs_todo_item_preview_has_caption_and_no_bar() {
        let mut c = preview("Hover me: Todo Item");
        c.props.insert("name".into(), "todo-item".into());
        push(&mut c, "title", "Review pull requests");
        push(&mut c, "text", "High · 1/2 subtasks");
        let html = render(&c);
        assert!(html.contains("<div class=\"preview-card\"><p>Review pull requests</p><p>High · 1/2 subtasks</p></div>"));
        assert!(!html.contains("data-slot=\"progress\""));
    }

    #[test]
    fn no_preview_shows_not_found_and_sides() {
        let html = render(&preview("Hover"));
        assert!(html.contains("aria-label=\"Component preview\"><div><div data-failed=\"true\">Preview not found</div></div></div>"));
        for side in ["top", "right", "bottom", "left"] {
            let mut c = preview("Hover");
            c.props.insert("side".into(), side.into());
            assert!(render(&c).contains(&format!("data-side=\"{side}\"")));
        }
        let mut c = preview("Hover");
        c.style = Some("component-preview-tooltip+bottom".into());
        assert!(render(&c).contains("data-side=\"bottom\""));
    }

    #[test]
    fn escapes_hostile_input() {
        let mut c = preview("<b>x</b> \"q\"");
        c.props.insert("name".into(), "\"n\"".into());
        push(&mut c, "title", "<i>t</i>");
        let html = render(&c);
        assert!(html.contains(">&lt;b&gt;x&lt;/b&gt; &quot;q&quot;</button>"));
        assert!(html.contains("aria-label=\"Component preview: &quot;n&quot;\""));
        assert!(html.contains("<p>&lt;i&gt;t&lt;/i&gt;</p>"));
        reject_js(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/component-preview-tooltip.css");
        assert!(css.contains("[data-slot=\"component-preview-tooltip\"]:is(:hover, :focus-within) > [data-slot=\"tooltip-content\"]"));
        assert!(css.contains("min-width: 18.75rem; max-width: 18.75rem; height: 12.5rem;"));
        assert!(css.contains("transform: scale(0.8)"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("[data-side=\"left\"]"));
        assert!(!css.contains("#"));
    }

    #[test]
    fn registered_as_dedicated() {
        let c = preview("Hover");
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), {
            reset_instance_ids();
            render(&c)
        });
        assert_eq!(
            dedicated_fn_name("component-preview-tooltip"),
            Some("cronus_ui_component_preview_tooltip::render")
        );
        assert_eq!(
            renderer_kind("component-preview-tooltip"),
            RendererKind::Dedicated("cronus_ui_component_preview_tooltip::render")
        );
    }
}
