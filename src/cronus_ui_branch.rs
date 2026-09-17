//! Dedicated Branch renderer (AI suite). DOM matches React `Branch`:
//! `<div data-slot="branch">` > `<div data-slot="branch-messages">` with one
//! slotless `<div>` per branch (a `<p>` per `text` line; only the current
//! branch is displayed) > `<div data-slot="branch-selector">` (hidden for a
//! single branch, like React) with `<button data-slot="branch-previous">`,
//! `<span data-slot="branch-page">1 of 2</span>` and
//! `<button data-slot="branch-next">` (ghost icon-sm Buttons, `data-variant`).
//!
//! Zero JS: the selector holds one visually hidden `<input type="radio">` per
//! branch (shared page-unique `name`, `value` = index, `defaultBranch:` picks
//! the checked one; Arrow keys move between them). Previous/next are
//! `<label for>` the neighbouring radio around React's decorative button, and
//! the page counter, previous and next each exist once per branch with
//! `data-branch="i"`; CSS shows the trio of the checked radio and the matching
//! message. At the ends React disables the button: those are plain `disabled`
//! buttons. `from:user` right-aligns the selector (React `from="user"`), the
//! default is `assistant`. The switching CSS covers the first [`MAX_BRANCHES`].
//! `labels`: `previous:"…"`, `next:"…"`, `page:"{current} of {total}"`.

use crate::cronus_ui_kit::{attr, attr_nonempty, attr_num, esc, instance_id, label_of};
use crate::parser::ComponentNode;

/// Branches beyond this index still render but have no switching rule.
pub const MAX_BRANCHES: usize = 12;

const CHEVRON_LEFT: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m15 18-6-6 6-6\"></path></svg>";
const CHEVRON_RIGHT: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m9 18 6-6-6-6\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let branches: Vec<String> = {
        let texts: Vec<String> = comp
            .items
            .iter()
            .filter(|i| i.item_type != "label" && !i.text.is_empty())
            .map(|i| esc(&i.text))
            .collect();
        if texts.is_empty() {
            vec![label_of(comp)]
        } else {
            texts
        }
    };
    let total = branches.len();
    let current = attr_num::<usize>(comp, "defaultBranch")
        .or_else(|| attr_num::<usize>(comp, "default-branch"))
        .unwrap_or(0)
        .min(total - 1);
    let messages: String = branches
        .iter()
        .map(|t| format!("<div><p>{t}</p></div>"))
        .collect();
    let from_class = if attr(comp, "from").map(str::trim) == Some("user") {
        " class=\"from-user\""
    } else {
        ""
    };
    let selector = if total > 1 {
        let name = instance_id(comp, "branch");
        let prev_label = attr_nonempty(comp, "previous")
            .map(esc)
            .unwrap_or_else(|| "Previous branch".into());
        let next_label = attr_nonempty(comp, "next")
            .map(esc)
            .unwrap_or_else(|| "Next branch".into());
        let page_label = attr_nonempty(comp, "page").unwrap_or("{current} of {total}");
        let radios: String = (0..total)
            .map(|i| {
                let checked = if i == current { " checked" } else { "" };
                format!(
                    "<input type=\"radio\" name=\"{name}\" id=\"{name}-{i}\" value=\"{i}\" aria-label=\"{}\"{checked}>",
                    esc(&page_text("Branch {current} of {total}", i + 1, total))
                )
            })
            .collect();
        let controls: String = (0..total)
            .map(|i| {
                let prev = if i == 0 {
                    format!(
                        "<button type=\"button\" data-slot=\"branch-previous\" data-variant=\"ghost\" data-branch=\"{i}\" aria-label=\"{prev_label}\" disabled>{CHEVRON_LEFT}</button>"
                    )
                } else {
                    format!(
                        "<label for=\"{name}-{}\" data-branch=\"{i}\"><button type=\"button\" data-slot=\"branch-previous\" data-variant=\"ghost\" aria-label=\"{prev_label}\" tabindex=\"-1\" aria-hidden=\"true\">{CHEVRON_LEFT}</button></label>",
                        i - 1
                    )
                };
                let page = format!(
                    "<span data-slot=\"branch-page\" data-branch=\"{i}\">{}</span>",
                    esc(&page_text(page_label, i + 1, total))
                );
                let next = if i + 1 == total {
                    format!(
                        "<button type=\"button\" data-slot=\"branch-next\" data-variant=\"ghost\" data-branch=\"{i}\" aria-label=\"{next_label}\" disabled>{CHEVRON_RIGHT}</button>"
                    )
                } else {
                    format!(
                        "<label for=\"{name}-{}\" data-branch=\"{i}\"><button type=\"button\" data-slot=\"branch-next\" data-variant=\"ghost\" aria-label=\"{next_label}\" tabindex=\"-1\" aria-hidden=\"true\">{CHEVRON_RIGHT}</button></label>",
                        i + 1
                    )
                };
                format!("{prev}{page}{next}")
            })
            .collect();
        format!("<div data-slot=\"branch-selector\" role=\"radiogroup\" aria-label=\"{}\">{radios}{controls}</div>", label_of(comp))
    } else {
        String::new()
    };
    format!(
        "<div data-slot=\"branch\"{from_class}><div data-slot=\"branch-messages\">{messages}</div>{selector}</div>"
    )
}

