# Cronus Audit Wave 1o — countdown / animated-button / card-stack

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1o-b`  
Branch: `feat/wave1o-count-anim-stack`  
Base: `24c4a6f`  
Date: 2026-09-13

Done: 3 dedicated modules, catalog `fx()` / `display()` skipped, interact skipped, tests green. Not pushed.  
Not touched: meteors (harness `Stub("fx")`), sankey-chart, aspect-ratio, charts.

Helpers: `src/cronus_ui_kit.rs` (`texts`, `label_of`, `esc`, `numeric_items`, test `stub`). Pattern: `src/cronus_ui_card.rs` / `src/cronus_ui_copy_button.rs` (dedicated slots + CSS chrome). Not catalog `fx()` title SURF box (`padding:0.75rem 1rem;position:relative;overflow:hidden` + inner `<span>`). Not catalog `display()` SURF `<section>`. Not interact generic HTML. Not voodoo.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `87f5274` | feat(ui): dedicated Countdown renderer |
| `87c74a4` | feat(ui): dedicated AnimatedButton renderer |
| `dc5f080` | feat(ui): dedicated CardStack renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1n list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … shiny-text …
    "countdown",
    "animated-button",
    "card-stack",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| countdown | `src/cronus_ui_countdown.rs` | `cronus_ui_countdown::render` (not `fx`) |
| animated-button | `src/cronus_ui_animated_button.rs` | `cronus_ui_animated_button::render` (not `fx`) |
| card-stack | `src/cronus_ui_card_stack.rs` | `cronus_ui_card_stack::render` (not `display`) |

`src/main.rs` mods: `cronus_ui_countdown`, `cronus_ui_animated_button`, `cronus_ui_card_stack`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three families.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render` and **before** catalog `fx()` / `display()`. Catalog stub / interact arms remain for unported families and were not rewritten.

142 `PORTED_FAMILIES`. 175 `FAMILIES`. Remaining chart stub: sankey-chart. meteors still `Stub("fx")`.

Fingerprint: dedicated HTML must not contain fx box / missing `countdown-unit`+`countdown-value`+`countdown-label` / timer JS (countdown), fx box / inner `<div>`/`<span>` (animated-button), display SURF `<section>` / missing `card-stack-item` (card-stack).

## countdown

React: `<div data-slot="countdown">` four `countdown-unit` tiles with `countdown-value` / `countdown-label` and a JS `setTimeout` tick (`packages/ui/src/components/countdown.tsx`). Kernel is **static**: numbers from texts / `00:00:00` clock strings / numeric items (right-aligned into days/hours/min/sec). Zero JS timer. Not catalog `fx()` SURF title box.

Dedicated DOM (static, zero JS):

```html
<div data-slot="countdown" role="timer" aria-live="off"><div data-slot="countdown-unit" data-unit="hours" aria-hidden="true"><span data-slot="countdown-value">00</span><span data-slot="countdown-label">hours</span></div></div>
```

Forbidden fx: `<div data-slot="countdown" style="…padding:0.75rem 1rem;position:relative;overflow:hidden;"><span>Demo</span></div>`. Asserts: no `style=`, no `setInterval` / `setTimeout` / `Date.now`, `dedicated_fn_name("countdown") == Some("cronus_ui_countdown::render")`, `renderer_kind("meteors") == Stub("fx")`.

Chrome: inline-flex tiles; unit `var(--cronus-surface-raised)` + `var(--cronus-border)`; value `font-variant-numeric: tabular-nums`; label `var(--cronus-fg-tertiary)`.

## animated-button

React: `<motion.button data-slot="animated-button">` with hover/tap springs (`packages/ui/src/components/animated-button.tsx`). Kernel is a **static** `<button type="button" data-slot="animated-button">` with label. CSS hover (`translateY(-1px)` / active `scale(0.97)`) in COMPONENT_CHROME. Not catalog `fx()` SURF title box.

Dedicated DOM (static, CSS hover OK):

```html
<button type="button" data-slot="animated-button">Launch</button>
```

Forbidden fx: `<div data-slot="animated-button" style="…padding:0.75rem 1rem;position:relative;overflow:hidden;"><span>Demo</span></div>`. Asserts: no `<div`, no `<span`, no `style=`, `dedicated_fn_name("animated-button") == Some("cronus_ui_animated_button::render")`, `renderer_kind("meteors") == Stub("fx")`.

