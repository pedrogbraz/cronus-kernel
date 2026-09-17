//! Dedicated AnimatedList renderer. DOM mirrors React: `<ul
//! data-slot="animated-list">` with one `<li data-slot="animated-list-item">`
//! per row (`item` / `text` lines; the field label is not a row). React
//! staggers the rows in with `motion.li` (`opacity 0 → 1`, `y 8px → 0`,
//! 350ms `[0.22, 1, 0.36, 1]`, `index × 0.08s` delay); COMPONENT_CHROME
//! runs the same entrance with a keyframe whose delay comes from
//! `nth-child`. Zero JS, no inline style.
//!
//! `card:true` is the docs example: `w-full max-w-sm` on the list (class
//! `card`) and each row body a `rounded-xl border bg-surface-raised px-4
//! py-3 text-sm` card (`<li><div>…</div></li>`).

use crate::cronus_ui_kit::{esc, flag, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let card = flag(comp, "card");
    let items = rows(comp)
        .into_iter()
        .map(|t| {
            if card {
                format!("<li data-slot=\"animated-list-item\"><div>{t}</div></li>")
            } else {
                format!("<li data-slot=\"animated-list-item\">{t}</li>")
            }
        })
        .collect::<Vec<_>>()
        .join("");
    let class = if card { " class=\"card\"" } else { "" };
    format!("<ul data-slot=\"animated-list\"{class}>{items}</ul>")
}

fn rows(comp: &ComponentNode) -> Vec<String> {
    let out: Vec<String> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty() && !matches!(i.item_type.as_str(), "label" | "title"))
        .map(|i| esc(&i.text))
        .collect();
    if out.is_empty() {
        vec![label_of(comp)]
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn list(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("animated-list", items.first().copied().unwrap_or("Alpha"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("<div"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_is_ul_of_items_not_fx_title_box() {
        let html = render(&list(&["Alpha", "Beta"]));
        assert_eq!(
            html,
            "<ul data-slot=\"animated-list\"><li data-slot=\"animated-list-item\">Alpha</li><li data-slot=\"animated-list-item\">Beta</li></ul>"
        );
        reject_fx(&html);
    }

    /// Audit fixture shape: `label "default"` + `text "Alpha"` + `text "Beta"`.
    /// React renders exactly the two items; the label must not leak as a row.
    #[test]
    fn label_is_not_a_row_when_texts_exist() {
        let mut c = stub("animated-list", "default");
        c.items.push(extra("text", "Alpha"));
        c.items.push(extra("text", "Beta"));
        let html = render(&c);
        assert_eq!(
            html,
            "<ul data-slot=\"animated-list\"><li data-slot=\"animated-list-item\">Alpha</li><li data-slot=\"animated-list-item\">Beta</li></ul>"
        );
        assert!(!html.contains("default"));
        reject_fx(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("animated-list", "Alpha"));
        assert_eq!(
            html,
            "<ul data-slot=\"animated-list\"><li data-slot=\"animated-list-item\">Alpha</li></ul>"
        );
        reject_fx(&html);
    }

    /// Docs example: `<AnimatedList className="w-full max-w-sm">` of
    /// `<div className="rounded-xl border … px-4 py-3 text-sm text-fg">` bodies.
    #[test]
    fn card_wraps_each_row_body_like_the_docs() {
        let mut c = list(&["Deploy finished", "Invite accepted", "Invoice paid"]);
        c.props.insert("card".into(), "true".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<ul data-slot=\"animated-list\" class=\"card\"><li data-slot=\"animated-list-item\"><div>Deploy finished</div></li><li data-slot=\"animated-list-item\"><div>Invite accepted</div></li><li data-slot=\"animated-list-item\"><div>Invoice paid</div></li></ul>"
        );
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("animated-list", "A <B> & \"C\""));
        assert!(html
            .contains("<li data-slot=\"animated-list-item\">A &lt;B&gt; &amp; &quot;C&quot;</li>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("animated-list", "Alpha"));
        reject_fx(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&list(&["Alpha"]));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"animated-list-item\""));
        });
    }

    #[test]
    fn chrome_staggers_rows_like_react() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"animated-list\"] {\n  display: flex; flex-direction: column; gap: 0.5rem;\n  list-style: none; margin: 0; padding: 0;\n}"));
        // React: duration 0.35s, ease [0.22, 1, 0.36, 1], delay index × 0.08s, y 8px → 0.
        assert!(css.contains("[data-slot=\"animated-list-item\"] {\n  --cui-i: 0;\n  animation: cui-animated-list 0.35s var(--cronus-ease) both;\n  animation-delay: calc(var(--cui-i) * 0.08s);\n}"));
        for k in 2..=12 {
            assert!(
                css.contains(&format!(
                    "[data-slot=\"animated-list-item\"]:nth-child(n+{k}) {{ --cui-i: {}; }}",
                    k - 1
                )),
                "{k}"
            );
        }
        assert!(css.contains("@keyframes cui-animated-list {\n  from { opacity: 0; transform: translateY(8px); }\n  to { opacity: 1; transform: translateY(0); }\n}"));
        assert!(css
            .contains("[data-slot=\"animated-list\"].card {\n  width: 100%; max-width: 24rem;\n}"));
        assert!(css.contains("[data-slot=\"animated-list\"].card > [data-slot=\"animated-list-item\"] > div {\n  padding: 0.75rem 1rem;\n  border: 1px solid var(--cronus-border);\n  border-radius: var(--cronus-radius-xl);\n  background: var(--cronus-surface-raised);\n  font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg);\n}"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
