//! Dedicated Meteors renderer. DOM mirrors other fx families:
//! `<div data-slot="meteors">` + an aria-hidden streak layer of eight `<div>`s
//! + a relative content `<div>` wrapping the label. Motion is CSS
//! `@keyframes cui-meteors` (zero JS, no inline style, no canvas).

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let streaks = "<div></div>".repeat(8);
    format!(
        "<div data-slot=\"meteors\"><div aria-hidden=\"true\">{streaks}</div><div>{}</div></div>",
        label_of(comp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;

    fn reject_fx(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("SURF"));
    }

    #[test]
    fn root_wraps_label_and_eight_streaks() {
        let html = render(&stub("meteors", "Night"));
        assert!(html.starts_with("<div data-slot=\"meteors\">"));
        assert_eq!(html.matches("<div></div>").count(), 8);
        assert!(html.contains(">Night</div></div>"));
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("meteors"),
            Some("cronus_ui_meteors::render")
        );
        assert_eq!(
            renderer_kind("meteors"),
            RendererKind::Dedicated("cronus_ui_meteors::render")
        );
        assert_eq!(
            crate::cronus_ui_widgets::render(&stub("meteors", "Night")).unwrap(),
            html
        );
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("meteors", "A <B>"));
        assert!(html.contains("A &lt;B&gt;"));
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn chrome_meteors_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"meteors\"] {\n  position: relative; overflow: hidden;\n  width: var(--cui-meteors-w, 100%); min-height: 8rem;\n}"
        ));
        assert!(css.contains("@keyframes cui-meteors"));
        assert!(css.contains("animation: cui-meteors"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("style="));
    }
}
