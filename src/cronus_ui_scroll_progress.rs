//! Dedicated ScrollProgress renderer. DOM:
//! `<div data-slot="scroll-progress">` plus `<div data-slot="scroll-progress-fill">`
//! (bar) or `scroll-progress-ring` (circle). Value from props/text number,
//! default 40. Not interact `progress()` (native `<progress>` / SURF bar).
//!
//! `target:"…"` renders the docs "Reading bar & ring" scaffold: a named,
//! keyboard-focusable scroll region (`<section tabindex="0">`) holding a
//! sticky bar and the `title` / `text` copy (`repeat:N` paragraphs), with a
//! `ring:<size>` circle beside it. React measures the container with JS; the
//! kernel drives fill, ring and percentage from a named CSS scroll timeline
//! (`scroll-timeline` on the region, `timeline-scope` on the wrapper) — zero JS.

use crate::cronus_ui_kit::{attr_nonempty, attr_num, esc, fmt_coord, item};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    if let Some(region) = attr_nonempty(comp, "target") {
        return scoped(comp, region);
    }
    let pct = value_of(comp);
    let now = fmt_num(pct);
    let label = aria_label(comp);
    if is_circle(comp) {
        circle(pct, &now, &label, 40.0, false)
    } else {
        bar(pct, &now, &label)
    }
}

/// Docs scaffold: `flex w-full max-w-md items-start gap-4` > named scroll
/// region (sticky bar + `space-y-4 p-4` copy) + optional ring.
fn scoped(comp: &ComponentNode, region: &str) -> String {
    let label = aria_label(comp);
    let mut copy = String::new();
    for i in &comp.items {
        if i.text.is_empty() {
            continue;
        }
        match i.item_type.as_str() {
            "title" => copy.push_str(&format!("<h4>{}</h4>", esc(&i.text))),
            "text" => {
                let n: usize = i
                    .config
                    .get("repeat")
                    .and_then(|v| v.trim().parse().ok())
                    .unwrap_or(1)
                    .clamp(1, 64);
                for _ in 0..n {
                    copy.push_str(&format!("<p>{}</p>", esc(&i.text)));
                }
            }
            _ => {}
        }
    }
    let ring = attr_num::<f64>(comp, "ring")
        .filter(|s| *s >= 8.0)
        .map(|size| circle(0.0, "0", &label, size, true))
        .unwrap_or_default();
    format!(
        "<div class=\"cui-scroll-progress-demo\"><section tabindex=\"0\" aria-label=\"{}\">{}<div>{copy}</div></section>{ring}</div>",
        esc(region),
        bar(0.0, "0", &label)
    )
}

/// React writes `style.width`; the kernel emits no inline style. The fill's
/// `data-value` is the value rounded to an integer 0..100; COMPONENT_CHROME
/// maps each `[data-value="N"]` to `--cui-progress-value: N` and the width
/// reads it (portable; typed `attr()` is Chromium-only).
fn bar(pct: f64, now: &str, label: &str) -> String {
    let step = if pct.is_finite() {
        pct.round().clamp(0.0, 100.0) as i64
    } else {
        0
    };
    format!(
        "<div data-slot=\"scroll-progress\" data-variant=\"bar\" role=\"progressbar\" aria-valuenow=\"{now}\" aria-valuemin=\"0\" aria-valuemax=\"100\" aria-label=\"{label}\"><div data-slot=\"scroll-progress-fill\" data-value=\"{step}\"></div></div>"
    )
}

/// React: `strokeWidth = max(2, round(size * 0.1))`, radius inset by half of
/// it, dash array = circumference. A scroll-driven ring also gets
/// `pathLength="100"` so one keyframe (`100 → 0`) fits every size; its
/// percentage is painted by a CSS counter, so the value span is empty.
fn circle(pct: f64, now: &str, label: &str, size: f64, scrolled: bool) -> String {
    let stroke = (size * 0.1).round().max(2.0);
    let radius = size / 2.0 - stroke / 2.0;
    let circ = 2.0 * std::f64::consts::PI * radius;
    let progress = (pct / 100.0).clamp(0.0, 1.0);
    let offset = circ * (1.0 - progress);
    let r = fmt_coord(radius);
    let sw = fmt_coord(stroke);
    let (c, off, path) = if scrolled {
        ("100".to_string(), "100".to_string(), " pathLength=\"100\"")
    } else {
        (fmt_coord(circ), fmt_coord(offset), "")
    };
    let s = fmt_coord(size);
    let half = fmt_coord(size / 2.0);
    let value = if scrolled {
        String::new()
    } else {
        format!("{now}%")
    };
    let class = if size != 40.0 {
        format!(" class=\"s-{s}\"")
    } else {
        String::new()
    };
    format!(
        "<div data-slot=\"scroll-progress\" data-variant=\"circle\"{class} role=\"progressbar\" aria-valuenow=\"{now}\" aria-valuemin=\"0\" aria-valuemax=\"100\" aria-label=\"{label}\"><svg width=\"{s}\" height=\"{s}\" viewBox=\"0 0 {s} {s}\" fill=\"none\" aria-hidden=\"true\"><circle cx=\"{half}\" cy=\"{half}\" r=\"{r}\" stroke-width=\"{sw}\" stroke=\"var(--cronus-border)\"></circle><circle data-slot=\"scroll-progress-ring\" cx=\"{half}\" cy=\"{half}\" r=\"{r}\" stroke-width=\"{sw}\" stroke-linecap=\"round\"{path} stroke-dasharray=\"{c}\" stroke-dashoffset=\"{off}\" stroke=\"var(--cronus-primary)\"></circle></svg><span data-slot=\"scroll-progress-value\">{value}</span></div>"
    )
}

