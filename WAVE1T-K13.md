# WAVE 1t — K13: geometry parity (drawer, sheet, lightbox, context-menu, shiny-text)

Contract: `e2e/audit/geometry.spec.ts` (Wave 1t UI overlay version). Pairing by
`data-slot` + occurrence. Rect ≤1px, equal fontSize/fontWeight/lineHeight, mono/sans,
color/background ≤2/255, used radius, border widths, innerText. No tolerance was
loosened and nothing was allowlisted.

Measurement:
- **drawer, sheet, lightbox (OVERLAY):** the kernel page and React (www `:4747`) are both
  measured in 640x777 contexts. Overlay/content subtrees are measured from the viewport origin.
- **context-menu (PORTAL):** anchored on `context-menu-content` itself.
- **shiny-text:** canvas-relative. A `display: contents` slot is measured through its
  single boxed child.

Kernel under test: this branch served by `cronus run --audit-canvas 5193`.
The fixture is a copy of `packages/audit/cronus-fixtures/app.cronus` plus
`description:"…"` on `DrawerDefault` / `SheetDefault` (see "Emitter").

Common rules:
- Zero JS, no `<script>`/`<style>`/`style=`.
- CSS only in `COMPONENT_CHROME`, via tokens.
- Controls that need JS are the same native `<button>` as React, with `disabled`, in
  React's idle look. They are dimmed only where React itself dims them.

### drawer

DOM (React vaul Drawer, open by default, no trigger and no `drawer` wrapper box):

```html
<div data-slot="drawer-overlay" aria-hidden="true"></div>
<div role="dialog" aria-labelledby="…" aria-describedby="…" data-slot="drawer-content">
  <div aria-hidden="true"></div>                       <!-- handle bar, no slot -->
  <div data-slot="drawer-header">
    <h2 id="…" data-slot="drawer-title">Filters</h2>
    <p id="…" data-slot="drawer-description">Narrow the list.</p>
  </div>
</div>
```

- The title comes from the `title` item, else the label.
- The description comes from `description:"…"` (props or item config), else extra
  `text` items. It is omitted when empty, as in React.

CSS (key values):
- **overlay:** fixed `inset: 0`, z 50, `color-mix(in srgb, black 50%, transparent)`.
- **content:**
  - fixed `left/right/bottom: 0`, flex column, `margin-top: 6rem`;
  - `1px solid var(--cronus-border)`, radius `var(--cronus-radius-xl)` top corners only;
  - `var(--cronus-surface-floating)`.
- **handle:** `3rem x 0.375rem`, `margin: 1rem auto 0`, pill, `var(--cronus-border)`.
- **header:** grid, `gap: 0.375rem`, `padding: 1rem`, centred below 40rem and start-aligned from 40rem.
- **title:**
  - scoped as `[data-slot="drawer-header"] > [data-slot="drawer-title"]`, so it beats
    `html[data-cronus-theme] h2` (400, -0.025em);
  - display font, `1.125rem/1.75rem`, 600, `letter-spacing: normal`.
- **description:** `0.875rem/1.25rem`, `var(--cronus-fg-secondary)`.

Limitation: swipe or overlay-click dismissal needs JS. React has no close control to mirror.

| slot | React | Cronus | result |
|---|---|---|---|
| drawer-overlay | div 0,0 640x777 | div 0,0 640x777 | = |
| drawer-content | div 0,667 640x110, bg rgb(22,22,25), border 1px, radius 18 18 0 0 | same | = |
| drawer-header | div 1,690 638x86 | same | = |
| drawer-title | h2 17,706 606x28, 18px/600/28px | same | = |
| drawer-description | p 17,740 606x20, 14px/20px, "Narrow the list." | same | = |

Result: **PASS (0 mismatches)**. Before the fix there were 14.

### sheet

DOM (Radix Sheet side=right, open by default, no trigger):

```html
<div data-slot="sheet-overlay" aria-hidden="true"></div>
<div role="dialog" aria-labelledby="…" aria-describedby="…" data-slot="sheet-content">
  <div data-slot="sheet-header">
    <h2 id="…" data-slot="sheet-title">Edit profile</h2>
    <p id="…" data-slot="sheet-description">Make changes to your profile here.</p>
  </div>
  <button type="button" data-slot="sheet-close" disabled><span>Close</span></button>
</div>
```

- The old closed `popover` sheet with `sheet-trigger` was removed. Its content had
  `display: none`, so it had no box.
- The shared-list selectors that hid or styled it were removed too:
  - `[data-slot="sheet-content"]:not(:popover-open)`;
  - `button:has(+ [popover][data-slot="sheet-content"])`;
  - the reduced-motion `[data-slot="sheet-content"][popover]`.

CSS (key values):
- **overlay:** fixed `inset: 0`, black 50% scrim, `backdrop-filter: blur(8px)`.
- **content:**
  - fixed `top/bottom/right: 0`, `width: 75%; max-width: 24rem; height: 100%`;
  - flex column, `gap: 1rem`, `padding: 1.5rem`;
  - `border-left-width: 1px`, surface-floating, `var(--cronus-shadow-lg)`.
