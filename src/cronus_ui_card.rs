//! Dedicated Card renderer. DOM matches React: `<div data-slot="card">` with
//! `card-header` / `card-title` (from label/title), optional `card-description`
//! (from a `description:"…"` attribute, else the first extra text), an optional
//! `card-action` holding a `badge` item, `card-content` (a `value "$24"
//! unit:"/ month"` price row, else the remaining texts) and a `card-footer` of
//! `action` buttons (`variant:`, `width:full`). `max-width:xs|sm|md|lg|xl|2xl`
//! mirrors the docs' `max-w-*` class (React's card is `w-full`).
//! Not interact `card()` (`<section data-slot="card" style=…SURF…>` without card-title).

use crate::cronus_ui_kit::{attr_nonempty, esc};
use crate::parser::{ComponentItemNode, ComponentNode};

/// Item kinds that never count as body text.
const PART_KINDS: &[&str] = &["label", "title", "badge", "value", "action"];

pub fn render(comp: &ComponentNode) -> String {
    let title = esc(title_of(comp));
    let extras = extras_of(comp);
    let (desc, rest): (Option<&str>, &[&str]) = match attr_nonempty(comp, "description") {
        Some(d) => (Some(d), &extras[..]),
        None if extras.is_empty() => (None, &extras[..]),
        None => (Some(extras[0]), &extras[1..]),
    };
    let mut header =
        format!("<div data-slot=\"card-header\"><div data-slot=\"card-title\">{title}</div>");
    if let Some(desc) = desc {
        header.push_str(&format!(
            "<div data-slot=\"card-description\">{}</div>",
            esc(desc)
        ));
    }
    if let Some(badge) = comp
        .items
        .iter()
        .find(|i| i.item_type == "badge" && !i.text.is_empty())
    {
        let variant = badge
            .config
            .get("variant")
            .map(String::as_str)
            .unwrap_or("default");
        header.push_str(&format!(
            "<div data-slot=\"card-action\">{}</div>",
            crate::cronus_ui_badge::badge_html(&esc(&badge.text), variant)
        ));
    }
    header.push_str("</div>");
    let mut content = String::new();
    if let Some(value) = comp
        .items
        .iter()
        .find(|i| i.item_type == "value" && !i.text.is_empty())
    {
        let unit = value
            .config
            .get("unit")
            .filter(|u| !u.is_empty())
            .map(|u| format!("<span class=\"cui-card-unit\">{}</span>", esc(u)))
            .unwrap_or_default();
        content = format!(
            "<div data-slot=\"card-content\" class=\"cui-card-price\"><span>{}</span>{unit}</div>",
            esc(&value.text)
        );
    } else if !rest.is_empty() {
        let body = rest.iter().map(|t| esc(t)).collect::<Vec<_>>().join(" ");
        content = format!("<div data-slot=\"card-content\">{body}</div>");
    }
    let actions: String = comp
        .items
        .iter()
        .filter(|i| i.item_type == "action" && !i.text.is_empty())
        .map(|a| {
            let variant = a
                .config
                .get("variant")
                .map(String::as_str)
                .unwrap_or("primary");
            let button = crate::cronus_ui::button_html(
                &esc(&a.text),
                variant,
                "md",
                a.link.as_deref(),
                a.config
                    .get("disabled")
                    .is_some_and(|d| crate::cronus_ui_kit::truthy(d)),
                None,
            );
            if a.config.get("width").is_some_and(|w| w == "full") {
                button.replacen("class=\"cui-btn\"", "class=\"cui-btn w-full\"", 1)
            } else {
                button
            }
        })
        .collect();
    let footer = if actions.is_empty() {
        String::new()
    } else {
        format!("<div data-slot=\"card-footer\">{actions}</div>")
    };
    let class = max_width_class(comp)
        .map(|c| format!(" class=\"{c}\""))
        .unwrap_or_default();
    format!("<div data-slot=\"card\"{class}>{header}{content}{footer}</div>")
}

/// A `max-width:` prop in Tailwind's named steps (`xs` 20rem … `2xl` 42rem) as
/// a `mw-*` class; families that take the docs' `max-w-*` share this mapping
/// and ship the matching rules in their own stylesheet.
pub fn max_width_class(comp: &ComponentNode) -> Option<&'static str> {
    match attr_nonempty(comp, "max-width").map(str::trim) {
        Some("xs") => Some("mw-xs"),
        Some("sm") => Some("mw-sm"),
        Some("md") => Some("mw-md"),
        Some("lg") => Some("mw-lg"),
        Some("xl") => Some("mw-xl"),
        Some("2xl") => Some("mw-2xl"),
        _ => None,
    }
}

