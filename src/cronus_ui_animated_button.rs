//! Dedicated AnimatedButton renderer. DOM matches React idle:
//! `<button type="button" data-slot="animated-button" data-variant data-size>`
//! with the label and optional lucide glyphs (`icon:` / `icon-end:`), the same
//! `buttonVariants` chrome as `button` (all six variants, five sizes).
//! Motion: React's `whileHover={{ y: -1 }}` / `whileTap={{ scale: 0.97 }}` on a
//! `springSnappy` transition become `:hover` / `:active` transforms with the
//! same spring as a `linear()` easing (`--cronus-spring-snappy`, base.css).
//! Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::{attr_nonempty, esc, flag_any, item};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let style = comp.style.as_deref().unwrap_or("");
    let seg = |names: &[&str]| -> Option<String> {
        style
            .split('+')
            .skip(1)
            .map(str::trim)
            .find(|s| names.contains(s))
            .map(str::to_string)
    };
    let variant = comp
        .props
        .get("variant")
        .cloned()
        .or_else(|| {
            seg(&[
                "primary",
                "secondary",
                "outline",
                "ghost",
                "destructive",
                "danger",
                "link",
            ])
        })
        .unwrap_or_else(|| "primary".into());
    let variant = if variant == "danger" {
        "destructive".to_string()
    } else {
        variant
    };
    let size = comp
        .props
        .get("size")
        .cloned()
        .or_else(|| seg(&["sm", "md", "lg", "icon", "icon-sm"]))
        .unwrap_or_else(|| "md".into());
    let text = item(comp, "label").unwrap_or(comp.name.as_str());
    let icon_only = size.starts_with("icon");
    let mut inner = String::new();
    if let Some(icon) = attr_nonempty(comp, "icon") {
        inner.push_str(&crate::cronus_ui_icons::svg_or_empty(icon));
    }
    if !icon_only {
        inner.push_str(&esc(text));
    }
    if let Some(icon) = attr_nonempty(comp, "icon-end") {
        inner.push_str(&crate::cronus_ui_icons::svg_or_empty(icon));
    }
    let aria = attr_nonempty(comp, "aria-label")
        .or(if icon_only { Some(text) } else { None })
        .map(|a| format!(" aria-label=\"{}\"", esc(a)))
        .unwrap_or_default();
    let disabled = if flag_any(comp, "disabled") {
        " disabled"
    } else {
        ""
    };
    format!(
        "<button type=\"button\" data-slot=\"animated-button\" data-variant=\"{}\" data-size=\"{}\"{aria}{disabled}>{inner}</button>",
        esc(&variant),
        esc(&size)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<div"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("whileHover"));
        assert!(!html.contains("whileTap"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_is_button_with_label_not_fx_title_box() {
        let html = render(&stub("animated-button", "Launch"));
        assert_eq!(
            html,
            "<button type=\"button\" data-slot=\"animated-button\" data-variant=\"primary\" data-size=\"md\">Launch</button>"
        );
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("animated-button", "A <B> & \"C\""));
        assert!(html.contains(">A &lt;B&gt; &amp; &quot;C&quot;</button>"));
        reject_fx(&html);
    }

    #[test]
    fn variant_size_and_glyphs_from_style_and_props() {
        let mut c = stub("animated-button", "Download");
        c.style = Some("animated-button+secondary+lg".into());
        c.props.insert("icon".into(), "download".into());
        let html = render(&c);
        assert!(html.contains("data-variant=\"secondary\" data-size=\"lg\""));
        assert!(html.contains("data-icon=\"download\""));
        assert!(html.ends_with("Download</button>"));
        c.props.insert("variant".into(), "danger".into());
        assert!(render(&c).contains("data-variant=\"destructive\""));
    }

    #[test]
    fn icon_size_drops_text_and_labels_control() {
        let mut c = stub("animated-button", "Settings");
        c.style = Some("animated-button+icon".into());
        c.props.insert("icon".into(), "settings".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Settings\""));
        assert!(!html.contains(">Settings<"));
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("animated-button", "Launch");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("animated-button"),
            Some("cronus_ui_animated_button::render")
        );
        assert_eq!(
            renderer_kind("animated-button"),
            RendererKind::Dedicated("cronus_ui_animated_button::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("animated-button", "Launch"));
            assert!(!html.contains("v-"));
            assert!(!html.contains("@click"));
        });
    }
}
