//! Dedicated ColorPicker renderer. DOM matches React's closed ColorPicker: a
//! single outline `<button data-slot="color-picker-trigger">` holding the
//! `<span data-slot="color-picker-swatch">` tile and the value text (React has
//! no wrapper element). React paints the tile with an inline `style`; the kernel
//! paints it two ways, both without `style=`:
//! - portable (Firefox, Safari, Chromium): an un-slotted `aria-hidden` `<svg>`
//!   inside the tile whose `<rect>` carries the SVG presentation attribute
//!   `fill="…"`. The chrome sizes it to the tile's padding box and the tile
//!   clips it (`overflow: hidden`), so the tile's rect/border/radius are unchanged.
//!   The color is validated strictly (`safe_color`); an unsafe or unknown value
//!   emits no `<svg>` and the tile shows the `--cronus-primary` fallback.
//! - Chromium 133+ progressive enhancement: `data-color` read by typed `attr()`
//!   in the chrome, which also paints under the (translucent) border like React.
//!
//! Zero JS: the trigger's `popovertarget` opens React's `color-picker-content`
//! as a native `popover="auto"` anchored under it (Esc / outside click dismiss)
//! with the `color-picker-value` readout and, when `item "…"` presets are given,
//! a `color-picker-swatches` radiogroup of `color-picker-swatch-button` labels
//! around visually hidden radios (`name:"…"` prop, else a widget id; the preset
//! equal to the value is checked), each painted by the same validated SVG
//! `fill` — so a preset submits with a form.
//! Gaps: the saturation area, hue slider and L/C/H inputs need JS and are not
//! rendered; picking a preset does not repaint the trigger swatch or text;
//! `aria-expanded` is not reflected. A real `disabled` prop adds
//! `data-disabled`, `disabled` and dims.
//! Not interact `input("color-picker", "color")` as the only control.

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, flag, item, label_of, widget_id};
use crate::parser::ComponentNode;

const DEFAULT_VALUE: &str = "oklch(0.62 0.21 256)";

pub fn render(comp: &ComponentNode) -> String {
    let raw = raw_value(comp);
    let value = esc(raw);
    let name = name_of(comp);
    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "editor");
    let mut btn = format!(
        "type=\"button\" id=\"{trigger_id}\" data-slot=\"color-picker-trigger\" data-variant=\"outline\" aria-label=\"{name}: {value}\" aria-haspopup=\"dialog\" aria-expanded=\"false\" data-state=\"closed\" popovertarget=\"{pop_id}\""
    );
    if flag(comp, "disabled") {
        btn.push_str(" data-disabled=\"\" disabled");
    }
    let fill = safe_color(raw)
        .map(|c| fill_svg(&esc(c)))
        .unwrap_or_default();
    let presets = presets_html(comp, raw);
    format!(
        "<button {btn}><span aria-hidden=\"true\" data-slot=\"color-picker-swatch\" data-color=\"{value}\">{fill}</span><span>{value}</span></button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"color-picker-content\" role=\"dialog\" aria-label=\"{name}\" anchor=\"{trigger_id}\"><span data-slot=\"color-picker-value\">{value}</span>{presets}</div>"
    )
}

/// Un-slotted swatch paint; `color` must be `safe_color`-validated and escaped.
fn fill_svg(color: &str) -> String {
    format!(
        "<svg aria-hidden=\"true\" focusable=\"false\" viewBox=\"0 0 1 1\" preserveAspectRatio=\"none\"><rect width=\"1\" height=\"1\" fill=\"{color}\"></rect></svg>"
    )
}

/// `item` / `preset` items with a safe color become radio swatches.
fn presets_html(comp: &ComponentNode, raw_value: &str) -> String {
    let group = attr_nonempty(comp, "name")
        .map(esc)
        .unwrap_or_else(|| widget_id(comp, "value"));
    let swatches: String = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "item" | "preset"))
        .filter_map(|i| safe_color(&i.text))
        .map(|c| {
            let v = esc(c);
            let checked = if c.eq_ignore_ascii_case(raw_value.trim()) {
                " checked"
            } else {
                ""
            };
            format!(
                "<label data-slot=\"color-picker-swatch-button\" aria-label=\"{v}\"><input type=\"radio\" name=\"{group}\" value=\"{v}\"{checked}>{}</label>",
                fill_svg(&v)
            )
        })
        .collect();
    if swatches.is_empty() {
        String::new()
    } else {
        format!(
            "<div data-slot=\"color-picker-swatches\" role=\"radiogroup\" aria-label=\"Presets\">{swatches}</div>"
        )
    }
}