/// The docs' title-only card (`CardHeader > CardTitle` + `CardContent > p`),
/// used by masonry tiles. Both arguments are already escaped.
pub fn titled_card(title: &str, body: &str) -> String {
    format!(
        "<div data-slot=\"card\"><div data-slot=\"card-header\"><div data-slot=\"card-title\">{title}</div></div><div data-slot=\"card-content\"><p>{body}</p></div></div>"
    )
}

/// A card whose only child is `CardContent` (the docs' stat cards); `inner`
/// is already-rendered HTML and `class` an extra content class or empty.
pub fn content_card(class: &str, inner: &str) -> String {
    let class = if class.is_empty() {
        String::new()
    } else {
        format!(" class=\"{class}\"")
    };
    format!("<div data-slot=\"card\"><div data-slot=\"card-content\"{class}>{inner}</div></div>")
}

fn title_of(comp: &ComponentNode) -> &str {
    for kind in ["label", "title", "text", "value"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() {
                return t;
            }
        }
    }
    for i in &comp.items {
        if !i.text.is_empty() {
            return i.text.as_str();
        }
    }
    &comp.name
}

fn extras_of(comp: &ComponentNode) -> Vec<&str> {
    let title = title_of(comp);
    let mut skipped = false;
    let mut out = Vec::new();
    for i in &comp.items {
        if i.text.is_empty() {
            continue;
        }
        if !skipped && i.text == title {
            skipped = true;
            continue;
        }
        if PART_KINDS.contains(&i.item_type.as_str()) {
            continue;
        }
        out.push(i.text.as_str());
    }
    out
}

