//! Dedicated GlassCard renderer. DOM matches React:
//! `<div data-slot="glass-card">` + `aria-hidden` top highlight line +
//! relative children div wrapping the content. Frost (`backdrop-filter`),
//! `rounded-2xl`, `border-soft` hairline and `p-6` live in the family CSS.
//! Not the catalog `display()` SURF `<section>`.
//!
//! Also home of [`feature_content`], the docs-example content block shared by
//! the premium surfaces (spotlight-card, gradient-border, border-beam,
//! tilt-card, flip-card, reveal): an icon chip, a display heading, body copy,
//! a price row, a pill, check rows, a hint line and a call-to-action button.

use crate::cronus_ui_kit::{attr_nonempty, esc, item_icon, item_icon_end, label_of, truthy};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let card = format!(
        "<div data-slot=\"glass-card\"><div aria-hidden=\"true\"></div><div>{}</div></div>",
        feature_content(comp)
    );
    // Docs example: the frost needs something behind it — a quiet inset field
    // (`rounded-2xl border bg-surface-inset` > `p-6`).
    match attr_nonempty(comp, "backdrop") {
        Some("inset") => format!("<div class=\"cui-glass-field\"><div>{card}</div></div>"),
        _ => card,
    }
}

/// Item kinds that make up a feature block. A component with none of them
/// renders its label text alone (audit fixtures, legacy stubs).
const CONTENT_KINDS: &[&str] = &[
    "title", "text", "value", "badge", "item", "meta", "action", "button", "link",
];

fn is_content(item: &ComponentItemNode) -> bool {
    CONTENT_KINDS.contains(&item.item_type.as_str()) && !item.text.is_empty()
}

fn cfg<'a>(item: &'a ComponentItemNode, key: &str) -> Option<&'a str> {
    item.config
        .get(key)
        .map(String::as_str)
        .filter(|v| !v.is_empty())
}

/// `tone:` is parsed into the item's own `tone` field; item configs built by
/// hand may carry it as a config key.
fn tone(item: &ComponentItemNode) -> Option<&str> {
    item.tone
        .as_deref()
        .filter(|t| !t.is_empty())
        .or_else(|| cfg(item, "tone"))
}

/// The icon chip named by `icon:` — `icon-size:lg` (size-11 rounded-xl),
/// `icon-shape:round`, `icon-tone:primary|secondary`.
pub fn glyph(comp: &ComponentNode) -> Option<String> {
    // Props only: an `icon:` after an item line belongs to that row.
    let svg = crate::cronus_ui_icons::svg(comp.props.get("icon")?)?;
    let mut class = String::from("glyph");
    if attr_nonempty(comp, "icon-size") == Some("lg") {
        class.push_str(" lg");
    }
    if attr_nonempty(comp, "icon-shape") == Some("round") {
        class.push_str(" round");
    }
    match attr_nonempty(comp, "icon-tone") {
        Some("primary") => class.push_str(" primary"),
        Some("secondary") => class.push_str(" secondary"),
        _ => {}
    }
    Some(format!("<span class=\"{class}\">{svg}</span>"))
}

/// A React `<Button>` for an `action` / `button` / `link` item: `variant:`,
/// `size:`, `icon:` / `icon-end:` config and `-> "/href"` links.
pub fn action_button(item: &ComponentItemNode) -> String {
    let variant = cfg(item, "variant").unwrap_or("primary");
    let size = cfg(item, "size").unwrap_or("md");
    // Icon sizes show the glyph only; the text becomes the accessible name.
    let icon_only = size.starts_with("icon");
    let mut inner = item_icon(item);
    if !icon_only {
        inner.push_str(&esc(&item.text));
    }
    inner.push_str(&item_icon_end(item));
    crate::cronus_ui::button_html(
        &inner,
        variant,
        size,
        item.link.as_deref(),
        item.config.get("disabled").is_some_and(|v| truthy(v)),
        icon_only.then_some(item.text.as_str()),
    )
}

