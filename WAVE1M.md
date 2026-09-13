# Cronus Audit Wave 1m — terminal / video-player / text-effect

Worktree: `/Users/pedrogbraz/projects/cooud/.wt/kernel-1m-b`  
Branch: `feat/wave1m-term-video-text`  
Base: `0054828`  
Date: 2026-09-13

Done: 3 dedicated modules, catalog `display()` / `fx()` skipped, interact skipped, tests green. Not pushed.  
Not touched: meteors (harness `Stub("fx")`), sankey-chart, tilt-card, toast.

Helpers: `src/cronus_ui_kit.rs` (`texts`, `label_of`, `esc`, test `stub`). Pattern: `src/cronus_ui_text_shimmer.rs` / `src/cronus_ui_timeline.rs` (dedicated slots + CSS chrome). Not catalog `fx()` title SURF box (`padding:0.75rem 1rem;position:relative;overflow:hidden` + inner `<span>`). Not catalog `display()` SURF `<section>` (`padding:1rem;display:flex;flex-direction:column;gap:0.5rem`). Not interact generic HTML.

Zero JS. Zero Voodoo. No HTML in `.cronus`. Token CSS only `var(--cronus-*)`.

## Commits (one per family)

| SHA | Message |
|-----|---------|
| `f810483` | feat(ui): dedicated Terminal renderer |
| `6d4f9e6` | feat(ui): dedicated VideoPlayer renderer |
| `02a9faa` | feat(ui): dedicated TextEffect renderer |

## Dispatch

`PORTED_FAMILIES` **appended** (does not reorder existing Wave 1a–1l list):

```rust
pub const PORTED_FAMILIES: &[&str] = &[
    // … button … tree-view …
    "terminal",
    "video-player",
    "text-effect",
];
```

`dedicated_render` / `dedicated_fn_name`:

| Family | Module | `dedicated_fn_name` |
|--------|--------|---------------------|
| terminal | `src/cronus_ui_terminal.rs` | `cronus_ui_terminal::render` |
| video-player | `src/cronus_ui_video_player.rs` | `cronus_ui_video_player::render` |
| text-effect | `src/cronus_ui_text_effect.rs` | `cronus_ui_text_effect::render` |

`src/main.rs` mods: `cronus_ui_terminal`, `cronus_ui_video_player`, `cronus_ui_text_effect`.  
Generator `scripts/gen_cronus_ui_widgets.py` stays in sync so a regen cannot drop the ports.  
Stub gate file list includes the three modules. `COMPONENT_CHROME` has token CSS for all three.

Ported families return from `dedicated_render` **before** `cronus_ui_interact::render` and **before** catalog `fx()` / `display()`. Catalog stub / interact arms remain for unported families and were not rewritten.

124 `PORTED_FAMILIES`. 175 `FAMILIES`. Remaining chart stub: sankey-chart. meteors still `Stub("fx")`. tilt-card / toast untouched.

Fingerprint: dedicated HTML must not contain catalog `display()` SURF `<section>` without `terminal-screen` (terminal), interact `<video data-slot="video-player">` / missing video element (video-player), catalog `fx()` title SURF box (text-effect).

## terminal

React: animated session inside `<div data-slot="terminal">` with `terminal-screen` and `terminal-prompt` (`packages/ui/src/components/terminal.tsx`). Kernel is **static** (finished transcript). Each `texts` item is a prompt line. Optional CSS blink on the last line (`::after` + `@keyframes cui-terminal-caret`). Zero JS. Not interact `codey()` `<pre data-slot="terminal" style=SURF>`, not catalog `display()` SURF `<section>` without `terminal-screen`.

Dedicated DOM (static, zero JS):

```html
<div data-slot="terminal"><div data-slot="terminal-screen"><div data-slot="terminal-line"><span data-slot="terminal-prompt">$</span><span>npm install</span></div></div></div>
```

Forbidden interact: `<pre data-slot="terminal" style="…"><code>…</code></pre>`. Forbidden display: `<section data-slot="terminal" style="…padding:1rem;display:flex;flex-direction:column;gap:0.5rem;">`. Asserts: no `<pre`, no `<section`, no `style=`.

Chrome: inset surface, `var(--cronus-font-mono)`, prompt `var(--cronus-fg-tertiary)`, optional caret blink (honours `prefers-reduced-motion`).

## video-player

