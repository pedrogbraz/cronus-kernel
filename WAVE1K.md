# Cronus Audit Wave 1k — shimmer / reveal / text-shimmer

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1k-c`  
Branch: `feat/wave1k-shimmer-reveal-text`  
Base: `8dab4c9`  
Date: 2026-09-13

Done: 3 dedicated modules, catalog `fx()` skipped, tests green. Not pushed.  
Not touched: meteors (harness `Stub("fx")`), sankey-chart, rte, other charts.

Helpers: `src/cronus_ui_kit.rs` (`label_of`, `esc`, test `stub`). Pattern: `src/cronus_ui_skeleton.rs` (empty decorative block / CSS-only chrome). Not catalog `fx()` title SURF box (`padding:0.75rem 1rem;position:relative;overflow:hidden` + inner `<span>`).

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `d53db7a` | feat(ui): dedicated Shimmer renderer |
| `b387569` | feat(ui): dedicated Reveal renderer |
| `9f07459` | feat(ui): dedicated TextShimmer renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1j list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … sunburst-chart …
    "shimmer",
    "reveal",
    "text-shimmer",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| shimmer | `src/cronus_ui_shimmer.rs` | `cronus_ui_shimmer::render` |
| reveal | `src/cronus_ui_reveal.rs` | `cronus_ui_reveal::render` |
| text-shimmer | `src/cronus_ui_text_shimmer.rs` | `cronus_ui_text_shimmer::render` |

`src/main.rs` mods: `cronus_ui_shimmer`, `cronus_ui_reveal`, `cronus_ui_text_shimmer`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS + keyframes for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render` and **before** catalog `fx()`. Catalog stub / interact arms remain for unported families and were not rewritten.

106 `PORTED_FAMILIES`. 174 `FAMILIES` (`text-shimmer` added as a dedicated family, not a stub). Remaining chart stub: sankey-chart. meteors still `Stub("fx")`.

Fingerprint: dedicated HTML must not contain `padding:0.75rem 1rem;position:relative;overflow:hidden`.

## shimmer

React: decorative `<div data-slot="shimmer" aria-hidden>` with inner sweep (`packages/ui/src/components/shimmer.tsx`). Kernel is a skeleton-like empty block; the sweep is CSS `::after` + `@keyframes cui-shimmer` (2s linear, `color-mix` 10% `--cronus-fg`). Not catalog `fx("shimmer")` SURF title span.

Dedicated DOM (static, zero JS):

```html
<div data-slot="shimmer" aria-hidden="true"></div>
```

Forbidden fx: `<div data-slot="shimmer" style="…padding:0.75rem 1rem;position:relative;overflow:hidden;"><span>Demo</span></div>`. Asserts: no `style=`, no `<span`, no `Demo`, no `v-data`.

Chrome: `height: 0.9rem` `width: 8rem` `border-radius: var(--cronus-radius-md)` `background: var(--cronus-surface-overlay)`; sweep `translateX(-100%)` → `100%`.

## reveal

React: `motion.div data-slot="reveal"` with `fadeInUp` (`opacity 0, y 16` → visible, 500ms ease-out-quart) (`packages/ui/src/components/reveal.tsx`). Kernel wraps label text; IntersectionObserver is CSS-only mount animation (`cui-reveal`). Not catalog `fx("reveal")` SURF title span.

Dedicated DOM (static, zero JS):

```html
<div data-slot="reveal">Headline</div>
```

Forbidden fx: inner `<span>` title only + SURF padding box. Asserts: no `style=`, no `<span`, label escaped.

Chrome: `animation: cui-reveal 500ms var(--ease-out-quart) both`; from `translateY(16px)` / `opacity: 0`.

## text-shimmer

React: default `<p data-slot="text-shimmer">` with clipped gradient + `backgroundPosition` loop (`packages/ui/src/components/text-shimmer.tsx`). Kernel DOM is the assigned `<span data-slot="text-shimmer">` with label text. Gradient animation is CSS (`cui-text-shimmer` 2s linear). Zero JS.

Dedicated DOM (static, zero JS):

```html
<span data-slot="text-shimmer">Thinking</span>
```

Forbidden fx: wrapping `<div data-slot="text-shimmer" style="…padding:0.75rem 1rem;position:relative;overflow:hidden;"><span>…</span></div>`. Asserts: no `<div`, no `style=`, no SURF.

Chrome: dual `background-image` (`--cronus-surface-base` sweep over `--cronus-fg-tertiary`), `background-clip: text`, `background-size: 250% 100%`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_shimmer
  running 4 tests
  test cronus_ui_shimmer::tests::chrome_shimmer_via_css ... ok
  test cronus_ui_shimmer::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_shimmer::tests::root_is_empty_div_not_fx_title_box ... ok
  test cronus_ui_shimmer::tests::skips_fx_surf_title_box ... ok
  test result: ok. 4 passed

cargo test --offline cronus_ui_reveal
  running 5 tests
  test cronus_ui_reveal::tests::chrome_reveal_via_css ... ok
  test cronus_ui_reveal::tests::label_is_escaped ... ok
  test cronus_ui_reveal::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_reveal::tests::root_wraps_label_text_not_fx_title_box ... ok
  test cronus_ui_reveal::tests::skips_fx_surf_title_box ... ok
  test result: ok. 5 passed

cargo test --offline cronus_ui_text_shimmer
  running 5 tests
  test cronus_ui_text_shimmer::tests::chrome_text_shimmer_via_css ... ok
  test cronus_ui_text_shimmer::tests::label_is_escaped ... ok
  test cronus_ui_text_shimmer::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_text_shimmer::tests::root_is_span_with_label_not_fx_title_box ... ok
  test cronus_ui_text_shimmer::tests::skips_fx_surf_title_box ... ok
  test result: ok. 5 passed

cargo test --offline stub_renderer_gate
  running 7 tests
  test cli::stub_renderer_gate::tests::button_is_dedicated ... ok
  test cli::stub_renderer_gate::tests::dedicated_module_no_sidecar_assets ... ok
  test cli::stub_renderer_gate::tests::every_family_is_classified ... ok
  test cli::stub_renderer_gate::tests::meteors_is_stub_fx ... ok
  test cli::stub_renderer_gate::tests::no_new_stub_families ... ok
  test cli::stub_renderer_gate::tests::ported_families_are_dedicated_and_skip_interact ... ok
  test cli::stub_renderer_gate::tests::sankey_chart_is_stub ... ok
  test result: ok. 7 passed
```

`meteors_is_stub_fx` asserts `renderer_kind("meteors") == Stub("fx")` and the catalog fx fingerprint. `sankey_chart_is_stub` remains `Stub("chart")`.

Wave total: **21 passed** (4 + 5 + 5 + 7).
