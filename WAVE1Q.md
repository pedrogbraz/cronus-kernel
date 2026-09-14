# Wave 1q — 9 dedicated kernel ports

A: click-spark, glare-hover, magnetic
B: dot-pattern, flickering-grid, grid-pattern
C: highlighter, scramble-text, spinning-text
sankey-chart and meteors remain stubs.

## A — click-spark / glare-hover / magnetic

Dedicated CONTRACT renderers. Zero JS, zero `<script>`, zero `<style>` tags,
zero inline `style=`, zero canvas, zero rAF. Token CSS in `COMPONENT_CHROME`.
`PORTED_FAMILIES` length 160 (157 + 3).

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
