//! Dedicated Tabs renderer. Native tablist + hidden panels + onclick.
//! Never mix `v-show` with `hidden`.

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let items = texts(comp);
    let buttons = items
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let selected = if i == 0 { "true" } else { "false" };
            let click = format!(
                " onclick=\"var r=this.closest('[data-slot=tabs]');r.querySelectorAll('[role=tab]').forEach(function(b,j){{b.setAttribute('aria-selected', String(j==={i}));}});r.querySelectorAll('[role=tabpanel]').forEach(function(p,j){{p.hidden=j!=={i};}});\""
            );
            format!(
                "<button type=\"button\" role=\"tab\" aria-selected=\"{selected}\"{click}>{t}</button>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let panels = items
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let hidden = if i == 0 { "" } else { " hidden" };
            format!("<div role=\"tabpanel\"{hidden}>{t}</div>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"tabs\" data-animate><div role=\"tablist\">{buttons}</div>{panels}</div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    #[test]
    fn tablist_without_vshow() {
        let mut c = stub("tabs", "Overview");
        c.items.push(crate::parser::ComponentItemNode {
            item_type: "tab".into(),
            text: "Usage".into(),
            link: None,
            tone: None,
            config: Default::default(),
        });
        let html = render(&c);
        assert!(html.contains("role=\"tablist\""));
        assert!(html.contains("onclick="));
        assert!(!html.contains("v-show"));
        assert!(!html.contains("zinc-"));
        let interact = crate::cronus_ui_interact::render("tabs", &c).unwrap();
        assert_ne!(html, interact);
    }
}
