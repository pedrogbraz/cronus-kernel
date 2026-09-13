# Cronus Audit Wave 1a — kbd / toggle / progress

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-c`  
Branch: `feat/wave1a-kbd-toggle-progress`  
Base: `feat/cronus-ui-tokens-button` @ `5e7e8d7`  
Date: 2026-09-13

Done: 3 dedicated modules, interact skipped, tests green. Not pushed.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `1aa2b2e` | feat(ui): dedicated Kbd renderer |
| `2b2f767` | feat(ui): dedicated Toggle renderer |
| `0ea2b2b` | feat(ui): dedicated Progress renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing `button, badge, input`):

```rust
pub const PORTED_FAMILIES: &[&str] = &["button", "badge", "input", "kbd", "toggle", "progress"];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| kbd | `src/cronus_ui_kbd.rs` | `cronus_ui_kbd::render` |
| toggle | `src/cronus_ui_toggle.rs` | `cronus_ui_toggle::render` |
| progress | `src/cronus_ui_progress.rs` | `cronus_ui_progress::render` |

`src/main.rs` mods: `cronus_ui_kbd`, `cronus_ui_progress`, `cronus_ui_toggle`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Stub `pill()` arms remain in the catalog match but never run for these families.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## kbd

React: `<kbd data-slot="kbd">⌘K</kbd>` (`packages/ui/src/components/kbd.tsx`).

Dedicated DOM:

```html
<kbd data-slot="kbd">⌘K</kbd>
```

Forbidden interact (`pill`): `<span data-slot="kbd" style=…>`.

Chrome (`COMPONENT_CHROME`): inline-flex, `h-5` / `min-w-5` (1.25rem), `rounded-md`, border, `surface-overlay`, `px-1.5` (0.375rem), font-mono, `~0.7rem`.

## toggle

React: Radix Toggle = **button**. Does **not** emit `data-variant` or `data-size`.

Dedicated DOM:

```html
<button type="button" data-slot="toggle" data-state="off" aria-pressed="false">Bold</button>
```

Pressed / on from `props.pressed`, item `pressed:true`, or style `+on` → `data-state="on"` `aria-pressed="true"`.

Forbidden interact (`switch()`): `<label data-slot="toggle"><input type="checkbox" data-slot="toggle-control">`.

Chrome: `h-10` (2.5rem) `px-3` (0.75rem) `rounded-lg`; `[data-state=on]` `surface-overlay`.

## progress

React: Radix Root = **div `role=progressbar`**, not HTML `<progress>`.

Dedicated DOM (default value 0):

```html
<div data-slot="progress" role="progressbar" aria-valuenow="0" aria-valuemin="0" aria-valuemax="100"><div data-slot="progress-indicator" style="transform:translateX(-100%)"></div></div>
```

Value from `props.value` / item text number / item config `value`. Default **0** (interact defaulted to 40). Indicator `transform:translateX(-(100-N)%)`.

Forbidden interact: `<div data-slot="progress">…<progress max="100">`.

Chrome: `h-2` (0.5rem) `w-full` `rounded-full` `surface-overlay`; indicator `primary` height 100%.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test cronus_ui_kbd
  running 3 tests
  test cronus_ui_kbd::tests::chrome_is_token_only ... ok
  test cronus_ui_kbd::tests::root_is_kbd_not_span_pill ... ok
  test cronus_ui_kbd::tests::skips_interact_pill ... ok
  test result: ok. 3 passed

cargo test cronus_ui_toggle
  running 6 tests
  test cronus_ui_toggle::tests::chrome_is_token_only ... ok
  test cronus_ui_toggle::tests::pressed_colon_pair_on_item ... ok
  test cronus_ui_toggle::tests::pressed_prop_turns_on ... ok
  test cronus_ui_toggle::tests::root_is_button_not_switch_label ... ok
  test cronus_ui_toggle::tests::skips_interact_switch ... ok
  test cronus_ui_toggle::tests::style_on_turns_on ... ok
  test result: ok. 6 passed

cargo test cronus_ui_progress
  running 6 tests
  test cronus_ui_progress::tests::chrome_is_token_only ... ok
  test cronus_ui_progress::tests::root_is_div_progressbar_not_html_progress ... ok
  test cronus_ui_progress::tests::skips_interact_html_progress ... ok
  test cronus_ui_progress::tests::value_from_item_config ... ok
  test cronus_ui_progress::tests::value_from_item_text_number ... ok
  test cronus_ui_progress::tests::value_from_props ... ok
  test result: ok. 6 passed

cargo test stub_renderer_gate
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

Also green: `cargo test cronus_ui_widgets` (14 passed), including `ported_family_skips_interact` over the appended list. Progress widgets cases now assert `role="progressbar"` instead of HTML `<progress>`; voodoo coverage moved to still-interact `usage-meter`.
