# Wave 1c — 9 dedicated kernel ports

field, input-group, rating, copy-button, fab, toggle-group, metric, avatar-group, button-group.

### rating
Wave 1t geometry parity (measured vs React `Rating` fixture `default`, aurora/dark, 1280x900): 0 mismatches.

DOM: `<div data-slot="rating" role="slider" aria-label aria-valuemin="0" aria-valuemax="5" aria-valuenow aria-valuetext="N out of 5" aria-readonly="true">`
+ 5 × `<span aria-hidden="true" data-slot="rating-item" data-state="on|off"><span data-slot="rating-star"><svg lucide-star/><span><svg lucide-star/></span></span></span>`.
CSS: root inline-flex, gap .25rem, radius-md; every svg 1.25rem; star `relative inline-flex`; outline svg `fg-muted`;
overlay span `absolute inset-0 overflow-hidden`, width 0% (off) / 100% (on); overlay svg stroke+fill `warning`.

| slot | React | Cronus before → after |
|---|---|---|
| rating | 24,24 116×20 r 10px | r 0 → equal |
| rating-item ×5 | 20×20, 16px/24px, color fg | 20px/20px `★` glyph, warning/tertiary color → equal |
| rating-star ×5 | span 20×20 | missing → equal |

Divergences: React clips the overlay with an inline `style="width:N%"` (half stars); the kernel has no inline
styles, so `data-state` drives 0%/100% (integer values). Keyboard/pointer rating needs JS → `aria-readonly`, no tabindex.
### toggle-group

Wave 1t geometry parity (2026-09-14, fixture Day/Week `value:"Day"`, aurora/dark, 1280x900).
DOM: `<div data-slot="toggle-group" role="group" aria-label>` + per option
`<button type="button" data-slot="toggle-group-item" data-state="on|off" aria-pressed disabled>`.
The emitted `label` ("Range") is the group name, never an option (it used to leak as a 3rd item).
Pressed option: item `pressed`/`on`, else group `value` (props or trailing `value:` item config), else first.
Roles stay `group`/`aria-pressed` (React/Radix uses `radiogroup`/`radio`; the stub gate reserves
`role="radiogroup"` as the interact radios fingerprint). Switching needs JS, so items are native
`disabled` buttons without dimming; only `[data-disabled]` dims (Radix's attribute).
CSS: item gains `line-height: 1.25rem` (React `text-sm` 14px/20px; was 21px).

| slot | React | Cronus |
|---|---|---|
| toggle-group | 24,24 | identical, text "Day Week" |
| item Day (on) | 24,24 49.31×40, 14px/20px | identical |
| item Week | 77.31,24 60.56×40 | identical |

### button-group

Wave 1t geometry parity (2026-09-14, fixture Save/Cancel, aurora/dark).
DOM: `<div data-slot="button-group" role="group" data-orientation aria-label>` +
`<button type="button" data-slot="button" data-variant="primary">` per item; `label` ("Actions") is aria only.
CSS (scoped, shared Button base untouched): `[data-slot="button-group"] > [data-slot="button"] { line-height: 1.25rem; }`
and primary `border-width: 0` (React primary Button has no border; the shared base has `line-height: 1` and a
1px transparent border).

| slot | React | Cronus |
|---|---|---|
| button-group | 24,24 140.11×40 | identical |
| button Save | 24,24 63.75×40, 14px/20px, border 0, r 14/0/0/14 | identical |
| button Cancel | 86.75,24 77.36×40 (−1px overlap), r 0/14/14/0 | identical |
## Wave 1t — geometry parity (React vs Cronus, aurora/dark, canvas 480)

### copy-button
DOM (React idle, Button ghost/icon): `<button data-slot="copy-button" data-variant="ghost" type="button" aria-label="Copy"><svg …lucide Copy…></svg><span aria-live="polite"></span></button>`.
No visible label (the emitter's `label "Copy"` was leaking as text). `aria-label` from props or item config.
CSS: `width/height: 2.25rem; padding: 0; border: 0; line-height: 1.25rem`; `svg` 1rem;
`> [aria-live]` sr-only.
Divergence (zero JS): no clipboard write, so the "copied" state (Check glyph + "Copied"
announcement) never appears; idle render is identical.

| slot | React | Cronus (before → after) |
|---|---|---|
| copy-button | 24,24 36x36 lh20 border 0 "" | 60.23x36 lh14 border 1 "Copy" → 24,24 36x36 lh20 border 0 "" |

### fab
DOM: `<div data-slot="fab"><button type="button" aria-label="Create"><span><svg plus/></span></button></div>`.
The main button has no `data-slot` (React has none); CSS `[data-slot="fab"] > button` (56px,
primary, `rounded-full`), `> span` inline-flex, `svg` 1.5rem. The label is only the accessible name.
Speed-dial `actions` need JS and are not rendered.

| slot | React | Cronus (before → after) |
|---|---|---|
| fab | 24,24 56x56 "" | "Create" → 24,24 56x56 "" |
| fab-button | — (not a slot) | 56x56 → — |

### metric
DOM: `<div data-slot="metric"><div data-slot="metric-label">Users</div><div data-slot="metric-value">1,240</div></div>`.
Value is now also read from item config (`value:"1,240"` after `label` attaches there).
CSS: label `0.75rem/1rem`, value `1.5rem/2rem`.

| slot | React | Cronus |
|---|---|---|
| metric | 24,24 432x52 | 24,24 432x52 |
| metric-label | 24,24 432x16 lh16 | 24,24 432x16 lh16 |
| metric-value | 24,44 432x32 lh32 "1,240" | 24,44 432x32 lh32 "1,240" |

### avatar-group
DOM: `<div data-slot="avatar-group" role="group" aria-label="Team">` + one `avatar`/`avatar-fallback`
per member. Members are the non-`label`/`title` items; the `label` line is only used as a
member when no other item exists. `aria-label` from props or item config.
CSS: overflow chip `line-height: 1.25rem` + `ring-2` shadow.

| slot | React | Cronus (before → after) |
|---|---|---|
| avatar-group | 24,24 432x36 "AL JB" | "TE AL JB" → "AL JB" |
| avatar#0 / #1 | 24,24 / 52,24 36x36 | 24,24 / 52,24 36x36 (extra avatar#2 removed) |
| avatar-fallback | lh20 | lh21 → lh20 |
