# Cronus Audit Wave 1i — app-shell / table-of-contents / form

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1i-a`  
Branch: `feat/wave1i-shell-toc-form`  
Base: `c068dc4`  
Date: 2026-09-13

Done: 3 dedicated modules, interact skipped, tests green. Not pushed.  
Not touched: signature-pad, lightbox, meteors, sankey-chart.

Helpers: `src/cronus_ui_kit.rs` (`choice_texts`, `esc`, `label_of`, `texts`). Pattern: `src/cronus_ui_checkbox.rs` (no voodoo, no inline BASE/SURF/CTRL). Not interact `nav()` / `field_form()`.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `635ca98` | feat(ui): dedicated AppShell renderer |
| `0937a0e` | feat(ui): dedicated TableOfContents renderer |
| `3e1d5d9` | feat(ui): dedicated Form renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1h list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … workspace-switcher …
    "app-shell",
    "table-of-contents",
    "form",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| app-shell | `src/cronus_ui_app_shell.rs` | `cronus_ui_app_shell::render` |
| table-of-contents | `src/cronus_ui_table_of_contents.rs` | `cronus_ui_table_of_contents::render` |
| form | `src/cronus_ui_form.rs` | `cronus_ui_form::render` |

`src/main.rs` mods: `cronus_ui_app_shell`, `cronus_ui_table_of_contents`, `cronus_ui_form`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

88 `PORTED_FAMILIES`. Remaining chart stub: sankey-chart. meteors still fx stub.

## app-shell

React: SidebarProvider composition with `data-slot="app-shell-header"` / `app-shell-body` / `app-shell-content` (`packages/ui/src/components/app-shell.tsx`). Kernel adds wrapper `data-slot="app-shell"` (no SidebarProvider). Header/title from label. Extra texts go in the body. Not a generic `nav("app-shell")` SURF `<nav>`.

Dedicated DOM (static, zero JS):

```html
<div data-slot="app-shell"><div data-slot="app-shell-content"><header data-slot="app-shell-header">Dashboard</header><div data-slot="app-shell-body"></div></div></div>
```

Forbidden interact (`nav("app-shell")`): `<nav data-slot="app-shell">` SURF box without header/body/content (`flex-wrap:wrap;gap:0.25rem`). Asserts: no `<nav`, no `style=`, no `flex-wrap:wrap`.

Chrome: shell `min-height: 100svh` on `var(--cronus-surface-base)`; header sticky `height: 3.5rem` with `var(--cronus-border)`.

## table-of-contents

React: `<nav data-slot="table-of-contents">` with `<ul data-slot="table-of-contents-list">` and `<a data-slot="table-of-contents-link">` (`packages/ui/src/components/table-of-contents.tsx`; file may contain 1 NUL — still TSX). Kernel links come from text items; href is the item link or a `#slug` of the label. Not a generic `nav("table-of-contents")` SURF.

Dedicated DOM (static, zero JS):

```html
<nav data-slot="table-of-contents" aria-label="On this page"><ul data-slot="table-of-contents-list"><li><a data-slot="table-of-contents-link" href="#intro">Intro</a></li><li><a data-slot="table-of-contents-link" href="#api">API</a></li></ul></nav>
```

Each text item is a link. Field labels are not links when `item` choices exist. Label-only still emits one link. Linked items keep their href.

Forbidden interact (`nav("table-of-contents")`): `<nav data-slot="table-of-contents">` SURF box without list/link (`flex-wrap:wrap;gap:0.25rem`). Asserts: no `style=`, no `flex-wrap:wrap`.

Chrome: list `border-left: 1px solid var(--cronus-border)`; links `var(--cronus-fg-tertiary)`.

## form

React rhf wrappers: FormItem `data-slot="form-item"`, FormLabel `form-label`, FormControl `form-control`, FormDescription `form-description` (`packages/ui/src/components/form.tsx`). Kernel emits `<form data-slot="form">` plus those slots and a native input. Not interact `field_form()` SURF dump + Submit button styles. No voodoo (`v-submit` / `v-method`) even when runtime is on.

Dedicated DOM (static, zero JS):

```html
<form data-slot="form"><div data-slot="form-item"><label data-slot="form-label">Email</label><div data-slot="form-control"><input type="text" name="Email" /></div></div></form>
```

Each text item is a field (label + control + input). Field labels are not fields when `item` choices exist. Label-only still emits one field. `description` items become `form-description`. Input is not `data-slot="form-control"` (that slot is the wrapper) so the field stub fingerprint (`<input data-slot=` + `-control"`) does not fire.

Forbidden interact (`field_form("form")`): `<form data-slot="form" style=BASE+SURF>` with inline-styled labels, `data-slot="button"` Submit, and optional `v-submit`. Asserts: no `style=`, no `>Submit</button>`, no `v-submit=`.

Chrome: form/item column flex; input on `var(--cronus-surface-inset)` with `var(--cronus-border)`; description `font-size: 0.75rem` `var(--cronus-fg-secondary)`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_app_shell
  running 7 tests
  test cronus_ui_app_shell::tests::chrome_is_token_only ... ok
  test cronus_ui_app_shell::tests::extra_text_items_go_in_body ... ok
  test cronus_ui_app_shell::tests::field_label_stays_in_header_when_items_exist ... ok
  test cronus_ui_app_shell::tests::label_only_still_emits_header ... ok
  test cronus_ui_app_shell::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_app_shell::tests::root_is_wrapper_with_header_body_content_not_nav_surf ... ok
  test cronus_ui_app_shell::tests::skips_interact_nav_surf ... ok
  test result: ok. 7 passed

cargo test --offline cronus_ui_table_of_contents
  running 8 tests
  test cronus_ui_table_of_contents::tests::chrome_is_token_only ... ok
  test cronus_ui_table_of_contents::tests::extra_text_items_become_links ... ok
  test cronus_ui_table_of_contents::tests::field_label_is_not_a_link_when_items_exist ... ok
  test cronus_ui_table_of_contents::tests::label_only_still_emits_one_link ... ok
  test cronus_ui_table_of_contents::tests::linked_item_keeps_href ... ok
  test cronus_ui_table_of_contents::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_table_of_contents::tests::root_is_nav_list_with_links_not_surf ... ok
  test cronus_ui_table_of_contents::tests::skips_interact_nav_surf ... ok
  test result: ok. 8 passed

cargo test --offline cronus_ui_form
  running 7 tests
  test cronus_ui_form::tests::chrome_is_token_only ... ok
  test cronus_ui_form::tests::description_from_description_item ... ok
  test cronus_ui_form::tests::extra_text_items_become_fields ... ok
  test cronus_ui_form::tests::field_label_is_not_a_field_when_items_exist ... ok
  test cronus_ui_form::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_form::tests::root_is_form_item_label_and_input_not_surf ... ok
  test cronus_ui_form::tests::skips_interact_field_form ... ok
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

Also green: `cargo test --offline cronus_ui_widgets` (19 passed), including `ported_family_skips_interact` over the appended list and `form_skips_vsubmit_even_when_voodoo_and_entity`.

Wave total: **29 passed** (7 + 8 + 7 + 7).
