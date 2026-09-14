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
