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

### toast
Wave 1t geometry parity. React `ToastFixture`: `<div data-slot="toast" role="status">` block box,
`rounded-lg border bg-surface-floating px-4 py-3 text-sm text-fg shadow-lg`. Kernel DOM unchanged
(`aria-live="polite"` extra). CSS: one `[data-slot="toast"]` rule — `display: block`, padding
`0.75rem 1rem`, `font-size: 0.875rem; line-height: 1.25rem`, radius-lg, 1px border, surface-floating,
shadow-lg. Removed from the f5f80d5 chrome: `inline-flex`, `min-width: 14rem; max-width: 22rem`,
`font-weight: 500`, pop-in animation and the `::before` dot (none exist in React). The duplicate
toast rule that lived next to sonner is gone.

| slot | React | Cronus |
|---|---|---|
| toast | 24,24 432×46, 14px/20px 400, bg rgba(22,22,25), r 14px, border 1px | identical |

0 mismatches (geometry spec rules). Screenshots visually identical.
