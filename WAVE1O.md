# Wave 1o — 3 dedicated kernel ports

aspect-ratio, frame, flip-card.
sankey-chart and meteors remain stubs. countdown and charts untouched.

142 `PORTED_FAMILIES`. Dedicated modules (not catalog `display()` / `fx()`, not voodoo).

## DOM

- **aspect-ratio** — `<div data-slot="aspect-ratio">` wrapping label. CSS `aspect-ratio: 16 / 9` default in COMPONENT_CHROME.
- **frame** — `<div data-slot="frame">` + `frame-chrome` (three traffic-light dots) + `frame-content` with label.
- **flip-card** — `<div data-slot="flip-card">` + `flip-card-front` (label) + `flip-card-back` (extra text). Static CSS 3D hover/focus flip; no JS.

## Wiring

`cronus_ui_aspect_ratio.rs`, `cronus_ui_frame.rs`, `cronus_ui_flip_card.rs`.
mods, `dedicated_render`, `dedicated_fn_name`, generator `PORTED_FAMILIES`, `COMPONENT_CHROME`, stub-gate file list + interact fingerprints.

## Evidence

```
cargo test -- --test-threads=1 cronus_ui_aspect_ratio
# 6 passed

cargo test -- --test-threads=1 cronus_ui_frame
# 6 passed

cargo test -- --test-threads=1 cronus_ui_flip_card
# 6 passed

cargo test -- --test-threads=1 stub_renderer_gate
# 7 passed (ported skip interact; meteors Stub("fx"); sankey-chart Stub("chart"))
```

Commits: `02f7c23` aspect-ratio, `c2e3636` frame, `e7f73f8` flip-card.
Base: `24c4a6f`. Branch: `feat/wave1o-aspect-frame-flip`. Not pushed.
