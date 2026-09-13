# Wave 1b — slider, radio-group, chip dedicated kernel ports

`PORTED_FAMILIES` after Wave 1a: slider, radio-group, chip.

Each family has a dedicated `cronus_ui_{family}.rs` (`radio-group` → `cronus_ui_radio_group.rs`). Interact is skipped. No stubs.

- **slider** — Radix `<span data-slot="slider">` + track/range + thumb `role="slider"`. Not `<label>` / `<input type="range">`.
- **radio-group** — `<div data-slot="radio-group" role="radiogroup">` + `<button data-slot="radio-group-item" role="radio">`. Not native radios.
- **chip** — `<span data-slot="chip">`. React does not emit `data-size`. Not interact `pill()`.
