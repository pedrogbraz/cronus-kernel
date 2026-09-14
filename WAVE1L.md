# Wave 1l — 9 dedicated kernel ports

particles, sparkles-text, noise, morphing-popover, bouncy-accordion, typing-text, word-rotate, timeline, tree-view.
sankey-chart and meteors remain stubs.

### timeline
Wave 1s geometry parity (measured end to end vs React `Timeline`, fixture Order placed / Order shipped, aurora/dark, 1280x900).

DOM (unchanged): `<ol role="list" data-slot="timeline" aria-label>` of `<li data-slot="timeline-item">` >
`<div data-slot="timeline-rail"><span data-slot="timeline-dot" data-tone="default"></span><div data-slot="timeline-connector" aria-hidden="true"></div></div>`
(no connector on the last item) + `<div data-slot="timeline-body"><div data-slot="timeline-content"><div data-slot="timeline-title">`.

CSS now mirrors the React utilities:
- **item:** `display:grid; grid-template-columns:auto minmax(0,1fr); column-gap:0.75rem; padding-bottom:1.5rem`, and `:last-child` has `padding-bottom:0`. Previously a flex row put the padding on the body.
- **rail:** `relative flex column centred`.
- **dot:** `0.625rem`, `border-radius:calc(infinity * 1px)`, `fg-tertiary`, no top margin. Previously it had `margin-top:0.375rem`.
- **connector:** `flex:1 1 0%; width:1px; margin-top:0.25rem; rounded-full; border`.
- **body:** `min-width:0; padding-top:0.125rem`.
- **content:** `flex column gap 0.25rem`, with no font-size.
- **title:** `font-size:0.875rem; font-weight:500; line-height:1; color:fg`. Previously `line-height:1.5`.

| slot | React (x,y,w,h) | Cronus | font / color |
|---|---|---|---|
| timeline | 24,24,432,56 | 24,24,432,56 | — |
| item #1 | 24,24,432,40 | 24,24,432,40 | — |
| rail #1 | 24,24,10,16 | 24,24,10,16 | — |
| dot #1 | 24,24,10,10 | 24,24,10,10 | rgb(133,133,142) both |
| connector #1 | 28.5,38,1,2 | 28.5,38,1,2 | rgba(245,255,255,.10) both |
| body #1 | 46,24,410,16 | 46,24,410,16 | — |
| title #1 | 46,26,410,14 | 46,26,410,14 | 14px/14px 500, rgb(250,250,249) both |
| item #2 | 24,64,432,16 | 24,64,432,16 | — |
| title #2 | 46,66,410,14 | 46,66,410,14 | same |

Unrounded rects and computed position/z-index/display are exactly equal on both sides.
Screenshot pixel diff shows 7 pixels (max 14/255), all on the lower anti-aliased edge of dot #2. The cause is the harness, not CSS:
- React's canvas sits at page y=184 and the kernel's at y=8.
- React's dot #2 therefore spans page rows 248–258, crossing Chromium's 256px raster tile boundary.
- The differing pixels are exactly page rows 253–256.
- Dot #1 does not cross a tile boundary and is pixel-identical.

## Wave 1t — geometry parity (2026-09-14)

### particles
DOM: `<div data-slot="particles"><div aria-hidden="true"></div><div>Field</div></div>`.
React's aria-hidden `<canvas>` (rAF specks) is an aria-hidden div with two layers of
`radial-gradient` dots (fg 35%) drifting via `cui-particles` 14s; hidden under reduced motion.
Root `relative overflow-hidden`, `width: 18rem; height: 8rem` (fixture `h-32 w-72`),
transparent background (previously `surface-overlay` + padding, inline-block 74×56).

| slot | React | Cronus |
|---|---|---|
| particles | 24,24 288×128 bg transparent | 24,24 288×128 bg transparent |

Divergence: speck placement/motion is a CSS dot lattice, not React's per-frame canvas.

### noise
DOM mirrors React: `<div data-slot="noise"><svg aria-hidden="true"><filter id="cui-{name}-grain">`
`<feTurbulence type="fractalNoise" baseFrequency="0.8" numOctaves="4" stitchTiles="stitch">`
`</feTurbulence></filter><rect width="100%" height="100%" filter="url(#…)"></rect></svg><div>Grain</div></div>`.
Filter id from the component name (React `useId`). CSS root `relative overflow-hidden`,
`18rem × 8rem`; svg `absolute inset-0 100%×100%`, `opacity: 0.08` (React inline); content relative.
The old `::after` block (missing `content` and a `background-image` property name) is gone.

| slot | React | Cronus |
|---|---|---|
| noise | 24,24 288×128 | 24,24 288×128 (was 432×24) |

### word-rotate
DOM mirrors React (`lockWidth`, `announce`): `<span data-slot="word-rotate"><span>Design</span>`
`<span aria-hidden="true" data-word-rotate-sizer="">Design</span><span aria-hidden="true" data-word-rotate-sizer="">System</span>`
`<span aria-hidden="true">Design</span></span>`. Words = `text` rows (the `label` row is not a word
unless it is the only row). CSS: root `relative inline-grid; height: 1.2em; overflow: hidden`;
first span sr-only; sizers `visibility: hidden; grid-area: 1 / 1; nowrap`; last span
`grid-area: 1 / 1; inline-block; nowrap; justify-self: start`. Static first word (no interval).

| slot | React | Cronus |
|---|---|---|
| word-rotate | 24,24 53.83×19.19 "Design Design" | 24,24 53.83×19.19 "Design Design" (was 50.31, "Design") |
