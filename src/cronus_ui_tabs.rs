//! Dedicated Tabs renderer — mirrors React `Tabs` (Radix) and switches panels
//! with zero JS.
//!
//! DOM: `<div data-slot="tabs">` > `<div data-slot="tabs-list">` of one
//! `<label>` per tab (visually hidden `<input type="radio">` + the
//! `tabs-trigger` button) > one `<div data-slot="tabs-content">` per tab.
//! Items are `trigger, body` pairs (the audit fixture's `items`); `value:"…"`
//! selects the tab whose trigger matches, else the first.
//!
//! Selection is native: the radios share a `name`, so clicking a label checks
//! its radio and Arrow keys move between tabs (one Tab stop for the group).
//! `COMPONENT_CHROME` shows the panel whose index matches the checked radio via
//! `:has(:checked)` and paints the checked trigger as active. The trigger keeps
//! React's `<button>` slot but is `disabled`, `aria-hidden` and
//! `pointer-events: none`, so clicks land on the label and it never takes focus.
//!
//! Gaps vs Radix (no JS): the group is exposed as radios (`role="radiogroup"`),
//! not `tablist`/`tab` with a live `aria-selected`; there is no `data-state` on
//! triggers/panels (it could not follow the selection); inactive panels are
//! `display: none` with their content present instead of empty + `hidden`;
//! `radiogroup` Arrow keys wrap and select, Home/End are not handled; the
//! panel-switching CSS covers the first `MAX_TABS` tabs.

use crate::cronus_ui_kit::{attr, content_texts, esc, label_of, widget_id};
use crate::parser::ComponentNode;

/// Tabs beyond this index still render but have no panel-switching rule.
pub const MAX_TABS: usize = 12;

/// `(trigger, body)`: `tab` / `item` lines are tabs whose panel repeats the
/// trigger; otherwise the `text` lines pair up (audit fixture), a lone trailing
/// trigger reusing its own text; with neither, the label is the only tab.
fn pairs(comp: &ComponentNode) -> Vec<(String, String)> {
    let listed: Vec<(String, String)> = comp
        .items
        .iter()
        .filter(|i| (i.item_type == "tab" || i.item_type == "item") && !i.text.is_empty())
        .map(|i| (esc(&i.text), esc(&i.text)))
        .collect();
    if !listed.is_empty() {
        return listed;
    }
    let texts = content_texts(comp);
    if texts.is_empty() {
        let label = label_of(comp);
        return vec![(label.clone(), label)];
    }
    texts
        .chunks(2)
        .map(|c| {
            (
                c[0].clone(),
                c.get(1).cloned().unwrap_or_else(|| c[0].clone()),
            )
        })
        .collect()
}

