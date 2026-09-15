//! Dedicated Loader renderer. DOM matches React `Loader`:
//! `<div data-slot="loader" role="status" aria-busy="true" aria-label>` around
//! the 16×16 ten-tick glyph (`<svg>` with one stroked `<path>` per tick, each at
//! React's opacity, clipped by a `<clipPath>`). React spins the root with
//! Tailwind `animate-spin`, a CSS animation, so `loader.css` does the same;
//! reduced motion stops it through base.css.

use crate::cronus_ui_kit::{attr_nonempty, attr_num, esc, fmt_coord, widget_id};
use crate::parser::ComponentNode;

/// React `size` default (px).
const DEFAULT_SIZE: f64 = 16.0;

/// `(d, opacity)` per tick, in React's order. `None` = fully opaque.
const TICKS: [(&str, Option<&str>); 10] = [
    ("M8 0V4", None),
    ("M8 16V12", Some("0.5")),
    ("M3.29773 1.52783L5.64887 4.7639", Some("0.9")),
    ("M12.7023 1.52783L10.3511 4.7639", Some("0.1")),
    ("M12.7023 14.472L10.3511 11.236", Some("0.4")),
    ("M3.29773 14.472L5.64887 11.236", Some("0.6")),
    ("M15.6085 5.52783L11.8043 6.7639", Some("0.2")),
    ("M0.391602 10.472L4.19583 9.23598", Some("0.7")),
    ("M15.6085 10.4722L11.8043 9.2361", Some("0.3")),
    ("M0.391602 5.52783L4.19583 6.7639", Some("0.8")),
];

pub fn render(comp: &ComponentNode) -> String {
    let size = fmt_coord(
        attr_num::<f64>(comp, "size")
            .filter(|n| n.is_finite() && *n > 0.0)
            .unwrap_or(DEFAULT_SIZE),
    );
    // `aria-label` wins over `labels.loading` (emitted as `loading:`), like React.
    let name = attr_nonempty(comp, "aria-label")
        .or_else(|| attr_nonempty(comp, "loading"))
        .unwrap_or("Loading");
    let clip = widget_id(comp, "loader-clip");
    let ticks: String = TICKS
        .iter()
        .map(|(d, opacity)| {
            let opacity = opacity
                .map(|o| format!(" opacity=\"{o}\""))
                .unwrap_or_default();
            format!("<path d=\"{d}\"{opacity} stroke=\"currentColor\" stroke-width=\"1.5\"></path>")
        })
        .collect();
    format!(
        "<div data-slot=\"loader\" role=\"status\" aria-busy=\"true\" aria-label=\"{name}\"><svg height=\"{size}\" stroke-linejoin=\"round\" viewBox=\"0 0 16 16\" width=\"{size}\" aria-hidden=\"true\"><g clip-path=\"url(#{clip})\">{ticks}</g><defs><clipPath id=\"{clip}\"><rect fill=\"white\" height=\"16\" width=\"16\"></rect></clipPath></defs></svg></div>",
        name = esc(name)
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
            "<canvas",
            "v-data=",
        ] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn root_is_status_div_with_sixteen_px_glyph() {
        let html = render(&stub("loader", "Demo"));
        assert!(
            html.starts_with(
                "<div data-slot=\"loader\" role=\"status\" aria-busy=\"true\" aria-label=\"Loading\"><svg height=\"16\" stroke-linejoin=\"round\" viewBox=\"0 0 16 16\" width=\"16\" aria-hidden=\"true\">"
            ),
            "{html}"
        );
        assert_eq!(html.matches("<path ").count(), 10);
        assert!(html.contains(
            "<path d=\"M8 0V4\" stroke=\"currentColor\" stroke-width=\"1.5\"></path><path d=\"M8 16V12\" opacity=\"0.5\""
        ));
        assert!(html.ends_with("</svg></div>"));
        assert!(!html.contains("Demo"), "label is not the accessible name");
        zero_js(&html);
    }

    #[test]
    fn clip_path_id_is_defined_and_referenced() {
        let mut c = stub("loader", "Demo");
        c.name = "LoaderDefault".into();
        let html = render(&c);
        assert!(html.contains("<g clip-path=\"url(#cui-loaderdefault-loader-clip)\">"));
        assert!(html.contains(
            "<clipPath id=\"cui-loaderdefault-loader-clip\"><rect fill=\"white\" height=\"16\" width=\"16\"></rect></clipPath>"
        ));
    }

    #[test]
    fn size_and_accessible_name_props() {
        let mut c = stub("loader", "Demo");
        c.props.insert("size".into(), "32".into());
        c.props.insert("aria-label".into(), "Saving <draft>".into());
        let html = render(&c);
        assert!(html.contains("<svg height=\"32\""), "{html}");
        assert!(html.contains("width=\"32\""), "{html}");
        assert!(
            html.contains("aria-label=\"Saving &lt;draft&gt;\""),
            "{html}"
        );

        let mut labels = stub("loader", "Demo");
        labels.props.insert("loading".into(), "Carregando".into());
        assert!(render(&labels).contains("aria-label=\"Carregando\""));
    }

    #[test]
    fn invalid_size_falls_back_to_default() {
        for bad in ["0", "-4", "big", "NaN"] {
            let mut c = stub("loader", "Demo");
            c.props.insert("size".into(), bad.into());
            assert!(render(&c).contains("<svg height=\"16\""), "{bad}");
        }
    }

    #[test]
    fn emitted_fixture_parses_to_props() {
        let src = "app \"x\" { port 1 }\ncomponent LoaderLarge layout:inline style:loader {\n  size:32\n  aria-label:\"Saving\"\n  label \"Saving\"\n}\n";
        let comp = crate::parser::parse(src)
            .expect("parse")
            .into_iter()
            .find_map(|n| match n {
                crate::parser::AstNode::Component(c) => Some(c),
                _ => None,
            })
            .expect("component");
        let html = crate::cronus_ui_widgets::render(&comp).expect("family");
        assert!(html.contains("<svg height=\"32\""), "{html}");
        assert!(html.contains("aria-label=\"Saving\""), "{html}");
    }

    #[test]
    fn chrome_spins_with_css_animation() {
        let css = crate::cronus_ui::component_chrome_css();
        let start = css.find("[data-slot=\"loader\"] {").expect("loader chrome");
        let block = &css[start..start + css[start..].find('}').unwrap()];
        assert!(block.contains("display: inline-flex;"), "{block}");
        assert!(
            block.contains("animation: cui-loader-spin 1s linear infinite;"),
            "{block}"
        );
        assert!(css.contains("@keyframes cui-loader-spin"));
        assert_eq!(
            crate::cli::stub_renderer_gate::dedicated_fn_name("loader"),
            Some("cronus_ui_loader::render")
        );
    }
}
