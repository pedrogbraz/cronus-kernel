# Wave 1m — 9 dedicated kernel ports

tilt-card, star-border, glass-card, terminal, video-player, text-effect, spotlight-card, animated-list, toast.
sankey-chart and meteors remain stubs.

## Wave 1s — geometry parity (2026-09-14)

### text-effect
DOM: `<p data-slot="text-effect">Headline</p>`.
CSS: `display: block; margin: 0; line-height: 1.5`; `cui-text-effect` 400ms
`ease-out` (framer `easeOut`) from `opacity: 0; filter: blur(8px)` to
`opacity: 1; filter: none` (React default `blur` preset; no translate, so the settled
computed transform is `none`). Reduced motion: no animation.

| slot | React (settled) | Cronus (settled) |
|---|---|---|
| text-effect | 24,24 432×24, 16/24, fg 250,250,249 | 24,24 432×24, 16/24, fg 250,250,249 |

Screenshot pixel diff: 0. Divergence: React renders an `sr-only` copy of the text plus
an `aria-hidden` span tree (one inline-block span per word, 40ms stagger). Cronus
animates the whole paragraph at once and has no inner spans, since per-word timing
would need a span per word plus inline delays.

## Wave 1t — geometry parity, effects A (2026-09-14)

Measured with the `e2e/audit/geometry.spec.ts` rules (FREEZE_CSS, settle loop, slot+occurrence pairing, rect ≤1px, colour ≤2/255) against the `default` fixture, aurora/dark, canvas 480px: **0 mismatches**.

### glass-card
DOM: `<div data-slot="glass-card"><div aria-hidden="true"></div><div>{label}</div></div>`
(React: top highlight line + relative children; only the root has a slot).
CSS: `position: relative; width: 18rem; padding: 1.5rem` (fixture `w-72` + `p-6`),
`border-radius: calc(var(--cronus-radius, 14px) + 8px)` (`rounded-2xl` = 22px),
`border: 1px solid var(--cronus-border-soft, var(--cronus-border))`, surface-raised 60%,
`backdrop-filter: blur(24px)`; highlight = `> [aria-hidden]` 1px gradient via `border-strong`
(the old `::before` had no `content` and never painted).

| slot | React | Cronus |
|---|---|---|
| glass-card | 24,24 288×74 r22 b1 bg 22,22,23,153 | same |

### animated-list
DOM unchanged (`ul` > `li data-slot="animated-list-item"`). Fix: `label` / `title` are never
rows when `text` / `item` rows exist — the emitter writes the fixture id (`label "default"`),
which leaked as a first row ("default Alpha Beta").

| slot | React | Cronus |
|---|---|---|
| animated-list | 24,24 432×56 "Alpha Beta" | same |
| animated-list-item #0/#1 | 432×24 "Alpha" / "Beta" | same |
