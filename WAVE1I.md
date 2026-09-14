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

### notification-center

Wave 1t geometry parity (2026-09-14). React `PopoverTrigger asChild` + ghost icon `Button`:
`button[data-slot="notification-trigger"][data-variant="ghost"][aria-label="Notifications, N unread"]
[popovertarget]` (Bell svg 1rem + `span[data-slot="notification-badge"][data-variant="primary"]` count,
"9+" above nine) + native `popover="auto"` `div[data-slot="notification-center"][role="dialog"]` >
header `<div><p>` title > `<div>` (max-height 20rem, p-1) > `div[data-slot="notification-list"] > ul > li >
button[data-slot="notification-row"]` (unread rows `data-unread` + `notification-unread-dot`); empty list →
`notification-empty` "You're all caught up". Label = title, texts = rows. Unread count: `unread` prop,
or the audit fixture's convention (first row unread). Trigger: 2.25rem square, fg-secondary, 14px/20px
500, radius-lg; badge: absolute -0.25rem/-0.25rem, h 1rem, min-w 1rem, p .125rem .25rem, 1px transparent
border, pill, primary, 0.625rem/1 500 tabular-nums. Row actions need JS: rows are rendered `disabled`.
Gate: `notification-center` without rows is accepted when `notification-empty` is present (React's empty
state). Limitation: the React fixture clicks the bell after mount and portals the panel out of the canvas;
the kernel panel stays closed until the bell is clicked.

| slot | React | Cronus (before → after) |
|---|---|---|
| notification-trigger | 24,24 36×36, 14px/20px 500, rgb(159,159,169), r 14px, text "1" | 115.48 wide "Notifications" 16px → identical |
| notification-badge | 47.44,20 16.56×16, 10px/10px 500, bg rgb(0,166,244), border 1px, r 8px | missing → identical |
| notification-center / rows | portaled (not in canvas) | were in-canvas (+4 slots) → closed popover |
