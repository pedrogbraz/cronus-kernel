# Cronus Audit Wave 1i — alert-dialog / lightbox / notification-center

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1i-c`  
Branch: `feat/wave1i-alert-light-notif`  
Base: `c068dc4`  
Date: 2026-09-13

Done: 3 dedicated modules, interact skipped, tests green. Not pushed.  
Not touched: app-shell, signature, meteors, sankey-chart.

Helpers: `src/cronus_ui_kit.rs` (`esc`, `item`, `label_of`, `texts`). Pattern: `src/cronus_ui_drawer.rs` / `src/cronus_ui_sheet.rs` (always-open static, no voodoo, no inline BASE/SURF/CTRL). Not interact `dialog()` / `popover()`.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

`PORTED_FAMILIES` is now 88 (85 after wave 1h + these 3).

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `2c89780` | feat(ui): dedicated AlertDialog renderer |
| `1015816` | feat(ui): dedicated Lightbox renderer |
| `ac12c91` | feat(ui): dedicated NotificationCenter renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1h list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … workspace-switcher …
    "alert-dialog",
    "lightbox",
    "notification-center",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| alert-dialog | `src/cronus_ui_alert_dialog.rs` | `cronus_ui_alert_dialog::render` |
| lightbox | `src/cronus_ui_lightbox.rs` | `cronus_ui_lightbox::render` |
| notification-center | `src/cronus_ui_notification_center.rs` | `cronus_ui_notification_center::render` |

`src/main.rs` mods: `cronus_ui_alert_dialog`, `cronus_ui_lightbox`, `cronus_ui_notification_center`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

## alert-dialog

React: Content `data-slot="alert-dialog-content"` + Title / Description / Action / Cancel (`packages/ui/src/components/alert-dialog.tsx`, Radix AlertDialog). Wrapper `data-slot="alert-dialog"` for the family slot test. Always-open static — no Radix portal, no native `<dialog>`.

Dedicated DOM:

```html
<div data-slot="alert-dialog"><button type="button">Delete</button><div data-slot="alert-dialog-content"><div data-slot="alert-dialog-title">Delete</div><div data-slot="alert-dialog-description">This cannot be undone.</div><button type="button" data-slot="alert-dialog-cancel">Cancel</button><button type="button" data-slot="alert-dialog-action">Continue</button></div></div>
```

Trigger text is the label. Title is `title` if present, else the label. Extra texts become optional `alert-dialog-description` (omitted when empty). Action/cancel default to Continue/Cancel; `action` / `cancel` items override.

Forbidden interact (`dialog("alert-dialog")`): native `<dialog data-slot="alert-dialog-content">` + `onclick="…showModal()"` + SURF `max-width:28rem`. Asserts: no `<dialog`, no `showModal`.

Chrome: trigger on overlay surface; content `z-index: 50`, `max-width: 32rem` (React `max-w-lg`) on `var(--cronus-surface-floating)`; title display font; description `var(--cronus-fg-secondary)`; cancel outline, action primary.

## lightbox

React: Root `data-slot="lightbox"` + caption / optional counter / close (`packages/ui/src/components/lightbox.tsx`). Always-open static — no Dialog `showModal()`, no JS.

Dedicated DOM:

```html
<div data-slot="lightbox"><span data-slot="lightbox-counter">1 / 1</span><button type="button" data-slot="lightbox-close">Close</button><p data-slot="lightbox-caption">Sunset over the bay</p></div>
```

Caption is the label. Counter (`1 / 1`) and close are present and static. Extra texts do not add slides.

Forbidden interact (`dialog("lightbox")`): native `<dialog data-slot="lightbox-content">` + `showModal()` + SURF. Asserts: no `<dialog`, no `showModal`.

Chrome: column on `var(--cronus-surface-base)`; caption/counter `var(--cronus-fg-secondary)`; `z-index: 50`.

## notification-center

React: Trigger `data-slot="notification-trigger"` + panel `data-slot="notification-center"` + rows `data-slot="notification-row"` (`packages/ui/src/components/notification-center.tsx`, Popover). Always-open static — no popover JS.

Dedicated DOM:

```html
<button type="button" data-slot="notification-trigger">Notifications</button><div data-slot="notification-center"><button type="button" data-slot="notification-row">Deployed to production</button><button type="button" data-slot="notification-row">Invite accepted</button></div>
```

Trigger label is `Notifications`. Each text item is a `notification-row`. Label-only still emits one row.

Forbidden interact (`popover("notification-center")`): `<details data-slot="notification-center">` SURF box. Asserts: no `<details`, no `<dialog`, no `showModal`.

Chrome: trigger ghost button; panel `z-index: 50`, `width: 20rem` (React `w-80`) on `var(--cronus-surface-floating)`; row hover `var(--cronus-surface-overlay)`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_alert_dialog
  running 7 tests
  test cronus_ui_alert_dialog::tests::action_and_cancel_items ... ok
  test cronus_ui_alert_dialog::tests::chrome_is_token_only ... ok
  test cronus_ui_alert_dialog::tests::description_is_optional ... ok
  test cronus_ui_alert_dialog::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_alert_dialog::tests::skips_interact_dialog_surf ... ok
  test cronus_ui_alert_dialog::tests::title_item_is_panel_title ... ok
  test cronus_ui_alert_dialog::tests::trigger_and_always_open_content ... ok
  test result: ok. 7 passed

cargo test --offline cronus_ui_lightbox
  running 6 tests
  test cronus_ui_lightbox::tests::always_open_caption_from_label ... ok
  test cronus_ui_lightbox::tests::chrome_is_token_only ... ok
  test cronus_ui_lightbox::tests::extra_text_does_not_change_caption ... ok
  test cronus_ui_lightbox::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_lightbox::tests::optional_counter_and_close ... ok
  test cronus_ui_lightbox::tests::skips_interact_dialog_surf ... ok
  test result: ok. 6 passed

cargo test --offline cronus_ui_notification_center
  running 6 tests
  test cronus_ui_notification_center::tests::chrome_is_token_only ... ok
  test cronus_ui_notification_center::tests::extra_texts_become_rows ... ok
  test cronus_ui_notification_center::tests::label_only_still_emits_one_row ... ok
  test cronus_ui_notification_center::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_notification_center::tests::skips_interact_popover_surf ... ok
  test cronus_ui_notification_center::tests::trigger_and_always_open_rows ... ok
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

Wave total: **26 passed** (7 + 6 + 6 + 7).

No `<dialog`. No `showModal`. No interact `<details>` SURF. No Voodoo even with runtime on.

Not pushed.
