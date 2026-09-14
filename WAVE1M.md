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

## Wave 1t — geometry parity (2026-09-14)

### tilt-card
DOM: `<div data-slot="tilt-card"><div>Hover me</div></div>` (React content wrapper; `glare`/`parallax`
off). CSS adds `width: 18rem` (fixture `w-72`), `border-radius: calc(var(--cronus-radius, 14px) + 8px)`
(`rounded-2xl` = 22px; was radius-xl 18px), content `relative`. Transform stays at the 0deg rest
(no pointer JS).

| slot | React | Cronus |
|---|---|---|
| tilt-card | 24,24 288×74 r22 | 24,24 288×74 r22 |

### star-border
DOM: `<div data-slot="star-border"><div aria-hidden="true"><span></span><span></span></div><div>Twinkle</div></div>`.
CSS root `relative; width: 18rem; rounded-2xl`; clip layer `absolute inset-0 overflow-hidden
radius inherit`; sparkles `0.375rem` `primary` round, `offset-path: rect(0 auto auto 0 round 12px)`,
`cui-star-border` 6s, second `animation-delay: -3s`; content relative; reduced motion hides layer.

| slot | React | Cronus |
|---|---|---|
| star-border | 24,24 288×24 r12 (used) | 24,24 288×24 r12 (was 432 wide, r0) |

### spotlight-card
DOM: `<div data-slot="spotlight-card"><div aria-hidden="true"></div><div>Spotlight</div></div>`.
Family block (after the shared card rule): `width: 18rem`, `rounded-2xl` (22px), `box-shadow: none`;
spotlight layer `absolute -1px` radial at `--spot-x/--spot-y` (default 50%), opacity 0 → 1 on
`:hover` (React: pointer JS); content relative; reduced motion hides layer.

| slot | React | Cronus |
|---|---|---|
| spotlight-card | 24,24 288×74 r22 | 24,24 288×74 r22 |
