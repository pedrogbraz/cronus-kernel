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
