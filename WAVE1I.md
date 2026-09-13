# Cronus Audit Wave 1i — signature-pad / resizable / scheduler

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1i-b`  
Branch: `feat/wave1i-sig-resize-sched`  
Base: `c068dc4`  
Date: 2026-09-13

Done: 3 dedicated modules, interact skipped, tests green. Not pushed.  
Not touched: app-shell, lightbox, meteors, sankey-chart.

Helpers: `src/cronus_ui_kit.rs` (`choice_texts`, `label_of`, `texts`). Pattern: `src/cronus_ui_calendar.rs` / `src/cronus_ui_scroll_area.rs` (no voodoo, no inline BASE/SURF/CTRL). Not interact `signature()` / `calendar()` / catalog `display()`.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `0b79e5f` | feat(ui): dedicated SignaturePad renderer |
| `b61448a` | feat(ui): dedicated Resizable renderer |
| `dfac782` | feat(ui): dedicated Scheduler renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1h list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … workspace-switcher …
    "signature-pad",
    "resizable",
    "scheduler",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| signature-pad | `src/cronus_ui_signature_pad.rs` | `cronus_ui_signature_pad::render` |
| resizable | `src/cronus_ui_resizable.rs` | `cronus_ui_resizable::render` |
| scheduler | `src/cronus_ui_scheduler.rs` | `cronus_ui_scheduler::render` |

`src/main.rs` mods: `cronus_ui_signature_pad`, `cronus_ui_resizable`, `cronus_ui_scheduler`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

88 `PORTED_FAMILIES`. Remaining chart stub: sankey-chart. meteors still fx stub.

## signature-pad

React: `<div data-slot="signature-pad">` plus `<canvas data-slot="signature-pad-canvas">` and optional `data-slot="signature-pad-hint"` (`packages/ui/src/components/signature-pad.tsx`). Kernel canvas is **static** — no pointer drawing, no undo/clear JS. Not interact `signature()` SURF box + `<canvas>` without the canvas slot.

Dedicated DOM (static, zero JS):

```html
<div data-slot="signature-pad" data-empty="true"><canvas role="img" aria-label="Sign here" data-slot="signature-pad-canvas" width="320" height="160"></canvas><div aria-hidden="true" data-slot="signature-pad-hint"><span>Sign here</span></div></div>
```

Hint uses the field label. Canvas always has `data-slot="signature-pad-canvas"`.

Forbidden interact (`signature()`): SURF box + `<canvas width="320" height="120">` **without** `signature-pad-canvas`. Asserts: no `style=`, no `onclick=`, no `onpointer`, no `v-data=`.

Chrome: relative `10rem` pad on `var(--cronus-surface-inset)` with `var(--cronus-border)`; canvas `absolute inset: 0`; hint `var(--cronus-fg-muted)`.

## resizable

React: `ResizablePanelGroup` `data-slot="resizable-panel-group"` plus `data-slot="resizable-handle"` (`packages/ui/src/components/resizable.tsx`). Kernel wraps that in `data-slot="resizable"` and emits two static panels + handle (no JS drag). Not catalog `display()` SURF `<section>` and not interact `scroll("resizable")`.

Dedicated DOM (static, zero JS):

```html
<div data-slot="resizable"><div data-slot="resizable-panel-group" data-panel-group-direction="horizontal"><div data-slot="resizable-panel">Sidebar</div><div data-slot="resizable-handle" role="separator" aria-orientation="vertical" aria-valuenow="50" aria-valuemin="0" aria-valuemax="100"></div><div data-slot="resizable-panel">Main</div></div></div>
```

Two panels from text items. Label-only still emits two panels (second empty). Handle is a `role="separator"` with aria values so AT never sees a splitter without `aria-valuenow`.

Forbidden interact (`scroll("resizable")` / `display("resizable")`): SURF box `max-height:12rem;overflow:auto` or `<section data-slot="resizable">` without panel-group/handle. Asserts: no `<section`, no `style=`.

Chrome: wrapper on `var(--cronus-surface-raised)` with `var(--cronus-border)`; group `display: flex`; handle `1px` `var(--cronus-border)` track.

## scheduler

React: `<div data-slot="scheduler">` plus `<h2 data-slot="scheduler-title">` and `<table data-slot="scheduler-grid">` weekday headers (`packages/ui/src/components/scheduler.tsx`). Kernel is a **static week** (days 1–7, Sun…Sat). Extra `item` texts become `data-slot="scheduler-event"` chips. Not interact `calendar("scheduler")` SURF CSS grid of 31 day buttons.

Dedicated DOM (static, zero JS):

```html
<div data-slot="scheduler"><h2 data-slot="scheduler-title">March</h2><table data-slot="scheduler-grid" aria-label="Event calendar"><thead data-slot="scheduler-weekdays"><tr><th scope="col">Sun</th><th scope="col">Mon</th><th scope="col">Tue</th><th scope="col">Wed</th><th scope="col">Thu</th><th scope="col">Fri</th><th scope="col">Sat</th></tr></thead><tbody><tr><td><span>1</span></td><td><span>2</span></td><td><span>3</span></td><td><span>4</span></td><td><span>5</span></td><td><span>6</span></td><td><span>7</span></td></tr></tbody></table></div>
```

Title from the field label. One week only (`<td>8` is not emitted). Events are spans, not buttons.

Forbidden interact (`calendar("scheduler")`): SURF `display:grid;grid-template-columns:repeat(7,1fr)` with 31 `<button>` cells, no `scheduler-title` / `scheduler-grid`. Asserts: no `style=`, no `grid-template-columns:repeat(7,1fr)`, no `<button`.

Chrome: `var(--cronus-surface-base)` with `var(--cronus-border)`; weekdays `var(--cronus-fg-tertiary)`; `border-collapse: collapse`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_signature_pad
  running 5 tests
  test cronus_ui_signature_pad::tests::chrome_is_token_only ... ok
  test cronus_ui_signature_pad::tests::hint_uses_label ... ok
  test cronus_ui_signature_pad::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_signature_pad::tests::root_is_pad_with_canvas_slot_not_interact_surf ... ok
  test cronus_ui_signature_pad::tests::skips_interact_signature_surf ... ok
  test result: ok. 5 passed

cargo test --offline cronus_ui_resizable
  running 6 tests
  test cronus_ui_resizable::tests::chrome_is_token_only ... ok
  test cronus_ui_resizable::tests::extra_text_items_become_panels ... ok
  test cronus_ui_resizable::tests::label_only_still_emits_two_panels ... ok
  test cronus_ui_resizable::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_resizable::tests::root_is_wrapper_with_group_two_panels_and_handle ... ok
  test cronus_ui_resizable::tests::skips_display_and_scroll_surf ... ok
  test result: ok. 6 passed

cargo test --offline cronus_ui_scheduler
  running 5 tests
  test cronus_ui_scheduler::tests::chrome_is_token_only ... ok
  test cronus_ui_scheduler::tests::extra_items_become_events ... ok
  test cronus_ui_scheduler::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_scheduler::tests::root_is_scheduler_with_title_and_week_grid ... ok
  test cronus_ui_scheduler::tests::skips_interact_calendar_surf ... ok
  test result: ok. 5 passed

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

Wave total: **23 passed** (5 + 6 + 5 + 7).

Not pushed.
