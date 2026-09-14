# Wave 1g — 9 dedicated kernel ports

radar-chart, scatter-chart, ring-chart, phone-input, currency-input, color-picker, scroll-area, toolbar, status-dot.
Remaining chart stub: sankey-chart. meteors still fx stub.

### scroll-area
Wave 1t geometry parity. React/Radix idle DOM has one slot: `div[data-slot=scroll-area]` >
viewport `div[role=region][aria-label][tabindex=0]` > content. Radix `type="hover"` mounts the
scrollbar only while hovering/scrolling, so no `scroll-bar` slot at idle. Kernel:
`<div data-slot="scroll-area"><div role="region" aria-label tabindex="0"><div>rows…</div></div></div>`;
rows = `text` items (the label names the region, it is not a row).
CSS: root `width: 12rem; height: 8rem` (audit harness default `h-32 w-48`), region
`overflow-y: scroll; scrollbar-width: none` (+ `::-webkit-scrollbar` hidden, like Radix), inner
`flex column gap 0.25rem padding 0.5rem`. Gate interact fingerprint now keys on the focusable
viewport (`tabindex="0"`) instead of the removed `scroll-area-viewport` slot.

| slot | React | Cronus |
|---|---|---|
| scroll-area | 24,24 192×128, text = 12 rows | identical |

0 mismatches. Divergence: no custom hover scrollbar thumb (Radix JS); native scrolling works.
