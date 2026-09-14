# Wave 1q — 9 dedicated kernel ports

A: click-spark, glare-hover, magnetic
B: dot-pattern, flickering-grid, grid-pattern
C: highlighter, scramble-text, spinning-text
sankey-chart and meteors remain stubs.

## C — highlighter / scramble-text / spinning-text

Dedicated CONTRACT renderers. Zero JS. Token CSS in `COMPONENT_CHROME`.
`PORTED_FAMILIES` length 160 (157 + 3).

### highlighter
React idle: `<span data-slot="highlighter">` + aria-hidden mark + children. CSS `scaleX` draw-in.
Kernel: `<span data-slot="highlighter"><span data-slot="highlighter-mark" aria-hidden="true"></span>{label}</span>`
Mark is a `var(--cronus-primary)` / 25% underline bar. `@keyframes cui-highlighter` `scaleX(0→1)`.
`prefers-reduced-motion`: `animation: none` (fully drawn).
Skips catalog `fx()` SURF title box. No `<style>`, no inline `style=`.

### scramble-text
React idle/staticMode: sr-only real text + aria-hidden scrambled glyphs (`setTimeout` ticks).
Kernel idle is STATIC final text (no scramble ticks):
`<span data-slot="scramble-text"><span data-slot="scramble-text-label">{label}</span></span>`
Optional CSS `font-family: var(--cronus-font-mono, …)`. No `setTimeout`. No random glyphs.

### spinning-text
React: `<div data-slot="spinning-text">` + sr-only phrase + aria-hidden per-glyph orbit with inline rotate and a `<style>` keyframes tag.
Kernel does not emit `<style>` or inline `style=`. Rotate the whole orbit in CSS:
`<div data-slot="spinning-text"><span data-slot="spinning-text-label">{label}</span><span data-slot="spinning-text-orbit" aria-hidden="true">{label}</span></div>`
Label is visually hidden (clip / sr-only equivalent in chrome); orbit is decorative and CSS-rotates.
`@keyframes cui-spinning-text` rotate 360. `prefers-reduced-motion`: `animation: none`.
No per-glyph inline transforms.

### leftovers
`renderer_kind("meteors")` = `Stub("fx")`
`renderer_kind("sankey-chart")` = `Stub("chart")`

### cargo test
```
export CARGO_TARGET_DIR=/Users/pedrogbraz/projects/cooud/cronus-kernel/target
cd /Users/pedrogbraz/projects/cooud/.wt/kernel-1q-c
cargo test --offline cronus_ui_highlighter -- --test-threads=1
cargo test --offline cronus_ui_scramble_text -- --test-threads=1
cargo test --offline cronus_ui_spinning_text -- --test-threads=1
```
15 passed (5 highlighter + 5 scramble-text + 5 spinning-text).
