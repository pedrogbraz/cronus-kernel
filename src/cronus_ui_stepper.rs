//! Dedicated Stepper renderer. DOM matches React `Stepper > StepperList >
//! StepperItem(StepperIndicator + StepperTitle)`:
//! `<div data-slot="stepper"><ol data-slot="stepper-list">` plus per step
//! `<li data-slot="stepper-item" data-state="completed|active|upcoming">` with
//! `<span data-slot="stepper-indicator">` (1-based number, check once completed),
//! `<div data-slot="stepper-title">` and sr-only `<span data-slot="stepper-item-state">`.
//! Non-interactive like the React fixture (no StepperTrigger).
//! The `label` names the stepper; it is never a step.
//! Not interact `stepper()` (`<ol style=BASE>` numbered pills without stepper-list).

use crate::cronus_ui_kit::{attr, choice_texts, esc, label_of};
use crate::parser::ComponentNode;

const CHECK: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2.5\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M20 6 9 17l-5-5\"/></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let o = orientation_of(comp);
    let steps = steps_of(comp);
    let descriptions: Vec<String> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "item" | "tab") && !i.text.is_empty())
        .map(|i| {
            i.config
                .get("description")
                .map(|d| esc(d))
                .unwrap_or_default()
        })
        .collect();
    let separators = crate::cronus_ui_kit::flag(comp, "separators");
    let current = current_of(comp, steps.len());
    let items = steps
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let (state, spoken) = if i < current {
                ("completed", "completed")
            } else if i == current {
                ("active", "current")
            } else {
                ("upcoming", "not started")
            };
            let aria_current = if i == current { " aria-current=\"step\"" } else { "" };
            let mark = if state == "completed" {
                CHECK.to_string()
            } else {
                format!("<span>{}</span>", i + 1)
            };
            let desc = descriptions
                .get(i)
                .map(|d| format!("<div data-slot=\"stepper-description\">{d}</div>"))
                .unwrap_or_default();
            let sep = if separators && i + 1 < steps.len() {
                format!("<div data-slot=\"stepper-separator\" data-orientation=\"{o}\" aria-hidden=\"true\"></div>")
            } else {
                String::new()
            };
            format!(
                "<li data-slot=\"stepper-item\" data-state=\"{state}\" data-orientation=\"{o}\"{aria_current}><span data-slot=\"stepper-indicator\" data-state=\"{state}\">{mark}</span><div data-slot=\"stepper-title\">{t}</div>{desc}<span data-slot=\"stepper-item-state\">{spoken}</span>{sep}</li>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"stepper\" data-orientation=\"{o}\"><ol data-slot=\"stepper-list\" data-orientation=\"{o}\">{items}</ol></div>"
    )
}

fn steps_of(comp: &ComponentNode) -> Vec<String> {
    let choices = choice_texts(comp);
    if !choices.is_empty() {
        return choices;
    }
    let body: Vec<String> = comp
        .items
        .iter()
        .filter(|i| !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if body.is_empty() {
        vec![label_of(comp)]
    } else {
        body
    }
}

fn current_of(comp: &ComponentNode, n: usize) -> usize {
    let raw = attr(comp, "value").or_else(|| attr(comp, "current"));
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
            item_type: "text".into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn steps(label: &str, titles: &[&str]) -> ComponentNode {
        let mut c = stub("stepper", label);
        for t in titles {
            c.items.push(extra(t));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<nav"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("-control"));
        assert!(!html.contains("<button"));
    }

    #[test]
    fn emitted_fixture_matches_react_dom_label_is_not_a_step() {
        let html = render(&steps("Onboarding", &["Account", "Shipping"]));
        assert_eq!(
            html,
            "<div data-slot=\"stepper\" data-orientation=\"horizontal\"><ol data-slot=\"stepper-list\" data-orientation=\"horizontal\"><li data-slot=\"stepper-item\" data-state=\"active\" data-orientation=\"horizontal\" aria-current=\"step\"><span data-slot=\"stepper-indicator\" data-state=\"active\"><span>1</span></span><div data-slot=\"stepper-title\">Account</div><span data-slot=\"stepper-item-state\">current</span></li><li data-slot=\"stepper-item\" data-state=\"upcoming\" data-orientation=\"horizontal\"><span data-slot=\"stepper-indicator\" data-state=\"upcoming\"><span>2</span></span><div data-slot=\"stepper-title\">Shipping</div><span data-slot=\"stepper-item-state\">not started</span></li></ol></div>"
        );
        reject_interact(&html);
    }

    #[test]
    fn value_marks_completed_with_check() {
        let mut c = steps("W", &["Account", "Plan", "Confirm"]);
        c.props.insert("value".into(), "1".into());
        let html = render(&c);
        assert!(html.contains(&format!(
            "data-state=\"completed\" data-orientation=\"horizontal\"><span data-slot=\"stepper-indicator\" data-state=\"completed\">{CHECK}</span><div data-slot=\"stepper-title\">Account</div><span data-slot=\"stepper-item-state\">completed</span>"
        )));
        assert!(html.contains(
            "data-state=\"active\" data-orientation=\"horizontal\" aria-current=\"step\""
        ));
        assert_eq!(html.matches("aria-current=\"step\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn vertical_from_props_and_style() {
        let mut c = steps("W", &["Top", "Bottom"]);
        c.props.insert("orientation".into(), "vertical".into());
        let html = render(&c);
        assert!(html.contains("<ol data-slot=\"stepper-list\" data-orientation=\"vertical\">"));
        assert!(!html.contains("data-orientation=\"horizontal\""));
        let mut s = steps("W", &["Top", "Bottom"]);
        s.style = Some("stepper+vertical".into());
        assert!(render(&s).contains("data-orientation=\"vertical\""));
    }

    #[test]
    fn skips_interact_ol_and_passes_stub_gate() {
        let c = steps("W", &["Account", "Plan"]);
        let html = render(&c);
        assert!(!crate::cli::stub_renderer_gate::looks_like_interact_generic(&html));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            reject_interact(&render(&steps("W", &["Account", "Plan"])));
        });
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"stepper-list\"]"));
        assert!(css.contains("[data-slot=\"stepper-item\"][data-orientation=\"horizontal\"]:not(:last-child) { flex: 1; }"));
        assert!(css.contains("[data-slot=\"stepper-indicator\"][data-state=\"active\"]"));
        assert!(css.contains("[data-slot=\"stepper-indicator\"][data-state=\"upcoming\"]"));
        assert!(css.contains("[data-slot=\"stepper-item-state\"]"));
        assert!(css.contains(
            "border-radius: 9999px; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;"
        ));
        assert!(!css.contains("[data-slot=\"stepper-trigger\"]"));
        assert!(!css.contains("zinc-"));
    }
}
