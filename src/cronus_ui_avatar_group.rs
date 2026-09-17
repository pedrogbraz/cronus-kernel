//! Dedicated AvatarGroup renderer. DOM matches React:
//! `<div data-slot="avatar-group" role="group" aria-label>` wrapping one
//! `<span data-slot="avatar">` per member (`item` lines: the text is the
//! fallback, `src:` / `alt:` add the `avatar-image`), then the `+N`
//! `data-slot="avatar-group-overflow"` chip once `max:` is below the count.
//! React's `size` (`sm` | `md` | `lg`, style segment or `size:` prop) sizes
//! the avatars and the chip through a class on the root (`s-sm`, `s-lg`; React
//! has no data attribute for it). The `label` names the group (never a member).
//! Not interact `avatar("avatar-group")` (single `<div style=circle>` one letter).

use crate::cronus_ui_avatar::{avatar_html, initials};
use crate::cronus_ui_kit::{attr_nonempty, choice, esc};
use crate::parser::ComponentNode;

struct Member {
    fallback: String,
    src: Option<String>,
    alt: String,
}

pub fn render(comp: &ComponentNode) -> String {
    let members = members(comp);
    let total = members.len();
    let limit = match max_of(comp) {
        Some(m) if m < total => m,
        _ => total,
    };
    let mut inner = String::new();
    for m in members.iter().take(limit) {
        inner.push_str(&avatar_html(&m.fallback, m.src.as_deref(), &m.alt, "", ""));
    }
    let overflow = total.saturating_sub(limit);
    if overflow > 0 {
        inner.push_str(&format!(
            "<span data-slot=\"avatar-group-overflow\" role=\"img\" aria-label=\"{overflow} more\">+{overflow}</span>"
        ));
    }
    let aria = esc(aria_label(comp).unwrap_or("Avatar group"));
    let class = match choice(comp, "size", &["sm", "lg"]) {
        Some(s) => format!(" class=\"s-{s}\""),
        None => String::new(),
    };
    format!(
        "<div data-slot=\"avatar-group\"{class} role=\"group\" aria-label=\"{aria}\">{inner}</div>"
    )
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    attr_nonempty(comp, "aria-label")
}

/// Members come from the non-label items (React: `avatars`). The emitter's
/// `label` line is the group's aria-label/title, never a member; it only
/// becomes the single member when no other item exists.
fn members(comp: &ComponentNode) -> Vec<Member> {
    let build = |i: &crate::parser::ComponentItemNode| Member {
        fallback: esc(&initials(&i.text)),
        src: i.config.get("src").filter(|s| !s.is_empty()).cloned(),
        alt: esc(i.config.get("alt").map(String::as_str).unwrap_or("")),
    };
    let members: Vec<Member> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty() && !matches!(i.item_type.as_str(), "label" | "title"))
        .map(build)
        .collect();
    let out: Vec<Member> = if members.is_empty() {
        comp.items
            .iter()
            .filter(|i| !i.text.is_empty())
            .map(build)
            .collect()
    } else {
        members
    };
    if out.is_empty() {
        let name = if comp.name.is_empty() {
            "A"
        } else {
            comp.name.as_str()
        };
        vec![Member {
            fallback: esc(&initials(name)),
            src: None,
            alt: String::new(),
        }]
    } else {
        out
    }
}

fn max_of(comp: &ComponentNode) -> Option<usize> {
    comp.props.get("max").and_then(|v| v.parse().ok())
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
        let css = include_str!("cronus_ui_css/avatar-group.css");
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

    /// Docs "With overflow": the first member carries `src:` / `alt:` and
    /// renders React's `AvatarImage` before its fallback.
    #[test]
    fn member_src_and_alt_render_avatar_image() {
        let mut c = stub("avatar-group", "Project collaborators");
        let mut cn = extra("CN");
        cn.config
            .insert("src".into(), "https://github.com/shadcn.png".into());
        cn.config.insert("alt".into(), "@shadcn".into());
        c.items.push(cn);
        c.items.push(extra("AL"));
        let html = render(&c);
        assert!(html.contains("<span data-slot=\"avatar\"><img data-slot=\"avatar-image\" src=\"https://github.com/shadcn.png\" alt=\"@shadcn\"><span data-slot=\"avatar-fallback\">CN</span></span>"));
        assert_eq!(html.matches("avatar-image").count(), 1);
        reject_interact(&html);
    }

    /// Docs "Sizes": `size` from the style segment (or `size:` prop) is a root
    /// class that sizes avatars and the chip (`size-7` / `size-11`, chip text).
    #[test]
    fn size_segment_is_root_class_sizing_avatars_and_chip() {
        let mut c = group(&["Jane Doe", "Ada Lovelace", "Alan Turing", "Grace Hopper"]);
        c.props.insert("max".into(), "3".into());
        c.style = Some("avatar-group+sm".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"avatar-group\" class=\"s-sm\" role=\"group\""));
        c.style = Some("avatar-group".into());
        c.props.insert("size".into(), "lg".into());
        assert!(render(&c).contains("class=\"s-lg\""));
        c.props.insert("size".into(), "md".into());
        assert!(!render(&c).contains("class=\"s-"));
        let css = include_str!("cronus_ui_css/avatar-group.css");
        assert!(css.contains("[data-slot=\"avatar-group\"].s-sm [data-slot=\"avatar\"],\n[data-slot=\"avatar-group\"].s-sm [data-slot=\"avatar-group-overflow\"] {\n  width: 1.75rem; height: 1.75rem;\n}"));
        assert!(css.contains("[data-slot=\"avatar-group\"].s-lg [data-slot=\"avatar-group-overflow\"] { font-size: 1rem; line-height: 1.5rem; }"));
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
        let css = include_str!("cronus_ui_css/avatar-group.css");
        assert!(css.contains("[data-slot=\"avatar-group\"]"));
        assert!(css.contains("[data-slot=\"avatar-group-overflow\"]"));
        assert!(css.contains("display: flex"));
        assert!(css.contains("align-items: center"));
        assert!(css.contains("margin-inline-start: -0.5rem"));
        assert!(css.contains("width: 2.25rem"));
        assert!(css.contains("height: 2.25rem"));
        assert!(css.contains("box-shadow: 0 0 0 2px var(--cronus-surface-base)"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("border-radius: 9999px"));
        assert!(!css.contains("zinc-"));
    }
}
