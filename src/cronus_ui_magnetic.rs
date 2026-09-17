//! Dedicated Magnetic renderer. DOM matches React idle:
//! `<div data-slot="magnetic">` wrapping `data-slot="magnetic-target"` and
//! the content — the escaped label, or the docs `Button` from an `action`
//! item (`variant:` / `size:` / `icon:` config, `shape:pill`, `glow:true`,
//! `px:8` for the docs classes). Several `action` items render one magnetic
//! per button in a `gap-1` row; with `caption:"…"` they become the docs
//! strength / radius field (`items-end gap-8` columns with a mono caption).
//!
//! React lerps the target toward the pointer with rAF (`strength` × a
//! falloff over `radius`), springing back on a 300 ms
//! `cubic-bezier(0.34, 1.56, 0.64, 1)`. Closest CSS: eight hover zones tile
//! the wrapper's padding ring (`padding:3|8|10`, Tailwind units, the docs'
//! `p-*`), and hovering one pulls the target toward it by `strength × 24px`
//! (`s-<strength×100>` class) on the same curve; over the content itself the
//! pull is ~0, as in React. `radius` shapes React's falloff only and has no
//! CSS knob. Off under reduced motion. Zero rAF / `data-magnetic-active`.
//! Not the catalog `fx()` title SURF box.

use crate::cronus_ui_glass_card::action_button;
use crate::cronus_ui_kit::{attr_nonempty, attr_num, label_of, truthy};
use crate::parser::{ComponentItemNode, ComponentNode};

const ZONES: &str = "<span class=\"z z1\"></span><span class=\"z z2\"></span><span class=\"z z3\"></span><span class=\"z z4\"></span><span class=\"z z5\"></span><span class=\"z z6\"></span><span class=\"z z7\"></span><span class=\"z z8\"></span>";

fn cfg<'a>(i: &'a ComponentItemNode, key: &str) -> Option<&'a str> {
    i.config
        .get(key)
        .map(String::as_str)
        .filter(|v| !v.is_empty())
}

fn button(item: &ComponentItemNode) -> String {
    let mut classes = vec!["cui-btn"];
    if cfg(item, "shape") == Some("pill") {
        classes.push("pill");
    }
    if cfg(item, "glow").is_some_and(truthy) {
        classes.push("glow");
    }
    if cfg(item, "px") == Some("8") {
        classes.push("px-8");
    }
    action_button(item).replacen(
        "class=\"cui-btn\"",
        &format!("class=\"{}\"", classes.join(" ")),
        1,
    )
}

/// One magnetic wrapper. `strength` / `padding` come from the action's config
/// first, then the component props (React defaults: 0.3, no padding).
fn wrapper(comp: &ComponentNode, action: Option<&ComponentItemNode>, inner: &str) -> String {
    let get = |key: &str| -> Option<String> {
        action
            .and_then(|a| cfg(a, key).map(str::to_string))
            .or_else(|| attr_nonempty(comp, key).map(str::to_string))
    };
    let mut classes: Vec<String> = Vec::new();
    if let Some(p @ ("3" | "8" | "10")) = get("padding").as_deref() {
        classes.push(format!("p-{p}"));
    }
    let strength = get("strength")
        .and_then(|s| s.trim().parse::<f64>().ok())
        .unwrap_or(0.3)
        .clamp(0.0, 1.0);
    let s = (strength * 100.0).round() as u32;
    if s != 30 {
        classes.push(format!("s-{s}"));
    }
    let class = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    format!(
        "<div data-slot=\"magnetic\"{class}>{ZONES}<div data-slot=\"magnetic-target\">{inner}</div></div>"
    )
}

