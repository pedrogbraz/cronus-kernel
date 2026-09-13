# Cronus Audit Wave 1d — file-dropzone / popover / hover-card

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1d-b`  
Branch: `feat/wave1d-dropzone-popover-hover`  
Base: `feat/cronus-ui-tokens-button` @ `37caf75`  
Date: 2026-09-13

Dedicated CONTRACT renderers. Interact skipped via `PORTED_FAMILIES`. Chrome is `var(--cronus-*)` only. Zero JS, zero Voodoo attrs, no HTML in `.cronus`. Not pushed. Not touched: combobox, dropdown-menu, mode-toggle.

Helpers: `src/cronus_ui_kit.rs` (`esc`, `label_of`, `texts`). Pattern: `src/cronus_ui_checkbox.rs` (no voodoo). Not copied: `cronus_ui_password_input.rs` (still emits `v-data`). Not interact `input()` / `popover()`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `7ffae6c` | feat(ui): dedicated FileDropzone renderer |
| `cbd2c94` | feat(ui): dedicated Popover renderer |
| `be78079` | feat(ui): dedicated HoverCard renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing families):

```rust
"file-dropzone", "popover", "hover-card",
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| file-dropzone | `src/cronus_ui_file_dropzone.rs` | `cronus_ui_file_dropzone::render` |
| popover | `src/cronus_ui_popover.rs` | `cronus_ui_popover::render` |
| hover-card | `src/cronus_ui_hover_card.rs` | `cronus_ui_hover_card::render` |

`src/main.rs` mods: `cronus_ui_file_dropzone`, `cronus_ui_popover`, `cronus_ui_hover_card`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
`stub_renderer_gate` lists the new modules in `dedicated_module_no_sidecar_assets`.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`.

## file-dropzone

React: `<label data-slot="file-dropzone">` wrapping visually-hidden `<input type="file" className="sr-only">` (`packages/ui/src/components/file-dropzone.tsx`). Input has no data-slot.

Dedicated DOM:

```html
<label data-slot="file-dropzone"><input type="file" class="sr-only" aria-label="Upload files" /><span>Upload files</span></label>
```

Label from `label_of`. Optional `accept` / `multiple` / `disabled` / `aria-label` / `aria-describedby` from props. Disabled sets `aria-disabled` on the label and `disabled` on the input.

Forbidden interact (`input("file-dropzone")`): `<label data-slot="file-dropzone"><input data-slot="file-dropzone-control" type="file" style=CTRL>`.

Chrome: flex column, `align-items: center`, `gap: 0.5rem`, `border: 2px dashed var(--cronus-border)`, `var(--cronus-surface-inset)`, `padding: 2.5rem 1.5rem`, `rounded-xl`, `text-align: center`, `font-size: 0.875rem`. `.sr-only` clip-hides the file input.

## popover

React: Root emits **no** `data-slot`. Content has `data-slot="popover-content"` (`packages/ui/src/components/popover.tsx`). Audit always-open static DOM (no hover/JS).

Dedicated DOM:

```html
<button type="button">More</button><div data-slot="popover-content">Extra actions.</div>
```

Trigger from label. Body from extra text items. Content slot is the contract — no `data-slot="popover"` wrapper.

Forbidden interact (`popover("popover")`): `<details data-slot="popover">` SURF box with inline styles, no `popover-content`.

Chrome: content `z-index: 50`, `width: 18rem` (`w-72`), `padding: 0.75rem`, `var(--cronus-surface-floating)`, `var(--cronus-shadow-lg)`.

## hover-card

React: Root emits **no** `data-slot`. Content has `data-slot="hover-card-content"` (`packages/ui/src/components/hover-card.tsx`). Same always-open static pattern as popover.

Dedicated DOM:

```html
<button type="button">Preview</button><div data-slot="hover-card-content">Native disclosure.</div>
```

Trigger from label. Body from extra text. Content slot is the contract — no `data-slot="hover-card"` wrapper.

Forbidden interact (`popover("hover-card")`): `<details data-slot="hover-card">` SURF box.

Chrome: content `z-index: 50`, `width: 16rem` (`w-64`), `padding: 0.75rem`, `var(--cronus-surface-floating)`, `var(--cronus-shadow-lg)`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_file_dropzone
  running 5 tests
  test cronus_ui_file_dropzone::tests::aria_label_from_props ... ok
  test cronus_ui_file_dropzone::tests::chrome_is_token_only ... ok
  test cronus_ui_file_dropzone::tests::disabled_multiple_accept ... ok
  test cronus_ui_file_dropzone::tests::root_is_label_with_sr_only_file_input ... ok
  test cronus_ui_file_dropzone::tests::skips_interact_control_slot ... ok
  test result: ok. 5 passed

cargo test --offline cronus_ui_popover
  running 4 tests
  test cronus_ui_popover::tests::chrome_is_token_only ... ok
  test cronus_ui_popover::tests::content_slot_is_the_contract ... ok
  test cronus_ui_popover::tests::skips_interact_surf_details ... ok
  test cronus_ui_popover::tests::trigger_button_and_always_open_content ... ok
  test result: ok. 4 passed

cargo test --offline cronus_ui_hover_card
  running 4 tests
  test cronus_ui_hover_card::tests::chrome_is_token_only ... ok
  test cronus_ui_hover_card::tests::content_slot_is_the_contract ... ok
  test cronus_ui_hover_card::tests::skips_interact_surf_details ... ok
  test cronus_ui_hover_card::tests::trigger_button_and_always_open_content ... ok
  test result: ok. 4 passed

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

Wave total: **20 passed**.
