//! Dedicated NumberFlow renderer. DOM matches React `NumberFlow`:
//! `<span role="img" aria-label="{formatted}" data-slot="number-flow">` +
//! `<span aria-hidden="true" dir="ltr">` holding one digit roller per digit
//! (`<span data-digit="d">` clipping the 12-glyph strip `9 0 1 … 9 0`) and one
//! `<span>` per prefix, sign, separator, percent or suffix glyph.
//!
//! React mounts at the value; digits only roll on later `value` changes, which
//! need JS. The kernel renders that settled frame: the strip offset React
//! writes inline (`margin-top: -(d+1)em`) is `[data-digit="d"]` CSS.
//!
//! Formatting mirrors `Intl.NumberFormat` for the locales in [`locale_of`]
//! (other tags format as en-US): the shortest decimal representation of the
//! value, rounded half-expand, with React's fraction-digit defaults per `format`.

use crate::cronus_ui_kit::{attr, attr_num, esc};
use crate::parser::ComponentNode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Format {
    Number,
    Currency,
    Percentage,
    Decimal,
}

impl Format {
    fn parse(raw: &str) -> Self {
        match raw.trim() {
            "currency" => Self::Currency,
            "percentage" => Self::Percentage,
            "decimal" => Self::Decimal,
            _ => Self::Number,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Number => "number",
            Self::Currency => "currency",
            Self::Percentage => "percentage",
            Self::Decimal => "decimal",
        }
    }

    /// `(minimumFractionDigits, maximumFractionDigits)` before overrides.
    fn default_fraction(self) -> (usize, usize) {
        match self {
            Self::Number => (0, 3),
            Self::Percentage => (0, 0),
            Self::Currency | Self::Decimal => (2, 2),
        }
    }
}

/// Separators `Intl.NumberFormat` uses for a locale.
struct Locale {
    group: &'static str,
    decimal: &'static str,
    /// Integer digits needed before grouping starts, minus 3 (ICU minimumGroupingDigits).
    min_grouping: usize,
    /// Literal between the number and `%`.
    percent_gap: &'static str,
}

