# Wave 1g — 9 dedicated kernel ports

radar-chart, scatter-chart, ring-chart, phone-input, currency-input, color-picker, scroll-area, toolbar, status-dot.
Remaining chart stub: sankey-chart. meteors still fx stub.

### phone-input
Wave 1t geometry parity (React `PhoneInput` fixture `default`): 0 mismatches.

DOM: `<div data-slot="phone-input" role="group" aria-label><button type="button" role="combobox" aria-expanded="false" aria-haspopup="listbox" aria-label="Select country. Brazil, +55" data-slot="phone-input-country" data-state="closed" disabled><span aria-hidden="true">🇧🇷</span><span>+55</span><svg lucide-chevron-down/></button><span aria-hidden="true"></span><input type="tel" … data-slot="phone-input-field" placeholder="00 00000 0000" value="11 98765 4321" /></div>`.
CSS: root keeps inherited 16px/24px (no text-sm), align-items center, no overflow clip; trigger h-full, rounded-s-lg,
no border (1px divider span h-5 bg-border instead), flag 1rem/1, chevron .875rem fg-tertiary; field text-sm/1.25rem.
Value grouped by the country mask (React `formatNational`).

| slot | React | Cronus before → after |
|---|---|---|
| phone-input | 432×40, 16px/24px | 14px/21px → equal |
| phone-input-country | 25,25 92.08×38 r 14 0 0 14, border 0 | 70.69×38, r 0, border-right 1 → equal |
| phone-input-field | 118.08,25 336.92×38, lh 20px | 95.69,25 359.31×38, lh 21px → equal |

Divergence: country list is a Radix popover (JS) → native trigger `disabled`, idle look kept (group `aria-disabled` dims).

### currency-input
Wave 1t geometry parity (React `CurrencyInput` fixture `default`): 0 mismatches.

DOM: `<div data-slot="currency-input"><button type="button" disabled aria-label="Select currency" data-slot="currency-input-selector" aria-haspopup="menu" aria-expanded="false" data-state="closed"><span>R$</span><span>BRL</span><svg lucide-chevron-down/></button><input type="text" inputmode="decimal" … placeholder="0,00" aria-label data-slot="currency-input-field" value="123,45" /></div>`.
Default currency BRL (React list head); integer `value` = minor units, formatted per locale (pt-BR, en-US, de-DE, en-GB, ja-JP).
Single currency (`currencies:"EUR"` or `prefix:`) → React's static `<span data-slot="currency-input-prefix" aria-hidden="true">`.
CSS: root 40px bordered rounded-lg inset (was an empty rule), selector border-e ps-3 pe-2.5 gap-1.5, code text-xs/1rem
fg-tertiary, field end-aligned text-sm/1.25rem tabular-nums.

| slot | React | Cronus before → after |
|---|---|---|
| currency-input | 24,24 432×40 r 14 border 1 inset, text "R$ BRL" | 432×24 unstyled, "$" → equal |
| currency-input-selector | button 25,25 90.02×38 | missing (prefix span instead) → equal |
| currency-input-field | 115.02,25 339.98×38, 14px/20px | 58.98,24 175×24, 16px/24px → equal |

Divergence: currency menu is a Radix dropdown (JS) → native selector `disabled`, idle look kept (root `data-disabled` dims).

### color-picker
Wave 1t geometry parity (React `ColorPickerFixture` `default`): 0 mismatches under the current spec (no PORTAL entry).

DOM (no wrapper): `<button type="button" data-slot="color-picker-trigger" data-variant="outline" aria-label="Color: oklch(0.62 0.21 256)" aria-haspopup="dialog" aria-expanded="false" data-state="closed" disabled><span aria-hidden="true" data-slot="color-picker-swatch" data-color="oklch(0.62 0.21 256)"></span><span>oklch(0.62 0.21 256)</span></button>`.
CSS: outline Button `h-10 px-4 w-full justify-start gap-2` text-sm/1.25rem; tile 1.25rem rounded-md bordered,
`background-color: attr(data-color type(<color>), var(--cronus-primary))` (React uses inline `style`; kernel emits none;
browsers without typed `attr()` fall back to the primary token).

| slot | React | Cronus before → after |
|---|---|---|
| color-picker-trigger | 24,24 432×40 r 14 border 1, lh 20px | 256×40 r 0 border 0, lh 21px → equal |
| color-picker-swatch | 41,34 20×20 bg rgb(4,130,255) | 36,34, bg rgb(0,166,244) → equal |
| color-picker / -content / -swatches / -swatch-button ×5 | — (portaled) | always-open in canvas → removed |

Divergence: the editor (saturation area, hue slider, L/C/H channels, value, presets) needs pointer/keyboard JS →
not emitted; trigger is React's native button `disabled` with its idle look (real `disabled` → `data-disabled`, opacity .5).
The React fixture clicks it open, but that content is portaled out of the canvas and not compared by the spec.
### toolbar

Wave 1t geometry parity (2026-09-14, fixture Bold/Italic, aurora/dark).
DOM: `<div data-slot="toolbar" role="toolbar" aria-orientation="horizontal" aria-label>` +
`<button type="button" data-slot="toolbar-button" disabled>` per item. `label` ("Formatting") is aria only
(it used to leak as a 3rd button). Editor commands need JS: native `disabled` buttons, not dimmed.
CSS: button gains `line-height: 1.25rem; background: transparent; color: var(--cronus-fg-secondary)`.

| slot | React | Cronus |
|---|---|---|
| toolbar | 24,24 107.36×42 | identical |
| toolbar-button Bold / Italic | 29,29 45.7×32 / 78.7,29 47.66×32, 14px/20px, rgb(159,159,169) | identical |
## Wave 1t — geometry parity (React vs Cronus, aurora/dark, canvas 480)

### status-dot
DOM (default, no `withLabel`): `<span data-slot="status-dot" data-status="online" role="status"><span aria-hidden="true" data-slot="status-dot-indicator"></span><span data-slot="status-dot-sr-label">Online</span></span>`.
The sr label is the status name (or an explicit `label:` attr), never the emitter's `label`
line (fixture id "default"). `withLabel:true` renders the visible `status-dot-label` instead.
CSS: `status-dot-sr-label` sr-only (1px, margin -1px, clip); `status-dot-label` `line-height: 1.25rem`.
Gate: status-dot needs `status-dot-indicator` plus `status-dot-label` **or** `status-dot-sr-label`.

| slot | React | Cronus (before → after) |
|---|---|---|
| status-dot | 24,32 10x10 "Online" | 24,26.5 60.94x21 "default" → 24,32 10x10 "Online" |
| status-dot-indicator | 24,32 10x10 | 24,32 10x10 |
| status-dot-sr-label | 23,36.5 1x1 | — → 23,36.5 1x1 |
