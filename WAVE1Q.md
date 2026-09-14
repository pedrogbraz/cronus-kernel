# Wave 1q — 9 dedicated kernel ports

A: click-spark, glare-hover, magnetic
B: dot-pattern, flickering-grid, grid-pattern
C: highlighter, scramble-text, spinning-text
sankey-chart and meteors remain stubs.

## B evidence (this tree)

- `dot-pattern`: `src/cronus_ui_dot_pattern.rs` — root `data-slot="dot-pattern"` + `dot-pattern-field` (`aria-hidden`) + `dot-pattern-content` wrapping escaped `label_of`. CSS `repeating-radial-gradient` dots use `var(--cronus-fg)` at low opacity. No SVG `useId`, no motion, no inline style, canvas, or JS.
- `flickering-grid`: `src/cronus_ui_flickering_grid.rs` — CSS-only stand-in for React's 160-cell grid with per-cell `--flicker-delay`. Root `data-slot="flickering-grid"` + `flickering-grid-field` (`aria-hidden`) + `flickering-grid-content`. Modest CSS grid tiles + `@keyframes cui-flicker` in COMPONENT_CHROME (never a `<style>` tag). `prefers-reduced-motion`: field `animation: none; opacity: 0.4`. No 160 cells, inline style, or JS.
- `grid-pattern`: `src/cronus_ui_grid_pattern.rs` — root `data-slot="grid-pattern"` + `grid-pattern-field` (`aria-hidden`) + `grid-pattern-content` wrapping the label. Two `repeating-linear-gradient`s (horizontal + vertical) use `var(--cronus-border)`. No SVG, motion, inline style, or JS.
- Catalog `fx()` SURF box (`padding:0.75rem 1rem;position:relative;overflow:hidden`) is leftover for `meteors` only. Dedicated HTML does not contain it.
- `PORTED_FAMILIES` 157 → 160. `renderer_kind("meteors")` stays `Stub("fx")`; `renderer_kind("sankey-chart")` stays `Stub("chart")`.
