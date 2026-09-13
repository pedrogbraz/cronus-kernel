//! Dedicated UsageMeter renderer. DOM:
//! `<div data-slot="usage-meter">` plus `<div data-slot="usage-meter-fill">`
//! and optional `usage-meter-label` from label. Value from props/text number,
//! default 40. Not interact `progress()` (native `<progress>` / SURF bar).

use crate::cronus_ui_kit::{esc, item};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let pct = value_of(comp);
    let width = fmt_num(pct);
    let fill = format!(
        "<div data-slot=\"usage-meter-fill\" style=\"width:{width}%\"></div>"
    );
    match label_of(comp) {
        Some(label) => format!(
            "<div data-slot=\"usage-meter\"><span data-slot=\"usage-meter-label\">{label}</span>{fill}</div>"
        ),
        None => format!("<div data-slot=\"usage-meter\">{fill}</div>"),
    }
}

fn label_of(comp: &ComponentNode) -> Option<String> {
    for kind in ["label", "title"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() && parse_num(t).is_none() {
                return Some(esc(t));
            }
        }
    }
    None
}

fn value_of(comp: &ComponentNode) -> f64 {
    if let Some(v) = comp.props.get("value").and_then(|s| parse_num(s)) {
        return clamp(v);
    }
    for i in &comp.items {
        if let Some(v) = parse_num(&i.text) {
            return clamp(v);
        }
    }
    for i in &comp.items {
        if let Some(v) = i.config.get("value").and_then(|s| parse_num(s)) {
            return clamp(v);
        }
    }
    40.0
}

fn parse_num(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    s.parse().ok()
}

fn clamp(v: f64) -> f64 {
    v.clamp(0.0, 100.0)
}

fn fmt_num(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
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

    fn reject_interact(html: &str) {
        assert!(!html.contains("<progress"));
        assert!(!html.contains("data-slot=\"progress-control\""));
        assert!(!html.contains("<section"));
        assert!(!html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("{ value }"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("progress("));
    }

    fn assert_meter(html: &str, width: &str) {
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"usage-meter\""));
        assert!(html.contains("data-slot=\"usage-meter-fill\""));
        assert!(html.contains(&format!("style=\"width:{width}%\"")));
        assert!(!html.contains("<progress"));
        reject_interact(html);
    }

    #[test]
    fn root_is_div_with_fill_default_40() {
        let html = render(&stub("usage-meter", "Storage"));
        assert_meter(&html, "40");
        assert!(html.contains("data-slot=\"usage-meter-label\">Storage</span>"));
        assert_eq!(
            html,
            "<div data-slot=\"usage-meter\"><span data-slot=\"usage-meter-label\">Storage</span><div data-slot=\"usage-meter-fill\" style=\"width:40%\"></div></div>"
        );
    }

    #[test]
    fn value_from_props() {
        let mut c = stub("usage-meter", "Storage");
        c.props.insert("value".into(), "72".into());
        let html = render(&c);
        assert_meter(&html, "72");
        assert!(html.contains("data-slot=\"usage-meter-label\">Storage</span>"));
    }

    #[test]
    fn value_from_item_text_number() {
        let mut c = stub("usage-meter", "Storage");
        c.items.push(extra("value", "75"));
        let html = render(&c);
        assert_meter(&html, "75");
        assert!(html.contains("data-slot=\"usage-meter-label\">Storage</span>"));
    }

    #[test]
    fn value_from_item_config() {
        let mut c = stub("usage-meter", "Storage");
        c.items[0].config.insert("value".into(), "10".into());
        let html = render(&c);
        assert_meter(&html, "10");
    }

    #[test]
    fn numeric_label_is_value_not_label() {
        let html = render(&stub("usage-meter", "55"));
        assert_meter(&html, "55");
        assert!(!html.contains("data-slot=\"usage-meter-label\""));
        assert_eq!(
            html,
            "<div data-slot=\"usage-meter\"><div data-slot=\"usage-meter-fill\" style=\"width:55%\"></div></div>"
        );
    }

    #[test]
    fn skips_interact_html_progress() {
        let c = stub("usage-meter", "Storage");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("usage-meter", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<progress"));
        assert!(interact.contains("max=\"100\""));
        assert!(!interact.contains("data-slot=\"usage-meter-fill\""));
        assert!(html.contains("data-slot=\"usage-meter-fill\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("usage-meter", "Storage"));
            reject_interact(&html);
            assert!(html.contains("style=\"width:40%\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"usage-meter\"]"));
        assert!(css.contains("[data-slot=\"usage-meter-fill\"]"));
        assert!(css.contains("[data-slot=\"usage-meter-label\"]"));
        assert!(css.contains("height: 0.5rem"));
        assert!(css.contains("border-radius: 9999px"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("<progress"));
    }
}
