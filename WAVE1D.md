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
