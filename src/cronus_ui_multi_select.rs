//! Dedicated MultiSelect renderer. DOM mirrors React (Radix popover open):
//! wrapper `data-slot="multi-select"`, trigger
//! `<div data-slot="multi-select-trigger" role="combobox" tabindex="0">` with
//! the placeholder/selection span and a chevron, plus an open listbox of
//! options. Not interact `select("multi-select")` native `<select multiple>`.

use crate::cronus_ui_kit::{choice_texts, esc, item, texts};
use crate::parser::ComponentNode;

const CHEVRON: &str = concat!(
    "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" ",
    "stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">",
    "<path d=\"m6 9 6 6 6-6\" />",
    "</svg>",
);

pub fn render(comp: &ComponentNode) -> String {
    let options = extra_texts(comp);
    let selected = selected_of(comp, &options);
    let trigger = if selected.is_empty() {
        placeholder_of(comp)
    } else {
        selected.join(", ")
    };
    let items = options
        .iter()
        .map(|t| {
            let aria = if selected.iter().any(|s| s == t) {
                "true"
            } else {
                "false"
            };
            format!(
                "<button type=\"button\" data-slot=\"multi-select-item\" role=\"option\" aria-selected=\"{aria}\">{t}</button>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let mut trigger_attrs = String::from("data-slot=\"multi-select-trigger\" role=\"combobox\"");
    if flag(comp, "disabled") {
        trigger_attrs.push_str(" tabindex=\"-1\" aria-disabled=\"true\"");
    } else {
        trigger_attrs.push_str(" tabindex=\"0\"");
    }
    trigger_attrs.push_str(" aria-expanded=\"true\" aria-haspopup=\"listbox\"");
    if let Some(label) = attr(comp, "aria-label").filter(|s| !s.is_empty()) {
        trigger_attrs.push_str(&format!(" aria-label=\"{}\"", esc(label)));
    }
    if flag(comp, "invalid") {
        trigger_attrs.push_str(" aria-invalid=\"true\"");
    }
    format!(
        "<div data-slot=\"multi-select\"><div {trigger_attrs}><span><span>{trigger}</span></span><span aria-hidden=\"true\">{CHEVRON}</span></div><div data-slot=\"multi-select-content\" role=\"listbox\">{items}</div></div>"
    )
}

/// Options from choice lines, else extra texts. The audit emitter writes the
/// placeholder as a `text` line too, so the placeholder is never an option.
fn extra_texts(comp: &ComponentNode) -> Vec<String> {
    let placeholder = placeholder_of(comp);
    let choices = choice_texts(comp);
    let base = if choices.is_empty() {
        texts(comp).into_iter().skip(1).collect()
    } else {
        choices
    };
    base.into_iter().filter(|t| *t != placeholder).collect()
}

fn placeholder_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr(comp, "placeholder").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    if let Some(t) = item(comp, "placeholder").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    for kind in ["label", "title"] {
        if let Some(t) = item(comp, kind).filter(|s| !s.is_empty()) {
            return esc(t);
        }
    }
    "Select…".into()
}

fn selected_of(comp: &ComponentNode, options: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(v) = attr(comp, "value").filter(|s| !s.is_empty()) {
        for part in v.split(',') {
            let e = esc(part.trim());
            if !e.is_empty() && options.iter().any(|o| o == &e) && !out.contains(&e) {
                out.push(e);
            }
        }
        if !out.is_empty() {
            return out;
        }
    }
    for i in &comp.items {
        if i.text.is_empty() {
            continue;
        }
        if is_true(i.config.get("selected")) || is_true(i.config.get("checked")) {
            let e = esc(&i.text);
            if options.iter().any(|o| o == &e) && !out.contains(&e) {
                out.push(e);
            }
        }
    }
    out
}

fn attr<'a>(comp: &'a ComponentNode, name: &str) -> Option<&'a str> {
    if let Some(v) = comp.props.get(name) {
        return Some(v.as_str());
    }
    comp.items
        .iter()
        .find_map(|i| i.config.get(name).map(String::as_str))
}

fn flag(comp: &ComponentNode, name: &str) -> bool {
    attr(comp, name).map(|s| s == "true").unwrap_or(false)
}

