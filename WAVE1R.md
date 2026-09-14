# Wave 1r — last real dedicated ports (7)

A: gradient-border, light-rays, orbit
B: progressive-blur, retro-grid, ripple
C: motion-presets
sankey-chart and meteors remain stubs.

## C — motion-presets

Dedicated CSS-only demo of named token presets (`fadeIn`, `fadeInUp`, `scaleIn`
from `motion-presets.ts`). Zero JS, zero framer-motion, zero `<script>`,
zero `<style>` tags, zero inline `style=`. Token CSS in `COMPONENT_CHROME`.
`PORTED_FAMILIES` length 167 (166 + 1).

### motion-presets
React SoT is a token module (no `data-slot="motion-presets"` today).
Kernel: `<div data-slot="motion-presets">` with three
`<div data-slot="motion-preset" data-preset="...">` rows (fade-in, fade-in-up,
scale-in). First row uses escaped label. CSS keyframes: opacity 0→1,
opacity + translateY(16px→0), opacity + scale(0.96→1). Timing
`cubic-bezier(0.16, 1, 0.3, 1)` ~0.5s (easeOutQuart).
`prefers-reduced-motion`: `animation: none` (final pose).
Skips catalog `fx()` SURF title box.

### leftovers
`renderer_kind("meteors")` = `Stub("fx")`
`renderer_kind("sankey-chart")` = `Stub("chart")`

### cargo test
```
export CARGO_TARGET_DIR=/Users/pedrogbraz/projects/cooud/cronus-kernel/target
cd /Users/pedrogbraz/projects/cooud/.wt/kernel-1r-c
cargo test --offline cronus_ui_motion_presets -- --test-threads=1
```
5 passed.
