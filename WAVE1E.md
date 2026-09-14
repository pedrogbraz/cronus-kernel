# Wave 1e — 9 dedicated kernel ports

command, menubar, context-menu, drawer, sheet, calendar, date-picker, time-picker, date-range-picker.

### time-picker

Wave 1s geometry parity (2026-09-14). Root `button[data-slot="time-picker"][data-variant="outline"]`
(Clock SVG + `<span>09:30</span>`) + hidden native `popover` sibling `time-picker-content` (display none
until opened: no layout impact). Own CSS block (no longer shared with `date-picker-trigger`), React
Button outline md + `w-[240px] justify-start gap-2 font-normal`: 15rem × 2.5rem, `padding: 0 1rem`,
1px `--cronus-border`, `radius-lg`, `background: transparent`, `box-shadow: var(--cronus-shadow-xs)`,
`0.875rem / 1.25rem` 400, hover `surface-overlay`; svg 1rem `color: var(--cronus-fg-tertiary)` opacity 1;
label span tabular-nums. Audit fixture needs `hourCycle:24` (React shows "09:30").

| slot | React | Cronus |
|---|---|---|
| time-picker (button) | 24,24 240×40, 14px/20px 400, bg transparent, shadow yes | identical |
| svg | 41,36 16×16, rgb(133,133,142) | identical |
| span | 65,34 38.5×20 | identical |

### command

