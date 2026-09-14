//! Dedicated MultiSelect renderer. DOM mirrors React (Radix popover open,
//! `defaultOpen`): trigger
//! `<div data-slot="multi-select-trigger" role="combobox" tabindex="0">` with
//! the placeholder/selection span and a chevron, then the open content tree
//! React portals — `popover-content > command > command-list[role=listbox] >
//! command-item[role=option]` with a `multi-select-indicator` box + label.
//! Kernel wraps both in `data-slot="multi-select"` (position: relative) so the
//! content sits absolutely 4px below the trigger, like the Radix popper.
//! React always renders cmdk's search row (`command-input-wrapper` > search
//! icon + `command-input`, placeholder `searchPlaceholder`, default "Search…")
//! above the list. Filtering needs JS, so the input is the native control
//! rendered `disabled` with React's idle look.
//! Not interact `select("multi-select")` native `<select multiple>`.

use crate::cronus_ui_kit::{attr_nonempty, choice_texts, esc, flag, item, texts};
use crate::parser::ComponentNode;

const CHEVRON: &str = concat!(
    "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" ",
    "stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">",
    "<path d=\"m6 9 6 6 6-6\" />",
    "</svg>",
);

const CHECK: &str = concat!(
    "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" ",
    "stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">",
    "<path d=\"M20 6 9 17l-5-5\" />",
    "</svg>",
);

