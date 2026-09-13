# Cronus Audit Wave 1n — kanban / json-viewer / animated-number

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1n-b`  
Branch: `feat/wave1n-kanban-json-anum`  
Base: `8c392a2`  
Date: 2026-09-13

Done: 3 dedicated modules, catalog `display()` / `fx()` skipped, interact skipped, tests green. Not pushed.  
Not touched: meteors (harness `Stub("fx")`), sankey-chart, carousel, marquee.

Helpers: `src/cronus_ui_kit.rs` (`texts`, `label_of`, `esc`, `item`, test `stub`). Pattern: `src/cronus_ui_metric.rs` / `src/cronus_ui_data_table.rs` (dedicated slots + CSS chrome). Not catalog `display()` SURF `<section>`. Not catalog `fx()` title SURF box (`padding:0.75rem 1rem;position:relative;overflow:hidden` + inner `<span>`). Not interact generic HTML. Not voodoo.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `5f8bb33` | feat(ui): dedicated Kanban renderer |
| `b27bbce` | feat(ui): dedicated JsonViewer renderer |
| `a008bea` | feat(ui): dedicated AnimatedNumber renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1m list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … toast …
    "kanban",
    "json-viewer",
    "animated-number",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| kanban | `src/cronus_ui_kanban.rs` | `cronus_ui_kanban::render` |
| json-viewer | `src/cronus_ui_json_viewer.rs` | `cronus_ui_json_viewer::render` |
| animated-number | `src/cronus_ui_animated_number.rs` | `cronus_ui_animated_number::render` (not `fx`) |

`src/main.rs` mods: `cronus_ui_kanban`, `cronus_ui_json_viewer`, `cronus_ui_animated_number`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three families.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render` and **before** catalog `fx()` / `display()`. Catalog stub / interact arms remain for unported families and were not rewritten.

133 `PORTED_FAMILIES`. 175 `FAMILIES`. Remaining chart stub: sankey-chart. meteors still `Stub("fx")`.

Fingerprint: dedicated HTML must not contain interact SURF columns / missing `kanban-column`+`kanban-card` (kanban), interact `<pre>` / missing row/key/value (json-viewer), fx box / `<div` / ticker JS (animated-number).

## kanban

React: `<div data-slot="kanban">` wrapping `<section data-slot="kanban-column">` lists of `<li data-slot="kanban-card">` with dnd-kit (`packages/ui/src/components/kanban.tsx`). Kernel is a **static board** from `texts`: one column, each text a card. Zero DnD JS. Not interact `kanban()` inline SURF columns. Not catalog `display()` SURF `<section>`.

Dedicated DOM (static, zero JS):

```html
<div data-slot="kanban"><div data-slot="kanban-column"><div data-slot="kanban-card">Ship login</div><div data-slot="kanban-card">Write docs</div></div></div>
```

Forbidden interact: `<div data-slot="kanban" style="…display:flex;gap:0.75rem;overflow:auto;">` columns with `min-width:10rem` and dummy `Item` (no `kanban-column` / `kanban-card` slots). Forbidden display: `<section data-slot="kanban" style="…padding:1rem;display:flex;flex-direction:column;gap:0.5rem;">`. Asserts: no `<section`, no `style=`, no SURF, no `draggable`.

Chrome: row flex `overflow-x: auto` `gap: 1rem`; column `var(--cronus-surface-inset)` + `var(--cronus-border)`; card `var(--cronus-surface-raised)`.

## json-viewer

React: `<div data-slot="json-viewer">` recursive tree of `json-viewer-row` / `json-viewer-key` / `json-viewer-value` (`packages/ui/src/components/json-viewer.tsx`). Kernel is a **static tree** from texts (`key: value` lines OK). Zero JS (no copy button, no expand). Not interact `codey()` `<pre style=SURF><code>`. Not catalog `display()` SURF `<section>`.

Dedicated DOM (static, zero JS):

```html
<div data-slot="json-viewer"><div data-slot="json-viewer-row"><span data-slot="json-viewer-key">name</span><span data-slot="json-viewer-value">Ada</span></div></div>
```

Forbidden interact: `<pre data-slot="json-viewer" style="…padding:0.85rem 1rem;overflow:auto;"><code>…`. Forbidden display: `<section data-slot="json-viewer" style="…padding:1rem;display:flex;flex-direction:column;gap:0.5rem;">`. Asserts: no `<pre`, no `<code`, no `style=`.

