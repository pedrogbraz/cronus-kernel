//! Dedicated GradientText renderer. DOM matches React:
//! `<span data-slot="gradient-text">` with the label, or — `asChild` on a
//! heading (`as:h1|h2|h3|h4`) — that heading element carrying the slot.
//! `size:5xl|6xl` is the docs display recipe (`font-display font-semibold
//! leading-[1.05] tracking-tight`). CSS gradient fill in the family CSS.
//! Zero JS. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::{attr_nonempty, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let tag = match attr_nonempty(comp, "as") {
        Some(t @ ("h1" | "h2" | "h3" | "h4" | "p")) => t,
        _ => "span",
    };
    let class = match attr_nonempty(comp, "size") {
        Some(s @ ("5xl" | "6xl")) => format!(" class=\"t-{s}\""),
        _ => String::new(),
    };
    format!(
        "<{tag} data-slot=\"gradient-text\"{class}>{}</{tag}>",
        label_of(comp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";
    const CSS: &str = include_str!("cronus_ui_css/gradient-text.css");

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<div"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_is_span_with_label_not_fx_title_box() {
        let html = render(&stub("gradient-text", "Proud of"));
        assert_eq!(html, "<span data-slot=\"gradient-text\">Proud of</span>");
        assert!(html.starts_with("<span "));
        assert!(html.contains("data-slot=\"gradient-text\""));
        assert!(html.contains(">Proud of</span>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("gradient-text", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<span data-slot=\"gradient-text\">A &lt;B&gt; &amp; &quot;C&quot;</span>"
        );
        reject_fx(&html);
    }

    /// Docs "Headline": `asChild` clips an `<h3>` at the 6xl display size.
    #[test]
    fn as_child_heading_with_display_size() {
        let mut c = stub("gradient-text", "Design that themes itself");
        c.props.insert("as".into(), "h3".into());
        c.props.insert("size".into(), "6xl".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<h3 data-slot=\"gradient-text\" class=\"t-6xl\">Design that themes itself</h3>"
        );
        reject_fx(&html);
        c.props.insert("as".into(), "script".into());
        assert!(render(&c).starts_with("<span "));
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("gradient-text", "Proud of");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        assert!(!html.contains("<div"));
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("gradient-text"),
            Some("cronus_ui_gradient_text::render")
        );
        assert_eq!(
            renderer_kind("gradient-text"),
            RendererKind::Dedicated("cronus_ui_gradient_text::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("gradient-text", "Proud of"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"gradient-text\""));
        });
    }

    #[test]
    fn chrome_gradient_fill_via_css() {
        assert!(CSS.contains("[data-slot=\"gradient-text\"]"));
        assert!(CSS.contains("background-clip: text"));
        assert!(CSS.contains("linear-gradient(135deg"));
        assert!(CSS.contains("var(--cronus-primary)"));
        assert!(CSS.contains("var(--cronus-accent)"));
        assert!(CSS.contains("[data-slot=\"gradient-text\"].t-6xl {"));
        assert!(CSS.contains("font-size: 3.75rem; line-height: 1.05;"));
        assert!(CSS.contains("letter-spacing: -0.025em;"));
        assert!(!CSS.contains("zinc-"));
        assert!(!CSS.contains(FX_BOX));
    }
}
