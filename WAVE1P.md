# Wave 1p — 9 dedicated kernel ports

A: logo-carousel, dynamic-island, image-zoom
B: aurora-background, border-beam, confetti
C: composed-chart, heatmap-chart, chart
sankey-chart and meteors remain stubs.

## B evidence (this tree)

- `aurora-background`: `src/cronus_ui_aurora_background.rs` — root `data-slot="aurora-background"`, `aria-hidden` layer, three empty `data-slot="aurora-blob"` divs, relative children wrapper with escaped `label_of`. CSS `@keyframes cui-aurora` uses `var(--cronus-primary)` / `var(--cronus-accent)`. `prefers-reduced-motion`: blob `animation: none`. No inline style, canvas, or JS.
- `border-beam`: `src/cronus_ui_border_beam.rs` — `data-slot="border-beam"` + `border-beam-layer` (`aria-hidden`) + `border-beam-content` wrapping the label. `@keyframes cui-border-beam` + `offset-path` on the layer `::after` in COMPONENT_CHROME (never a `<style>` tag). Reduced motion hides the layer. No inline style or JS.
- `confetti`: `src/cronus_ui_confetti.rs` — CSS-only stand-in for React canvas + rAF. Root `data-slot="confetti"` wraps the escaped label plus six empty `span data-slot="confetti-piece"`. Reduced motion hides pieces. No `<canvas>`, `<script>`, or `requestAnimationFrame`.
- Catalog `fx()` SURF box (`padding:0.75rem 1rem;position:relative;overflow:hidden`) is leftover for `meteors` only. Dedicated HTML does not contain it.
- `PORTED_FAMILIES` 148 → 151. `renderer_kind("meteors")` stays `Stub("fx")`; `renderer_kind("sankey-chart")` stays `Stub("chart")`.
