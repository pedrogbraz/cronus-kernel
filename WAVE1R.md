# Wave 1r — last real dedicated ports (7)

A: gradient-border, light-rays, orbit
B: progressive-blur, retro-grid, ripple
C: motion-presets
sankey-chart and meteors remain stubs.

## B — progressive-blur / retro-grid / ripple

Dedicated CONTRACT renderers. Zero JS, zero `<script>`, zero `<style>` tags,
zero inline `style=`, zero canvas, zero rAF. Token CSS in `COMPONENT_CHROME`.
`PORTED_FAMILIES` length 169 (166 + 3).

### progressive-blur
React idle: overlay-only `<div data-slot="progressive-blur" aria-hidden="true">`
with three stacked backdrop-blur layers and inline mask styles.
Kernel: host + overlay + content so audit has visible text:
`<div data-slot="progressive-blur-host"><div data-slot="progressive-blur" aria-hidden="true">` + 3
`progressive-blur-layer` + `progressive-blur-content` wrapping escaped label.
Masks live in CSS (`backdrop-filter` blur 1px/4px/12px + `mask-image` linear-gradient).
`prefers-reduced-motion`: overlay `{ display: none }` (React `motion-reduce:hidden`).
Skips catalog `fx()` SURF title box.

### retro-grid
React: `<div data-slot="retro-grid">` + aria-hidden perspective floor + children.
Inline perspective/transform and `<style>` keyframes.
Kernel: `<div data-slot="retro-grid"><div data-slot="retro-grid-field" aria-hidden="true"></div><div data-slot="retro-grid-content">{label}</div></div>`.
CSS: perspective, rotateX floor, repeating grid with `var(--cronus-border)`,
`@keyframes cui-retro-grid` translateY.
`prefers-reduced-motion`: animation none (floor still visible).
Skips catalog `fx()` SURF title box.

### ripple
React: `<div data-slot="ripple">` + 8 aria-hidden rings with inline `--ripple-delay` + children.
Kernel: 4 empty `span data-slot="ripple-ring"` (nth-of-type animation-delay in CSS).
`<div data-slot="ripple"><div data-slot="ripple-field" aria-hidden="true">` + 4 rings +
`ripple-content` wrapping escaped label.
`@keyframes cui-ripple` scale+fade.
`prefers-reduced-motion`: field `{ display: none }`.
Skips catalog `fx()` SURF title box.

### leftovers
`renderer_kind("meteors")` = `Stub("fx")`
`renderer_kind("sankey-chart")` = `Stub("chart")`

### cargo test
```
export CARGO_TARGET_DIR=/Users/pedrogbraz/projects/cooud/cronus-kernel/target
cd /Users/pedrogbraz/projects/cooud/.wt/kernel-1r-b
cargo test --offline cronus_ui_progressive_blur cronus_ui_retro_grid cronus_ui_ripple -- --test-threads=1
```
15 passed (5 progressive-blur + 5 retro-grid + 5 ripple).
