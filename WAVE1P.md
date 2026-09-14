# Wave 1p — 9 dedicated kernel ports

A: logo-carousel, dynamic-island, image-zoom
B: aurora-background, border-beam, confetti
C: composed-chart, heatmap-chart, chart
sankey-chart and meteors remain stubs.

## A — logo-carousel / dynamic-island / image-zoom

Dedicated CONTRACT renderers. Zero JS. Token CSS in `COMPONENT_CHROME`.
`PORTED_FAMILIES` length 151 (148 + 3).

### logo-carousel
React idle (page 0, `motionPreference="never"`): `<ul data-slot="logo-carousel" aria-label aria-live="off">` + up to 3 (React default `columns`)
`<li data-slot="logo-carousel-item" aria-label="{name}"><div><span aria-hidden="true"><span>{initials}</span></span></div></li>`.
Initials follow React `getInitials`: the first letter of up to two whitespace-separated words, uppercased ("Acme" → "A", "Globex Corp" → "GC"). They are computed on raw text and escaped once.
Skips interact `carousel("logo-carousel")` flex-overflow `<div style="...display:flex;gap:0.75rem;overflow:auto;">` SURF slides.
No `setInterval`. No inline `style=`.

Wave 1s geometry parity (measured end to end vs React fixture, aurora/dark, 1280x900). CSS mirrors the `ghost` variant at `lg` size:
- **ul:** `display:grid; grid-auto-flow:column; grid-auto-columns:minmax(0,1fr); gap:0.75rem; width:18rem`. Below 640px it switches to `grid-template-columns:repeat(2,minmax(0,1fr))`.
- **li:** `relative; box-sizing:border-box; flex centred; overflow:hidden; height:5rem` (`6rem` at ≥640px); `padding:0 0.75rem`; `border:1px solid transparent`; transparent background; `border-radius:var(--cronus-radius-xl)`; `color:color-mix(in oklab, var(--cronus-fg-secondary) 70%, transparent)`; hover `fg`.
- **Wrapper div:** `absolute inset-0 flex centred; padding:0 1rem`.
- **Mark span:** `inline-flex; max-width:100%; truncate; font-family:var(--cronus-font-display); font-weight:600; line-height:1; font-size:2.25rem` (`3.75rem` at ≥640px).

| slot | React (x,y,w,h) | Cronus | font / color |
|---|---|---|---|
| logo-carousel | 24,24,288,96 | 24,24,288,96 | — |
| item Acme | 24,24,138,96 | 24,24,138,96 | rgba(160,160,170,.70) both, r18 |
| item Globex | 174,24,138,96 | 174,24,138,96 | same |
| wrapper div | 25,25,136,94 | 25,25,136,94 | — |
| mark "A" | 72.5,42,40.5,60 | 72.5,42,40.5,60 | SF Pro Display 60px/60px 600 both |
| mark "G" | 221.5,42,43.5,60 | 221.5,42,43.5,60 | same |

Screenshot pixel diff: 1618 glyph pixels differ by at most 1/255 per channel. The cause is `color-mix` rounding of the 70% text color, inside the 2/255 tolerance, and it is not visible.

Divergences:
1. **Width.** `width:18rem` reproduces the audit fixture's `className="w-72"`, which `react-fixture-render` also applies by default. The React component itself defaults to `w-full`, but `.cronus` has no className channel.
2. **No paging.** The timer that pages through logos (`setInterval` + motion enter/exit) cannot exist with zero JS. Only the idle first page is emitted, and logos beyond the column count are not rendered, exactly as on React's page 0.
3. **Shadow.** React computes a Tailwind `shadow-none` box-shadow composite made entirely of transparent layers. The kernel emits no box-shadow. Visually identical.

### dynamic-island
React idle: `<div data-slot="dynamic-island">` + `dynamic-island-shell` + `role="tablist"` of `button data-slot="dynamic-island-trigger"`.
Static first view (first text). First trigger `aria-selected="true"`.
Skips catalog `fx()` SURF title box. No motion/framer.

### image-zoom
React idle: `<button type="button" data-slot="image-zoom" data-state="idle">` wrapping `image-zoom-content` + optional `image-zoom-indicator`.
URL/src texts → escaped `<img src alt>`. Else label text in content.
No JS zoom / onclick. CSS hover scale; `@media (prefers-reduced-motion: reduce) { transform: none }`.
Skips catalog `fx()` SURF title box.

### leftovers
`renderer_kind("meteors")` = `Stub("fx")`
`renderer_kind("sankey-chart")` = `Stub("chart")`

### cargo test
```
export CARGO_TARGET_DIR=/Users/pedrogbraz/projects/cooud/cronus-kernel/target
cd /Users/pedrogbraz/projects/cooud/.wt/kernel-1p-a
cargo test --offline -- --test-threads=1 cronus_ui_logo_carousel cronus_ui_dynamic_island cronus_ui_image_zoom
```
22 passed (7 logo-carousel + 7 dynamic-island + 8 image-zoom).

## B — aurora-background / border-beam / confetti

CSS-only fx. No canvas/script. leftover stub is meteors.

## C — composed-chart / heatmap-chart / chart

SVG dedicated charts. leftover stub is sankey-chart.

## Wave 1t — geometry parity, effects A (2026-09-14)

Measured with the `e2e/audit/geometry.spec.ts` rules (FREEZE_CSS, settle loop, slot+occurrence pairing, rect ≤1px, colour ≤2/255) against the `default` fixture, aurora/dark, canvas 480px: **0 mismatches**.

### aurora-background
DOM: `<div data-slot="aurora-background"><div aria-hidden="true"><div></div><div></div><div></div></div><div>{label}</div></div>`.
No `aurora-blob` slots (React blobs have none). CSS: root `width: 18rem; min-height: 8rem`
(fixture `w-72 min-h-32`); layer `z-index: -10` like React's `-z-10` — the root is not a
stacking context, so the blobs sit behind the ground and are invisible in both panes.

### border-beam
DOM: `<div data-slot="border-beam"><div aria-hidden="true"></div><div>{label}</div></div>`.
CSS: root `width: 18rem; padding: 1.5rem` (fixture `w-72 p-6`), radius +8px (`rounded-2xl`);
layer 1px transparent border + two-layer `mask-composite: intersect` (beam only on the
border band, as React); beam gradient `fg → transparent`.

### confetti
DOM: `<div data-slot="confetti"><div>{label}</div></div>`. React's idle canvas is empty
(`fireOnMount` false) and the zero-JS kernel emits no canvas; the six CSS `confetti-piece`
spans were painting pieces React never shows and are removed. CSS `width: 18rem;
min-height: 8rem`.

| slot | React | Cronus |
|---|---|---|
| aurora-background | 24,24 288×128 "Aurora" | same (was 432×24 + 3 blob slots) |
| border-beam | 24,24 288×72 r22 "Beam" | same (was 432×24 r12 + 2 slots) |
| confetti | 24,24 288×128 "Celebrate" | same (was 432×24 + 6 piece slots) |
