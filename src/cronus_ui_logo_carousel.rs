//! Dedicated LogoCarousel renderer. DOM matches React:
//! `<ul data-slot="logo-carousel">` plus one `logo-carousel-item`
//! `<li aria-label>` per column, each holding `div > span[aria-hidden] > span`
//! with the label initials (React `LogoMark` without icon).
//!
//! React pages the columns with a timer (`interval`, default 2200 ms) and
//! cross-fades each slot with Motion (0.5 s ease-out-quart, `y` 22 → 0,
//! scale 0.96, blur 10 px, `staggerDelay` per column). The kernel renders
//! every logo a column will ever show as a stacked `div` (`k-<page>` class)
//! and runs the same cross-fade as one CSS keyframe cycle per logo: cycle =
//! pages × interval, page `k` enters at `k × interval + column × stagger`.
//! Timing rides classes on the root (`n-<pages> i-<interval> s-<stagger>`),
//! so the keyframes exist for intervals 1600 / 2200 ms, staggers 0.07 / 0.12 s
//! and 2–8 pages (see `logo-carousel.css`); other values snap to the nearest.
//! Hover / focus-within pauses the cycle. No JS.
//!
//! `subtitle` / `title` items wrap the list in the docs hero lockup
//! (`cui-logo-hero`). Not interact flex-overflow `<div data-slot=…>`.

use crate::cronus_ui_kit::{attr, attr_num, esc, item, label_of};
use crate::parser::ComponentNode;

/// Logos come from `text` / `item` lines. The `label` names the list
/// (aria-label) and is only a logo when no other line exists.
/// Returns `(raw, escaped)` so initials are computed on unescaped text.
fn logos(comp: &ComponentNode) -> Vec<(String, String)> {
    let out: Vec<(String, String)> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
        .map(|i| (i.text.clone(), esc(&i.text)))
        .collect();
    if !out.is_empty() {
        return out;
    }
    // Same lookup order as `label_of`, but unescaped for the initials.
    let raw = ["label", "title", "text", "value"]
        .iter()
        .find_map(|k| item(comp, *k).filter(|t| !t.is_empty()))
        .map(str::to_string)
        .or_else(|| {
            comp.items
                .iter()
                .find(|i| !i.text.is_empty())
                .map(|i| i.text.clone())
        })
        .unwrap_or_else(|| comp.name.clone());
    vec![(raw, label_of(comp))]
}

/// React `MAX_COLUMNS`.
const MAX_COLUMNS: usize = 8;
/// Page counts with a keyframe cycle in the family CSS.
pub const PAGES: std::ops::RangeInclusive<usize> = 2..=8;
/// Intervals (ms) and staggers (hundredths of a second) with CSS timing.
pub const INTERVALS: [u32; 2] = [1600, 2200];
pub const STAGGERS: [u32; 2] = [7, 12];