pub fn render(comp: &ComponentNode) -> String {
    let actions: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "action" | "button") && !i.text.is_empty())
        .collect();
    match actions.as_slice() {
        [] => wrapper(comp, None, &label_of(comp)),
        [one] => wrapper(comp, Some(one), &button(one)),
        many => {
            let field = many.iter().any(|a| cfg(a, "caption").is_some());
            let cells: String = many
                .iter()
                .map(|a| {
                    let w = wrapper(comp, Some(a), &button(a));
                    match cfg(a, "caption") {
                        Some(c) if field => format!(
                            "<div>{w}<span>{}</span></div>",
                            crate::cronus_ui_kit::esc(c)
                        ),
                        _ if field => format!("<div>{w}</div>"),
                        _ => w,
                    }
                })
                .collect();
            let class = if field {
                "cui-magnetic-field"
            } else {
                "cui-magnetic-row"
            };
            format!("<div class=\"{class}\">{cells}</div>")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_glass_card::tests::item;
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";
    const CSS: &str = include_str!("cronus_ui_css/magnetic.css");

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("<style"));
        assert!(!html.contains("@keyframes"));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("requestAnimationFrame"));
        assert!(!html.contains("data-magnetic-active"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("magnetic", "Magnet"));
        assert_eq!(
            html,
            format!("<div data-slot=\"magnetic\">{ZONES}<div data-slot=\"magnetic-target\">Magnet</div></div>")
        );
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"magnetic\""));
        assert!(html.contains("data-slot=\"magnetic-target\""));
        assert!(html.contains(">Magnet</div></div>"));
        assert_eq!(html.matches("<span class=\"z ").count(), 8);
        reject_fx(&html);
    }

    /// Docs "Magnetic call-to-action": `p-10` wrapper around a large pill CTA
    /// with the glow shadow and `px-8`.
    #[test]
    fn single_action_is_the_docs_cta() {
        let mut c = stub("magnetic", "CTA");
        c.props.insert("padding".into(), "10".into());
        c.items.push(item(
            "action",
            "Começar agora",
            &[
                ("size", "lg"),
                ("icon-end", "arrow-right"),
                ("shape", "pill"),
                ("glow", "true"),
                ("px", "8"),
            ],
        ));
        let html = render(&c);
        assert!(html.starts_with(&format!("<div data-slot=\"magnetic\" class=\"p-10\">{ZONES}<div data-slot=\"magnetic-target\"><button type=\"button\" data-slot=\"button\" data-variant=\"primary\" data-size=\"lg\" class=\"cui-btn pill glow px-8\">Começar agora<svg")));
        assert!(!html.contains("CTA"));
        reject_fx(&html);
    }

    /// Docs "Icon row": one magnetic per ghost icon button in a `gap-1` row,
    /// strength 0.25 / `p-3` on each; "Strength & radius": captioned columns.
    #[test]
    fn many_actions_make_a_row_or_a_captioned_field() {
        let mut c = stub("magnetic", "Row");
        c.props.insert("strength".into(), "0.25".into());
        c.props.insert("radius".into(), "60".into());
        c.props.insert("padding".into(), "3".into());
        for (label, icon) in [
            ("GitHub", "github"),
            ("LinkedIn", "link-2"),
            ("Compartilhar", "share-2"),
        ] {
            c.items.push(item(
                "action",
                label,
                &[
                    ("variant", "ghost"),
                    ("size", "icon"),
                    ("icon", icon),
                    ("shape", "pill"),
                ],
            ));
        }
        let html = render(&c);
        assert!(html.starts_with(&format!("<div class=\"cui-magnetic-row\"><div data-slot=\"magnetic\" class=\"p-3 s-25\">{ZONES}<div data-slot=\"magnetic-target\"><button type=\"button\" aria-label=\"GitHub\" data-slot=\"button\" data-variant=\"ghost\" data-size=\"icon\" class=\"cui-btn pill\"><svg")));
        assert_eq!(html.matches("data-slot=\"magnetic\"").count(), 3);
        reject_fx(&html);
        let mut f = stub("magnetic", "Field");
        f.props.insert("padding".into(), "8".into());
        f.items.push(item(
            "action",
            "Sutil",
            &[
                ("variant", "outline"),
                ("shape", "pill"),
                ("strength", "0.15"),
                ("radius", "80"),
                ("caption", "strength 0.15 · radius 80"),
            ],
        ));
        f.items.push(item(
            "action",
            "Grudento",
            &[
                ("variant", "outline"),
                ("shape", "pill"),
                ("strength", "0.6"),
                ("radius", "160"),
                ("caption", "strength 0.6 · radius 160"),
            ],
        ));
        let html = render(&f);
        assert!(html.starts_with(&format!("<div class=\"cui-magnetic-field\"><div><div data-slot=\"magnetic\" class=\"p-8 s-15\">{ZONES}<div data-slot=\"magnetic-target\"><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-size=\"md\" class=\"cui-btn pill\">Sutil</button></div></div><span>strength 0.15 · radius 80</span></div><div><div data-slot=\"magnetic\" class=\"p-8 s-60\">")));
        assert!(html.ends_with("<span>strength 0.6 · radius 160</span></div></div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("magnetic", "A <B> & \"C\""));
        assert!(html.ends_with(
            "<div data-slot=\"magnetic-target\">A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        ));
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("magnetic", "Magnet");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("magnetic"),
            Some("cronus_ui_magnetic::render")
        );
        assert_eq!(
            renderer_kind("magnetic"),
            RendererKind::Dedicated("cronus_ui_magnetic::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("magnetic", "Magnet"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"magnetic\""));
            assert!(html.contains("data-slot=\"magnetic-target\""));
        });
    }

    /// Hover zones in the padding ring pull the target toward the pointer on
    /// React's 300 ms overshoot curve; the pull scales with `strength`.
    #[test]
    fn chrome_magnetic_via_css() {
        // Fixture `w-72` on the inline-block wrapper; the target is a block
        // `<div>` in React (no display utility), so it spans the wrapper.
        assert!(CSS.contains("[data-slot=\"magnetic\"] {\n  display: inline-block;\n  width: var(--cui-magnetic-w, 100%);\n  color: var(--cronus-fg);\n  position: relative;"));
        assert!(CSS.contains("[data-slot=\"magnetic-target\"] {\n  display: block;\n  transition: transform 300ms cubic-bezier(0.34, 1.56, 0.64, 1);"));
        assert!(CSS.contains("[data-slot=\"magnetic\"] > .z { position: absolute; }"));
        assert!(CSS.contains("[data-slot=\"magnetic\"] > .z1 { inset-block-start: 0; inset-inline-start: 0; width: var(--cui-magnetic-pad, 0px); height: var(--cui-magnetic-pad, 0px); }"));
        assert!(CSS.contains("[data-slot=\"magnetic\"] > .z2:hover ~ [data-slot=\"magnetic-target\"] { transform: translate(0, calc(-1 * var(--cui-magnetic-pull, 7px))); }"));
        assert!(CSS.contains("[data-slot=\"magnetic\"].s-60 { --cui-magnetic-pull: 14.4px; }"));
        assert!(CSS.contains(
            "[data-slot=\"magnetic\"].p-10 { padding: 2.5rem; --cui-magnetic-pad: 2.5rem; }"
        ));
        assert!(CSS.contains(
            "[data-slot=\"magnetic\"] [data-slot=\"button\"].pill { border-radius: 9999px; }"
        ));
        assert!(CSS.contains("[data-slot=\"magnetic\"] [data-slot=\"button\"].glow { box-shadow: var(--cronus-shadow-glow"));
        assert!(
            CSS.contains(".cui-magnetic-row { display: flex; align-items: center; gap: 0.25rem; }")
        );
        assert!(CSS.contains(".cui-magnetic-field {"));
        assert!(CSS.contains("prefers-reduced-motion"));
        assert!(CSS.contains("transform: none"));
        assert!(CSS.contains("var(--cronus-fg)"));
        assert!(!CSS.contains("requestAnimationFrame"));
        assert!(!CSS.contains("data-magnetic-active"));
        assert!(!CSS.contains("zinc-"));
        assert!(!CSS.contains(FX_BOX));
        assert!(!CSS.contains("<style"));
    }
}
