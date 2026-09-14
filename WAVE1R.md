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

## C — motion-presets

CSS demo of fade-in / fade-in-up / scale-in. leftover stubs: meteors, sankey-chart.

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
Kernel emits no `<style>` and no inline style. Up to 6 items.
Kernel (parity pass 2026-09-14, mirrors React DOM):
```
<div data-slot="orbit" aria-label="{aria-label}">
  {label}
  <div data-slot="orbit-ring">
    <div data-slot="orbit-positioner"><div data-slot="orbit-holder">
      <div data-slot="orbit-item">{each text/item line}</div>
    </div></div>
  </div>
</div>
```
Nucleus is the `label` as bare text (React renders it as a text node). Orbit items are the
`text` / `item` lines (cap 6); with none, 3 slots repeat the label. `aria-label` is read from
props or item config (the parser attaches `aria-label:` to the preceding item).
CSS: stage 18rem (`size-72`), ring 16rem (radius 128px) with
`color-mix(in oklch, var(--cronus-border) 40%, transparent)` (`border-border/40`).
Slot angle `--orbit-angle` = 360/n via `:nth-child(k):nth-last-child(n-k+1)` quantity queries.
Positioner: `rotate: var(--orbit-angle)` + `cui-orbit-spin`; item: negated `rotate` + same
keyframe `reverse`, so content stays upright like React. `:hover` / `:focus-within` pause.
`prefers-reduced-motion`: animations none (static angles keep placement).
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
