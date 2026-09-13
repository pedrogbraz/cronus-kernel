# Cronus Audit Wave 1e — drawer / sheet / calendar

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1e-b`  
Branch: `feat/wave1e-drawer-sheet-calendar`  
Base: `feat/cronus-ui-tokens-button` @ `7ca631b`  
Date: 2026-09-13

Done: 3 dedicated modules, interact skipped, tests green. Not pushed.  
Not touched: command, date-picker.

Helpers: `src/cronus_ui_kit.rs` (`esc`, `item`, `label_of`, `texts`). Pattern: `src/cronus_ui_checkbox.rs` (no voodoo, no inline BASE/SURF/CTRL). Not copied: `cronus_ui_dialog.rs` / interact `dialog()` / interact `calendar()`.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `ad5c09b` | feat(ui): dedicated Drawer renderer |
| `4bf9bcb` | feat(ui): dedicated Sheet renderer |
| `a90338b` | feat(ui): dedicated Calendar renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1d list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … mode-toggle …
    "drawer",
    "sheet",
    "calendar",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| drawer | `src/cronus_ui_drawer.rs` | `cronus_ui_drawer::render` |
| sheet | `src/cronus_ui_sheet.rs` | `cronus_ui_sheet::render` |
| calendar | `src/cronus_ui_calendar.rs` | `cronus_ui_calendar::render` |

`src/main.rs` mods: `cronus_ui_drawer`, `cronus_ui_sheet`, `cronus_ui_calendar`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

## drawer

React: Root `data-slot="drawer"` + Content `data-slot="drawer-content"` + Title / Description (`packages/ui/src/components/drawer.tsx`, vaul). Always-open static — no Vaul portal, no overlay JS.

Dedicated DOM:

```html
<div data-slot="drawer"><button type="button">Menu</button><div data-slot="drawer-content"><div data-slot="drawer-title">Menu</div><div data-slot="drawer-description">Slide-up panel.</div></div></div>
```

Trigger text is the label. Title is `title` if present, else the label. Extra texts become `drawer-description` (empty description div is still emitted).

Forbidden interact (`dialog("drawer")`): native `<dialog data-slot="drawer-content">` + `onclick="…showModal()"` + SURF `max-width:28rem`. Asserts: no `<dialog`, no `showModal`.

Chrome: trigger button on overlay surface; content `z-index: 50` on `var(--cronus-surface-floating)` with top radius; title `1.125rem` / display font; description `var(--cronus-fg-secondary)`.

## sheet

React: Content `data-slot="sheet-content"` + Title + optional Description (`packages/ui/src/components/sheet.tsx`, Radix Dialog). Root has no typed `data-slot="sheet"`. Always-open static — no Radix portal, no native `<dialog>`.

Dedicated DOM:

```html
<button type="button">Filters</button><div data-slot="sheet-content"><div data-slot="sheet-title">Filters</div><div data-slot="sheet-description">Narrow the list.</div></div>
```

Trigger + `sheet-title` from label/`title`. Extra texts become optional `sheet-description` (omitted when empty).

Forbidden interact (`dialog("sheet")`): native `<dialog data-slot="sheet-content">` + `showModal()` + SURF. Asserts: no `<dialog`, no `showModal`.

Chrome: sibling trigger via `button:has(+ [data-slot="sheet-content"])`; content `z-index: 50`, `width: 75%`, `max-width: 24rem` (React `w-3/4 max-w-sm`), `var(--cronus-surface-floating)` + `var(--cronus-shadow-lg)`.

## calendar

React: DayPicker with class `calendar` (no typed `data-slot`; comment in `packages/ui/src/components/calendar.tsx`). Kernel **must** emit `data-slot="calendar"` for the generic family test.

Dedicated DOM (static 4×7 month grid, days 1–28, no JS):

```html
<div data-slot="calendar"><table><caption>March</caption><thead><tr><th>Mo</th>…<th>Su</th></tr></thead><tbody><tr><td>1</td>…<td>7</td></tr>…<tr><td>22</td>…<td>28</td></tr></tbody></table></div>
```

Forbidden interact (`calendar("calendar")`): SURF CSS-grid `grid-template-columns:repeat(7,1fr)` + inline-styled day `<button>`s 1–31. Not a date `<input>`. Asserts: no `style=`, no `<input`, no `showModal`.

Chrome: padded raised surface, `border-collapse: collapse`, weekday `var(--cronus-fg-tertiary)`, day cells `2.25rem`.

## Tests

```
cargo test cronus_ui_drawer      # 6 passed
cargo test cronus_ui_sheet       # 6 passed
cargo test cronus_ui_calendar    # 4 passed
cargo test stub_renderer_gate    # 7 passed
```

Also green: `every_family_renders_slot_without_palette_scales`, `ported_family_skips_interact`.

Each family asserts no `<dialog` and no `showModal` in dedicated HTML. Stub gate fingerprints reject interact dialog/SURF calendar if a port regresses.

`looks_like_interact_generic` additions:

- drawer/sheet: `data-slot="drawer|sheet…"` plus `<dialog` or `showModal()`
- calendar: `data-slot="calendar"` plus `style=`, or `grid-template-columns:repeat(7,1fr)`
