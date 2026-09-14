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

### stepper

Wave 1t geometry parity (2026-09-14, fixture Account/Shipping, aurora/dark). DOM now mirrors React
`Stepper > StepperList > StepperItem(StepperIndicator + StepperTitle)` (no StepperTrigger in the fixture):
`<div data-slot="stepper" data-orientation><ol data-slot="stepper-list" data-orientation>` + per step
`<li data-slot="stepper-item" data-state="completed|active|upcoming" data-orientation [aria-current="step"]>`
`<span data-slot="stepper-indicator" data-state><span>{n}</span></span>` (lucide check svg once completed)
`<div data-slot="stepper-title">` `<span data-slot="stepper-item-state">current|not started|completed</span></li>`.
`label` ("Onboarding") is never a step. Gate: `<ol` is only a stub fingerprint without `stepper-list`.
CSS: list flex row items-center; item `:not(:last-child) { flex: 1 }`; indicator 2rem circle 14px/20px 500
(active `primary` + `#fff` + shadow-xs, completed primary 15% mix, upcoming 1px border `surface-overlay`
`fg-tertiary`); title 14px/1 500 (upcoming `fg-tertiary`); item-state sr-only. `stepper-trigger` CSS removed.

| slot | React | Cronus |
|---|---|---|
| stepper / stepper-list | 24,24 432×32 | identical, text "1 Account current 2 Shipping not started" |
| stepper-item #0 / #1 | 24,24 341.86×32 / 365.86,24 90.14×32 | identical |
| stepper-indicator #0 / #1 | 24,24 32×32 / 365.86,24 32×32 | identical |
| stepper-title #0 / #1 | 56,33 54.66×14 / 397.86,33 58.14×14 | identical |
| stepper-item-state | 23,39.5 1×1 | identical |