fn page_text(template: &str, current: usize, total: usize) -> String {
    template
        .replacen("{current}", &current.to_string(), 1)
        .replacen("{total}", &total.to_string(), 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
    use crate::parser::ComponentItemNode;

    fn text(t: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "text".into(),
            text: t.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn docs() -> ComponentNode {
        let mut c = stub("branch", "Alternative generations");
        c.items.push(text("First draft of the answer."));
        c.items.push(text("A tighter rewrite of the same answer."));
        c
    }

    #[test]
    fn docs_example_has_two_messages_radios_and_control_trios() {
        reset_instance_ids();
        let html = render(&docs());
        assert!(html.starts_with("<div data-slot=\"branch\"><div data-slot=\"branch-messages\"><div><p>First draft of the answer.</p></div><div><p>A tighter rewrite of the same answer.</p></div></div><div data-slot=\"branch-selector\" role=\"radiogroup\" aria-label=\"Alternative generations\">"));
        assert!(html.contains("<input type=\"radio\" name=\"cui-branch-branch\" id=\"cui-branch-branch-0\" value=\"0\" aria-label=\"Branch 1 of 2\" checked><input type=\"radio\" name=\"cui-branch-branch\" id=\"cui-branch-branch-1\" value=\"1\" aria-label=\"Branch 2 of 2\">"));
        assert!(html.contains("<button type=\"button\" data-slot=\"branch-previous\" data-variant=\"ghost\" data-branch=\"0\" aria-label=\"Previous branch\" disabled><svg"));
        assert!(html.contains("<span data-slot=\"branch-page\" data-branch=\"0\">1 of 2</span><label for=\"cui-branch-branch-1\" data-branch=\"0\"><button type=\"button\" data-slot=\"branch-next\" data-variant=\"ghost\" aria-label=\"Next branch\" tabindex=\"-1\" aria-hidden=\"true\"><svg"));
        assert!(html.contains("<label for=\"cui-branch-branch-0\" data-branch=\"1\"><button type=\"button\" data-slot=\"branch-previous\""));
        assert!(html.contains("<span data-slot=\"branch-page\" data-branch=\"1\">2 of 2</span><button type=\"button\" data-slot=\"branch-next\" data-variant=\"ghost\" data-branch=\"1\" aria-label=\"Next branch\" disabled>"));
        assert_eq!(html.matches("data-slot=\"branch-page\"").count(), 2);
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn default_branch_from_user_and_labels() {
        reset_instance_ids();
        let mut c = docs();
        c.props.insert("defaultBranch".into(), "1".into());
        c.props.insert("from".into(), "user".into());
        c.props.insert("page".into(), "{current}/{total}".into());
        c.props.insert("previous".into(), "Back".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"branch\" class=\"from-user\">"));
        assert!(html.contains("value=\"0\" aria-label=\"Branch 1 of 2\"><input"));
        assert!(html.contains("value=\"1\" aria-label=\"Branch 2 of 2\" checked>"));
        assert!(html.contains(">1/2</span>"));
        assert!(html.contains("aria-label=\"Back\""));
    }

    #[test]
    fn single_branch_has_no_selector_and_escapes() {
        let html = render(&stub("branch", "<b>\"x\"</b>"));
        assert_eq!(
            html,
            "<div data-slot=\"branch\"><div data-slot=\"branch-messages\"><div><p>&lt;b&gt;&quot;x&quot;&lt;/b&gt;</p></div></div></div>"
        );
    }

    #[test]
    fn two_branches_on_one_page_get_distinct_radio_names() {
        reset_instance_ids();
        let a = render(&docs());
        let b = render(&docs());
        assert!(a.contains("name=\"cui-branch-branch\""));
        assert!(b.contains("name=\"cui-branch-branch-2\""));
        assert!(!b.contains("name=\"cui-branch-branch\""));
    }

    #[test]
    fn chrome_switches_messages_and_controls_by_radio() {
        let css = include_str!("cronus_ui_css/branch.css");
        assert!(css.contains("[data-slot=\"branch\"]:has(> [data-slot=\"branch-selector\"] > input[value=\"1\"]:checked) > [data-slot=\"branch-messages\"] > div:nth-child(2)"));
        assert!(css.contains("[data-slot=\"branch-selector\"]:has(> input[value=\"11\"]:checked) > [data-branch=\"11\"]"));
        assert!(css.contains("padding: 0 2.5rem"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn registered_as_dedicated() {
        use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
        assert_eq!(
            dedicated_fn_name("branch"),
            Some("cronus_ui_branch::render")
        );
        assert_eq!(
            renderer_kind("branch"),
            RendererKind::Dedicated("cronus_ui_branch::render")
        );
        let c = stub("branch", "Only answer");
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), render(&c));
    }
}
