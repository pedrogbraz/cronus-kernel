# Cronus Audit Wave 1l — particles / sparkles-text / noise

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1l-a`  
Branch: `feat/wave1l-particles-sparkles-noise`  
Base: `1693093`  
Date: 2026-09-13

Done: 3 dedicated modules, catalog `fx()` skipped, tests green. Not pushed.  
Not touched: meteors (harness `Stub("fx")`), sankey-chart, morphing-popover, timeline.

Helpers: `src/cronus_ui_kit.rs` (`label_of`, `esc`, test `stub`). Pattern: `src/cronus_ui_shimmer.rs` / `src/cronus_ui_reveal.rs` / `src/cronus_ui_text_shimmer.rs` (dedicated module, CSS-only chrome, no voodoo). Not catalog `fx()` title SURF box (`padding:0.75rem 1rem;position:relative;overflow:hidden` + inner `<span>`).

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `5776082` | feat(ui): dedicated Particles renderer |
| `5f7eda5` | feat(ui): dedicated SparklesText renderer |
| `850b404` | feat(ui): dedicated Noise renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1k list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … text-shimmer …
    "particles",
    "sparkles-text",
    "noise",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| particles | `src/cronus_ui_particles.rs` | `cronus_ui_particles::render` |
| sparkles-text | `src/cronus_ui_sparkles_text.rs` | `cronus_ui_sparkles_text::render` |
| noise | `src/cronus_ui_noise.rs` | `cronus_ui_noise::render` |

`src/main.rs` mods: `cronus_ui_particles`, `cronus_ui_sparkles_text`, `cronus_ui_noise`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS + keyframes/overlays for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render` and **before** catalog `fx()`. Catalog stub / interact arms remain for unported families and were not rewritten.

115 `PORTED_FAMILIES`. 174 `FAMILIES`. Remaining stubs: sankey-chart (`Stub("chart")`), meteors (`Stub("fx")`).

Fingerprint: dedicated HTML must not contain `padding:0.75rem 1rem;position:relative;overflow:hidden`.

## particles

React: canvas 2D particle field behind children (`packages/ui/src/components/particles.tsx`). Kernel is CSS-only dots via `COMPONENT_CHROME` background (`radial-gradient` specks + `@keyframes cui-particles` drift). No JS canvas. Not catalog `fx("particles")` SURF title span.

Dedicated DOM (static, zero JS):

```html
<div data-slot="particles">Specks</div>
```

Forbidden fx: `<div data-slot="particles" style="…padding:0.75rem 1rem;position:relative;overflow:hidden;"><span>Demo</span></div>`. Asserts: no `style=`, no `<span`, no `<canvas`, no `v-data`.

Chrome: layered `radial-gradient` dots in `color-mix(in oklch, var(--cronus-fg) …)` on `var(--cronus-surface-overlay)`; `::after` drifts 14s linear.

## sparkles-text

React: `<span data-slot="sparkles-text">` with decorative SVG sparkles (`packages/ui/src/components/sparkles-text.tsx`). Kernel is a span with label; sparkles are CSS `::before`/`::after` stars (`clip-path: polygon` + `@keyframes cui-sparkle`). Zero JS.

Dedicated DOM (static, zero JS):

```html
<span data-slot="sparkles-text">Magic</span>
```

Forbidden fx: wrapping `<div data-slot="sparkles-text" style="…padding:0.75rem 1rem;position:relative;overflow:hidden;"><span>…</span></div>`. Asserts: no `<div`, no `style=`, no SURF, label escaped.

Chrome: `var(--cronus-primary)` clipped stars; scale/opacity loop matching React `cronus-sparkle` keyframes.

## noise

React: SVG `feTurbulence` grain overlay (`packages/ui/src/components/noise.tsx`). Kernel wraps label text; grain is CSS `::after` (`repeating-radial-gradient` + `repeating-conic-gradient`, `opacity: 0.08`, `mix-blend-mode: overlay`). Still texture, no animation of its own.

Dedicated DOM (static, zero JS):

```html
<div data-slot="noise">Grain</div>
```

Forbidden fx: inner `<span>` title only + SURF padding box. Asserts: no `style=`, no `<span`, label escaped.

Chrome: overlay `opacity: 0.08` over `var(--cronus-fg)` grain; `isolation: isolate`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_particles
  running 5 tests
  test cronus_ui_particles::tests::chrome_particles_via_css ... ok
  test cronus_ui_particles::tests::label_is_escaped ... ok
  test cronus_ui_particles::tests::root_wraps_label_text_not_fx_title_box ... ok
  test cronus_ui_particles::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_particles::tests::skips_fx_surf_title_box ... ok
  test result: ok. 5 passed

cargo test --offline cronus_ui_sparkles_text
  running 5 tests
  test cronus_ui_sparkles_text::tests::label_is_escaped ... ok
  test cronus_ui_sparkles_text::tests::root_is_span_with_label_not_fx_title_box ... ok
  test cronus_ui_sparkles_text::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_sparkles_text::tests::skips_fx_surf_title_box ... ok
  test cronus_ui_sparkles_text::tests::chrome_sparkles_via_css ... ok
  test result: ok. 5 passed

cargo test --offline cronus_ui_noise
  running 5 tests
  test cronus_ui_noise::tests::label_is_escaped ... ok
  test cronus_ui_noise::tests::root_wraps_label_text_not_fx_title_box ... ok
  test cronus_ui_noise::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_noise::tests::skips_fx_surf_title_box ... ok
  test cronus_ui_noise::tests::chrome_noise_via_css ... ok
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

Wave total: **22 passed** (5 + 5 + 5 + 7).
