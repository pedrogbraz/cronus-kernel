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
### app-shell

Wave 1t geometry parity (2026-09-14, fixture title Acme, items Home/Inbox, aurora/dark). React `AppShell`
is a composition over the Sidebar system and has no `app-shell` root slot. DOM:
`<div data-slot="sidebar-wrapper" data-state="expanded" data-app-shell="">` + the Sidebar `aside()` subtree
(text items = menu buttons) + `<div data-slot="app-shell-content"><header data-slot="app-shell-header"><span>{label}</span></header>`
`<div data-slot="app-shell-body">[<div>{description}</div>]</div></div></div>`.
Body content comes from the `description` attr (props or item config) — the React fixture hardcodes
`<div class="p-3 text-sm">Inbox</div>`. Gate keys app-shell checks on `app-shell-content` + `sidebar-wrapper`.
CSS: `[data-slot="app-shell"]` rule removed; content flex-1 min-w-0 min-h 100svh; header sticky 3.5rem,
border-bottom, `color-mix(in oklab, surface-base 80%, transparent)`, blur(8px), `0 1rem`, inherits 16px/24px;
body `> div` .75rem 14px/20px. Fixture wrapper `h-56 w-80 overflow-hidden` mirrored under `[data-audit-canvas]`.
Note: content height is `100svh` on both sides (900px in a 900px viewport).

| slot | React | Cronus |
|---|---|---|
| sidebar-wrapper | 24,24 320×224 | identical |
| sidebar / sidebar-content | 24,24 256×224 / 255×224 | identical |
| sidebar-menu-button ×2 | 32,32 / 32,68 239×32 | identical |
| app-shell-content | 280,24 64×900 | identical |
| app-shell-header | 280,24 64×56, 16px/24px 400, bg rgba(10,10,12,.80) | identical |
| app-shell-body | 280,80 64×844, text "Inbox" | identical (with `description:"Inbox"`) |

### table-of-contents

Wave 1t geometry parity (2026-09-14, fixture Overview/Usage, aurora/dark).
DOM: `<nav data-slot="table-of-contents" aria-label><span aria-hidden="true" data-slot="table-of-contents-indicator"></span>`
`<ul data-slot="table-of-contents-list"><li><a data-slot="table-of-contents-link" href="#slug" data-depth="0">`.
Indicator idle = opacity 0 (no scroll-spy without JS). `label` is aria only (it used to leak as a 1st link).
CSS: nav 14px/20px; indicator absolute start 0 top 0 width .125rem 9999px `primary` opacity 0;
link end radii `radius-md`, leading 1.375.

| slot | React | Cronus |
|---|---|---|
| table-of-contents / list | 24,24 432×62.5, 14px/20px | identical |
| indicator | 24,24 2×0, bg primary | identical |
| link ×2 | 25,24 / 25,55.25 431×31.25, r 0/10/10/0 | identical |

### resizable

Wave 1t geometry parity (2026-09-14, fixture One/Two, aurora/dark). DOM mirrors react-resizable-panels
(panels carry no data-slot; no kernel `resizable` wrapper):
`<div data-slot="resizable-panel-group" data-panel-group-direction="horizontal" aria-label>`
`<div data-panel="" data-panel-size="50.0">One</div><div data-slot="resizable-handle" role="separator" aria-valuenow="50" aria-valuemin="0" aria-valuemax="100" data-panel-group-direction="horizontal"></div><div data-panel="" data-panel-size="50.0">Two</div></div>`.
Dragging needs JS: the handle is a plain non-focusable separator (React's is a `div` with `tabindex`,
which cannot be `disabled`). CSS: group flex row 100%/100% overflow hidden; panels `flex: 50 1 0px`; handle 1px
`border` colour + 0.25rem `::after` hit area. React's inline `height:100%` overrides the fixture `h-32 w-72`,
so no fixture size is mirrored. `resizable-panel` / `resizable` CSS removed.

| slot | React | Cronus |
|---|---|---|
| resizable-panel-group | 24,24 432×24, text "One Two" | identical |
| resizable-handle | 239.5,24 1×24 | identical |
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
### signature-pad
Wave 1t geometry parity. DOM = React idle (empty pad):
`div[data-slot=signature-pad][data-empty=true]` > `canvas[data-slot=signature-pad-canvas][role=img]`
+ `div[data-slot=signature-pad-hint][aria-hidden]` (`<div>` dashed rule + `<span>Sign here</span>`,
React's fixed caption) + `<div>` with two `button[data-slot=button][data-variant=ghost][data-size=icon-sm]`
(Undo last stroke / Clear signature, lucide Undo2 / Eraser) `disabled`, exactly as React while the
pad has no ink. The label/aria-label names the canvas only.
CSS: pad gets `background: var(--cronus-surface-inset); color: var(--cronus-fg)`; hint `> div`
`border-top: 1px dashed var(--cronus-border-strong)`, `> span` block, `margin-top: 0.375rem`,
`0.75rem/1rem`, fg-muted; actions `absolute right/bottom 0.375rem, gap 0.25rem`; buttons scoped
`border-width: 0; font-size: 0.875rem; line-height: 1.25rem`, `:disabled { opacity: .5 }` (React
`disabled:opacity-50`), svg 0.875rem.

| slot | React | Cronus |
|---|---|---|
| signature-pad | 24,24 432×160, bg rgba(14,14,16), r 18px, border 1px | identical |
| signature-pad-canvas | 25,25 430×158 | identical |
| signature-pad-hint | 45,132 390×23, 16px/24px | identical |
| button ×2 | 381,145 / 417,145 32×32, 14px/20px 500, rgb(159,159,169), r 14px, border 0 | identical |

0 mismatches. Divergence: drawing needs pointer JS, so the canvas never inks and both buttons stay
disabled. Shared-base note: kernel Button base has `line-height: 1` and `border: 1px solid transparent`
(React ghost: 20px line, no border); overridden only inside the pad.
