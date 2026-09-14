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
