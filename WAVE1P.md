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

## Wave 1t — geometry parity: charts (2026-09-14)

Families: chart, area-chart, bar-chart, line-chart, live-line-chart, pie-chart, radar-chart,
ring-chart, scatter-chart, composed-chart, candlestick-chart, funnel-chart, gauge-chart,
heatmap-chart, profit-loss-chart, usage-meter.

### shared recharts engine (`src/cronus_ui_chart.rs`)
React wraps every chart in `ChartContainer` (`div[data-slot=chart]`, `flex aspect-video
justify-center text-xs h-full w-full`) whose `ResponsiveContainer` measures 432×256 in the audit
canvas. The kernel emits the settled SVG for that box, zero JS:
`<div data-slot="{family}" role="img" aria-label><div data-slot="chart"><svg viewBox="0 0 432 256" aria-hidden="true">…</svg></div></div>`
(the `chart` family is the container itself). Ported recharts 3 defaults: cartesian plot
x 8..424 / y 8..226 (x-axis band 30), `getNiceTickValues` (tickCount 5) and
`getTickValuesFixedDomain`, point vs band scale (band when a Bar exists, barCategoryGap 10%),
d3 `curveMonotoneX`, `Rectangle` radius-4 paths, `getSectorPath` / `getSectorWithCorner`,
polar box 422×246 (centre 216,128), tick labels `y=244 dy=0.71em` with `preserveEnd`
(last label pulled inside 432px, labels dropped right-to-left when clipped or closer than
`minTickGap`). Label widths use a 12px SF Pro advance table (only decides hiding).
Literal recharts defaults the React DOM also paints are kept as SVG presentation attributes:
tick fill `#666`, polar grid `#ccc`, radar tick `#808080`, gauge track `#eee`, funnel stroke
`#fff`. Colours otherwise `var(--cronus-chart-N)` / `--cronus-success|error|border`.
Category data: the emitter only writes the `text` lines, so values follow the React fixture
series (cycle 4, 8, 6, 10, 7; P/L -4, 8, 2; composed desktop (n-i)×4 / mobile (i+1)×2; funnel
(n-i)×4; OHLC fixture candles). Numeric items override where React takes numbers (`chart`,
`heatmap-chart`, area/bar/line values).
CSS: `[data-slot=chart] { display:flex; justify-content:center; width:100%; height:16rem;
aspect-ratio:16/9; font-size:.75rem; line-height:1rem }`, dashed grid lines
`color-mix(in oklch, var(--cronus-border) 50%, transparent)` (`stroke-border/50`); each family
root `display:block; width:100%; height:16rem` (`h-64 w-full`). Old `aspect-ratio: 2/1`,
12rem svg and path/polyline fill rules removed.
Not emitted (need JS): tooltips, `ChartCursor`, candlestick hover hit-rects, live-line ticking.
At widths other than 432px the SVG scales uniformly (`meet`) instead of re-laying out.

### chart
Bar + XAxis like `ChartFixture`: numeric items are the values, labels `1..n`, radius-4 bars
(chart-1) on a band scale, `<g><text><tspan>` ticks → text "1 2 3".

### area-chart
Grid rows, `<defs><linearGradient>` (stop 0.4 → 0) + area path fill-opacity 0.6, monotone
stroke chart-1 width 2, minTickGap 24 → text "Feb Mar" (Jan clipped at x=8).

### bar-chart
Grid rows, 3 bars 111 wide (`M 21.8667,121 A 4,4…`), all ticks → "Jan Feb Mar".

### line-chart
Grid rows, monotone line chart-1, minTickGap 24 → "Feb Mar".

### live-line-chart
First paint of React's state (`setInterval` appends random points — not emitted): values
4, 8, 6, every `text` line is a tick → "0 1 2".

### pie-chart
Two sectors, outer radius 98.4, start angle 0 counter-clockwise (`M 314.4,128 A 98.4,98.4,0, 0,0,
166.8,42.7831 L 216,128 Z`), chart-1/chart-2, no text.

### radar-chart
Five grid polygons (radius ticks 0..8) + spokes `#ccc`, series path chart-1 fill-opacity .25,
angle labels radius 94.1 (anchor start/end/middle, dy 0em/.355em/.71em) → "Speed Reliability Comfort".

### ring-chart
Donut sectors inner 64 / outer 98, paddingAngle 3 (354° shared), centre
`<text><tspan>12</tspan><tspan>Total</tspan></text>`; CSS `tspan:first-child` fg 1.5rem/500,
`tspan + tspan` fg-tertiary .75rem → text "12Total".

