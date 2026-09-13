# Cronus Audit Wave 1c — field / input-group / rating

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1c-a`  
Branch: `feat/wave1c-field-input-group-rating`  
Base: `feat/cronus-ui-tokens-button` @ `ef08106`  
Date: 2026-09-13

Dedicated CONTRACT renderers. Interact skipped via `PORTED_FAMILIES`. Chrome is `var(--cronus-*)` only. Zero JS, zero Voodoo attrs, no HTML in `.cronus`. Not pushed. Not touched: copy-button, fab, metric, toggle-group, avatar-group, button-group.

Helpers: `src/cronus_ui_kit.rs` (`esc`, `label_of`, `texts`, `item`). Pattern: `src/cronus_ui_checkbox.rs` (no voodoo). Not copied: `cronus_ui_password_input.rs` (still emits `v-data`).

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `9bc0e0d` | feat(ui): dedicated Field renderer |
| `dff6c4c` | feat(ui): dedicated InputGroup renderer |
| `0feaab7` | feat(ui): dedicated Rating renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing families):

```rust
"password-input", "number-input", "field", "input-group", "rating",
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| field | `src/cronus_ui_field.rs` | `cronus_ui_field::render` |
| input-group | `src/cronus_ui_input_group.rs` | `cronus_ui_input_group::render` |
| rating | `src/cronus_ui_rating.rs` | `cronus_ui_rating::render` |

`src/main.rs` mods: `cronus_ui_field`, `cronus_ui_input_group`, `cronus_ui_rating`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
`stub_renderer_gate` lists the new modules in `dedicated_module_no_sidecar_assets`. Catalog `include_str!("…app.cronus")` in tests is allowed (declaration source, not a CSS/JS sidecar).

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`.

## field

React: `<div data-slot="field">` + `<label data-slot="field-label">` + `<p data-slot="field-description">` (`packages/ui/src/components/field.tsx`).

Dedicated DOM:

```html
<div data-slot="field"><label data-slot="field-label">Email</label><p data-slot="field-description">We'll never share this.</p></div>
```

Label from `texts` / `label_of`. Description from extra text items after the label. No control inside the field.

Forbidden interact (`field_form()`): `<form data-slot="field">` stacking `<label>…<input style=CTRL>` without `field-label`.

Chrome: flex column, `gap: 0.375rem`; label `text-sm` / `font-weight: 500`; description `text-xs` / `var(--cronus-fg-secondary)`.

## input-group

React: `<div data-slot="input-group">` + addon + nested `<input data-slot="input">` (`packages/ui/src/components/input-group.tsx`).

Dedicated DOM:

```html
<div data-slot="input-group"><span data-slot="input-group-addon">https://</span><input data-slot="input" type="text" placeholder="example.com" /></div>
```

Addon from `props.addon` or first extra item (label used as prefix when there is no extra). Placeholder from `text` / `props.placeholder`.

Forbidden interact (`input("input-group")`): `<label data-slot="input-group"><input data-slot="input-group-control">`.

Chrome: flex `h-10` (`2.5rem`) stretch, `overflow: hidden`, `rounded-lg` border, `var(--cronus-surface-inset)`; addon `px-3` / `var(--cronus-fg-tertiary)` / border-r; inner `[data-slot=input]` borderless transparent.

## rating

React: `<div data-slot="rating" role="slider">` + pointer-only `<span data-slot="rating-item" aria-hidden="true">` (`packages/ui/src/components/rating.tsx`). Not radios.

Dedicated DOM (default value 0):

```html
<div data-slot="rating" role="slider" aria-valuemin="0" aria-valuemax="5" aria-valuenow="0" aria-label="Rating"><span data-slot="rating-item" aria-hidden="true" data-state="off"></span>×5</div>
```

Value from `props.value` / item number, default **0**, clamped 0–5. First N items `data-state="on"`. Stars drawn in CSS (`::before { content: "★" }`) — HTML has no radio inputs and no `★` text.

Forbidden interact: `role="radiogroup"` + hidden `<input type="radio">` stars.

Chrome: inline-flex, `gap: 0.25rem`; items `1.25rem`; on-state `var(--cronus-warning, var(--cronus-primary))`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_field
  running 4 tests
  test cronus_ui_field::tests::chrome_is_token_only ... ok
  test cronus_ui_field::tests::description_from_extra_text ... ok
  test cronus_ui_field::tests::root_is_div_with_field_label_not_form_stack ... ok
  test cronus_ui_field::tests::skips_interact_field_form ... ok
  test result: ok. 4 passed

cargo test --offline cronus_ui_input_group
  running 7 tests
  test cronus_ui_input_group::tests::addon_from_first_extra_item ... ok
  test cronus_ui_input_group::tests::addon_from_props ... ok
  test cronus_ui_input_group::tests::chrome_is_token_only ... ok
  test cronus_ui_input_group::tests::placeholder_from_props ... ok
  test cronus_ui_input_group::tests::placeholder_from_text_item ... ok
  test cronus_ui_input_group::tests::root_is_div_with_input_slot_not_label_control ... ok
  test cronus_ui_input_group::tests::skips_interact_label_control ... ok
  test result: ok. 7 passed

cargo test --offline cronus_ui_rating
  running 6 tests
  test cronus_ui_rating::tests::chrome_is_token_only ... ok
  test cronus_ui_rating::tests::root_is_slider_of_spans_not_radiogroup ... ok
  test cronus_ui_rating::tests::skips_interact_radio_stars ... ok
  test cronus_ui_rating::tests::value_defaults_to_zero ... ok
  test cronus_ui_rating::tests::value_from_item_number ... ok
  test cronus_ui_rating::tests::value_from_props ... ok
  test result: ok. 6 passed

cargo test --offline stub_renderer_gate
  running 7 tests
  test cli::stub_renderer_gate::tests::area_chart_is_stub ... ok
  test cli::stub_renderer_gate::tests::button_is_dedicated ... ok
  test cli::stub_renderer_gate::tests::dedicated_module_no_sidecar_assets ... ok
  test cli::stub_renderer_gate::tests::every_family_is_classified ... ok
  test cli::stub_renderer_gate::tests::meteors_is_stub_fx ... ok
  test cli::stub_renderer_gate::tests::no_new_stub_families ... ok
  test cli::stub_renderer_gate::tests::ported_families_are_dedicated_and_skip_interact ... ok
  test result: ok. 7 passed
```

Wave total: **24 passed**.
