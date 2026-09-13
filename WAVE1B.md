# Cronus Audit Wave 1b — avatar / card / empty

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1b-c`  
Branch: `feat/wave1b-avatar-card-empty`  
Base: `feat/cronus-ui-tokens-button` @ `9066344`  
Date: 2026-09-13

Done: 3 dedicated modules, interact skipped, tests green. Not pushed.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `1a99247` | feat(ui): dedicated Avatar renderer |
| `96b7d63` | feat(ui): dedicated Card renderer |
| `0df4881` | feat(ui): dedicated Empty renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    "button", "badge", "input", "label", "textarea", "checkbox",
    "switch", "spinner", "separator", "kbd", "toggle", "progress",
    "avatar", "card", "empty",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| avatar | `src/cronus_ui_avatar.rs` | `cronus_ui_avatar::render` |
| card | `src/cronus_ui_card.rs` | `cronus_ui_card::render` |
| empty | `src/cronus_ui_empty.rs` | `cronus_ui_empty::render` |

`src/main.rs` mods: `cronus_ui_avatar`, `cronus_ui_card`, `cronus_ui_empty`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render`. Catalog stub / interact arms remain for unported families (`alert`, `slider`, `chip`, `avatar-group`, `card-stack`, …) and were not touched.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## avatar

React: Radix Root is **span** (`packages/ui/src/components/avatar.tsx`).

Dedicated DOM:

```html
<span data-slot="avatar"><span data-slot="avatar-fallback">JD</span></span>
```

Fallback initials from the label (1–2 chars: first letters of two words, else first two of one word). Optional image: if a url item/link exists, also `<img data-slot="avatar-image" src>`.

Forbidden interact (`avatar()`): `<div data-slot="avatar" style="width:2.25rem;…">` single letter, no `avatar-fallback` slot.

Chrome (`COMPONENT_CHROME`): `relative flex size-10` (2.5rem) `shrink-0 overflow-hidden rounded-full`; fallback `surface-overlay` `text-sm`.

## card

React: root is **div**, not `<section>` (`packages/ui/src/components/card.tsx`).

Dedicated DOM:

```html
<div data-slot="card"><div data-slot="card-header"><div data-slot="card-title">Overview</div></div></div>
```

Title from label. Optional extra text → `data-slot="card-description"`. Further items → `data-slot="card-content"`.

Forbidden interact (`card()`): `<section data-slot="card" style="…SURF…">` without `card-title` slot.

Chrome: `flex flex-col gap-6` (1.5rem) `rounded-xl border surface-raised py-6 shadow-sm`; title `font-medium`; description `text-sm fg-secondary` with horizontal padding.

## empty

React: `<div data-slot="empty">` (`packages/ui/src/components/empty.tsx`).

Dedicated DOM:

```html
<div data-slot="empty"><div data-slot="empty-title">No results</div></div>
```

Title from label. Optional extra text → `data-slot="empty-description"`.

Forbidden interact (`alert("empty")`): SURF box with raw divs, `role="status"`, no `empty-title`.

Chrome: `flex flex-col items-center gap-3` (0.75rem) `rounded-xl border-dashed px-6 py-12 text-center`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test cronus_ui_avatar
  running 6 tests
  test cronus_ui_avatar::tests::chrome_is_token_only ... ok
  test cronus_ui_avatar::tests::image_from_item_link ... ok
  test cronus_ui_avatar::tests::image_from_source_item ... ok
  test cronus_ui_avatar::tests::initials_from_two_words ... ok
  test cronus_ui_avatar::tests::root_is_span_with_fallback_not_div ... ok
  test cronus_ui_avatar::tests::skips_interact_div ... ok
  test result: ok. 6 passed

cargo test cronus_ui_card
  running 5 tests
  test cronus_ui_card::tests::chrome_is_token_only ... ok
  test cronus_ui_card::tests::content_when_more ... ok
  test cronus_ui_card::tests::description_from_extra_text ... ok
  test cronus_ui_card::tests::root_is_div_with_title_not_section ... ok
  test cronus_ui_card::tests::skips_interact_section ... ok
  test result: ok. 5 passed

cargo test cronus_ui_empty
  running 4 tests
  test cronus_ui_empty::tests::chrome_is_token_only ... ok
  test cronus_ui_empty::tests::description_from_extra_text ... ok
  test cronus_ui_empty::tests::root_is_div_with_title_not_alert_surf ... ok
  test cronus_ui_empty::tests::skips_interact_alert ... ok
  test result: ok. 4 passed

cargo test stub_renderer_gate
  running 7 tests
  test cli::stub_renderer_gate::tests::area_chart_is_stub ... ok
  test cli::stub_renderer_gate::tests::button_is_dedicated ... ok
  test cli::stub_renderer_gate::tests::dedicated_module_no_sidecar_assets ... ok
  test cli::stub_renderer_gate::tests::every_family_is_classified ... ok
  test cli::stub_renderer_gate::tests::meteors_is_stub_fx ... ok
  test cli::stub_renderer_gate::tests::no_new_stub_families ... ok
  test cli::stub_renderer_gate::tests::ported_families_are_dedicated_and_skip_interact ... ok
  test result: ok. 7 passed
```

Also green: `cargo test cronus_ui_widgets` (14 passed), including `ported_family_skips_interact` over the appended list.
