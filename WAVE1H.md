# Wave 1h — tags-input / autocomplete / multi-select kernel ports

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1h-a`
Branch: `feat/wave1h-tags-auto-multi`
Base: `53428e2`
Date: 2026-09-13

Dedicated CONTRACT renderers for `tags-input`, `autocomplete`, `multi-select`.
Interact `select("tags-input")` / `select("autocomplete")` / `select("multi-select")`
is skipped. Zero JS, zero Voodoo. Credit-card and dock were not touched.

Helpers: `src/cronus_ui_kit.rs` (`choice_texts`, `esc`, `item`, `texts`).
Pattern: `src/cronus_ui_combobox.rs` / `src/cronus_ui_input.rs` (no voodoo, no interact `select()`).

`PORTED_FAMILIES` is now 79 (76 after wave 1g + these 3).

## DOM (React slots)

| Family | Contract | Forbidden interact |
| --- | --- | --- |
| tags-input | `<div data-slot="tags-input">` chips (`tags-input-item` + `tags-input-remove`) from extra texts + `<input data-slot="tags-input-field">` | `<label data-slot="tags-input"><select data-slot="tags-input-control">` |
| autocomplete | `<div data-slot="autocomplete"><input data-slot="autocomplete-input">` + always-open `<div data-slot="autocomplete-content">` options from extra texts | `select("autocomplete")` native `<select>` + `*-control` |
| multi-select | wrapper `data-slot="multi-select"` + `<button data-slot="multi-select-trigger">` + open list of options from texts | `select("multi-select")` native `<select multiple>` + `*-control` |

Tags-input never wraps a native `<select>` in `<label data-slot="tags-input">`. Autocomplete is a text combobox with an always-open listbox, not a native select. Multi-select is a button trigger plus open option list, not `<select multiple>`.

No `*-control`. No `<select>`.

## Wiring

- modules: `src/cronus_ui_tags_input.rs`, `src/cronus_ui_autocomplete.rs`, `src/cronus_ui_multi_select.rs`
- `mod` in `src/main.rs`
- `PORTED_FAMILIES` + `dedicated_render` in `src/cronus_ui_widgets.rs` and `scripts/gen_cronus_ui_widgets.py` (appended, not reordered)
- `dedicated_fn_name` + interact fingerprints + sidecar file list in `src/cli/stub_renderer_gate.rs`
- token-only `COMPONENT_CHROME` in `src/cronus_ui.rs` (`var(--cronus-*)`, no zinc palette)

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families and were not rewritten.

## Evidence

```
cargo test cronus_ui_tags_input
# 8 passed (chips + tags-input-field, not label/select *-control)

cargo test cronus_ui_autocomplete
# 9 passed (input + always-open content options, not native select)

cargo test cronus_ui_multi_select
# 10 passed (wrapper + trigger button + open list, not select multiple)

cargo test stub_renderer_gate
# 7 passed (Dedicated, skip interact, no sidecar assets)
```

No `*-control`. No `<select>`. No interact `<label data-slot>` wrap. No inline CTRL. No Voodoo even with runtime on.

Commits on `feat/wave1h-tags-auto-multi` (one per family):

1. `e6d4b5c` feat(ui): dedicated TagsInput renderer
2. `fbf2d64` feat(ui): dedicated Autocomplete renderer
3. `60e50d3` feat(ui): dedicated MultiSelect renderer

Not pushed.