React: `<div data-slot="video-player">` wrapping `<video data-slot="video-player-video">` plus `video-player-controls` / play (`packages/ui/src/components/video-player.tsx`). Kernel is **static**. No `src` required. Play button is markup only (no `onclick`). Zero JS. Not interact `video()` (`<video data-slot="video-player" controls style=SURF>`), not catalog `display()` SURF `<section>` without a video element.

Dedicated DOM (static, zero JS):

```html
<div data-slot="video-player"><video data-slot="video-player-video" playsinline preload="metadata"></video><div data-slot="video-player-controls"><button type="button" data-slot="video-player-play" aria-label="Play">Play</button></div></div>
```

Forbidden interact: `<video data-slot="video-player" controls style="…">`. Forbidden display: `<section data-slot="video-player">` with no `<video>`. Asserts: wrapping `div`, inner `video-player-video`, `video-player-controls` + play, no `src=`, no `style=`.

Chrome: `aspect-ratio: 16 / 9`, inset surface, absolute video, bottom controls bar `var(--cronus-surface-base)`.

## text-effect

React: staggered enter on `<p data-slot="text-effect">` / `as` (`packages/ui/src/components/text-effect.tsx`). Kernel is a **span** with the label (same idle shape as `text-shimmer` / `typing-text`). CSS blur+fade (`@keyframes cui-text-effect`). Zero JS. Not catalog `fx()` title SURF box.

Dedicated DOM (static, zero JS):

```html
<span data-slot="text-effect">Ship faster</span>
```

Forbidden fx: `<div data-slot="text-effect" style="…padding:0.75rem 1rem;position:relative;overflow:hidden;"><span>Demo</span></div>`. Asserts: no `<div`, no `style=`, no SURF.

Chrome: `display: inline-block` `animation: cui-text-effect 400ms var(--ease-out-quart) both`; reduced-motion disables animation.

## Tests

`cargo test` takes one filter; ran each required name:

```
cargo test --offline cronus_ui_terminal
  running 7 tests
  test cronus_ui_terminal::tests::chrome_is_token_only ... ok
  test cronus_ui_terminal::tests::extra_text_items_become_prompt_lines ... ok
  test cronus_ui_terminal::tests::label_is_escaped ... ok
  test cronus_ui_terminal::tests::label_only_still_emits_one_line ... ok
  test cronus_ui_terminal::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_terminal::tests::root_is_div_with_screen_and_prompt_lines ... ok
  test cronus_ui_terminal::tests::skips_interact_pre_and_display_surf ... ok
  test result: ok. 7 passed

cargo test --offline cronus_ui_video_player
  running 5 tests
  test cronus_ui_video_player::tests::chrome_is_token_only ... ok
  test cronus_ui_video_player::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_video_player::tests::root_is_div_with_video_and_play_controls ... ok
  test cronus_ui_video_player::tests::skips_interact_video_and_display_surf ... ok
  test cronus_ui_video_player::tests::video_has_no_src ... ok
  test result: ok. 5 passed

cargo test --offline cronus_ui_text_effect
  running 5 tests
  test cronus_ui_text_effect::tests::chrome_text_effect_via_css ... ok
  test cronus_ui_text_effect::tests::label_is_escaped ... ok
  test cronus_ui_text_effect::tests::no_voodoo_even_when_runtime_on ... ok
  test cronus_ui_text_effect::tests::root_is_span_with_label_not_fx_title_box ... ok
  test cronus_ui_text_effect::tests::skips_fx_surf_title_box ... ok
  test result: ok. 5 passed

cargo test --offline stub_renderer_gate
  running 7 tests
  test cli::stub_renderer_gate::tests::button_is_dedicated ... ok
  test cli::stub_renderer_gate::tests::dedicated_module_no_sidecar_assets ... ok
  test cli::stub_renderer_gate::tests::every_family_is_classified ... ok
  test cli::stub_renderer_gate::tests::meteors_is_stub_fx ... ok
  test cli::stub_renderer_gate::tests::no_new_stub_families ... ok
  test cli::stub_renderer_gate::tests::ported_families_are_dedicated_and_skip_interact ... ok
  test cli::stub_renderer_gate::tests::sankey_chart_is_stub ... ok
  test result: ok. 7 passed
```

`meteors_is_stub_fx` asserts `renderer_kind("meteors") == Stub("fx")` and the catalog fx fingerprint. `sankey_chart_is_stub` remains `Stub("chart")`.

Wave total: **24 passed** (7 + 5 + 5 + 7).
