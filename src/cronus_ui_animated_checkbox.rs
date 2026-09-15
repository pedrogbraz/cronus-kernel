//! Dedicated AnimatedCheckbox renderer. DOM matches React `AnimatedCheckbox`:
//! `<label data-slot="animated-checkbox">` + a visually hidden native
//! `<input type="checkbox">` + the box `<span>` holding the check
//! `<svg><path>` + a `<span>` with the title and its strike line.
//! Toggling needs no JS: the label forwards clicks to the native input and
//! every visual state is CSS `:checked`. The check draw and the strike width
//! that React tweens with motion are CSS transitions (`pathLength="1"` +
//! `stroke-dashoffset`, `width`), so a checked control mounts drawn, like React.

use crate::cronus_ui_kit::{attr_nonempty, esc, flag, item};
use crate::parser::ComponentNode;

/// React `title` default.
const DEFAULT_TITLE: &str = "Implement Checkbox";

pub fn render(comp: &ComponentNode) -> String {
    let title = attr_nonempty(comp, "title")
        .or_else(|| item(comp, "label").filter(|t| !t.is_empty()))
        .unwrap_or(DEFAULT_TITLE);
    let checked = flag(comp, "checked") || flag(comp, "defaultChecked");
    let name = attr_nonempty(comp, "name")
        .map(|n| format!(" name=\"{}\"", esc(n)))
        .unwrap_or_default();
    format!(
        "<label data-slot=\"animated-checkbox\"><input type=\"checkbox\"{name}{checked}{disabled}><span><svg viewBox=\"0 0 20 20\" aria-hidden=\"true\"><path d=\"M 0 4.5 L 3.182 8 L 10 0\" fill=\"transparent\" stroke=\"currentColor\" stroke-width=\"1.5\" stroke-linecap=\"round\" stroke-linejoin=\"round\" transform=\"translate(5 6)\" pathLength=\"1\"></path></svg></span><span><span>{title}</span><span></span></span></label>",
        checked = if checked { " checked" } else { "" },
        disabled = if flag(comp, "disabled") { " disabled" } else { "" },
        title = esc(title),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn zero_js(html: &str) {
        for bad in [
            "<script",
            "<style",
            " style=",
            " onclick=",
            " onchange=",
            "v-model=",
        ] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn label_wraps_native_checkbox_box_and_title() {
        let html = render(&stub("animated-checkbox", "Ship the release"));
        assert_eq!(
            html,
            "<label data-slot=\"animated-checkbox\"><input type=\"checkbox\"><span><svg viewBox=\"0 0 20 20\" aria-hidden=\"true\"><path d=\"M 0 4.5 L 3.182 8 L 10 0\" fill=\"transparent\" stroke=\"currentColor\" stroke-width=\"1.5\" stroke-linecap=\"round\" stroke-linejoin=\"round\" transform=\"translate(5 6)\" pathLength=\"1\"></path></svg></span><span><span>Ship the release</span><span></span></span></label>"
        );
        zero_js(&html);
    }

    #[test]
    fn default_title_matches_react() {
        let mut c = stub("animated-checkbox", "");
        c.items.clear();
        assert!(render(&c).contains("<span>Implement Checkbox</span>"));
    }

    #[test]
    fn checked_disabled_name_and_title_props() {
        let mut c = stub("animated-checkbox", "Label item");
        c.props.insert("checked".into(), "true".into());
        c.props.insert("disabled".into(), "true".into());
        c.props.insert("name".into(), "todo".into());
        c.props.insert("title".into(), "A & B".into());
        let html = render(&c);
        assert!(
            html.contains("<input type=\"checkbox\" name=\"todo\" checked disabled>"),
            "{html}"
        );
        assert!(html.contains("<span>A &amp; B</span>"), "{html}");
        assert!(!html.contains("Label item"));

        let mut d = stub("animated-checkbox", "x");
        d.props.insert("defaultChecked".into(), "true".into());
        assert!(render(&d).contains("<input type=\"checkbox\" checked>"));
        let mut off = stub("animated-checkbox", "x");
        off.props.insert("checked".into(), "false".into());
        assert!(render(&off).contains("<input type=\"checkbox\">"));
    }

    #[test]
    fn emitted_fixture_parses_to_checked_control() {
        let src = "app \"x\" { port 1 }\ncomponent AnimatedCheckboxChecked layout:inline style:animated-checkbox {\n  checked:true\n  label \"Write tests\"\n}\n";
        let comp = crate::parser::parse(src)
            .expect("parse")
            .into_iter()
            .find_map(|n| match n {
                crate::parser::AstNode::Component(c) => Some(c),
                _ => None,
            })
            .expect("component");
        let html = crate::cronus_ui_widgets::render(&comp).expect("family");
        assert!(html.contains("<input type=\"checkbox\" checked>"), "{html}");
        assert!(html.contains("<span>Write tests</span>"), "{html}");
    }

    #[test]
    fn chrome_draws_state_from_checked_with_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"animated-checkbox\"] > input:checked + span { border-color: transparent; background-color: var(--cronus-fg); }"
        ));
        assert!(css.contains("stroke-dasharray: 1 1; stroke-dashoffset: 1; opacity: 0;"));
        assert!(css.contains(
            "[data-slot=\"animated-checkbox\"] > input:checked + span path { stroke-dashoffset: 0; opacity: 1; }"
        ));
        assert!(css.contains("[data-slot=\"animated-checkbox\"]:has(:disabled)"));
        assert_eq!(
            crate::cli::stub_renderer_gate::dedicated_fn_name("animated-checkbox"),
            Some("cronus_ui_animated_checkbox::render")
        );
    }
}
