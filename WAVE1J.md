# Cronus Audit Wave 1j — heatmap / comparison-slider / code-tabs

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1j-b`  
Branch: `feat/wave1j-heat-compare-code`  
Base: `e5b577d`  
Date: 2026-09-13

Done: 3 dedicated modules, interact/stub skipped, tests green. Not pushed.  
Not touched: segmented-control, sunburst-chart, meteors, sankey-chart, heatmap-chart.

Helpers: `src/cronus_ui_kit.rs` (`esc`, `item`, `label_of`, `texts`, `numeric_series`, `fmt_coord`). Pattern: checkbox (NO voodoo) — static HTML, no interact `tabs()` / catalog `display()` / `fx()` stubs.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

`PORTED_FAMILIES` is now 97 (94 after wave 1i + these 3).

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `438d63b` | feat(ui): dedicated Heatmap renderer |
| `ea041a8` | feat(ui): dedicated ComparisonSlider renderer |
| `0fde8d3` | feat(ui): dedicated CodeTabs renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1i list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … notification-center …
    "heatmap",
    "comparison-slider",
    "code-tabs",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| heatmap | `src/cronus_ui_heatmap.rs` | `cronus_ui_heatmap::render` |
| comparison-slider | `src/cronus_ui_comparison_slider.rs` | `cronus_ui_comparison_slider::render` |
| code-tabs | `src/cronus_ui_code_tabs.rs` | `cronus_ui_code_tabs::render` |

`src/main.rs` mods: `cronus_ui_heatmap`, `cronus_ui_comparison_slider`, `cronus_ui_code_tabs`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

## heatmap

React: Root `data-slot="heatmap"` + day cells `data-slot="heatmap-day"` + legend (`packages/ui/src/components/heatmap.tsx`). Calendar-style grid (`grid-flow-col` / 7 rows). Level 0 empty inset; 1..4 ramp primary opacity.

Dedicated DOM (default series 4,8,6,10,7):

```html
<div data-slot="heatmap"><div role="img" aria-label="Activity"><div data-slot="heatmap-day" data-level="2" title="4 on 2026-01-01"></div>…</div><div data-slot="heatmap-legend"><span>Less</span><span data-slot="heatmap-legend-swatch" data-level="0"></span>…<span>More</span></div></div>
```

Numeric items (or comma lists) drive cell count. Legend is always emitted (five swatches). Dates start at `2026-01-01`.

Forbidden catalog `chart("heatmap")`: `<figure data-slot="heatmap"><figcaption>` + dummy polyline `0,30 20,22…`. Asserts: no `<figure`, no `figcaption`. `heatmap-chart` remains a chart stub.

Chrome: inline-flex column; day grid `grid-auto-flow: column` / 7 rows; cells `0.75rem` on `var(--cronus-surface-inset)` with `color-mix` primary ramps.

## comparison-slider

React: Root `data-slot="comparison-slider"` + `comparison-after` (full base) + `comparison-before` (clipped overlay) (`packages/ui/src/components/comparison-slider.tsx`). Default divider 50%. Kernel port is a **static** split — no pointer/keyboard drag JS.

Dedicated DOM:

```html
<div data-slot="comparison-slider"><div data-slot="comparison-after">After</div><div data-slot="comparison-before">Before</div></div>
```

First text / `before` item is the before layer. Second text / `after` item is the after layer. Label-only still emits After as the base.

Forbidden catalog `fx("comparison-slider")`: SURF title box `padding:0.75rem 1rem;position:relative;overflow:hidden` + `<span>{title}</span>`. Asserts: no inline `style=`, no SURF, no title `<span>`.

Chrome: `aspect-ratio: 16 / 9`, inset surface, before `clip-path: inset(0 50% 0 0)`, static `::after` divider at 50%.

## code-tabs

React: Root `data-slot="code-tabs"` + `code-tabs-list` / triggers + panels with `code-tabs-pre` (`packages/ui/src/components/code-tabs.tsx`). Kernel port is **static first panel** — no tab-switch JS.

Dedicated DOM:

```html
<div data-slot="code-tabs"><div data-slot="code-tabs-list" role="tablist"><button type="button" role="tab" data-slot="code-tabs-trigger" data-state="active" aria-selected="true">bun</button>…</div><div data-slot="code-tabs-panel" role="tabpanel"><pre data-slot="code-tabs-pre"><code data-slot="code-tabs-code">bunx cronus-ui add code-tabs</code></pre></div></div>
```

Triggers come from texts (excluding `code` items). First panel `code` is item-type `code`, else first item `config.code`, else the first trigger label. Only the first panel is emitted.

Forbidden interact `tabs("code-tabs")`: generic `role="tablist"` + `onclick` + inline BASE styles **without** `code-tabs-pre` / `code-tabs-list`. Asserts: no `onclick`, has `code-tabs-pre`.

Chrome: raised surface, overlay list, tertiary triggers, active `var(--cronus-fg)`, mono `var(--cronus-font-mono)`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_heatmap
  running 9 tests
  test cronus_ui_heatmap::tests::chrome_is_token_only ... ok
  test cronus_ui_heatmap::tests::comma_list_item_is_series ... ok
  test cronus_ui_heatmap::tests::default_series_when_only_label ... ok
  test cronus_ui_heatmap::tests::heatmap_level_matches_react_buckets ... ok
  test cronus_ui_heatmap::tests::legend_has_five_swatches ... ok
  test cronus_ui_heatmap::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_heatmap::tests::numeric_items_drive_days ... ok
  test cronus_ui_heatmap::tests::root_is_div_with_day_cells_not_figure ... ok
  test cronus_ui_heatmap::tests::skips_chart_figure_stub ... ok
  test result: ok. 9 passed

cargo test --offline cronus_ui_comparison_slider
  running 6 tests
  test cronus_ui_comparison_slider::tests::before_after_from_texts ... ok
  test cronus_ui_comparison_slider::tests::root_is_static_split_not_fx_box ... ok
  test cronus_ui_comparison_slider::tests::before_after_item_kinds ... ok
  test cronus_ui_comparison_slider::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_comparison_slider::tests::skips_fx_surf_title_box ... ok
  test cronus_ui_comparison_slider::tests::chrome_is_token_only ... ok
  test result: ok. 6 passed

cargo test --offline cronus_ui_code_tabs
  running 6 tests
  test cronus_ui_code_tabs::tests::root_list_triggers_first_panel_pre ... ok
  test cronus_ui_code_tabs::tests::triggers_from_texts ... ok
  test cronus_ui_code_tabs::tests::skips_interact_tabs ... ok
  test cronus_ui_code_tabs::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_code_tabs::tests::chrome_is_token_only ... ok
  test cronus_ui_code_tabs::tests::first_panel_code_from_item_config ... ok
  test result: ok. 6 passed

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

Also green: `cargo test --offline ported_family_skips_interact`. `meteors` remains fx stub; `sankey-chart` remains chart stub; `heatmap-chart` remains chart stub.

Wave total: **28 passed** (9 + 6 + 6 + 7).

No chart `<figure>`. No fx SURF title box. No interact `tabs()` generic tablist. No Voodoo even with runtime on.

Not pushed.
