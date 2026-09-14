# Wave 1t (K6) — geometry parity: terminal, rich-text-editor, carousel, video-player, masonry, card, alert, banner

Gate: `cooud-ui/e2e/audit/geometry.spec.ts` rules (rect ≤1px vs canvas, font size/weight/line-height,
mono/sans, color/background ≤2/255, used radius, border widths, innerText), measured with a
standalone Playwright copy of the spec against `cronus run --audit-canvas` (aurora / dark, 1280x900).

Zero-JS policy used below: a control whose behaviour needs script (clipboard copy, Tiptap commands,
Embla navigation, media playback) is rendered as the same native element **`disabled`** — inert and
announced as unavailable, never a fake working control — at React's idle geometry. `data-disabled`
marks the controls React itself disables at idle (Undo/Redo, carousel previous), and only those dim.

### terminal

DOM: `terminal[data-state=static]` > `terminal-chrome` (3 dots, `terminal-title`, `copy-button` disabled)
+ `div` > `terminal-body` (`pre.sr-only terminal-transcript`, `div[aria-hidden]` > `terminal-sizer`
(visibility hidden) + `terminal-screen` (absolute inset 0)). Lines alternate input (`$` prompt) /
output (fg-secondary), the audit fixture's mapping for string items. Label = title (no longer a line).
CSS: root 18rem (fixture `w-72`), border + radius-xl; chrome py .625rem px 1rem; title 12/16 500;
copy button 2rem, margin-right -.375rem; body mono 14px / 1.625. Caret keyframes removed (static state).

| slot | React | Cronus |
|---|---|---|
| terminal | 24,24 288x132.5 | 24,24 288x132.5 |
| terminal-title | 91,43 170x16 | 91,43 170x16 |
| terminal-line#2 (screen) | 41,94 254x22.75 | 41,94 254x22.75 |

### rich-text-editor

DOM: toolbar with 14 icon `toggle` buttons (lucide SVGs, disabled) + 3 vertical `separator`s;
`rich-text-editor-content-wrapper` > `p rich-text-editor-placeholder` + `div > div[role=textbox][aria-readonly]`
with an empty paragraph. CSS: toggles 2rem, radius-lg; placeholder absolute start 1rem top .75rem;
textbox min-height 10rem, padding .75rem 1rem.

| slot | React | Cronus |
|---|---|---|
| rich-text-editor | 24,24 432x241 | 24,24 432x241 |
| toggle#11 (wrapped) | 31,65 32x32 | 31,65 32x32 |
| rich-text-editor-placeholder | 41,116 131.44x24 | 41,116 131.44x24 |

### carousel

DOM: `carousel[aria-label]` > `carousel-content` > `carousel-item` > card `div`; then `div` with
icon-only `carousel-previous` / `carousel-next` (both disabled; previous `data-disabled`).
Slides = text items only (label no longer becomes a slide). CSS: root 18rem; content margin-left -1rem;
item padding-left 1rem, basis 100%; card p 1.5rem radius-lg; button row mt .75rem gap .5rem; buttons 2.25rem.

| slot | React | Cronus |
|---|---|---|
| carousel | 24,24 288x118 | 24,24 288x118 |
| carousel-item#1 | 312,24 304x70 | 312,24 304x70 |
| carousel-next | 172,106 36x36 | 172,106 36x36 |

### video-player

DOM: `video-player[aria-label]` > `video-player-video` (no src) + `video-player-overlay-play` +
`video-player-controls` (play, `video-player-time` "0:00 / 0:00", seek range, rate "1x", mute, volume range,
fullscreen) — all controls disabled. CSS: border + radius-xl, 16/9; overlay 3.5rem circle; controls
px .625rem py .5rem gap .375rem; icon buttons 2rem radius-md; ranges .25rem track.

| slot | React | Cronus |
|---|---|---|
| video-player | 24,24 432x243 | 24,24 432x243 |
| video-player-seek | 149.91,240 111.09x4 | 149.91,240 111.09x4 |
| video-player-fullscreen | 413,226 32x32 | 413,226 32x32 |

### masonry

DOM: `masonry[aria-label]` with plain `div` children (React has no `masonry-cell` slot; the slot was
removed and the stub-gate fingerprint now keys on `style=`). CSS: 18rem, `column-count: 2`
(harness `columns={2}`), gap 1rem; cards p .75rem, 14/20, radius-lg, border.

| slot | React | Cronus |
|---|---|---|
| masonry | 24,24 288x124 | 24,24 288x124 |

### card

Description now read from a `description:"…"` attribute (prop or item config), else first extra text.
CSS: title font-display 600 line-height 1; description 14/20; header/content px 1rem below 640px.

| slot | React | Cronus (current emitter) | Cronus (with `description:`) |
|---|---|---|---|
| card | 24,24 432x92 | 24,24 432x66 | 24,24 432x92 |
| card-description | 49,71 382x20 | — | 49,71 382x20 |

### alert

Description from `description:"…"` attribute first, then extra texts. CSS: grid `0 1fr`, row-gap .25rem,
line-height 1.25rem; title col 2, min-height 1rem, 1-line clamp; description col 2, grid gap .25rem.
Destructive wash now `error 10%` over transparent (React `bg-error/10`).

| slot | React | Cronus (current emitter) | Cronus (with `description:`) |
|---|---|---|---|
| alert | 24,24 432x70 | 24,24 432x46 | 24,24 432x70 |
| alert-description | 41,61 398x20 | — | 41,61 398x20 |

### banner

Description from `description:"…"` attribute too. CSS: line-height 1.25rem, `text-align: center`
(React default `align="center"`), gaps `.25rem .75rem` / `.125rem .5rem`.

| slot | React | Cronus |
|---|---|---|
| banner | 24,24 432x41 (lh 20px) | 24,24 432x41 (lh 20px) |

### Emitter gap (cooud-ui, not editable here)

`packages/audit/src/emit-cronus-fixture.ts` never emits `description`, but the React harness renders it
for alert, card and banner. Proposed, after the `language` line:

```ts
if (typeof fixture.props.description === "string") {
  lines.push(`  description:"${cronusEscape(fixture.props.description)}"`);
}
```