fn aria_label(comp: &ComponentNode) -> String {
    if let Some(l) = attr_nonempty(comp, "aria-label") {
        return esc(l);
    }
    if let Some(t) = item(comp, "label") {
        if !t.is_empty() && parse_num(t).is_none() {
            return esc(t);
        }
    }
    if attr_nonempty(comp, "target").is_none() {
        if let Some(t) = item(comp, "title") {
            if !t.is_empty() && parse_num(t).is_none() {
                return esc(t);
            }
        }
    }
    "Scroll progress".into()
}

fn is_circle(comp: &ComponentNode) -> bool {
    if comp
        .props
        .get("variant")
        .map(|s| s == "circle")
        .unwrap_or(false)
    {
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
    fn fractional_value_rounds_fill_data_value() {
        let mut c = stub("scroll-progress", "Reading");
        c.props.insert("value".into(), "36.6".into());
        let html = render(&c);
        assert!(html.contains("aria-valuenow=\"36.6\""));
        assert!(
            html.contains("<div data-slot=\"scroll-progress-fill\" data-value=\"37\"></div>"),
            "{html}"
        );
        assert!(!html.contains("style="));
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
        assert!(css.contains(
            "[data-slot=\"scroll-progress\"] {\n  height: 0.25rem; width: var(--cui-scroll-progress-w, 100%); overflow: hidden;"
        ));
        assert!(!css.contains("attr(data-value type("));
        assert!(css.contains("[data-slot=\"scroll-progress-fill\"] {\n  height: 100%; background: var(--cronus-primary);\n  width: calc(var(--cui-progress-value, 0) * 1%);"));
        assert!(css.contains("[data-slot=\"scroll-progress-fill\"])[data-value=\"37\"] { --cui-progress-value: 37; }"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("<progress"));
    }

    /// Docs "Reading bar & ring": a named scroll region with a sticky bar,
    /// heading + 12 paragraphs, and a 48px ring — all scroll-driven by CSS.
    #[test]
    fn target_renders_scroll_region_bar_copy_and_ring() {
        let mut c = stub("scroll-progress", "Reading");
        c.props
            .insert("target".into(), "Release notes, scrollable".into());
        c.props.insert("ring".into(), "48".into());
        c.items.push(extra("title", "Release notes"));
        let mut p = extra("text", "Scroll this panel.");
        p.config.insert("repeat".into(), "12".into());
        c.items.push(p);
        let html = render(&c);
        assert!(html.starts_with(
            "<div class=\"cui-scroll-progress-demo\"><section tabindex=\"0\" aria-label=\"Release notes, scrollable\"><div data-slot=\"scroll-progress\" data-variant=\"bar\" role=\"progressbar\" aria-valuenow=\"0\""
        ));
        assert!(html.contains("<div data-slot=\"scroll-progress-fill\" data-value=\"0\"></div></div><div><h4>Release notes</h4><p>Scroll this panel.</p>"));
        assert_eq!(html.matches("<p>").count(), 12);
        assert!(html.contains("</div></section><div data-slot=\"scroll-progress\" data-variant=\"circle\" class=\"s-48\" role=\"progressbar\" aria-valuenow=\"0\""));
        // 48px ring: stroke 5, radius 21.5, drawn with pathLength 100.
        assert!(html.contains("<svg width=\"48\" height=\"48\" viewBox=\"0 0 48 48\""));
        assert!(html.contains("<circle cx=\"24\" cy=\"24\" r=\"21.5\" stroke-width=\"5\" stroke=\"var(--cronus-border)\">"));
        assert!(html.contains("stroke-linecap=\"round\" pathLength=\"100\" stroke-dasharray=\"100\" stroke-dashoffset=\"100\""));
        assert!(html.ends_with("<span data-slot=\"scroll-progress-value\"></span></div></div>"));
        assert!(html.contains("aria-label=\"Reading\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn scroll_driven_chrome_uses_named_timeline() {
        let css = include_str!("cronus_ui_css/scroll-progress.css");
        assert!(css.contains(".cui-scroll-progress-demo {"));
        assert!(css.contains("timeline-scope: --cui-scroll-progress;"));
        assert!(css.contains("scroll-timeline: --cui-scroll-progress block;"));
        assert!(css.contains("animation-timeline: --cui-scroll-progress;"));
        assert!(css.contains("@keyframes cui-scroll-progress-fill"));
        assert!(css.contains("@keyframes cui-scroll-progress-ring"));
        assert!(css.contains("@property --cui-scroll-pct"));
        assert!(css.contains("content: counter(cui-scroll-pct) \"%\";"));
        assert!(css.contains("[data-slot=\"scroll-progress\"][data-variant=\"circle\"][class^=\"s-\"] { width: auto; height: auto; }"));
    }
}
