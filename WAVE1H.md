# Cronus Audit Wave 1h — pill-nav / dock / workspace-switcher

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1h-c`  
Branch: `feat/wave1h-pill-dock-workspace`  
Base: `53428e2`  
Date: 2026-09-13

Done: 3 dedicated modules, interact skipped, tests green. Not pushed.  
Not touched: tags, credit-card-input.

Helpers: `src/cronus_ui_kit.rs` (`choice_texts`, `esc`, `label_of`, `texts`). Pattern: `src/cronus_ui_checkbox.rs` (no voodoo, no inline BASE/SURF/CTRL). Not interact `nav()` / `popover()`.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `d2bd09c` | feat(ui): dedicated PillNav renderer |
| `8903279` | feat(ui): dedicated Dock renderer |
| `353558e` | feat(ui): dedicated WorkspaceSwitcher renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1g list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … status-dot …
    "pill-nav",
    "dock",
    "workspace-switcher",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| pill-nav | `src/cronus_ui_pill_nav.rs` | `cronus_ui_pill_nav::render` |
| dock | `src/cronus_ui_dock.rs` | `cronus_ui_dock::render` |
| workspace-switcher | `src/cronus_ui_workspace_switcher.rs` | `cronus_ui_workspace_switcher::render` |

`src/main.rs` mods: `cronus_ui_pill_nav`, `cronus_ui_dock`, `cronus_ui_workspace_switcher`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

79 `PORTED_FAMILIES`. Remaining chart stub: sankey-chart. meteors still fx stub.

## pill-nav

React: `<nav data-slot="pill-nav">` plus each control `<button data-slot="pill-nav-item">` with the first selected as `aria-current="page"` (`packages/ui/src/components/pill-nav.tsx`). Kernel also emits `<a data-slot="pill-nav-item">` when the item has a link. Not a generic `nav("pill-nav")` SURF without pill-nav-item.

Dedicated DOM (static, zero JS):

```html
<nav data-slot="pill-nav"><button type="button" data-slot="pill-nav-item" aria-current="page">Home</button><button type="button" data-slot="pill-nav-item">Docs</button></nav>
```

Each text item is a pill-nav-item. First is current. Field labels are not items when `item` choices exist. Label-only still emits one current item. Linked items become `<a href>`.

Forbidden interact (`nav("pill-nav")`): `<nav data-slot="pill-nav">` SURF box without pill-nav-item (`flex-wrap:wrap;gap:0.25rem`). Asserts: no `style=`, no `flex-wrap:wrap`.

Chrome: `inline-flex` pill track on `var(--cronus-surface-inset)` with `var(--cronus-border)` and `border-radius: 9999px`; current item `var(--cronus-surface-floating)`.

## dock

React: icon dock with `data-slot="dock"` and each entry `data-slot="dock-item"` as `<a>` when `href` is set, otherwise a button (`packages/ui/src/components/dock.tsx`). Kernel root is `<nav data-slot="dock">` (not generic SURF). Not a generic `nav("dock")` SURF without dock-item.

Dedicated DOM (static, zero JS):

```html
<nav data-slot="dock"><button type="button" data-slot="dock-item" title="Home" aria-label="Home">Home</button><button type="button" data-slot="dock-item" title="Search" aria-label="Search">Search</button></nav>
```

Each text item is a dock-item. Field labels are not items when `item` choices exist. Label-only still emits one item. Linked items become `<a href>`.

Forbidden interact (`nav("dock")`): `<nav data-slot="dock">` SURF box without dock-item. Asserts: no `style=`, no `flex-wrap:wrap`.

Chrome: `inline-flex` bar on `var(--cronus-surface-raised)` with `var(--cronus-border)`; items `2.75rem` square on `var(--cronus-surface-overlay)`.

## workspace-switcher

React: trigger `<button data-slot="workspace-switcher">` plus dropdown content `<div data-slot="workspace-switcher-content">` (`packages/ui/src/components/workspace-switcher.tsx`). Kernel is always-open (no JS). Family slot is on the trigger (no extra wrapper required for the family test). Not interact `nav("workspace-switcher")` SURF and not `popover()` `<details>`.

Dedicated DOM (static, zero JS):

```html
<button type="button" data-slot="workspace-switcher" aria-label="Switch workspace, Acme">Acme</button><div data-slot="workspace-switcher-content"><div data-slot="workspace-switcher-item">Acme</div><div data-slot="workspace-switcher-item">Globex</div></div>
```

Trigger is the first workspace. Content lists every text item. Field labels are not workspaces when `item` choices exist. Label-only still opens content with one item.

Forbidden interact (`nav("workspace-switcher")`): `<nav data-slot="workspace-switcher">` SURF box without workspace-switcher-content. Asserts: no `<nav`, no `<details`, no `style=`, no `flex-wrap:wrap`.

Chrome: trigger `flex` row; content `min-width: 14rem` on `var(--cronus-surface-floating)` with `var(--cronus-border)`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_pill_nav
  running 8 tests
  test cronus_ui_pill_nav::tests::chrome_is_token_only ... ok
  test cronus_ui_pill_nav::tests::extra_text_items_become_items ... ok
  test cronus_ui_pill_nav::tests::field_label_is_not_an_item_when_items_exist ... ok
  test cronus_ui_pill_nav::tests::label_only_still_emits_current_item ... ok
  test cronus_ui_pill_nav::tests::linked_item_is_anchor ... ok
  test cronus_ui_pill_nav::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_pill_nav::tests::root_is_nav_with_items_first_current_not_surf ... ok
  test cronus_ui_pill_nav::tests::skips_interact_nav_surf ... ok
  test result: ok. 8 passed

cargo test --offline cronus_ui_dock
  running 8 tests
  test cronus_ui_dock::tests::chrome_is_token_only ... ok
  test cronus_ui_dock::tests::extra_text_items_become_items ... ok
  test cronus_ui_dock::tests::field_label_is_not_an_item_when_items_exist ... ok
  test cronus_ui_dock::tests::label_only_still_emits_one_item ... ok
  test cronus_ui_dock::tests::linked_item_is_anchor ... ok
  test cronus_ui_dock::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_dock::tests::root_is_nav_with_items_not_surf ... ok
  test cronus_ui_dock::tests::skips_interact_nav_surf ... ok
  test result: ok. 8 passed

cargo test --offline cronus_ui_workspace_switcher
  running 7 tests
  test cronus_ui_workspace_switcher::tests::chrome_is_token_only ... ok
  test cronus_ui_workspace_switcher::tests::extra_text_items_become_workspaces ... ok
  test cronus_ui_workspace_switcher::tests::field_label_is_not_a_workspace_when_items_exist ... ok
  test cronus_ui_workspace_switcher::tests::label_only_still_opens_content ... ok
  test cronus_ui_workspace_switcher::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_workspace_switcher::tests::root_is_trigger_with_always_open_content_not_nav ... ok
  test cronus_ui_workspace_switcher::tests::skips_interact_nav_surf ... ok
  test result: ok. 7 passed

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

Also green: `cargo test --offline cronus_ui_widgets` (19 passed), including `ported_family_skips_interact` over the appended list.

Wave total: **30 passed** (8 + 8 + 7 + 7).
