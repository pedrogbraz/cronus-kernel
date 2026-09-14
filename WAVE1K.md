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
