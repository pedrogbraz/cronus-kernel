# Wave 1a — kernel ports (merged)

9 dedicated families beyond button/badge/input. Interact skipped via `PORTED_FAMILIES`.

| Family | Module | DOM |
|---|---|---|
| label | `cronus_ui_label.rs` | `<label data-slot="label">` |
| textarea | `cronus_ui_textarea.rs` | `<textarea data-slot="textarea">` |
| checkbox | `cronus_ui_checkbox.rs` | `<button data-slot="checkbox" role="checkbox">` |
| switch | `cronus_ui_switch.rs` | `<button data-slot="switch" role="switch">` |
| spinner | `cronus_ui_spinner.rs` | `<svg data-slot="spinner" role="status">` |
| separator | `cronus_ui_separator.rs` | `<div data-slot="separator">` |
| kbd | `cronus_ui_kbd.rs` | `<kbd data-slot="kbd">` (pending merge C) |
| toggle | `cronus_ui_toggle.rs` | `<button data-slot="toggle">` (pending merge C) |
| progress | `cronus_ui_progress.rs` | `<div data-slot="progress" role="progressbar">` (pending merge C) |