Chrome: mono `var(--cronus-font-mono)` on inset surface; key `var(--cronus-fg-secondary)`; row flex.

## animated-number

React: `<span data-slot="animated-number">` with a motion ticker (`packages/ui/src/components/animated-number.tsx`). Kernel shows a **static formatted number** (value item / props / extra text / bound scalar; fallback `0`). Thousands grouping for plain integers. Zero JS ticker. Not catalog `fx()` SURF title box.

Dedicated DOM (static, zero JS):

```html
<span data-slot="animated-number">1,234</span>
```

Forbidden fx: `<div data-slot="animated-number" style="…padding:0.75rem 1rem;position:relative;overflow:hidden;"><span>Demo</span></div>`. Asserts: no `<div`, no `style=`, no `setInterval` / `requestAnimationFrame`, `dedicated_fn_name("animated-number") == Some("cronus_ui_animated_number::render")`, `renderer_kind("meteors") == Stub("fx")`.

Chrome: `font-variant-numeric: tabular-nums`; `var(--cronus-fg)`; `var(--cronus-font-display)`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_kanban
  running 7 tests
  test cronus_ui_kanban::tests::chrome_is_token_only ... ok
  test cronus_ui_kanban::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_kanban::tests::label_is_escaped ... ok
  test cronus_ui_kanban::tests::root_is_div_of_column_and_cards_not_section ... ok
  test cronus_ui_kanban::tests::label_only_still_emits_one_column_and_card ... ok
  test cronus_ui_kanban::tests::extra_text_items_become_cards ... ok
  test cronus_ui_kanban::tests::skips_interact_surf_columns_and_display_section ... ok
  test result: ok. 7 passed

cargo test --offline cronus_ui_json_viewer
  running 7 tests
  test cronus_ui_json_viewer::tests::chrome_is_token_only ... ok
  test cronus_ui_json_viewer::tests::label_is_escaped ... ok
  test cronus_ui_json_viewer::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_json_viewer::tests::label_without_colon_still_emits_key_slot ... ok
  test cronus_ui_json_viewer::tests::extra_text_items_become_rows ... ok
  test cronus_ui_json_viewer::tests::root_is_div_of_key_value_rows_not_pre_or_section ... ok
  test cronus_ui_json_viewer::tests::skips_interact_pre_and_display_surf ... ok
  test result: ok. 7 passed

cargo test --offline cronus_ui_animated_number
  running 10 tests
  test cronus_ui_animated_number::tests::chrome_is_token_only ... ok
  test cronus_ui_animated_number::tests::already_formatted_value_is_kept ... ok
  test cronus_ui_animated_number::tests::label_is_escaped ... ok
  test cronus_ui_animated_number::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_animated_number::tests::value_item_is_formatted ... ok
  test cronus_ui_animated_number::tests::root_is_span_with_formatted_number_not_fx_title_box ... ok
  test cronus_ui_animated_number::tests::extra_text_becomes_the_number ... ok
  test cronus_ui_animated_number::tests::bound_scalar_when_no_value_sources ... ok
  test cronus_ui_animated_number::tests::skips_fx_surf_title_box ... ok
  test cronus_ui_animated_number::tests::value_from_props ... ok
  test result: ok. 10 passed

cargo test --offline stub_renderer_gate
  running 7 tests
  test cli::stub_renderer_gate::tests::meteors_is_stub_fx ... ok
  test cli::stub_renderer_gate::tests::sankey_chart_is_stub ... ok
  test cli::stub_renderer_gate::tests::button_is_dedicated ... ok
  test cli::stub_renderer_gate::tests::every_family_is_classified ... ok
  test cli::stub_renderer_gate::tests::no_new_stub_families ... ok
  test cli::stub_renderer_gate::tests::ported_families_are_dedicated_and_skip_interact ... ok
  test cli::stub_renderer_gate::tests::dedicated_module_no_sidecar_assets ... ok
  test result: ok. 7 passed
```

`meteors_is_stub_fx` asserts `renderer_kind("meteors") == Stub("fx")` and the catalog fx fingerprint. `sankey_chart_is_stub` remains `Stub("chart")`. Animated-number `dedicated_fn_name` is `cronus_ui_animated_number::render`, not `fx`.

Wave total: **31 passed** (7 + 7 + 10 + 7).