Chrome: primary button tokens; `:hover { transform: translateY(-1px); }`; `:active { transform: scale(0.97); }`; reduced-motion disables transform.

## card-stack

React: `<section data-slot="card-stack">` of motion `card-stack-item` cards with cycle-on-click (`packages/ui/src/components/card-stack.tsx`). Kernel is a **static stack**: `<div data-slot="card-stack">` plus 2–3 `card-stack-item` from texts. CSS offset in COMPONENT_CHROME. Not catalog `display()` SURF `<section>`. Not interact `card()`.

Dedicated DOM (static, CSS offset OK):

```html
<div data-slot="card-stack"><div data-slot="card-stack-item">Alpha</div><div data-slot="card-stack-item">Beta</div></div>
```

Forbidden display: `<section data-slot="card-stack" style="…padding:1rem;display:flex;flex-direction:column;gap:0.5rem;">`. Forbidden interact: SURF `card()` `<section>`. Asserts: no `<section`, no `style=`, `dedicated_fn_name("card-stack") == Some("cronus_ui_card_stack::render")`, `renderer_kind("sankey-chart") == Stub("chart")`.

Glass-card / tilt-card skip-display tests now contrast against unported `flip-card` (card-stack is no longer a display stub).

Chrome: relative isolate stack; items absolute; nth-child translate/scale offsets; `var(--cronus-surface-raised)` + `var(--cronus-border)`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_countdown
  running 9 tests
  test cronus_ui_countdown::tests::chrome_is_token_only ... ok
  test cronus_ui_countdown::tests::label_is_escaped_out_of_tiles ... ok
  test cronus_ui_countdown::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_countdown::tests::root_is_timer_with_four_static_units_not_fx_title_box ... ok
  test cronus_ui_countdown::tests::clock_text_fills_static_values ... ok
  test cronus_ui_countdown::tests::clock_from_props_value ... ok
  test cronus_ui_countdown::tests::numeric_texts_fill_units_from_the_right ... ok
  test cronus_ui_countdown::tests::three_numbers_are_hours_minutes_seconds ... ok
  test cronus_ui_countdown::tests::skips_fx_surf_title_box ... ok
  test result: ok. 9 passed

cargo test --offline cronus_ui_animated_button
  running 5 tests
  test cronus_ui_animated_button::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_animated_button::tests::label_is_escaped ... ok
  test cronus_ui_animated_button::tests::root_is_button_with_label_not_fx_title_box ... ok
  test cronus_ui_animated_button::tests::skips_fx_surf_title_box ... ok
  test cronus_ui_animated_button::tests::chrome_hover_via_css ... ok
  test result: ok. 5 passed

cargo test --offline cronus_ui_card_stack
  running 8 tests
  test cronus_ui_card_stack::tests::chrome_offset_via_css ... ok
  test cronus_ui_card_stack::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_card_stack::tests::extra_texts_capped_at_three ... ok
  test cronus_ui_card_stack::tests::three_texts_stack ... ok
  test cronus_ui_card_stack::tests::root_is_div_with_stacked_items_not_display_section ... ok
  test cronus_ui_card_stack::tests::label_only_still_emits_two_items ... ok
  test cronus_ui_card_stack::tests::label_is_escaped ... ok
  test cronus_ui_card_stack::tests::skips_display_surf_and_interact_card ... ok
  test result: ok. 8 passed

cargo test --offline stub_renderer_gate
  running 7 tests
  test cli::stub_renderer_gate::tests::meteors_is_stub_fx ... ok
  test cli::stub_renderer_gate::tests::every_family_is_classified ... ok
  test cli::stub_renderer_gate::tests::button_is_dedicated ... ok
  test cli::stub_renderer_gate::tests::sankey_chart_is_stub ... ok
  test cli::stub_renderer_gate::tests::no_new_stub_families ... ok
  test cli::stub_renderer_gate::tests::ported_families_are_dedicated_and_skip_interact ... ok
  test cli::stub_renderer_gate::tests::dedicated_module_no_sidecar_assets ... ok
  test result: ok. 7 passed
```

`meteors_is_stub_fx` asserts `renderer_kind("meteors") == Stub("fx")` and the catalog fx fingerprint. `sankey_chart_is_stub` remains `Stub("chart")`. Countdown / animated-button `dedicated_fn_name` is the named module `render`, not `fx`. Card-stack `dedicated_fn_name` is `cronus_ui_card_stack::render`, not `display`.

Wave total: **29 passed** (9 + 5 + 8 + 7).
