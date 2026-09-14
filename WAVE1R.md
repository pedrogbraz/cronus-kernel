# Wave 1r — last real dedicated ports (7)

A: gradient-border, light-rays, orbit
B: progressive-blur, retro-grid, ripple
C: motion-presets
sankey-chart and meteors remain stubs.

## A — gradient-border / light-rays / orbit

Dedicated CONTRACT renderers. Zero JS, zero `<script>`, zero `<style>` tags,
zero inline `style=`, zero canvas, zero rAF. Token CSS in `COMPONENT_CHROME`.
`PORTED_FAMILIES` length 169 (166 + 3).

## B — progressive-blur / retro-grid / ripple

CSS overlay/field + content. leftover stub is meteors.

### gradient-border
React idle: `<div data-slot="gradient-border">` wrapping an inner surface div + children.
Kernel: `<div data-slot="gradient-border"><div data-slot="gradient-border-inner">{label}</div></div>`.
CSS: 1px padding, `linear-gradient` using `var(--cronus-primary)` / `var(--cronus-accent)`,
inner `background: var(--cronus-surface-raised)`, radius. No glow JS.
Skips catalog `fx()` SURF title box.

### light-rays
React: `<div data-slot="light-rays">` + aria-hidden rotating conic-gradient layer + relative children.
React injects `<style>` keyframes — kernel does not.
Kernel: `<div data-slot="light-rays"><div data-slot="light-rays-field" aria-hidden="true"></div><div data-slot="light-rays-content">{label}</div></div>`.
CSS: `repeating-conic-gradient` with `var(--cronus-primary)`, `@keyframes cui-light-rays` rotate 360, radial mask.
`prefers-reduced-motion`: field `{ display: none }`.
Skips catalog `fx()` SURF title box.

### orbit
React: `<div data-slot="orbit">` stage + `<style>` keyframes + `OrbitRing` (`data-slot="orbit-ring"`)
+ positioners + `data-slot="orbit-item"`. Inline `--orbit-angle`.
Kernel emits no `<style>` and no inline style. CSS nth-child for up to 6 items.
Kernel:
```
<div data-slot="orbit">
  <div data-slot="orbit-nucleus">{first text or label}</div>
  <div data-slot="orbit-ring" aria-hidden="true">
    <div data-slot="orbit-item">{each extra text}</div>
  </div>
</div>
```
If `texts` has 1 item, that string is the nucleus and 3 decorative orbit-items (same escaped label).
If `texts` has 2+, first is nucleus, rest are orbit-items (cap 6).
CSS: ring is a circle `border: 1px solid var(--cronus-border)`; items placed with nth-child
rotate (0, 60, 120, …). `@keyframes cui-orbit-spin` on the ring. `:hover` / `:focus-within`
pause. `prefers-reduced-motion`: animation none (items stay at nth-child angles).
Skips catalog `fx()` SURF title box.

### leftovers
`renderer_kind("meteors")` = `Stub("fx")`
`renderer_kind("sankey-chart")` = `Stub("chart")`

### cargo test
```
export CARGO_TARGET_DIR=/Users/pedrogbraz/projects/cooud/cronus-kernel/target
cd /Users/pedrogbraz/projects/cooud/.wt/kernel-1r-a
cargo test --offline -- --test-threads=1 cronus_ui_gradient_border cronus_ui_light_rays cronus_ui_orbit
```
17 passed (5 gradient-border + 5 light-rays + 7 orbit).
