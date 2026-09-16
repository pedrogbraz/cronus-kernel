//! Dedicated AnimatedNumber renderer. DOM matches React:
//! `<span data-slot="animated-number"><span>{formatted}</span></span>` with the
//! settled value rendered statically (React mounts *at* the target, no JS
//! ticker here). CSS tabular-nums in COMPONENT_CHROME.
//! Not the catalog `fx()` title SURF box.
//! Wave 1t: `value:` written after an item attaches to that item's config
//! (the emitter writes `label "Count"` then `value:1234`), so it is read there
//! too; the root is `display: inline` like React's unstyled span.

use crate::cronus_ui_kit::{esc, item};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<span data-slot=\"animated-number\"><span>{}</span></span>",
        value_of(comp)
    )
}

fn value_of(comp: &ComponentNode) -> String {
    if let Some(t) = item(comp, "value") {
        if let Some(v) = display_value(t) {
            return v;
        }
    }
    if let Some(v) = comp.props.get("value") {
        if let Some(s) = display_value(v) {
            return s;
        }
    }
    for i in &comp.items {
        if let Some(v) = i.config.get("value").and_then(|v| display_value(v)) {
            return v;
        }
    }
    let label = item(comp, "label")
        .or_else(|| item(comp, "title"))
        .unwrap_or("");
    for i in &comp.items {
        if i.text.is_empty() {
            continue;
        }
        if matches!(i.item_type.as_str(), "label" | "title" | "value") {
            continue;
        }
        if i.text == label {
            continue;
        }
        if let Some(v) = display_value(&i.text) {
            return v;
        }
    }
    if let Some(s) = crate::cronus_ui_data::scalar() {
        if let Some(v) = display_value(&s) {
            return v;
        }
        return esc(&s);
    }
    if let Some(v) = display_value(label) {
        return v;
    }
    "0".into()
}

fn display_value(raw: &str) -> Option<String> {
    let t = raw.trim();
    if t.is_empty() || !t.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }
    if t.chars()
        .any(|c| matches!(c, ',' | '$' | '%' | '+') || c.is_ascii_alphabetic())
    {
        return Some(esc(t));
    }
    match t.parse::<f64>() {
        Ok(n) => Some(format_num(n)),
        Err(_) => Some(esc(t)),
    }
}

fn format_num(n: f64) -> String {
    if !n.is_finite() {
        return "0".into();
    }
    if (n - n.round()).abs() < 1e-9 {
        return group_int(n.round() as i64);
    }
    let neg = n < 0.0;
    let n = n.abs();
    let int_part = n.trunc() as i64;
    let frac = format!("{n:.2}");
    let decimals = frac.split_once('.').map(|(_, d)| d).unwrap_or("00");
    let grouped = group_int(int_part);
    if neg {
        format!("-{grouped}.{decimals}")
    } else {
        format!("{grouped}.{decimals}")
    }
}

fn group_int(n: i64) -> String {
    let neg = n < 0;
    let s = n.unsigned_abs().to_string();
    let bytes = s.as_bytes();
    let mut out = String::new();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*b as char);
    }
    if neg {
        format!("-{out}")
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<div"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("setTimeout"));
        assert!(!html.contains("requestAnimationFrame"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_is_span_with_formatted_number_not_fx_title_box() {
        let html = render(&stub("animated-number", "Demo"));
        assert_eq!(
            html,
            "<span data-slot=\"animated-number\"><span>0</span></span>"
        );
        reject_fx(&html);
    }

    /// Audit fixture shape: `label "Count"` then `value:1234` (item config).
    /// React shows the settled `1,234`, never `0`.
    #[test]
    fn value_from_label_item_config_is_settled_value() {
        let mut c = stub("animated-number", "Count");
        c.items[0].config.insert("value".into(), "1234".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<span data-slot=\"animated-number\"><span>1,234</span></span>"
        );
        reject_fx(&html);
    }

    #[test]
    fn value_item_is_formatted() {
        let mut c = stub("animated-number", "Users");
        c.items.push(extra("value", "1234"));
        let html = render(&c);
        assert_eq!(
            html,
            "<span data-slot=\"animated-number\"><span>1,234</span></span>"
        );
        reject_fx(&html);
    }

    #[test]
    fn extra_text_becomes_the_number() {
        let mut c = stub("animated-number", "Users");
        c.items.push(extra("text", "42"));
        let html = render(&c);
        assert!(html.contains("><span>42</span></span>"));
        reject_fx(&html);
    }

    #[test]
    fn value_from_props() {
        let mut c = stub("animated-number", "Users");
        c.props.insert("value".into(), "1200".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<span data-slot=\"animated-number\"><span>1,200</span></span>"
        );
        reject_fx(&html);
    }

    #[test]
    fn already_formatted_value_is_kept() {
        let mut c = stub("animated-number", "MRR");
        c.items.push(extra("value", "$12.4k"));
        let html = render(&c);
        assert_eq!(
            html,
            "<span data-slot=\"animated-number\"><span>$12.4k</span></span>"
        );
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let mut c = stub("animated-number", "Users");
        c.items.push(extra("value", "1 < 2 & \"3\""));
        let html = render(&c);
        assert_eq!(
            html,
            "<span data-slot=\"animated-number\"><span>1 &lt; 2 &amp; &quot;3&quot;</span></span>"
        );
        reject_fx(&html);
    }

    #[test]
    fn bound_scalar_when_no_value_sources() {
        use crate::binding::ResolvedData;
        crate::cronus_ui_data::with_binding("Lead", &ResolvedData::Count(12), || {
            let html = render(&stub("animated-number", "Leads"));
            assert_eq!(
                html,
                "<span data-slot=\"animated-number\"><span>12</span></span>"
            );
            reject_fx(&html);
        });
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("animated-number", "Demo"));
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::dedicated_fn_name("animated-number"),
            Some("cronus_ui_animated_number::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("animated-number", "Demo"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"animated-number\""));
        });
    }

    #[test]
    fn chrome_is_token_only_and_inline() {
        let css = crate::cronus_ui::component_chrome_css();
        let start = css.find("[data-slot=\"animated-number\"] {").unwrap();
        let block = &css[start..start + css[start..].find('}').unwrap()];
        assert!(block.contains("display: inline;"));
        assert!(!block.contains("inline-block"));
        assert!(block.contains("font-variant-numeric: tabular-nums"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
