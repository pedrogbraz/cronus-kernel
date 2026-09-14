//! Dedicated AvatarGroup renderer. DOM matches React:
//! `<div data-slot="avatar-group">` wrapping 2–3
//! `<span data-slot="avatar"><span data-slot="avatar-fallback">XX</span></span>`
//! from text items (initials). Optional overflow `+N` span
//! `data-slot="avatar-group-overflow"`.
//! Not interact `avatar("avatar-group")` (single `<div style=circle>` one letter).

use crate::cronus_ui_kit::esc;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let names = names(comp);
    let total = names.len();
    let limit = match max_of(comp) {
        Some(m) if m < total => m,
        _ => total,
    };
    let mut inner = String::new();
    for name in names.iter().take(limit) {
        let fb = esc(&initials(name));
        inner.push_str(&format!(
            "<span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">{fb}</span></span>"
        ));
    }
    let overflow = total.saturating_sub(limit);
    if overflow > 0 {
        inner.push_str(&format!(
            "<span data-slot=\"avatar-group-overflow\" role=\"img\" aria-label=\"{overflow} more\">+{overflow}</span>"
        ));
    }
    let aria = esc(aria_label(comp).unwrap_or("Avatar group"));
    format!("<div data-slot=\"avatar-group\" role=\"group\" aria-label=\"{aria}\">{inner}</div>")
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    comp.props
        .get("aria-label")
        .map(String::as_str)
        .or_else(|| {
            comp.items
                .iter()
                .find_map(|i| i.config.get("aria-label").map(String::as_str))
        })
        .filter(|s| !s.is_empty())
}

/// Members come from the non-label items (React: `items`/`options`). The
/// emitter's `label` line is the group's aria-label/title, never a member;
/// it only becomes the single member when no other item exists.
fn names(comp: &ComponentNode) -> Vec<String> {
    let members: Vec<String> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty() && !matches!(i.item_type.as_str(), "label" | "title"))
        .map(|i| i.text.clone())
        .collect();
    let out: Vec<String> = if members.is_empty() {
        comp.items
            .iter()
            .filter(|i| !i.text.is_empty())
            .map(|i| i.text.clone())
            .collect()
    } else {
        members
    };
    if out.is_empty() {
        if !comp.name.is_empty() {
            vec![comp.name.clone()]
        } else {
            vec!["A".into()]
        }
    } else {
        out
    }
}

fn max_of(comp: &ComponentNode) -> Option<usize> {
    comp.props.get("max").and_then(|v| v.parse().ok())
}

fn initials(label: &str) -> String {
    let words: Vec<&str> = label.split_whitespace().filter(|w| !w.is_empty()).collect();
    let mut out = String::new();
    if words.len() >= 2 {
        for w in words.iter().take(2) {
            if let Some(ch) = w.chars().next() {
                out.extend(ch.to_uppercase().take(1));
            }
        }
    } else if let Some(w) = words.first() {
        for ch in w.chars().take(2) {
            out.extend(ch.to_uppercase());
        }
        if out.chars().count() > 2 {
            out = out.chars().take(2).collect();
        }
    }
    if out.is_empty() {
        "A".into()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "item".into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn group(names: &[&str]) -> crate::parser::ComponentNode {
        // Members are `text` items, as the audit emitter writes them.
        let mut c = stub("avatar-group", names[0]);
        c.items[0].item_type = "text".into();
        for n in names.iter().skip(1) {
            c.items.push(extra(n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("width:2.25rem"));
        assert!(!html.contains("border-radius:999px"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("{ value }"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<div data-slot=\"avatar\""));
    }

    #[test]
    fn root_wraps_avatars_not_single_circle() {
        let mut c = group(&["Jane Doe", "Ada Lovelace"]);
        c.items[0].item_type = "text".into();
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"avatar-group\""));
        assert!(html.contains("role=\"group\""));
        assert!(html.contains("aria-label=\"Avatar group\""));
        assert_eq!(html.matches("data-slot=\"avatar\"").count(), 2);
        assert_eq!(html.matches("data-slot=\"avatar-fallback\"").count(), 2);
        assert!(html.contains(">JD</span>"));
        assert!(html.contains(">AL</span>"));
        assert!(!html.contains("data-slot=\"avatar-group-overflow\""));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"avatar-group\" role=\"group\" aria-label=\"Avatar group\"><span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">JD</span></span><span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">AL</span></span></div>"
        );
    }

    #[test]
    fn label_is_aria_label_not_a_member() {
        // Wave 1t: emitter writes `label "Team"` + `text "AL"` + `text "JB"` +
        // `aria-label:"Team"` (config of the last item). React: 2 avatars.
        let mut c = stub("avatar-group", "Team");
        c.items.push(extra("AL"));
        let mut jb = extra("JB");
        jb.config.insert("aria-label".into(), "Team".into());
        c.items.push(jb);
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"avatar-group\" role=\"group\" aria-label=\"Team\"><span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">AL</span></span><span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">JB</span></span></div>"
        );
        assert!(!html.contains(">TE<"));
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("font-weight: 500; font-size: 0.875rem; line-height: 1.25rem;\n  box-shadow: 0 0 0 2px var(--cronus-surface-base);"));
    }

    #[test]
    fn three_text_items_are_three_avatars() {
        let html = render(&group(&["Jane Doe", "Ada Lovelace", "Alan Turing"]));
        assert_eq!(html.matches("data-slot=\"avatar-fallback\"").count(), 3);
        assert!(html.contains(">AT</span>"));
        assert!(!html.contains("data-slot=\"avatar-group-overflow\""));
        reject_interact(&html);
    }

    #[test]
    fn overflow_when_max_below_total() {
        let mut c = group(&["Jane Doe", "Ada Lovelace", "Alan Turing", "Grace Hopper"]);
        c.props.insert("max".into(), "3".into());
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"avatar\"").count(), 3);
        assert!(html.contains("data-slot=\"avatar-group-overflow\""));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("aria-label=\"1 more\""));
        assert!(html.contains(">+1</span>"));
        assert!(!html.contains(">GH</span>"));
        reject_interact(&html);
    }

    #[test]
    fn initials_from_single_word() {
        let html = render(&stub("avatar-group", "Demo"));
        assert!(html.contains(">DE</span>"));
        assert_eq!(html.matches("data-slot=\"avatar\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_circle() {
        let html = render(&group(&["Jane Doe", "Ada Lovelace"]));
        let interact = crate::cronus_ui_interact::render(
            "avatar-group",
            &stub("avatar-group", "Jane Doe"),
        )
        .unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<div data-slot=\"avatar-group\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("width:2.25rem"));
        assert!(interact.contains("border-radius:999px"));
        assert!(!interact.contains("data-slot=\"avatar-fallback\""));
        assert!(!interact.contains("data-slot=\"avatar\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&group(&["Jane Doe", "Ada Lovelace"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"avatar-group\"]"));
        assert!(css.contains("[data-slot=\"avatar-group-overflow\"]"));
        assert!(css.contains("display: flex"));
        assert!(css.contains("align-items: center"));
        assert!(css.contains("margin-left: -0.5rem"));
        assert!(css.contains("width: 2.25rem"));
        assert!(css.contains("height: 2.25rem"));
        assert!(css.contains("box-shadow: 0 0 0 2px var(--cronus-surface-base)"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("border-radius: 9999px"));
        assert!(!css.contains("zinc-"));
    }
}