fn locale_of(tag: &str) -> Locale {
    match tag.trim() {
        "de-DE" => Locale {
            group: ".",
            decimal: ",",
            min_grouping: 1,
            percent_gap: "\u{a0}",
        },
        "es-ES" => Locale {
            group: ".",
            decimal: ",",
            min_grouping: 2,
            percent_gap: "\u{a0}",
        },
        "fr-FR" => Locale {
            group: "\u{202f}",
            decimal: ",",
            min_grouping: 1,
            percent_gap: "\u{202f}",
        },
        "it-IT" => Locale {
            group: ".",
            decimal: ",",
            min_grouping: 1,
            percent_gap: "",
        },
        // en-US, ja-JP, ko-KR, zh-CN, zh-TW and unknown tags.
        _ => Locale {
            group: ",",
            decimal: ".",
            min_grouping: 1,
            percent_gap: "",
        },
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Token {
    Digit(u8),
    Symbol(String),
}

struct Options<'a> {
    prefix: &'a str,
    suffix: &'a str,
    format: Format,
    locale: Locale,
    min_fraction: Option<usize>,
    max_fraction: Option<usize>,
}

/// Fraction digits after React's `resolveFormatOptions` and Intl's defaulting.
fn fraction_range(format: Format, min: Option<usize>, max: Option<usize>) -> (usize, usize) {
    let (dmin, dmax) = format.default_fraction();
    let (lo, hi) = match (min, max) {
        (Some(lo), Some(hi)) => (lo, hi.max(lo)),
        (Some(lo), None) => (lo, dmax.max(lo)),
        (None, Some(hi)) => (dmin.min(hi), hi),
        (None, None) => (dmin, dmax),
    };
    (lo.min(20), hi.min(20))
}

/// Integer and fraction digits of `|value| * 10^shift`, rounded half-expand.
fn digits(value: f64, shift: usize, lo: usize, hi: usize) -> (Vec<u8>, Vec<u8>) {
    // Rust's `{}` for f64 is the shortest round-trip decimal, never exponential.
    let repr = format!("{}", value.abs());
    let (int_s, frac_s) = repr.split_once('.').unwrap_or((repr.as_str(), ""));
    let mut int: Vec<u8> = int_s.bytes().map(|b| b - b'0').collect();
    let mut frac: Vec<u8> = frac_s.bytes().map(|b| b - b'0').collect();
    for _ in 0..shift {
        int.push(if frac.is_empty() { 0 } else { frac.remove(0) });
    }
    if frac.len() > hi {
        let round_up = frac[hi] >= 5;
        frac.truncate(hi);
        if round_up {
            let mut all: Vec<u8> = int.iter().chain(frac.iter()).copied().collect();
            let mut i = all.len();
            loop {
                if i == 0 {
                    all.insert(0, 1);
                    break;
                }
                i -= 1;
                if all[i] == 9 {
                    all[i] = 0;
                } else {
                    all[i] += 1;
                    break;
                }
            }
            let split = all.len() - frac.len();
            frac = all.split_off(split);
            int = all;
        }
    }
    while frac.len() > lo && frac.last() == Some(&0) {
        frac.pop();
    }
    while frac.len() < lo {
        frac.push(0);
    }
    while int.len() > 1 && int[0] == 0 {
        int.remove(0);
    }
    if int.is_empty() {
        int.push(0);
    }
    (int, frac)
}

fn tokens(value: f64, opts: &Options) -> Vec<Token> {
    let (lo, hi) = fraction_range(opts.format, opts.min_fraction, opts.max_fraction);
    let percent = opts.format == Format::Percentage;
    let (int, frac) = digits(value, if percent { 2 } else { 0 }, lo, hi);
    let mut out = Vec::new();
    if !opts.prefix.is_empty() {
        out.push(Token::Symbol(opts.prefix.to_string()));
    }
    if value.is_sign_negative() {
        out.push(Token::Symbol("-".into()));
    }
    let grouped = int.len() >= 3 + opts.locale.min_grouping;
    for (i, d) in int.iter().enumerate() {
        if grouped && i > 0 && (int.len() - i) % 3 == 0 {
            out.push(Token::Symbol(opts.locale.group.to_string()));
        }
        out.push(Token::Digit(*d));
    }
    if !frac.is_empty() {
        out.push(Token::Symbol(opts.locale.decimal.to_string()));
        out.extend(frac.iter().map(|d| Token::Digit(*d)));
    }
    if percent {
        if !opts.locale.percent_gap.is_empty() {
            out.push(Token::Symbol(opts.locale.percent_gap.to_string()));
        }
        out.push(Token::Symbol("%".into()));
    }
    if !opts.suffix.is_empty() {
        out.push(Token::Symbol(opts.suffix.to_string()));
    }
    out
}

fn display(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(|t| match t {
            Token::Digit(d) => d.to_string(),
            Token::Symbol(s) => s.clone(),
        })
        .collect()
}

/// Strip glyphs, top to bottom (React `DIGIT_STRIP`).
const STRIP: &str = "<span>9</span><span>0</span><span>1</span><span>2</span><span>3</span><span>4</span><span>5</span><span>6</span><span>7</span><span>8</span><span>9</span><span>0</span>";

fn numeric_attr(n: f64) -> String {
    if n.is_finite() {
        format!("{n}")
    } else {
        "0".into()
    }
}

pub fn render(comp: &ComponentNode) -> String {
    let value = attr_num::<f64>(comp, "value")
        .filter(|v| v.is_finite())
        .unwrap_or(0.0);
    let prefix = attr(comp, "prefix").unwrap_or("");
    let suffix = attr(comp, "suffix").unwrap_or("");
    let format = Format::parse(attr(comp, "format").unwrap_or(""));
    let locale_tag = attr(comp, "locale")
        .filter(|s| !s.is_empty())
        .unwrap_or("en-US");
    let opts = Options {
        prefix,
        suffix,
        format,
        locale: locale_of(locale_tag),
        min_fraction: attr_num(comp, "minimumFractionDigits"),
        max_fraction: attr_num(comp, "maximumFractionDigits"),
    };
    let tokens = tokens(value, &opts);
    let glyphs: String = tokens
        .iter()
        .map(|t| match t {
            Token::Digit(d) => format!("<span data-digit=\"{d}\"><span>{STRIP}</span></span>"),
            Token::Symbol(s) => format!("<span>{}</span>", esc(s)),
        })
        .collect();
    let class = match attr(comp, "size") {
        Some(s @ ("3xl" | "4xl" | "5xl")) => format!(" class=\"t-{s}\""),
        _ => String::new(),
    };
    let mut live = format!(
        " data-value=\"{}\" data-format=\"{}\"",
        numeric_attr(value),
        format.as_str()
    );
    if !prefix.is_empty() {
        live.push_str(&format!(" data-prefix=\"{}\"", esc(prefix)));
    }
    if !suffix.is_empty() {
        live.push_str(&format!(" data-suffix=\"{}\"", esc(suffix)));
    }
    live.push_str(&format!(" data-locale=\"{}\"", esc(locale_tag)));
    let flow = format!(
        "<span role=\"img\" aria-label=\"{}\" data-slot=\"number-flow\"{live}{class}><span aria-hidden=\"true\" dir=\"ltr\">{glyphs}</span></span>",
        esc(&display(&tokens))
    );
    // Docs demos: the number over a button; page runtime rolls on
    // `data-number-flow-action` (shuffle vs increment).
    match comp
        .items
        .iter()
        .find(|i| matches!(i.item_type.as_str(), "action" | "button") && !i.text.is_empty())
    {
        Some(action) => {
            let kind = if action.text.to_ascii_lowercase().contains("shuffle") {
                "shuffle"
            } else {
                "increment"
            };
            let btn = crate::cronus_ui_glass_card::action_button(action).replacen(
                "<button ",
                &format!("<button data-number-flow-action=\"{kind}\" "),
                1,
            );
            format!("<div class=\"cui-number-flow-demo\">{flow}{btn}</div>")
        }
        None => flow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn fmt(
        value: f64,
        format: &str,
        locale: &str,
        min: Option<usize>,
        max: Option<usize>,
    ) -> String {
        let opts = Options {
            prefix: "",
            suffix: "",
            format: Format::parse(format),
            locale: locale_of(locale),
            min_fraction: min,
            max_fraction: max,
        };
        display(&tokens(value, &opts))
    }

    #[test]
    fn formats_like_intl_en_us() {
        assert_eq!(fmt(1234.0, "number", "en-US", None, None), "1,234");
        assert_eq!(fmt(999.0, "number", "en-US", None, None), "999");
        assert_eq!(
            fmt(1234567.891, "number", "en-US", None, None),
            "1,234,567.891"
        );
        assert_eq!(fmt(1.23456, "number", "en-US", None, None), "1.235");
        assert_eq!(fmt(0.0, "number", "en-US", None, None), "0");
        assert_eq!(fmt(-0.5, "number", "en-US", None, None), "-0.5");
        assert_eq!(fmt(1234.5, "currency", "en-US", None, None), "1,234.50");
        assert_eq!(fmt(3.0, "decimal", "en-US", None, None), "3.00");
        // Half-expand on the shortest decimal (toFixed would give 1.00).
        assert_eq!(fmt(1.005, "decimal", "en-US", None, None), "1.01");
        assert_eq!(fmt(999.995, "currency", "en-US", None, None), "1,000.00");
        assert_eq!(fmt(0.42, "percentage", "en-US", None, None), "42%");
        assert_eq!(fmt(0.425, "percentage", "en-US", None, None), "43%");
        assert_eq!(fmt(0.1234, "percentage", "en-US", Some(1), None), "12.3%");
    }

    #[test]
    fn fraction_overrides_follow_react_and_intl() {
        assert_eq!(fmt(2.5, "number", "en-US", None, Some(0)), "3");
        assert_eq!(fmt(2.0, "number", "en-US", Some(2), None), "2.00");
        assert_eq!(fmt(2.123456, "number", "en-US", Some(4), None), "2.1235");
        // min > max: React raises max to min.
        assert_eq!(fmt(1.5, "number", "en-US", Some(3), Some(1)), "1.500");
        assert_eq!(fmt(9.999, "currency", "en-US", None, Some(1)), "10.0");
    }

    #[test]
    fn locales_use_their_separators() {
        assert_eq!(fmt(1234.5, "decimal", "de-DE", None, None), "1.234,50");
        assert_eq!(fmt(0.425, "percentage", "de-DE", None, None), "43\u{a0}%");
        assert_eq!(fmt(1234.0, "number", "es-ES", None, None), "1234");
        assert_eq!(fmt(12345.0, "number", "es-ES", None, None), "12.345");
        assert_eq!(fmt(1234.5, "number", "fr-FR", None, None), "1\u{202f}234,5");
        assert_eq!(fmt(0.5, "percentage", "it-IT", None, None), "50%");
        assert_eq!(fmt(1234.0, "number", "ja-JP", None, None), "1,234");
        assert_eq!(fmt(1234.0, "number", "xx-YY", None, None), "1,234");
    }

    #[test]
    fn root_is_img_span_with_rollers_and_symbols() {
        let mut c = stub("number-flow", "Revenue");
        c.props.insert("value".into(), "1234".into());
        let html = render(&c);
        let roller = |d: u8| format!("<span data-digit=\"{d}\"><span>{STRIP}</span></span>");
        assert_eq!(
            html,
            format!(
                "<span role=\"img\" aria-label=\"1,234\" data-slot=\"number-flow\" data-value=\"1234\" data-format=\"number\" data-locale=\"en-US\"><span aria-hidden=\"true\" dir=\"ltr\">{}<span>,</span>{}{}{}</span></span>",
                roller(1),
                roller(2),
                roller(3),
                roller(4)
            )
        );
        for bad in ["<script", "<style", " style=", "margin-top", "v-data="] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn prefix_suffix_are_symbols_and_escaped() {
        let mut c = stub("number-flow", "x");
        c.props.insert("value".into(), "-7".into());
        c.props.insert("prefix".into(), "<$".into());
        c.props.insert("suffix".into(), " & up".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"&lt;$-7 &amp; up\""), "{html}");
        assert!(
            html.contains("data-value=\"-7\" data-format=\"number\" data-prefix=\"&lt;$\" data-suffix=\" &amp; up\" data-locale=\"en-US\""),
            "{html}"
        );
        assert!(
            html.contains("dir=\"ltr\"><span>&lt;$</span><span>-</span><span data-digit=\"7\">"),
            "{html}"
        );
        assert!(
            html.ends_with("</span></span><span> &amp; up</span></span></span>"),
            "{html}"
        );
    }

    #[test]
    fn missing_or_non_finite_value_renders_zero() {
        for raw in [None, Some("NaN"), Some("inf"), Some("abc")] {
            let mut c = stub("number-flow", "x");
            if let Some(v) = raw {
                c.props.insert("value".into(), v.into());
            }
            let html = render(&c);
            assert!(html.contains("aria-label=\"0\""), "{raw:?}: {html}");
            assert_eq!(html.matches("data-digit=").count(), 1);
        }
    }

    #[test]
    fn emitted_fixture_parses_to_props() {
        let src = "app \"x\" { port 1 }\ncomponent NumberFlowCurrency layout:inline style:number-flow {\n  value:1234.5\n  prefix:\"$\"\n  format:\"currency\"\n  locale:\"en-US\"\n  minimumFractionDigits:2\n  label \"currency\"\n}\n";
        let comp = crate::parser::parse(src)
            .expect("parse")
            .into_iter()
            .find_map(|n| match n {
                crate::parser::AstNode::Component(c) => Some(c),
                _ => None,
            })
            .expect("component");
        let html = crate::cronus_ui_widgets::render(&comp).expect("family");
        assert!(html.contains("aria-label=\"$1,234.50\""), "{html}");
    }

    #[test]
    fn chrome_offsets_strip_per_digit() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"number-flow\"] {"));
        assert!(css.contains("font-variant-numeric: tabular-nums;"));
        for d in 0..10 {
            assert!(
                css.contains(&format!(
                    "[data-slot=\"number-flow\"] [data-digit=\"{d}\"] > span {{ margin-block-start: -{}em; }}",
                    d + 1
                )),
                "digit {d}"
            );
        }
        assert_eq!(
            crate::cli::stub_renderer_gate::dedicated_fn_name("number-flow"),
            Some("cronus_ui_number_flow::render")
        );
        assert!(css.contains(
            "[data-slot=\"number-flow\"] [data-digit] > span {\n  transition: margin-block-start 500ms var(--cronus-spring-soft, var(--ease-spring, cubic-bezier(0.34, 1.56, 0.64, 1)));\n}"
        ));
        assert!(
            css.contains("[data-slot=\"number-flow\"] [data-digit] > span { transition: none; }")
        );
    }

    /// Docs "Currency": `$19,348.43` at the 5xl display size over an outline
    /// "Update value" button (`data-number-flow-action=increment`).
    #[test]
    fn docs_currency_demo_with_disabled_button() {
        let mut c = stub("number-flow", "Revenue");
        c.props.insert("value".into(), "19348.43".into());
        c.props.insert("prefix".into(), "$".into());
        c.props.insert("format".into(), "currency".into());
        c.props.insert("size".into(), "5xl".into());
        c.items.push(crate::parser::ComponentItemNode {
            item_type: "action".into(),
            text: "Update value".into(),
            link: None,
            tone: None,
            config: [("variant", "outline"), ("size", "sm")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        });
        let html = render(&c);
        assert!(html.starts_with("<div class=\"cui-number-flow-demo\"><span role=\"img\" aria-label=\"$19,348.43\" data-slot=\"number-flow\" data-value=\"19348.43\" data-format=\"currency\" data-prefix=\"$\" data-locale=\"en-US\" class=\"t-5xl\"><span aria-hidden=\"true\" dir=\"ltr\"><span>$</span><span data-digit=\"1\">"));
        assert!(html.ends_with("</span></span><button data-number-flow-action=\"increment\" type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-size=\"sm\" class=\"cui-btn\">Update value</button></div>"));
        assert!(!html.contains("style="));
        assert!(!html.contains(" disabled"));
    }

    #[test]
    fn chrome_display_size_and_demo_stack() {
        let css = include_str!("cronus_ui_css/number-flow.css");
        assert!(css.contains("[data-slot=\"number-flow\"].t-5xl {"));
        assert!(css.contains("font-size: 3rem; letter-spacing: -0.03em;"));
        assert!(css.contains(".cui-number-flow-demo { display: flex; width: 100%; flex-direction: column; align-items: center; gap: 1.5rem; }"));
    }
}
