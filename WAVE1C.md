# Wave 1c — kernel ports (copy-button, fab, toggle-group)

Dedicated CONTRACT renderers. Interact skipped. Zero JS, zero Voodoo.
Kit helpers (`cronus_ui_kit`), checkbox-style static HTML. Not password-input.

Branch: `feat/wave1c-copy-fab-toggle-group`  
Base: `feat/cronus-ui-tokens-button` @ ef08106

## Families

| Family | Module | Idle DOM | Forbidden interact |
| --- | --- | --- | --- |
| copy-button | `src/cronus_ui_copy_button.rs` | `<button type="button" data-slot="copy-button" aria-label="Copy">` + label text; optional `data-copy` from `props.value` / label | inline BASE/SURF + `onclick="navigator.clipboard.writeText…"` |
| fab | `src/cronus_ui_fab.rs` | `<div data-slot="fab"><button type="button" data-slot="fab-button" aria-label>` | `buttonish()`: `<div data-slot="fab" style=><button data-slot="button" style=primary>` |
| toggle-group | `src/cronus_ui_toggle_group.rs` | `<div data-slot="toggle-group" role="group">` + `<button type="button" data-slot="toggle-group-item" data-state="on\|off" aria-pressed>` from text items; first or `pressed:true` is on | `radios()`: native `<input type="radio">` labels, no `toggle-group-item` |

Chrome is token-only CSS in `COMPONENT_CHROME` (`src/cronus_ui.rs`). No `zinc-`, no inline styles, no `onclick`.

## Wiring

`PORTED_FAMILIES` appends `"copy-button", "fab", "toggle-group"`.

Also: `mod` in `src/main.rs`, `dedicated_render` in `src/cronus_ui_widgets.rs`, `dedicated_fn_name` + interact fingerprints + sidecar file list in `src/cli/stub_renderer_gate.rs`, generator `scripts/gen_cronus_ui_widgets.py`.

## Commits

1. `b46343c` feat(ui): dedicated CopyButton renderer
2. `59f8583` feat(ui): dedicated Fab renderer
3. `7275ff8` feat(ui): dedicated ToggleGroup renderer

## Tests

```
cargo test cronus_ui_copy_button
cargo test cronus_ui_fab
cargo test cronus_ui_toggle_group
cargo test stub_renderer_gate
```

Evidence (worktree `/Users/pedrogbraz/projects/cooud/.wt/kernel-1c-b`):

```
cronus_ui_copy_button:  5 passed; 0 failed
cronus_ui_fab:          4 passed; 0 failed
cronus_ui_toggle_group: 6 passed; 0 failed
stub_renderer_gate:     7 passed; 0 failed
```

`ported_families_are_dedicated_and_skip_interact` covers the three new families.
Not pushed.
