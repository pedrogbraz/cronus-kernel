# Wave 1h — 9 dedicated kernel ports

tags-input, autocomplete, multi-select, credit-card-input, floating-label-input, split-button, pill-nav, dock, workspace-switcher.

### dock
Wave 1s geometry parity (measured end to end vs React `DockFixture`, aurora/dark, 1280x900).

DOM: `<div data-slot="dock" aria-label>` + per entry
`<button type="button" data-slot="dock-item" title aria-label><span aria-hidden="true"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="12" cy="12" r="7"></circle></svg></span></button>`
(`<a data-slot="dock-item" href title aria-label>` when linked). The name is only in `title`/`aria-label`, never visible text.
The kernel has no icon source, so every item shows the same decorative circle glyph the React fixture passes.

CSS (`COMPONENT_CHROME`): dock `inline-flex; align-items:flex-end; gap:0.5rem; padding:0.5rem 0.75rem; border:1px solid var(--cronus-border); border-radius:calc(var(--cronus-radius,14px) + 8px); background:color-mix(in oklab, var(--cronus-surface-raised) 70%, transparent); backdrop-filter:blur(8px); line-height:1.5`.
Item `inline-flex` centred, `2.75rem` square, `border-radius:var(--cronus-radius-xl)`, `background:var(--cronus-surface-overlay)`, `color:var(--cronus-fg)`, hover `surface-base`. `> span { display: contents }`, `svg { width:50%; height:50% }`.

| slot | React (x,y,w,h) | Cronus | radius | bg / color |
|---|---|---|---|---|
| dock | 24,24,122,62 | 24,24,122,62 | 22px / 22px | rgba(21,21,23,.70) both, blur(8px) both |
| dock-item #1 | 37,33,44,44 | 37,33,44,44 | 18px / 18px | rgb(30,30,33) / fg rgb(250,250,249) both |
| dock-item #2 | 89,33,44,44 | 89,33,44,44 | 18px / 18px | same |
| svg | 48,44,22,22 | 48,44,22,22 | — | currentColor |

Screenshots of both canvases are byte-identical PNGs.
Remaining delta: the `display:contents` glyph span reports a 0x0 rect on both sides; only its origin differs, and that origin is meaningless.
