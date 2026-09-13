# Cronus Audit Wave 1l — word-rotate / timeline / tree-view

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1l-c`  
Branch: `feat/wave1l-rotate-timeline-tree`  
Base: `1693093`  
Date: 2026-09-13

Done: 3 dedicated modules, catalog `fx()` / `display()` skipped, interact skipped, tests green. Not pushed.  
Not touched: meteors (harness `Stub("fx")`), sankey-chart, particles, morphing.

Helpers: `src/cronus_ui_kit.rs` (`texts`, `label_of`, `esc`, test `stub`). Pattern: `src/cronus_ui_text_shimmer.rs` / `src/cronus_ui_stepper.rs` (dedicated slots + CSS chrome). Not catalog `fx()` title SURF box (`padding:0.75rem 1rem;position:relative;overflow:hidden` + inner `<span>`). Not catalog `display()` SURF `<section>`. Not interact generic HTML.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `8ca48f8` | feat(ui): dedicated WordRotate renderer |
| `d186bb8` | feat(ui): dedicated Timeline renderer |
| `2d6b666` | feat(ui): dedicated TreeView renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1k list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … text-shimmer …
    "word-rotate",
    "timeline",
    "tree-view",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| word-rotate | `src/cronus_ui_word_rotate.rs` | `cronus_ui_word_rotate::render` |
| timeline | `src/cronus_ui_timeline.rs` | `cronus_ui_timeline::render` |
| tree-view | `src/cronus_ui_tree_view.rs` | `cronus_ui_tree_view::render` |

`src/main.rs` mods: `cronus_ui_word_rotate`, `cronus_ui_timeline`, `cronus_ui_tree_view`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render` and **before** catalog `fx()` / `display()`. Catalog stub / interact arms remain for unported families and were not rewritten.

115 `PORTED_FAMILIES`. 174 `FAMILIES`. Remaining chart stub: sankey-chart. meteors still `Stub("fx")`.

Fingerprint: dedicated HTML must not contain `padding:0.75rem 1rem;position:relative;overflow:hidden` (word-rotate), interact `<ol style>` / missing item+content (timeline), interact `<ul style>` / missing item+trigger (tree-view).

## word-rotate

React: cycles `words` inside `<span data-slot="word-rotate">` with motion swap (`packages/ui/src/components/word-rotate.tsx`). Kernel shows the **first** word from `texts` (static). CSS only (`inline-grid`, `height: 1.2em`, `overflow: hidden`). Zero JS.

Dedicated DOM (static, zero JS):

```html
<span data-slot="word-rotate">Ship</span>
```

Forbidden fx: `<div data-slot="word-rotate" style="…padding:0.75rem 1rem;position:relative;overflow:hidden;"><span>Demo</span></div>`. Asserts: no `<div`, no `style=`, no SURF, later words omitted.

Chrome: `display: inline-grid` `height: 1.2em` `overflow: hidden` `vertical-align: baseline`.

## timeline

React: `<ol data-slot="timeline">` of `timeline-item` + rail/dot + `timeline-content` (`packages/ui/src/components/timeline.tsx`). Kernel DOM is the assigned `<div data-slot="timeline">` plus each text as `timeline-item` / `timeline-content`. Rail is CSS (`::before` / `::after`). Not interact `timeline()` `<ol style=BASE>` bordered `<li>`, not catalog `display()` SURF `<section>`.

Dedicated DOM (static, zero JS):

```html
<div data-slot="timeline"><div data-slot="timeline-item"><div data-slot="timeline-content">Shipped</div></div></div>
```

Forbidden interact: `<ol data-slot="timeline" style="…"><li style="padding-left:1rem;border-left:2px solid…">`. Asserts: no `<ol`, no `<li`, no `style=`.

Chrome: column flex; item grid with `::before` disc `var(--cronus-fg-tertiary)` and connector `var(--cronus-border)`.

## tree-view

React: `<div data-slot="tree-view">` with nested `tree-view-item` / `tree-view-item-trigger` (`packages/ui/src/components/tree-view.tsx`). Kernel is a **flat list** from `texts`. Zero JS (no expand/select). Not interact `tree()` `<ul style=SURF>` `<li>`, not catalog `display()` SURF `<section>`.

Dedicated DOM (static, zero JS):

```html
<div data-slot="tree-view"><div data-slot="tree-view-item"><div data-slot="tree-view-item-trigger">src</div></div></div>
```

Forbidden interact: `<ul data-slot="tree-view" style="…"><li>…</li></ul>`. Asserts: no `<ul`, no `<li`, no `style=`.

Chrome: column flex; trigger row `height: 2rem` `var(--cronus-fg-secondary)` hover `var(--cronus-surface-overlay)`.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_word_rotate
  running 6 tests
  test cronus_ui_word_rotate::tests::chrome_word_rotate_via_css ... ok
  test cronus_ui_word_rotate::tests::shows_first_word_from_texts_statically ... ok
  test cronus_ui_word_rotate::tests::label_is_escaped ... ok
  test cronus_ui_word_rotate::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_word_rotate::tests::root_is_span_with_first_word_not_fx_title_box ... ok
  test cronus_ui_word_rotate::tests::skips_fx_surf_title_box ... ok
  test result: ok. 6 passed

cargo test --offline cronus_ui_timeline
  running 7 tests
  test cronus_ui_timeline::tests::chrome_is_token_only ... ok
  test cronus_ui_timeline::tests::label_is_escaped ... ok
  test cronus_ui_timeline::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_timeline::tests::label_only_still_emits_one_item ... ok
  test cronus_ui_timeline::tests::root_is_div_of_item_content_not_ol_or_section ... ok
  test cronus_ui_timeline::tests::extra_text_items_become_events ... ok
  test cronus_ui_timeline::tests::skips_interact_ol_and_display_surf ... ok
  test result: ok. 7 passed

cargo test --offline cronus_ui_tree_view
  running 7 tests
  test cronus_ui_tree_view::tests::label_only_still_emits_one_item ... ok
  test cronus_ui_tree_view::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_tree_view::tests::extra_text_items_become_nodes ... ok
  test cronus_ui_tree_view::tests::label_is_escaped ... ok
  test cronus_ui_tree_view::tests::chrome_is_token_only ... ok
  test cronus_ui_tree_view::tests::root_is_div_of_item_triggers_not_ul_or_section ... ok
  test cronus_ui_tree_view::tests::skips_interact_ul_and_display_surf ... ok
  test result: ok. 7 passed

cargo test --offline stub_renderer_gate
  running 7 tests
  test cli::stub_renderer_gate::tests::meteors_is_stub_fx ... ok
  test cli::stub_renderer_gate::tests::every_family_is_classified ... ok
  test cli::stub_renderer_gate::tests::sankey_chart_is_stub ... ok
  test cli::stub_renderer_gate::tests::button_is_dedicated ... ok
  test cli::stub_renderer_gate::tests::no_new_stub_families ... ok
  test cli::stub_renderer_gate::tests::ported_families_are_dedicated_and_skip_interact ... ok
  test cli::stub_renderer_gate::tests::dedicated_module_no_sidecar_assets ... ok
  test result: ok. 7 passed
```

`meteors_is_stub_fx` asserts `renderer_kind("meteors") == Stub("fx")` and the catalog fx fingerprint. `sankey_chart_is_stub` remains `Stub("chart")`.

Wave total: **27 passed** (6 + 7 + 7 + 7).