### scatter-chart
Numeric X (0..3, step .75) and Y (0..8) axes, YAxis width 60 → plot x 68..424, grid rows +
columns, circles r 4.5135, X labels (y 234) then Y labels (x 60, anchor end) →
"0 0.75 1.5 2.25 3 0 2 4 6 8".

### composed-chart
Band scale (Bar present): area desktop 12, 8, 4 (gradient .35) + bars mobile 2, 4, 6 (chart-2),
→ "Jan Feb Mar".

### candlestick-chart
Hidden YAxis domain [0, 11] → grid rows 0/3/6/9/11; `<g data-slot="candlestick-marks">` with
wick `line` + `rect` body (width 12, rx 1, success/error) per candle on a point scale;
minTickGap 24 → "Tue Wed". Marks bbox 428×138.73 at 26,71.64 like React.

### funnel-chart
Trapezoids in the 422×246 box (`M 5,5L 427,5L 321.5,128L 110.5,128L 5,5 Z`), chart-1/2, stroke
`#fff`, LabelList right (`x = 216 + (top+bottom)/4 + 5`, fill fg) → settled text "Visit Signup".
Divergence: React animates the funnel (isAnimationActive) and only mounts the LabelList after
~1.9s; the geometry gate measures after a few frames, so React reports text "". Kernel emits
the settled frame (see measured table).

### gauge-chart
`value` read from props or the label item's config (`value:72`; was falling back to 60).
RadialBar background `#eee` + value sector chart-1, radius 83..107, corner 8, 210° → -30°,
centre `<text x=50% y=50%>72</text>` (CSS fg 1.5rem/500).

### heatmap-chart
Now wraps the Heatmap like React: `heatmap-chart > heatmap > [role=img] + heatmap-legend`.
CSS: root `display:block; width:100%`; heatmap `inline-flex column gap .5rem` (6px baseline
strut offset like React); grid `grid-template-rows: repeat(7, minmax(0, 1fr))` (108px column);
legend `font-size .75rem; line-height 1rem`.

### usage-meter
Linear variant DOM: header `div` (flex baseline justify-between gap .5rem, .875rem/1.25rem) with
`usage-meter-label` + `usage-meter-value` (`<span>40 / 100</span><span>40%</span>`,
tabular-nums, fg-secondary / fg-tertiary), `usage-meter-track` (role=meter, h .5rem, full radius,
surface-overlay) > `usage-meter-fill` `data-tone` + `data-value`. Width is
`calc(attr(data-value type(<number>), 0) * 1%)` — the inline `style="width"` is gone.
Tone auto: >90% error, >75% warning. `value`/`max`/`unit`/`aria-label` from props or item config.

### measured (React vs Cronus, aurora/dark, default fixtures)

| family | before | after | note |
|---|---|---|---|
| chart | 432×216, 16px/24px, text "" | 432×256, 12px/16px, "1 2 3" | PASS |
| area-chart | root 216 tall, no `chart`, text "" | 432×256 + chart 432×256, "Feb Mar" | PASS |
| bar-chart | same | "Jan Feb Mar" | PASS |
| line-chart | same | "Feb Mar" | PASS |
| live-line-chart | same | "0 1 2" | PASS |
| pie-chart | no `chart` | chart 432×256 | PASS |
| radar-chart | no `chart`, text "" | "Speed Reliability Comfort" | PASS |
| ring-chart | no `chart`, text "" | "12Total" | PASS |
| scatter-chart | 216 tall, text "" | axes text equal | PASS |
| composed-chart | 216 tall, text "" | "Jan Feb Mar" | PASS |
| candlestick-chart | no `candlestick-marks` | marks 26,71.64 428×138.73 | PASS |
| funnel-chart | 216 tall, no `chart` | 432×256; text "" vs "Visit Signup" | 2 text rows (animation timing) |
| gauge-chart | text "60" | "72" | PASS |
| heatmap-chart | 138.66×98, no `heatmap`, legend lh 18 | 432×138, heatmap 138.66×132, lh 16 | PASS |
| profit-loss-chart | 216 tall, text "" | "Feb Mar" | PASS |
| usage-meter | label 432×21, no value/track | label 46.7×20, value 93.19×20, track 432×8 | PASS |

### cargo test
```
export CARGO_TARGET_DIR=/Users/pedrogbraz/projects/cooud/cronus-kernel/target
touch src/main.rs && cargo test --offline
```
1247 passed, 0 failed.
