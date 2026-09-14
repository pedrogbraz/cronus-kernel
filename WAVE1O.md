# Wave 1o — 9 dedicated kernel ports

aspect-ratio, frame, flip-card, countdown, animated-button, card-stack, gauge-chart, funnel-chart, candlestick-chart.
sankey-chart and meteors remain stubs.

## Wave 1s — geometry parity (2026-09-14)

Measured end to end (Playwright 1280×900, DPR 1, dark, aurora; React
`localhost:4747/audit/{family}` vs kernel `/audit/{family}/default`). Rects are
relative to the 480px audit canvas.

### flip-card
DOM: `<div data-slot="flip-card" role="group" tabindex="0" aria-label="Plan"><div>`
`<div data-slot="flip-card-front">Front</div><div data-slot="flip-card-back">Back</div></div></div>`.
`role="group"` only when `aria-label` is set; `tabindex="0"` always (React hover trigger).
The unslotted inner `<div>` is React's preserve-3d stage; `:hover` / `:focus-within`
rotate it `rotateY(180deg)` (CSS only, so focus is a working control). Back face
pre-rotated 180deg.
CSS: root `width: 18rem` (fixture `w-72`), `min-height: 16rem`, radius
`calc(var(--cronus-radius) + 8px)` (rounded-2xl, 22px), `perspective: 1600px`,
`line-height: 1.5`; stage `position: relative; width/height: 100%`; faces absolute
`inset: 0`, border, `shadow-sm`, `backface-visibility: hidden`, front
`surface-raised`, back `surface-elevated`. Reduced motion: no rotation, visibility swap.

| slot | React | Cronus |
|---|---|---|
| flip-card | 24,24 288×256 r22 | 24,24 288×256 r22 |
| stage div | 24,24 288×0 | 24,24 288×0 |
| flip-card-front | 24,24 288×2 bg 21,21,23 | 24,24 288×2 bg 21,21,23 |
| flip-card-back | 24,24 288×2 bg 38,38,41 matrix3d(-1,…) | same |

Screenshot pixel diff: 0. Note: in React the stage is `size-full` inside a
min-height-only parent, so it resolves to 0px and both faces collapse to their 2px
borders (text clipped). Cronus mirrors that geometry; fixing it belongs in the React
component (e.g. `absolute inset-0` stage). Divergence: no `inert` / `data-active`
toggling (needs JS), so the back face text stays in the accessibility tree.

### card-stack
DOM: `<section data-slot="card-stack" aria-label="Stack"><span>Card stack</span>`
`<div data-slot="card-stack-item">One</div><div data-slot="card-stack-item" aria-hidden="true">Two</div></section>`.
The first `<span>` is React's `sr-only` region label (hidden via `[data-slot="card-stack"] > span`).
CSS: section `height: 14rem; width: 18rem; max-width: 24rem; line-height: 1.5`; items
absolute `inset: 0`, radius 22px, border, `surface-raised`, padding 1.25rem, `shadow-sm`,
`transform-origin: top center`; `nth-of-type(1)` `translate: 0px 0px`; `(2)`
`translate: 10px 10px; transform: scale(0.96) rotate(-1.5deg)`; `(3)`
`translate: 20px 20px; scale(0.92) rotate(-3deg)`; z-index by `nth-last-of-type` (last = 1),
same as React `visible.length - index`.

| slot | React | Cronus |
|---|---|---|
| card-stack | 24,24 288×224 | 24,24 288×224 |
| sr-only span | 23,23 1×1 "Card stack" | 23,23 1×1 "Card stack" |
| item 1 | 24,24 288×224 r22 pad 20 | 24,24 288×224 r22 pad 20 |
| item 2 | 40,30.5 282×222 matrix(0.959671,-0.0251299,…) | identical |

Screenshot pixel diff: 34 px, max 13/255, all on the rotated back card's antialiased
edge (computed transforms identical; framer composites the layer differently).
Divergence: React front card is `role="button"` `tabindex=0` "Show next card" and
cycles on click/keys; that needs JS, so Cronus emits no control. React's accessible
name is "Card stack" (`aria-labelledby` wins over `aria-label`); Cronus keeps
`aria-label`.
