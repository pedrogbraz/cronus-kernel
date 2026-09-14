# Wave 1h — 9 dedicated kernel ports

tags-input, autocomplete, multi-select, credit-card-input, floating-label-input, split-button, pill-nav, dock, workspace-switcher.

### multi-select

Wave 1s geometry parity (2026-09-14). DOM now mirrors React's open tree (Radix portal + cmdk):
```
<div data-slot="multi-select">                       (kernel-only wrapper, position: relative)
  <div data-slot="multi-select-trigger" role="combobox" tabindex="0" aria-expanded="true"
       aria-haspopup="listbox" aria-label data-state="open" data-placeholder="">
    <span><span>{placeholder}</span></span><span aria-hidden="true">{chevron svg}</span></div>
  <div data-slot="popover-content" role="dialog" data-state="open"><div data-slot="command">
    <div data-slot="command-list" role="listbox" aria-multiselectable="true">
      <div data-slot="command-item" role="option" aria-selected="false">
        <span data-slot="multi-select-indicator" data-state="unchecked" aria-hidden="true"></span><span>React</span>
      </div>…
```
CSS (all scoped under `multi-select`): trigger flex 100% min-h 2.5rem, `0.375rem 0.75rem`, 1px border
(`border-strong` while `aria-expanded="true"`), `radius-lg`, `surface-inset`, 14px/20px; text span flex-1
gap .375rem, placeholder `fg-muted`; chevron 1rem opacity .6. Content `position: absolute;
top: calc(100% + 4px); left: 0; width: 100%` (Radix sideOffset 4, trigger width), 1px border, `radius-lg`,
`surface-floating`, `shadow-lg`, p 0; list p .25rem max-h 20rem; item flex gap .5rem `radius-md`
`0.375rem 0.5rem` 14px/20px; indicator 1rem box 1px border `radius-sm` `surface-inset`
(checked: primary + check svg).

| slot | React | Cronus |
|---|---|---|
| multi-select-trigger | 24,24 432×40, bg rgb(14,14,16), r 14px | identical |
| text span / placeholder | 37,34 382×20 / 120×20 rgb(82,82,90) | identical |
| chevron svg | 427,36 16×16 | identical |
| popover-content | 24,68 432×115 | 24,68 432×74 |
| command-item ×2 | 29,114 / 29,146 422×32 | 29,73 / 29,105 422×32 |
| indicator | 37,122 16×16 r 6px | 37,81 16×16 r 6px |

Remaining divergences (documented, intentional):
- No cmdk search row (`command-input-wrapper` 41px + `<input>`): filtering needs JS; a non-filtering input
  would be a dead control. Content is therefore 41px shorter and rows sit 41px higher.
- React highlights the first row (`data-selected`, cmdk keyboard state) → kernel shows hover only.
- React portal colours come from `<html data-cronus-theme="neutral">` (the portal is outside the aurora
  pane: bg rgb(14,14,14), fg rgb(232,232,232)); kernel content stays inside the canvas and uses the aurora
  tokens (rgb(22,22,25) / rgb(250,250,249)). Harness artifact, not a token bug.
- Kernel-only `data-slot="multi-select"` wrapper (same rect as trigger) anchors the absolute content.
