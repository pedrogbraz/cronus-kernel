//! Dedicated FlipCard renderer. DOM matches React:
//! `<div data-slot="flip-card">` > preserve-3d stage `<div>` > `flip-card-front`
//! + `flip-card-back`. Static CSS 3D in the family CSS; no JS.
//!
//! Faces: `slot "front"` / `slot "back"` lines split the items into the two
//! faces, each rendered by `cronus_ui_glass_card::feature_content` (the
//! `icon:` prop belongs to the front). Slot config: `layout:center`
//! (`items-center justify-center text-center`), `avatar:"AR"` (size-16 round
//! chip), `rating:5` (filled stars), `icons:"github,link-2"` (glyph row).
//! Without slots, `text` / `item` lines are the faces (`items[0]` /
//! `items[1]`, the audit emitter's shape) and the `label` names the card.
//!
//! Triggers — `trigger:hover` (default): `tabindex="0"` (+ `role="group"` when
//! named), CSS `:hover` / `:focus-within` turn the stage. `trigger:click`:
//! the card sits in a `<label>` with a visually hidden checkbox (React's
//! `role="button"` + `aria-pressed`), `:has(:checked)` turns it.
//! `trigger:controlled`: inert card; an `action` item (React's toggle button,
//! parent state) becomes the same checkbox pattern below the card, its label
//! swapping with `swap:"…"`. `axis:vertical` tumbles on `rotateX`.
//! Not the catalog `display()` SURF `<section>`.

use crate::cronus_ui_glass_card::feature_content;
use crate::cronus_ui_kit::{attr, attr_nonempty, esc, instance_id, label_of};
use crate::parser::{ComponentItemNode, ComponentNode};

fn cfg<'a>(i: &'a ComponentItemNode, key: &str) -> Option<&'a str> {
    i.config
        .get(key)
        .map(String::as_str)
        .filter(|v| !v.is_empty())
}

/// A face's markup: slot extras (avatar / stars / links) around the feature
/// content of the items that belong to it.
fn face(
    comp: &ComponentNode,
    slot: Option<&ComponentItemNode>,
    items: Vec<ComponentItemNode>,
    front: bool,
) -> String {
    let mut clone = comp.clone();
    clone.items = items;
    if !front {
        clone.props.remove("icon");
    }
    let mut out = String::new();
    if let Some(a) = slot.and_then(|s| cfg(s, "avatar")) {
        out.push_str(&format!("<span class=\"avatar\">{}</span>", esc(a)));
    }
    if let Some(n) = slot
        .and_then(|s| cfg(s, "rating"))
        .and_then(|n| n.trim().parse::<u32>().ok())
    {
        let star = crate::cronus_ui_icons::svg_or_empty("star");
        out.push_str(&format!(
            "<div class=\"stars\">{}</div>",
            star.repeat(n.min(5) as usize)
        ));
    }
    out.push_str(&feature_content(&clone));
    if let Some(names) = slot.and_then(|s| cfg(s, "icons")) {
        let glyphs: String = names
            .split(',')
            .map(|n| crate::cronus_ui_icons::svg_or_empty(n.trim()))
            .collect();
        out.push_str(&format!("<div class=\"links\">{glyphs}</div>"));
    }
    out
}

fn face_attr(slot: Option<&ComponentItemNode>) -> &'static str {
    match slot.and_then(|s| cfg(s, "layout")) {
        Some("center") => " class=\"center\"",
        _ => "",
    }
}

/// `(front attrs, front html, back attrs, back html)`.
fn faces(comp: &ComponentNode) -> (String, String, String, String) {
    if !comp.items.iter().any(|i| i.item_type == "slot") {
        let faces: Vec<String> = comp
            .items
            .iter()
            .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
            .map(|i| esc(&i.text))
            .collect();
        let (front, back) = match faces.as_slice() {
            [f, b, ..] => (f.clone(), b.clone()),
            [b] => (label_of(comp), b.clone()),
            [] => (label_of(comp), String::new()),
        };
        return (String::new(), front, String::new(), back);
    }
    let controlled = attr(comp, "trigger") == Some("controlled");
    let mut front_items: Vec<ComponentItemNode> = Vec::new();
    let mut back_items: Vec<ComponentItemNode> = Vec::new();
    let mut front_slot: Option<&ComponentItemNode> = None;
    let mut back_slot: Option<&ComponentItemNode> = None;
    let mut on_back = false;
    for i in &comp.items {
        if i.item_type == "slot" {
            on_back = i.text.trim().eq_ignore_ascii_case("back");
            if on_back {
                back_slot = Some(i);
            } else {
                front_slot = Some(i);
            }
            continue;
        }
        // The controlled example's `action` is the parent's toggle, not face content.
        if controlled && matches!(i.item_type.as_str(), "action" | "button") {
            continue;
        }
        if on_back {
            back_items.push(i.clone());
        } else {
            front_items.push(i.clone());
        }
    }
    (
        face_attr(front_slot).to_string(),
        face(comp, front_slot, front_items, true),
        face_attr(back_slot).to_string(),
        face(comp, back_slot, back_items, false),
    )
}

