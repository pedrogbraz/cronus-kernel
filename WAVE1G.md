# Wave 1g — 9 dedicated kernel ports

radar-chart, scatter-chart, ring-chart, phone-input, currency-input, color-picker, scroll-area, toolbar, status-dot.
Remaining chart stub: sankey-chart. meteors still fx stub.

## Wave 1t — geometry parity (React vs Cronus, aurora/dark, canvas 480)

### status-dot
DOM (default, no `withLabel`): `<span data-slot="status-dot" data-status="online" role="status"><span aria-hidden="true" data-slot="status-dot-indicator"></span><span data-slot="status-dot-sr-label">Online</span></span>`.
The sr label is the status name (or an explicit `label:` attr), never the emitter's `label`
line (fixture id "default"). `withLabel:true` renders the visible `status-dot-label` instead.
CSS: `status-dot-sr-label` sr-only (1px, margin -1px, clip); `status-dot-label` `line-height: 1.25rem`.
Gate: status-dot needs `status-dot-indicator` plus `status-dot-label` **or** `status-dot-sr-label`.

| slot | React | Cronus (before → after) |
|---|---|---|
| status-dot | 24,32 10x10 "Online" | 24,26.5 60.94x21 "default" → 24,32 10x10 "Online" |
| status-dot-indicator | 24,32 10x10 | 24,32 10x10 |
| status-dot-sr-label | 23,36.5 1x1 | — → 23,36.5 1x1 |