- **header:** flex column, `gap: 0.375rem`.
- **title and description:** the same as drawer (title scoped under the header).
- **close:**
  - absolute `top: 1rem; inset-inline-end: 1rem`, `1rem x 1rem`, radius md;
  - `var(--cronus-fg-tertiary)`, X drawn with `::before/::after`;
  - the `<span>` is sr-only.

Limitation: closing needs JS. `sheet-close` is `disabled`, not dimmed (React's idle
close is not dimmed). Esc and overlay click are not reproduced.

| slot | React | Cronus | result |
|---|---|---|---|
| sheet-overlay | div 0,0 640x777 | same | = |
| sheet-content | div 256,0 384x777, border-left 1px | same | = |
| sheet-header | div 281,24 335x54 | same | = |
| sheet-title | h2 281,24 335x28, 18px/600/28px | same | = |
| sheet-description | p 281,58 335x20 | same | = |
| sheet-close | button 608,16 16x16, used radius 8px, rgb(133,133,142), text "Close" | same | = |

Result: **PASS (0 mismatches)**. Before the fix there were 7.

### lightbox

DOM (full-viewport Radix Dialog, at index 0):

```html
<div data-slot="dialog-overlay" aria-hidden="true"></div>
<div role="dialog" aria-label="Image gallery" data-slot="dialog-content">
  <div data-slot="lightbox">
    <div>
      <span data-slot="lightbox-counter">1 / 2</span>
      <button type="button" data-slot="lightbox-close" aria-label="Close" disabled></button>
    </div>
    <div>
      <button type="button" aria-label="Previous image" data-edge disabled></button>
      <img data-slot="lightbox-image" src="data:image/svg+xml,…64x64…" alt="First image">
      <button type="button" aria-label="Next image" disabled></button>
    </div>
    <div data-slot="lightbox-thumbnails">
      <button type="button" aria-label="View image 1" aria-current="true" disabled><img src="…" alt=""></button>
      <button type="button" aria-label="View image 2" aria-current="false" disabled><img src="…" alt=""></button>
    </div>
  </div>
</div>
```

- **Images and counter:** the images are the `text` items (alts). The counter reads
  `1 / N` and `0 / 0` without items. Thumbnails appear only when N > 1.
- **Placeholder image:** `.cronus` has no image source, so each image is a transparent
  64x64 SVG `data:` placeholder, painted with an inset box-shadow of
  `var(--cronus-fg-tertiary)`. That keeps the measured background transparent, as in React.
- **Caption:** `lightbox-caption` was removed (React has none without a caption).

CSS (key values):
- **Scoping:** rules are scoped with `:has(> [data-slot="lightbox"])` so the generic
  `dialog-content` / `dialog-overlay` of other families is not touched.
- **overlay:** fixed, black 50%, blur 8px.
- **content:**
  - fixed `inset: 0`, 100% x 100%, grid, `gap: 1rem`;
  - `padding: 0; border: 0; border-radius: 0`;
  - `color-mix(in srgb, black 95%, transparent)`, `color: white`.
- **header:** flex, space-between, `padding: 1rem`.
- **counter:** `0.875rem/1.25rem`, `color-mix(in srgb, white 70%, transparent)`.
- **close:** `2rem x 2rem`, `padding: 0.375rem`, radius md, white 70%, X via pseudo-elements.
- **stage:** `flex: 1 1 0%`, centred, `padding: 0 0.5rem`.
- **prev/next:** absolute, `2.5rem` round, black 40% / white 80%, `[data-edge]` opacity 0.4.
- **thumbnails:** flex centred, `gap: 0.5rem`, `overflow-x: auto`, `padding: 1rem`.
- **thumbnail buttons:** `3.5rem`, radius md, opacity 0.6; the current one is opacity 1 with
  a `0 0 0 2px white` ring.

Non-token colours: no black or white token exists in `cronus_ui_tokens.css`.
`bg-black/95`, `/50`, `/40` and `text-white`, `/70`, `/80` use `color-mix(in srgb, black|white N%, transparent)`
and the `white` keyword. The precedent is the old sheet `::backdrop` `color-mix(in oklch, black 45%, transparent)`.

Limitations:
- Close, previous, next and thumbnail selection need JS, so they are `disabled`.
- Only "previous" (at the first image) and "next" (at the last) are dimmed, as React dims them.
- Arrow-key navigation is not reproduced.

| slot | React | Cronus | result |
|---|---|---|---|
| dialog-overlay | div 0,0 640x777 | same | = |
| dialog-content | div 0,0 640x777, bg black/95, radius 0, border 0 | same | = |
| lightbox | div 0,0 640x777, white | same | = |
| lightbox-counter | span 16,22 **29.03**x20, 14px/20px, white/70, "1 / 2" | span 16,22 **26.34**x20, same style/text | **Δw 2.7** |
| lightbox-close | button 592,16 32x32, radius 10px, text "" | same | = |
| lightbox-image | img 288,344.5 64x64 | same | = |
| lightbox-thumbnails | div 0,689 640x88 | same | = |

