# Cronus Audit Wave 1d — dropdown-menu / collapsible / mode-toggle

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1d-c`  
Branch: `feat/wave1d-dropdown-collapsible-mode`  
Base: `feat/cronus-ui-tokens-button` @ `37caf75`  
Date: 2026-09-13

Done: 3 dedicated modules, interact skipped, tests green. Not pushed. Not touched: combobox, popover, file-dropzone.

Helpers: `src/cronus_ui_kit.rs` (`esc`, `label_of`, `texts`, `choice_texts`). Pattern: `src/cronus_ui_checkbox.rs` (no voodoo). Not interact `popover()` / `accordion()` / `mode_toggle` onclick.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `b844d8f` | feat(ui): dedicated DropdownMenu renderer |
| `480f258` | feat(ui): dedicated Collapsible renderer |
| `ab8fee7` | feat(ui): dedicated ModeToggle renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a/1b/1c list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … button-group …
    "dropdown-menu",
    "collapsible",
    "mode-toggle",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| dropdown-menu | `src/cronus_ui_dropdown_menu.rs` | `cronus_ui_dropdown_menu::render` |
| collapsible | `src/cronus_ui_collapsible.rs` | `cronus_ui_collapsible::render` |
| mode-toggle | `src/cronus_ui_mode_toggle.rs` | `cronus_ui_mode_toggle::render` |

`src/main.rs` mods: `cronus_ui_dropdown_menu`, `cronus_ui_collapsible`, `cronus_ui_mode_toggle`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## dropdown-menu

React: `DropdownMenuContent` `data-slot="dropdown-menu-content"` + `DropdownMenuItem` `data-slot="dropdown-menu-item"` (`packages/ui/src/components/dropdown-menu.tsx`).

Dedicated DOM (always-open static):

```html
<div data-slot="dropdown-menu"><button type="button">Actions</button><div data-slot="dropdown-menu-content" role="menu"><div data-slot="dropdown-menu-item" role="menuitem">Edit</div><div data-slot="dropdown-menu-item" role="menuitem">Share</div></div></div>
```

Trigger button from label. Menu items from `choice_texts` (`item`) or extra text after the label. Label is not a menuitem.

Forbidden interact (`popover("dropdown-menu")`): `<details data-slot="dropdown-menu">` + SURF `position:absolute;z-index:20` box. No `dropdown-menu-content` / `role="menu"`.

Chrome: inline-flex column; content `min-width: 8rem`, `var(--cronus-surface-floating)`, border, `shadow-lg`; items `px/py` on `surface-overlay` hover.

## collapsible

React: `CollapsibleContent` `data-slot="collapsible-content"` (`packages/ui/src/components/collapsible.tsx`).

Dedicated DOM (open static):

```html
<div data-slot="collapsible"><button type="button">Show more</button><div data-slot="collapsible-content" data-state="open">Hidden details here.</div></div>
```

Trigger button from label. Extra text after the label is the open content.

Forbidden interact (`accordion("collapsible")`): `<details>` / `<summary>` SURF box **without** `data-slot="collapsible-content"`.

Chrome: flex column; content `overflow: hidden`, `text-sm`, `var(--cronus-fg-secondary)`; open state `display: block`.

## mode-toggle

React: `<button type="button" data-slot="mode-toggle" data-mode>` + sun/moon SVG (`packages/ui/src/components/mode-toggle.tsx`).

Dedicated DOM (v1 static, default dark, no JS theme flip):

```html
<button type="button" data-slot="mode-toggle" data-mode="dark" aria-label="Switch to light mode">…svg…</button>
```

`data-mode` from `props.mode` / style `+light` / default **dark**. `aria-label` is `Switch to {opposite} mode`. Inner SVG matches React slots (`mode-toggle-core`, `mode-toggle-rays`).

Forbidden interact (`mode_toggle()`): `onclick="document.documentElement.classList.toggle('dark')"` + BASE/SURF inline styles + `Theme` label.

Chrome: `2.25rem` square, `rounded-lg`, `var(--cronus-fg-secondary)`; hover `surface-overlay`; dark mode scales the core and fades the rays.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_dropdown_menu
  running 6 tests
  test cronus_ui_dropdown_menu::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_dropdown_menu::tests::label_only_still_opens_empty_menu ... ok
  test cronus_ui_dropdown_menu::tests::extra_text_items_become_menuitems ... ok
  test cronus_ui_dropdown_menu::tests::skips_interact_popover_surf ... ok
  test cronus_ui_dropdown_menu::tests::chrome_is_token_only ... ok
  test cronus_ui_dropdown_menu::tests::root_is_always_open_menu_not_popover_details ... ok
  test result: ok. 6 passed

cargo test --offline cronus_ui_collapsible
  running 5 tests
  test cronus_ui_collapsible::tests::chrome_is_token_only ... ok
  test cronus_ui_collapsible::tests::label_only_still_opens_empty_content ... ok
  test cronus_ui_collapsible::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_collapsible::tests::root_is_open_content_not_accordion_details ... ok
  test cronus_ui_collapsible::tests::skips_interact_accordion_surf ... ok
  test result: ok. 5 passed

cargo test --offline cronus_ui_mode_toggle
  running 6 tests
  test cronus_ui_mode_toggle::tests::light_from_style ... ok
  test cronus_ui_mode_toggle::tests::light_from_props ... ok
  test cronus_ui_mode_toggle::tests::root_is_static_dark_button_not_onclick_theme ... ok
  test cronus_ui_mode_toggle::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_mode_toggle::tests::skips_interact_classlist_toggle ... ok
  test cronus_ui_mode_toggle::tests::chrome_is_token_only ... ok
  test result: ok. 6 passed

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

Wave total: **24 passed**. Also green: `cargo test --offline cronus_ui_widgets` (19 passed), including `ported_family_skips_interact` and `every_family_renders_slot_without_palette_scales` over the appended list.
