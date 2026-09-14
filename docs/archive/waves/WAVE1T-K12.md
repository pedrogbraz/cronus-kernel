# Wave 1t — K12: overlay geometry parity

Families: alert-dialog, confirmation-dialog, invite-dialog.
Measured with the overlay logic of `e2e/audit/geometry.spec.ts` (ui-1t), copied verbatim:
- React: `http://localhost:4747/audit/{family}?fixture=default&preset=aurora&mode=dark` in a 640x777 context.
- Kernel: `http://127.0.0.1:5192/audit/{family}/default?preset=aurora&mode=dark&dir=ltr` in a 640x777 context.
- Overlay roots are measured from the viewport origin on both sides.
- Tolerances are unchanged. Nothing is allowlisted.

## Shared model

All three dialogs are open by default and ship zero JS.

- A non-modal native `<dialog open>` wraps them. It has no data-slot, and chrome gives it `display: contents; color: inherit`, so it paints no box. The chrome selector is `dialog:has(> [data-slot=…])`.
- The overlay scrim is `position: fixed; inset: 0; z-index: 50`, with black at 50% and `backdrop-filter: blur(8px)`.
- The panel is `position: fixed; inset: 0; margin: auto; height: fit-content`, with `max-width` 32rem (alert) or 28rem (confirmation, invite). It has `p-6`, `gap-4`, a 1px border and `radius-xl`.
- Buttons submit a `<form method="dialog">`, so Confirm, Cancel, Send and Close really close the dialog without script.
- Not reproduced, because it is JS-only: Radix focus trap and auto-focus (React shows a focus ring on the first control), modal inertness, Esc while modal, and async spinner/error states.
- The gate (`stub_renderer_gate.rs`) now flags `<dialog data-slot=` (the interact `showModal` version) instead of any `<dialog`.

### alert-dialog

DOM:

```html
<dialog open role="alertdialog" aria-labelledby="…-title">
  <form method="dialog" id="…-form" hidden></form>
  <div data-slot="alert-dialog-overlay" data-state="open" aria-hidden="true"></div>
  <div data-slot="alert-dialog-content" data-state="open">
    <h2 data-slot="alert-dialog-title">Delete account</h2>
    <button type="submit" form="…-form" value="action" data-slot="alert-dialog-action">Confirm</button>
  </div>
</dialog>
```

- Removed: the `alert-dialog` wrapper, the trigger, the implicit description and the default cancel.
- Action label: the `action` item, else the first `text` item (the fixture's `items[0]`), else "Confirm".
- Optional description: `description:"…"` prop, item config, or a `description` item.
- Optional cancel: a `cancel` item.
- Title selector needs two attribute selectors to beat the base `html[data-cronus-theme] h2 { font-weight: 400 }`.

| slot | React = Cronus |
|---|---|
| alert-dialog-overlay | div 0,0 640x777 |
| alert-dialog-content | div 64,321.5 512x134, radius 18px, border 1px |
| alert-dialog-title | h2 89,346.5 462x28, 18px/600/28px |
| alert-dialog-action | button 89,390.5 462x40, 14px/500/20px, radius 14px, border 0 |

Result: **0 mismatches**.

### confirmation-dialog

DOM:

```html
<dialog open role="alertdialog">
  hidden form
  <div data-slot="alert-dialog-overlay"></div>
  <div data-slot="confirmation-dialog">
    <div data-slot="alert-dialog-header">
      <h2 data-slot="alert-dialog-title">Delete project</h2>
      [<p data-slot="alert-dialog-description">]
    </div>
    <div data-slot="alert-dialog-footer">
      <button data-slot="alert-dialog-cancel">Cancel</button>
      <button data-slot="confirmation-dialog-confirm">Confirm</button>
    </div>
  </div>
</dialog>
```

- The header and footer use React's breakpoint layout: centred header and `column-reverse` footer below 640px, row and `justify-end` at 640px and up.
- Cancel is outline style. Confirm is primary with `min-width: 6rem`.

| slot | React = Cronus |
|---|---|
| alert-dialog-overlay | div 0,0 640x777 |
| confirmation-dialog | div 96,321.5 448x134, border 1px |
| alert-dialog-header | div 121,346.5 398x28 |
| alert-dialog-title | h2 121,346.5 398x28 |
| alert-dialog-footer | div 121,390.5 398x40 |
| alert-dialog-cancel | button 335.64,390.5 79.36x40, border 1px |
| confirmation-dialog-confirm | button 423,390.5 96x40, bg rgb(0,166,244), fg rgb(10,10,12), radius 14px, border 0 |

Result: **0 mismatches**.

### invite-dialog

DOM:

```html
<dialog open>
  <div data-slot="dialog-overlay"></div>
  <div data-slot="invite-dialog">
    <div data-slot="dialog-header">
      <h2 data-slot="dialog-title">Invite member</h2>
      <p data-slot="dialog-description">Send an invitation to join this workspace.</p>
    </div>
    <form method="dialog">
      <div data-slot="field">
        <label data-slot="field-label">Email</label>
        <input data-slot="input" type="email" required>
      </div>
      <div data-slot="field">
        <label data-slot="field-label">Role</label>
        <button data-slot="select-trigger" disabled><span>Member</span><svg/></button>
        <select aria-hidden tabindex="-1" name="role">Member / Admin</select>
      </div>
      <div data-slot="dialog-footer">
        <button data-slot="button" data-variant="outline" formnovalidate>Cancel</button>
        <button data-slot="invite-dialog-send" type="submit">Send invite</button>
      </div>
    </form>
    <button data-slot="dialog-close" formnovalidate><svg/><span>Close</span></button>
  </div>
</dialog>
```

- The role picker (Radix Select) needs JS. Following the Wave 1t rule, the trigger is the same native `button` with `disabled` and keeps the idle look.
- The visually hidden native `<select>` mirrors the one Radix renders. It carries the form value and matches React's field innerText ("Role Member Member Admin").
- Send validates the email and closes the dialog. Cancel and Close use `formnovalidate`.
- The close label is an sr-only span.
- All chrome is scoped under `[data-slot="invite-dialog"]`, so the plain `dialog` family's rules are not touched.
- Description: `description:"…"` prop, item config, or a `description` item, else the React default.

| slot | React = Cronus |
|---|---|
| dialog-overlay | div 0,0 640x777 |
| invite-dialog | div 96,232.5 448x312, bg surface-floating, radius 18px, border 1px |
| dialog-header | div 121,257.5 398x54 |
| dialog-title | h2 121,257.5 398x28 |
| dialog-description | p 121,291.5 398x20, 14px/20px |
| field #0 / #1 | div 121,327.5 / 121,403.5 398x60 |
| field-label #0 / #1 | label 398x14 |
| input | input 121,347.5 398x40 |
| select-trigger | button 121,423.5 398x40 |
| dialog-footer | div 121,479.5 398x40 |
| button (Cancel) | button 326.53,479.5 79.36x40 |
| invite-dialog-send | button 413.89,479.5 105.11x40 |
| dialog-close | button 511,249.5 16x16 |

Result: **0 mismatches**.

## Notes

- Emitter: nothing is needed for these fixtures.
  - React's invite-dialog fixture ignores `label` (the title is the component default "Invite member", which is the same text).
  - Once the emitter writes `description:"…"`, all three renderers read it.
- Pre-existing, not K12: in `COMPONENT_CHROME`, the `[popover]` `@supports (top: anchor(bottom)) {` and `@supports not (…) {` blocks close immediately on the next line, leaving stray `}`s. The braces balance, but the nesting is wrong.
