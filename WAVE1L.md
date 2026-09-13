# Cronus Audit Wave 1l — morphing-popover / bouncy-accordion / typing-text

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1l-b`  
Branch: `feat/wave1l-morph-bouncy-typing`  
Base: `1693093`  
Date: 2026-09-13

Done: 3 dedicated modules, interact `popover()` / `accordion()` and catalog `fx()` skipped, tests green. Not pushed.  
Not touched: meteors (harness `Stub("fx")`), sankey-chart, particles, timeline.

Helpers: `src/cronus_ui_kit.rs` (`label_of`, `texts`, `choice_texts`, `esc`, test `stub`). Pattern: popover / accordion / text-shimmer dedicated modules. Not interact SURF `<details>` and not catalog `fx()` title SURF box (`padding:0.75rem 1rem;position:relative;overflow:hidden` + inner `<span>`).

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `fe0b7b1` | feat(ui): dedicated MorphingPopover renderer |
| `b308dd2` | feat(ui): dedicated BouncyAccordion renderer |
| `9f7f9e8` | feat(ui): dedicated TypingText renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1k list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … text-shimmer …
    "morphing-popover",
    "bouncy-accordion",
    "typing-text",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| morphing-popover | `src/cronus_ui_morphing_popover.rs` | `cronus_ui_morphing_popover::render` |
| bouncy-accordion | `src/cronus_ui_bouncy_accordion.rs` | `cronus_ui_bouncy_accordion::render` |
| typing-text | `src/cronus_ui_typing_text.rs` | `cronus_ui_typing_text::render` |

`src/main.rs` mods: `cronus_ui_morphing_popover`, `cronus_ui_bouncy_accordion`, `cronus_ui_typing_text`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three (caret keyframes for typing-text).

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render` and **before** catalog `overlay()` / `nav()` / `fx()`. Catalog stub / interact arms remain for unported families and were not rewritten.

115 `PORTED_FAMILIES`. 175 `FAMILIES` (`bouncy-accordion` added as a dedicated family, not a stub). Remaining chart stub: sankey-chart. meteors still `Stub("fx")`.

Fingerprint: morphing-popover must not be `<details>`; bouncy-accordion must emit `bouncy-accordion-trigger` and must not be interact `accordion()` details; typing-text must not contain `padding:0.75rem 1rem;position:relative;overflow:hidden`.

## morphing-popover

React: root `data-slot="morphing-popover"` + trigger button `data-slot="morphing-popover-trigger"` + content `data-slot="morphing-popover-content"` (`packages/ui/src/components/morphing-popover.tsx`). Kernel is always-open (trigger + content both in the DOM). Not interact `popover("morphing-popover")` SURF `<details>`.

Dedicated DOM (static, zero JS):

```html
<div data-slot="morphing-popover" data-state="open"><button type="button" data-slot="morphing-popover-trigger" aria-haspopup="dialog" aria-expanded="true">Open menu</button><div data-slot="morphing-popover-content" data-state="open" role="dialog" aria-modal="false">First action</div></div>
```

Forbidden interact: `<details data-slot="morphing-popover" style="…">`. Asserts: no `<details`, no `<summary`, no `style=`, no `onclick`, no `v-data`.

Chrome: trigger `var(--cronus-surface-floating)` / `var(--cronus-border)`; content `z-index: 50` `width: 18rem` `box-shadow: var(--cronus-shadow-lg`.

## bouncy-accordion

React: stacked `<div data-slot="bouncy-accordion">` with per-item `data-slot="bouncy-accordion-trigger"` (`packages/ui/src/components/bouncy-accordion.tsx`). Kernel emits triggers from texts; first item open with content. Not interact `accordion()` SURF `<details>` (no `bouncy-accordion-trigger`).

Dedicated DOM (static, zero JS):

```html
<div data-slot="bouncy-accordion"><button type="button" data-slot="bouncy-accordion-trigger" aria-expanded="true">Type Shit</button><div data-slot="bouncy-accordion-content">Type Shit</div><button type="button" data-slot="bouncy-accordion-trigger" aria-expanded="false">Schedule</button></div>
```

Forbidden interact: `<div data-slot="accordion" style="…"><details open style="…">`. Asserts: no `<details`, no `<summary`, no `style=`, interact `bouncy-accordion` is `None`.

Chrome: `max-width: 300px` `min-height: 45px` `background: var(--cronus-surface-base)` content `var(--cronus-fg-tertiary`.

## typing-text

React: `<span data-slot="typing-text">` typewriter (`packages/ui/src/components/typing-text.tsx`). Kernel is static full label text (no JS typewriter). CSS caret via `::after` + `@keyframes cui-typing-caret`. Not catalog `fx("typing-text")` SURF title span.

Dedicated DOM (static, zero JS):

```html
<span data-slot="typing-text">Ship faster</span>
```

Forbidden fx: wrapping `<div data-slot="typing-text" style="…padding:0.75rem 1rem;position:relative;overflow:hidden;"><span>…</span></div>`. Asserts: no `<div`, no `style=`, no SURF, no `setTimeout`.

Chrome: `white-space: pre-wrap`; caret `var(--cronus-fg)` blink; `prefers-reduced-motion` hides `::after`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_morphing_popover
  running 5 tests
  test cronus_ui_morphing_popover::tests::chrome_is_token_only ... ok
  test cronus_ui_morphing_popover::tests::content_slot_is_the_contract ... ok
  test cronus_ui_morphing_popover::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_morphing_popover::tests::trigger_button_and_always_open_content ... ok
  test cronus_ui_morphing_popover::tests::skips_interact_surf_details ... ok
  test result: ok. 5 passed

cargo test --offline cronus_ui_bouncy_accordion
  running 5 tests
  test cronus_ui_bouncy_accordion::tests::label_only_opens_first_with_content ... ok
  test cronus_ui_bouncy_accordion::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_bouncy_accordion::tests::skips_interact_accordion_surf_details ... ok
  test cronus_ui_bouncy_accordion::tests::chrome_is_token_only ... ok
  test cronus_ui_bouncy_accordion::tests::root_items_first_open_with_content ... ok
  test result: ok. 5 passed

cargo test --offline cronus_ui_typing_text
  running 5 tests
  test cronus_ui_typing_text::tests::label_is_escaped ... ok
  test cronus_ui_typing_text::tests::root_is_span_with_full_label_not_fx_title_box ... ok
  test cronus_ui_typing_text::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_typing_text::tests::chrome_typing_caret_via_css ... ok
  test cronus_ui_typing_text::tests::skips_fx_surf_title_box ... ok
  test result: ok. 5 passed

cargo test --offline stub_renderer_gate
  running 7 tests
  test cli::stub_renderer_gate::tests::meteors_is_stub_fx ... ok
  test cli::stub_renderer_gate::tests::sankey_chart_is_stub ... ok
  test cli::stub_renderer_gate::tests::every_family_is_classified ... ok
  test cli::stub_renderer_gate::tests::button_is_dedicated ... ok
  test cli::stub_renderer_gate::tests::no_new_stub_families ... ok
  test cli::stub_renderer_gate::tests::ported_families_are_dedicated_and_skip_interact ... ok
  test cli::stub_renderer_gate::tests::dedicated_module_no_sidecar_assets ... ok
  test result: ok. 7 passed
```

`meteors_is_stub_fx` asserts `renderer_kind("meteors") == Stub("fx")` and the catalog fx fingerprint. `sankey_chart_is_stub` remains `Stub("chart")`.

Wave total: **22 passed** (5 + 5 + 5 + 7).
