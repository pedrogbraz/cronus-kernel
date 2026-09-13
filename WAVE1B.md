# Cronus Audit Wave 1b — alert, skeleton, banner

Dedicated kernel ports. Interact / display / pill wrappers are skipped via `PORTED_FAMILIES`. Chrome is `var(--cronus-*)` only. Zero JS, zero Voodoo attrs, no HTML in `.cronus` files.

`PORTED_FAMILIES` after button/badge/input/label/textarea/checkbox/switch/spinner/separator/kbd/toggle/progress: **alert, skeleton, banner**.

## DOM

| Family | Dedicated HTML | Forbidden interact fingerprint |
| --- | --- | --- |
| alert | `<div data-slot="alert" role="status">` with `<div data-slot="alert-title">` from label/title and optional `<div data-slot="alert-description">` from extra text. `role="alert"` only if variant is destructive. React does **not** emit `data-variant`; ports do not require it. | `<div data-slot="alert" role="status" style="…BASE SURF…">` wrapping raw `<div>text</div>` with no `alert-title` slot |
| skeleton | `<div data-slot="skeleton" aria-hidden="true"></div>` (empty). Default size ~8rem × 0.9rem via CSS, not inline. | `height:0.9rem;width:8rem;…background:var(--cronus-border)` inline dump |
| banner | `<section data-slot="banner" aria-label="Announcement">` containing `<div data-slot="banner-content">` + `<span data-slot="banner-title">`. Static section — no `motion.div` / JS. | Same `alert("banner")` SURF box (`<div data-slot="banner" role="status" style=…>`) |

## Chrome (`COMPONENT_CHROME`, token CSS only)

- **alert:** rounded-lg border, `px-4 py-3` (`0.75rem 1rem`), text-sm, `var(--cronus-surface-overlay)`
- **skeleton:** animate-pulse (`cui-pulse`), rounded-md, `var(--cronus-surface-overlay)`; default `width: 8rem; height: 0.9rem`
- **banner:** flex w-full items-center gap, border-b, `px-4 py-2.5` (`0.625rem 1rem`), `var(--cronus-surface-overlay)`

## Files

New:

- `src/cronus_ui_alert.rs` — `cronus_ui_alert::render`
- `src/cronus_ui_skeleton.rs` — `cronus_ui_skeleton::render`
- `src/cronus_ui_banner.rs` — `cronus_ui_banner::render`

Wiring:

- `src/main.rs` — `mod cronus_ui_{alert,skeleton,banner}` (alpha-ish with existing `cronus_ui_*`)
- `src/cronus_ui_widgets.rs` — `PORTED_FAMILIES` append `"alert", "skeleton", "banner"`; `dedicated_render` arms; skip interact
- `src/cli/stub_renderer_gate.rs` — `dedicated_fn_name` → `"cronus_ui_{family}::render"`; interact fingerprints; sidecar file list
- `src/cronus_ui.rs` — `COMPONENT_CHROME` as above
- `scripts/gen_cronus_ui_widgets.py` — generator template in sync

Not touched: slider, radio-group, chip, avatar, card, empty.

## Commits

```
a253f13 feat(ui): dedicated Alert renderer
25b5961 feat(ui): dedicated Skeleton renderer
747254c feat(ui): dedicated Banner renderer
```

Not pushed.

## cargo test

Wave filters (`cargo test --offline <filter>`):

| Filter | Result |
| --- | --- |
| `cronus_ui_alert` | 6 passed |
| `cronus_ui_skeleton` | 3 passed |
| `cronus_ui_banner` | 5 passed |
| `stub_renderer_gate` | 7 passed |
| **wave total** | **21 passed** |

Full suite:

```
cargo test --offline
test result: ok. 342 passed; 0 failed; 0 ignored; 0 measured
```
