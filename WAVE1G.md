# Wave 1g — 9 dedicated kernel ports

radar-chart, scatter-chart, ring-chart, phone-input, currency-input, color-picker, scroll-area, toolbar, status-dot.
Remaining chart stub: sankey-chart. meteors still fx stub.

### toolbar

Wave 1t geometry parity (2026-09-14, fixture Bold/Italic, aurora/dark).
DOM: `<div data-slot="toolbar" role="toolbar" aria-orientation="horizontal" aria-label>` +
`<button type="button" data-slot="toolbar-button" disabled>` per item. `label` ("Formatting") is aria only
(it used to leak as a 3rd button). Editor commands need JS: native `disabled` buttons, not dimmed.
CSS: button gains `line-height: 1.25rem; background: transparent; color: var(--cronus-fg-secondary)`.

| slot | React | Cronus |
|---|---|---|
| toolbar | 24,24 107.36×42 | identical |
| toolbar-button Bold / Italic | 29,29 45.7×32 / 78.7,29 47.66×32, 14px/20px, rgb(159,159,169) | identical |
