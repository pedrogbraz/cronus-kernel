//! Dedicated WordsPreloader renderer (Preloaders). DOM matches React
//! `WordsPreloader`: `<div data-slot="words-preloader" role="status"
//! aria-live="polite" aria-busy="true" aria-label="Loading">` (the `label`
//! item) > a slotless centred stage > the copy (`SlideUpText` of the current
//! line, `data-slot="slide-up-text"`) and, after the last line, the optional
//! end beat (`end:"Cronus"` text mark or `end-icon:sparkles` glyph). The
//! curved lip that flattens on exit is the panel's `::after`. Layout is the
//! style segment or `variant:`: `page` (fixed, full viewport — the default)
//! or `contained` (absolute in the slotless `words-preloader-stage` wrapper
//! the kernel adds, the docs' 36rem frame). Lines are the `item`s (React's
//! `words`; the product lines by default).
//!
//! React sequences the beats with timers; the kernel plays the same timeline
//! with CSS animations scheduled by absolute `data-delay` steps of 50 ms
//! (`words-preloader.css` maps 0–240 to `--cui-wp-delay`; longer timelines
//! clamp at 12 s). Every line sits in its own wrapper, invisible until its
//! start beat; the slide-up pieces carry their stagger offset by that start;
//! a nested wrapper hides the line after `revealDurationMs + hold`
//! (`first-delay:500`, `step-delay:500`, `last-hold:800`, ms like React); the
//! end beat scales in for 480 ms; then the panel slides up (800 ms, +120 ms)
//! while the lip flattens (700 ms, +180 ms). The last copy dissolves (320 ms
//! fade + blur) only before an end beat, like React. Reduced motion hides the
//! panel at once (React finishes and unmounts it). `onComplete` / replay need
//! JS.

use crate::cronus_ui_kit::{attr_nonempty, attr_num, choice, esc, item};
use crate::parser::{ComponentItemNode, ComponentNode};

const DEFAULT_WORDS: [&str; 2] = [
    "The innovation of interfaces.",
    "One system. The whole product follows.",
];
const SLIDE_UP_DURATION_MS: u64 = 500;
const SLIDE_UP_STAGGER_MS: u64 = 100;
const FIRST_DELAY_MS: u64 = 500;
const STEP_DELAY_MS: u64 = 500;
const LAST_HOLD_MS: u64 = 800;
const END_ENTER_MS: u64 = 480;
const STEP_MS: u64 = 50;
/// Last `data-delay` step with a CSS rule (12 s).
pub const MAX_STEP: u64 = 240;

fn reveal_duration_ms(line: &str) -> u64 {
    let units = line.split(' ').filter(|w| !w.is_empty()).count() as u64;
    SLIDE_UP_DURATION_MS + units.saturating_sub(1) * SLIDE_UP_STAGGER_MS
}

fn step(ms: u64) -> u64 {
    ms.div_ceil(STEP_MS).min(MAX_STEP)
}