Result: **1 mismatch (not in the renderer)**. Before the fix there were 19.
- **Cause:** www `apps/www` has a global `body { font-variant-numeric: tabular-nums; font-feature-settings: "cv02", "cv03", "cv04", "cv11"; }`.
  - Every React canvas and portal inherits it.
  - The kernel audit document (`src/ui/audit_layout.rs` + `AUDIT_PREFLIGHT`) does not declare it.
  - Tabular digits make "1 / 2" 2.7px wider.
- **Why not in the renderer:** the React Lightbox does not set it, so copying it into the
  renderer would diverge from real Cronus apps.
- **Proof:** injecting exactly that `body` rule into the kernel page, as a scratch-only
  diagnostic that is not committed, gives **lightbox 0 mismatches**.
  - With the rule: `lightbox-counter` measures 16,22 29.03x20 on both sides.
  - Colour 255,255,255,178 vs 179 (within 2/255).

### context-menu

DOM (Radix ContextMenu open, span trigger without a slot):

```html
<span>Right click</span>
<div data-slot="context-menu-content" role="menu" aria-orientation="vertical">
  <div data-slot="context-menu-item" role="menuitem">Back</div>
  <div data-slot="context-menu-item" role="menuitem">Reload</div>
</div>
```

- The trigger was `<button>` and is now `<span>`, as in React: a dead button removed.
- React opens the content at the pointer. Without a popper, the kernel keeps it in flow
  below the trigger (`margin-top: 0.5rem`). The spec measures the content against itself.

CSS (key values):
- **content:**
  - `width: max-content; min-width: 8rem`, `padding: 0.25rem`;
  - `1px solid var(--cronus-border)`, `var(--cronus-radius-lg)`;
  - surface-floating, `var(--cronus-shadow-lg)`.
- **item:**
  - flex, `gap: 0.5rem`, `padding: 0.375rem 0.5rem`, `var(--cronus-radius-md)`;
  - `0.875rem/1.25rem`;
  - hover `var(--cronus-surface-overlay)`.
- The old context-menu trigger rule was removed. It was nested inside
  `[data-slot="menubar-item"]:hover {}`; that empty menubar block stays untouched.

| slot | React | Cronus | result |
|---|---|---|---|
| context-menu-content | 0,0 128x74, bg rgb(22,22,25), radius 14px, border 1px | same | = |
| context-menu-item #0 | 5,5 118x32, 14px/20px, radius 10px | same | = |
| context-menu-item #1 | 5,37 118x32 | same | = |

Result: **PASS (0 mismatches)**. Before the fix there were 12.

### shiny-text

DOM (React: `display: contents` slot wrapping the painted span; the inline `<style>`
keyframes live in COMPONENT_CHROME):

```html
<span data-slot="shiny-text"><span>Sheen</span></span>
```

CSS:
- `[data-slot="shiny-text"] { display: contents; }`
- The painted `> span`:
  - `color: transparent`, `-webkit-background-clip: text; background-clip: text`;
  - `linear-gradient(90deg, fg-tertiary 0%, fg-tertiary 40%, fg 50%, fg-tertiary 60%, fg-tertiary 100%)`;
  - `background-size: 200% 100%`;
  - `animation: cui-shiny-text 3s linear infinite`, which is off under reduced motion.

| slot | React | Cronus | result |
|---|---|---|---|
| shiny-text | span (box of the child) 24,27 45.67x18, 16px/400/24px, color rgba(0,0,0,0) | same | = |

Result: **PASS (0 mismatches)**. Before the fix there was 1.

### Emitter and base proposals

- **Emitter** (`packages/audit/src/emit-cronus-fixture.ts`): React uses `description` for
  drawer and sheet, but it is not emitted. The coordinator decided to emit
  `description:"…"` for fixtures that have `description` (a string):
  ```ts
  if (typeof fixture.props.description === "string") {
    lines.push(`  description:"${cronusEscape(fixture.props.description)}"`);
  }
  ```
  The renderers read `description` from props or item config. Measured with the
  current, unpatched emitted `app.cronus`:
  - **drawer: 6 mismatches.**
    - `drawer-description` is missing.
    - `drawer-content` becomes 0,693 640x84 and `drawer-header` 1,716 638x60.
    - `drawer-title` moves to y 732.
    - The text of content and header is "Filters".
  - **sheet: 4 mismatches.**
    - `sheet-description` is missing.
    - `sheet-header` becomes 335x28.
    - The text of content and header lacks the description.

  Every one of these diffs is the missing description. With `description:"…"` both are 0.
- **Kernel audit document** (shared base, not changed here): declare the same global as www, e.g. in `render_audit_document`:
  `body { font-variant-numeric: tabular-nums; font-feature-settings: "cv02", "cv03", "cv04", "cv11"; }`.
  - It affects every family that renders digits.
  - Here it closes the last lightbox diff.
