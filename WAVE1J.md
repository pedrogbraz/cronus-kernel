# Wave 1j — 9 dedicated kernel ports

segmented-control, usage-meter, masonry, heatmap, comparison-slider, code-tabs, expandable-tabs, live-line-chart, sunburst-chart.
sankey-chart and meteors remain stubs.

### segmented-control
Wave 1s geometry parity (measured end to end vs React `SegmentedControl`, fixture Day/Week value Day, aurora/dark, 1280x900).

DOM (unchanged): `<div data-slot="segmented-control" data-size="md" role="radiogroup" aria-label>` +
`<button type="button" data-slot="segmented-control-item" data-state role="radio" aria-checked tabindex>`; the active item hosts `<div data-slot="segmented-control-thumb" aria-hidden="true">` before `<span>label</span>`.

CSS changes: root adds `position:relative; isolation:isolate; vertical-align:middle; line-height:1.5`.
Items get `line-height:1.25rem` (React `text-sm` → 14px/20px, previously `normal` = 17px, which made items 29px tall), plus `gap:0.375rem; white-space:nowrap; user-select:none`.
The label span becomes `inline-flex; align-items:center; gap:0.375rem`.

| slot | React (x,y,w,h) | Cronus | font | colors |
|---|---|---|---|---|
| segmented-control | 24,24,124,42 | 24,24,124,42 | — | bg rgb(30,30,33), r14 both |
| item Day (active) | 29,29,49.5,32 | 29,29,49.5,32 | 14px/20px 500 both | fg rgb(250,250,249) both |
| thumb | 29,29,49.5,32 | 29,29,49.5,32 | — | rgb(22,22,25), r10, shadow both |
| item Week | 82.5,29,60.5,32 | 82.5,29,60.5,32 | 14px/20px 500 both | rgb(159,159,169) both |
| span Day / Week | 41,35,25.5,20 / 94.5,35,36.5,20 | identical | 14px/20px 500 | — |

Screenshots of both canvases are byte-identical PNGs. No remaining delta.

### comparison-slider
Wave 1t geometry parity. DOM = React idle at 50%: `comparison-slider` > `comparison-after` >
`<div>After</div>`, `comparison-before` > `<div>Before</div>`, `<div aria-hidden>` divider,
`<div role="slider" aria-label aria-valuemin=0 aria-valuemax=100 aria-valuenow=50
aria-orientation=horizontal aria-disabled="true">` with lucide ChevronsLeftRight. Layer texts come
from `text` items (fixture `label "Comparison"` is the handle's name, never a layer).
CSS: layer slots are bare absolute boxes; `> div` carries flex centering, `0.875rem/1.25rem`,
surface-raised (after) / surface-overlay (before); before `clip-path: inset(0 50% 0 0)`
(`:dir(rtl)` → `inset(0 0 0 50%)`); divider 2px surface-base 90% + shadow-sm; handle 2.25rem round,
border, surface-raised, svg 1rem fg-secondary.

| slot | React | Cronus |
|---|---|---|
| comparison-slider | 24,24 432×243, bg rgba(14,14,16), r 18px, border 1px, text "After Before" | identical |
| comparison-after / -before | 25,25 430×241, 16px/24px, transparent | identical |

0 mismatches. Divergence: dragging and arrow keys need JS; the handle keeps React's element and
values but is `aria-disabled` and not focusable (a `<div>` has no `disabled`), not dimmed.
