# Wave 1n — 3 dedicated kernel ports

carousel, code-block, description-list.
sankey-chart and meteors remain stubs. kanban and marquee untouched.

133 `PORTED_FAMILIES`. Dedicated modules (not catalog `display()` / `fx()`, not voodoo).

## DOM

- **carousel** — `<div data-slot="carousel">` + `carousel-content` + `carousel-item` from texts; static prev/next; no JS swipe.
- **code-block** — `<pre data-slot="code-block"><code>` from label/items. Not ai-code-block, not interact `codey()`.
- **description-list** — `<dl data-slot="description-list">` + `description-item` / `description-term` / `description-details` from paired texts (odd=term, even=details).

## Wiring

`cronus_ui_carousel.rs`, `cronus_ui_code_block.rs`, `cronus_ui_description_list.rs`.
mods, `dedicated_render`, `dedicated_fn_name`, generator `PORTED_FAMILIES`, `COMPONENT_CHROME`, stub-gate file list + interact fingerprints.

## Evidence

```
cargo test -- --test-threads=1 cronus_ui_carousel
# 7 passed

cargo test -- --test-threads=1 cronus_ui_code_block
# 7 passed

cargo test -- --test-threads=1 cronus_ui_description_list
# 8 passed

cargo test -- --test-threads=1 stub_renderer_gate
# 7 passed (ported skip interact; meteors Stub("fx"); sankey-chart Stub("chart"))
```

Commits: `5437ed2` carousel, `3a9b9e1` code-block, `fd9ef36` description-list.
Base: `8c392a2`. Branch: `feat/wave1n-carousel-code-dl`. Not pushed.
