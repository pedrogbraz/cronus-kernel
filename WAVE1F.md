# Cronus Audit Wave 1f — sidebar / sonner / navigation-menu

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1f-c`  
Branch: `feat/wave1f-sidebar-sonner-nav`  
Base: `4630a19`  
Date: 2026-09-13

Done: 3 dedicated modules, interact skipped, tests green. Not pushed.  
Not touched: charts, data-table.

Helpers: `src/cronus_ui_kit.rs` (`choice_texts`, `esc`, `item`, `texts`, `label_of`). Pattern: `src/cronus_ui_checkbox.rs` (no voodoo, no inline BASE/SURF/CTRL). Not interact `nav()` / `popover()`.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `412e6c3` | feat(ui): dedicated Sidebar renderer |
| `ae3be1f` | feat(ui): dedicated Sonner renderer |
| `f069f56` | feat(ui): dedicated NavigationMenu renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1e list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … date-range-picker …
    "sidebar",
    "sonner",
    "navigation-menu",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| sidebar | `src/cronus_ui_sidebar.rs` | `cronus_ui_sidebar::render` |
| sonner | `src/cronus_ui_sonner.rs` | `cronus_ui_sonner::render` |
| navigation-menu | `src/cronus_ui_navigation_menu.rs` | `cronus_ui_navigation_menu::render` |

`src/main.rs` mods: `cronus_ui_sidebar`, `cronus_ui_sonner`, `cronus_ui_navigation_menu`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

## sidebar

React desktop: `<aside data-slot="sidebar">` plus `<div data-slot="sidebar-content">` and `<ul data-slot="sidebar-menu">` (`packages/ui/src/components/sidebar.tsx`). Menu items are `<li data-slot="sidebar-menu-item">` with `<a data-slot="sidebar-menu-button">` (asChild). Not a generic `<nav data-slot="sidebar">`.

Dedicated DOM (static, zero JS):

```html
<aside data-slot="sidebar"><div data-slot="sidebar-content"><ul data-slot="sidebar-menu"><li data-slot="sidebar-menu-item"><a data-slot="sidebar-menu-button" href="#">Home</a></li><li data-slot="sidebar-menu-item"><a data-slot="sidebar-menu-button" href="#">Settings</a></li></ul></div></aside>
```

Items from `choice_texts` — field labels are not menu buttons when `item` choices exist. Label-only still emits one menu item.

Forbidden interact (`nav("sidebar")`): `<nav data-slot="sidebar">` SURF box without sidebar-content/menu. Asserts: no `<nav data-slot="sidebar"`, no `style=`, no `flex-wrap:wrap`.

Chrome: column `width: 16rem` on `var(--cronus-surface-base)` with `var(--cronus-border)`; menu-button hover `surface-inset`.

## sonner

React Toaster: `<div data-slot="toaster">` wrapping Sonner (`packages/ui/src/components/sonner.tsx`). Family name is still `sonner`, so the kernel also emits `data-slot="sonner"`.

Dedicated DOM (one sample toast from the label):

```html
<div data-slot="sonner"><div data-slot="toaster"><div data-slot="toast" role="status" aria-live="polite">Saved</div></div></div>
```

Forbidden interact (`popover("sonner")`): `<details data-slot="sonner">` SURF box. Asserts: no `<details`, no `*-control`.

Chrome: fixed `z-index: 50` host; toast on `var(--cronus-surface-floating)` with `var(--cronus-border)`.

## navigation-menu

React: `<nav data-slot="navigation-menu">` + `<ul data-slot="navigation-menu-list">` + `<li data-slot="navigation-menu-item">` + `<button data-slot="navigation-menu-trigger">` + `<div data-slot="navigation-menu-content">` (`packages/ui/src/components/navigation-menu.tsx`).

Dedicated DOM (always-open first item):

```html
<nav data-slot="navigation-menu"><ul data-slot="navigation-menu-list"><li data-slot="navigation-menu-item"><button type="button" data-slot="navigation-menu-trigger" data-state="open">Products</button><div data-slot="navigation-menu-content">Products</div></li><li data-slot="navigation-menu-item"><button type="button" data-slot="navigation-menu-trigger">Docs</button></li></ul></nav>
```

Each text item is a trigger. First trigger is `data-state="open"` and is followed by always-open content. Field labels are not triggers when `item` choices exist.

Forbidden interact (`nav("navigation-menu")`): generic SURF `<nav>` without list/trigger/content. Asserts: no `style=`, no `flex-wrap:wrap`.

Chrome: flex row list; open trigger `surface-overlay`; content `min-width: 12rem` on `var(--cronus-surface-floating)`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_sidebar
  running 7 tests
  test cronus_ui_sidebar::tests::chrome_is_token_only ... ok
  test cronus_ui_sidebar::tests::extra_text_items_become_menu_buttons ... ok
  test cronus_ui_sidebar::tests::field_label_is_not_a_button_when_items_exist ... ok
  test cronus_ui_sidebar::tests::label_only_still_emits_one_item ... ok
  test cronus_ui_sidebar::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_sidebar::tests::root_is_aside_with_content_and_menu_not_nav_surf ... ok
  test cronus_ui_sidebar::tests::skips_interact_nav_surf ... ok
  test result: ok. 7 passed

cargo test --offline cronus_ui_sonner
  running 6 tests
  test cronus_ui_sonner::tests::chrome_is_token_only ... ok
  test cronus_ui_sonner::tests::extra_text_does_not_add_toasts ... ok
  test cronus_ui_sonner::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_sonner::tests::root_is_sonner_toaster_toast_not_popover_details ... ok
  test cronus_ui_sonner::tests::skips_interact_popover_surf ... ok
  test cronus_ui_sonner::tests::toast_text_from_label ... ok
  test result: ok. 6 passed

cargo test --offline cronus_ui_navigation_menu
  running 7 tests
  test cronus_ui_navigation_menu::tests::chrome_is_token_only ... ok
  test cronus_ui_navigation_menu::tests::extra_text_items_become_triggers ... ok
  test cronus_ui_navigation_menu::tests::field_label_is_not_a_trigger_when_items_exist ... ok
  test cronus_ui_navigation_menu::tests::label_only_still_opens_first_content ... ok
  test cronus_ui_navigation_menu::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_navigation_menu::tests::root_is_nav_list_with_open_first_content ... ok
  test cronus_ui_navigation_menu::tests::skips_interact_nav_surf ... ok
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

Wave total: **27 passed** (7 + 6 + 7 + 7).
