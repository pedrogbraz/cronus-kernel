# Wave 1h — credit-card / floating-label / split-button kernel ports

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1h-b`
Branch: `feat/wave1h-card-float-split`
Base: `53428e2`
Date: 2026-09-13

Dedicated CONTRACT renderers for `credit-card-input`, `floating-label-input`, `split-button`.
Interact `input("text")` / `input("floating-label-input")` / `buttonish()` is skipped.
Zero JS, zero Voodoo. Tags and dock were not touched.

Helpers: `src/cronus_ui_kit.rs` (`esc`, `item`, `label_of`). Pattern: `src/cronus_ui_input.rs` / checkbox (no voodoo, no interact `input()` / `buttonish()`).

`PORTED_FAMILIES` is now 79 (76 after wave 1g + these 3).

## DOM (React slots)

| Family | Contract | Forbidden interact |
| --- | --- | --- |
| credit-card-input | `<div data-slot="credit-card-input">` + native number field `<input type="text" inputmode="numeric">` (plus expiry / CVC) | `<label data-slot="credit-card-input"><input type="text" data-slot="credit-card-input-control" style=CTRL>` |
| floating-label-input | `<div data-slot="floating-label-input"><label data-slot="floating-label-input-label">` + native `<input data-slot="input">` (no `*-control`) | interact `input("floating-label-input")` label wrapping `*-control` |
| split-button | `<div data-slot="split-button" role="group">` + primary `<button data-slot="button">` from label + chevron button | `buttonish()` single primary + inline SURF |

Credit-card-input never wraps the number field in `<label data-slot="credit-card-input">`. Floating-label-input uses a dedicated label slot plus a native input, not a `*-control`. Split-button is a fused group of two buttons (primary + chevron), not a single `buttonish()` primary.

## Wiring

- modules: `src/cronus_ui_credit_card_input.rs`, `src/cronus_ui_floating_label_input.rs`, `src/cronus_ui_split_button.rs`
- `mod` in `src/main.rs`
- `PORTED_FAMILIES` + `dedicated_render` in `src/cronus_ui_widgets.rs` and `scripts/gen_cronus_ui_widgets.py` (appended, not reordered)
- `dedicated_fn_name` + interact fingerprints + sidecar file list in `src/cli/stub_renderer_gate.rs`
- token-only `COMPONENT_CHROME` in `src/cronus_ui.rs` (`var(--cronus-*)`, no zinc palette)

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

## Evidence

```
cargo test cronus_ui_credit_card_input
# 6 passed (div + native number/expiry/CVC inputs, not label *-control)

cargo test cronus_ui_floating_label_input
# 6 passed (div + floating-label-input-label + native input, not *-control)

cargo test cronus_ui_split_button
# 6 passed (role=group, primary + chevron data-slot=button, not buttonish SURF)

cargo test stub_renderer_gate
# 7 passed (Dedicated, skip interact, no sidecar assets)
```

No `*-control`. No interact `<label data-slot>` wrap for credit-card. No inline CTRL / SURF. No Voodoo even with runtime on.

Commits on `feat/wave1h-card-float-split` (one per family):

1. `1ea097b` feat(ui): dedicated CreditCardInput renderer
2. `7cbb50d` feat(ui): dedicated FloatingLabelInput renderer
3. `55133e6` feat(ui): dedicated SplitButton renderer

Not pushed.
