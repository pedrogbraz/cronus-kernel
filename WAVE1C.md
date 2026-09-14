# Wave 1c — 9 dedicated kernel ports

field, input-group, rating, copy-button, fab, toggle-group, metric, avatar-group, button-group.

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
