# Cronus Audit Wave 1e — command / menubar / context-menu

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1e-a`  
Branch: `feat/wave1e-command-menubar-context`  
Base: `feat/cronus-ui-tokens-button` @ `7ca631b`  
Date: 2026-09-13

Done: 3 dedicated modules, interact skipped, tests green. Not pushed.  
Not touched: drawer, sheet, date-picker.

Helpers: `src/cronus_ui_kit.rs` (`choice_texts`, `esc`, `item`, `texts`, `label_of`). Pattern: `src/cronus_ui_checkbox.rs` (no voodoo, no inline BASE/SURF/CTRL). Not interact `popover()`.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `80bf953` | feat(ui): dedicated Command renderer |
| `23477f1` | feat(ui): dedicated Menubar renderer |
| `70642c4` | feat(ui): dedicated ContextMenu renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1d list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … mode-toggle …
    "command",
    "menubar",
    "context-menu",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| command | `src/cronus_ui_command.rs` | `cronus_ui_command::render` |
| menubar | `src/cronus_ui_menubar.rs` | `cronus_ui_menubar::render` |
| context-menu | `src/cronus_ui_context_menu.rs` | `cronus_ui_context_menu::render` |

`src/main.rs` mods: `cronus_ui_command`, `cronus_ui_menubar`, `cronus_ui_context_menu`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

## command

React: `<div data-slot="command">` + `<input data-slot="command-input">` + `<div data-slot="command-list">` + `<div data-slot="command-item">` (`packages/ui/src/components/command.tsx`). Not a `<details>` overlay.

Dedicated DOM (static, zero JS):

```html
<div data-slot="command"><input data-slot="command-input" type="text" placeholder="Search" /><div data-slot="command-list"><div data-slot="command-item">Calendar</div><div data-slot="command-item">Profile</div></div></div>
```

Placeholder is `props.placeholder` or the field label. Items from `choice_texts` — field labels are not command-items.

Forbidden interact (`popover("command")`): `<details data-slot="command">` SURF box. Asserts: no `<details`, no `*-control`.

Chrome: column on `var(--cronus-surface-floating)`; input `h-10` with bottom `var(--cronus-border)`; list `max-height: 20rem`; item hover `surface-overlay`.

## menubar

React: `<div data-slot="menubar">` + `<button data-slot="menubar-trigger">` + `<div data-slot="menubar-content">` + `<div data-slot="menubar-item">` (`packages/ui/src/components/menubar.tsx`).

Dedicated DOM (always-open first menu):

```html
<div data-slot="menubar" role="menubar"><button type="button" data-slot="menubar-trigger" data-state="open">File</button><div data-slot="menubar-content" role="menu"><div data-slot="menubar-item" role="menuitem">File</div></div><button type="button" data-slot="menubar-trigger">Edit</button></div>
```

Each text item is a trigger. First trigger is `data-state="open"` and is followed by always-open content with one menubar-item. Field labels are not triggers when `item` choices exist.

Forbidden interact (`popover("menubar")`): `<details data-slot="menubar">` SURF box. Asserts: no `<details`, no `*-control`.

Chrome: flex row, `var(--cronus-surface-floating)` bar; open trigger `surface-overlay`; content `min-width: 12rem`.

## context-menu

React: content slot is the contract (`data-slot="context-menu-content"`); Root emits no data-slot (`packages/ui/src/components/context-menu.tsx`). Audit cannot right-click.

Dedicated DOM (always-open):

```html
<button type="button">Surface</button><div data-slot="context-menu-content" role="menu"><div data-slot="context-menu-item" role="menuitem">Cut</div><div data-slot="context-menu-item" role="menuitem">Copy</div></div>
```

Optional trigger button from the label. Items from `choice_texts` — field labels are not menuitems.

Forbidden interact (`popover("context-menu")`): `<details data-slot="context-menu">` SURF box. Asserts: no `<details`, no `*-control`.

Chrome: content `min-width: 8rem` on `var(--cronus-surface-floating)`; item hover `surface-overlay`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_command
  running 8 tests
  test cronus_ui_command::tests::chrome_is_token_only ... ok
  test cronus_ui_command::tests::extra_text_items_become_command_items ... ok
  test cronus_ui_command::tests::label_is_placeholder_not_an_item ... ok
  test cronus_ui_command::tests::label_only_keeps_empty_list ... ok
  test cronus_ui_command::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_command::tests::placeholder_from_props ... ok
  test cronus_ui_command::tests::root_is_input_and_list_not_popover_details ... ok
  test cronus_ui_command::tests::skips_interact_popover_surf ... ok
  test result: ok. 8 passed

cargo test --offline cronus_ui_menubar
  running 7 tests
  test cronus_ui_menubar::tests::chrome_is_token_only ... ok
  test cronus_ui_menubar::tests::extra_text_items_become_triggers ... ok
  test cronus_ui_menubar::tests::field_label_is_not_a_trigger_when_items_exist ... ok
  test cronus_ui_menubar::tests::label_only_still_opens_first_menu ... ok
  test cronus_ui_menubar::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_menubar::tests::root_is_menubar_with_always_open_first_menu ... ok
  test cronus_ui_menubar::tests::skips_interact_popover_surf ... ok
  test result: ok. 7 passed

cargo test --offline cronus_ui_context_menu
  running 7 tests
  test cronus_ui_context_menu::tests::always_open_content_not_popover_details ... ok
  test cronus_ui_context_menu::tests::chrome_is_token_only ... ok
  test cronus_ui_context_menu::tests::content_slot_is_the_contract ... ok
  test cronus_ui_context_menu::tests::extra_text_items_become_menuitems ... ok
  test cronus_ui_context_menu::tests::label_only_still_opens_empty_menu ... ok
  test cronus_ui_context_menu::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_context_menu::tests::skips_interact_popover_surf ... ok
  test result: ok. 7 passed

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

Also green: `cargo test --offline cronus_ui_widgets` (19 passed), including `ported_family_skips_interact` over the appended list.

Wave total: **29 passed** (8 + 7 + 7 + 7).
