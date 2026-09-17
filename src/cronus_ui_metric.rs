//! Dedicated Metric renderer. DOM matches React:
//! `<div data-slot="metric"><div data-slot="metric-label">…</div><div data-slot="metric-value">…</div></div>`
//! plus, for a `trend "+12.5%" tone:up` item (or `delta:` / `trend:` props),
//! React's `<span data-slot="metric-delta">` with the lucide TrendingUp /
//! TrendingDown / Minus glyph. React has no data attribute for the trend, so
//! it travels as class `t-up` | `t-down` | `t-neutral`.
//! Label from label/title; value from item type value / extra text / props.value
//! (bound scalar as a last resort). Not interact `metric()` (`<section style=SURF>`
//! + v-data `{ value }` interp).

use crate::cronus_ui_kit::{attr_nonempty, esc, item};
use crate::parser::ComponentNode;

const SVG_OPEN: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\">";
/// lucide `TrendingUp`.
const TRENDING_UP: &str =
    "<path d=\"M16 7h6v6\"></path><path d=\"m22 7-8.5 8.5-5-5L2 17\"></path></svg>";
/// lucide `TrendingDown`.
const TRENDING_DOWN: &str =
    "<path d=\"M16 17h6v-6\"></path><path d=\"m22 17-8.5-8.5-5 5L2 7\"></path></svg>";
/// lucide `Minus`.
const MINUS: &str = "<path d=\"M5 12h14\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let label = esc(label_raw(comp));
    let value = value_of(comp);
    let delta = delta_of(comp);
    metric_html(
        &label,
        &value,
        delta.as_ref().map(|(t, d)| (t.as_str(), d.as_str())),
    )
}

/// One React `Metric` (label, value, optional `(trend, delta text)`), every
/// argument already escaped. Shared with sparkline's stat cards.
pub fn metric_html(label: &str, value: &str, delta: Option<(&str, &str)>) -> String {
    let delta = delta
        .map(|(trend, text)| {
            let glyph = match trend {
                "up" => TRENDING_UP,
                "down" => TRENDING_DOWN,
                _ => MINUS,
            };
            format!(
                "<span data-slot=\"metric-delta\" class=\"t-{trend}\">{SVG_OPEN}{glyph}{text}</span>"
            )
        })
        .unwrap_or_default();
    format!(
        "<div data-slot=\"metric\"><div data-slot=\"metric-label\">{label}</div><div data-slot=\"metric-value\">{value}</div>{delta}</div>"
    )
}

/// `(trend, escaped delta text)` from a `trend` item (`tone:` = up | down |
/// neutral) or the `delta:` + `trend:` props. Unknown trends read as neutral.
fn delta_of(comp: &ComponentNode) -> Option<(String, String)> {
    let normalize = |t: Option<&str>| {
        match t.map(str::trim) {
            Some("up") | Some("success") | Some("positive") => "up",
            Some("down") | Some("danger") | Some("error") | Some("negative") => "down",
            _ => "neutral",
        }
        .to_string()
    };
    if let Some(i) = comp
        .items
        .iter()
        .find(|i| i.item_type == "trend" && !i.text.is_empty())
    {
        let tone = i
            .tone
            .as_deref()
            .or_else(|| i.config.get("trend").map(String::as_str))
            .or_else(|| attr_nonempty(comp, "trend"));
        return Some((normalize(tone), esc(&i.text)));
    }
    attr_nonempty(comp, "delta").map(|d| (normalize(attr_nonempty(comp, "trend")), esc(d)))
}

fn label_raw(comp: &ComponentNode) -> &str {
    for kind in ["label", "title"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() {
                return t;
            }
        }
    }
    &comp.name
}

