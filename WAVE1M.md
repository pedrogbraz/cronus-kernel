# Cronus Audit Wave 1m — spotlight-card / animated-list / toast

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1m-c`  
Branch: `feat/wave1m-spot-list-toast`  
Base: `0054828`  
Date: 2026-09-13

Done: 3 dedicated modules, catalog `display()` / `fx()` skipped, interact skipped, tests green. Not pushed.  
Not touched: meteors (harness `Stub("fx")`), sankey-chart, terminal, tilt-card. Do not confuse `toast` with already-ported **sonner**.

Helpers: `src/cronus_ui_kit.rs` (`texts`, `label_of`, `esc`, test `stub`). Pattern: `src/cronus_ui_card.rs` / `src/cronus_ui_sonner.rs` (dedicated slots + CSS chrome). Not catalog `display()` SURF `<section>`. Not catalog `fx()` title SURF box (`padding:0.75rem 1rem;position:relative;overflow:hidden` + inner `<span>`). Not interact generic HTML. Not voodoo.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `ecaf777` | feat(ui): dedicated SpotlightCard renderer |
| `a758652` | feat(ui): dedicated AnimatedList renderer |
| `b1982c6` | feat(ui): dedicated Toast renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1l list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … tree-view …
    "spotlight-card",
    "animated-list",
    "toast",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| spotlight-card | `src/cronus_ui_spotlight_card.rs` | `cronus_ui_spotlight_card::render` |
| animated-list | `src/cronus_ui_animated_list.rs` | `cronus_ui_animated_list::render` |
| toast | `src/cronus_ui_toast.rs` | `cronus_ui_toast::render` (not `fx`) |

`src/main.rs` mods: `cronus_ui_spotlight_card`, `cronus_ui_animated_list`, `cronus_ui_toast`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for spotlight + animated-list; toast reuses the existing `[data-slot="toast"]` token block (not sonner's fixed toaster wrapper).

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render` and **before** catalog `fx()` / `display()`. Catalog stub / interact arms remain for unported families and were not rewritten.

124 `PORTED_FAMILIES`. 175 `FAMILIES`. Remaining chart stub: sankey-chart. meteors still `Stub("fx")`.

Fingerprint: dedicated HTML must not contain `padding:1rem;display:flex;flex-direction:column;gap:0.5rem` / `<section>` (spotlight-card), fx box / missing `animated-list-item` (animated-list), fx box or inline `style=` without sonner/toaster (toast).

## spotlight-card

React: `<div data-slot="spotlight-card">` with a pointer-following radial overlay (`packages/ui/src/components/spotlight-card.tsx`). Kernel wraps the **label** in that slot. CSS spotlight (`::after` at `--spot-x` / `--spot-y`, hover opacity, `color-mix` of `var(--cronus-primary)`). Zero JS (no rAF pointer tracking). Not catalog `display()` SURF `<section>`. Not interact `card()`.

Dedicated DOM (static, zero JS):

```html
<div data-slot="spotlight-card">Hover me</div>
```

Forbidden display: `<section data-slot="spotlight-card" style="…padding:1rem;display:flex;flex-direction:column;gap:0.5rem;"><div style="font-weight:500;">…`. Asserts: no `<section`, no `style=`, no SURF.

Chrome: `position: relative` `overflow: hidden` `padding: 1.5rem`; `::after` `radial-gradient(320px circle at var(--spot-x, 50%) var(--spot-y, 50%), …)`.

## animated-list

React: `<ul data-slot="animated-list">` wrapping each child in `<li data-slot="animated-list-item">` with motion stagger (`packages/ui/src/components/animated-list.tsx`). Kernel is a **static list** from `texts`. CSS rise/fade (`cui-animated-list`, nth-child delays). Zero JS. Not catalog `fx()` SURF title box.

Dedicated DOM (static, zero JS):

```html
<ul data-slot="animated-list"><li data-slot="animated-list-item">Alpha</li><li data-slot="animated-list-item">Beta</li></ul>
```

Forbidden fx: `<div data-slot="animated-list" style="…padding:0.75rem 1rem;position:relative;overflow:hidden;"><span>Demo</span></div>`. Asserts: no `<div`, no `<span`, no `style=`.

