//! Dedicated Command renderer. DOM mirrors React/cmdk:
//! `<div data-slot="command">` > visually-hidden `<label>` (accessible name) >
//! `<div data-slot="command-input-wrapper">` (search icon + `command-input`) >
//! `<div data-slot="command-list" role="listbox">` > sizer `<div>` >
//! `command-item[role=option]`. The first item carries cmdk's initial highlight
//! (`data-selected="true"`). The search field is a live native `<input
//! data-slot="command-input">` (disabled only when the author sets `disabled`);
//! page runtime filters `[data-slot=command-item]` text against it.
//!
//! Emitted source order is `label` (widget name), then the placeholder `text`
//! when the fixture has one, then one `text` per item. A first text equal to the
//! label or ending in an ellipsis is the placeholder, not an item.
//! Rich `item` lines carry `icon:` (lucide), `shortcut:"⌘S"` (`command-shortcut`)
//! and `group:"Suggestions"`: grouped items render as cmdk `command-group`s
//! (heading + `role="group"` list) with a `command-separator` between groups.
//! `empty:"…"` is the `command-empty` message (rendered, like cmdk, only when
//! the list has no items).
//!
//! Closed mode (`trigger:"…"`; `cronus_ui_kit::overlay_trigger`): the docs'
//! `CommandDialog`. The trigger (`cronus_ui_dialog::trigger_button`, outline)
//! carries a secondary `badge` (`trigger-badge:"⌘K"`) and `command="show-modal"`
//! opens a native `<dialog data-slot="dialog-content" class="command-dialog">`
//! (React's glassy `DialogContent` overrides) holding the visually hidden
//! `dialog-header` (title = label, `description:`), the `command` and the
//! `dialog-close` button. Gap: the ⌘K shortcut needs JS.
//! Not interact `popover("command")` (`<details>` SURF box).

use crate::cronus_ui_dialog::trigger_button;
use crate::cronus_ui_kit::{
    attr_nonempty, choice_texts, esc, flag, item, item_icon, modal_close_attrs, modal_dialog_open,
    overlay_trigger, widget_id,
};
use crate::parser::{ComponentItemNode, ComponentNode};

const CROSS: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M18 6 6 18\"></path><path d=\"m6 6 12 12\"></path></svg>";

const SEARCH_ICON: &str = concat!(
    "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" ",
    "stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">",
    "<circle cx=\"11\" cy=\"11\" r=\"8\" /><path d=\"m21 21-4.3-4.3\" /></svg>",
);

