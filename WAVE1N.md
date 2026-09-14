# Wave 1n — 9 dedicated kernel ports

carousel, code-block, description-list, kanban, json-viewer, animated-number, marquee, gradient-text, shiny-text.
sankey-chart and meteors remain stubs.

## Wave 1s — geometry parity (2026-09-14)

### code-block
DOM: `<div data-slot="code-block"><div data-slot="code-block-header"><div>`
`<span data-slot="code-block-filename">index.ts</span>`
`<span data-slot="code-block-language" data-variant="secondary">ts</span></div></div>`
`<div><section data-slot="code-block-scroll" tabindex="0" aria-label="Code block, index.ts">`
`<pre data-slot="code-block-pre"><code data-slot="code-block-code"><span>…</span></code></pre></section></div></div>`.
CSS: root `box-sizing: border-box; width: 18rem; max-width: 100%`, radius-xl (18px),
border, `surface-raised`, `line-height: 1.5`; header flex `gap: 0.75rem`,
`padding: 0.5rem 1rem`, bottom border, `surface-overlay`; header inner div
`min-height: 2rem` (holds React's 32px copy-button row height); filename mono 12/16
`fg-secondary` truncate; language = Badge secondary (`padding: 0.125rem 0.5rem`,
radius-md 10px, border, `surface-overlay`, 12/16, weight 500); pre `padding: 1rem`,
14px / 1.625, mono.

| slot | React | Cronus |
|---|---|---|
| code-block | 24,24 288×106 bg 21,21,23 r18 | 24,24 288×106 bg 21,21,23 r18 |
| code-block-header | 25,25 286×49 bg 30,30,33 pad 8/16 | same |
| code-block-filename | 41,41 58×16 SF Mono 12/16 | same |
| code-block-language | 107,38 29×22 r10 500 | same |
| code-block-pre | 25,74 286×55 14/22.75 | same |
| code-block-code | 41,90 254×23 | same |

Screenshot pixel diff: 101 px, all inside the 13×13 icon at 278–291, 42–55: React's
copy button. Divergence: no copy button (clipboard needs JS); the header row keeps
its 49px height and the button's slot on the right is empty.

## Wave 1t — geometry parity, effects A (2026-09-14)

Measured with the `e2e/audit/geometry.spec.ts` rules (FREEZE_CSS, settle loop, slot+occurrence pairing, rect ≤1px, colour ≤2/255) against the `default` fixture, aurora/dark, canvas 480px: **0 mismatches**.

### animated-number
DOM: `<span data-slot="animated-number"><span>{formatted}</span></span>` (React nesting).
Value is the settled target, read from `value` item, props, **or any item's config**
(`value:1234` after `label "Count"` attaches to the label item). CSS `display: inline;
font-variant-numeric: tabular-nums` (was `inline-block` + display font → 24px-tall box).
Divergence: no count-up tween (React mounts at the target anyway).

| slot | React | Cronus |
|---|---|---|
| animated-number | 24,27 43.5×18 "1,234" | same (was 9.8×24 "0") |
