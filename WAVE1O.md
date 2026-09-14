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
`line-height: 1.5`; stage `position: absolute; inset: 0`; faces absolute
`inset: 0`, border, `shadow-sm`, `backface-visibility: hidden`, front
`surface-raised`, back `surface-elevated`. Reduced motion: no rotation, visibility swap.

| slot | React | Cronus |
|---|---|---|
| slot | React (corrected stage) | Cronus |
|---|---|---|
| flip-card | 24,24 288×256 r22 role=group | 24,24 288×256 r22 role=group |
| stage div | 24,24 288×256 transform none | 24,24 288×256 transform none |
| flip-card-front | 24,24 288×256 bg 21,21,23 border 1px, 16/24 "Front" | same |
| flip-card-back | 24,24 288×256 bg 38,38,41 matrix3d(-1,…) "Back" | same |

Screenshot pixel diff: 0; "Front" visible on both sides.

The React stage used to be `size-full` inside a min-height-only card, so it resolved
to 0px and both faces collapsed to their 2px borders. React now uses `absolute inset-0`
(cooud-ui `flip-card.tsx`), and Cronus follows it. Divergence: no `inert` /
`data-active` toggling (needs JS), so the back face text stays in the accessibility tree.

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

### aspect-ratio
Wave 1t geometry parity. DOM unchanged (`<div data-slot="aspect-ratio">label</div>`). CSS width
`18rem` mirrors the fixture `className="w-72"` (React component default is `w-full`; `.cronus` has
no className channel), `aspect-ratio: 16 / 9`.

| slot | React | Cronus |
|---|---|---|
| aspect-ratio | 24,24 288×162 | identical (was 432×243) |

0 mismatches.

### frame
Wave 1t geometry parity. DOM = React `variant="browser"`: `div[data-slot=frame][data-variant=browser]`
> `frame-chrome` (3 dots + `div[data-slot=frame-address-bar]` with `url` from prop or item config,
empty when absent, like React's `{url}`) > `frame-content`. CSS: frame `width: 18rem` (fixture
`w-72`); address bar `margin 0 auto; max-width 100%; ellipsis; radius-md; surface-raised;
padding 0.25rem 0.75rem; 0.75rem/1rem; fg-tertiary`.

| slot | React | Cronus (source with `url:"cronus.dev"`) |
|---|---|---|
| frame | 24,24 288×67, bg rgba(21,21,23), r 18px | identical |
| frame-chrome | 25,25 286×41, bg rgba(14,14,16), border-bottom 1px | identical |
| frame-address-bar | 154.78,33 86.44×24, 12px/16px, rgb(133,133,142), r 10px | identical |
| frame-content | 25,66 286×24 | identical |

0 mismatches once the fixture source carries `url`. Current `cronus-fixtures/app.cronus` has no
`url` line (the emitter drops the prop), leaving 8 mismatches (empty bar: chrome 29px, no
"cronus.dev" text). Needed emitter change in `emit-cronus-fixture.ts`:
`if (typeof fixture.props.url === "string") lines.push(\`  url:"${cronusEscape(fixture.props.url)}"\`);`
