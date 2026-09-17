//! Dedicated DescriptionList renderer. DOM matches React:
//! `<dl data-slot="description-list">` of `description-item` groups wrapping
//! `description-term` / `description-details` — or, with `raw:true`, the bare
//! alternating `<dt>`/`<dd>` pairs the docs' stacked example composes.
//! Pairs come from `item "Term" details:"…"` lines (`badge:<variant>` renders
//! the details as a Badge, `font:mono` as the docs' mono tabular span) or from
//! paired content texts (odd = term, even = details). The `label` names the
//! widget (`aria-label` fallback) and is never a term.
//!
//! React's `descriptionListVariants` have no data attributes, so they travel
//! as classes on the `<dl>`: `layout` (`l-horizontal` | `l-grid`; stacked is
//! the default), `size:sm` (`s-sm`), `striped`, `bordered`, `details-align:end`
//! (`align-end`), plus `max-width:` (`mw-*`, the docs' `max-w-*`). Not interact
//! styled `<dl>` rows without term/details slots, not catalog `display()`.

use crate::cronus_ui_kit::{attr_nonempty, choice, esc, flag};
use crate::parser::ComponentNode;

/// Item kinds that name the widget rather than carry list content.
const NAME_KINDS: &[&str] = &["label", "title"];

struct Pair {
    term: String,
    details: String,
}

pub fn render(comp: &ComponentNode) -> String {
    let raw = flag(comp, "raw");
    let rows = pairs(comp)
        .into_iter()
        .map(|p| {
            let dt = format!("<dt data-slot=\"description-term\">{}</dt>", p.term);
            let dd = format!("<dd data-slot=\"description-details\">{}</dd>", p.details);
            if raw {
                format!("{dt}{dd}")
            } else {
                format!("<div data-slot=\"description-item\">{dt}{dd}</div>")
            }
        })
        .collect::<Vec<_>>()
        .join("");
    let mut classes: Vec<String> = Vec::new();
    if let Some(layout) = choice(comp, "layout", &["horizontal", "grid"]) {
        classes.push(format!("l-{layout}"));
    }
    if choice(comp, "size", &["sm"]).is_some() {
        classes.push("s-sm".into());
    }
    if flag(comp, "striped") {
        classes.push("striped".into());
    }
    if flag(comp, "bordered") {
        classes.push("bordered".into());
    }
    if attr_nonempty(comp, "details-align")
        .or_else(|| attr_nonempty(comp, "detailsAlign"))
        .is_some_and(|a| a.trim() == "end")
    {
        classes.push("align-end".into());
    }
    if let Some(mw) = crate::cronus_ui_card::max_width_class(comp) {
        classes.push(mw.into());
    }
    let class = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    let aria = aria_label(comp)
        .map(|v| format!(" aria-label=\"{}\"", esc(v)))
        .unwrap_or_default();
    format!("<dl data-slot=\"description-list\"{class}{aria}>{rows}</dl>")
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    attr_nonempty(comp, "aria-label")
}

fn pairs(comp: &ComponentNode) -> Vec<Pair> {
    let keyed: Vec<Pair> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty() && i.config.contains_key("details"))
        .map(|i| {
            let details = esc(i.config.get("details").map(String::as_str).unwrap_or(""));
            let details = match i.config.get("badge").filter(|b| !b.is_empty()) {
                Some(variant) => crate::cronus_ui_badge::badge_html(&details, variant),
                None if i.config.get("font").is_some_and(|f| f == "mono") => {
                    format!("<span class=\"font-mono\">{details}</span>")
                }
                None => details,
            };
            Pair {
                term: esc(&i.text),
                details,
            }
        })
        .collect();
    if !keyed.is_empty() {
        return keyed;
    }
    content(comp)
        .chunks(2)
        .map(|pair| Pair {
            term: pair[0].clone(),
            details: pair.get(1).cloned().unwrap_or_default(),
        })
        .collect()
}

