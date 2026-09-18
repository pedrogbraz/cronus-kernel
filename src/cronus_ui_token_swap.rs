//! Dedicated TokenSwap renderer (Skiper 22 Aave swap). DOM matches React:
//! `<div data-slot="token-swap">` (rounded-3xl, the swap's own near-black
//! chrome) > the *from* card (asset mark + name + balance, the `Use Max`
//! button, the amount field with its glyph overlay, the USD readout row —
//! or `Not Enough ETH` over the balance — and the chevron-down badge), the
//! *to* card (asset mark + name + `Receive AAVE`, the rolling amount) and
//! the `Clear` button.
//!
//! Zero JS in the renderer: the amount field is a real `input` (text +
//! `inputmode="decimal"`), Max is `data-slot="token-swap-max"` and Clear is
//! `data-slot="token-swap-clear"`. Page runtime fills max from `data-balance`
//! and clears the field. Author `disabled:` keeps the input and both buttons
//! inert (`disabled` + `data-disabled` on the root). The numbers come from
//! the `.cronus` and are formatted like `Intl.NumberFormat` (`en-US`, USD
//! with two decimals, receive with two to three). The ETH and AAVE marks are
//! the inline SVGs React ships. The `Use → Using` morph and the digit fade
//! are state changes and do not animate here.
//!
//! Inputs: `value:"1.5"` (amount, default empty → `0`), `balance:111.82`,
//! `usd-per-from:3445.86`, `to-per-from:10.87`, `from:"Ethereum"`
//! `from-symbol:ETH`, `to:"Aave"` `to-symbol:AAVE`.

use crate::cronus_ui_kit::{attr_nonempty, esc, flag};
use crate::parser::ComponentNode;

const DEFAULT_USD_PER_FROM: f64 = 3445.86;
const DEFAULT_TO_PER_FROM: f64 = 10.87;
const DEFAULT_BALANCE: f64 = 111.82;

const ETH_MARK: &str = "<svg viewBox=\"0 0 41 41\" aria-hidden=\"true\" data-icon=\"eth\"><circle cx=\"20.5171\" cy=\"20.4634\" r=\"20.1069\" fill=\"#6D7FF9\"></circle><path d=\"M20.5148 9.28857L20.3477 9.85672V26.3431L20.5148 26.51L28.1676 21.9864L20.5148 9.28857Z\" fill=\"#C6C6C6\"></path><path d=\"M20.5161 9.28857L12.8633 21.9864L20.5161 26.51V18.508V9.28857Z\" fill=\"white\"></path><path d=\"M20.5122 27.9573L20.418 28.0721V33.945L20.5122 34.2201L28.1695 23.436L20.5122 27.9573Z\" fill=\"#C6C6C6\"></path><path d=\"M20.5161 34.2201V27.9573L12.8633 23.436L20.5161 34.2201Z\" fill=\"white\"></path><path d=\"M20.5137 26.5093L28.1669 21.9855L20.5137 18.5068V26.5093Z\" fill=\"white\"></path><path d=\"M12.8633 21.9855L20.5165 26.5093V18.5068L12.8633 21.9855Z\" fill=\"#C6C6C6\"></path></svg>";
const EQUAL_ICON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-icon=\"equal\"><line x1=\"5\" x2=\"19\" y1=\"9\" y2=\"9\"/><line x1=\"5\" x2=\"19\" y1=\"15\" y2=\"15\"/></svg>";
const ARROW_DOWN_UP_ICON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-icon=\"arrow-down-up\"><path d=\"m3 16 4 4 4-4\"/><path d=\"M7 20V4\"/><path d=\"m21 8-4-4-4 4\"/><path d=\"M17 4v16\"/></svg>";

