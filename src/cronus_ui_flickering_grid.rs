//! Dedicated FlickeringGrid renderer. DOM mirrors React without the injected
//! `<style>`: `<div data-slot="flickering-grid">` + an aria-hidden grid of
//! `columns × rows` cells + a relative content `<div>` wrapping the label.
//! Only the root has a `data-slot`. React writes each cell's delay
//! (`(i·37) % 1600ms`) and duration (`1.4 + ((i·11) % 18) / 10 s`) as inline
//! custom properties; the kernel wraps each row in a `display: contents`
//! `<div>` so COMPONENT_CHROME derives the cell index from `nth-child`
//! (row × columns + column) and computes the same timings with `mod()` —
//! zero JS, no inline style. `@keyframes cui-flicker` lives in
//! COMPONENT_CHROME. Not the catalog `fx()` title SURF box.
//!
//! Props: `columns:` / `rows:` (React defaults 16 / 10, snapped to the sizes
//! the stylesheet knows), `card:true` (docs wrapper, class `card`) and
//! `heading:2xl` (`<p class="h-2xl">`).

use crate::cronus_ui_dot_pattern::HEADINGS;
use crate::cronus_ui_kit::{attr_num, choice, flag, label_of};
use crate::parser::ComponentNode;

/// Column counts the stylesheet has a `.c-N` rule for.
pub const COLUMNS: &[usize] = &[8, 12, 16, 20, 24, 32, 48];
/// Row counts the stylesheet has a `.r-N` rule for.
pub const ROWS: &[usize] = &[4, 5, 6, 8, 10, 12, 16, 20, 24, 32];

fn snap(value: Option<usize>, fallback: usize, allowed: &[usize]) -> usize {
    let wanted = value.unwrap_or(fallback);
    *allowed
        .iter()
        .min_by_key(|a| a.abs_diff(wanted))
        .unwrap_or(&fallback)
}

pub fn render(comp: &ComponentNode) -> String {
    let cols = snap(attr_num(comp, "columns"), 16, COLUMNS);
    let rows = snap(attr_num(comp, "rows"), 10, ROWS);
    let class = if flag(comp, "card") {
        " class=\"card\""
    } else {
        ""
    };
    let text = label_of(comp);
    let content = match choice(comp, "heading", HEADINGS) {
        Some(h) => format!("<p class=\"h-{h}\">{text}</p>"),
        None => text,
    };
    let row = format!("<div>{}</div>", "<span></span>".repeat(cols));
    let cells = row.repeat(rows);
    format!(
        "<div data-slot=\"flickering-grid\"{class}><div aria-hidden=\"true\" class=\"c-{cols} r-{rows}\">{cells}</div><div>{content}</div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("<style"));
        assert!(!html.contains("--flicker-delay"));
        assert!(!html.contains("@keyframes"));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("<svg"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("requestAnimationFrame"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
        assert_eq!(html.matches("data-slot=").count(), 1);
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("flickering-grid", "Flicker"));
        assert!(html.starts_with(
            "<div data-slot=\"flickering-grid\"><div aria-hidden=\"true\" class=\"c-16 r-10\"><div><span></span>"
        ));
        assert!(html.ends_with("</div><div>Flicker</div></div>"));
        // React: 16 × 10 = 160 cells, row-major.
        assert_eq!(html.matches("<span></span>").count(), 160);
        assert_eq!(html.matches("<div><span>").count(), 10);
        assert!(!html.contains("flickering-grid-field"));
        reject_fx(&html);
    }

    #[test]
    fn columns_and_rows_snap_to_stylesheet_sizes() {
        let mut c = stub("flickering-grid", "Flicker");
        c.props.insert("columns".into(), "9".into());
        c.props.insert("rows".into(), "3".into());
        let html = render(&c);
        assert!(html.contains("class=\"c-8 r-4\""));
        assert_eq!(html.matches("<span></span>").count(), 32);
        c.props.insert("columns".into(), "garbage".into());
        c.props.insert("rows".into(), "".into());
        assert!(render(&c).contains("class=\"c-16 r-10\""));
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("flickering-grid", "A <B> & \"C\""));
        assert!(html.ends_with("<div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"));
        reject_fx(&html);
    }

    /// Docs example: card wrapper + `<p className="font-display text-2xl text-fg">Signal</p>`.
    #[test]
    fn card_and_heading_render_the_docs_wrapper() {
        let mut c = stub("flickering-grid", "Signal");
        c.props.insert("card".into(), "true".into());
        c.props.insert("heading".into(), "2xl".into());
        let html = render(&c);
        assert!(
            html.starts_with("<div data-slot=\"flickering-grid\" class=\"card\"><div aria-hidden")
        );
        assert!(html.ends_with("<div><p class=\"h-2xl\">Signal</p></div></div>"));
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("flickering-grid", "Flicker");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("flickering-grid"),
            Some("cronus_ui_flickering_grid::render")
        );
        assert_eq!(
            renderer_kind("flickering-grid"),
            RendererKind::Dedicated("cronus_ui_flickering_grid::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("flickering-grid", "Flicker"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"flickering-grid\""));
        });
    }

    #[test]
    fn chrome_flickering_grid_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        let start = css.find("[data-slot=\"flickering-grid\"] {").unwrap();
        let block = &css[start..start + css[start..].find('}').unwrap()];
        assert!(block.contains("width: var(--cui-flickering-grid-w, 100%); min-height: 8rem;"));
        assert!(css.contains("[data-slot=\"flickering-grid\"] > [aria-hidden] {"));
        assert!(css.contains("grid-template-columns: repeat(var(--cui-cols, 16), minmax(0, 1fr));"));
        assert!(css.contains(
            "[data-slot=\"flickering-grid\"] > [aria-hidden] > div { display: contents; }"
        ));
        assert!(css.contains("[data-slot=\"flickering-grid\"] > div:last-child {"));
        assert!(!css.contains("[data-slot=\"flickering-grid-field\"]"));
        assert!(css.contains("@keyframes cui-flicker"));
        // React: delay (i·37) % 1600 ms, duration 1.4 + ((i·11) % 18) / 10 s, fill backwards.
        assert!(css.contains("animation: cui-flicker calc(1.4s + mod(var(--cui-i) * 11, 18) * 0.1s) ease infinite backwards;"));
        assert!(css.contains("animation-delay: calc(mod(var(--cui-i) * 37, 1600) * 1ms);"));
        assert!(css.contains("--cui-i: calc(var(--cui-r) * var(--cui-cols, 16) + var(--cui-c));"));
        for cols in COLUMNS {
            assert!(
                css.contains(&format!(".c-{cols} {{ --cui-cols: {cols}; }}")),
                "{cols}"
            );
        }
        for rows in ROWS {
            assert!(
                css.contains(&format!(".r-{rows} {{ --cui-rows: {rows}; }}")),
                "{rows}"
            );
        }
        assert!(css.contains("prefers-reduced-motion"));
        assert!(css.contains("opacity: 0.4"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(css.contains("[data-slot=\"flickering-grid\"].card {\n  display: grid; place-items: center; min-height: 14rem;"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("--flicker-delay"));
    }
}
