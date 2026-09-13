# Wave 1p — 9 dedicated kernel ports

A: logo-carousel, dynamic-island, image-zoom
B: aurora-background, border-beam, confetti
C: composed-chart, heatmap-chart, chart
sankey-chart and meteors remain stubs.

## A — logo-carousel / dynamic-island / image-zoom

Dedicated CONTRACT renderers. Zero JS. Token CSS in `COMPONENT_CHROME`.
`PORTED_FAMILIES` length 151 (148 + 3).

### logo-carousel
React idle: `<ul data-slot="logo-carousel">` + `<li data-slot="logo-carousel-item">` per text.
`aria-label` from label if present; `aria-live="off"`.
Skips interact `carousel("logo-carousel")` flex-overflow `<div style="...display:flex;gap:0.75rem;overflow:auto;">` SURF slides.
No `setInterval`. No inline `style=`.

### dynamic-island
React idle: `<div data-slot="dynamic-island">` + `dynamic-island-shell` + `role="tablist"` of `button data-slot="dynamic-island-trigger"`.
Static first view (first text). First trigger `aria-selected="true"`.
Skips catalog `fx()` SURF title box. No motion/framer.

### image-zoom
React idle: `<button type="button" data-slot="image-zoom" data-state="idle">` wrapping `image-zoom-content` + optional `image-zoom-indicator`.
URL/src texts → escaped `<img src alt>`. Else label text in content.
No JS zoom / onclick. CSS hover scale; `@media (prefers-reduced-motion: reduce) { transform: none }`.
Skips catalog `fx()` SURF title box.

### leftovers
`renderer_kind("meteors")` = `Stub("fx")`
`renderer_kind("sankey-chart")` = `Stub("chart")`

### cargo test
```
export CARGO_TARGET_DIR=/Users/pedrogbraz/projects/cooud/cronus-kernel/target
cd /Users/pedrogbraz/projects/cooud/.wt/kernel-1p-a
cargo test --offline -- --test-threads=1 cronus_ui_logo_carousel cronus_ui_dynamic_island cronus_ui_image_zoom
```
22 passed (7 logo-carousel + 7 dynamic-island + 8 image-zoom).
