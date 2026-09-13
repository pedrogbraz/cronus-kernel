# Wave 1m — 3 dedicated kernel ports

tilt-card, star-border, glass-card.
sankey-chart and meteors remain stubs. terminal/toast untouched.

`PORTED_FAMILIES` = 124. Appended `"tilt-card", "star-border", "glass-card"`.
Each family is a dedicated module via `cronus_ui_kit` (`label_of`). No voodoo.
Not catalog `display()` / `fx()` SURF.

## DOM

| family | dedicated HTML | skipped |
| --- | --- | --- |
| tilt-card | `<div data-slot="tilt-card">Tilt</div>` | interact/display `<section … style=…SURF…>` |
| star-border | `<div data-slot="star-border">Sparkle</div>` | fx() `<div … overflow:hidden"><span>…</span>` |
| glass-card | `<div data-slot="glass-card">Frost</div>` | interact/display `<section … style=…SURF…>` |

## COMPONENT_CHROME

- tilt-card: static 3D (`transform-style: preserve-3d`, `perspective(1000px)`, `--tilt-rx/ry` defaults 0deg)
- star-border: `::before`/`::after` sparkle heads, `@keyframes cui-star-border`, `offset-path`
- glass-card: `backdrop-filter: blur(24px)`, translucent `color-mix` of `--cronus-surface-raised`, top-edge `::before`

## Wiring

- `src/cronus_ui_tilt_card.rs`, `src/cronus_ui_star_border.rs`, `src/cronus_ui_glass_card.rs`
- mods in `src/main.rs`
- `dedicated_render` + `PORTED_FAMILIES` in `src/cronus_ui_widgets.rs` and `scripts/gen_cronus_ui_widgets.py`
- `dedicated_fn_name`, interact fingerprints, stub-gate file list in `src/cli/stub_renderer_gate.rs`

## Commits (not pushed)

- `ec97793` feat(ui): dedicated TiltCard renderer
- `75b706f` feat(ui): dedicated StarBorder renderer
- `e964330` feat(ui): dedicated GlassCard renderer

## Tests

```
cargo test cronus_ui_tilt_card
  5 passed  (root div, escaped label, skips display SURF, no voodoo, chrome 3D)

cargo test cronus_ui_star_border
  5 passed  (root div, escaped label, skips fx SURF, no voodoo, chrome keyframes)

cargo test cronus_ui_glass_card
  5 passed  (root div, escaped label, skips display SURF, no voodoo, chrome backdrop-filter)

cargo test stub_renderer_gate
  7 passed  (button dedicated, no sidecar, every family classified,
             meteors Stub("fx"), sankey-chart Stub("chart"),
             ported skip interact, no new stubs)
```
