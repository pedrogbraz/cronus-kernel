# Wave 1f — 9 dedicated kernel ports

area-chart, bar-chart, line-chart, sparkline, pie-chart, data-table, sidebar, sonner, navigation-menu.

### navigation-menu

Wave 1t geometry parity (2026-09-14). DOM mirrors the React fixture: `nav[data-slot="navigation-menu"]
[aria-label="Main"]` > Radix's slotless `<div>` > `ul[data-slot="navigation-menu-list"]` > one
`li[data-slot="navigation-menu-item"]` per text (label + items) with
`button[data-slot="navigation-menu-trigger"][data-state="closed"]` = label + ChevronDown svg (0.875rem,
top 1px, fg-tertiary). The fixture triggers have no content, so the old always-open
`navigation-menu-content` (label duplicated as content) is gone, and so is its CSS. Trigger: h 2.25rem,
padding .5rem 1rem, gap .25rem, 0.875rem/1.25rem 500, fg-secondary, transparent, radius-lg. Radix
toggles need JS: triggers are rendered `disabled` with React's closed, undimmed look. No Radix viewport
(empty, absolutely positioned, slotless).

| slot | React | Cronus (before → after) |
|---|---|---|
| navigation-menu / list | 24,24 311.89×36, text "Products Analytics Docs" | 257.89 wide, "Products Products …" → identical |
| item/trigger #0 | 24,24 109.63×36, 14px/20px 500, rgb(159,159,169), bg transparent | 91.63 wide, open bg → identical |
| item/trigger #1 | 137.63,24 110.69×36 | 119.63 → identical |
| item/trigger #2 | 252.31,24 83.58×36 | 216.31 → identical |