pub fn render(comp: &ComponentNode) -> String {
    let parts = parts_of(comp);
    let input_id = widget_id(comp, "input");
    let rich: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .collect();
    let grouped = rich.iter().any(|i| i.config.contains_key("group"));
    let mut n = 0usize;
    let mut item_html = |t: String, icon: String, shortcut: String| {
        let selected = if n == 0 { "true" } else { "false" };
        n += 1;
        format!(
            "<div data-slot=\"command-item\" role=\"option\" aria-selected=\"{selected}\" data-selected=\"{selected}\">{icon}{t}{shortcut}</div>"
        )
    };
    let list = if grouped {
        let groups = rich.iter().fold(Vec::<String>::new(), |mut acc, i| {
            let g = i
                .config
                .get("group")
                .map(|g| g.trim())
                .unwrap_or("")
                .to_string();
            if !acc.contains(&g) {
                acc.push(g);
            }
            acc
        });
        groups
            .iter()
            .enumerate()
            .map(|(gi, g)| {
                let entries: String = rich
                    .iter()
                    .filter(|i| i.config.get("group").map(|x| x.trim()).unwrap_or("") == g)
                    .map(|i| item_html(esc(&i.text), item_icon(i), shortcut_of(i)))
                    .collect();
                let heading = if g.is_empty() {
                    String::new()
                } else {
                    format!("<div cmdk-group-heading=\"\" aria-hidden=\"true\">{}</div>", esc(g))
                };
                let sep = if gi > 0 {
                    "<div data-slot=\"command-separator\" role=\"separator\"></div>"
                } else {
                    ""
                };
                format!(
                    "{sep}<div data-slot=\"command-group\" role=\"presentation\">{heading}<div role=\"group\">{entries}</div></div>"
                )
            })
            .collect::<String>()
    } else if !rich.is_empty() {
        rich.iter()
            .map(|i| item_html(esc(&i.text), item_icon(i), shortcut_of(i)))
            .collect()
    } else {
        parts
            .items
            .iter()
            .map(|t| item_html(t.clone(), String::new(), String::new()))
            .collect()
    };
    let empty = if n == 0 {
        attr_nonempty(comp, "empty")
            .map(|e| {
                format!(
                    "<div data-slot=\"command-empty\" role=\"presentation\">{}</div>",
                    esc(e)
                )
            })
            .unwrap_or_default()
    } else {
        String::new()
    };
    let disabled = if flag(comp, "disabled") {
        " disabled"
    } else {
        ""
    };
    let command = format!(
        "<div data-slot=\"command\"><label for=\"{input_id}\">{label}</label><div data-slot=\"command-input-wrapper\">{SEARCH_ICON}<input data-slot=\"command-input\" id=\"{input_id}\" type=\"text\" placeholder=\"{placeholder}\" role=\"combobox\" aria-autocomplete=\"list\" aria-expanded=\"true\" autocomplete=\"off\" spellcheck=\"false\"{disabled} /></div><div data-slot=\"command-list\" role=\"listbox\" aria-label=\"Suggestions\"><div>{empty}{list}</div></div></div>",
        label = parts.label,
        placeholder = parts.placeholder,
    );
    let Some(trigger) = overlay_trigger(comp, "Open") else {
        return command;
    };
    let dialog_id = widget_id(comp, "dialog");
    let trigger_id = widget_id(comp, "trigger");
    let title_id = widget_id(comp, "dialog-title");
    let desc_id = widget_id(comp, "dialog-description");
    let badge = attr_nonempty(comp, "trigger-badge")
        .map(|b| {
            format!(
                "<span data-slot=\"badge\" data-variant=\"secondary\">{}</span>",
                esc(b)
            )
        })
        .unwrap_or_default();
    let description = attr_nonempty(comp, "description")
        .map(esc)
        .unwrap_or_else(|| "Search for a command to run...".into());
    let described = format!(" aria-describedby=\"{desc_id}\"");
    let open = modal_dialog_open(
        &dialog_id,
        "dialog-content",
        "dialog",
        &title_id,
        &described,
        true,
    )
    .replace(
        " role=\"dialog\"",
        " class=\"command-dialog\" role=\"dialog\"",
    );
    format!(
        "{}{open}<div data-slot=\"dialog-header\"><h2 data-slot=\"dialog-title\" id=\"{title_id}\">{}</h2><p data-slot=\"dialog-description\" id=\"{desc_id}\">{description}</p></div>{command}<button type=\"button\" data-slot=\"dialog-close\"{}>{CROSS}<span>Close</span></button></dialog>",
        trigger_button(comp, &trigger_id, &dialog_id, &format!("{trigger}{badge}")),
        parts.label,
        modal_close_attrs(&dialog_id),
    )
}

fn shortcut_of(i: &ComponentItemNode) -> String {
    i.config
        .get("shortcut")
        .filter(|s| !s.trim().is_empty())
        .map(|s| format!("<span data-slot=\"command-shortcut\">{}</span>", esc(s)))
        .unwrap_or_default()
}

struct Parts {
    label: String,
    placeholder: String,
    items: Vec<String>,
}