const SEARCH_ICON: &str = concat!(
    "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" ",
    "stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">",
    "<circle cx=\"11\" cy=\"11\" r=\"8\" /><path d=\"m21 21-4.3-4.3\" /></svg>",
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
        .enumerate()
        .map(|(i, t)| {
            let on = selected.iter().any(|s| s == t);
            let (aria, state, mark) = if on {
                ("true", "checked", CHECK)
            } else {
                ("false", "unchecked", "")
            };
            // cmdk highlights the first row on open (`data-selected`); CSS moves
            // the highlight to the hovered row, so no JS is needed.
            let highlighted = if i == 0 { " data-selected=\"true\"" } else { "" };
            format!(
                "<div data-slot=\"command-item\" role=\"option\" aria-selected=\"{aria}\"{highlighted}><span data-slot=\"multi-select-indicator\" data-state=\"{state}\" aria-hidden=\"true\">{mark}</span><span>{t}</span></div>"
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
    if let Some(label) = attr_nonempty(comp, "aria-label") {
        trigger_attrs.push_str(&format!(" aria-label=\"{}\"", esc(label)));
    }
    if flag(comp, "invalid") {
        trigger_attrs.push_str(" aria-invalid=\"true\"");
    }
    trigger_attrs.push_str(" data-state=\"open\"");
    if selected.is_empty() {
        trigger_attrs.push_str(" data-placeholder=\"\"");
    }
    let search = search_row(comp);
    format!(
        "<div data-slot=\"multi-select\"><div {trigger_attrs}><span><span>{trigger}</span></span><span aria-hidden=\"true\">{CHEVRON}</span></div><div data-slot=\"popover-content\" role=\"dialog\" data-state=\"open\"><div data-slot=\"command\">{search}<div data-slot=\"command-list\" role=\"listbox\" aria-multiselectable=\"true\">{items}</div></div></div></div>"
    )
}

/// cmdk `CommandInput`: icon + native text input, `disabled` (filtering needs JS).
fn search_row(comp: &ComponentNode) -> String {
    let placeholder = attr_nonempty(comp, "searchPlaceholder")
        .or_else(|| attr_nonempty(comp, "search-placeholder"))
        .map(esc)
        .unwrap_or_else(|| "Search…".into());
    format!(
        "<div data-slot=\"command-input-wrapper\">{SEARCH_ICON}<input data-slot=\"command-input\" type=\"text\" placeholder=\"{placeholder}\" role=\"combobox\" aria-autocomplete=\"list\" aria-expanded=\"true\" autocomplete=\"off\" spellcheck=\"false\" disabled /></div>"
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
    if let Some(v) = attr_nonempty(comp, "placeholder") {
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
    if let Some(v) = attr_nonempty(comp, "value") {
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

fn is_true(raw: Option<&String>) -> bool {
    matches!(raw.map(String::as_str), Some("true" | "on" | "1"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const TRIGGER_OPEN: &str = "<div data-slot=\"multi-select-trigger\" role=\"combobox\" tabindex=\"0\" aria-expanded=\"true\" aria-haspopup=\"listbox\" data-state=\"open\" data-placeholder=\"\">";

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

    fn opt(t: &str, on: bool) -> String {
        if on {
            format!("<div data-slot=\"command-item\" role=\"option\" aria-selected=\"true\"><span data-slot=\"multi-select-indicator\" data-state=\"checked\" aria-hidden=\"true\">{CHECK}</span><span>{t}</span></div>")
        } else {
            format!("<div data-slot=\"command-item\" role=\"option\" aria-selected=\"false\"><span data-slot=\"multi-select-indicator\" data-state=\"unchecked\" aria-hidden=\"true\"></span><span>{t}</span></div>")
        }
    }

    /// First row: cmdk's initial keyboard highlight (`data-selected="true"`).
    fn first(t: &str, on: bool) -> String {
        opt(t, on).replacen("\">", "\" data-selected=\"true\">", 1)
    }

    /// React measured the first command-item with the highlight background on
    /// open; only that row is marked, the rest stay plain.
    #[test]
    fn only_first_row_carries_initial_highlight() {
        let html = render(&multi("Pick", &["Ada", "Grace", "Linus"]));
        assert_eq!(html.matches("data-selected=\"true\"").count(), 1);
        assert!(html.contains(&first("Ada", false)));
        assert!(html.contains(&opt("Grace", false)));
        assert!(html.contains(&opt("Linus", false)));
        reject_interact(&html);
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<select"));
        assert!(!html.contains("</select>"));
        assert!(!html.contains(" multiple"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("<button"));
        // The only input is cmdk's search field, always native + disabled.
        assert_eq!(
            html.matches("<input").count(),
            html.matches("<input data-slot=\"command-input\" type=\"text\"")
                .count()
        );
        assert_eq!(
            html.matches("<input").count(),
            html.matches("spellcheck=\"false\" disabled />").count()
        );
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
    }

    /// React (defaultOpen): div combobox trigger + portal
    /// `popover-content > command > command-list > command-item`. Options are
    /// divs with an indicator box, never buttons (wave1s geometry parity).
    #[test]
    fn trigger_is_div_combobox_and_list_is_open() {
        let html = render(&multi("Pick", &["Ada", "Grace"]));
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"multi-select\">{TRIGGER_OPEN}{}<span aria-hidden=\"true\">{CHEVRON}</span></div><div data-slot=\"popover-content\" role=\"dialog\" data-state=\"open\"><div data-slot=\"command\"><div data-slot=\"command-input-wrapper\">{SEARCH_ICON}<input data-slot=\"command-input\" type=\"text\" placeholder=\"Search…\" role=\"combobox\" aria-autocomplete=\"list\" aria-expanded=\"true\" autocomplete=\"off\" spellcheck=\"false\" disabled /></div><div data-slot=\"command-list\" role=\"listbox\" aria-multiselectable=\"true\">{}{}</div></div></div></div>",
                trig("Pick"),
                first("Ada", false),
                opt("Grace", false)
            )
        );
        assert!(!html.contains("data-slot=\"multi-select-content\""));
        assert!(!html.contains("data-slot=\"multi-select-item\""));
        reject_interact(&html);
    }

    /// Pixel parity (multi-select/default): React's open content starts with
    /// cmdk's search row before the options; the kernel used to omit it.
    #[test]
    fn content_has_disabled_search_row_before_list() {
        let html = render(&multi("Pick", &["Ada"]));
        let row = html
            .find("<div data-slot=\"command-input-wrapper\">")
            .expect("search row");
        let list = html.find("data-slot=\"command-list\"").unwrap();
        assert!(row < list, "{html}");
        assert!(html.contains("placeholder=\"Search…\""));
        let mut c = multi("Pick", &["Ada"]);
        c.props
            .insert("searchPlaceholder".into(), "Find \"x\"".into());
        assert!(render(&c).contains("placeholder=\"Find &quot;x&quot;\""));
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
            "<div data-slot=\"multi-select-trigger\" role=\"combobox\" tabindex=\"0\" aria-expanded=\"true\" aria-haspopup=\"listbox\" aria-label=\"Stack\" data-state=\"open\" data-placeholder=\"\">"
        ));
        assert!(html.contains(&trig("Select frameworks")));
        assert_eq!(html.matches("role=\"option\"").count(), 2);
        assert!(html.contains(&first("React", false)));
        assert!(html.contains(&opt("Vue", false)));
        assert!(!html.contains("<span>Select frameworks</span></div>"));
        reject_interact(&html);
    }

    #[test]
    fn label_is_placeholder_not_an_option() {
        let html = render(&multi("Pick", &["Ada", "Grace"]));
        assert!(html.contains(&trig("Pick")));
        assert_eq!(html.matches("role=\"option\"").count(), 2);
        assert!(!html.contains(&opt("Pick", false)));
        reject_interact(&html);
    }

    #[test]
    fn extra_text_items_become_options() {
        let mut c = stub("multi-select", "Pick");
        c.items.push(node("text", "Ada"));
        c.items.push(node("text", "Grace"));
        let html = render(&c);
        assert!(html.contains(&trig("Pick")));
        assert!(html.contains(&first("Ada", false)));
        assert!(html.contains(&opt("Grace", false)));
        assert_eq!(html.matches("data-slot=\"command-item\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn value_selects_matching_options() {
        let mut c = multi("Pick", &["Ada", "Grace", "Linus"]);
        c.props.insert("value".into(), "Ada, Linus".into());
        let html = render(&c);
        assert!(html.contains(&trig("Ada, Linus")));
        assert!(html.contains(&first("Ada", true)));
        assert!(html.contains(&opt("Grace", false)));
        assert!(html.contains(&opt("Linus", true)));
        assert!(!html.contains("data-placeholder"));
        reject_interact(&html);
    }

    #[test]
    fn selected_item_config_marks_options() {
        let mut c = multi("Pick", &["Ada", "Grace"]);
        c.items[1].config.insert("selected".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(&trig("Ada")));
        assert!(html.contains(&first("Ada", true)));
        assert!(html.contains(&opt("Grace", false)));
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
        assert!(!html.contains("data-slot=\"command-item\""));
        assert!(html.contains(&trig("Pick")));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_native_select() {
        let c = multi("Pick", &["Ada", "Grace"]);
        let html = render(&c);
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

    /// Measured against React (wave1s): trigger 432x40 radius-lg surface-inset
    /// with border-strong while open; content 4px below, full trigger width;
    /// rows 32px (px-2 py-1.5, 20px line) with a 16px indicator box.
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"multi-select\"] {\n  position: relative; display: block; width: 100%;"
        ));
        assert!(css.contains(
            "[data-slot=\"multi-select-trigger\"] {\n  display: flex; align-items: center; justify-content: space-between; gap: 0.5rem;\n  width: 100%; min-height: 2.5rem; padding: 0.375rem 0.75rem; box-sizing: border-box;"
        ));
        assert!(css.contains("[data-slot=\"multi-select-trigger\"][aria-expanded=\"true\"] { border-color: var(--cronus-border-strong); }"));
        assert!(css.contains("[data-slot=\"multi-select\"] > [data-slot=\"popover-content\"] {\n  position: absolute; top: calc(100% + 4px); inset-inline-start: 0; z-index: 50;"));
        assert!(css.contains("[data-slot=\"multi-select\"] [data-slot=\"command-item\"] {\n  position: relative; display: flex; align-items: center; gap: 0.5rem;"));
        assert!(css.contains("padding: 0.375rem 0.5rem;"));
        assert!(css.contains("[data-slot=\"multi-select-indicator\"] {\n  display: flex; align-items: center; justify-content: center; flex-shrink: 0;\n  width: 1rem; height: 1rem;"));
        assert!(css.contains("[data-slot=\"multi-select-indicator\"][data-state=\"checked\"]"));
        // cmdk initial highlight on the first row, handed to the hovered row by CSS.
        assert!(css.contains(
            "[data-slot=\"multi-select\"] [data-slot=\"command-list\"]:not(:hover) [data-slot=\"command-item\"][data-selected=\"true\"],\n[data-slot=\"multi-select\"] [data-slot=\"command-item\"]:hover {"
        ));
        assert!(!css.contains("[data-slot=\"multi-select-item\"]"));
        assert!(!css.contains("[data-slot=\"multi-select-content\"]"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