/// React `getInitials`: first letter of up to two whitespace words, uppercased.
/// Works on the raw text; the result is escaped by the caller.
fn initials(label: &str) -> String {
    label
        .split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .flat_map(char::to_uppercase)
        .collect()
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

fn nearest(value: u32, options: &[u32]) -> u32 {
    options
        .iter()
        .copied()
        .min_by_key(|o| o.abs_diff(value))
        .unwrap_or(options[0])
}

fn mark(escaped_label: &str, raw: &str, class: &str) -> String {
    format!(
        "<div{class}><span aria-hidden=\"true\"><span>{}</span></span></div>",
        esc(&initials(raw)),
        class = if class.is_empty() {
            String::new()
        } else {
            format!(" class=\"{class}\"")
        },
    )
    .replace("{label}", escaped_label)
}

pub fn render(comp: &ComponentNode) -> String {
    let logos = logos(comp);
    let columns = attr_num::<f64>(comp, "columns")
        .map(|c| c.round().max(1.0) as usize)
        .unwrap_or(3)
        .min(MAX_COLUMNS)
        .min(logos.len())
        .max(1);
    // React: pages only when there are more logos than columns; the cycle
    // repeats after `logos / gcd(logos, columns)` pages.
    let pages = if logos.len() > columns {
        logos.len() / gcd(logos.len(), columns)
    } else {
        1
    };
    let cycling = pages > 1 && attr(comp, "motion") != Some("never");
    let pages = if cycling {
        pages.clamp(*PAGES.start(), *PAGES.end())
    } else {
        1
    };
    let items = (0..columns)
        .map(|c| {
            let (_, first) = &logos[c % logos.len()];
            let marks: String = (0..pages)
                .map(|k| {
                    let (raw, _) = &logos[(k * columns + c) % logos.len()];
                    let class = if cycling {
                        format!("k-{k}")
                    } else {
                        String::new()
                    };
                    mark(first, raw, &class)
                })
                .collect();
            let class = if cycling {
                format!(" class=\"c-{c}\"")
            } else {
                String::new()
            };
            format!(
                "<li data-slot=\"logo-carousel-item\"{class} aria-label=\"{first}\">{marks}</li>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    // `aria-label:` after an item line lands in that item's config (the
    // tokenizer has no newlines), so look there as well as in props.
    let aria = match attr(comp, "aria-label")
        .or_else(|| item(comp, "label"))
        .filter(|t| !t.is_empty())
    {
        Some(label) => esc(label),
        None => "Logo carousel".into(),
    };
    let class = if cycling {
        let interval = nearest(
            attr_num::<f64>(comp, "interval")
                .unwrap_or(2200.0)
                .max(800.0) as u32,
            &INTERVALS,
        );
        let stagger = nearest(
            (attr_num::<f64>(comp, "stagger").unwrap_or(0.07).max(0.0) * 100.0).round() as u32,
            &STAGGERS,
        );
        format!(" class=\"n-{pages} i-{interval} s-{stagger}\"")
    } else {
        String::new()
    };
    let list = format!(
        "<ul data-slot=\"logo-carousel\"{class} aria-label=\"{aria}\" aria-live=\"off\">{items}</ul>"
    );
    // Docs hero lockup: eyebrow + display heading above the list.
    let subtitle = item(comp, "subtitle").filter(|t| !t.is_empty());
    let title = item(comp, "title").filter(|t| !t.is_empty());
    if subtitle.is_none() && title.is_none() {
        return list;
    }
    let eyebrow = subtitle
        .map(|t| format!("<p>{}</p>", esc(t)))
        .unwrap_or_default();
    let heading = title
        .map(|t| format!("<h3>{}</h3>", esc(t)))
        .unwrap_or_default();
    format!("<section class=\"cui-logo-hero\"><div>{eyebrow}{heading}</div>{list}</section>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const INTERACT_ROW: &str = "display:flex;gap:0.75rem;overflow:auto";
    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem";
    const CSS: &str = include_str!("cronus_ui_css/logo-carousel.css");

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn logos(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("logo-carousel", items.first().copied().unwrap_or("Acme"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_stub(html: &str) {
        assert!(!html.contains("<section data-slot"));
        assert!(!html.contains("<div data-slot=\"logo-carousel\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("onscroll"));
        assert!(!html.contains("addEventListener"));
        assert!(!html.contains(INTERACT_ROW));
        assert!(!html.contains("min-width:12rem"));
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains("display("));
        assert!(!html.contains("carousel("));
        assert!(!html.contains("zinc-"));
    }

    /// React `LogoMark` without icon/node: motion wrapper `div` >
    /// `span[aria-hidden]` > `span` holding the initials; name on the `li`.
    fn li(label: &str, initials: &str) -> String {
        format!(
            "<li data-slot=\"logo-carousel-item\" aria-label=\"{label}\"><div><span aria-hidden=\"true\"><span>{initials}</span></span></div></li>"
        )
    }

    #[test]
    fn root_is_ul_with_items_not_interact_flex() {
        let html = render(&logos(&["Acme", "Stripe"]));
        assert!(html.starts_with("<ul data-slot=\"logo-carousel\""));
        assert!(html.contains("aria-live=\"off\""));
        assert!(html.contains("aria-label=\"Logo carousel\""));
        assert_eq!(html.matches("data-slot=\"logo-carousel-item\"").count(), 2);
        reject_stub(&html);
        assert_eq!(
            html,
            format!(
                "<ul data-slot=\"logo-carousel\" aria-label=\"Logo carousel\" aria-live=\"off\">{}{}</ul>",
                li("Acme", "A"),
                li("Stripe", "S")
            )
        );
    }

    /// React shows initials, never the full name, when a logo has no icon.
    #[test]
    fn items_show_initials_not_names() {
        let html = render(&logos(&["Acme", "Globex Corp", "big  blue  sky"]));
        assert!(html.contains(&li("Acme", "A")));
        assert!(html.contains(&li("Globex Corp", "GC")));
        assert!(html.contains(&li("big  blue  sky", "BB")));
        assert!(!html.contains(">Acme<"));
        assert!(!html.contains(">Globex Corp<"));
        reject_stub(&html);
    }

    /// Four logos over three columns page every interval: the cycle is four
    /// pages long, each column stacks the four logos it will show, timing
    /// classes ride the root (React defaults: 2200 ms, 0.07 s stagger).
    #[test]
    fn more_logos_than_columns_cycle_pages() {
        let html = render(&logos(&["Acme", "Globex", "Initech", "Umbrella"]));
        assert!(html.starts_with(
            "<ul data-slot=\"logo-carousel\" class=\"n-4 i-2200 s-7\" aria-label=\"Logo carousel\" aria-live=\"off\">"
        ));
        assert_eq!(html.matches("data-slot=\"logo-carousel-item\"").count(), 3);
        // Column 0 pages: Acme, Umbrella, Initech, Globex.
        assert!(html.contains(
            "<li data-slot=\"logo-carousel-item\" class=\"c-0\" aria-label=\"Acme\"><div class=\"k-0\"><span aria-hidden=\"true\"><span>A</span></span></div><div class=\"k-1\"><span aria-hidden=\"true\"><span>U</span></span></div><div class=\"k-2\"><span aria-hidden=\"true\"><span>I</span></span></div><div class=\"k-3\"><span aria-hidden=\"true\"><span>G</span></span></div></li>"
        ));
        assert!(html.contains("class=\"c-2\" aria-label=\"Initech\"><div class=\"k-0\"><span aria-hidden=\"true\"><span>I</span></span></div><div class=\"k-1\"><span aria-hidden=\"true\"><span>G</span></span></div>"));
        reject_stub(&html);
    }

    /// Docs "Hero lockup": eight logos, three columns, 1600 ms, 0.12 s
    /// stagger, wrapped in the eyebrow + heading lockup.
    #[test]
    fn docs_hero_lockup() {
        let mut c = logos(&[
            "Next.js",
            "BMW",
            "TypeScript",
            "Stripe",
            "Spiral",
            "Apple",
            "Tailwind CSS",
            "Vercel",
        ]);
        c.items.insert(0, extra("label", "Customer logos"));
        c.items
            .push(extra("subtitle", "The best teams are already here"));
        c.items.push(extra("title", "Join Cronus UI"));
        c.props.insert("columns".into(), "3".into());
        c.props.insert("interval".into(), "1600".into());
        c.props.insert("stagger".into(), "0.12".into());
        let html = render(&c);
        assert!(html.starts_with(
            "<section class=\"cui-logo-hero\"><div><p>The best teams are already here</p><h3>Join Cronus UI</h3></div><ul data-slot=\"logo-carousel\" class=\"n-8 i-1600 s-12\" aria-label=\"Customer logos\" aria-live=\"off\"><li data-slot=\"logo-carousel-item\" class=\"c-0\" aria-label=\"Next.js\"><div class=\"k-0\">"
        ));
        assert_eq!(html.matches("<li ").count(), 3);
        assert_eq!(html.matches("class=\"k-7\"").count(), 3);
        assert!(html.contains("<span>TC</span>"));
        assert!(html.ends_with("</ul></section>"));
        reject_stub(&html);
        c.props.insert("motion".into(), "never".into());
        let still = render(&c);
        assert!(!still.contains("k-1"));
        assert!(still.contains("<ul data-slot=\"logo-carousel\" aria-label=\"Customer logos\""));
    }

    #[test]
    fn extra_text_items_become_logos_label_names_list() {
        let mut c = stub("logo-carousel", "Acme");
        c.items.push(extra("text", "Stripe"));
        c.items.push(extra("text", "Vercel"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"logo-carousel-item\"").count(), 2);
        assert!(!html.contains("aria-label=\"Acme\"><div>"));
        assert!(html.contains(&li("Stripe", "S")));
        assert!(html.contains(&li("Vercel", "V")));
        assert!(html.starts_with("<ul data-slot=\"logo-carousel\" aria-label=\"Acme\""));
        reject_stub(&html);
    }

    /// Audit fixture shape: emitter writes `label "Logos"` (from aria-label),
    /// `text` per item and `aria-label:"Logos"`. React renders only Acme/Globex.
    #[test]
    fn fixture_aria_label_is_not_a_logo() {
        // Label differs from aria-label to prove the item-config value wins.
        let mut c = stub("logo-carousel", "Brands");
        c.items.push(extra("text", "Acme"));
        let mut globex = extra("text", "Globex");
        globex.config.insert("aria-label".into(), "Logos".into());
        c.items.push(globex);
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<ul data-slot=\"logo-carousel\" aria-label=\"Logos\" aria-live=\"off\">{}{}</ul>",
                li("Acme", "A"),
                li("Globex", "G")
            )
        );
        reject_stub(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("logo-carousel", "Acme"));
        assert!(html.starts_with("<ul data-slot=\"logo-carousel\""));
        assert_eq!(html.matches("data-slot=\"logo-carousel-item\"").count(), 1);
        assert!(html.contains(&li("Acme", "A")));
        assert!(html.contains("aria-live=\"off\""));
        reject_stub(&html);
    }

    /// Initials come from raw text (`<B>` → `<`), then get escaped once.
    #[test]
    fn label_is_escaped() {
        let html = render(&stub("logo-carousel", "A <B> & \"C\""));
        assert!(html.contains(&li("A &lt;B&gt; &amp; &quot;C&quot;", "A&lt;")));
        assert!(!html.contains("&amp;lt;"));
        reject_stub(&html);
    }

    #[test]
    fn skips_interact_flex_and_display_surf() {
        let c = logos(&["Acme", "Stripe"]);
        let html = render(&c);
        assert!(html.contains("data-slot=\"logo-carousel-item\""));
        assert!(html.starts_with("<ul"));
        assert!(!html.contains("<section"));
        assert!(!html.contains(DISPLAY_SURF));
        reject_stub(&html);
        assert_eq!(
            dedicated_fn_name("logo-carousel"),
            Some("cronus_ui_logo_carousel::render")
        );
        assert_eq!(
            renderer_kind("logo-carousel"),
            RendererKind::Dedicated("cronus_ui_logo_carousel::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&logos(&["Acme"]));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"logo-carousel-item\""));
            assert!(html.contains("aria-live=\"off\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        assert!(CSS.contains("[data-slot=\"logo-carousel\"]"));
        assert!(CSS.contains("[data-slot=\"logo-carousel-item\"]"));
        assert!(CSS.contains("list-style: none"));
        // Wave 1s geometry parity (React measured: ul 288x96, li 138x96, 60px initials).
        assert!(CSS.contains(
            "display: grid; grid-auto-flow: column; grid-auto-columns: minmax(0, 1fr);\n  gap: 0.75rem; width: var(--cui-logo-carousel-w, 100%);"
        ));
        assert!(CSS.contains("position: relative; box-sizing: border-box;"));
        assert!(CSS.contains("overflow: hidden; height: 5rem; padding: 0 0.75rem;"));
        assert!(CSS.contains("border: 1px solid transparent;\n  background: transparent;"));
        assert!(CSS
            .contains("color: color-mix(in oklab, var(--cronus-fg-secondary) 70%, transparent);"));
        assert!(CSS.contains(
            "font-size: 2.25rem; font-weight: 600; line-height: 1; color: currentColor;"
        ));
        assert!(CSS.contains(
            "[data-slot=\"logo-carousel-item\"] { height: 6rem; }\n  [data-slot=\"logo-carousel-item\"] > div > span { font-size: 3.75rem; }"
        ));
        assert!(!CSS.contains("repeat(auto-fit, minmax(6rem, 1fr))"));
        assert!(!CSS.contains("zinc-"));
        assert!(!CSS.contains("onclick"));
        assert!(!CSS.contains("setInterval"));
        assert!(!CSS.contains(INTERACT_ROW));
        assert!(!CSS.contains(DISPLAY_SURF));
    }

    /// One keyframe cycle per (pages, interval): plateau to `(T-0.5)/C`, exit
    /// to `T/C`, hidden until `(C-0.5)/C`, enter to 100 % — Motion's 0.5 s
    /// ease-out-quart cross-fade with `y` 22 / scale 0.96 / blur 10 px.
    #[test]
    fn chrome_cycle_keyframes_and_timing_classes() {
        for n in PAGES {
            for ms in INTERVALS {
                assert!(
                    CSS.contains(&format!("@keyframes cui-logo-cycle-{n}x{ms} {{")),
                    "{n}x{ms}"
                );
                assert!(CSS.contains(&format!(
                    "[data-slot=\"logo-carousel\"].n-{n}.i-{ms} > li > div {{ animation-name: cui-logo-cycle-{n}x{ms}; }}"
                )));
            }
            assert!(CSS.contains(&format!(
                "[data-slot=\"logo-carousel\"].n-{n} {{ --cui-logo-n: {n}; }}"
            )));
        }
        // 8 pages × 1.6 s: plateau ends 8.59375 %, exit done 12.5 %, re-enter from 96.09375 %.
        assert!(CSS.contains("@keyframes cui-logo-cycle-8x1600 {\n  0% { opacity: 1; transform: none; filter: blur(0); }\n  8.59375% { opacity: 1; transform: none; filter: blur(0); animation-timing-function: cubic-bezier(0.16, 1, 0.3, 1); }\n  12.5% { opacity: 0; transform: translateY(-22px) scale(0.96); filter: blur(10px); }\n  96.09375% { opacity: 0; transform: translateY(22px) scale(0.96); filter: blur(10px); animation-timing-function: cubic-bezier(0.16, 1, 0.3, 1); }\n  100% { opacity: 1; transform: none; filter: blur(0); }\n}"));
        assert!(CSS.contains("[data-slot=\"logo-carousel\"].i-1600 { --cui-logo-t: 1.6s; }"));
        assert!(CSS.contains("[data-slot=\"logo-carousel\"].s-12 { --cui-logo-stagger: 0.12s; }"));
        assert!(CSS.contains("animation-duration: calc(var(--cui-logo-n) * var(--cui-logo-t));"));
        assert!(CSS.contains("animation-delay: calc(var(--cui-logo-k) * var(--cui-logo-t) + var(--cui-logo-c) * var(--cui-logo-stagger) - var(--cui-logo-n) * var(--cui-logo-t) + 0.5s);"));
        assert!(CSS.contains("[data-slot=\"logo-carousel\"] > li > div.k-0 { animation-delay: calc(var(--cui-logo-c) * var(--cui-logo-stagger) + 0.5s); }"));
        assert!(CSS.contains("[data-slot=\"logo-carousel\"]:hover > li > div,\n[data-slot=\"logo-carousel\"]:focus-within > li > div { animation-play-state: paused; }"));
        assert!(
            CSS.contains("[data-slot=\"logo-carousel\"] > li > div:not(.k-0) { display: none; }")
        );
        assert!(CSS.contains(".cui-logo-hero {"));
    }
}
