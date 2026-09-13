# Cronus Audit Wave 1k — choropleth-chart / profit-loss-chart / scroll-progress

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1k-a`  
Branch: `feat/wave1k-choro-pl-scroll`  
Base: `8dab4c9`  
Date: 2026-09-13

Done: 3 dedicated modules, interact skipped, tests green. Not pushed.  
Not touched: meteors, sankey-chart, shimmer, rich-text-editor.

Helpers: `src/cronus_ui_kit.rs` (`label_of`, `numeric_items`, `fmt_coord`, `chart_signed_line_points`, `chart_zero_y`). Pattern: `src/cronus_ui_line_chart.rs` / `src/cronus_ui_progress.rs` / `src/cronus_ui_usage_meter.rs` (no voodoo, no inline BASE/SURF/CTRL). Not catalog `chart()` figure stub. Not interact `progress()`.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `021d081` | feat(ui): dedicated ChoroplethChart renderer |
| `564ad94` | feat(ui): dedicated ProfitLossChart renderer |
| `78a4b0c` | feat(ui): dedicated ScrollProgress renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1j list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … sunburst-chart …
    "choropleth-chart",
    "profit-loss-chart",
    "scroll-progress",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| choropleth-chart | `src/cronus_ui_choropleth_chart.rs` | `cronus_ui_choropleth_chart::render` |
| profit-loss-chart | `src/cronus_ui_profit_loss_chart.rs` | `cronus_ui_profit_loss_chart::render` |
| scroll-progress | `src/cronus_ui_scroll_progress.rs` | `cronus_ui_scroll_progress::render` |

`src/main.rs` mods: `cronus_ui_choropleth_chart`, `cronus_ui_profit_loss_chart`, `cronus_ui_scroll_progress`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

106 `PORTED_FAMILIES`. Remaining chart stub: sankey-chart (`Stub("chart")`). meteors still fx stub.

## choropleth-chart

React: root `<div data-slot="choropleth-chart">` plus SVG region paths (`packages/ui/src/components/choropleth-chart.tsx`). Kernel emits the same 7 demo cells (`M10 10 h 80 v 50 h -80 z` …) with `fill="var(--cronus-primary)"` and `fill-opacity` from value/max (`0.15 + ratio * 0.85`). Numeric items overlay the first cells; missing cells keep the demo values. Not catalog `chart()` `<figure><figcaption>` + dummy polyline `0,30 20,22…`.

Dedicated DOM (static, zero JS):

```html
<div data-slot="choropleth-chart" role="img" aria-label="Map"><svg viewBox="0 0 200 190" aria-hidden="true"><path d="M10 10 h 80 v 50 h -80 z" fill="var(--cronus-primary)" stroke="var(--cronus-border)" fill-opacity="0.53"><title>Northwest: 42</title></path><!-- 6 more regions --></svg></div>
```

Forbidden stub (`chart("choropleth-chart")`): `<figure data-slot="choropleth-chart">` + `<figcaption>` + `points="0,30 20,22 40,26 60,12 80,16 100,8 120,14"`. Asserts: no `<figure`, no `figcaption`, no stub polyline, 7 `<path`, token fills.

Chrome: block `height: 16rem`; path `fill: var(--cronus-primary)` `stroke: var(--cronus-border)`.

## profit-loss-chart

React: `<div data-slot="profit-loss-chart">` wrapping a P/L line (`packages/ui/src/components/profit-loss-chart.tsx` / `charts/profit-loss-line.tsx`). Kernel maps a signed series (default `4, -2, 6, -1, 3`) onto viewBox `0 0 200 100` with 0 in domain, draws a zero `<line>` (`var(--cronus-border)`), and splits polylines at zero crossings: `stroke="var(--cronus-success)"` above, `stroke="var(--cronus-error)"` below. Not stub polyline `0,30 20,22…`.

Dedicated DOM (static, zero JS):

```html
<div data-slot="profit-loss-chart" role="img" aria-label="P/L"><svg viewBox="0 0 200 100" aria-hidden="true"><line x1="10" y1="70" x2="190" y2="70" stroke="var(--cronus-border)" stroke-width="1"></line><polyline fill="none" stroke="var(--cronus-success)" stroke-width="2" points="…"></polyline><polyline fill="none" stroke="var(--cronus-error)" stroke-width="2" points="…"></polyline></svg></div>
```

Forbidden stub (`chart("profit-loss-chart")`): `<figure>` + dummy polyline. Asserts: no `<figure`, no stub polyline, has both success and error strokes. sankey-chart still renders the figure stub.

Chrome: `aspect-ratio: 2 / 1`; polyline strokes `var(--cronus-success)` / `var(--cronus-error)`.

## scroll-progress

React: `<div data-slot="scroll-progress">` plus `scroll-progress-fill` (bar) or `scroll-progress-ring` (circle) (`packages/ui/src/components/scroll-progress.tsx`). Kernel is a **static** snapshot: value from props / numeric item text / item config, **default 40**. Default variant is bar. `variant=circle` (or style `circle`) emits the ring. Not interact `progress("scroll-progress")` native `<progress>` or catalog `fx()` SURF.

Dedicated DOM (static, zero JS):

```html
<div data-slot="scroll-progress" data-variant="bar" role="progressbar" aria-valuenow="40" aria-valuemin="0" aria-valuemax="100" aria-label="Reading"><div data-slot="scroll-progress-fill" style="width:40%"></div></div>
```

Forbidden interact (`progress("scroll-progress")`): `<div data-slot="scroll-progress">` + `<progress max="100">` and `{ value }` interp. Forbidden fx stub: `padding:0.75rem 1rem;position:relative;overflow:hidden`. Asserts: no `<progress`, has `scroll-progress-fill`.

Chrome: bar `height: 0.25rem` `var(--cronus-surface-inset)`; fill `var(--cronus-primary)`. Circle uses ring + `scroll-progress-value`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_choropleth_chart
  running 7 tests
  test cronus_ui_choropleth_chart::tests::chrome_is_token_only ... ok
  test cronus_ui_choropleth_chart::tests::comma_list_item_is_series ... ok
  test cronus_ui_choropleth_chart::tests::default_regions_when_only_label ... ok
  test cronus_ui_choropleth_chart::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_choropleth_chart::tests::numeric_items_drive_values ... ok
  test cronus_ui_choropleth_chart::tests::root_is_div_with_svg_regions_not_figure ... ok
  test cronus_ui_choropleth_chart::tests::skips_chart_figure_stub ... ok
  test result: ok. 7 passed

cargo test --offline cronus_ui_profit_loss_chart
  running 7 tests
  test cronus_ui_profit_loss_chart::tests::chrome_is_token_only ... ok
  test cronus_ui_profit_loss_chart::tests::comma_list_item_is_series ... ok
  test cronus_ui_profit_loss_chart::tests::default_series_when_only_label ... ok
  test cronus_ui_profit_loss_chart::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_profit_loss_chart::tests::numeric_items_drive_series ... ok
  test cronus_ui_profit_loss_chart::tests::root_is_div_with_svg_polylines_not_figure ... ok
  test cronus_ui_profit_loss_chart::tests::skips_chart_figure_stub ... ok
  test result: ok. 7 passed

cargo test --offline cronus_ui_scroll_progress
  running 9 tests
  test cronus_ui_scroll_progress::tests::chrome_is_token_only ... ok
  test cronus_ui_scroll_progress::tests::circle_variant_emits_ring ... ok
  test cronus_ui_scroll_progress::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_scroll_progress::tests::numeric_label_is_value_not_label ... ok
  test cronus_ui_scroll_progress::tests::root_is_div_with_fill_default_40 ... ok
  test cronus_ui_scroll_progress::tests::skips_interact_html_progress ... ok
  test cronus_ui_scroll_progress::tests::value_from_item_config ... ok
  test cronus_ui_scroll_progress::tests::value_from_item_text_number ... ok
  test cronus_ui_scroll_progress::tests::value_from_props ... ok
  test result: ok. 9 passed

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

`sankey_chart_is_stub` still asserts `RendererKind::Stub("chart")` and the dummy polyline figure. meteors remains `Stub("fx")`.

Also green: `cargo test --offline cronus_ui_widgets` (19 passed), including `ported_family_skips_interact` over the appended list. `voodoo_attrs_only_when_runtime_on` now asserts scroll-progress has no `{ value }` / `v-data` (dedicated fill, not interact `<progress>`).

Wave total: **30 passed** (7 + 7 + 9 + 7).
