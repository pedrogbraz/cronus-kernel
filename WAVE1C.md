# Wave 1c — 9 dedicated kernel ports

field, input-group, rating, copy-button, fab, toggle-group, metric, avatar-group, button-group.

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
