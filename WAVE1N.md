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

## Wave 1t — geometry parity (2026-09-14)

Shared: the `label` names the widget (`aria-label`, read from props or item config) and is no
longer rendered as content. Zero-JS control contract (Wave 1t): controls that need a runtime are React's own native element marked `disabled`, in React's idle look (dimmed only where React dims).

### description-list
DOM: `<dl data-slot="description-list" aria-label="Details">` + `<div data-slot="description-item">`
`<dt data-slot="description-term">Name</dt><dd data-slot="description-details">Ada</dd></div>` per pair.
CSS (stacked, md): root block `width: 18rem; max-width: 100%` (harness `w-72`) 14/20; items after the
first `margin-top: 1rem`; term w500 `fg-secondary`; details `margin-top: 0.25rem` `fg`.

| slot | React | Cronus |
|---|---|---|
| description-list | 24,24 288×104 14/20 | same |
| description-item ×2 | 288×44 at y 24 / 84 | same |
| description-term / details | 288×20 at y 24 / 48 | same |

Result: 0 mismatches.

### kanban
DOM: `<div data-slot="kanban" aria-label="Board">` + per column `<section data-slot="kanban-column" aria-label="Todo">`
`<header data-slot="kanban-column-header"><h3 data-slot="kanban-column-title">Todo</h3>`
`<span data-slot="kanban-column-count" data-variant="secondary">1</span></header>`
`<ul data-slot="kanban-column-list"><li data-slot="kanban-card">`
`<button type="button" data-slot="kanban-drag-handle" aria-label="Reorder card" disabled>`grip svg`</button>`
`<div data-slot="kanban-card-body"><span data-slot="kanban-card-title">Ship</span>`[`<span data-slot="kanban-card-description">` from `description:` config]`</div></li></ul></section>`.
Texts pair `title, card, …` like `kanban-fixture.tsx`; label-only → one column with the label as its card.
Drag/keyboard reorder need JS: grip is `disabled`.
CSS: board flex gap 1rem pb 0.5rem overflow-x auto; column 18rem radius-xl border `surface-inset`;
header p 0.625rem 0.75rem; title display font 14/20 w600 truncate (header-scoped override beats
`html[data-cronus-theme] h3` weight 400 / -0.02em); count = secondary badge 12/16 w500 radius-md;
title + count `tabular-nums` (inherited on the React page); list min-h 6rem p 0.5rem gap 0.5rem;
card flex gap 0.5rem p 0.75rem radius-lg border `surface-raised` (16/24); grip 20px mt 2px `fg-muted`;
card title w500 leading 1.375.

| slot | React | Cronus |
|---|---|---|
| kanban | 24,24 432×148 | same |
| kanban-column ×2 | 288×140 r18 at x 24 / 328 | same |
| kanban-column-header | 286×42 | same |
| kanban-column-title | 37,36 33.23×20 w600 | same |
| kanban-column-count | 273.27,35 25.73×22 r10 | same |
| kanban-column-list | 25,67 286×96 | same |
| kanban-card / drag-handle / card-body | 33,75 270×48 r14 · 46,90 20×20 · 74,88 216×22 | same |

Result: 0 mismatches.

### json-viewer
DOM (root object, depth 1): `<div data-slot="json-viewer" aria-label="Payload"><div data-slot="json-viewer-branch">`
`<div data-slot="json-viewer-row"><button data-slot="json-viewer-toggle" data-state="open" aria-expanded="true" aria-label="Toggle root" disabled>`
chevron `</button><span><span data-json="punct">{</span></span><button data-slot="copy-button" data-variant="ghost" aria-label="Copy value" disabled></div>`
`<div data-slot="json-viewer-children">` leaf rows (spacer, `<span data-slot="json-viewer-key">"name"</span>`,
`: `, `<span data-slot="json-viewer-value" data-type="string">"Ada"</span>`, `,`, copy button)
`</div><div data-slot="json-viewer-closer">` spacer `}` `</div>`.
Entries from `key: value` texts typed like JSON literals; no entries → harness payload
`{ "name": "Ada", "ok": true }`. Toggle (collapse) and copy buttons (clipboard) are `disabled`;
copy buttons keep React's idle `opacity: 0`.
CSS: root block 18rem (harness `w-72`) p 1rem border radius-xl `surface-inset` mono 14/24; rows flex
gap 0.25rem; toggle/spacer/copy 20px (toggle/copy mt 2px, radius-md); children ml 0.625rem pl 0.875rem
left border; punct `fg-tertiary`, key `fg-secondary`, string `success-text`, number `info-text`,
boolean `warning-text`, null `fg-tertiary`.

| slot | React | Cronus |
|---|---|---|
| json-viewer | 24,24 288×130 r18 mono 14/24 | same |
| json-viewer-branch | 41,41 254×96 | same |
| json-viewer-toggle / copy-button#0 | 41,43 20×20 r10 / 275,43 20×20 | same |
| json-viewer-children | 51,65 244×48 | same |
| json-viewer-key / value (name) | 90,69 50.58×16 / 157.44,69 42.16×16 rgb 5,192,134 | same |
| json-viewer-closer | 41,113 254×24 | same |

Result: 0 mismatches.
