# Cronus Audit Wave 1a — switch, spinner, separator

Kernel worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-b`  
Branch: `feat/wave1a-switch-spinner-separator`  
Base: `feat/cronus-ui-tokens-button` @ `5e7e8d7`

Dedicated CONTRACT ports for three pill-catalog families. Interact generic HTML is skipped. Zero JS, zero Voodoo, no HTML in `.cronus`. Token CSS only (`var(--cronus-*)`).

## Commits (one per family)

| SHA | Family | Message |
|-----|--------|---------|
| `3787783` | switch | `feat(ui): dedicated Switch renderer` |
| `6f4adc2` | spinner | `feat(ui): dedicated Spinner renderer` |
| `013d52c` | separator | `feat(ui): dedicated Separator renderer` |

Not pushed. Did not touch label / textarea / checkbox / kbd / toggle / progress.

## Modules

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| switch | `src/cronus_ui_switch.rs` | `cronus_ui_switch::render` |
| spinner | `src/cronus_ui_spinner.rs` | `cronus_ui_spinner::render` |
| separator | `src/cronus_ui_separator.rs` | `cronus_ui_separator::render` |

`mod` in `src/main.rs`. Appended to `PORTED_FAMILIES`:

```rust
pub const PORTED_FAMILIES: &[&str] = &["button", "badge", "input", "switch", "spinner", "separator"];
```

`dedicated_render` arms call the named modules. `scripts/gen_cronus_ui_widgets.py` kept in sync so a regen does not drop the ports.

## DOM vs forbidden interact

### switch

React: Radix Switch Root (`button`) + Thumb.

Dedicated:

```html
<button type="button" data-slot="switch" role="switch" aria-checked="false" data-state="unchecked" aria-label="…">
  <span data-slot="switch-thumb"></span>
</button>
```

- `aria-checked` / `data-state` from `props.checked` or item config `checked:true`.
- Default unchecked.

Forbidden interact (not emitted):

```html
<label data-slot="switch"><input type="checkbox" role="switch" data-slot="switch-control">
```

### spinner

React: SVG (`spinner.tsx`), not a CSS border div.

Dedicated:

```html
<svg data-slot="spinner" role="status" aria-label="Loading" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
  <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" opacity="0.25"></circle>
  <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" opacity="0.9"></path>
</svg>
```

Default `aria-label="Loading"`. Override via `props["aria-label"]`.

Forbidden interact (not emitted):

```html
<div data-slot="spinner" … border-top-color>
```

### separator

React: Radix Separator Root is a **div**, `decorative=true` by default.

Dedicated:

```html
<div data-slot="separator" data-orientation="horizontal"></div>
```

- Decorative → no `role`. `decorative:false` → `role="separator"` + `aria-orientation`.
- Orientation from `props.orientation` or style `+vertical`.

Forbidden interact (not emitted):

```html
<hr data-slot="separator" style="border:0;border-top:…">
```

## COMPONENT_CHROME (token CSS only)

All in `src/cronus_ui.rs` `COMPONENT_CHROME`. No palette scales.

- **switch:** `h-5 w-9` (1.25rem × 2.25rem) `rounded-full`; thumb `1rem`; unchecked `var(--cronus-fg-tertiary)`; checked `var(--cronus-primary)`; thumb `var(--cronus-primary-foreground)`, slides `18px` when checked.
- **spinner:** `size-5` (1.25rem); `animation: spin 1s linear infinite` with `@keyframes spin` in chrome (audit stylesheet does not include layout `.animate-spin`). Not the interact border-div.
- **separator:** horizontal `height: 1px; width: 100%`; vertical `height: 100%; width: 1px`; `background: var(--cronus-border)`.

`prefers-reduced-motion` already disables `[data-slot]` animation in `FALLBACK_ROOT`.

## Tests

Command (libtest filter is a regex; run as separate filters — `cargo test A B` is not valid Cargo syntax):

```
cargo test cronus_ui_switch
cargo test cronus_ui_spinner
cargo test cronus_ui_separator
cargo test stub_renderer_gate
```

Results (2026-09-13, this worktree):

```
cronus_ui_switch     4 passed
cronus_ui_spinner    3 passed
cronus_ui_separator  5 passed
stub_renderer_gate   7 passed
```

Also green: `ported_family_skips_interact`, `ported_families_are_dedicated_and_skip_interact`, `interactive_families_emit_real_controls`, `interactive_controls_are_labelled`, `every_family_renders_slot_without_palette_scales`.

Family tests prove tag/slot and **not** interact HTML. Gate tests prove `RendererKind::Dedicated` and HTML ≠ `cronus_ui_interact::render`.

## Diff vs base

```
 scripts/gen_cronus_ui_widgets.py |   5 +-
 src/cli/stub_renderer_gate.rs    |  10 +++
 src/cronus_ui.rs                 |  43 ++++++++++++
 src/cronus_ui_separator.rs       | 129 ++++++++++++++++++++++++++++++++++++
 src/cronus_ui_spinner.rs         |  97 +++++++++++++++++++++++++++
 src/cronus_ui_switch.rs          | 137 +++++++++++++++++++++++++++++++++++++++
 src/cronus_ui_widgets.rs         |   5 +-
 src/main.rs                      |   3 +
 8 files changed, 427 insertions(+), 2 deletions(-)
```

Done = 3 dedicated modules, skip interact, tests green.