/// Returns the color when it is a strictly-shaped CSS color that is safe to
/// place in an attribute: `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, or
/// `rgb|rgba|hsl|hsla|oklch|oklab|lab|lch(…)` whose arguments use only digits,
/// ASCII letters (units, `none`), spaces, `.`, `%`, `,`, `/`, `+`, `-`.
/// Named colors, `var()`, `url()`, nested functions and quotes are rejected.
fn safe_color(raw: &str) -> Option<&str> {
    let s = raw.trim();
    if s.len() > 64 {
        return None;
    }
    if let Some(hex) = s.strip_prefix('#') {
        let ok = matches!(hex.len(), 3 | 4 | 6 | 8) && hex.bytes().all(|b| b.is_ascii_hexdigit());
        return ok.then_some(s);
    }
    let open = s.find('(')?;
    let func = s[..open].to_ascii_lowercase();
    if !matches!(
        func.as_str(),
        "rgb" | "rgba" | "hsl" | "hsla" | "oklch" | "oklab" | "lab" | "lch"
    ) {
        return None;
    }
    let args = s[open + 1..].strip_suffix(')')?;
    let ok = !args.trim().is_empty()
        && args.bytes().any(|b| b.is_ascii_digit())
        && args
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b" .%,/+-".contains(&b));
    ok.then_some(s)
}

fn raw_value(comp: &ComponentNode) -> &str {
    if let Some(v) = attr_nonempty(comp, "value") {
        return v;
    }
    if let Some(v) = attr(comp, "defaultValue")
        .or_else(|| attr(comp, "default-value"))
        .filter(|s| !s.is_empty())
    {
        return v;
    }
    if let Some(t) = item(comp, "value").filter(|s| !s.is_empty()) {
        return t;
    }
    DEFAULT_VALUE
}

