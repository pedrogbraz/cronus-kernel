//! Dedicated NavigationMenu renderer. DOM mirrors the React audit fixture:
//! `<nav data-slot="navigation-menu" aria-label="Main">` > Radix's slotless
//! `<div>` > `<ul data-slot="navigation-menu-list">` > one
//! `<li data-slot="navigation-menu-item">` per text with a closed
//! `<button data-slot="navigation-menu-trigger">` (label + ChevronDown 14px).
//! The fixture's triggers carry no `NavigationMenuContent`, so nothing opens:
//! those triggers are native buttons rendered `disabled` with React's idle
//! (closed, undimmed) look. No Radix viewport (empty, slotless).
//! A trigger item with `content:"…"` (item config) gets real content: its
//! enabled trigger's `popovertarget` opens a native `popover="auto"`
//! `navigation-menu-content` anchored under it (Esc / outside click dismiss).
//! Gaps: no hover-to-open or pointer-grace between triggers (JS), and
//! `aria-expanded` / `data-state` stay "closed".
//! Docs menu bar: `link "Analytics" -> "#" description:"…" menu:"Products"`
//! lines fill the `Products` trigger's panel with the docs' `<ul>` of
//! `navigation-menu-link`s (name + description spans); a `link` without
//! `menu:` is the flat `Docs` entry, a `navigation-menu-link` styled like a
//! trigger (`navigationMenuTriggerStyle`, a `trigger` class). Trigger panels
//! open by click or hover (`popovertarget` + `interestfor`).
//! Not interact `nav("navigation-menu")` (generic SURF `<nav>`).

use crate::cronus_ui_kit::{choice_texts, esc, safe_url, texts, widget_id};
use crate::parser::{ComponentItemNode, ComponentNode};

const CHEVRON: &str = concat!(
    "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" ",
    "stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">",
    "<path d=\"m6 9 6 6 6-6\" /></svg>",
);

pub fn render(comp: &ComponentNode) -> String {
    let links: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "link" && !i.text.is_empty())
        .collect();
    let mut items = Vec::new();
    for (i, t) in menu_triggers(comp).iter().enumerate() {
        let panel_links: Vec<&ComponentItemNode> = links
            .iter()
            .copied()
            .filter(|l| l.config.get("menu").map(|m| esc(m.trim())) == Some(t.clone()))
            .collect();
        let content = if panel_links.is_empty() {
            content_of(comp, t)
        } else {
            Some(format!(
                "<ul>{}</ul>",
                panel_links
                    .iter()
                    .map(|l| {
                        let description = l
                            .config
                            .get("description")
                            .filter(|d| !d.trim().is_empty())
                            .map(|d| format!("<span>{}</span>", esc(d)))
                            .unwrap_or_default();
                        format!(
                            "<li><a data-slot=\"navigation-menu-link\" href=\"{}\"><span>{}</span>{description}</a></li>",
                            safe_url(l.link.as_deref().unwrap_or("#")),
                            esc(&l.text)
                        )
                    })
                    .collect::<String>()
            ))
        };
        items.push(match content {
            Some(content) => {
                let trigger_id = widget_id(comp, &format!("trigger-{}", i + 1));
                let pop_id = widget_id(comp, &format!("content-{}", i + 1));
                format!(
                    "<li data-slot=\"navigation-menu-item\"><button type=\"button\" id=\"{trigger_id}\" data-slot=\"navigation-menu-trigger\" data-state=\"closed\" aria-expanded=\"false\" popovertarget=\"{pop_id}\" interestfor=\"{pop_id}\">{t}{CHEVRON}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"navigation-menu-content\" anchor=\"{trigger_id}\">{content}</div></li>"
                )
            }
            None => format!(
                "<li data-slot=\"navigation-menu-item\"><button type=\"button\" data-slot=\"navigation-menu-trigger\" data-state=\"closed\" aria-expanded=\"false\" disabled>{t}{CHEVRON}</button></li>"
            ),
        });
    }
    // Flat links (no `menu:`) are trigger-styled `navigation-menu-link`s.
    for l in links
        .iter()
        .filter(|l| l.config.get("menu").map_or(true, |m| m.trim().is_empty()))
    {
        items.push(format!(
            "<li data-slot=\"navigation-menu-item\"><a data-slot=\"navigation-menu-link\" class=\"trigger\" href=\"{}\">{}</a></li>",
            safe_url(l.link.as_deref().unwrap_or("#")),
            esc(&l.text)
        ));
    }
    format!(
        "<nav data-slot=\"navigation-menu\" aria-label=\"Main\"><div><ul data-slot=\"navigation-menu-list\">{}</ul></div></nav>",
        items.join("")
    )
}

/// `content:"…"` in the config of the item whose (escaped) text is `trigger`.
fn content_of(comp: &ComponentNode, trigger: &str) -> Option<String> {
    comp.items
        .iter()
        .find(|i| esc(&i.text) == trigger)
        .and_then(|i| i.config.get("content"))
        .filter(|c| !c.trim().is_empty())
        .map(|c| esc(c))
}

