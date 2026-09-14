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

### combobox
Wave 1t geometry parity (React `Combobox` fixture `default`, closed, aurora/dark, 1280x900): 0 mismatches.

DOM (no wrapper — React has none):
`<button type="button" data-slot="combobox-trigger" data-variant="outline" role="combobox" aria-expanded="false" aria-haspopup="listbox" aria-label data-state="closed" data-placeholder="" disabled><span>Select fruit</span><svg lucide-chevrons-up-down/></button>`.
CSS: outline Button `h-10 px-4 w-full justify-between gap-2`, transparent bg, text-sm/1.25rem, weight 400,
`[data-placeholder]` → fg-tertiary, chevron 1rem ml .5rem opacity .6. Old `::after` caret and 16rem width removed.

| slot | React | Cronus before → after |
|---|---|---|
| combobox-trigger | 24,24 432×40, lh 20px, color fg-tertiary, bg transparent | 256×40, lh 21px, fg, inset → equal |
| combobox (wrapper) | — | kernel-only div → removed |

Divergence: opening, filtering and picking need JS → React's native trigger rendered `disabled` with React's idle
look (not dimmed); options are not emitted (React only mounts them while open). Real `disabled` → `data-disabled` (opacity .5).

### file-dropzone
Wave 1t geometry parity (React `FileDropzoneFixture` `default`): 0 mismatches.

DOM: `<label data-slot="file-dropzone" data-dragging="false"><input type="file" aria-label class="sr-only" /><svg lucide-cloud-upload/><span>Drag &amp; drop or <span>browse</span></span></label>`.
The fixture label names the input (aria-label); visible copy is React's default (`drop:` / `browse:` override it).
CSS adds line-height 1.25rem, svg 1.5rem fg-muted, inner span weight 500 fg.

| slot | React | Cronus before → after |
|---|---|---|
| file-dropzone | 24,24 432×136, lh 20px, text "Drag & drop or browse" | 432×105, lh 21px, "Upload files" → equal |

Divergence: drag-over highlight (`data-dragging="true"`) needs JS; click-to-pick works natively through the label.
### popover

Wave 1t geometry parity (2026-09-14). React `PopoverTrigger asChild` + `Button`: Radix Slot keeps the
Button's own slot, so the trigger is `button[data-slot="button"][data-variant="primary"][popovertarget]`
(no `popover-trigger` slot) followed by a native `popover="auto"`
`div[data-slot="popover-content"][role="dialog"][aria-label]` (aria-label from props/item config,
default "Details"). Shared Button chrome plus a scoped
`[data-slot="button"]:has(+ [data-slot="popover-content"]) { border: 0; line-height: 1.25rem; }`
(the base Button block has a 1px transparent border and `line-height: 1`; React has neither).
`:popover-open` content: 18rem, p-3, 1px border, radius-lg, surface-floating, shadow-lg.
Removed stale rules: the `button:has(+ [popover][data-slot="popover-content"])` list entry, the
`popover-trigger` block, and a `button:has(+ popover-content)` rule that sat nested inside an unclosed
`file-dropzone:has(:focus-visible)` block (brace restored).

Limitation: the React fixture forces `defaultOpen` and Radix portals the open content to `<body>`, so
it sits outside the canvas and the gate does not measure it. Without JS a top-layer popover cannot open
at load, so the kernel content stays closed (no layout box) until the trigger is clicked. No PORTAL
entry is proposed: Radix centres the 18rem content on a 67.5px trigger and collision-clamps it to the
viewport edge, so its trigger-relative x depends on the audit page layout, not on CSS.

| slot | React | Cronus |
|---|---|---|
| button (trigger) | 24,24 67.5×40, 14px/20px 500, bg rgb(0,166,244), r 14px, border 0 | identical |

### hover-card

Wave 1t geometry parity (2026-09-14). React `HoverCardTrigger asChild` + `Button variant="link"`:
`<span>` (slotless anchor) > `button[data-slot="button"][data-variant="link"]` +
`div[data-slot="hover-card-content"]`. The `hover-card` wrapper slot and `hover-card-trigger` slot are
gone (React DOM has neither). Content is `display: none` and shows on `span:hover` / `:focus-within`
(absolute, centred, 4px below, 16rem, p-3, radius-lg, surface-floating, shadow-lg), so hover works with
zero JS. The catalog specimen z-index hook is now `:has(span:hover > [data-slot="hover-card-content"])`.
Limitation: React's fixture forces `open` and portals the card out of the canvas; the kernel card shows
on hover only.

| slot | React | Cronus |
|---|---|---|
| button (trigger) | 24,24 89.89×40, 14px/20px 500, color rgb(0,166,244), bg transparent, border 0 | identical |

### dropdown-menu

Wave 1t geometry parity (2026-09-14). React `DropdownMenuTrigger asChild` + `Button`:
`button[data-slot="button"][data-variant="primary"][popovertarget][aria-haspopup="menu"]` + native
`popover="auto"` `div[data-slot="dropdown-menu-content"][role="menu"][aria-orientation="vertical"]` with
`div[data-slot="dropdown-menu-item"][role="menuitem"]` per text. No `dropdown-menu` wrapper or
`dropdown-menu-trigger` slot (React has none). Items: flex, gap .5rem, padding .375rem .5rem,
0.875rem/1.25rem, radius-md, hover surface-overlay. `:popover-open` content: min-width 8rem, p-1,
1px border, radius-lg, surface-floating, shadow-lg. Same portal/defaultOpen limitation as popover (no PORTAL entry:
centre-aligned content is viewport-clamped in React).

| slot | React | Cronus |
|---|---|---|
| button (trigger) | 24,24 81.59×40, 14px/20px 500, bg rgb(0,166,244), r 14px, border 0 | identical |

### mode-toggle

Wave 1t geometry parity (2026-09-14). The audit emitter drops `mode`; the React fixture defaults to
`light` and names the *next* mode in `aria-label`. The kernel now defaults to `light` (`mode` prop,
`+dark`/`+light` style, or `aria-label="Switch to light mode"` select dark). Switching theme needs JS,
so the button is always rendered `disabled` with React's idle look; it is dimmed only with
`data-disabled` (author `disabled:true`), mirroring React's `disabled:opacity-50`. Chrome fix: two stray
`}` after the `:disabled` and `svg` rules broke the next rule (the 1.25rem svg size was dropped); hover
is now surface-overlay / fg.

| slot | React | Cronus (before → after) |
|---|---|---|
| mode-toggle-core | 37,37 10×10 | 39.75,39.75 31.5×31.5 (dark scale) → identical |
| mode-toggle-rays | 32.83,32.83 18.33×18.33 | 33.75,33.75 16.5×16.5 → identical |
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
