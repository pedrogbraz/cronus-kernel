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
## Wave 1t — geometry parity (2026-09-14)

### heatmap
DOM unchanged. Label-only source (the emitted fixture has no `data`) renders the React harness
fallback series `[0,1,4,2,8,3,1,0,2,6,4,1,3,5]` dated from 2026-06-01 (`heatmap-fixture.tsx`),
instead of the old unrelated 5-value placeholder; numeric items still drive the days. `aria-label`
config wins over the label. CSS: `[data-slot="heatmap"] > [data-slot="heatmap-legend"]
{ line-height: 1rem; }` (text-xs line box), scoped so `heatmap-chart` is untouched.

| slot | React | Cronus |
|---|---|---|
| heatmap | 24,30 138.66×132 | same |
| heatmap-day ×14 | 12×12 r3, 2 columns at x 24 / 95.33 | same (levels 0,1,2,1,4,2,1,0,1,3,2,1,2,3) |
| heatmap-legend | 24,146 138.66×16 12/16 | same |
| heatmap-legend-swatch ×5 | 12×12 from x 53.83 | same |

Result: 0 mismatches.

### code-tabs
DOM (React idle): `<div data-slot="code-tabs" data-orientation="horizontal" aria-label="Install">`
`<div data-slot="code-tabs-header"><div role="tablist" aria-orientation="horizontal" data-slot="code-tabs-list">`
triggers (`role="tab"`, first `data-state="active"`, rest `disabled`) +
`<span aria-hidden="true" data-slot="code-tabs-indicator">` `</div><span data-slot="code-tabs-language">bash</span></div>`
`<div data-slot="code-tabs-panels"><button data-slot="copy-button" data-variant="ghost" aria-label="Copy bun snippet" disabled>`
`<div role="tabpanel" data-state="active" tabindex="0" data-slot="code-tabs-panel"><pre data-slot="code-tabs-pre"><code data-slot="code-tabs-code">bun install</code>`.
Tabs = content texts (the label is the `aria-label`); code/language from `code:` / `language:` item
config or a `code` item; with no code the harness placeholder `"{label} install"` + `bash`.
Zero-JS control contract (Wave 1t): controls that need a runtime are React's own native element marked `disabled`, in React's idle look (dimmed only where React dims). Inactive triggers (tab switching) and the copy button (clipboard) are `disabled`,
un-dimmed; hidden inactive panels are not emitted. The underline uses CSS anchor positioning
(`anchor-name` on the active trigger, `anchor()`/`anchor-size()` on the indicator, `anchor-scope` on the list).
CSS: root flex column radius-xl border `surface-raised`; header flex space-between gap 0.75rem pr 0.75rem
bottom border `surface-overlay`; list inline-flex `fg-secondary`; trigger 14/20 w500 p 0.625rem 0.875rem
`fg-tertiary` (active `fg`); indicator 2px `primary`; language mono 12/16; copy button absolute
top/right 0.5rem 32px radius-lg overlay 80% (overrides the global copy-button box and disabled dim);
pre p 1rem 3.5rem 1rem 1rem 14/22.75 mono.

| slot | React | Cronus |
|---|---|---|
| code-tabs | 24,24 432×97.75 r18 | same |
| code-tabs-header / list | 25,25 430×41 / 110.19×40 | same |
| code-tabs-trigger ×2 | 25,25 53.08×40 · 78.08,25 57.11×40 14/20 w500 | same |
| code-tabs-indicator | 25,63 53×2 | same |
| code-tabs-language | 414.09,37 28.91×16 mono | same |
| copy-button | 415,74 32×32 r14 | same |
| code-tabs-panel / pre / code | 25,66 430×54.75 / code 41,82 358×22.75 | same |

Result: 0 mismatches.
