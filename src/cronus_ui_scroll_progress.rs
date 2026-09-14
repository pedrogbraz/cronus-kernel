//! Dedicated ScrollProgress renderer. DOM:
//! `<div data-slot="scroll-progress">` plus `<div data-slot="scroll-progress-fill">`
//! (bar) or `scroll-progress-ring` (circle). Value from props/text number,
//! default 40. Not interact `progress()` (native `<progress>` / SURF bar).

use crate::cronus_ui_kit::{esc, fmt_coord, item};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let pct = value_of(comp);
    let now = fmt_num(pct);
    let label = aria_label(comp);
    if is_circle(comp) {
        circle(pct, &now, &label)
    } else {
        bar(&now, &label)
    }
}

/// Fill width comes from `data-value` via typed `attr()` in COMPONENT_CHROME
/// (React writes `style.width`; the kernel emits no inline style).
fn bar(now: &str, label: &str) -> String {
    format!(
        "<div data-slot=\"scroll-progress\" data-variant=\"bar\" role=\"progressbar\" aria-valuenow=\"{now}\" aria-valuemin=\"0\" aria-valuemax=\"100\" aria-label=\"{label}\"><div data-slot=\"scroll-progress-fill\" data-value=\"{now}\"></div></div>"
    )
}

fn circle(pct: f64, now: &str, label: &str) -> String {
    let size = 40.0;
    let stroke = 4.0;
    let radius = size / 2.0 - stroke / 2.0;
    let circ = 2.0 * std::f64::consts::PI * radius;
    let progress = (pct / 100.0).clamp(0.0, 1.0);
    let offset = circ * (1.0 - progress);
    let r = fmt_coord(radius);
    let sw = fmt_coord(stroke);
    let c = fmt_coord(circ);
    let off = fmt_coord(offset);
    format!(
        "<div data-slot=\"scroll-progress\" data-variant=\"circle\" role=\"progressbar\" aria-valuenow=\"{now}\" aria-valuemin=\"0\" aria-valuemax=\"100\" aria-label=\"{label}\"><svg width=\"40\" height=\"40\" viewBox=\"0 0 40 40\" fill=\"none\" aria-hidden=\"true\"><circle cx=\"20\" cy=\"20\" r=\"{r}\" stroke-width=\"{sw}\" stroke=\"var(--cronus-border)\"></circle><circle data-slot=\"scroll-progress-ring\" cx=\"20\" cy=\"20\" r=\"{r}\" stroke-width=\"{sw}\" stroke-linecap=\"round\" stroke-dasharray=\"{c}\" stroke-dashoffset=\"{off}\" stroke=\"var(--cronus-primary)\"></circle></svg><span data-slot=\"scroll-progress-value\">{now}%</span></div>"
    )
}

fn aria_label(comp: &ComponentNode) -> String {
    for kind in ["label", "title"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() && parse_num(t).is_none() {
                return esc(t);
            }
        }
    }
    "Scroll progress".into()
}

fn is_circle(comp: &ComponentNode) -> bool {
    if comp.props.get("variant").map(|s| s == "circle").unwrap_or(false) {
        return true;
    }
    comp.style
        .as_deref()
        .unwrap_or("")
        .split('+')
        .any(|part| part == "circle")
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
        assert!(!html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden"));
        assert!(!html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("{ value }"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("progress("));
    }

    fn assert_bar(html: &str, width: &str) {
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"scroll-progress\""));
        assert!(html.contains("data-variant=\"bar\""));
        assert!(html.contains("role=\"progressbar\""));
        assert!(html.contains(&format!("aria-valuenow=\"{width}\"")));
        assert!(html.contains("aria-valuemin=\"0\""));
        assert!(html.contains("aria-valuemax=\"100\""));
        assert!(html.contains("data-slot=\"scroll-progress-fill\""));
        assert!(html.contains(&format!("data-value=\"{width}\"")));
        assert!(!html.contains("style="));
        assert!(!html.contains("<progress"));
        reject_interact(html);
    }

    #[test]
    fn root_is_div_with_fill_default_40() {
        let html = render(&stub("scroll-progress", "Reading"));
        assert_bar(&html, "40");
        assert!(html.contains("aria-label=\"Reading\""));
        assert_eq!(
            html,
            "<div data-slot=\"scroll-progress\" data-variant=\"bar\" role=\"progressbar\" aria-valuenow=\"40\" aria-valuemin=\"0\" aria-valuemax=\"100\" aria-label=\"Reading\"><div data-slot=\"scroll-progress-fill\" data-value=\"40\"></div></div>"
        );
    }

    #[test]
    fn value_from_props() {
        let mut c = stub("scroll-progress", "Reading");
        c.props.insert("value".into(), "72".into());
        let html = render(&c);
        assert_bar(&html, "72");
        assert!(html.contains("aria-label=\"Reading\""));
    }

    #[test]
    fn value_from_item_text_number() {
        let mut c = stub("scroll-progress", "Reading");
        c.items.push(extra("value", "75"));
        let html = render(&c);
        assert_bar(&html, "75");
    }

    #[test]
    fn value_from_item_config() {
        let mut c = stub("scroll-progress", "Reading");
        c.items[0].config.insert("value".into(), "10".into());
        let html = render(&c);
        assert_bar(&html, "10");
    }

    #[test]
    fn numeric_label_is_value_not_label() {
        let html = render(&stub("scroll-progress", "55"));
        assert_bar(&html, "55");
        assert!(html.contains("aria-label=\"Scroll progress\""));
        assert!(!html.contains("aria-label=\"55\""));
    }

    #[test]
    fn circle_variant_emits_ring() {
        let mut c = stub("scroll-progress", "Reading");
        c.props.insert("variant".into(), "circle".into());
        let html = render(&c);
        assert!(html.contains("data-variant=\"circle\""));
        assert!(html.contains("data-slot=\"scroll-progress-ring\""));
        assert!(html.contains("data-slot=\"scroll-progress-value\">40%</span>"));
        assert!(html.contains("<svg"));
        assert!(!html.contains("data-slot=\"scroll-progress-fill\""));
        assert!(!html.contains("<progress"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_html_progress() {
        let c = stub("scroll-progress", "Reading");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("scroll-progress", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<progress"));
        assert!(interact.contains("max=\"100\""));
        assert!(!interact.contains("data-slot=\"scroll-progress-fill\""));
        assert!(html.contains("data-slot=\"scroll-progress-fill\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("scroll-progress", "Reading"));
            reject_interact(&html);
            assert!(html.contains("data-value=\"40\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"scroll-progress\"]"));
        assert!(css.contains("[data-slot=\"scroll-progress-fill\"]"));
        assert!(css.contains("height: 0.25rem"));
        // Fixture wrapper `w-72` (React bar is `w-full` inside it).
        assert!(css.contains("[data-slot=\"scroll-progress\"] {\n  height: 0.25rem; width: 18rem; overflow: hidden;"));
        assert!(css.contains("width: calc(attr(data-value type(<number>), 0) * 1%);"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("<progress"));
    }
}