/// The line's `SlideUpText` with every piece delay shifted by `offset` steps,
/// so the stagger runs from the line's start beat.
fn slide_up(comp: &ComponentNode, line: &str, offset: u64) -> String {
    const KEY: &str = "data-delay=\"";
    let html = slide_up_text(comp, line);
    let mut out = String::with_capacity(html.len());
    let mut rest = html.as_str();
    while let Some(p) = rest.find(KEY) {
        let start = p + KEY.len();
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('"')
            .map(|e| start + e)
            .unwrap_or(rest.len());
        let shifted = rest[start..end]
            .parse::<u64>()
            .map(|k| (k + offset).min(MAX_STEP))
            .unwrap_or(offset);
        out.push_str(&shifted.to_string());
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

fn slide_up_text(comp: &ComponentNode, line: &str) -> String {
    let node = ComponentNode {
        name: comp.name.clone(),
        layout: None,
        style: Some("slide-up-text".into()),
        items: vec![ComponentItemNode {
            item_type: "label".into(),
            text: line.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }],
        props: Default::default(),
        params: vec![],
        template: None,
        sections: vec![],
        state: vec![],
        tests: vec![],
        binding: None,
        span: Default::default(),
    };
    crate::cronus_ui_slide_up_text::render(&node)
}

pub fn render(comp: &ComponentNode) -> String {
    let contained = choice(comp, "variant", &["page", "contained"]) == Some("contained");
    let status = item(comp, "label")
        .filter(|l| !l.is_empty())
        .map(esc)
        .unwrap_or_else(|| "Loading".into());
    let words: Vec<String> = {
        let listed: Vec<String> = comp
            .items
            .iter()
            .filter(|i| (i.item_type == "item" || i.item_type == "text") && !i.text.is_empty())
            .map(|i| i.text.clone())
            .collect();
        if listed.is_empty() {
            DEFAULT_WORDS.iter().map(|w| w.to_string()).collect()
        } else {
            listed
        }
    };
    let ms = |key: &str, default: u64| {
        attr_num::<f64>(comp, key)
            .filter(|v| v.is_finite() && *v >= 0.0)
            .map(|v| v.round() as u64)
            .unwrap_or(default)
    };
    let first_delay = ms("first-delay", FIRST_DELAY_MS);
    let step_delay = ms("step-delay", STEP_DELAY_MS);
    let last_hold = ms("last-hold", LAST_HOLD_MS);
    let end = attr_nonempty(comp, "end-icon")
        .and_then(crate::cronus_ui_icons::svg)
        .or_else(|| attr_nonempty(comp, "end").map(|t| format!("<span>{}</span>", esc(t))));
    let n = words.len();
    let mut t = 0u64;
    let mut copy = String::new();
    for (i, line) in words.iter().enumerate() {
        let last = i + 1 == n;
        let hold = if end.is_some() || !last {
            if i == 0 {
                first_delay
            } else {
                step_delay
            }
        } else {
            last_hold
        };
        let duration = reveal_duration_ms(line) + hold;
        let text = slide_up(comp, line, step(t));
        let inner = if last && end.is_none() {
            format!("<div>{text}</div>")
        } else {
            let out = if last { " class=\"out\"" } else { "" };
            format!(
                "<div{out} data-delay=\"{}\">{text}</div>",
                step(t + duration)
            )
        };
        copy.push_str(&format!(
            "<div class=\"copy\" data-delay=\"{}\">{inner}</div>",
            step(t)
        ));
        t += duration;
    }
    let finish = match &end {
        Some(mark) => {
            copy.push_str(&format!(
                "<div class=\"end\" data-delay=\"{}\"><div>{mark}</div></div>",
                step(t)
            ));
            t + END_ENTER_MS + last_hold
        }
        None => t,
    };
    let panel = format!(
        "<div data-slot=\"words-preloader\" role=\"status\" aria-live=\"polite\" aria-busy=\"true\" aria-label=\"{status}\" class=\"{}\" data-delay=\"{}\"><div>{copy}</div></div>",
        if contained { "contained" } else { "page" },
        step(finish)
    );
    if contained {
        format!("<div class=\"words-preloader-stage\">{panel}</div>")
    } else {
        panel
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn line(t: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "item".into(),
            text: t.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn docs() -> ComponentNode {
        let mut c = stub("words-preloader", "Loading");
        c.style = Some("words-preloader+contained".into());
        c.props.insert("end".into(), "Cronus".into());
        c.items.push(line("The innovation of interfaces."));
        c.items.push(line("One system. The whole product follows."));
        c
    }

    #[test]
    fn docs_example_timeline_matches_react_beats() {
        let html = render(&docs());
        // Line 1: 4 units → 800 ms reveal + 500 ms first hold; hides at 1300 ms (26 steps).
        // Line 2: starts at 1300 ms, 6 units → 1000 ms + 500 ms step hold; dissolves at 2800 ms (56).
        // End beat at 2800 ms (56); finish = 2800 + 480 + 800 = 4080 ms (82).
        assert!(html.starts_with("<div class=\"words-preloader-stage\"><div data-slot=\"words-preloader\" role=\"status\" aria-live=\"polite\" aria-busy=\"true\" aria-label=\"Loading\" class=\"contained\" data-delay=\"82\"><div><div class=\"copy\" data-delay=\"0\"><div data-delay=\"26\"><span data-slot=\"slide-up-text\"><span>The innovation of interfaces.</span><span aria-hidden=\"true\"><span><span data-delay=\"0\">The</span></span><span><span data-delay=\"2\"> </span></span></span>"));
        assert!(html.contains("<div class=\"copy\" data-delay=\"26\"><div class=\"out\" data-delay=\"56\"><span data-slot=\"slide-up-text\"><span>One system. The whole product follows.</span><span aria-hidden=\"true\"><span><span data-delay=\"26\">One</span></span><span><span data-delay=\"28\"> </span></span></span>"));
        assert!(html.contains("<span data-delay=\"36\">follows.</span>"));
        assert!(html.contains("<div class=\"end\" data-delay=\"56\"><div><span>Cronus</span></div></div></div></div></div>"));
        assert_eq!(html.matches("data-slot=\"slide-up-text\"").count(), 2);
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn page_layout_without_end_holds_the_last_line() {
        let mut c = stub("words-preloader", "<b>\"Loading\"</b>");
        c.items.push(line("Hello world"));
        c.props.insert("last-hold".into(), "1000".into());
        c.props.insert("end-icon".into(), "sparkles".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"words-preloader\" role=\"status\" aria-live=\"polite\" aria-busy=\"true\" aria-label=\"&lt;b&gt;&quot;Loading&quot;&lt;/b&gt;\" class=\"page\""));
        assert!(html.contains("data-icon=\"sparkles\""));
        assert!(!html.contains("words-preloader-stage"));
        c.props.remove("end-icon");
        let html = render(&c);
        // 2 units → 600 ms reveal + 1000 ms last hold = 1600 ms (32 steps); no hide wrapper.
        assert!(html.contains("class=\"page\" data-delay=\"32\"><div><div class=\"copy\" data-delay=\"0\"><div><span data-slot=\"slide-up-text\">"));
        assert!(!html.contains("class=\"end\""));
    }

    #[test]
    fn default_words_and_step_clamp() {
        let html = render(&stub("words-preloader", "Loading"));
        assert!(html.contains("The innovation of interfaces."));
        assert!(html.contains("One system. The whole product follows."));
        assert_eq!(step(13_000), MAX_STEP);
        assert_eq!(step(1_300), 26);
        assert_eq!(
            reveal_duration_ms("One system. The whole product follows."),
            1000
        );
    }

    #[test]
    fn chrome_schedules_by_data_delay() {
        let css = include_str!("cronus_ui_css/words-preloader.css");
        assert!(css.contains("[data-delay=\"240\"] { --cui-wp-delay: 12000ms; }"));
        assert!(css.contains("animation-delay: calc(var(--cui-wp-delay, 0ms) + 120ms)"));
        assert!(css.contains("[data-slot=\"words-preloader\"] [data-slot=\"slide-up-text\"] > span[aria-hidden] > span > span { animation-delay: var(--cui-wp-delay, 0ms); }"));
        assert!(css.contains("cubic-bezier(0.22, 1, 0.36, 1)"));
        assert!(css.contains(".words-preloader-stage"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn registered_as_dedicated() {
        use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
        assert_eq!(
            dedicated_fn_name("words-preloader"),
            Some("cronus_ui_words_preloader::render")
        );
        assert_eq!(
            renderer_kind("words-preloader"),
            RendererKind::Dedicated("cronus_ui_words_preloader::render")
        );
        let c = stub("words-preloader", "Loading");
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), render(&c));
    }
}