fn aave_mark(mask_id: &str) -> String {
    format!(
        "<svg viewBox=\"0 0 45 45\" aria-hidden=\"true\" data-icon=\"aave\"><mask id=\"{mask_id}\" maskUnits=\"userSpaceOnUse\" x=\"0\" y=\"0\" width=\"45\" height=\"45\"><path d=\"M44.6772 0.960938H0.677246V44.9609H44.6772V0.960938Z\" fill=\"white\"></path></mask><g mask=\"url(#{mask_id})\"><path d=\"M44.6772 22.9724C44.6772 10.8003 34.8378 0.960938 22.6887 0.960938C10.5397 0.960938 0.677246 10.8003 0.677246 22.9724C0.677246 35.1446 10.5166 44.984 22.6887 44.984C34.8609 44.984 44.6772 35.1215 44.6772 22.9724Z\" fill=\"#9391F7\"></path><path d=\"M18.5767 24.0349C20.4707 23.7347 21.741 21.9562 21.4408 20.0622C21.1405 18.1683 19.362 16.8979 17.4681 17.1982C15.5741 17.4984 14.3038 19.2769 14.604 21.1709C14.9274 23.0648 16.7059 24.3352 18.5767 24.0349ZM27.6308 24.0349C29.5248 23.7347 30.7951 21.9562 30.4948 20.0622C30.1946 18.1683 28.4161 16.8979 26.5221 17.1982C24.6282 17.4984 23.3578 19.2769 23.6581 21.1709C23.9584 23.0648 25.7368 24.3352 27.6308 24.0349Z\" fill=\"white\"></path><path d=\"M22.5513 6.34277C13.1508 6.34277 5.52881 14.1034 5.52881 23.6656H9.87107C9.87107 16.4824 15.5067 10.685 22.5282 10.685C29.5729 10.685 35.1855 16.5055 35.1855 23.6656H39.5277C39.5739 14.1034 31.9519 6.34277 22.5513 6.34277Z\" fill=\"white\"></path></g></svg>"
    )
}

fn num(comp: &ComponentNode, key: &str, default: f64) -> f64 {
    attr_nonempty(comp, key)
        .and_then(|v| v.trim().replace(',', "").parse::<f64>().ok())
        .filter(|n| n.is_finite() && *n >= 0.0)
        .unwrap_or(default)
}

/// `Intl.NumberFormat("en-US", { minimumFractionDigits: min,
/// maximumFractionDigits: max })`: grouped integer part, trailing zeros
/// trimmed down to `min` decimals.
pub(crate) fn format_fixed(value: f64, min: usize, max: usize) -> String {
    let s = format!("{value:.max$}");
    let (int, frac) = s.split_once('.').unwrap_or((&s, ""));
    let mut frac = frac.to_string();
    while frac.len() > min && frac.ends_with('0') {
        frac.pop();
    }
    let digits: Vec<char> = int.chars().collect();
    let mut grouped = String::new();
    for (i, ch) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(*ch);
    }
    if frac.is_empty() {
        grouped
    } else {
        format!("{grouped}.{frac}")
    }
}

/// React `sanitizeAmount`: digits and a single dot; a leading dot gets a 0.
fn sanitize(raw: &str) -> String {
    let mut out = String::new();
    let mut dot = false;
    for ch in raw.chars() {
        if ch.is_ascii_digit() {
            out.push(ch);
        } else if ch == '.' && !dot {
            dot = true;
            out.push('.');
        }
    }
    if out.starts_with('.') {
        out.insert(0, '0');
    }
    out
}

