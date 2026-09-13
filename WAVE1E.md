# Wave 1e — date / time / range kernel ports

Dedicated CONTRACT renderers for `date-picker`, `time-picker`, `date-range-picker`.
Interact `input("date")` / `input("time")` / `date_range()` is skipped. Zero JS, zero Voodoo.
Command and drawer were not touched.

`PORTED_FAMILIES` is now 52 (49 after wave 1d + these 3).

## DOM (React slots, always-open)

| Family | Contract | Forbidden interact |
| --- | --- | --- |
| date-picker | `<button type="button" data-slot="date-picker-trigger">` + open `<div data-slot="date-picker-content">` with a static 4×7 calendar grid | `<label data-slot="date-picker">` + `<input type="date" data-slot="date-picker-control">` |
| time-picker | `<div data-slot="time-picker">` root, open `<div data-slot="time-picker-content">` with hour/minute/(AM-PM) option columns | `<label data-slot="time-picker">` + `<input type="time" data-slot="time-picker-control">` |
| date-range-picker | `<button data-slot="date-range-picker-trigger">` + open `<div data-slot="date-range-picker-content">` (presets + two static months) | two native `<input type="date">` with CTRL styles |

Date-picker never uses `type="date"` as the root control — the trigger is a button.

## Wiring

- modules: `src/cronus_ui_date_picker.rs`, `src/cronus_ui_time_picker.rs`, `src/cronus_ui_date_range_picker.rs`
- `mod` in `src/main.rs`
- `PORTED_FAMILIES` + `dedicated_render` in `src/cronus_ui_widgets.rs` and `scripts/gen_cronus_ui_widgets.py`
- `dedicated_fn_name` + interact fingerprints + sidecar file list in `src/cli/stub_renderer_gate.rs`
- token-only `COMPONENT_CHROME` in `src/cronus_ui.rs` (`var(--cronus-*)`, no zinc palette)

## Evidence

```
cargo test cronus_ui_date_picker
# 7 passed (root is trigger button, not type="date")

cargo test cronus_ui_time_picker
# 8 passed (open columns, not type="time")

cargo test cronus_ui_date_range_picker
# 7 passed (trigger + content, not two type="date" inputs)

cargo test stub_renderer_gate
# 7 passed (Dedicated, skip interact, no sidecar assets)
```

Commits on `feat/wave1e-date-time-range` (one per family):

1. `feat(ui): dedicated DatePicker renderer`
2. `feat(ui): dedicated TimePicker renderer`
3. `feat(ui): dedicated DateRangePicker renderer`