fn menu_triggers(comp: &ComponentNode) -> Vec<String> {
    let choices = choice_texts(comp);
    if !choices.is_empty() {
        return choices;
    }
    if comp.items.iter().any(|i| i.item_type == "link") {
        return Vec::new();
    }
    texts(comp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<details"));
        assert!(!html.contains("<summary"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("nav("));
    }

    #[test]
    fn label_and_texts_become_closed_triggers_without_content() {
        let mut c = stub("navigation-menu", "Products");
        c.items.push(extra("text", "Analytics"));
        c.items.push(extra("text", "Docs"));
        let html = render(&c);
        let li = |t: &str| {
            format!(
                "<li data-slot=\"navigation-menu-item\"><button type=\"button\" data-slot=\"navigation-menu-trigger\" data-state=\"closed\" aria-expanded=\"false\" disabled>{t}{CHEVRON}</button></li>"
            )
        };
        assert_eq!(
            html,
            format!(
                "<nav data-slot=\"navigation-menu\" aria-label=\"Main\"><div><ul data-slot=\"navigation-menu-list\">{}{}{}</ul></div></nav>",
                li("Products"),
                li("Analytics"),
                li("Docs")
            )
        );
        assert!(!html.contains("navigation-menu-content"));
        assert!(!html.contains("data-state=\"open\""));
        reject_interact(&html);
    }

    #[test]
    fn item_content_opens_a_native_popover_panel() {
        let mut c = stub("navigation-menu", "Menus");
        let mut products = extra("item", "Products");
        products
            .config
            .insert("content".into(), "Analytics & dashboards".into());
        c.items.push(products);
        c.items.push(extra("item", "Docs"));
        let html = render(&c);
        let tid = crate::cronus_ui_kit::widget_id(&c, "trigger-1");
        let pid = crate::cronus_ui_kit::widget_id(&c, "content-1");
        assert!(html.contains(&format!(
            "<li data-slot=\"navigation-menu-item\"><button type=\"button\" id=\"{tid}\" data-slot=\"navigation-menu-trigger\" data-state=\"closed\" aria-expanded=\"false\" popovertarget=\"{pid}\" interestfor=\"{pid}\">Products{CHEVRON}</button><div id=\"{pid}\" popover=\"auto\" data-slot=\"navigation-menu-content\" anchor=\"{tid}\">Analytics &amp; dashboards</div></li>"
        )));
        // No content: nothing to open, the JS-only trigger stays inert.
        assert!(html.contains("aria-expanded=\"false\" disabled>Docs<svg"));
        reject_interact(&html);
    }

    #[test]
    fn field_label_is_not_a_trigger_when_items_exist() {
        let mut c = stub("navigation-menu", "Menus");
        c.items.push(extra("item", "Products"));
        c.items.push(extra("item", "Docs"));
        let html = render(&c);
        assert!(html.contains("aria-expanded=\"false\" disabled>Products<svg"));
        assert!(html.contains("aria-expanded=\"false\" disabled>Docs<svg"));
        assert!(!html.contains(">Menus<svg"));
        assert_eq!(
            html.matches("data-slot=\"navigation-menu-trigger\"")
                .count(),
            2
        );
        reject_interact(&html);
    }

    #[test]
    fn label_only_is_one_trigger() {
        let html = render(&stub("navigation-menu", "Products"));
        assert_eq!(
            html.matches("data-slot=\"navigation-menu-trigger\"")
                .count(),
            1
        );
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf() {
        let mut c = stub("navigation-menu", "Menus");
        c.items.push(extra("item", "Products"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"navigation-menu-list\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("navigation-menu", "Products"));
            reject_interact(&html);
        });
    }

    #[test]
    fn docs_menu_bar_links_fill_the_panel_and_flat_link_is_trigger_styled() {
        let mut c = stub("navigation-menu", "Main");
        c.items.push(extra("item", "Products"));
        let mut analytics = extra("link", "Analytics");
        analytics.link = Some("#".into());
        analytics.config.insert("menu".into(), "Products".into());
        analytics.config.insert(
            "description".into(),
            "Real-time dashboards and reports.".into(),
        );
        c.items.push(analytics);
        let mut docs = extra("link", "Docs");
        docs.link = Some("#".into());
        c.items.push(docs);
        let html = render(&c);
        let tid = widget_id(&c, "trigger-1");
        let pid = widget_id(&c, "content-1");
        assert!(html.contains(&format!(
            "<li data-slot=\"navigation-menu-item\"><button type=\"button\" id=\"{tid}\" data-slot=\"navigation-menu-trigger\" data-state=\"closed\" aria-expanded=\"false\" popovertarget=\"{pid}\" interestfor=\"{pid}\">Products{CHEVRON}</button><div id=\"{pid}\" popover=\"auto\" data-slot=\"navigation-menu-content\" anchor=\"{tid}\"><ul><li><a data-slot=\"navigation-menu-link\" href=\"#\"><span>Analytics</span><span>Real-time dashboards and reports.</span></a></li></ul></div></li><li data-slot=\"navigation-menu-item\"><a data-slot=\"navigation-menu-link\" class=\"trigger\" href=\"#\">Docs</a></li></ul></div></nav>"
        )), "{html}");
        assert!(!html.contains(">Main<"));
        reject_interact(&html);
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"navigation-menu-content\"] > ul {\n  display: grid; gap: 0.25rem; margin: 0; padding: 0; list-style: none; min-width: 22rem;"));
        assert!(css.contains("[data-slot=\"navigation-menu-link\"] {\n  display: flex; flex-direction: column; gap: 0.125rem;"));
        assert!(css.contains("[data-slot=\"navigation-menu-link\"].trigger {\n  display: inline-flex; flex-direction: row;"));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"navigation-menu\"]"));
        assert!(css.contains(
            "[data-slot=\"navigation-menu-list\"] {\n  display: flex; flex: 1; list-style: none;"
        ));
        assert!(css.contains("[data-slot=\"navigation-menu-item\"]"));
        assert!(css.contains("[data-slot=\"navigation-menu-trigger\"] > svg {"));
        assert!(css.contains("font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(css.contains("[data-slot=\"navigation-menu-content\"]:popover-open {"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
