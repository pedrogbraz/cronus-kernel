# Wave 1f — 9 dedicated kernel ports

area-chart, bar-chart, line-chart, sparkline, pie-chart, data-table, sidebar, sonner, navigation-menu.

## Wave 1t — geometry parity (2026-09-14)

### data-table
DOM (React `data-table.tsx` over `table.tsx`, no toolbar/pagination):
`<div data-slot="data-table"><div data-slot="data-table-container">`
`<section data-slot="table-container" tabindex="0" aria-label="Table"><table data-slot="table">`
`<thead data-slot="table-header"><tr data-slot="table-row"><th data-slot="table-head" colspan="1" scope="col" aria-sort="none">…`
`<tbody data-slot="table-body"><tr data-slot="table-row"><td data-slot="table-cell">…`.
Content: bound rows → `columns` items + texts → first two content texts as headers, rest as
two-column rows. `label` names the table (never content); `text` items now count as content
(they were dropped as "field" kinds). Sorting/filtering/pagination need JS: unsorted idle state.
CSS: root flex column gap 0.75rem; container radius-xl + border + overflow hidden; table 14/20;
row bottom border (none on last body row); head h 2.5rem, px 0.75rem, weight 500, `fg-secondary`;
cell p 0.75rem, nowrap. Slot-scoped selectors (0,2,0) override the shared `[data-slot="data-table"] th/td` rules.

| slot | React | Cronus |
|---|---|---|
| data-table / container | 24,24 432×131.5 r18 b1 | same |
| table | 25,25 430×129.5 14/20 | same |
| table-head ×2 | 211.16×40 / 218.84×40 w500 | same |
| table-cell (row 1 / row 2) | h45 / h44.5 p12 | same |

Result: 0 mismatches **only with the proposed fixture source** (`text` Name, Role, Ada, Admin,
Linus, Editor). With today's emitted source (`label "Members"`, `text "Ada"`, `text "Linus"`)
25 mismatches remain, all text/row-count: `data-table-fixture.tsx` ignores fixture props and
hardcodes Name/Role · Ada/Admin · Linus/Editor, which the kernel cannot (and must not) invent.
