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

### expandable-tabs

Wave 1t geometry parity (2026-09-14, fixture Home/Search, aurora/dark).
DOM: `<div data-slot="expandable-tabs" role="tablist" aria-label>` + per tab
`<button type="button" role="tab" data-slot="expandable-tabs-item" aria-selected disabled><span aria-hidden="true">{circle svg}</span><span>{label}</span></button>`.
The kernel has no icon source, so every tab shows the same circle glyph the React fixture passes.
Unselected labels are sr-only (React collapsed state). Switching tabs needs JS: native `disabled`, not dimmed.
`label` ("Sections") is aria only (it used to leak as a 1st tab).
CSS: item `line-height: 1.25rem`; icon span grid 1rem; label span nowrap; `[aria-selected="false"] > span:last-child` sr-only.

| slot | React | Cronus |
|---|---|---|
| expandable-tabs | 24,24 133×42, r 21px | identical |
| item Home (selected) | 29,29 83×32, r 16px, 14px/20px | identical |
| item Search | 116,31 36×28, r 14px | identical |