fn value_of(comp: &ComponentNode) -> String {
    if let Some(t) = item(comp, "value") {
        if !t.is_empty() {
            return esc(t);
        }
    }
    if let Some(v) = comp.props.get("value") {
        if !v.is_empty() {
            return esc(v);
        }
    }
    // The tokenizer attaches `value:"1,240"` after `label "Users"` to the
    // label item's config, not to props.
    if let Some(v) = comp.items.iter().find_map(|i| i.config.get("value")) {
        if !v.is_empty() {
            return esc(v);
        }
    }
    let label = label_raw(comp);
    for i in &comp.items {
        if i.text.is_empty() {
            continue;
        }
        if matches!(i.item_type.as_str(), "label" | "title" | "trend") {
            continue;
        }
        if i.text == label {
            continue;
        }
        return esc(&i.text);
    }
    if let Some(s) = crate::cronus_ui_data::scalar() {
        return esc(&s);
    }
    "0".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(item_type: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: item_type.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<section"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("{ value }"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
    }

    #[test]
    fn root_is_div_with_label_and_value_not_section() {
        let html = render(&stub("metric", "Revenue"));
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"metric\""));
        assert!(html.contains("data-slot=\"metric-label\">Revenue</div>"));
        assert!(html.contains("data-slot=\"metric-value\">0</div>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"metric\"><div data-slot=\"metric-label\">Revenue</div><div data-slot=\"metric-value\">0</div></div>"
        );
    }

    #[test]
    fn label_from_title_item() {
        let mut c = stub("metric", "");
        c.items[0].item_type = "title".into();
        c.items[0].text = "Active users".into();
        let html = render(&c);
        assert!(html.contains("data-slot=\"metric-label\">Active users</div>"));
        reject_interact(&html);
    }

    #[test]
    fn value_from_value_item() {
        let mut c = stub("metric", "MRR");
        c.items.push(extra("value", "$12.4k"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"metric-label\">MRR</div>"));
        assert!(html.contains("data-slot=\"metric-value\">$12.4k</div>"));
        reject_interact(&html);
    }

    #[test]
    fn value_from_extra_text() {
        let mut c = stub("metric", "MRR");
        c.items.push(extra("text", "12,400"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"metric-label\">MRR</div>"));
        assert!(html.contains("data-slot=\"metric-value\">12,400</div>"));
        reject_interact(&html);
    }

    #[test]
    fn value_from_props() {
        let mut c = stub("metric", "Churn");
        c.props.insert("value".into(), "2.1%".into());
        let html = render(&c);
        assert!(html.contains("data-slot=\"metric-value\">2.1%</div>"));
        reject_interact(&html);
    }

    #[test]
    fn value_from_label_item_config() {
        // Wave 1t: emitter writes `label "Users"` + `value:"1,240"` → item config.
        let mut c = stub("metric", "Users");
        c.items[0].config.insert("value".into(), "1,240".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"metric\"><div data-slot=\"metric-label\">Users</div><div data-slot=\"metric-value\">1,240</div></div>"
        );
        let css = include_str!("cronus_ui_css/metric.css");
        assert!(css.contains(
            "font-size: 0.75rem; line-height: 1rem; font-weight: 500; text-transform: uppercase;"
        ));
        assert!(css.contains("font-size: 1.5rem; line-height: 2rem; font-weight: 600;"));
    }

    /// Docs "Stat tiles": a `trend` item with `tone:up|down|neutral` renders
    /// React's MetricDelta (glyph + text, trend as a class).
    #[test]
    fn trend_item_renders_metric_delta() {
        let mut c = stub("metric", "Revenue");
        c.items.push(extra("value", "$48,290"));
        let mut t = extra("trend", "+12.5%");
        t.tone = Some("up".into());
        c.items.push(t);
        let html = render(&c);
        assert!(html.contains("<div data-slot=\"metric-value\">$48,290</div><span data-slot=\"metric-delta\" class=\"t-up\"><svg "));
        assert!(html.contains("<path d=\"M16 7h6v6\"></path>"));
        assert!(html.ends_with("</svg>+12.5%</span></div>"));
        assert!(!html.contains("data-trend"));
        c.items[2].tone = Some("down".into());
        assert!(render(&c).contains("class=\"t-down\"><svg "));
        assert!(render(&c).contains("<path d=\"M16 17h6v-6\"></path>"));
        c.items[2].tone = Some("neutral".into());
        assert!(render(&c).contains("class=\"t-neutral\"><svg "));
        assert!(render(&c).contains("<path d=\"M5 12h14\"></path>"));
        let mut p = stub("metric", "Churn");
        p.props.insert("value".into(), "2.1%".into());
        p.props.insert("delta".into(), "-0.4%".into());
        p.props.insert("trend".into(), "down".into());
        assert!(render(&p).contains("class=\"t-down\""));
        reject_interact(&html);
        let css = include_str!("cronus_ui_css/metric.css");
        assert!(css.contains("[data-slot=\"metric-delta\"] {\n  display: inline-flex; align-items: center; gap: 0.25rem;\n  font-size: 0.75rem; line-height: 1rem; font-weight: 500; color: var(--cronus-fg-tertiary);\n}"));
        assert!(css
            .contains("[data-slot=\"metric-delta\"].t-up { color: var(--cronus-success-text); }"));
        assert!(
            css.contains("[data-slot=\"metric-delta\"] svg { width: 0.875rem; height: 0.875rem; }")
        );
    }

    #[test]
    fn value_item_wins_over_extra_and_props() {
        let mut c = stub("metric", "MRR");
        c.props.insert("value".into(), "props".into());
        c.items.push(extra("text", "extra"));
        c.items.push(extra("value", "item"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"metric-value\">item</div>"));
        assert!(!html.contains(">props</div>"));
        assert!(!html.contains(">extra</div>"));
    }

    #[test]
    fn bound_scalar_when_no_value_sources() {
        use crate::binding::ResolvedData;
        crate::cronus_ui_data::with_binding("Lead", &ResolvedData::Count(12), || {
            let html = render(&stub("metric", "Leads"));
            assert!(html.contains("data-slot=\"metric-value\">12</div>"));
            reject_interact(&html);
        });
    }

    #[test]
    fn skips_interact_section() {
        let html = render(&stub("metric", "Revenue"));
        assert!(html.starts_with("<div data-slot=\"metric\">"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("metric", "Revenue"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"metric-value\">0</div>"));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/metric.css");
        assert!(css.contains("[data-slot=\"metric\"]"));
        assert!(css.contains("[data-slot=\"metric-label\"]"));
        assert!(css.contains("[data-slot=\"metric-value\"]"));
        assert!(css.contains("flex-direction: column"));
        assert!(css.contains("gap: 0.25rem"));
        assert!(css.contains("text-transform: uppercase"));
        assert!(css.contains("letter-spacing: 0.05em"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        assert!(css.contains("var(--cronus-font-display"));
        assert!(css.contains("font-size: 1.5rem"));
        assert!(css.contains("font-variant-numeric: tabular-nums"));
        assert!(!css.contains("zinc-"));
    }
}