Chrome: column flex `gap: 0.5rem` `list-style: none`; item animation `0.35s var(--cronus-ease)`.

## toast

React docs use Toaster / `data-slot="toaster"` (already dedicated as **sonner**). Kernel family `toast` is one visible toast: `<div data-slot="toast" role="status">` with label text. `aria-live="polite"` kept for a11y. Not catalog `fx()` SURF title box. Not interact inline SURF. Distinct from sonner's `<div data-slot="sonner"><div data-slot="toaster">…` wrapper.

Dedicated DOM (static, zero JS):

```html
<div data-slot="toast" role="status" aria-live="polite">Saved</div>
```

Forbidden fx: `<div data-slot="toast" style="…padding:0.75rem 1rem;position:relative;overflow:hidden;"><span>Demo</span></div>`. Forbidden sonner wrap: `data-slot="sonner"` / `data-slot="toaster"`. Asserts: one `data-slot="toast"`, no `style=`, `dedicated_fn_name("toast") == Some("cronus_ui_toast::render")`, `renderer_kind("meteors") == Stub("fx")`.

Chrome: existing `[data-slot="toast"]` token surface (`var(--cronus-surface-floating)`, `var(--cronus-border)`, `padding: 0.75rem 1rem`). Not `[data-slot="sonner"]` `position: fixed`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_spotlight_card
  running 6 tests
  test cronus_ui_spotlight_card::tests::extra_text_does_not_add_nodes ... ok
  test cronus_ui_spotlight_card::tests::label_is_escaped ... ok
  test cronus_ui_spotlight_card::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_spotlight_card::tests::root_is_div_wrapping_label_not_display_section ... ok
  test cronus_ui_spotlight_card::tests::skips_display_surf_and_interact_card ... ok
  test cronus_ui_spotlight_card::tests::chrome_spotlight_via_css ... ok
  test result: ok. 6 passed

cargo test --offline cronus_ui_animated_list
  running 7 tests
  test cronus_ui_animated_list::tests::label_only_still_emits_one_item ... ok
  test cronus_ui_animated_list::tests::label_is_escaped ... ok
  test cronus_ui_animated_list::tests::extra_text_items_become_rows ... ok
  test cronus_ui_animated_list::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_animated_list::tests::root_is_ul_of_items_not_fx_title_box ... ok
  test cronus_ui_animated_list::tests::skips_fx_surf_title_box ... ok
  test cronus_ui_animated_list::tests::chrome_is_token_only ... ok
  test result: ok. 7 passed

cargo test --offline cronus_ui_toast
  running 6 tests
  test cronus_ui_toast::tests::extra_text_does_not_add_toasts ... ok
  test cronus_ui_toast::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_toast::tests::label_is_escaped ... ok
  test cronus_ui_toast::tests::root_is_one_toast_not_fx_or_sonner_toaster ... ok
  test cronus_ui_toast::tests::skips_fx_surf_interact_and_sonner_wrapper ... ok
  test cronus_ui_toast::tests::chrome_is_token_only ... ok
  test result: ok. 6 passed

cargo test --offline stub_renderer_gate
  running 7 tests
  test cli::stub_renderer_gate::tests::meteors_is_stub_fx ... ok
  test cli::stub_renderer_gate::tests::sankey_chart_is_stub ... ok
  test cli::stub_renderer_gate::tests::button_is_dedicated ... ok
  test cli::stub_renderer_gate::tests::every_family_is_classified ... ok
  test cli::stub_renderer_gate::tests::no_new_stub_families ... ok
  test cli::stub_renderer_gate::tests::ported_families_are_dedicated_and_skip_interact ... ok
  test cli::stub_renderer_gate::tests::dedicated_module_no_sidecar_assets ... ok
  test result: ok. 7 passed
```

`meteors_is_stub_fx` asserts `renderer_kind("meteors") == Stub("fx")` and the catalog fx fingerprint. `sankey_chart_is_stub` remains `Stub("chart")`. Toast `dedicated_fn_name` is `cronus_ui_toast::render`, not `fx`.

Wave total: **26 passed** (6 + 7 + 6 + 7).
