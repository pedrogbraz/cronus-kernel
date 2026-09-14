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