fn content(comp: &ComponentNode) -> Vec<String> {
    let texts: Vec<String> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty() && !NAME_KINDS.contains(&i.item_type.as_str()))
        .map(|i| esc(&i.text))
        .collect();
    if !texts.is_empty() {
        return texts;
    }
    // Label-only list: the label is the single term.
    comp.items
        .iter()
        .find(|i| !i.text.is_empty())
        .map(|i| vec![esc(&i.text)])
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem";
    const INTERACT_ROW: &str =
        "display:flex;justify-content:space-between;gap:1rem;font-size:0.875rem";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn keyed(term: &str, pairs: &[(&str, &str)]) -> ComponentItemNode {
        let mut i = extra("item", term);
        for (k, v) in pairs {
            i.config.insert(k.to_string(), v.to_string());
        }
        i
    }

    fn list(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("description-list", items.first().copied().unwrap_or("Name"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_stub(html: &str) {
        assert!(!html.contains("<section"));
        assert!(!html.contains("<nav"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("v-show"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains(INTERACT_ROW));
        assert!(!html.contains("display("));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_dl_with_paired_items() {
        let html = render(&list(&["Order", "#10245", "Status", "Paid"]));
        reject_stub(&html);
        assert_eq!(
            html,
            "<dl data-slot=\"description-list\"><div data-slot=\"description-item\"><dt data-slot=\"description-term\">Order</dt><dd data-slot=\"description-details\">#10245</dd></div><div data-slot=\"description-item\"><dt data-slot=\"description-term\">Status</dt><dd data-slot=\"description-details\">Paid</dd></div></dl>"
        );
    }

    /// Emitted audit source: `label "Details"` + four texts + `aria-label:"Details"`.
    /// The label names the widget; it must not shift the term/details pairing.
    #[test]
    fn label_names_the_list_and_is_not_a_term() {
        let mut c = stub("description-list", "Details");
        for t in ["Name", "Ada", "Status", "Paid"] {
            c.items.push(extra("text", t));
        }
        c.items
            .last_mut()
            .unwrap()
            .config
            .insert("aria-label".into(), "Details".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<dl data-slot=\"description-list\" aria-label=\"Details\"><div data-slot=\"description-item\"><dt data-slot=\"description-term\">Name</dt><dd data-slot=\"description-details\">Ada</dd></div><div data-slot=\"description-item\"><dt data-slot=\"description-term\">Status</dt><dd data-slot=\"description-details\">Paid</dd></div></dl>"
        );
        reject_stub(&html);
    }

    #[test]
    fn odd_leftover_term_has_empty_details() {
        let html = render(&list(&["Name", "Ada", "Role"]));
        assert_eq!(html.matches("data-slot=\"description-item\"").count(), 2);
        assert!(html.contains("data-slot=\"description-term\">Role</dt>"));
        assert!(html.contains("data-slot=\"description-details\"></dd>"));
        reject_stub(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("description-list", "Order"));
        assert_eq!(html.matches("data-slot=\"description-item\"").count(), 1);
        assert!(html.contains("data-slot=\"description-term\">Order</dt>"));
        assert!(html.contains("data-slot=\"description-details\"></dd>"));
        reject_stub(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("description-list", "A <B> & \"C\""));
        assert!(
            html.contains("data-slot=\"description-term\">A &lt;B&gt; &amp; &quot;C&quot;</dt>")
        );
        reject_stub(&html);
    }

    /// Docs "Stacked": `raw:true` composes bare dt/dd pairs (no item groups)
    /// and `max-width:sm` is the example's `max-w-sm`.
    #[test]
    fn raw_pairs_without_item_groups() {
        let mut c = list(&["Full name", "Margot Foster", "Email", "margot@example.com"]);
        c.props.insert("raw".into(), "true".into());
        c.props.insert("max-width".into(), "sm".into());
        assert_eq!(
            render(&c),
            "<dl data-slot=\"description-list\" class=\"mw-sm\"><dt data-slot=\"description-term\">Full name</dt><dd data-slot=\"description-details\">Margot Foster</dd><dt data-slot=\"description-term\">Email</dt><dd data-slot=\"description-details\">margot@example.com</dd></dl>"
        );
    }

    /// Docs "Horizontal order summary" / "Striped rows" / "Grid cards": keyed
    /// items, variant classes on the `<dl>`, Badge and mono details.
    #[test]
    fn keyed_items_and_variant_classes() {
        let mut c = stub("description-list", "Order");
        c.style = Some("description-list+horizontal".into());
        c.props.insert("details-align".into(), "end".into());
        c.props.insert("bordered".into(), "true".into());
        c.props.insert("max-width".into(), "md".into());
        c.items = vec![
            keyed("Order", &[("details", "#10245")]),
            keyed("Status", &[("details", "Paid"), ("badge", "success")]),
            keyed("Total", &[("details", "$149.00"), ("font", "mono")]),
        ];
        let html = render(&c);
        assert_eq!(
            html,
            "<dl data-slot=\"description-list\" class=\"l-horizontal bordered align-end mw-md\"><div data-slot=\"description-item\"><dt data-slot=\"description-term\">Order</dt><dd data-slot=\"description-details\">#10245</dd></div><div data-slot=\"description-item\"><dt data-slot=\"description-term\">Status</dt><dd data-slot=\"description-details\"><span data-slot=\"badge\" data-variant=\"success\">Paid</span></dd></div><div data-slot=\"description-item\"><dt data-slot=\"description-term\">Total</dt><dd data-slot=\"description-details\"><span class=\"font-mono\">$149.00</span></dd></div></dl>"
        );
        let mut s = stub("description-list", "Env");
        s.style = Some("description-list+horizontal".into());
        s.props.insert("striped".into(), "true".into());
        s.items = vec![keyed(
            "Mode",
            &[("details", "Live"), ("badge", "secondary")],
        )];
        assert!(render(&s)
            .starts_with("<dl data-slot=\"description-list\" class=\"l-horizontal striped\">"));
        let mut g = stub("description-list", "Deploy");
        g.style = Some("description-list+grid+sm".into());
        g.items = vec![keyed("Region", &[("details", "São Paulo (GRU)")])];
        assert!(render(&g).starts_with("<dl data-slot=\"description-list\" class=\"l-grid s-sm\">"));
        reject_stub(&html);
        let css = include_str!("cronus_ui_css/description-list.css");
        assert!(css.contains("[data-slot=\"description-list\"].l-horizontal {\n  display: grid; grid-template-columns: fit-content(50%) minmax(0, 1fr);\n}"));
        assert!(css.contains("[data-slot=\"description-list\"].l-grid > div {\n  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border); background: var(--cronus-surface-raised); padding: 1rem;\n}"));
        assert!(css.contains("[data-slot=\"description-list\"].striped > div:nth-of-type(even) { background: var(--cronus-surface-inset); }"));
        assert!(css.contains("[data-slot=\"description-list\"].align-end [data-slot=\"description-details\"] { text-align: end; }"));
        assert!(css.contains("[data-slot=\"description-list\"].mw-md { max-width: 28rem; }"));
        assert!(css.contains("@media (min-width: 1024px) {\n  [data-slot=\"description-list\"].l-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); }\n}"));
    }

    #[test]
    fn skips_interact_styled_dl_and_display_surf() {
        let c = list(&["Order", "#10245"]);
        let html = render(&c);
        assert!(html.contains("data-slot=\"description-term\""));
        reject_stub(&html);
        assert_eq!(
            dedicated_fn_name("description-list"),
            Some("cronus_ui_description_list::render")
        );
        assert_eq!(
            renderer_kind("description-list"),
            RendererKind::Dedicated("cronus_ui_description_list::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&list(&["Order", "#10245"]));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"description-term\""));
        });
    }

    /// Geometry parity (Wave 1t): React `text-sm` pairs 14px with a 20px line
    /// box, groups stack with `mt-4`, details sit `mt-1` below the term, and the
    /// audit harness sizes the list `w-72` (18rem).
    #[test]
    fn chrome_matches_react_stacked_md_geometry() {
        let css = include_str!("cronus_ui_css/description-list.css");
        assert!(css.contains(
            "[data-slot=\"description-list\"] {\n  display: block; width: var(--cui-description-list-w, 100%); max-width: 100%; min-width: 0; margin: 0;\n  font-size: 0.875rem; line-height: 1.25rem;\n}"
        ));
        assert!(css.contains(
            "[data-slot=\"description-item\"] + [data-slot=\"description-item\"] { margin-top: 1rem; }"
        ));
        assert!(css.contains("[data-slot=\"description-term\"] { font-weight: 500; color: var(--cronus-fg-secondary); }"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(!css.contains("zinc-"));
    }
}
