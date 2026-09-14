# Wave 1k — 9 dedicated kernel ports

choropleth-chart, profit-loss-chart, scroll-progress, rich-text-editor, confirmation-dialog, invite-dialog, shimmer, reveal, text-shimmer.
sankey-chart and meteors remain stubs.

## Wave 1s — geometry parity (2026-09-14)

### text-shimmer
DOM: `<p data-slot="text-shimmer">Loading</p>`.
CSS: `display: inline-block; margin: 0; position: relative; line-height: 1.5;
color: transparent`; background layers `linear-gradient(90deg, transparent 40%,
var(--cronus-surface-base), transparent 60%)` + solid `fg-tertiary`,
`background-size: 250% 100%, auto`, clipped to text, `cui-text-shimmer` 2s linear
infinite from `100%` to `0%` (same as framer's `backgroundPosition`).

| slot | React | Cronus |
|---|---|---|
| text-shimmer | 24,24 57.5×24, 16/24 | 24,24 57.5×24, 16/24 |

Paused both at `background-position: 50% center`: screenshot diff 30 px, max 5/255,
inside the glyphs. Reason: React's highlight half-width is an inline
`--spread: {chars × 2}px` (14px for "Loading"); Cronus cannot emit inline style, so
it uses 10% of the 250% background (≈14.4px here), which scales with text width
instead of character count.

## Wave 1t — geometry parity (2026-09-14)

### scroll-progress
DOM (bar): `<div data-slot="scroll-progress" data-variant="bar" role="progressbar" aria-valuenow="40" …>`
`<div data-slot="scroll-progress-fill" data-value="40"></div></div>`. No inline style: the fill
width is `calc(attr(data-value type(<number>), 0) * 1%)` (typed `attr()`, Chromium 133+; engines
without typed `attr()` show an empty track). Root `height: 0.25rem; width: 18rem` (fixture wrapper
`w-72`; React bar is `w-full` inside it), `surface-inset`; fill `primary`.

| slot | React | Cronus |
|---|---|---|
| scroll-progress | 24,24 288×4 | 24,24 288×4 |
| scroll-progress-fill | 24,24 115.5×4 | 24,24 115.2×4 |

Divergence: React's fixture scrolls a real `h-32` box to 40% (77px/192px = 40.10%); the kernel
renders the declared value 40%. Δ0.3px, inside tolerance. The fixture's scroll box has no slot.

### shimmer
DOM: `<div data-slot="shimmer" aria-hidden="true"><div></div></div>` (inner div = React's
`absolute inset-0 animate-shimmer` sweep). CSS root `block relative overflow-hidden`,
`height: 2rem; width: 12rem` (fixture default `h-8 w-48`), `radius-md` (10px),
`surface-overlay`; sweep `absolute inset-0`, fg/10 gradient, `cui-shimmer` 2s.

| slot | React | Cronus |
|---|---|---|
| shimmer | 24,24 192×32 r10 | 24,24 192×32 r10 |
