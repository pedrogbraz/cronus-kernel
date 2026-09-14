# Wave 1b — kernel ports

Dedicated CONTRACT renderers: alert, skeleton, banner, slider, radio-group, chip, avatar, card, empty.

Interact skipped for those families. Tests: `cargo test cronus_ui`.

## Wave 1t — geometry parity (React vs Cronus, aurora/dark, canvas 480)

### skeleton
DOM: `<div data-slot="skeleton" aria-hidden="true"></div>`.
CSS: `height: 1rem; width: 8rem` (harness default `h-4 w-32`; was 0.9rem → 14.4px, which also
clamped the used radius to 7.2px).

| slot | React | Cronus |
|---|---|---|
| skeleton | 24,24 128x16 r8 | 24,24 128x16 r8 |

### chip
DOM: `<span data-slot="chip">Design</span>`. CSS: `font-size: 0.875rem; line-height: 1.25rem`.

| slot | React | Cronus |
|---|---|---|
| chip | 24,24 67.75x28 lh20 | 24,24 67.75x28 lh20 |

### avatar
DOM: `<span data-slot="avatar"><span data-slot="avatar-fallback">AL</span></span>`.
CSS: fallback `font-size: 0.875rem; line-height: 1.25rem` (was 21px).

| slot | React | Cronus |
|---|---|---|
| avatar | 24,24 40x40 | 24,24 40x40 |
| avatar-fallback | 24,24 40x40 lh20 "AL" | 24,24 40x40 lh20 "AL" |

### empty
DOM: `<div data-slot="empty"><div data-slot="empty-title">No results</div></div>`.
CSS: title `font-family: var(--cronus-font-display, inherit); font-size: 0.875rem; line-height: 1.25rem`;
description also `line-height: 1.25rem`.

| slot | React | Cronus |
|---|---|---|
| empty | 24,24 432x118 | 24,24 432x118 |
| empty-title | 205.63,73 68.75x20 lh20 | 205.63,73 68.75x20 lh20 |
