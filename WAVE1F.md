# Wave 1f — 9 dedicated kernel ports

area-chart, bar-chart, line-chart, sparkline, pie-chart, data-table, sidebar, sonner, navigation-menu.

### sonner
Wave 1t geometry parity. React `<Toaster />` idle = `<div data-slot="toaster">` around Sonner's empty
live region `<section aria-label="Notifications alt+T" tabindex="-1" aria-live="polite"
aria-relevant="additions text" aria-atomic="false">`. No `sonner` slot and no toast until `toast()`.
Kernel: same empty toaster + section (label = `aria-label` prop/config, else label). CSS:
`[data-slot="toaster"] { display: block; }`. The fixed `[data-slot="sonner"]` rule and the sample
toast were removed.

| slot | React | Cronus |
|---|---|---|
| toaster | 24,24 432×0, text "" | identical |

0 mismatches. Divergences: queued toasts need `toast()` + timers (JS), so none are faked; the
region name has no Sonner hotkey suffix (" alt+T" needs a key listener).
`cronus_ui_widgets` slot test accepts `toaster` for `sonner`.
