# Wave 1i — 9 dedicated kernel ports

app-shell, table-of-contents, form, signature-pad, resizable, scheduler, alert-dialog, lightbox, notification-center.
meteors and sankey-chart remain stubs.

### form

Wave 1s geometry parity (2026-09-14). DOM now matches the React fixture exactly:
`<form data-slot="form"><div data-slot="form-item"><label data-slot="label" for="{id}">Email</label>`
`<input data-slot="input" id="{id}" name="Email" placeholder="ada@cronus.dev" /></div></form>`
(ids `widget_id(comp, "input")`, `-2`, `-3` … for extra fields; no `form-label` / `form-control`).
Label/input use the shared Label/Input chrome; form block adds `display: block; line-height: 1.5`,
item flex column gap .375rem (+ .75rem between items), input `line-height: 1.25rem`, placeholder
`fg-tertiary`. Gate (`stub_renderer_gate.rs`) accepts `<label data-slot="label"` as well as `form-label`;
the id avoids the `-control"` field-stub fingerprint.

| slot | React | Cronus |
|---|---|---|
| form / form-item | 24,24 432×60 | 24,24 432×60 |
| label | 24,24 432×14, 14px/14px 500 | identical |
| input | 24,44 432×40, 14px/20px, bg rgb(14,14,16), r 14px, placeholder rgb(133,133,142) | identical |

## Wave 1t — geometry parity (2026-09-14)

### scheduler
DOM (React `scheduler.tsx`, month view, Sunday first): `<div data-slot="scheduler"><div>`
`<h2 data-slot="scheduler-title">June 2026</h2><div>` Prev (`data-size="icon-sm"`), Today
(`data-size="sm"`), Next outline `data-slot="button"`s `</div></div>`
`<table data-slot="scheduler-grid" aria-label="Event calendar"><thead data-slot="scheduler-weekdays">…`
then one `<tr>` per week of `<td aria-label="Sunday, May 31, 2026" [data-outside]>`
`<span>31</span><span>` chips `<button data-slot="scheduler-event" title="…">` `</span></td>`.
Month from the title (`label "June 2026"`) or `month:"YYYY-MM"`; events are content texts on
`date:"YYYY-MM-DD"`, undated ones on the 15th (first) / 20th (rest) like `scheduler-fixture.tsx`;
`today:"YYYY-MM-DD"` sets `aria-current="date"`. Max 3 chips + `scheduler-event-overflow`.
Zero-JS control contract (Wave 1t): controls that need a runtime are React's own native element marked `disabled`, in React's idle look (dimmed only where React dims). Prev/Today/Next (month navigation) and event chips (`onEventClick`) are `disabled`.
CSS: root radius-xl border `surface-base`; header flex space-between gap 0.5rem p 0.75rem bottom
border; header buttons pinned to React line boxes (14/20, sm 12/16, svg 14px); title 14/20 w600 —
header-scoped override (0,3,0) beats `html[data-cronus-theme] h2` (weight 400, -0.025em) and adds
the React page's inherited `tabular-nums`; th 12/16 `fg-tertiary` px 0.5rem py 0.375rem bottom border;
td h 6rem p 0.375rem right+bottom borders; day number 24px circle 12/16 w500; chips 12/16 w500
radius 0.25rem, primary 15% tint + `primary-text`.

| slot | React | Cronus |
|---|---|---|
| scheduler | 24,24 432×568 r18 | same |
| scheduler-title | 37,43 72.75×20 w600 | same |
| button ×3 | 310.42,37 32×32 · 346.42,37 60.58×32 · 411,37 32×32 r14 | same |
| scheduler-grid / weekdays | 25,82 430×509 / 430×28.5 | same |
| scheduler-event ×2 | 92.92,339 48.42×20 · 400.03,339 r4 | same |

Result: 0 mismatches. Screenshot delta: React's harness passes `today` (15th → primary dot);
the kernel only highlights with explicit `today:` (unmeasured, non-slot span).
