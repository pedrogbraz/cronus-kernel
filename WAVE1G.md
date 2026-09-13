# Wave 1g — phone / currency / color kernel ports

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1g-b`
Branch: `feat/wave1g-phone-currency-color`
Base: `91df802`
Date: 2026-09-13

Dedicated CONTRACT renderers for `phone-input`, `currency-input`, `color-picker`.
Interact `input("tel")` / `input("number")` / `input("color")` is skipped.
Zero JS, zero Voodoo. Charts and scroll were not touched.

Helpers: `src/cronus_ui_kit.rs` (`esc`, `item`, `label_of`). Pattern: `src/cronus_ui_input.rs` / checkbox (no voodoo, no interact `input()` CTRL).

`PORTED_FAMILIES` is now 70 (67 after wave 1f + these 3).

## DOM (React slots)

| Family | Contract | Forbidden interact |
| --- | --- | --- |
| phone-input | `<div data-slot="phone-input" role="group"><button data-slot="phone-input-country">` + `<input data-slot="phone-input-field" type="tel">` | `<label data-slot="phone-input"><input type="tel" data-slot="phone-input-control" style=CTRL>` |
| currency-input | `<div data-slot="currency-input"><span data-slot="currency-input-prefix">$</span><input data-slot="currency-input-field">` | `input("currency-input", "number")` with `*-control` |
| color-picker | wrapper `data-slot="color-picker"` + `<button data-slot="color-picker-trigger">` + always-open `<div data-slot="color-picker-content">` (swatch) | `input("color-picker", "color")` as the only control |

Phone-input never wraps the tel field in `<label data-slot="phone-input">`. Currency-input is a fused prefix + text field (`inputmode="decimal"`), not `type="number"`. Color-picker is a trigger + open swatch panel, not a native `<input type="color">`.

## Wiring

- modules: `src/cronus_ui_phone_input.rs`, `src/cronus_ui_currency_input.rs`, `src/cronus_ui_color_picker.rs`
- `mod` in `src/main.rs`
- `PORTED_FAMILIES` + `dedicated_render` in `src/cronus_ui_widgets.rs` and `scripts/gen_cronus_ui_widgets.py` (appended, not reordered)
- `dedicated_fn_name` + interact fingerprints + sidecar file list in `src/cli/stub_renderer_gate.rs`
- token-only `COMPONENT_CHROME` in `src/cronus_ui.rs` (`var(--cronus-*)`, no zinc palette)

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

## Evidence

```
cargo test cronus_ui_phone_input
# 7 passed (role=group, country button + type=tel field, not label *-control)

cargo test cronus_ui_currency_input
# 7 passed ($ prefix + currency-input-field, not type=number *-control)

cargo test cronus_ui_color_picker
# 6 passed (wrapper + trigger + open content swatch, not type=color)

cargo test stub_renderer_gate
# 7 passed (Dedicated, skip interact, no sidecar assets)
```

No `*-control`. No interact `<label data-slot>` wrap. No inline CTRL. No Voodoo even with runtime on.

Commits on `feat/wave1g-phone-currency-color` (one per family):

1. `dfeb882` feat(ui): dedicated PhoneInput renderer
2. `f55abbc` feat(ui): dedicated CurrencyInput renderer
3. `69e4666` feat(ui): dedicated ColorPicker renderer

Not pushed.
