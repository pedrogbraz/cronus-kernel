# Wave 1a — 9 dedicated kernel ports

`PORTED_FAMILIES` after button/badge/input: label, textarea, checkbox, switch, spinner, separator, kbd, toggle, progress.

Each family has a dedicated `cronus_ui_{family}.rs`. Interact is skipped. No stubs.

## Wave 1t — geometry parity (React vs Cronus, aurora/dark, canvas 480)

### badge
DOM: `<span data-slot="badge" data-variant="default">New</span>`.
CSS: `font-size: 0.75rem; line-height: 1rem` (Tailwind `text-xs` pair; was canvas 1.5 → 24px tall).

| slot | React | Cronus |
|---|---|---|
| badge | 24,27 43.53x22 lh16 | 24,27 43.53x22 lh16 |

### spinner
DOM unchanged (lucide-style svg, `role="status"`, `aria-label="Loading"`).
CSS: `display: block` (Tailwind preflight `svg { display: block }`); `inline-block` sat on the
24px line box and dropped the svg 3.8px.

| slot | React | Cronus |
|---|---|---|
| spinner | 24,24 20x20 | 24,24 20x20 |

### progress
DOM: `<div data-slot="progress" role="progressbar" aria-valuenow aria-valuemin aria-valuemax aria-valuetext="50%" data-state="loading" data-value data-max aria-label><div data-state data-value data-max style="transform:translateX(-50%)"></div></div>`.
The indicator has **no** `data-slot` (Radix Indicator in React has none); CSS targets
`[data-slot="progress"] > div`. `aria-label` read from props or item config. The inline
`transform` is pre-existing (value-driven, same as React's inline style).

| slot | React | Cronus |
|---|---|---|
| progress | 24,24 432x8 | 24,24 432x8 |
| progress-indicator | — (not a slot) | — (was a slot at -192,24 432x8) |