fn name_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr_nonempty(comp, "aria-label") {
        return esc(v);
    }
    let label = label_of(comp);
    if label.is_empty() || label == "color-picker" {
        "Color".into()
    } else {
        label
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_interact(html: &str) {
        assert!(!html.contains("color-picker-control"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("type=\"color\""));
        assert!(!html.contains("<input"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("height:2.5rem;padding:0 0.75rem"));
    }

    #[test]
    fn trigger_opens_native_popover_with_value() {
        let c = stub("color-picker", "Accent");
        let html = render(&c);
        assert!(
            !html.contains("data-slot=\"color-picker\""),
            "React has no wrapper slot: {html}"
        );
        assert!(
            !html.contains("color-picker-swatch-button"),
            "no presets given"
        );
        reject_interact(&html);
        let tid = crate::cronus_ui_kit::widget_id(&c, "trigger");
        let pid = crate::cronus_ui_kit::widget_id(&c, "editor");
        let trigger = format!(
            "<button type=\"button\" id=\"{tid}\" data-slot=\"color-picker-trigger\" data-variant=\"outline\" aria-label=\"Accent: oklch(0.62 0.21 256)\" aria-haspopup=\"dialog\" aria-expanded=\"false\" data-state=\"closed\" popovertarget=\"{pid}\"><span aria-hidden=\"true\" data-slot=\"color-picker-swatch\" data-color=\"oklch(0.62 0.21 256)\"><svg aria-hidden=\"true\" focusable=\"false\" viewBox=\"0 0 1 1\" preserveAspectRatio=\"none\"><rect width=\"1\" height=\"1\" fill=\"oklch(0.62 0.21 256)\"></rect></svg></span><span>oklch(0.62 0.21 256)</span></button>"
        );
        assert_eq!(
            html,
            format!(
                "{trigger}<div id=\"{pid}\" popover=\"auto\" data-slot=\"color-picker-content\" role=\"dialog\" aria-label=\"Accent\" anchor=\"{tid}\"><span data-slot=\"color-picker-value\">oklch(0.62 0.21 256)</span></div>"
            )
        );
        assert_eq!(
            trigger.matches("data-slot=").count(),
            2,
            "svg must stay un-slotted"
        );
    }

    /// Preset items become radio swatches (form value); unsafe colors are skipped.
    #[test]
    fn preset_items_are_radio_swatches_with_validated_fill() {
        let mut c = stub("color-picker", "Accent");
        c.props.insert("value".into(), "#00aaff".into());
        for t in ["#0af", "red\" onload=\"x", "#00aaff"] {
            c.items.push(crate::parser::ComponentItemNode {
                item_type: "item".into(),
                text: t.into(),
                link: None,
                tone: None,
                config: Default::default(),
            });
        }
        let html = render(&c);
        let name = crate::cronus_ui_kit::widget_id(&c, "value");
        let swatch = |v: &str, checked: &str| {
            format!(
                "<label data-slot=\"color-picker-swatch-button\" aria-label=\"{v}\"><input type=\"radio\" name=\"{name}\" value=\"{v}\"{checked}><svg aria-hidden=\"true\" focusable=\"false\" viewBox=\"0 0 1 1\" preserveAspectRatio=\"none\"><rect width=\"1\" height=\"1\" fill=\"{v}\"></rect></svg></label>"
            )
        };
        assert!(html.contains(&format!(
            "<div data-slot=\"color-picker-swatches\" role=\"radiogroup\" aria-label=\"Presets\">{}{}</div></div>",
            swatch("#0af", ""),
            swatch("#00aaff", " checked")
        )));
        assert!(!html.contains("onload=\""));
        assert!(!html.contains("type=\"color\""));
    }

    #[test]
    fn value_and_aria_label_name_trigger() {
        let mut c = stub("color-picker", "Accent");
        c.props
            .insert("value".into(), "oklch(0.72 0.19 145)".into());
        c.props.insert("aria-label".into(), "Color".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Color: oklch(0.72 0.19 145)\""));
        assert!(html.contains("data-color=\"oklch(0.72 0.19 145)\""));
        assert!(html.contains("fill=\"oklch(0.72 0.19 145)\""));
        assert!(html.contains("<span>oklch(0.72 0.19 145)</span>"));
        reject_interact(&html);
    }

    /// Portable paint: the swatch carries an SVG with a validated `fill`
    /// presentation attribute (typed `attr()` alone only paints in Chromium).
    #[test]
    fn swatch_svg_carries_validated_fill() {
        for ok in [
            "#0af",
            "#0AF8",
            "#00aaff",
            "#00aaff80",
            "rgb(0 170 255 / 50%)",
            "hsl(200deg, 100%, 50%)",
            "OKLCH(0.62 0.21 256)",
        ] {
            let mut c = stub("color-picker", "Accent");
            c.props.insert("value".into(), ok.into());
            let html = render(&c);
            assert!(
                html.contains(&format!(
                    "<rect width=\"1\" height=\"1\" fill=\"{ok}\"></rect>"
                )),
                "{ok}: {html}"
            );
        }
    }

    #[test]
    fn malicious_or_unknown_color_emits_no_fill() {
        for bad in [
            "red\" onload=\"x",
            "red",
            "#12345",
            "#ggg",
            "url(#x)",
            "var(--cronus-primary)",
            "rgb(1,2,3)\"><script>",
            "rgb(calc(1) 2 3)",
            "oklch(0.6 0.2 256",
            "rgb()",
            "expression(alert(1))",
        ] {
            let mut c = stub("color-picker", "Accent");
            c.props.insert("value".into(), bad.into());
            let html = render(&c);
            assert!(!html.contains("<svg"), "{bad}: {html}");
            assert!(!html.contains("fill="), "{bad}: {html}");
            assert!(!html.contains("onload=\""), "{bad}: {html}");
            assert!(!html.contains("<script"), "{bad}: {html}");
            reject_interact(&html);
        }
        assert_eq!(safe_color("red\" onload=\"x"), None);
    }

    #[test]
    fn disabled_prop_marks_data_disabled() {
        let mut c = stub("color-picker", "Accent");
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" data-disabled=\"\" disabled>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_native_color_input() {
        let c = stub("color-picker", "Accent");
        let html = render(&c);
        assert!(!html.contains("color-picker-control"));
        assert!(!html.contains("type=\"color\""));
        assert!(html.contains("data-slot=\"color-picker-trigger\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("color-picker", "Accent"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"color-picker-trigger\"]"));
        assert!(css.contains("[data-slot=\"color-picker-swatch\"]"));
        assert!(css.contains("[data-slot=\"color-picker-content\"]:popover-open {"));
        assert!(css.contains("[data-slot=\"color-picker-swatch-button\"]:has(> input:checked) {"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }

    /// Wave 1t geometry: outline Button `h-10 px-4 w-full justify-start gap-2`
    /// at text-sm/20px; 20px rounded-md bordered tile painted from `data-color`
    /// (React paints oklch(0.62 0.21 256), not the primary token).
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"color-picker-trigger\"] {\n  display: inline-flex; align-items: center; justify-content: flex-start; gap: 0.5rem;\n  width: 100%; height: 2.5rem; padding: 0 1rem; box-sizing: border-box; white-space: nowrap;"
        ));
        assert!(css
            .contains("background-color: attr(data-color type(<color>), var(--cronus-primary));"));
        assert!(
            css.contains("[data-slot=\"color-picker-trigger\"][data-disabled] { opacity: 0.5; }")
        );
        assert!(!css.contains("[data-slot=\"color-picker\"] {"));
        assert!(css.contains("[data-slot=\"color-picker-swatch-button\"] {"));
    }

    /// The portable SVG fills the tile's padding box and is clipped to its radius.
    #[test]
    fn chrome_sizes_and_clips_swatch_svg() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "border-radius: var(--cronus-radius-md); border: 1px solid var(--cronus-border);\n  overflow: hidden;"
        ));
        assert!(css.contains(
            "[data-slot=\"color-picker-swatch\"] > svg {\n  display: block; width: 100%; height: 100%; pointer-events: none;\n}"
        ));
    }
}