/// Docs-example content of a premium surface, from the component's items and
/// `icon:` prop, in declaration order:
///
/// - `icon:` prop → `<span class="glyph">` chip (first).
/// - `title "…"` → `<h3>` (`size:lg|xl|2xl|3xl|4xl|5xl`), `text "…"` → `<p>`
///   (`tone:fg`, `size:xs`); `text … mt:1` (or `mt:0`) joins the element
///   before it in a `<div class="copy">` (React's `<div><h3/><p class="mt-1"/></div>`).
/// - `value "$29" meta:"/ month"` → price row (`size:3xl`).
/// - `item "…" icon:check` → consecutive rows become one `<ul>`;
///   `item "Subtotal" value:"R$ 320,00"` rows become a `<dl>` (`tone:success`,
///   `total:true`).
/// - `meta "…"` → hint line (`eyebrow:true` for the uppercase label);
///   `badge "…"` → pill (`tone:success`), paired in a space-between `row`
///   with the glyph or `meta` right before it.
/// - `action` / `button` / `link "…"` → a React button (see [`action_button`]).
pub fn feature_content(comp: &ComponentNode) -> String {
    let items: Vec<&ComponentItemNode> = comp.items.iter().filter(|i| is_content(i)).collect();
    if items.is_empty() {
        return match glyph(comp) {
            Some(g) => format!("{g}{}", label_of(comp)),
            None => label_of(comp),
        };
    }
    // Top-level elements; some items fold the previous one into a group.
    let mut out: Vec<String> = Vec::new();
    if let Some(g) = glyph(comp) {
        out.push(g);
    }
    let mut i = 0;
    while i < items.len() {
        let it = items[i];
        match it.item_type.as_str() {
            "title" => {
                let class = match cfg(it, "size") {
                    Some(s @ ("lg" | "xl" | "2xl" | "3xl" | "4xl" | "5xl")) => {
                        format!(" class=\"t-{s}\"")
                    }
                    _ => String::new(),
                };
                out.push(format!("<h3{class}>{}</h3>", esc(&it.text)));
            }
            "text" => {
                let mut classes: Vec<&str> = Vec::new();
                match (tone(it), cfg(it, "size")) {
                    (Some("fg"), _) => classes.push("fg"),
                    (_, Some("xs")) => classes.push("xs"),
                    _ => {}
                }
                // `mt:1` / `mt:0`: React's `<div><h3/><p class="mt-1"/></div>` grouping.
                let attached = matches!(cfg(it, "mt"), Some("0" | "1")) && !out.is_empty();
                if cfg(it, "mt") == Some("1") && attached {
                    classes.push("mt-1");
                }
                let class = if classes.is_empty() {
                    String::new()
                } else {
                    format!(" class=\"{}\"", classes.join(" "))
                };
                let p = format!("<p{class}>{}</p>", esc(&it.text));
                if attached {
                    let prev = out.pop().unwrap_or_default();
                    out.push(format!("<div class=\"copy\">{prev}{p}</div>"));
                } else {
                    out.push(p);
                }
            }
            "value" => {
                let class = match cfg(it, "size") {
                    Some("3xl") => " class=\"price v-3xl\"",
                    _ => " class=\"price\"",
                };
                let meta = cfg(it, "meta")
                    .map(|m| format!("<span>{}</span>", esc(m)))
                    .unwrap_or_default();
                out.push(format!(
                    "<div{class}><span>{}</span>{meta}</div>",
                    esc(&it.text)
                ));
            }
            "badge" => {
                let class = match tone(it) {
                    Some(t @ ("success" | "warning" | "error" | "info" | "primary")) => {
                        format!("pill {t}")
                    }
                    _ => "pill".into(),
                };
                let pill = format!("<span class=\"{class}\">{}</span>", esc(&it.text));
                match out.last() {
                    Some(prev)
                        if prev.starts_with("<span class=\"glyph")
                            || prev.starts_with("<span class=\"hint") =>
                    {
                        let prev = out.pop().unwrap_or_default();
                        out.push(format!("<div class=\"row\">{prev}{pill}</div>"));
                    }
                    _ => out.push(pill),
                }
            }
            "item" => {
                let described = cfg(it, "value").is_some();
                let mut rows = String::new();
                while i < items.len() && items[i].item_type == "item" {
                    let row = items[i];
                    if described {
                        let mut class: Vec<&str> = Vec::new();
                        if let Some(t @ ("success" | "error" | "warning")) = tone(row) {
                            class.push(t);
                        }
                        if cfg(row, "total").is_some_and(truthy) {
                            class.push("total");
                        }
                        let class = if class.is_empty() {
                            String::new()
                        } else {
                            format!(" class=\"{}\"", class.join(" "))
                        };
                        rows.push_str(&format!(
                            "<div{class}><dt>{}</dt><dd>{}</dd></div>",
                            esc(&row.text),
                            esc(cfg(row, "value").unwrap_or(""))
                        ));
                    } else {
                        rows.push_str(&format!("<li>{}{}</li>", item_icon(row), esc(&row.text)));
                    }
                    i += 1;
                }
                out.push(if described {
                    format!("<dl class=\"rows\">{rows}</dl>")
                } else {
                    format!("<ul class=\"rows\">{rows}</ul>")
                });
                continue;
            }
            "meta" => {
                let class = if cfg(it, "eyebrow").is_some_and(truthy) {
                    "hint eyebrow"
                } else {
                    "hint"
                };
                out.push(format!("<span class=\"{class}\">{}</span>", esc(&it.text)));
            }
            "action" | "button" | "link" => {
                out.push(format!("<div class=\"cta\">{}</div>", action_button(it)));
            }
            _ => {}
        }
        i += 1;
    }
    out.concat()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem;";
    const CSS: &str = include_str!("cronus_ui_css/glass-card.css");

    fn reject_display(html: &str) {
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    pub(crate) fn item(kind: &str, text: &str, cfg: &[(&str, &str)]) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: cfg
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    #[test]
    fn root_wraps_highlight_and_label_not_display_surf_section() {
        let html = render(&stub("glass-card", "Frost"));
        assert_eq!(
            html,
            "<div data-slot=\"glass-card\"><div aria-hidden=\"true\"></div><div>Frost</div></div>"
        );
        assert_eq!(html.matches("data-slot=").count(), 1);
        reject_display(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("glass-card", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"glass-card\"><div aria-hidden=\"true\"></div><div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        reject_display(&html);
    }

    /// Docs "Frosted surface": icon chip + display heading + body copy over an
    /// inset field.
    #[test]
    fn docs_frosted_surface_content_and_backdrop() {
        let mut c = stub("glass-card", "Frosted");
        c.props.insert("icon".into(), "sparkles".into());
        c.props.insert("backdrop".into(), "inset".into());
        c.items.push(item("title", "Premium by default", &[]));
        c.items
            .push(item("text", "Frosted blur ships out of the box.", &[]));
        let html = render(&c);
        assert!(
            html.starts_with("<div class=\"cui-glass-field\"><div><div data-slot=\"glass-card\">")
        );
        assert!(html.contains("<div><span class=\"glyph\"><svg"));
        assert!(html.contains("data-icon=\"sparkles\""));
        assert!(html.contains("</span><h3>Premium by default</h3><p>Frosted blur ships out of the box.</p></div></div></div></div>"));
        assert!(!html.contains("Frosted<"));
        reject_display(&html);
    }

    #[test]
    fn feature_content_renders_every_kind_in_order() {
        let mut c = stub("glass-card", "Pro");
        c.props.insert("icon".into(), "sparkles".into());
        c.props.insert("icon-size".into(), "lg".into());
        c.props.insert("icon-tone".into(), "primary".into());
        c.items.push(item("badge", "Popular", &[]));
        c.items.push(item("title", "Pro", &[("size", "lg")]));
        c.items.push(item("text", "Everything.", &[("tone", "fg")]));
        c.items.push(item(
            "value",
            "R$ 79",
            &[("meta", "/ mês"), ("size", "3xl")],
        ));
        c.items
            .push(item("item", "Repasses em D+2", &[("icon", "check")]));
        c.items.push(item("item", "Suporte", &[("icon", "check")]));
        c.items.push(item("meta", "Passe o mouse →", &[]));
        let mut cta = item(
            "action",
            "Assinar",
            &[("icon-end", "arrow-right"), ("size", "sm")],
        );
        cta.link = Some("/pro".into());
        c.items.push(cta);
        let html = feature_content(&c);
        assert!(html.starts_with("<div class=\"row\"><span class=\"glyph lg primary\"><svg"));
        assert!(html.contains("</span><span class=\"pill\">Popular</span></div><h3 class=\"t-lg\">Pro</h3><p class=\"fg\">Everything.</p><div class=\"price v-3xl\"><span>R$ 79</span><span>/ mês</span></div><ul class=\"rows\"><li><svg"));
        assert_eq!(html.matches("<li>").count(), 2);
        assert!(html.contains("</ul><span class=\"hint\">Passe o mouse →</span><div class=\"cta\"><a href=\"/pro\" data-slot=\"button\" data-variant=\"primary\" data-size=\"sm\" class=\"cui-btn\">Assinar<svg"));
        assert_eq!(html.matches("Popular").count(), 1);
    }

    #[test]
    fn icon_size_buttons_are_glyph_only_with_an_accessible_name() {
        let send = item(
            "action",
            "Enviar",
            &[("size", "icon"), ("icon", "arrow-right")],
        );
        let html = action_button(&send);
        assert!(html.starts_with("<button type=\"button\" aria-label=\"Enviar\" data-slot=\"button\" data-variant=\"primary\" data-size=\"icon\" class=\"cui-btn\"><svg"));
        assert!(!html.contains(">Enviar<"));
    }

    /// `mt:1` copy joins the heading before it; `meta` + `badge` make a row;
    /// `value:` rows are a description list with tone and total classes.
    #[test]
    fn attached_copy_eyebrow_rows_and_description_lists() {
        let mut c = stub("glass-card", "Order");
        c.items
            .push(item("meta", "Pedido #4821", &[("eyebrow", "true")]));
        c.items.push(item("badge", "Pago", &[("tone", "success")]));
        c.items.push(item("value", "R$ 297,00", &[("size", "3xl")]));
        c.items
            .push(item("text", "Curso de Copywriting", &[("mt", "1")]));
        c.items
            .push(item("text", "Toque em detalhes.", &[("size", "xs")]));
        c.items
            .push(item("item", "Subtotal", &[("value", "R$ 320,00")]));
        c.items.push(item(
            "item",
            "Cupom",
            &[("value", "− R$ 23,00"), ("tone", "success")],
        ));
        c.items.push(item(
            "item",
            "Total",
            &[("value", "R$ 297,00"), ("total", "true")],
        ));
        let html = feature_content(&c);
        assert_eq!(
            html,
            "<div class=\"row\"><span class=\"hint eyebrow\">Pedido #4821</span><span class=\"pill success\">Pago</span></div><div class=\"copy\"><div class=\"price v-3xl\"><span>R$ 297,00</span></div><p class=\"mt-1\">Curso de Copywriting</p></div><p class=\"xs\">Toque em detalhes.</p><dl class=\"rows\"><div><dt>Subtotal</dt><dd>R$ 320,00</dd></div><div class=\"success\"><dt>Cupom</dt><dd>− R$ 23,00</dd></div><div class=\"total\"><dt>Total</dt><dd>R$ 297,00</dd></div></dl>"
        );
    }

    #[test]
    fn badge_without_icon_is_a_plain_pill_and_buttons_default_primary_md() {
        let mut c = stub("glass-card", "x");
        c.items.push(item("badge", "New", &[]));
        c.items
            .push(item("button", "Go", &[("variant", "outline")]));
        let html = feature_content(&c);
        assert_eq!(html.matches("<span class=\"pill\">New</span>").count(), 1);
        assert!(html.contains("<button type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-size=\"md\" class=\"cui-btn\">Go</button>"));
        assert!(!html.contains("glyph"));
    }

    #[test]
    fn chrome_glass_via_css() {
        assert!(CSS.contains("backdrop-filter: blur(24px)"));
        assert!(CSS.contains("var(--cronus-surface-raised)"));
        assert!(CSS.contains("var(--cronus-shadow-lg"));
        assert!(CSS.contains("[data-slot=\"glass-card\"] > [aria-hidden] {"));
        assert!(CSS.contains("[data-slot=\"glass-card\"] > div:last-child {"));
        assert!(!CSS.contains("[data-slot=\"glass-card\"]::before"));
        assert!(!CSS.contains("zinc-"));
        assert!(!CSS.contains(DISPLAY_SURF));
    }

    /// Wave 1t geometry: React `relative rounded-2xl border border-border-soft p-6`
    /// + fixture `w-72` → 288×74, radius 22px, 1px border.
    #[test]
    fn chrome_matches_react_box() {
        let start = CSS.find("[data-slot=\"glass-card\"] {").unwrap();
        let block = &CSS[start..start + CSS[start..].find('}').unwrap()];
        assert!(block.contains("position: relative;"));
        assert!(block.contains(
            "width: var(--cui-glass-card-w, 100%); box-sizing: border-box; padding: 1.5rem;"
        ));
        assert!(block.contains("border-radius: calc(var(--cronus-radius, 14px) + 8px);"));
        assert!(
            block.contains("border: 1px solid var(--cronus-border-soft, var(--cronus-border));")
        );
    }

    /// Docs example chrome: `size-9 rounded-lg bg-surface-overlay` chip,
    /// `font-display text-base font-semibold` heading, `text-sm fg-secondary`
    /// copy, and the `bg-surface-inset` field with `p-6`.
    #[test]
    fn chrome_feature_content_and_inset_field() {
        assert!(CSS.contains("[data-slot=\"glass-card\"] .glyph {"));
        assert!(CSS.contains("width: 2.25rem; height: 2.25rem;"));
        assert!(CSS.contains("[data-slot=\"glass-card\"] h3 {"));
        assert!(CSS.contains("font-family: var(--cronus-font-display"));
        assert!(CSS.contains("[data-slot=\"glass-card\"] p {"));
        assert!(CSS.contains("color: var(--cronus-fg-secondary)"));
        assert!(CSS.contains(".cui-glass-field {"));
        assert!(CSS.contains("background: var(--cronus-surface-inset);"));
        assert!(CSS.contains(".cui-glass-field > div { position: relative; padding: 1.5rem; }"));
    }
}
