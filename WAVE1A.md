# Cronus Audit Wave 1a — label, textarea, checkbox

Dedicated kernel ports. Interact / pill / field wrappers are skipped via `PORTED_FAMILIES`. Chrome is `var(--cronus-*)` only. Zero JS, zero Voodoo attrs, no HTML in `.cronus` files.

## DOM

| Family | Dedicated HTML | Forbidden interact fingerprint |
| --- | --- | --- |
| label | `<label data-slot="label">…</label>` | `<span data-slot="label" style=…>` (`pill`) |
| textarea | `<textarea data-slot="textarea" …>` | `<label data-slot="textarea"><textarea data-slot="textarea-control">` |
| checkbox | `<button type="button" data-slot="checkbox" role="checkbox" aria-checked="true\|false" data-state="checked\|unchecked">` | `<label data-slot="checkbox"><input type="checkbox" data-slot="checkbox-control">` |

Checked on checkbox: `props.checked`, item config `checked:true`, or style `+checked`.

## Files

New:

- `src/cronus_ui_label.rs` — `cronus_ui_label::render`
- `src/cronus_ui_textarea.rs` — `cronus_ui_textarea::render`
- `src/cronus_ui_checkbox.rs` — `cronus_ui_checkbox::render`

Wiring:

- `src/main.rs` — `mod cronus_ui_{label,textarea,checkbox}`
- `src/cronus_ui_widgets.rs` — `PORTED_FAMILIES` append `"label", "textarea", "checkbox"`; `dedicated_render` arms; skip interact
- `src/cli/stub_renderer_gate.rs` — `dedicated_fn_name` → `"cronus_ui_{family}::render"`
- `src/cronus_ui.rs` — `COMPONENT_CHROME` for label (text-sm / font-medium / fg), textarea (min-height 5rem field), checkbox (1rem rounded-sm, `[data-state=checked]` primary fill)
- `scripts/gen_cronus_ui_widgets.py` — keep generator template in sync

## Commits

```
888adc4 feat(ui): dedicated Label renderer
53abb52 feat(ui): dedicated Textarea renderer
dbe3963 feat(ui): dedicated Checkbox renderer
```

Not pushed.

## cargo test

Wave filters (`--test-threads=1`):

| Filter | Result |
| --- | --- |
| `cronus_ui_label` | 2 passed |
| `cronus_ui_textarea` | 4 passed |
| `cronus_ui_checkbox` | 5 passed |
| `stub_renderer_gate` | 7 passed |
| **wave total** | **18 passed** |

Full suite:

```
cargo test -- --test-threads=1
test result: ok. 301 passed; 0 failed; 0 ignored; 0 measured
```
