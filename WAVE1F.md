# Wave 1f — 9 dedicated kernel ports

area-chart, bar-chart, line-chart, sparkline, pie-chart, data-table, sidebar, sonner, navigation-menu.

### sidebar

Wave 1t geometry parity (2026-09-14, fixture Home/Inbox, aurora/dark). DOM mirrors React
`SidebarProvider` + `Sidebar collapsible="none"`:
`<div data-slot="sidebar-wrapper" data-state="expanded"><aside data-slot="sidebar" data-variant="sidebar" data-side="left">`
`<nav aria-label><div data-slot="sidebar-content"><div data-slot="scroll-area"><div><ul data-slot="sidebar-menu">`
+ `<li data-slot="sidebar-menu-item"><button type="button" data-slot="sidebar-menu-button" data-active [aria-current="page"] disabled>`
(`<a>` when linked). `label` is the nav `aria-label`, never an item. Link-less navigation needs JS, so those
buttons are native `disabled`, not dimmed. `aside()` / `menu_entries()` are reused by AppShell.
CSS: wrapper flex 100% min-h 100svh; aside 16rem, `height: 100svh`, border-right; nav/content flex column;
`sidebar-content > scroll-area` height 100% (overrides the ScrollArea family's 12rem max-height); inner div
flex column gap .5rem p .5rem; menu gap .25rem; button 2rem, `0 .5rem`, `radius-md`, 14px/20px `fg-secondary`;
active `surface-inset` + `fg` + 500. Fixture `className="h-56 w-52 min-h-0"` / `h-full` are mirrored only under
`[data-audit-canvas]` (13rem × 14rem wrapper, aside height 100%). Gate: a `scroll-area` without
`scroll-area-viewport` is accepted inside `sidebar-content` (React's sidebar ScrollArea has no viewport slot).

| slot | React | Cronus |
|---|---|---|
| sidebar-wrapper / sidebar | 24,24 208×224 | identical |
| sidebar-content / scroll-area | 24,24 207×224 | identical |
| sidebar-menu | 32,32 191×68 | identical |
| menu-button Home (active) | 32,32 191×32, 14px/20px 500, bg rgb(14,14,16), fg rgb(250,250,249) | identical |
| menu-button Inbox | 32,68 191×32, 400, fg-secondary | identical |
