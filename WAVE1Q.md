# Wave 1q — 9 dedicated kernel ports

A: click-spark, glare-hover, magnetic
B: dot-pattern, flickering-grid, grid-pattern
C: highlighter, scramble-text, spinning-text
sankey-chart and meteors remain stubs.

## A — click-spark / glare-hover / magnetic

Dedicated CONTRACT renderers. Zero JS, zero `<script>`, zero `<style>` tags,
zero inline `style=`, zero canvas, zero rAF. Token CSS in `COMPONENT_CHROME`.
`PORTED_FAMILIES` length 160 (157 + 3).

## B — dot-pattern / flickering-grid / grid-pattern

CSS field + content. No SVG useId, no 160 cells, leftover stub is meteors.

## C — highlighter / scramble-text / spinning-text

Span highlighter + static scramble + CSS-rotated spinning-text. leftover stub is meteors.

### click-spark
React idle: `<div data-slot="click-spark">` + relative children. Sparks only on pointer (JS).
Kernel idle: `<div data-slot="click-spark"><div data-slot="click-spark-content">{label}</div></div>`.
No spark spans at idle. No onclick / setTimeout. CSS: relative overflow.
Skips catalog `fx()` SURF title box.

### glare-hover
React: `<div data-slot="glare-hover">` + aria-hidden glare layer (opacity 0 until hover via `data-glare-hovered` + rAF CSS vars) + relative children.
Kernel: `<div data-slot="glare-hover"><div data-slot="glare-hover-layer" aria-hidden="true"></div><div data-slot="glare-hover-content">{label}</div></div>`.
Layer is a diagonal gradient using `var(--cronus-fg)`. Shown on `:hover` / `:focus-within`.
Default position `50% 50%` via CSS (no `--glare-x` JS).
`prefers-reduced-motion`: layer `{ display: none }`.
Skips catalog `fx()` SURF title box.

### magnetic
React: `<div data-slot="magnetic"><div data-slot="magnetic-target">{children}</div></div>` with rAF pointer follow.
Kernel: same slots. Target wraps escaped label. CSS hover translate `4px`.
`prefers-reduced-motion`: `transform: none`. No rAF, no `data-magnetic-active`.
Skips catalog `fx()` SURF title box.

### leftovers
`renderer_kind("meteors")` = `Stub("fx")`
`renderer_kind("sankey-chart")` = `Stub("chart")`

### cargo test
```
export CARGO_TARGET_DIR=/Users/pedrogbraz/projects/cooud/cronus-kernel/target
cd /Users/pedrogbraz/projects/cooud/.wt/kernel-1q-a
cargo test --offline -- --test-threads=1 cronus_ui_click_spark cronus_ui_glare_hover cronus_ui_magnetic
```
15 passed (5 click-spark + 5 glare-hover + 5 magnetic).

## Wave 1t — geometry parity (2026-09-14)

### magnetic
DOM unchanged. CSS root `inline-block; width: 18rem` (fixture `w-72`); target `display: block`
(React target is a plain div; it was inline-block, 26px wide).

| slot | React | Cronus |
|---|---|---|
| magnetic | 24,24 288×24 | 24,24 288×24 |
| magnetic-target | 24,24 288×24 | 24,24 288×24 |

### scramble-text
DOM mirrors React's settled render: `<span data-slot="scramble-text"><span>Decode</span><span aria-hidden="true">Decode</span></span>`
(the `scramble-text-label` slot is gone: React has no inner slot). First span is sr-only
(`[data-slot="scramble-text"] > span:first-child`); root `inline`, mono.

| slot | React | Cronus |
|---|---|---|
| scramble-text | text "Decode Decode" | text "Decode Decode" |

### spinning-text
DOM mirrors React: `<div data-slot="spinning-text"><span>SPIN</span><div aria-hidden="true">`
`<span data-angle="0deg">S</span><span data-angle="90deg">P</span>…</div></div>` (one span per Unicode
scalar; space → U+00A0). React's inline per-glyph `rotate(i/n·360deg) translateY(-48px)` is
`transform: rotate(attr(data-angle type(<angle>), 0deg)) translateY(-3rem)` in CSS (typed `attr()`,
Chromium 133+). Glyphs `absolute start-50% top-50% origin 0 0`, 12/16, 500, uppercase,
`letter-spacing: 0.1em`, `fg-secondary`; orbit `absolute inset-0` + `cui-spinning-text` 16s.

| slot | React | Cronus |
|---|---|---|
| spinning-text | 24,24 96×96 "SPIN S P I N" | 24,24 96×96 "SPIN S P I N" |

Screenshot: same glyph placement. Divergence: graphemes vs chars for combined emoji.