pub fn render(comp: &ComponentNode) -> String {
    let (front_attr, front, back_attr, back) = faces(comp);
    // `aria-label:` after an item line lands in that item's config (the
    // tokenizer has no newlines), so look there as well as in props.
    let aria_label = attr(comp, "aria-label").filter(|s| !s.is_empty());
    let trigger = attr(comp, "trigger").unwrap_or("hover");
    let mut classes: Vec<&str> = Vec::new();
    if attr(comp, "axis") == Some("vertical") {
        classes.push("axis-y");
    }
    match attr_nonempty(comp, "width") {
        Some("xs") => classes.push("w-xs"),
        Some("sm") => classes.push("w-sm"),
        Some("md") => classes.push("w-md"),
        _ => {}
    }
    match attr_nonempty(comp, "height") {
        Some("64") => classes.push("h-64"),
        Some("72") => classes.push("h-72"),
        _ => {}
    }
    let class = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    let stage = format!(
        "<div><div data-slot=\"flip-card-front\"{front_attr}>{front}</div><div data-slot=\"flip-card-back\"{back_attr}>{back}</div></div>"
    );
    match trigger {
        "click" => {
            let name = aria_label
                .map(|v| format!(" aria-label=\"{}\"", esc(v)))
                .unwrap_or_default();
            format!(
                "<label class=\"cui-flip-card-toggle\"><input type=\"checkbox\"{name}><div data-slot=\"flip-card\"{class} role=\"button\" aria-pressed=\"false\" aria-hidden=\"true\">{stage}</div></label>"
            )
        }
        "controlled" => {
            let name = aria_label
                .map(|v| format!(" role=\"group\" aria-label=\"{}\"", esc(v)))
                .unwrap_or_default();
            let card = format!("<div data-slot=\"flip-card\"{class}{name}>{stage}</div>");
            let toggle = comp.items.iter().find(|i| {
                matches!(i.item_type.as_str(), "action" | "button") && !i.text.is_empty()
            });
            match toggle {
                Some(toggle) => {
                    let id = instance_id(comp, "flip");
                    let swap = cfg(toggle, "swap")
                        .map(esc)
                        .unwrap_or_else(|| esc(&toggle.text));
                    let mut inner = crate::cronus_ui_kit::item_icon(toggle);
                    inner.push_str(&format!(
                        "<span>{}</span><span>{swap}</span>",
                        esc(&toggle.text)
                    ));
                    let button = crate::cronus_ui::button_html(
                        &inner,
                        cfg(toggle, "variant").unwrap_or("outline"),
                        cfg(toggle, "size").unwrap_or("sm"),
                        None,
                        false,
                        None,
                    )
                    .replacen(
                        "<button type=\"button\"",
                        "<button type=\"button\" tabindex=\"-1\" aria-hidden=\"true\"",
                        1,
                    );
                    format!(
                        "<div class=\"cui-flip-card-demo\">{card}<label class=\"cui-flip-card-toggle\"><input type=\"checkbox\" id=\"{id}\" aria-label=\"{}\">{button}</label></div>",
                        esc(&toggle.text)
                    )
                }
                None => card,
            }
        }
        _ => {
            // React hover trigger: `role="group"` only when named, always `tabindex=0`
            // (CSS `:focus-within` flips it, so focus is a real, JS-free control).
            let aria = match aria_label {
                Some(v) => format!(" role=\"group\" tabindex=\"0\" aria-label=\"{}\"", esc(v)),
                None => " tabindex=\"0\"".to_string(),
            };
            format!("<div data-slot=\"flip-card\"{class}{aria}>{stage}</div>")
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem;";
    const CSS: &str = include_str!("cronus_ui_css/flip-card.css");

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn with(kind: &str, text: &str, cfg: &[(&str, &str)]) -> ComponentItemNode {
        let mut i = extra(kind, text);
        for (k, v) in cfg {
            i.config.insert(k.to_string(), v.to_string());
        }
        i
    }

    fn reject_display(html: &str) {
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("onmouseenter"));
        assert!(!html.contains("addEventListener"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("display("));
    }

    #[test]
    fn root_is_flip_card_with_front_label_and_back() {
        let html = render(&stub("flip-card", "Front"));
        assert_eq!(
            html,
            "<div data-slot=\"flip-card\" tabindex=\"0\"><div><div data-slot=\"flip-card-front\">Front</div><div data-slot=\"flip-card-back\"></div></div></div>"
        );
        assert!(html.starts_with("<div data-slot=\"flip-card\""));
        assert!(html.contains("data-slot=\"flip-card-front\">Front</div>"));
        assert!(html.contains("data-slot=\"flip-card-back\"></div>"));
        reject_display(&html);
    }

    #[test]
    fn extra_text_becomes_back_face() {
        let mut c = stub("flip-card", "Front");
        c.items.push(extra("text", "Back copy"));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"flip-card\" tabindex=\"0\"><div><div data-slot=\"flip-card-front\">Front</div><div data-slot=\"flip-card-back\">Back copy</div></div></div>"
        );
        reject_display(&html);
    }

    /// Audit fixture shape: emitter writes `label "Plan"` (from aria-label),
    /// `text "Front"`, `text "Back"`, `aria-label:"Plan"`. React faces are
    /// items[0] / items[1]; "Plan" only names the card.
    #[test]
    fn fixture_items_are_faces_label_is_aria() {
        let mut c = stub("flip-card", "Plan");
        c.items.push(extra("text", "Front"));
        let mut back = extra("text", "Back");
        back.config.insert("aria-label".into(), "Plan".into());
        c.items.push(back);
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"flip-card\" role=\"group\" tabindex=\"0\" aria-label=\"Plan\"><div><div data-slot=\"flip-card-front\">Front</div><div data-slot=\"flip-card-back\">Back</div></div></div>"
        );
        reject_display(&html);
    }

    /// Docs "Hover to flip": chip + copy + hint on the front, check rows and a
    /// full-width CTA on the back, `h-72 max-w-xs`.
    #[test]
    fn slots_split_the_faces() {
        let mut c = stub("flip-card", "Plano Pro");
        c.props.insert("aria-label".into(), "Plano Pro".into());
        c.props.insert("icon".into(), "sparkles".into());
        c.props.insert("icon-size".into(), "lg".into());
        c.props.insert("width".into(), "xs".into());
        c.props.insert("height".into(), "72".into());
        c.items.push(with("title", "Plano Pro", &[("size", "lg")]));
        c.items
            .push(with("text", "Tudo o que você precisa.", &[("mt", "1")]));
        c.items.push(extra("meta", "Passe o mouse →"));
        c.items.push(extra("slot", "back"));
        c.items
            .push(with("item", "Repasses em D+2", &[("icon", "check")]));
        c.items
            .push(with("item", "Checkout sem marca", &[("icon", "check")]));
        c.items.push(with(
            "action",
            "Assinar o Pro",
            &[("icon-end", "arrow-right")],
        ));
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"flip-card\" class=\"w-xs h-72\" role=\"group\" tabindex=\"0\" aria-label=\"Plano Pro\"><div><div data-slot=\"flip-card-front\"><span class=\"glyph lg\"><svg"));
        assert!(html.contains("</span><div class=\"copy\"><h3 class=\"t-lg\">Plano Pro</h3><p class=\"mt-1\">Tudo o que você precisa.</p></div><span class=\"hint\">Passe o mouse →</span></div><div data-slot=\"flip-card-back\"><ul class=\"rows\"><li><svg"));
        assert!(html.contains("</ul><div class=\"cta\"><button type=\"button\" data-slot=\"button\" data-variant=\"primary\" data-size=\"md\" class=\"cui-btn\">Assinar o Pro<svg"));
        assert_eq!(html.matches("class=\"glyph").count(), 1);
        reject_display(&html);
    }

    /// Docs "Click to flip": the card is a labelled checkbox toggle; centred
    /// faces with an avatar, five stars, a quote and social glyphs.
    #[test]
    fn click_trigger_is_a_checkbox_label() {
        let mut c = stub("flip-card", "Ana");
        c.props.insert("trigger".into(), "click".into());
        c.props
            .insert("aria-label".into(), "Ver depoimento de Ana Ribeiro".into());
        c.items.push(with(
            "slot",
            "front",
            &[("layout", "center"), ("avatar", "AR")],
        ));
        c.items.push(extra("title", "Ana Ribeiro"));
        c.items
            .push(with("text", "Head of Design, Northwind", &[("mt", "0")]));
        c.items.push(with(
            "slot",
            "back",
            &[
                ("layout", "center"),
                ("rating", "5"),
                ("icons", "github,link-2"),
            ],
        ));
        c.items
            .push(with("text", "“Shipped a polished UI.”", &[("tone", "fg")]));
        let html = render(&c);
        assert!(html.starts_with("<label class=\"cui-flip-card-toggle\"><input type=\"checkbox\" aria-label=\"Ver depoimento de Ana Ribeiro\"><div data-slot=\"flip-card\" role=\"button\" aria-pressed=\"false\" aria-hidden=\"true\"><div><div data-slot=\"flip-card-front\" class=\"center\"><span class=\"avatar\">AR</span><div class=\"copy\"><h3>Ana Ribeiro</h3><p>Head of Design, Northwind</p></div></div><div data-slot=\"flip-card-back\" class=\"center\"><div class=\"stars\"><svg"));
        assert_eq!(html.matches("data-icon=\"star\"").count(), 5);
        assert!(html.contains(
            "</div><p class=\"fg\">“Shipped a polished UI.”</p><div class=\"links\"><svg"
        ));
        assert!(html.contains("data-icon=\"github\""));
        assert!(html.contains("data-icon=\"link-2\""));
        assert!(html.ends_with("</div></div></div></div></label>"));
        assert!(!html.contains("tabindex"));
        reject_display(&html);
    }

    /// Docs "Controlled": inert vertical-axis card, the parent's toggle button
    /// is a checkbox label below it whose text swaps when flipped.
    #[test]
    fn controlled_trigger_uses_the_action_as_toggle() {
        crate::cronus_ui_kit::reset_instance_ids();
        let mut c = stub("flip-card", "Pedido");
        c.props.insert("trigger".into(), "controlled".into());
        c.props.insert("axis".into(), "vertical".into());
        c.props
            .insert("aria-label".into(), "Detalhe do pedido".into());
        c.items
            .push(with("meta", "Pedido #4821", &[("eyebrow", "true")]));
        c.items.push(with("badge", "Pago", &[("tone", "success")]));
        c.items.push(extra("slot", "back"));
        c.items
            .push(with("item", "Subtotal", &[("value", "R$ 320,00")]));
        c.items.push(with(
            "action",
            "Ver detalhes",
            &[("swap", "Ver resumo"), ("icon", "rotate-cw")],
        ));
        let html = render(&c);
        assert!(html.starts_with("<div class=\"cui-flip-card-demo\"><div data-slot=\"flip-card\" class=\"axis-y\" role=\"group\" aria-label=\"Detalhe do pedido\"><div><div data-slot=\"flip-card-front\"><div class=\"row\"><span class=\"hint eyebrow\">Pedido #4821</span><span class=\"pill success\">Pago</span></div></div><div data-slot=\"flip-card-back\"><dl class=\"rows\"><div><dt>Subtotal</dt><dd>R$ 320,00</dd></div></dl></div></div></div><label class=\"cui-flip-card-toggle\"><input type=\"checkbox\" id=\"cui-flip-card-flip\" aria-label=\"Ver detalhes\"><button type=\"button\" tabindex=\"-1\" aria-hidden=\"true\" data-slot=\"button\" data-variant=\"outline\" data-size=\"sm\" class=\"cui-btn\"><svg"));
        assert!(html.ends_with(
            "</svg><span>Ver detalhes</span><span>Ver resumo</span></button></label></div>"
        ));
        assert!(!html.contains("Ver detalhes</dd>"));
        reject_display(&html);
    }

    #[test]
    fn label_is_escaped() {
        let mut c = stub("flip-card", "A <B> & \"C\"");
        c.items.push(extra("text", "D <E>"));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"flip-card\" tabindex=\"0\"><div><div data-slot=\"flip-card-front\">A &lt;B&gt; &amp; &quot;C&quot;</div><div data-slot=\"flip-card-back\">D &lt;E&gt;</div></div></div>"
        );
        reject_display(&html);
    }

    #[test]
    fn skips_display_surf_section() {
        let html = render(&stub("flip-card", "Front"));
        assert!(!html.contains("<section"));
        reject_display(&html);
        assert_eq!(
            dedicated_fn_name("flip-card"),
            Some("cronus_ui_flip_card::render")
        );
        assert_eq!(
            renderer_kind("flip-card"),
            RendererKind::Dedicated("cronus_ui_flip_card::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("flip-card", "Front"));
            reject_display(&html);
            assert!(html.contains("data-slot=\"flip-card-front\""));
            assert!(html.contains("data-slot=\"flip-card-back\""));
        });
    }

    #[test]
    fn chrome_flip_via_css() {
        assert!(CSS.contains("[data-slot=\"flip-card\"]"));
        assert!(CSS.contains("[data-slot=\"flip-card-front\"]"));
        assert!(CSS.contains("[data-slot=\"flip-card-back\"]"));
        assert!(CSS.contains("perspective: 1600px"));
        assert!(CSS.contains("transform-style: preserve-3d"));
        assert!(CSS.contains("backface-visibility: hidden"));
        assert!(CSS.contains("rotateY(180deg)"));
        assert!(CSS.contains("var(--cronus-surface-raised)"));
        assert!(CSS.contains("var(--cronus-surface-elevated"));
        assert!(CSS.contains("var(--cronus-border)"));
        assert!(!CSS.contains("zinc-"));
        assert!(!CSS.contains(DISPLAY_SURF));
        assert!(!CSS.contains("onclick"));
    }

    /// Wave 1s geometry parity with the corrected React `FlipCard
    /// className="w-72"`: root 288×256 rounded-2xl (22px); the preserve-3d
    /// stage is `absolute inset-0` (was `size-full`, which resolved to 0px
    /// against a min-height-only card), so both faces measure 288×256.
    #[test]
    fn chrome_geometry_matches_react() {
        assert!(CSS.contains(
            "[data-slot=\"flip-card\"] {\n  position: relative; isolation: isolate;\n  width: var(--cui-flip-card-w, 100%);\n  min-height: 16rem; border-radius: calc(var(--cronus-radius, 14px) + 8px);\n  perspective: 1600px;\n  line-height: 1.5;"
        ));
        assert!(CSS.contains(
            "[data-slot=\"flip-card\"] > div {\n  position: absolute; inset: 0;\n  transform-style: preserve-3d;"
        ));
        assert!(!CSS.contains(
            "position: relative; width: 100%; height: 100%;\n  transform-style: preserve-3d;"
        ));
        assert!(CSS.contains(
            "[data-slot=\"flip-card\"]:hover > div,\n[data-slot=\"flip-card\"]:focus-within > div {\n  transform: rotateY(180deg);\n}"
        ));
        assert!(CSS.contains(
            "overflow: hidden; border-radius: calc(var(--cronus-radius, 14px) + 8px);\n  border: 1px solid var(--cronus-border);"
        ));
        assert!(!CSS.contains("transform: rotateY(360deg)"));
        assert!(!CSS.contains("min-height: 16rem; border-radius: var(--cronus-radius-xl);"));
    }

    /// Click / controlled toggles turn the stage from `:has(:checked)`; the
    /// vertical axis uses `rotateX`; faces carry the docs `p-6` layouts.
    #[test]
    fn chrome_toggles_axis_and_face_layouts() {
        assert!(CSS.contains(".cui-flip-card-toggle:has(> input:checked) [data-slot=\"flip-card\"] > div,\n.cui-flip-card-demo:has(> .cui-flip-card-toggle > input:checked) [data-slot=\"flip-card\"] > div { transform: rotateY(180deg); }"));
        assert!(
            CSS.contains("[data-slot=\"flip-card\"].axis-y > div { transform: rotateX(180deg); }")
        );
        assert!(CSS.contains("[data-slot=\"flip-card\"].axis-y [data-slot=\"flip-card-back\"] { transform: rotateX(180deg); }"));
        assert!(CSS.contains(".cui-flip-card-toggle > input:checked + [data-slot=\"button\"] > span:nth-of-type(1),\n.cui-flip-card-toggle > input:not(:checked) + [data-slot=\"button\"] > span:nth-of-type(2) { display: none; }"));
        assert!(CSS.contains("[data-slot=\"flip-card-front\"], [data-slot=\"flip-card-back\"] {\n  justify-content: space-between; padding: 1.5rem;\n}"));
        assert!(CSS.contains(
            "[data-slot=\"flip-card-front\"].center, [data-slot=\"flip-card-back\"].center {"
        ));
        assert!(CSS.contains("[data-slot=\"flip-card\"].w-xs { max-width: 20rem; }"));
        assert!(CSS.contains("[data-slot=\"flip-card\"].h-72 { height: 18rem; }"));
        assert!(CSS.contains("[data-slot=\"flip-card\"] .avatar {"));
        assert!(CSS.contains("[data-slot=\"flip-card\"] .stars svg { width: 1rem; height: 1rem; fill: currentColor; }"));
        assert!(CSS.contains("[data-slot=\"flip-card\"] dl.rows > div.total {"));
    }
}