Wave 1t geometry parity (2026-09-14). DOM mirrors cmdk: `div[data-slot="command"]` > visually hidden
`<label for>` (the accessible name, part of innerText like React's cmdk label) >
`div[data-slot="command-input-wrapper"]` (search svg 1rem fg-tertiary + `input[data-slot="command-input"]`)
> `div[data-slot="command-list"][role="listbox"]` > sizer `<div>` >
`div[data-slot="command-item"][role="option"]`; the first item carries cmdk's initial highlight
(`data-selected="true"`, surface-overlay while the list is not hovered). Source split: `label` = name,
first `text` = placeholder when it equals the label or ends with an ellipsis (the emitter writes the
placeholder right after the label), remaining texts = items. The label no longer leaks as placeholder or
as the first item. Fixture `className="h-48 w-72"` mirrored as 18rem × 12rem; no border (React has none).
Filtering needs JS, so the input is the same native `<input>` rendered `disabled` (undimmed).

| slot | React | Cronus (before → after) |
|---|---|---|
| command | 24,24 288×192, border 0, text "Command menu Calendar Search" | 432×149 border 1 → identical |
| command-input-wrapper | 24,24 288×41, border-bottom 1px | missing → identical |
| command-input | 60,24 240×40, 14px/20px | 25,25 430×40 21px → identical |
| command-list | 24,65 288×72 | 430×107 → identical |
| command-item #0 / #1 | 28,69 / 28,101 280×32, 14px/20px, #0 bg rgb(30,30,33) | "Type a command…" leaked as item → identical |

### menubar

Wave 1t geometry parity (2026-09-14). React fixture is `<Menubar defaultValue="file">` with one menu:
`div[data-slot="menubar"][role="menubar"]` > the label as the single
`button[data-slot="menubar-trigger"][role="menuitem"][data-state="open"][popovertarget]` + native
`popover="auto"` `div[data-slot="menubar-content"][role="menu"]` whose `menubar-item`s are the extra
texts. Before, every text became a trigger and the label leaked as an item. Trigger: padding .25rem
.75rem, 0.875rem/1.25rem 500, radius-md, open/hover surface-overlay. Chrome fix: an unclosed
`menubar-item:hover {` swallowed the `context-menu` trigger rule (brace restored; the context-menu rule is
unchanged). Limitation: Radix portals the open menu out of the canvas; zero-JS keeps it closed until the
trigger is clicked, while the trigger keeps Radix's `data-state="open"` look for the active menu value.

| slot | React | Cronus (before → after) |
|---|---|---|
| menubar | 24,24 432×38, r 10px, border 1px, bg rgb(22,22,25), text "File" | 432×44 "File File New Tab Open" → identical |
| menubar-trigger | 29,29 47.05×28, 14px/20px 500, bg rgb(30,30,33) | y 31.5 h 29 21px transparent → identical |

### calendar

Wave 1t geometry parity (2026-09-14). React fixture: `div[data-slot="calendar"]` around react-day-picker.
Kernel mirrors that tree: root `<div>` (p-3) > months `<div>` (flex, `flex-direction: row` from 40rem like
`sm:flex-row`, gap 1rem) > `<nav>` (previous/next month buttons, absolute, 1.75rem, outline, opacity .6)
+ month `<div>` (flex column gap 1rem) > caption `<div>` (pt-1) `<span role="status">` 0.875rem/1.25rem 500
+ `<table role="grid">` (thead Su–Sa 0.75rem/1rem fg-tertiary; one flex `<tr>` per week, mt .5rem, of
2.25rem day buttons, outside days fg-muted). Real month maths (Sakamoto weekday, leap years): leading and
trailing outside days and only the weeks the month needs (June 2026 = 5 weeks, 31 … 4). Month from
`defaultMonth` / `value` / `Month YYYY` label; deterministic fallback September 2026. Navigation and
selection need JS: all buttons are rendered `disabled` with React's idle look. Background is transparent
(was surface-raised). Divergence: the React harness passes `selected={defaultMonth}` (June 1 chip); the
emitted source carries no selection, so no day is selected (not a measured slot).

| slot | React | Cronus (before → after) |
|---|---|---|
| calendar | 24,24 432×300, bg transparent, text "June 2026 Su Mo … 30 1 2 3 4" | 432×215 bg rgb(21,21,23), Mo-first 1–28 → identical |

### date-picker

Wave 1t geometry parity (2026-09-14). No wrapper slot (React has none):
`button[data-slot="date-picker-trigger"][data-variant="outline"][popovertarget]` (calendar svg +
`<span>` label) + native `popover="auto"` `div[data-slot="date-picker-content"][role="dialog"]`. Value
`YYYY-MM-DD` is formatted like date-fns `PPP` ("June 15th, 2026"); without a value the trigger gets
`data-empty` (fg-tertiary). Trigger = React Button outline md + `w-[240px] justify-start gap-2
font-normal`: transparent background, shadow-xs, padding 0 1rem, 0.875rem/1.25rem 400; the duplicate
second trigger block (padding .75rem, inset background) was removed. Day buttons inside the closed
content are rendered `disabled` (selection needs JS). Limitation: the React fixture opens the popover and
portals it out of the canvas; the kernel keeps it closed until clicked.

| slot | React | Cronus (before → after) |
|---|---|---|
| date-picker-trigger | 24,24 240×40, 14px/20px 400, bg transparent, text "June 15th, 2026" | 21px, bg rgb(14,14,16), "2026-06-15", extra `date-picker` wrapper → identical |

### date-range-picker

Wave 1t geometry parity (2026-09-14). `button[data-slot="date-range-picker-trigger"][data-variant="outline"]
[popovertarget]` + native `popover="auto"` `div[data-slot="date-range-picker-content"][role="dialog"]`
(was always open in the canvas, 76 extra slots). Trigger mirrors React Button outline md +
`w-[300px] justify-start gap-2 font-normal`, `data-empty` → fg-tertiary. `:popover-open` content is a
flex row with border, radius-lg, surface-floating, shadow-lg. Preset and day buttons are rendered
`disabled` (they need JS). React's `open` starts false, so both sides show only the trigger.

| slot | React | Cronus (before → after) |
|---|---|---|
| date-range-picker-trigger | 24,24 300×40, 14px/20px 400, color rgb(133,133,142), r 14px, border 1px | 16px/24px fg r 0 border 0 → identical |
