# Cronus Audit Wave 1c — metric / avatar-group / button-group

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1c-c`  
Branch: `feat/wave1c-metric-avatar-group-button-group`  
Base: `feat/cronus-ui-tokens-button` @ `ef08106`  
Date: 2026-09-13

Done: 3 dedicated modules, interact skipped, tests green. Not pushed.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `70402c7` | feat(ui): dedicated Metric renderer |
| `bdf6e41` | feat(ui): dedicated AvatarGroup renderer |
| `826accf` | feat(ui): dedicated ButtonGroup renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a/1b list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … number-input …
    "metric",
    "avatar-group",
    "button-group",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| metric | `src/cronus_ui_metric.rs` | `cronus_ui_metric::render` |
| avatar-group | `src/cronus_ui_avatar_group.rs` | `cronus_ui_avatar_group::render` |
| button-group | `src/cronus_ui_button_group.rs` | `cronus_ui_button_group::render` |

`src/main.rs` mods: `cronus_ui_metric`, `cronus_ui_avatar_group`, `cronus_ui_button_group`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`. Helpers from `src/cronus_ui_kit.rs`. Pattern is checkbox (static DOM), not password-input / metric interact interp.

## metric

React: `<div data-slot="metric">` (`packages/ui/src/components/metric.tsx`).

Dedicated DOM:

```html
<div data-slot="metric"><div data-slot="metric-label">Revenue</div><div data-slot="metric-value">0</div></div>
```

Label from label/title. Value from item type `value` / extra text / `props.value` (bound scalar last).

Forbidden interact (`metric()`): `<section data-slot="metric" style=SURF>` + v-data `{ value }` interp.

Chrome (`COMPONENT_CHROME`): `flex flex-col gap-1`; label `text-xs font-medium uppercase tracking-wider fg-tertiary`; value `font-display text-2xl font-semibold tabular-nums`.

## avatar-group

React: `<div data-slot="avatar-group">` wrapping avatars (`packages/ui/src/components/avatar-group.tsx`).

Dedicated DOM:

```html
<div data-slot="avatar-group" role="group" aria-label="Avatar group"><span data-slot="avatar"><span data-slot="avatar-fallback">JD</span></span><span data-slot="avatar"><span data-slot="avatar-fallback">AL</span></span></div>
```

2–3 `<span data-slot="avatar">` from text items (initials). Optional overflow `+N` span `data-slot="avatar-group-overflow"` when `max` < total.

Forbidden interact (`avatar("avatar-group")`): single `<div data-slot="avatar-group" style=circle>` one letter.

Chrome: overlapping `size-9` (2.25rem) rings (`-space-x-2`) on `surface-base`; overflow chip `surface-overlay`.

## button-group

React: `<div data-slot="button-group" role="group" data-orientation>` (`packages/ui/src/components/button-group.tsx`).

Dedicated DOM:

```html
<div data-slot="button-group" role="group" data-orientation="horizontal"><button type="button" data-slot="button">Edit</button><button type="button" data-slot="button">Share</button></div>
```

Child buttons from text items. Orientation from props/style (`horizontal` default).

Forbidden interact (`buttonish()`): wrapper with a SINGLE primary button and inline styles.

Chrome: `inline-flex`; joined radii / `-1px` overlap; `focus-visible` raises z-index.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test cronus_ui_metric
  running 10 tests
  test cronus_ui_metric::tests::chrome_is_token_only ... ok
  test cronus_ui_metric::tests::value_from_value_item ... ok
  test cronus_ui_metric::tests::value_from_extra_text ... ok
  test cronus_ui_metric::tests::value_from_props ... ok
  test cronus_ui_metric::tests::value_item_wins_over_extra_and_props ... ok
  test cronus_ui_metric::tests::bound_scalar_when_no_value_sources ... ok
  test cronus_ui_metric::tests::label_from_title_item ... ok
  test cronus_ui_metric::tests::root_is_div_with_label_and_value_not_section ... ok
  test cronus_ui_metric::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_metric::tests::skips_interact_section ... ok
  test result: ok. 10 passed

cargo test cronus_ui_avatar_group
  running 7 tests
  test cronus_ui_avatar_group::tests::chrome_is_token_only ... ok
  test cronus_ui_avatar_group::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_avatar_group::tests::three_text_items_are_three_avatars ... ok
  test cronus_ui_avatar_group::tests::initials_from_single_word ... ok
  test cronus_ui_avatar_group::tests::overflow_when_max_below_total ... ok
  test cronus_ui_avatar_group::tests::root_wraps_avatars_not_single_circle ... ok
  test cronus_ui_avatar_group::tests::skips_interact_circle ... ok
  test result: ok. 7 passed

cargo test cronus_ui_button_group
  running 6 tests
  test cronus_ui_button_group::tests::chrome_is_token_only ... ok
  test cronus_ui_button_group::tests::vertical_from_props ... ok
  test cronus_ui_button_group::tests::vertical_from_style ... ok
  test cronus_ui_button_group::tests::root_is_group_of_buttons_not_single_buttonish ... ok
  test cronus_ui_button_group::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_button_group::tests::skips_interact_buttonish ... ok
  test result: ok. 6 passed

cargo test stub_renderer_gate
  running 7 tests
  test cli::stub_renderer_gate::tests::meteors_is_stub_fx ... ok
  test cli::stub_renderer_gate::tests::area_chart_is_stub ... ok
  test cli::stub_renderer_gate::tests::button_is_dedicated ... ok
  test cli::stub_renderer_gate::tests::no_new_stub_families ... ok
  test cli::stub_renderer_gate::tests::ported_families_are_dedicated_and_skip_interact ... ok
  test cli::stub_renderer_gate::tests::every_family_is_classified ... ok
  test cli::stub_renderer_gate::tests::dedicated_module_no_sidecar_assets ... ok
  test result: ok. 7 passed
```

Also green: `cargo test cronus_ui_widgets` (17 passed), including `ported_family_skips_interact` and `metric_reads_bound_count` over the appended list.
