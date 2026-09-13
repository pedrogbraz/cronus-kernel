# Cronus Audit Wave 1j — segmented-control / usage-meter / masonry

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1j-a`  
Branch: `feat/wave1j-seg-meter-masonry`  
Base: `e5b577d`  
Date: 2026-09-13

Done: 3 dedicated modules, interact skipped, tests green. Not pushed.  
Not touched: heatmap, charts, meteors, sankey-chart.

Helpers: `src/cronus_ui_kit.rs` (`esc`, `item`, `texts`). Pattern: `src/cronus_ui_toggle_group.rs` / `src/cronus_ui_progress.rs` (no voodoo, no inline BASE/SURF/CTRL). Not interact `radios()` / `progress()`. Not catalog `display()`.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `857c6cb` | feat(ui): dedicated SegmentedControl renderer |
| `4cd92ea` | feat(ui): dedicated UsageMeter renderer |
| `e45336f` | feat(ui): dedicated Masonry renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1i list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … notification-center …
    "segmented-control",
    "usage-meter",
    "masonry",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| segmented-control | `src/cronus_ui_segmented_control.rs` | `cronus_ui_segmented_control::render` |
| usage-meter | `src/cronus_ui_usage_meter.rs` | `cronus_ui_usage_meter::render` |
| masonry | `src/cronus_ui_masonry.rs` | `cronus_ui_masonry::render` |

`src/main.rs` mods: `cronus_ui_segmented_control`, `cronus_ui_usage_meter`, `cronus_ui_masonry`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

97 `PORTED_FAMILIES`. Remaining chart stub: sankey-chart. meteors still fx stub.

## segmented-control

React: track `data-slot="segmented-control"` `role="radiogroup"` plus item buttons `data-slot="segmented-control-item"` (`packages/ui/src/components/segmented-control.tsx`). Kernel uses `role="tablist"` + `role="tab"` buttons so the dedicated DOM is not interact `radios()` (`role="radiogroup"` + native `<input type="radio">`). Items come from texts. First selected (`aria-selected="true"` `data-state="active"`).

Dedicated DOM (static, zero JS):

```html
<div data-slot="segmented-control" role="tablist"><button type="button" data-slot="segmented-control-item" role="tab" aria-selected="true" data-state="active">Day</button><button type="button" data-slot="segmented-control-item" role="tab" aria-selected="false" data-state="inactive">Week</button></div>
```

Forbidden interact (`radios("segmented-control")`): `<div data-slot="segmented-control" role="radiogroup">` wrapping `<label><input type="radio">`. Asserts: no `<input`, no `type="radio"`, no `<label`, no `role="radiogroup"`, no `style=`.

Chrome: inline-flex track `gap: 0.25rem` on `var(--cronus-surface-overlay)` with `var(--cronus-border)`; active item `var(--cronus-surface-floating)`.

## usage-meter

React: `<div data-slot="usage-meter">` plus fill `data-slot="usage-meter-fill"` and optional `usage-meter-label` (`packages/ui/src/components/usage-meter.tsx`). Kernel fill width is the 0–100 value (props / numeric item text / item config, **default 40**). Non-numeric `label`/`title` becomes `usage-meter-label`. Not interact `progress("usage-meter")` native `<progress>` or catalog `display()` SURF without `usage-meter-fill`.

Dedicated DOM (static, zero JS):

```html
<div data-slot="usage-meter"><span data-slot="usage-meter-label">Storage</span><div data-slot="usage-meter-fill" style="width:40%"></div></div>
```

Forbidden interact (`progress("usage-meter")`): `<div data-slot="usage-meter">` + `<progress max="100">` and `{ value }` interp. Asserts: no `<progress`, has `usage-meter-fill`.

Chrome: column flex `gap: 0.5rem`; fill `height: 0.5rem` `border-radius: 9999px` `var(--cronus-primary)`.

## masonry

React: `<div data-slot="masonry">` CSS columns container (`packages/ui/src/components/masonry.tsx`). Kernel emits a cell per text item (`data-slot="masonry-cell"`). Not catalog `display("masonry")` SURF `<section>` and not interact `scroll("masonry")` overflow box.

Dedicated DOM (static, zero JS):

```html
<div data-slot="masonry"><div data-slot="masonry-cell">Alpha</div><div data-slot="masonry-cell">Beta</div></div>
```

Each text item is a cell. Label-only still emits one cell.

Forbidden interact (`scroll("masonry")`): `<div data-slot="masonry" style=BASE+SURF max-height:12rem;overflow:auto>`. Forbidden stub: `<section data-slot="masonry">` SURF. Asserts: no `<section`, no `style=`, no `max-height:12rem;overflow:auto`.

Chrome: `column-count: 3` `column-gap: 1rem`; cells `break-inside: avoid`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_segmented_control
  running 8 tests
  test cronus_ui_segmented_control::tests::chrome_is_token_only ... ok
  test cronus_ui_segmented_control::tests::first_item_is_selected_by_default ... ok
  test cronus_ui_segmented_control::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_segmented_control::tests::options_from_text_items ... ok
  test cronus_ui_segmented_control::tests::root_is_tablist_of_segment_items ... ok
  test cronus_ui_segmented_control::tests::selected_true_item_is_active ... ok
  test cronus_ui_segmented_control::tests::skips_interact_native_radios ... ok
  test cronus_ui_segmented_control::tests::value_prop_selects_matching_item ... ok
  test result: ok. 8 passed

cargo test --offline cronus_ui_usage_meter
  running 8 tests
  test cronus_ui_usage_meter::tests::chrome_is_token_only ... ok
  test cronus_ui_usage_meter::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_usage_meter::tests::numeric_label_is_value_not_label ... ok
  test cronus_ui_usage_meter::tests::root_is_div_with_fill_default_40 ... ok
  test cronus_ui_usage_meter::tests::skips_interact_html_progress ... ok
  test cronus_ui_usage_meter::tests::value_from_item_config ... ok
  test cronus_ui_usage_meter::tests::value_from_item_text_number ... ok
  test cronus_ui_usage_meter::tests::value_from_props ... ok
  test result: ok. 8 passed

cargo test --offline cronus_ui_masonry
  running 6 tests
  test cronus_ui_masonry::tests::chrome_is_token_only ... ok
  test cronus_ui_masonry::tests::extra_text_items_become_cells ... ok
  test cronus_ui_masonry::tests::label_only_still_emits_one_cell ... ok
  test cronus_ui_masonry::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_masonry::tests::root_is_masonry_grid_of_cells_not_display_surf ... ok
  test cronus_ui_masonry::tests::skips_interact_scroll_and_display_surf ... ok
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

Also green: `cargo test --offline cronus_ui_widgets` (19 passed), including `ported_family_skips_interact` over the appended list. `voodoo_attrs_only_when_runtime_on` now asserts usage-meter has no `{ value }` / `v-data` (scroll-progress still covers interact progress interp).

Wave total: **29 passed** (8 + 8 + 6 + 7).
