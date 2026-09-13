# Cronus Audit Wave 1n — marquee / gradient-text / shiny-text

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1n-c`  
Branch: `feat/wave1n-marquee-grad-shiny`  
Base: `8c392a2`  
Date: 2026-09-13

Done: 3 dedicated modules, catalog `fx()` skipped, interact skipped, tests green. Not pushed.  
Not touched: meteors (harness `Stub("fx")`), sankey-chart, carousel, kanban.

Helpers: `src/cronus_ui_kit.rs` (`texts`, `label_of`, `esc`, test `stub`). Pattern: `src/cronus_ui_text_shimmer.rs` (dedicated slot + CSS chrome). Not catalog `fx()` title SURF box (`padding:0.75rem 1rem;position:relative;overflow:hidden` + inner `<span>`). Not interact generic HTML. Not voodoo.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `079dede` | feat(ui): dedicated Marquee renderer |
| `0a0594c` | feat(ui): dedicated GradientText renderer |
| `782febb` | feat(ui): dedicated ShinyText renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1m list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … toast …
    "marquee",
    "gradient-text",
    "shiny-text",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| marquee | `src/cronus_ui_marquee.rs` | `cronus_ui_marquee::render` |
| gradient-text | `src/cronus_ui_gradient_text.rs` | `cronus_ui_gradient_text::render` |
| shiny-text | `src/cronus_ui_shiny_text.rs` | `cronus_ui_shiny_text::render` |

`src/main.rs` mods: `cronus_ui_marquee`, `cronus_ui_gradient_text`, `cronus_ui_shiny_text`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for marquee scroll, gradient fill, and shiny sheen.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render` and **before** catalog `fx()`. Catalog stub / interact arms remain for unported families and were not rewritten.

133 `PORTED_FAMILIES`. 175 `FAMILIES`. Remaining chart stub: sankey-chart. meteors still `Stub("fx")`.

Fingerprint: dedicated HTML must not contain `padding:0.75rem 1rem;position:relative;overflow:hidden`. Marquee also requires `data-slot="marquee-group"`.

## marquee

React: `<div data-slot="marquee">` wrapping `<div data-slot="marquee-group">` copies (`packages/ui/src/components/marquee.tsx`). Kernel is a **static track** from `texts`. CSS translate (`cui-marquee`), pause-on-hover, edge fade, reduced-motion off. Zero JS (no ResizeObserver). Not catalog `fx()` SURF title box.

Dedicated DOM (static, zero JS):

```html
<div data-slot="marquee"><div data-slot="marquee-group"><span>Acme</span><span>Stripe</span></div></div>
```

Forbidden fx: `<div data-slot="marquee" style="…padding:0.75rem 1rem;position:relative;overflow:hidden;"><span>Demo</span></div>`. Asserts: no `style=`, no SURF, has `marquee-group`.

Chrome: `overflow: hidden` + mask fade; group `animation: cui-marquee 20s linear infinite`; `var(--cronus-fg)`.

## gradient-text

React: `<span data-slot="gradient-text">` with `bg-gradient-primary bg-clip-text text-transparent` (`packages/ui/src/components/gradient-text.tsx`). Kernel wraps the **label**. CSS fill (`linear-gradient(135deg, var(--cronus-primary), var(--cronus-accent))` + `background-clip: text`). Zero JS. Not catalog `fx()` SURF title box.

Dedicated DOM (static, zero JS):

```html
<span data-slot="gradient-text">Proud of</span>
```

Forbidden fx: `<div data-slot="gradient-text" style="…padding:0.75rem 1rem;position:relative;overflow:hidden;"><span>Demo</span></div>`. Asserts: no `<div`, no `style=`, no SURF.

Chrome: `color: transparent`; `background-clip: text`; primary → accent 135deg.

## shiny-text

React: `<span data-slot="shiny-text">` wrapping inner text with a sheen sweep (`packages/ui/src/components/shiny-text.tsx`). Kernel wraps the **label**. CSS shine (`cui-shiny-text` background-position, `var(--cronus-fg-tertiary)` / `var(--cronus-fg)`). Zero JS. Not catalog `fx()` SURF title box.

Dedicated DOM (static, zero JS):

```html
<span data-slot="shiny-text">Shimmer</span>
```

Forbidden fx: `<div data-slot="shiny-text" style="…padding:0.75rem 1rem;position:relative;overflow:hidden;"><span>Demo</span></div>`. Asserts: no `<div`, no `style=`, no SURF, `dedicated_fn_name("shiny-text") == Some("cronus_ui_shiny_text::render")`, `renderer_kind("meteors") == Stub("fx")`.

Chrome: `background-size: 200% 100%`; `animation: cui-shiny-text 3s linear infinite`; reduced-motion `animation: none`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_marquee
  running 6 tests
  test cronus_ui_marquee::tests::label_is_escaped ... ok
  test cronus_ui_marquee::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_marquee::tests::extra_text_items_become_spans ... ok
  test cronus_ui_marquee::tests::skips_fx_surf_title_box ... ok
  test cronus_ui_marquee::tests::root_is_marquee_group_of_texts_not_fx_title_box ... ok
  test cronus_ui_marquee::tests::chrome_marquee_via_css ... ok
  test result: ok. 6 passed

cargo test --offline cronus_ui_gradient_text
  running 5 tests
  test cronus_ui_gradient_text::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_gradient_text::tests::label_is_escaped ... ok
  test cronus_ui_gradient_text::tests::root_is_span_with_label_not_fx_title_box ... ok
  test cronus_ui_gradient_text::tests::skips_fx_surf_title_box ... ok
  test cronus_ui_gradient_text::tests::chrome_gradient_fill_via_css ... ok
  test result: ok. 5 passed

cargo test --offline cronus_ui_shiny_text
  running 5 tests
  test cronus_ui_shiny_text::tests::label_is_escaped ... ok
  test cronus_ui_shiny_text::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_shiny_text::tests::root_is_span_wrapping_text_not_fx_title_box ... ok
  test cronus_ui_shiny_text::tests::skips_fx_surf_title_box ... ok
  test cronus_ui_shiny_text::tests::chrome_shine_via_css ... ok
  test result: ok. 5 passed

cargo test --offline stub_renderer_gate
  running 7 tests
  test cli::stub_renderer_gate::tests::meteors_is_stub_fx ... ok
  test cli::stub_renderer_gate::tests::button_is_dedicated ... ok
  test cli::stub_renderer_gate::tests::sankey_chart_is_stub ... ok
  test cli::stub_renderer_gate::tests::every_family_is_classified ... ok
  test cli::stub_renderer_gate::tests::no_new_stub_families ... ok
  test cli::stub_renderer_gate::tests::dedicated_module_no_sidecar_assets ... ok
  test cli::stub_renderer_gate::tests::ported_families_are_dedicated_and_skip_interact ... ok
  test result: ok. 7 passed
```

`meteors_is_stub_fx` asserts `renderer_kind("meteors") == Stub("fx")` and the catalog fx fingerprint. `sankey_chart_is_stub` remains `Stub("chart")`.

Wave total: **23 passed** (6 + 5 + 5 + 7).
