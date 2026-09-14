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