fn parts_of(comp: &ComponentNode) -> Parts {
    let label = ["label", "title"]
        .iter()
        .find_map(|k| item(comp, k).filter(|s| !s.is_empty()))
        .map(esc)
        .unwrap_or_else(|| "Search…".into());
    let choices = choice_texts(comp);
    let mut rest: Vec<String> = if choices.is_empty() {
        comp.items
            .iter()
            .filter(|i| i.item_type == "text" && !i.text.is_empty())
            .map(|i| esc(&i.text))
            .collect()
    } else {
        Vec::new()
    };
    let explicit = comp
        .props
        .get("placeholder")
        .map(String::as_str)
        .or_else(|| item(comp, "placeholder"))
        .filter(|s| !s.is_empty())
        .map(esc);
    let placeholder = match explicit {
        Some(p) => p,
        None => match rest.first() {
            Some(first) if *first == label || first.ends_with('…') || first.ends_with("...") => {
                rest.remove(0)
            }
            _ => label.clone(),
        },
    };
    let items = if choices.is_empty() { rest } else { choices };
    Parts {
        label,
        placeholder,
        items,
    }
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

    fn palette(label: &str, items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("command", label);
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<details"));
        assert!(!html.contains("<summary"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("position:absolute;z-index:20"));
        assert!(!html.contains("role=\"dialog\""));
        assert!(!html.contains("max-width:28rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("popover("));
    }

    #[test]
    fn emitted_fixture_splits_label_placeholder_and_items() {
        let mut c = stub("command", "Command menu");
        c.items.push(extra("text", "Type a command…"));
        c.items.push(extra("text", "Calendar"));
        c.items.push(extra("text", "Search"));
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"command\"><label for=\"cui-command-input\">Command menu</label><div data-slot=\"command-input-wrapper\">{SEARCH_ICON}<input data-slot=\"command-input\" id=\"cui-command-input\" type=\"text\" placeholder=\"Type a command…\" role=\"combobox\" aria-autocomplete=\"list\" aria-expanded=\"true\" autocomplete=\"off\" spellcheck=\"false\" /></div><div data-slot=\"command-list\" role=\"listbox\" aria-label=\"Suggestions\"><div><div data-slot=\"command-item\" role=\"option\" aria-selected=\"true\" data-selected=\"true\">Calendar</div><div data-slot=\"command-item\" role=\"option\" aria-selected=\"false\" data-selected=\"false\">Search</div></div></div></div>"
            )
        );
        assert!(!html.contains("command-item\" role=\"option\" aria-selected=\"true\" data-selected=\"true\">Type a command"));
        reject_interact(&html);
    }

    #[test]
    fn item_kinds_are_items_and_label_is_placeholder() {
        let html = render(&palette("Search", &["Calendar", "Profile"]));
        assert!(html.contains("placeholder=\"Search\""));
        assert!(html.contains("<label for=\"cui-command-input\">Search</label>"));
        assert!(html.contains("data-slot=\"command-input\""));
        assert!(!html.contains("spellcheck=\"false\" disabled"));
        assert_eq!(html.matches("data-slot=\"command-item\"").count(), 2);
        assert!(html.contains("data-selected=\"true\">Calendar</div>"));
        assert!(html.contains("data-selected=\"false\">Profile</div>"));
        reject_interact(&html);
    }

    #[test]
    fn disabled_prop_disables_search_input() {
        let mut c = palette("Search", &["Calendar"]);
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("spellcheck=\"false\" disabled"));
        reject_interact(&html);
    }

    #[test]
    fn plain_texts_without_placeholder_are_items() {
        let mut c = stub("command", "Search");
        c.items.push(extra("text", "Calendar"));
        c.items.push(extra("text", "Profile"));
        let html = render(&c);
        assert!(html.contains("placeholder=\"Search\""));
        assert_eq!(html.matches("data-slot=\"command-item\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn label_only_keeps_empty_list() {
        let html = render(&stub("command", "Search"));
        assert!(html.contains("placeholder=\"Search\""));
        assert!(html.contains("aria-label=\"Suggestions\"><div></div></div>"));
        assert!(!html.contains("data-slot=\"command-item\""));
        reject_interact(&html);
    }

    #[test]
    fn placeholder_from_props() {
        let mut c = palette("Search", &["Calendar"]);
        c.props
            .insert("placeholder".into(), "Type a command…".into());
        let html = render(&c);
        assert!(html.contains("placeholder=\"Type a command…\""));
        assert!(html.contains("data-selected=\"true\">Calendar</div>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_popover_surf() {
        let c = palette("Search", &["Calendar"]);
        let html = render(&c);
        assert!(html.contains("data-slot=\"command-input\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&palette("Search", &["Calendar"]));
            reject_interact(&html);
        });
    }

    fn docs_palette() -> crate::parser::ComponentNode {
        let mut c = stub("command", "Command palette");
        c.props
            .insert("trigger".into(), "Open command palette".into());
        c.props.insert("trigger-variant".into(), "outline".into());
        c.props.insert("trigger-badge".into(), "⌘K".into());
        c.props
            .insert("placeholder".into(), "Type a command or search…".into());
        c.props.insert("empty".into(), "No results found.".into());
        c.props
            .insert("description".into(), "Search for a command to run.".into());
        let mut cal = extra("item", "Calendar");
        cal.config.insert("icon".into(), "calendar-days".into());
        cal.config.insert("group".into(), "Suggestions".into());
        c.items.push(cal);
        let mut settings = extra("item", "Settings");
        settings.config.insert("icon".into(), "settings".into());
        settings.config.insert("shortcut".into(), "⌘S".into());
        settings.config.insert("group".into(), "Settings".into());
        c.items.push(settings);
        c
    }

    #[test]
    fn docs_command_dialog_with_groups_shortcuts_and_badge() {
        let c = docs_palette();
        let html = render(&c);
        let did = widget_id(&c, "dialog");
        let tid = widget_id(&c, "trigger");
        assert!(html.starts_with(&format!(
            "<button type=\"button\" id=\"{tid}\" data-slot=\"button\" data-variant=\"outline\" commandfor=\"{did}\" command=\"show-modal\" aria-haspopup=\"dialog\">Open command palette<span data-slot=\"badge\" data-variant=\"secondary\">⌘K</span></button><dialog id=\"{did}\" data-slot=\"dialog-content\" class=\"command-dialog\" role=\"dialog\" aria-modal=\"true\" aria-labelledby=\"{}\" aria-describedby=\"{}\" closedby=\"any\"><div data-slot=\"dialog-header\"><h2 data-slot=\"dialog-title\" id=\"{}\">Command palette</h2><p data-slot=\"dialog-description\" id=\"{}\">Search for a command to run.</p></div><div data-slot=\"command\">",
            widget_id(&c, "dialog-title"),
            widget_id(&c, "dialog-description"),
            widget_id(&c, "dialog-title"),
            widget_id(&c, "dialog-description"),
        )), "{html}");
        assert!(html.contains("placeholder=\"Type a command or search…\""));
        assert!(html.contains("<div data-slot=\"command-list\" role=\"listbox\" aria-label=\"Suggestions\"><div><div data-slot=\"command-group\" role=\"presentation\"><div cmdk-group-heading=\"\" aria-hidden=\"true\">Suggestions</div><div role=\"group\"><div data-slot=\"command-item\" role=\"option\" aria-selected=\"true\" data-selected=\"true\"><svg "), "{html}");
        assert!(html.contains("data-icon=\"calendar-days\""));
        assert!(html.contains("</div></div><div data-slot=\"command-separator\" role=\"separator\"></div><div data-slot=\"command-group\" role=\"presentation\"><div cmdk-group-heading=\"\" aria-hidden=\"true\">Settings</div><div role=\"group\"><div data-slot=\"command-item\" role=\"option\" aria-selected=\"false\" data-selected=\"false\"><svg "));
        assert!(html.contains("</svg>Settings<span data-slot=\"command-shortcut\">⌘S</span></div></div></div></div></div></div>"));
        // cmdk renders Empty only when nothing matches.
        assert!(!html.contains("command-empty"));
        assert!(html.ends_with(&format!(
            "<button type=\"button\" data-slot=\"dialog-close\" commandfor=\"{did}\" command=\"close\">{CROSS}<span>Close</span></button></dialog>"
        )));
        for bad in [
            "<details",
            "<summary",
            "style=",
            "onclick=",
            "<script",
            "popovertarget",
        ] {
            assert!(!html.contains(bad), "{bad}");
        }
        let mut none = stub("command", "Search");
        none.props
            .insert("empty".into(), "No results found.".into());
        assert!(render(&none).contains(
            "<div data-slot=\"command-empty\" role=\"presentation\">No results found.</div>"
        ));
    }

    #[test]
    fn chrome_command_dialog_is_glassy_and_pops_in() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"dialog-content\"].command-dialog {\n  gap: 0; padding: 0; overflow: hidden;"));
        assert!(css.contains("backdrop-filter: blur(40px) saturate(1.5);"));
        assert!(css.contains("[data-slot=\"dialog-content\"].command-dialog > [data-slot=\"dialog-header\"] {\n  position: absolute; width: 1px; height: 1px;"));
        assert!(
            css.contains("[data-slot=\"command-group\"] {\n  overflow: hidden; padding: 0.25rem;")
        );
        assert!(css.contains("[data-slot=\"command-shortcut\"] {\n  margin-inline-start: auto;"));
        assert!(
            css.contains("[data-slot=\"command-separator\"] {\n  margin: 0 -0.25rem; height: 1px;")
        );
        assert!(css
            .contains("[data-slot=\"command-empty\"] {\n  padding: 1.5rem 0; text-align: center;"));
        assert!(css.contains(".command-dialog [data-slot=\"command-input\"] { height: 3rem; }"));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"command\"] {\n  display: flex; flex-direction: column; overflow: hidden;\n  width: var(--cui-command-w, 100%); height: 12rem;"));
        assert!(css.contains("[data-slot=\"command-input-wrapper\"]"));
        assert!(css.contains("[data-slot=\"command\"] > label {"));
        assert!(css.contains("[data-slot=\"command-list\"]"));
        assert!(css.contains("[data-slot=\"command-item\"][data-selected=\"true\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("max-height: 20rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
