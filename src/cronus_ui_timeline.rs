//! Dedicated Timeline renderer. DOM matches React slots:
//! `<div data-slot="timeline">` plus each text as
//! `<div data-slot="timeline-item"><div data-slot="timeline-content">`.
//! Not interact `timeline()` (`<ol style=BASE>` bordered list) or catalog
//! `display()` SURF `<section>`.

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let items = texts(comp)
        .into_iter()
        .map(|t| {
            format!(
                "<div data-slot=\"timeline-item\"><div data-slot=\"timeline-content\">{t}</div></div>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"timeline\">{items}</div>")
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

    fn feed(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("timeline", items.first().copied().unwrap_or("Shipped"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<ol"));
        assert!(!html.contains("<li"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("<nav"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("border-left:2px solid"));
        assert!(!html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem"));
        assert!(!html.contains("display("));
    }

    #[test]
    fn root_is_div_of_item_content_not_ol_or_section() {
        let html = render(&feed(&["Shipped", "Delivered"]));
        assert!(html.starts_with("<div data-slot=\"timeline\">"));
        assert_eq!(html.matches("data-slot=\"timeline-item\"").count(), 2);
        assert_eq!(html.matches("data-slot=\"timeline-content\"").count(), 2);
        assert!(html.contains(
            "<div data-slot=\"timeline-item\"><div data-slot=\"timeline-content\">Shipped</div></div>"
        ));
        assert!(html.contains(">Delivered</div>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"timeline\"><div data-slot=\"timeline-item\"><div data-slot=\"timeline-content\">Shipped</div></div><div data-slot=\"timeline-item\"><div data-slot=\"timeline-content\">Delivered</div></div></div>"
        );
    }

    #[test]
    fn extra_text_items_become_events() {
        let mut c = stub("timeline", "Shipped");
        c.items.push(extra("text", "Delivered"));
        c.items.push(extra("text", "Returned"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"timeline-item\"").count(), 3);
        assert!(html.contains(">Shipped</div>"));
        assert!(html.contains(">Delivered</div>"));
        assert!(html.contains(">Returned</div>"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("timeline", "Shipped"));
        assert!(html.starts_with("<div data-slot=\"timeline\">"));
        assert_eq!(html.matches("data-slot=\"timeline-item\"").count(), 1);
        assert!(html.contains(
            "<div data-slot=\"timeline-item\"><div data-slot=\"timeline-content\">Shipped</div></div>"
        ));
        reject_interact(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("timeline", "A <B> & \"C\""));
        assert!(html.contains(
            "<div data-slot=\"timeline-content\">A &lt;B&gt; &amp; &quot;C&quot;</div>"
        ));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_ol_and_display_surf() {
        let c = feed(&["Shipped", "Delivered"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("timeline", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<ol data-slot=\"timeline\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("<li"));
        assert!(!interact.contains("data-slot=\"timeline-item\""));
        assert!(!interact.contains("data-slot=\"timeline-content\""));
        assert!(html.contains("data-slot=\"timeline-item\""));
        assert!(!html.contains("<ol"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&feed(&["Shipped"]));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"timeline-item\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"timeline\"]"));
        assert!(css.contains("[data-slot=\"timeline-item\"]"));
        assert!(css.contains("[data-slot=\"timeline-content\"]"));
        assert!(css.contains("flex-direction: column"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