pub fn render(comp: &ComponentNode) -> String {
    let tabs = pairs(comp);
    let selected = attr(comp, "value")
        .map(esc)
        .and_then(|v| tabs.iter().position(|(t, _)| *t == v))
        .unwrap_or(0);
    let name = widget_id(comp, "tabs");
    let triggers = tabs
        .iter()
        .enumerate()
        .map(|(i, (label, _))| {
            let checked = if i == selected { " checked" } else { "" };
            format!(
                "<label><input type=\"radio\" name=\"{name}\" id=\"{name}-t{i}\" value=\"{i}\" aria-label=\"{label}\" aria-controls=\"{name}-p{i}\"{checked}><button type=\"button\" data-slot=\"tabs-trigger\" tabindex=\"-1\" aria-hidden=\"true\" disabled>{label}</button></label>"
            )
        })
        .collect::<String>();
    let panels = tabs
        .iter()
        .enumerate()
        .map(|(i, (_, body))| {
            format!(
                "<div data-slot=\"tabs-content\" id=\"{name}-p{i}\" role=\"tabpanel\" aria-labelledby=\"{name}-t{i}\">{body}</div>"
            )
        })
        .collect::<String>();
    format!(
        "<div data-slot=\"tabs\" data-orientation=\"horizontal\"><div data-slot=\"tabs-list\" role=\"radiogroup\" aria-orientation=\"horizontal\">{triggers}</div>{panels}</div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
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

    /// Emitted audit fixture: `label "default"` + trigger/body text pairs.
    fn fixture() -> ComponentNode {
        let mut c = stub("tabs", "default");
        for t in [
            "Account", "Manage.", "Password", "Change.", "Team", "Invite.",
        ] {
            c.items.push(text(t));
        }
        c
    }

    fn reject_js(html: &str) {
        for bad in [
            "onclick", "<script", "style=", "v-data", "v-show", " hidden",
        ] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn fixture_matches_react_slots_with_radio_labels() {
        let html = render(&fixture());
        let n = widget_id(&fixture(), "tabs");
        assert!(html.starts_with(
            "<div data-slot=\"tabs\" data-orientation=\"horizontal\"><div data-slot=\"tabs-list\" role=\"radiogroup\" aria-orientation=\"horizontal\"><label>"
        ));
        assert_eq!(html.matches("data-slot=\"tabs-trigger\"").count(), 3);
        assert_eq!(html.matches("data-slot=\"tabs-content\"").count(), 3);
        assert_eq!(html.matches(&format!("name=\"{n}\"")).count(), 3);
        assert!(html.contains(&format!(
            "<input type=\"radio\" name=\"{n}\" id=\"{n}-t0\" value=\"0\" aria-label=\"Account\" aria-controls=\"{n}-p0\" checked><button type=\"button\" data-slot=\"tabs-trigger\" tabindex=\"-1\" aria-hidden=\"true\" disabled>Account</button>"
        )));
        assert!(html.contains(&format!(
            "<div data-slot=\"tabs-content\" id=\"{n}-p1\" role=\"tabpanel\" aria-labelledby=\"{n}-t1\">Change.</div>"
        )));
        assert_eq!(html.matches(" checked").count(), 1);
        assert!(!html.contains(">default<"));
        reject_js(&html);
    }

    #[test]
    fn value_selects_matching_tab() {
        let mut c = fixture();
        c.props.insert("value".into(), "Password".into());
        let html = render(&c);
        assert_eq!(html.matches(" checked").count(), 1);
        assert!(
            html.contains("aria-label=\"Password\" aria-controls=\"cui-tabs-tabs-p1\" checked>")
        );
        c.props.insert("value".into(), "Nope".into());
        assert!(render(&c).contains("aria-controls=\"cui-tabs-tabs-p0\" checked>"));
    }

    #[test]
    fn hostile_text_is_escaped() {
        let mut c = stub("tabs", "x");
        c.items.push(text("<b>\"a\"</b>"));
        c.props.insert("value".into(), "<b>\"a\"</b>".into());
        let html = render(&c);
        assert!(!html.contains("<b>"));
        assert!(html.contains("aria-label=\"&lt;b&gt;&quot;a&quot;&lt;/b&gt;\""));
        assert!(html.contains(" checked>"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || reject_js(&render(&fixture())));
    }

    #[test]
    fn chrome_switches_panels_with_checked_radio() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(
            css.contains("[data-slot=\"tabs\"] > [data-slot=\"tabs-content\"] {\n  display: none;")
        );
        for i in 1..=MAX_TABS {
            assert!(
                css.contains(&format!(
                    "[data-slot=\"tabs\"]:has(> [data-slot=\"tabs-list\"] > label:nth-child({i}) > input:checked) > [data-slot=\"tabs-content\"]:nth-child({})",
                    i + 1
                )),
                "tab {i}"
            );
        }
        assert!(css.contains(
            "[data-slot=\"tabs-list\"] > label:has(> input:checked) > [data-slot=\"tabs-trigger\"]"
        ));
        assert!(css.contains(
            "[data-slot=\"tabs-list\"] > label:has(> input:focus-visible) > [data-slot=\"tabs-trigger\"]"
        ));
        assert!(!css.contains("[data-slot=\"tabs\"] [role=\"tab\"]"));
    }
}
