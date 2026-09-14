# Wave 1t K1 — geometry parity: form controls

Families: button, input, textarea, checkbox, switch, toggle, radio-group, slider, field, input-group.
Gate: `cooud-ui/e2e/audit/geometry.spec.ts` logic (same tolerances: rect ≤1px, colour ≤2/255),
measured React `localhost:4747` vs kernel `--audit-canvas`, aurora/dark, 1280×900 dsf 1.

| family / fixture | before | after |
|---|---|---|
| button/primary-md | 3 | 0 |
| input/empty | 1 | 0 |
| textarea/empty | 1 | 0 |
| checkbox/off | 2 | 0 |
| switch/off | 2 | 0 |
| toggle/off | 1 | 0 |
| radio-group/default | 2 | 0 |
| slider/half | 3 | 0 |
| field/default | 3 | 3 (emitter gap, see field) |
| input-group/default | 4 | 0 |

### button
DOM: `<button type="button" data-slot="button" data-variant data-size class="cui-btn">`.
CSS: base `border: 0 solid transparent` (React primary/ghost/destructive have no border;
secondary/outline set `border-width: 1px`), `line-height: 1.25rem` (text-sm), sm `1rem`, lg `1.5rem`.
Disabled: `disabled` attr + `[data-slot="button"]:disabled { opacity: 0.5; pointer-events: none; }`
— the inline `style=` is gone.
Measured: React 24,24 109.69×40 lh 20px border 0 · Cronus identical (was 111.69 wide, lh 14px, border 1px).

### input
DOM unchanged (`<input data-slot="input" type="text" placeholder aria-label>`).
CSS: `[data-slot="input"] { line-height: 1.25rem; }`. Measured lh 20px both (was 21px).

### textarea
DOM unchanged. CSS: `[data-slot="textarea"] { line-height: 1.25rem; }`. lh 20px both (was 21px).

### checkbox
DOM (Radix): `<button type="button" role="checkbox" aria-checked data-state value="on" data-slot="checkbox" aria-label>`;
checked adds an unslotted `<span data-state="checked">` + lucide Check svg. The label is the
accessible name only — no `checkbox-text` span (React renders none).
CSS: unchecked colour inherits (React sets none; was primary-foreground); `> span` flex centre, svg 0.875rem.
Measured 24,24 16×16, colour rgba(250,250,249) both.

### switch
DOM: `<button … role="switch" … value="on" data-slot="switch" aria-label><span data-state></span></button>`.
Radix Thumb has no data-slot, so the thumb is `[data-slot="switch"] > span`; no `switch-text` span.
Measured 24,24 36×20 both; no extra slots.

### toggle
DOM unchanged. CSS: `line-height: 1.25rem`. lh 20px both (was 21px).

### radio-group
DOM (Radix): `<div role="radiogroup" aria-required="false" dir="ltr" data-slot="radio-group" aria-label>`
+ per option `<button type="button" role="radio" aria-checked data-state value data-slot="radio-group-item" aria-label>`;
the checked item holds an unslotted `<span data-state="checked">` + lucide Circle (0.5rem, fill primary).
Options are every non-`label`/`title` item (`text` lines from the emitter); `value` (props or item config)
selects; no value → nothing checked (Radix). Previously `text` items were dropped and a single fake
option named after the component rendered.
Measured group 432×40 (two 16px items + 8px gap) both; was 432×16 with one item.

### slider
DOM (Radix, no inline style):
`<span dir="ltr" data-orientation="horizontal" aria-disabled="false" data-slot="slider" data-value="{0..100}">`
`<span data-orientation="horizontal"><span data-orientation="horizontal"></span></span>` (track/range)
`<span><span role="slider" aria-label aria-valuemin aria-valuemax aria-orientation data-orientation aria-valuenow></span></span>` (positioner/thumb).
React has no data-slot on track/range/thumb, so those slots were removed.
CSS: 101 rules `[data-slot="slider"][data-value="N"] { --cui-slider-value: N; }` (value rounded to an integer
for positioning; `aria-valuenow` keeps the exact value). Range `right: calc(100% - v*1%)`; positioner
`left: calc(v*1% + 0.5rem - v*0.01rem)` + `translateX(-50%)` = Radix thumb-in-bounds offset (+8px at 0, −8px at 100).
Divergence: no dragging/keyboard (zero JS); static idle state only.

### field
DOM: `<div data-slot="field"><label data-slot="field-label">…</label>[<p data-slot="field-description">…</p>]</div>`.
CSS: description `line-height: 1rem` (text-xs).
**Remaining (3 mismatches, not fixable in the kernel):** the fixture has `description: "We'll never share it."`
but `packages/audit/src/emit-cronus-fixture.ts` never writes `description`, so `app.cronus`
`FieldDefault` only has `label "Email"`. React: field 432×36 + `field-description` p 432×16; Cronus field 432×14.
The kernel renders a description from a second `text` item — the emitter needs `text "<description>"` for field.

### input-group
DOM: `<div data-slot="input-group"><div data-slot="input-group-addon" data-align="start">$</div><input data-slot="input" type="text" placeholder aria-label /></div>`
(addon was a `span`; input now carries the aria-label from props/item config).
CSS: group `line-height: 1.25rem` (text-sm, addon inherits). Measured group/addon/input lh 20px both.