fn is_true(raw: Option<&String>) -> bool {
    matches!(raw.map(String::as_str), Some("true" | "on" | "1"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const TRIGGER_OPEN: &str = "<div data-slot=\"multi-select-trigger\" role=\"combobox\" tabindex=\"0\" aria-expanded=\"true\" aria-haspopup=\"listbox\">";

    fn node(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn multi(placeholder: &str, items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("multi-select", placeholder);
        for t in items {
            c.items.push(node("item", t));
        }
        c
    }

    fn trig(t: &str) -> String {
        format!("<span><span>{t}</span></span>")
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<select"));
        assert!(!html.contains("</select>"));
        assert!(!html.contains(" multiple"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
    }

    #[test]
    fn trigger_is_div_combobox_and_list_is_open() {
        let html = render(&multi("Pick", &["Ada", "Grace"]));
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"multi-select\">{TRIGGER_OPEN}{}<span aria-hidden=\"true\">{CHEVRON}</span></div><div data-slot=\"multi-select-content\" role=\"listbox\"><button type=\"button\" data-slot=\"multi-select-item\" role=\"option\" aria-selected=\"false\">Ada</button><button type=\"button\" data-slot=\"multi-select-item\" role=\"option\" aria-selected=\"false\">Grace</button></div></div>",
                trig("Pick")
            )
        );
        assert!(!html.contains("<button type=\"button\" data-slot=\"multi-select-trigger\""));
        reject_interact(&html);
    }

    /// Audit fixture: `label` + `text` both carry the placeholder, then the
    /// options; `aria-label:` lands on the last text item. React trigger is a
    /// DIV labelled "Stack" and the placeholder is not an option.
    #[test]
    fn fixture_placeholder_is_not_an_option_and_aria_label_applies() {
        let mut c = stub("multi-select", "Select frameworks");
        c.items.push(node("text", "Select frameworks"));
        c.items.push(node("text", "React"));
        let mut vue = node("text", "Vue");
        vue.config.insert("aria-label".into(), "Stack".into());
        c.items.push(vue);
        let html = render(&c);
        assert!(html.contains(
            "<div data-slot=\"multi-select-trigger\" role=\"combobox\" tabindex=\"0\" aria-expanded=\"true\" aria-haspopup=\"listbox\" aria-label=\"Stack\">"
        ));
        assert!(html.contains(&trig("Select frameworks")));
        assert_eq!(html.matches("role=\"option\"").count(), 2);
        assert!(!html.contains("aria-selected=\"false\">Select frameworks</button>"));
        reject_interact(&html);
    }

    #[test]
    fn label_is_placeholder_not_an_option() {
        let html = render(&multi("Pick", &["Ada", "Grace"]));
        assert!(html.contains(&trig("Pick")));
        assert_eq!(html.matches("role=\"option\"").count(), 2);
        for chunk in html.split("data-slot=\"multi-select-item\"").skip(1) {
            assert!(
                !chunk.contains(">Pick</button>"),
                "Pick leaked as option: {html}"
            );
        }
        reject_interact(&html);
    }

    #[test]
    fn extra_text_items_become_options() {
        let mut c = stub("multi-select", "Pick");
        c.items.push(node("text", "Ada"));
        c.items.push(node("text", "Grace"));
        let html = render(&c);
        assert!(html.contains(&trig("Pick")));
        assert!(html.contains("aria-selected=\"false\">Ada</button>"));
        assert!(html.contains("aria-selected=\"false\">Grace</button>"));
        assert_eq!(html.matches("data-slot=\"multi-select-item\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn value_selects_matching_options() {
        let mut c = multi("Pick", &["Ada", "Grace", "Linus"]);
        c.props.insert("value".into(), "Ada, Linus".into());
        let html = render(&c);
        assert!(html.contains(&trig("Ada, Linus")));
        assert!(html.contains("aria-selected=\"true\">Ada</button>"));
        assert!(html.contains("aria-selected=\"false\">Grace</button>"));
        assert!(html.contains("aria-selected=\"true\">Linus</button>"));
        reject_interact(&html);
    }

    #[test]
    fn selected_item_config_marks_options() {
        let mut c = multi("Pick", &["Ada", "Grace"]);
        c.items[1]
            .config
            .insert("selected".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(&trig("Ada")));
        assert!(html.contains("aria-selected=\"true\">Ada</button>"));
        assert!(html.contains("aria-selected=\"false\">Grace</button>"));
        reject_interact(&html);
    }

    #[test]
    fn disabled_trigger_is_aria_disabled_and_unfocusable() {
        let mut c = multi("Pick", &["Ada"]);
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("role=\"combobox\" tabindex=\"-1\" aria-disabled=\"true\""));
        assert!(!html.contains("tabindex=\"0\""));
        reject_interact(&html);
    }

    #[test]
    fn empty_choices_keep_open_list() {
        let html = render(&stub("multi-select", "Pick"));
        assert!(html.contains("data-slot=\"multi-select-trigger\""));
        assert!(html.contains("role=\"listbox\""));
        assert!(!html.contains("data-slot=\"multi-select-item\""));
        assert!(html.contains(&trig("Pick")));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_native_select() {
        let c = multi("Pick", &["Ada", "Grace"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("multi-select", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<label data-slot=\"multi-select\""));
        assert!(interact.contains("<select data-slot=\"multi-select-control\""));
        assert!(interact.contains("style="));
        assert!(!html.contains("multi-select-control"));
        assert!(!html.contains("<select"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&multi("Pick", &["Ada", "Grace"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"multi-select\"]"));
        assert!(css.contains("[data-slot=\"multi-select-trigger\"]"));
        assert!(css.contains("[data-slot=\"multi-select-trigger\"][aria-disabled=\"true\"]"));
        assert!(css.contains("[data-slot=\"multi-select-trigger\"] svg"));
        assert!(css.contains("[data-slot=\"multi-select-content\"]"));
        assert!(css.contains("[data-slot=\"multi-select-item\"]"));
        assert!(css.contains("justify-content: space-between"));
        assert!(css.contains("min-width: 8rem"));
        assert!(css.contains("var(--cronus-surface-floating"));
        assert!(css.contains("[data-slot=\"multi-select-item\"][aria-selected=\"true\"]"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
