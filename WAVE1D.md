# Cronus Audit Wave 1d — combobox / stepper / input-otp

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1d-a`  
Branch: `feat/wave1d-combobox-stepper-otp`  
Base: `feat/cronus-ui-tokens-button` @ `37caf75`  
Date: 2026-09-13

Done: 3 dedicated modules, interact skipped, tests green. Not pushed.  
Not touched: file-dropzone, popover, dropdown-menu.

Helpers: `src/cronus_ui_kit.rs` (`choice_texts`, `esc`, `item`, `texts`). Pattern: `src/cronus_ui_checkbox.rs` (no voodoo, no inline BASE/SURF/CTRL). Not copied: `cronus_ui_password_input.rs` / interact `select()`.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `e0bd754` | feat(ui): dedicated Combobox renderer |
| `773b3bd` | feat(ui): dedicated Stepper renderer |
| `e19b93d` | feat(ui): dedicated InputOTP renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a/1b/1c list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … button-group …
    "combobox",
    "stepper",
    "input-otp",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| combobox | `src/cronus_ui_combobox.rs` | `cronus_ui_combobox::render` |
| stepper | `src/cronus_ui_stepper.rs` | `cronus_ui_stepper::render` |
| input-otp | `src/cronus_ui_input_otp.rs` | `cronus_ui_input_otp::render` |

`src/main.rs` mods: `cronus_ui_combobox`, `cronus_ui_stepper`, `cronus_ui_input_otp`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

## combobox

React: `<button data-slot="combobox-trigger" role="combobox" aria-haspopup="listbox">` + `<div data-slot="combobox-content">` (`packages/ui/src/components/combobox.tsx`). Not a native `<select>`.

Dedicated DOM (open, zero JS):

```html
<div data-slot="combobox"><button type="button" data-slot="combobox-trigger" role="combobox" aria-expanded="true" aria-haspopup="listbox">Search</button><div data-slot="combobox-content" role="listbox"><button type="button" data-slot="combobox-item" role="option" aria-selected="false">Ada</button><button type="button" data-slot="combobox-item" role="option" aria-selected="false">Grace</button></div></div>
```

Trigger text is the selected option (`props.value` / item `selected`) or the placeholder (label). Options from `choice_texts` — field labels are not options.

Forbidden interact (`select("combobox")`): `<label data-slot="combobox"><select data-slot="combobox-control">`. Asserts: no `*-control`, no `<select`.

Chrome: full-width trigger (`justify-content: space-between`, `h-10`, outline border); content `min-width: 8rem` on `var(--cronus-surface-floating)`; selected item `surface-overlay`.

## stepper

React: `<div data-slot="stepper" data-orientation>` + item / trigger / title (`packages/ui/src/components/stepper.tsx`).

Dedicated DOM:

```html
<div data-slot="stepper" data-orientation="horizontal"><div data-slot="stepper-item" data-state="current"><button type="button" data-slot="stepper-trigger"><span data-slot="stepper-title">Account</span></button></div><div data-slot="stepper-item" data-state="upcoming">…</div></div>
```

Text items become steps. First item `data-state="current"` (later steps `upcoming`; `props.value` index marks `completed` / `current`). Orientation from props/style (`horizontal` default). No JS.

Forbidden interact (`stepper()`): `<ol data-slot="stepper" style=BASE>` numbered pills — also stub `<nav data-slot="stepper">`.

Chrome: flex row (column when vertical); upcoming titles `var(--cronus-fg-tertiary)`.

## input-otp

React: `data-slot="input-otp"` + `input-otp-group` + `input-otp-slot` (`packages/ui/src/components/input-otp.tsx`). Slots are decorative divs.

Dedicated DOM (6 slots):

```html
<div data-slot="input-otp" role="group" aria-label="One-time passcode"><div data-slot="input-otp-group"><div data-slot="input-otp-slot"></div>×6</div></div>
```

No `<input>`, no inline CTRL. Digit `props.value` fills slots; `length` 1–8 (default 6).

Forbidden interact (`otp()`): `<fieldset data-slot="input-otp" style=BASE>` + `maxlength=1` inputs with CTRL (`width:2.5rem;text-align:center;font-variant-numeric:tabular-nums`).

Chrome: flex group; each slot `2.5rem` square, shared border, first/last radius `var(--cronus-radius-lg)`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_combobox
  running 9 tests
  test cronus_ui_combobox::tests::skips_interact_native_select ... ok
  test cronus_ui_combobox::tests::selected_item_config_marks_option ... ok
  test cronus_ui_combobox::tests::root_is_trigger_and_open_listbox_not_native_select ... ok
  test cronus_ui_combobox::tests::empty_choices_keep_open_listbox ... ok
  test cronus_ui_combobox::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_combobox::tests::disabled_trigger ... ok
  test cronus_ui_combobox::tests::label_is_placeholder_not_an_option ... ok
  test cronus_ui_combobox::tests::value_selects_matching_option ... ok
  test cronus_ui_combobox::tests::chrome_is_token_only ... ok
  test result: ok. 9 passed

cargo test --offline cronus_ui_stepper
  running 9 tests
  test cronus_ui_stepper::tests::vertical_from_style ... ok
  test cronus_ui_stepper::tests::first_item_is_current_by_default ... ok
  test cronus_ui_stepper::tests::vertical_from_props ... ok
  test cronus_ui_stepper::tests::field_label_is_not_a_step ... ok
  test cronus_ui_stepper::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_stepper::tests::value_marks_current_and_completed ... ok
  test cronus_ui_stepper::tests::root_is_div_of_item_triggers_not_ol_or_nav ... ok
  test cronus_ui_stepper::tests::skips_interact_ol_nav ... ok
  test cronus_ui_stepper::tests::chrome_is_token_only ... ok
  test result: ok. 9 passed

cargo test --offline cronus_ui_input_otp
  running 7 tests
  test cronus_ui_input_otp::tests::root_is_group_of_six_slots_not_fieldset ... ok
  test cronus_ui_input_otp::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_input_otp::tests::chrome_is_token_only ... ok
  test cronus_ui_input_otp::tests::length_from_props ... ok
  test cronus_ui_input_otp::tests::aria_label_from_props ... ok
  test cronus_ui_input_otp::tests::value_fills_slots ... ok
  test cronus_ui_input_otp::tests::skips_interact_fieldset_ctrl ... ok
  test result: ok. 7 passed

cargo test --offline stub_renderer_gate
  running 7 tests
  test cli::stub_renderer_gate::tests::meteors_is_stub_fx ... ok
  test cli::stub_renderer_gate::tests::area_chart_is_stub ... ok
  test cli::stub_renderer_gate::tests::button_is_dedicated ... ok
  test cli::stub_renderer_gate::tests::no_new_stub_families ... ok
  test cli::stub_renderer_gate::tests::every_family_is_classified ... ok
  test cli::stub_renderer_gate::tests::ported_families_are_dedicated_and_skip_interact ... ok
  test cli::stub_renderer_gate::tests::dedicated_module_no_sidecar_assets ... ok
  test result: ok. 7 passed
```

Also green: `cargo test --offline cronus_ui_widgets` (19 passed), including `ported_family_skips_interact` over the appended list.

Wave total: **32 passed** (9 + 9 + 7 + 7).