pub fn render(comp: &ComponentNode) -> String {
    let amount = attr_nonempty(comp, "value")
        .or_else(|| attr_nonempty(comp, "amount"))
        .map(sanitize)
        .unwrap_or_default();
    let parsed: f64 = if amount.is_empty() || amount == "." {
        0.0
    } else {
        amount.parse().unwrap_or(0.0)
    };
    let balance = num(comp, "balance", DEFAULT_BALANCE);
    let usd_per_from = num(comp, "usd-per-from", DEFAULT_USD_PER_FROM);
    let to_per_from = num(comp, "to-per-from", DEFAULT_TO_PER_FROM);
    let from_name = attr_nonempty(comp, "from")
        .map(esc)
        .unwrap_or_else(|| "Ethereum".into());
    let from_symbol = attr_nonempty(comp, "from-symbol")
        .map(esc)
        .unwrap_or_else(|| "ETH".into());
    let to_name = attr_nonempty(comp, "to")
        .map(esc)
        .unwrap_or_else(|| "Aave".into());
    let to_symbol = attr_nonempty(comp, "to-symbol")
        .map(esc)
        .unwrap_or_else(|| "AAVE".into());
    let using_max = parsed >= balance && parsed > 0.0;
    let insufficient = parsed > balance;
    let usd_text = format_fixed(parsed * usd_per_from, 2, 2);
    let receive = format_fixed(parsed * to_per_from, 2, 3);
    let prefix = if using_max { "Using" } else { "Use" };
    let display = if amount.is_empty() {
        "0".to_string()
    } else {
        esc(&amount)
    };
    let glyphs: String = display
        .chars()
        .map(|c| format!("<span>{c}</span>"))
        .collect();
    let usd_glyphs: String = usd_text
        .chars()
        .map(|c| format!("<span>{c}</span>"))
        .collect();
    let mask_id = crate::cronus_ui_kit::instance_id(comp, "token-swap-aave");
    let invalid = if insufficient {
        " aria-invalid=\"true\""
    } else {
        ""
    };
    let disabled = flag(comp, "disabled");
    let dis = if disabled { " disabled" } else { "" };
    let root_disabled = if disabled { " data-disabled" } else { "" };
    let balance_attr = format_fixed(balance, 0, 2).replace(',', "");
    let readout = if insufficient {
        format!("<p class=\"error\">Not Enough {from_symbol}</p>")
    } else {
        format!(
            "<div class=\"usd\"><span class=\"sr-only\">${usd_text}</span><div class=\"eq\">{EQUAL_ICON}</div><span class=\"morph\"><span aria-hidden=\"true\"><span>$</span>{usd_glyphs}</span></span>{ARROW_DOWN_UP_ICON}</div>"
        )
    };
    format!(
        "<div data-slot=\"token-swap\" data-balance=\"{balance_attr}\"{root_disabled}><div class=\"card from\"><div class=\"head\"><div class=\"asset\">{ETH_MARK}<div><h2>{from_name}</h2><p> {} {from_symbol}</p></div></div><button type=\"button\" data-slot=\"token-swap-max\" aria-label=\"{prefix} Max\"{dis}><span class=\"morph\"><span>{prefix} </span></span>Max</button></div><div class=\"rule\"></div><div class=\"amount\"><div class=\"field\"><input type=\"text\" inputmode=\"decimal\" placeholder=\"0\" value=\"{}\" aria-label=\"Amount\"{invalid}{dis}><div aria-hidden=\"true\">{glyphs}</div></div><div class=\"readout\" aria-live=\"polite\">{readout}</div></div><div class=\"chev\">{}</div></div><div class=\"card to\"><div class=\"head\"><div class=\"asset\">{}<div><h2>{to_name}</h2><p> Receive {to_symbol}</p></div></div><p class=\"receive\">{receive}</p></div></div><button type=\"button\" class=\"clear\" data-slot=\"token-swap-clear\"{dis}>Clear</button></div>",
        format_fixed(balance, 0, 2),
        esc(&amount),
        crate::cronus_ui_icons::svg_or_empty("chevron-down"),
        aave_mark(&mask_id)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::{reset_instance_ids, stub};

    fn swap() -> ComponentNode {
        reset_instance_ids();
        stub("token-swap", "Swap")
    }

    fn reject_js(html: &str) {
        for bad in [
            "<script", "style=", "onclick", "onmouse", "onchange", "<canvas",
        ] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn docs_aave_swap_idle_dom() {
        let html = render(&swap());
        assert!(html.starts_with("<div data-slot=\"token-swap\" data-balance=\"111.82\"><div class=\"card from\"><div class=\"head\"><div class=\"asset\"><svg viewBox=\"0 0 41 41\" aria-hidden=\"true\" data-icon=\"eth\">"));
        assert!(html.contains("<div><h2>Ethereum</h2><p> 111.82 ETH</p></div></div><button type=\"button\" data-slot=\"token-swap-max\" aria-label=\"Use Max\"><span class=\"morph\"><span>Use </span></span>Max</button></div><div class=\"rule\"></div><div class=\"amount\"><div class=\"field\"><input type=\"text\" inputmode=\"decimal\" placeholder=\"0\" value=\"\" aria-label=\"Amount\"><div aria-hidden=\"true\"><span>0</span></div></div><div class=\"readout\" aria-live=\"polite\"><div class=\"usd\"><span class=\"sr-only\">$0.00</span><div class=\"eq\"><svg"));
        assert!(html.contains("data-icon=\"equal\""));
        assert!(html.contains("<span class=\"morph\"><span aria-hidden=\"true\"><span>$</span><span>0</span><span>.</span><span>0</span><span>0</span></span></span><svg"));
        assert!(html.contains("data-icon=\"arrow-down-up\""));
        assert!(html.contains("<div class=\"chev\"><svg"));
        assert!(html.contains("<div class=\"card to\"><div class=\"head\"><div class=\"asset\"><svg viewBox=\"0 0 45 45\" aria-hidden=\"true\" data-icon=\"aave\"><mask id=\"cui-token-swap-token-swap-aave\""));
        assert!(html.contains("<div><h2>Aave</h2><p> Receive AAVE</p></div></div><p class=\"receive\">0.00</p></div></div><button type=\"button\" class=\"clear\" data-slot=\"token-swap-clear\">Clear</button></div>"));
        assert!(!html.contains(" disabled"));
        reject_js(&html);
    }

    #[test]
    fn typed_amount_converts_like_react() {
        let mut c = swap();
        c.props.insert("value".into(), "1.5".into());
        let html = render(&c);
        assert!(html.contains("value=\"1.5\" aria-label=\"Amount\"><div aria-hidden=\"true\"><span>1</span><span>.</span><span>5</span></div>"));
        assert!(html.contains("<span class=\"sr-only\">$5,168.79</span>"));
        assert!(html.contains("<p class=\"receive\">16.305</p>"));
        assert!(html.contains("aria-label=\"Use Max\""));
    }

    #[test]
    fn max_and_over_balance_states() {
        let mut c = swap();
        c.props.insert("value".into(), "111.82".into());
        let html = render(&c);
        assert!(html.contains(
            "aria-label=\"Using Max\"><span class=\"morph\"><span>Using </span></span>Max"
        ));
        assert!(html.contains("<span class=\"sr-only\">$385,316.07</span>"));
        assert!(!html.contains("aria-invalid"));
        c.props.insert("value".into(), "200".into());
        let html = render(&c);
        assert!(html.contains("aria-invalid=\"true\">"));
        assert!(html.contains("<div class=\"readout\" aria-live=\"polite\"><p class=\"error\">Not Enough ETH</p></div>"));
        assert!(!html.contains("class=\"usd\""));
        assert!(html.contains("<p class=\"receive\">2,174.00</p>"));
    }

    #[test]
    fn custom_assets_rates_and_sanitizing() {
        let mut c = swap();
        c.props.insert("from".into(), "<b>Bitcoin</b>".into());
        c.props.insert("from-symbol".into(), "BTC".into());
        c.props.insert("to".into(), "Uniswap \"v4\"".into());
        c.props.insert("to-symbol".into(), "UNI".into());
        c.props.insert("balance".into(), "2".into());
        c.props.insert("usd-per-from".into(), "60000".into());
        c.props.insert("to-per-from".into(), "8000".into());
        c.props.insert("value".into(), "1a.b2.5".into());
        let html = render(&c);
        assert!(html.contains("<h2>&lt;b&gt;Bitcoin&lt;/b&gt;</h2><p> 2 BTC</p>"));
        assert!(html.contains("<h2>Uniswap &quot;v4&quot;</h2><p> Receive UNI</p>"));
        assert!(html.contains("value=\"1.25\""));
        assert!(html.contains("<span class=\"sr-only\">$75,000.00</span>"));
        assert!(html.contains("<p class=\"receive\">10,000.00</p>"));
        reject_js(&html);
        assert_eq!(format_fixed(1234567.891, 2, 2), "1,234,567.89");
        assert_eq!(format_fixed(0.5, 2, 3), "0.50");
        assert_eq!(format_fixed(111.82, 0, 2), "111.82");
        assert_eq!(format_fixed(100.0, 0, 2), "100");
        assert_eq!(sanitize(".5"), "0.5");
    }

    #[test]
    fn author_disabled_keeps_amount_max_and_clear_inert() {
        let mut c = swap();
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("data-slot=\"token-swap\" data-balance=\"111.82\" data-disabled>"));
        assert!(html.contains("data-slot=\"token-swap-max\" aria-label=\"Use Max\" disabled>"));
        assert!(html.contains("aria-label=\"Amount\" disabled>"));
        assert!(html.contains("data-slot=\"token-swap-clear\" disabled>Clear</button>"));
    }

    #[test]
    fn chrome_is_token_only_or_component_painted() {
        let css = include_str!("cronus_ui_css/token-swap.css");
        assert!(css.contains("[data-slot=\"token-swap\"] {"));
        assert!(css.contains("--cui-swap-bg"));
        assert!(css.contains("max-width: 18.75rem"));
        assert!(css.contains("font-size: 45px"));
        assert!(css.contains("color: transparent; caret-color: transparent"));
        assert!(css.contains("[data-slot=\"token-swap\"] .error"));
        assert!(css.contains("var(--cronus-error-text)"));
    }

    #[test]
    fn registered_as_dedicated() {
        let c = swap();
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), {
            reset_instance_ids();
            render(&c)
        });
        assert_eq!(
            dedicated_fn_name("token-swap"),
            Some("cronus_ui_token_swap::render")
        );
        assert_eq!(
            renderer_kind("token-swap"),
            RendererKind::Dedicated("cronus_ui_token_swap::render")
        );
    }
}
