# Wave 1d — 9 dedicated kernel ports

combobox, stepper, input-otp, file-dropzone, popover, hover-card, dropdown-menu, collapsible, mode-toggle.

### input-otp

Wave 1s geometry parity (2026-09-14). DOM (unchanged, mirrors the `input-otp` lib):
`<div data-input-otp-container="true"><div data-slot="input-otp-group">` + 6 × `<div data-slot="input-otp-slot">`
+ `<div><input data-slot="input-otp" autocomplete="one-time-code" inputmode="numeric" maxlength value></div></div>`.
CSS: container flex gap 0.5rem, `line-height: 1.5`, `cursor: text; user-select: none; pointer-events: none`
(lib inline styles); slot 2.5rem square, `border: 0 solid var(--cronus-border)` with top/right/bottom 1px,
first child left 1px + `radius-lg` left corners, last child `radius-lg` right corners, `font-size: 0.875rem;
line-height: 1.25rem`. Overlay input absolute inset 0, transparent text/caret.
Zero JS: slots show the initial value only (no live typing / active-slot ring).

Measured (Playwright 1280×900 dsf 1, aurora dark, rects relative to the 480px canvas):

| slot | React | Cronus |
|---|---|---|
| container (div) | 24,24 432×40 | 24,24 432×40 |
| input-otp-group | 24,24 240×40 | 24,24 240×40 |
| input-otp-slot ×6 | x 24…224 step 40, 40×40, 14px/20px, border 1px rgba(245,255,255,.10), r 14px | identical |
| input-otp (input) | 24,24 432×40, 40px/40px, ls −20px, transparent | identical |

### collapsible
Wave 1t geometry parity. React/Radix open state: root `<div data-state="open">` has no data-slot,
trigger `<button aria-expanded="true" data-state="open">`, content
`<div data-state="open" data-slot="collapsible-content">`. Kernel mirrors it (root slot removed);
body = non-label texts. CSS: `[data-slot="collapsible-content"]` gains `line-height: 1.25rem`
(`text-sm` pair).

| slot | React | Cronus |
|---|---|---|
| collapsible-content | 24,48 432×20, 14px/20px, rgb(159,159,169) | identical |

0 mismatches. Divergence: toggling needs JS; the fixture's `defaultOpen` state is rendered and the
trigger is `disabled` (not dimmed).
