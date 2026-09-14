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
