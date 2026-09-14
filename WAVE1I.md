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
