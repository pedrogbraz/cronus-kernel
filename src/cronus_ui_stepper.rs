//! Dedicated Stepper renderer. DOM matches React:
//! `<div data-slot="stepper" data-orientation>` plus each text item
//! `<div data-slot="stepper-item"><button data-slot="stepper-trigger"><span data-slot="stepper-title">`.
//! Not interact `stepper()` (`<ol style=BASE>` numbered pills) or stub `<nav>`.

use crate::cronus_ui_kit::{choice_texts, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let orientation = orientation_of(comp);
    let steps = steps_of(comp);
    let current = current_of(comp, steps.len());
    let items = steps
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let state = if i < current {
                "completed"
            } else if i == current {
                "current"
            } else {
                "upcoming"
            };
            format!(
                "<div data-slot=\"stepper-item\" data-state=\"{state}\"><button type=\"button\" data-slot=\"stepper-trigger\"><span data-slot=\"stepper-title\">{t}</span></button></div>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"stepper\" data-orientation=\"{orientation}\">{items}</div>")
}

fn steps_of(comp: &ComponentNode) -> Vec<String> {
    let choices = choice_texts(comp);
    if !choices.is_empty() {
        return choices;
    }
    texts(comp)
}

fn current_of(comp: &ComponentNode, n: usize) -> usize {
    let raw = comp
        .props
        .get("value")
        .or_else(|| comp.props.get("current"))
        .map(String::as_str);
    let parsed = raw.and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
    if n == 0 {
        0
    } else {
        parsed.min(n - 1)
    }
}

fn orientation_of(comp: &ComponentNode) -> &'static str {
    if let Some(v) = comp.props.get("orientation") {
        if v == "vertical" {
            return "vertical";
        }
        if v == "horizontal" {
            return "horizontal";
        }
    }
    if comp
        .style
        .as_deref()
        .unwrap_or("")
        .split('+')
        .any(|part| part.trim() == "vertical")
    {
        return "vertical";
    }
    "horizontal"
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

    fn steps(titles: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("stepper", titles[0]);
        c.items[0].item_type = "item".into();
        for t in titles.iter().skip(1) {
            c.items.push(extra(t));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<ol"));
        assert!(!html.contains("<nav"));
        assert!(!html.contains("<li"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("-control"));
    }

    #[test]
    fn root_is_div_of_item_triggers_not_ol_or_nav() {
        let html = render(&steps(&["Account", "Plan", "Confirm"]));
        assert!(html.starts_with(
            "<div data-slot=\"stepper\" data-orientation=\"horizontal\">"
        ));
        assert_eq!(html.matches("data-slot=\"stepper-item\"").count(), 3);
        assert_eq!(html.matches("data-slot=\"stepper-trigger\"").count(), 3);
        assert!(html.contains(
            "<div data-slot=\"stepper-item\" data-state=\"current\"><button type=\"button\" data-slot=\"stepper-trigger\"><span data-slot=\"stepper-title\">Account</span></button></div>"
        ));
        assert!(html.contains("data-state=\"upcoming\""));
        assert!(html.contains(">Plan</span>"));
        assert!(html.contains(">Confirm</span>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"stepper\" data-orientation=\"horizontal\"><div data-slot=\"stepper-item\" data-state=\"current\"><button type=\"button\" data-slot=\"stepper-trigger\"><span data-slot=\"stepper-title\">Account</span></button></div><div data-slot=\"stepper-item\" data-state=\"upcoming\"><button type=\"button\" data-slot=\"stepper-trigger\"><span data-slot=\"stepper-title\">Plan</span></button></div><div data-slot=\"stepper-item\" data-state=\"upcoming\"><button type=\"button\" data-slot=\"stepper-trigger\"><span data-slot=\"stepper-title\">Confirm</span></button></div></div>"
        );
    }

    #[test]
    fn first_item_is_current_by_default() {
        let html = render(&steps(&["One", "Two"]));
        assert!(html.contains("data-state=\"current\""));
        assert!(html.contains(">One</span>"));
        assert!(html.contains("data-state=\"upcoming\""));
        assert!(!html.contains("data-state=\"completed\""));
        reject_interact(&html);
    }

    #[test]
    fn value_marks_current_and_completed() {
        let mut c = steps(&["Account", "Plan", "Confirm"]);
        c.props.insert("value".into(), "1".into());
        let html = render(&c);
        assert!(html.contains(
            "data-state=\"completed\"><button type=\"button\" data-slot=\"stepper-trigger\"><span data-slot=\"stepper-title\">Account</span>"
        ));
        assert!(html.contains(
            "data-state=\"current\"><button type=\"button\" data-slot=\"stepper-trigger\"><span data-slot=\"stepper-title\">Plan</span>"
        ));
        assert!(html.contains(
            "data-state=\"upcoming\"><button type=\"button\" data-slot=\"stepper-trigger\"><span data-slot=\"stepper-title\">Confirm</span>"
        ));
        reject_interact(&html);
    }

    #[test]
    fn vertical_from_props() {
        let mut c = steps(&["Top", "Bottom"]);
        c.props.insert("orientation".into(), "vertical".into());
        let html = render(&c);
        assert!(html.contains("data-orientation=\"vertical\""));
        assert!(!html.contains("data-orientation=\"horizontal\""));
        reject_interact(&html);
    }

    #[test]
    fn vertical_from_style() {
        let mut c = steps(&["Top", "Bottom"]);
        c.style = Some("stepper+vertical".into());
        let html = render(&c);
        assert!(html.contains("data-orientation=\"vertical\""));
        reject_interact(&html);
    }

    #[test]
    fn field_label_is_not_a_step() {
        let mut c = stub("stepper", "Wizard");
        c.items.push(extra("Account"));
        c.items.push(extra("Plan"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"stepper-item\"").count(), 2);
        assert!(html.contains(">Account</span>"));
        assert!(html.contains(">Plan</span>"));
        assert!(!html.contains(">Wizard</span>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_ol_nav() {
        let c = steps(&["Account", "Plan"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("stepper", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<ol data-slot=\"stepper\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("<li"));
        assert!(!interact.contains("data-slot=\"stepper-item\""));
        assert!(!interact.contains("data-slot=\"stepper-trigger\""));
        assert!(html.contains("data-slot=\"stepper-item\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&steps(&["Account", "Plan"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"stepper\"]"));
        assert!(css.contains("[data-slot=\"stepper-item\"]"));
        assert!(css.contains("[data-slot=\"stepper-trigger\"]"));
        assert!(css.contains("[data-slot=\"stepper-title\"]"));
        assert!(css.contains("flex-direction: row"));
        assert!(css.contains("flex-direction: column"));
        assert!(css.contains("[data-slot=\"stepper-item\"][data-state=\"upcoming\"]"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        assert!(!css.contains("zinc-"));
    }
}
