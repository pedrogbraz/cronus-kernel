# Cronus Audit Wave 1g — scroll-area / toolbar / status-dot

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1g-c`  
Branch: `feat/wave1g-scroll-toolbar-dot`  
Base: `91df802`  
Date: 2026-09-13

Done: 3 dedicated modules, interact skipped, tests green. Not pushed.  
Not touched: charts, phone.

Helpers: `src/cronus_ui_kit.rs` (`choice_texts`, `esc`, `item`, `texts`, `label_of`). Pattern: `src/cronus_ui_checkbox.rs` (no voodoo, no inline BASE/SURF/CTRL). Not interact `nav()` / `pill()`.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `8fcbe8a` | feat(ui): dedicated ScrollArea renderer |
| `4a1f3e3` | feat(ui): dedicated Toolbar renderer |
| `dddbe4f` | feat(ui): dedicated StatusDot renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1f list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … navigation-menu …
    "scroll-area",
    "toolbar",
    "status-dot",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| scroll-area | `src/cronus_ui_scroll_area.rs` | `cronus_ui_scroll_area::render` |
| toolbar | `src/cronus_ui_toolbar.rs` | `cronus_ui_toolbar::render` |
| status-dot | `src/cronus_ui_status_dot.rs` | `cronus_ui_status_dot::render` |

`src/main.rs` mods: `cronus_ui_scroll_area`, `cronus_ui_toolbar`, `cronus_ui_status_dot`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

## scroll-area

React: `<div data-slot="scroll-area">` wrapping a keyboard-focusable Viewport of children, plus `<div data-slot="scroll-bar">` (`packages/ui/src/components/scroll-area.tsx`). Not a generic display/nav SURF box.

Dedicated DOM (static, zero JS):

```html
<div data-slot="scroll-area"><div data-slot="scroll-area-viewport" tabindex="0"><div>Alpha</div><div>Beta</div></div><div data-slot="scroll-bar"></div></div>
```

Viewport children come from `texts`. Label-only still emits one child. Scroll-bar is always present (React always mounts `ScrollBar`).

Forbidden interact (`scroll("scroll-area")` / catalog `display()`): SURF box without viewport/scroll-bar (`max-height:12rem;overflow:auto` or `<section data-slot="scroll-area">`). Asserts: no `<section`, no `style=`, no `nav(`.

Chrome: root `position: relative; overflow: hidden`; viewport `overflow: auto`; bar thumb `var(--cronus-border)`.

## toolbar

React: `<div data-slot="toolbar" role="toolbar">` plus each control `<button data-slot="toolbar-button">` (`packages/ui/src/components/toolbar.tsx`). Not a generic `nav("toolbar")` SURF.

Dedicated DOM (static, zero JS):

```html
<div data-slot="toolbar" role="toolbar"><button type="button" data-slot="toolbar-button">Bold</button><button type="button" data-slot="toolbar-button">Italic</button></div>
```

Each text item is a toolbar-button. Field labels are not buttons when `item` choices exist. Label-only still emits one button.

Forbidden interact (`nav("toolbar")`): `<nav data-slot="toolbar">` SURF box without toolbar-button. Asserts: no `<nav data-slot="toolbar"`, no `style=`, no `flex-wrap:wrap`.

Chrome: `inline-flex` bar on `var(--cronus-surface-raised)` with `var(--cronus-border)`; buttons `height: 2rem`; hover `surface-overlay`.

## status-dot

React: `<span data-slot="status-dot" role="status" data-status>` plus `<span data-slot="status-dot-indicator">` and a label slot (`packages/ui/src/components/status-dot.tsx`). Kernel always emits the visible `status-dot-label` from the component label.

Dedicated DOM (static, zero JS):

```html
<span data-slot="status-dot" role="status" data-status="online"><span data-slot="status-dot-indicator"></span><span data-slot="status-dot-label">Online</span></span>
```

Default `data-status="online"`. Status from `props.status` or style segment (`status-dot+offline`). Label from `label_of`.

Forbidden interact (`pill("status-dot")`): `<span data-slot="status-dot" style=…BASE SURF padding:0.15rem 0.55rem…>` without indicator/label slots. Asserts: no `style=`, no pill padding.

Chrome: `inline-flex` with `9999px` indicator; fill `var(--cronus-success)` (offline outline `var(--cronus-fg-muted)`); label `var(--cronus-fg-secondary)`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_scroll_area
  running 6 tests
  test cronus_ui_scroll_area::tests::chrome_is_token_only ... ok
  test cronus_ui_scroll_area::tests::extra_text_items_become_viewport_children ... ok
  test cronus_ui_scroll_area::tests::label_only_still_emits_viewport_child ... ok
  test cronus_ui_scroll_area::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_scroll_area::tests::root_is_scroll_area_with_viewport_not_display_surf ... ok
  test cronus_ui_scroll_area::tests::skips_interact_scroll_surf ... ok
  test result: ok. 6 passed

cargo test --offline cronus_ui_toolbar
  running 7 tests
  test cronus_ui_toolbar::tests::chrome_is_token_only ... ok
  test cronus_ui_toolbar::tests::extra_text_items_become_buttons ... ok
  test cronus_ui_toolbar::tests::field_label_is_not_a_button_when_items_exist ... ok
  test cronus_ui_toolbar::tests::label_only_still_emits_one_button ... ok
  test cronus_ui_toolbar::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_toolbar::tests::root_is_toolbar_with_buttons_not_nav_surf ... ok
  test cronus_ui_toolbar::tests::skips_interact_nav_surf ... ok
  test result: ok. 7 passed

cargo test --offline cronus_ui_status_dot
  running 6 tests
  test cronus_ui_status_dot::tests::chrome_is_token_only ... ok
  test cronus_ui_status_dot::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_status_dot::tests::root_is_status_dot_with_indicator_and_label ... ok
  test cronus_ui_status_dot::tests::skips_interact_pill ... ok
  test cronus_ui_status_dot::tests::status_from_props ... ok
  test cronus_ui_status_dot::tests::status_from_style ... ok
  test result: ok. 6 passed

cargo test --offline stub_renderer_gate
  running 7 tests
  test cli::stub_renderer_gate::tests::button_is_dedicated ... ok
  test cli::stub_renderer_gate::tests::dedicated_module_no_sidecar_assets ... ok
  test cli::stub_renderer_gate::tests::every_family_is_classified ... ok
  test cli::stub_renderer_gate::tests::meteors_is_stub_fx ... ok
  test cli::stub_renderer_gate::tests::no_new_stub_families ... ok
  test cli::stub_renderer_gate::tests::ported_families_are_dedicated_and_skip_interact ... ok
  test cli::stub_renderer_gate::tests::radar_chart_is_stub ... ok
  test result: ok. 7 passed
```

Also green: `cargo test --offline cronus_ui_widgets` (19 passed), including `ported_family_skips_interact` over the appended list.

Wave total: **26 passed** (6 + 7 + 6 + 7).