fn item<'a>(comp: &'a ComponentNode, kind: &str) -> Option<&'a str> {
    comp.items
        .iter()
        .find(|i| i.item_type == kind)
        .map(|i| i.text.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn stub(title: &str) -> ComponentNode {
        ComponentNode {
            name: "Card".into(),
            layout: Some("stack".into()),
            style: Some("card".into()),
            items: vec![ComponentItemNode {
                item_type: "label".into(),
                text: title.into(),
                link: None,
                tone: None,
                config: HashMap::new(),
            }],
            props: HashMap::new(),
            params: vec![],
            template: None,
            sections: vec![],
            state: vec![],
            tests: vec![],
            binding: None,
            span: Default::default(),
        }
    }

    fn extra(item_type: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: item_type.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        }
    }

    #[test]
    fn root_is_div_with_title_not_section() {
        let html = render(&stub("Overview"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("style="));
        assert_eq!(
            html,
            "<div data-slot=\"card\"><div data-slot=\"card-header\"><div data-slot=\"card-title\">Overview</div></div></div>"
        );
    }

    #[test]
    fn description_attribute_matches_react_header() {
        let mut c = stub("Subscription");
        c.items[0]
            .config
            .insert("description".into(), "Monthly plan".into());
        assert_eq!(
            render(&c),
            "<div data-slot=\"card\"><div data-slot=\"card-header\"><div data-slot=\"card-title\">Subscription</div><div data-slot=\"card-description\">Monthly plan</div></div></div>"
        );
        let mut p = stub("Subscription");
        p.props.insert("description".into(), "A & B".into());
        assert!(render(&p).contains("<div data-slot=\"card-description\">A &amp; B</div>"));
    }

    #[test]
    fn description_from_extra_text() {
        let mut c = stub("Overview");
        c.items.push(extra("text", "Weekly usage"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"card-description\">Weekly usage</div>"));
        assert!(!html.contains("data-slot=\"card-content\""));
    }

    #[test]
    fn content_when_more() {
        let mut c = stub("Overview");
        c.items.push(extra("text", "Weekly usage"));
        c.items.push(extra("item", "12,400 requests"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"card-description\">Weekly usage</div>"));
        assert!(html.contains("data-slot=\"card-content\">12,400 requests</div>"));
        let mut d = stub("Overview");
        d.items[0]
            .config
            .insert("description".into(), "Weekly usage".into());
        d.items.push(extra("text", "12,400 requests"));
        assert!(render(&d).contains("data-slot=\"card-content\">12,400 requests</div>"));
    }

    /// Docs "Anatomy": title + description, a primary badge in `card-action`,
    /// the `$24 / month` price row as content and a full-width primary button
    /// in the footer; `max-width:md` is the example's `max-w-md`.
    #[test]
    fn anatomy_renders_action_badge_price_row_and_footer_button() {
        let mut c = stub("Pro plan");
        c.items[0].item_type = "title".into();
        c.items.push(extra(
            "text",
            "Everything you need to ship a polished product.",
        ));
        let mut badge = extra("badge", "Popular");
        badge.config.insert("variant".into(), "primary".into());
        c.items.push(badge);
        let mut price = extra("value", "$24");
        price.config.insert("unit".into(), "/ month".into());
        c.items.push(price);
        let mut action = extra("action", "Upgrade now");
        action.config.insert("variant".into(), "primary".into());
        action.config.insert("width".into(), "full".into());
        c.items.push(action);
        c.props.insert("max-width".into(), "md".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"card\" class=\"mw-md\"><div data-slot=\"card-header\"><div data-slot=\"card-title\">Pro plan</div><div data-slot=\"card-description\">Everything you need to ship a polished product.</div><div data-slot=\"card-action\"><span data-slot=\"badge\" data-variant=\"primary\">Popular</span></div></div><div data-slot=\"card-content\" class=\"cui-card-price\"><span>$24</span><span class=\"cui-card-unit\">/ month</span></div><div data-slot=\"card-footer\"><button type=\"button\" data-slot=\"button\" data-variant=\"primary\" data-size=\"md\" class=\"cui-btn w-full\">Upgrade now</button></div></div>"
        );
        assert!(!html.contains("style="));
        let css = include_str!("cronus_ui_css/card.css");
        assert!(css.contains("[data-slot=\"card-action\"] {\n  grid-column-start: 2; grid-row: 1 / span 2; align-self: start; justify-self: end;\n}"));
        assert!(css.contains("[data-slot=\"card-footer\"] {\n  display: flex; min-width: 0; align-items: center; gap: 0.75rem;"));
        assert!(css.contains("[data-slot=\"card-content\"].cui-card-price > span:first-child {\n  font-family: var(--cronus-font-display, inherit); font-size: 1.875rem; line-height: 2.25rem; font-weight: 600; color: var(--cronus-fg);\n}"));
        assert!(css.contains("[data-slot=\"card\"].mw-md { max-width: 28rem; }"));
        assert!(css.contains("[data-slot=\"card-footer\"] > .w-full { width: 100%; }"));
    }

    #[test]
    fn shared_card_shells_for_other_families() {
        assert_eq!(
            titled_card("Checkout", "Card and boleto."),
            "<div data-slot=\"card\"><div data-slot=\"card-header\"><div data-slot=\"card-title\">Checkout</div></div><div data-slot=\"card-content\"><p>Card and boleto.</p></div></div>"
        );
        assert_eq!(
            content_card("cui-stat", "<b>x</b>"),
            "<div data-slot=\"card\"><div data-slot=\"card-content\" class=\"cui-stat\"><b>x</b></div></div>"
        );
        let mut c = stub("x");
        c.props.insert("max-width".into(), "2xl".into());
        assert_eq!(max_width_class(&c), Some("mw-2xl"));
        c.props.insert("max-width".into(), "huge".into());
        assert_eq!(max_width_class(&c), None);
    }

    #[test]
    fn skips_interact_section() {
        let html = render(&stub("Overview"));
        assert!(!html.contains("v-data="));
    }

    #[test]
    fn chrome_is_token_only_and_matches_react_geometry() {
        let css = include_str!("cronus_ui_css/card.css");
        assert!(css.contains("gap: 1.5rem"));
        assert!(css.contains("padding-top: 1.5rem"));
        // The surface, border and radius come from the shared card shell.
        assert!(crate::cronus_ui::component_chrome_css().contains("var(--cronus-surface-raised)"));
        assert!(css.contains("[data-slot=\"card-title\"] {\n  min-width: 0; overflow-wrap: break-word;\n  font-family: var(--cronus-font-display, inherit); font-weight: 600; line-height: 1;"));
        assert!(css.contains("grid-column: 1 / -1; font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(css.contains("@media (max-width: 639.98px) {\n  [data-slot=\"card-header\"], [data-slot=\"card-content\"] { padding-inline-start: 1rem; padding-inline-end: 1rem; }"));
        assert!(!css.contains("zinc-"));
    }
}
