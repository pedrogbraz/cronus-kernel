# Cronus Audit Wave 1k — rich-text-editor / confirmation-dialog / invite-dialog

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1k-b`  
Branch: `feat/wave1k-rte-confirm-invite`  
Base: `8dab4c9`  
Date: 2026-09-13

Done: 3 dedicated modules, interact skipped, tests green. Not pushed.  
Not touched: charts, shimmer, meteors, sankey-chart.

Helpers: `src/cronus_ui_kit.rs` (`esc`, `item`, `label_of`). Pattern: `src/cronus_ui_textarea.rs` / `src/cronus_ui_alert_dialog.rs` (dedicated modules, no voodoo, no inline BASE/SURF/CTRL). Not interact `textarea()` / `dialog()`.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

`PORTED_FAMILIES` is now 106 (103 after wave 1j + these 3).

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `8a76b5a` | feat(ui): dedicated RichTextEditor renderer |
| `99c3735` | feat(ui): dedicated ConfirmationDialog renderer |
| `55ac5b4` | feat(ui): dedicated InviteDialog renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1j list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … sunburst-chart …
    "rich-text-editor",
    "confirmation-dialog",
    "invite-dialog",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| rich-text-editor | `src/cronus_ui_rich_text_editor.rs` | `cronus_ui_rich_text_editor::render` |
| confirmation-dialog | `src/cronus_ui_confirmation_dialog.rs` | `cronus_ui_confirmation_dialog::render` |
| invite-dialog | `src/cronus_ui_invite_dialog.rs` | `cronus_ui_invite_dialog::render` |

`src/main.rs` mods: `cronus_ui_rich_text_editor`, `cronus_ui_confirmation_dialog`, `cronus_ui_invite_dialog`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

## rich-text-editor

React: Root `data-slot="rich-text-editor"` + toolbar `rich-text-editor-toolbar` + `rich-text-editor-content-wrapper` (`packages/ui/src/components/rich-text-editor.tsx`, Tiptap). Kernel is static — no Tiptap JS, no `contenteditable="true"`. Content is a textbox-looking div with the label as static text.

Dedicated DOM:

```html
<div data-slot="rich-text-editor">
  <div data-slot="rich-text-editor-toolbar" role="toolbar" aria-label="Text formatting">
    <button type="button" aria-label="Bold">Bold</button>
    <!-- Italic, Strikethrough, Inline code, H1–H3, lists, quote, code block, HR, Undo, Redo -->
  </div>
  <div data-slot="rich-text-editor-content-wrapper">
    <div role="textbox" aria-multiline="true" aria-label="Release notes">Release notes</div>
  </div>
</div>
```

Forbidden interact (`textarea("rich-text-editor")`): `<label data-slot="rich-text-editor"><textarea data-slot="rich-text-editor-control">`. Asserts: no `<textarea`, no `*-control`, no `<dialog`, no `showModal`.

Chrome: column shell on `var(--cronus-surface-base)` with `var(--cronus-border)`; toolbar overlay mix; content wrapper `min-height: 10rem`. Replaced the old `[data-slot="rich-text-editor"] textarea` interact leftover.

## confirmation-dialog

React: Content `data-slot="confirmation-dialog"` + confirm `confirmation-dialog-confirm` (`packages/ui/src/components/confirmation-dialog.tsx`, AlertDialog). Always-open static — no Radix portal, no native `<dialog>`.

Dedicated DOM:

```html
<div data-slot="confirmation-dialog"><div data-slot="confirmation-dialog-title">Delete project</div><div data-slot="confirmation-dialog-description">This cannot be undone.</div><button type="button">Cancel</button><button type="button" data-slot="confirmation-dialog-confirm">Confirm</button></div>
```

Title is the label. Extra texts become optional `confirmation-dialog-description` (omitted when empty). Confirm/cancel default to Confirm/Cancel; `confirm` / `action` / `cancel` items override.

Forbidden interact (`dialog("confirmation-dialog")`): native `<dialog data-slot="confirmation-dialog-content">` + `onclick="…showModal()"` + SURF `max-width:28rem`. Asserts: no `<dialog`, no `*-control`, no `showModal`.

Chrome: content `z-index: 50`, `max-width: 28rem` (React `max-w-md`) on `var(--cronus-surface-floating)`; title display font; description `var(--cronus-fg-secondary)`; confirm primary.

## invite-dialog

React: Content `data-slot="invite-dialog"` + email field + send `invite-dialog-send` (`packages/ui/src/components/invite-dialog.tsx`, Dialog). Always-open static — no Dialog `showModal()`, no JS.

Dedicated DOM:

```html
<div data-slot="invite-dialog"><div data-slot="invite-dialog-title">Invite member</div><label>Email<input type="email" name="email" placeholder="name@example.com" autocomplete="email" required /></label><button type="button">Cancel</button><button type="button" data-slot="invite-dialog-send">Send invite</button></div>
```

Title is the label. Email field is always present (no `*-control` slot). Send defaults to `Send invite`; `send` / `cancel` / `placeholder` items override.

Forbidden interact (`dialog("invite-dialog")`): native `<dialog data-slot="invite-dialog-content">` + `showModal()` + SURF. Asserts: no `<dialog`, no `*-control`, no `showModal`.

Chrome: panel `z-index: 50`, `max-width: 28rem` (React `max-w-md`) on `var(--cronus-surface-floating)`; email input on `var(--cronus-surface-inset)`; send primary.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_rich_text_editor
  running 6 tests
  test cronus_ui_rich_text_editor::tests::chrome_is_token_only ... ok
  test cronus_ui_rich_text_editor::tests::content_is_label_not_a_textarea ... ok
  test cronus_ui_rich_text_editor::tests::extra_text_does_not_become_a_control ... ok
  test cronus_ui_rich_text_editor::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_rich_text_editor::tests::root_is_editor_with_toolbar_and_static_content ... ok
  test cronus_ui_rich_text_editor::tests::skips_interact_textarea_surf ... ok
  test result: ok. 6 passed

cargo test --offline cronus_ui_confirmation_dialog
  running 6 tests
  test cronus_ui_confirmation_dialog::tests::always_open_title_from_label_and_confirm ... ok
  test cronus_ui_confirmation_dialog::tests::chrome_is_token_only ... ok
  test cronus_ui_confirmation_dialog::tests::confirm_and_cancel_items ... ok
  test cronus_ui_confirmation_dialog::tests::description_is_optional ... ok
  test cronus_ui_confirmation_dialog::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_confirmation_dialog::tests::skips_interact_dialog_surf ... ok
  test result: ok. 6 passed

cargo test --offline cronus_ui_invite_dialog
  running 6 tests
  test cronus_ui_invite_dialog::tests::always_open_email_field_and_send ... ok
  test cronus_ui_invite_dialog::tests::chrome_is_token_only ... ok
  test cronus_ui_invite_dialog::tests::description_and_send_override ... ok
  test cronus_ui_invite_dialog::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_invite_dialog::tests::placeholder_from_item ... ok
  test cronus_ui_invite_dialog::tests::skips_interact_dialog_surf ... ok
  test result: ok. 6 passed

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

Also green: `cargo test --offline cronus_ui_widgets` (19 passed), including `ported_family_skips_interact` over the appended list. `meteors` remains fx stub; `sankey-chart` remains chart stub.

Wave total: **25 passed** (6 + 6 + 6 + 7).

No `<dialog`. No `*-control`. No `showModal`. No Tiptap JS. No Voodoo even with runtime on.

Not pushed.
