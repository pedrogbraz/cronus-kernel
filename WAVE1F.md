# Wave 1f — area / bar / line kernel ports

Dedicated zero-JS renderers for `area-chart`, `bar-chart`, `line-chart`.
React root is `<div data-slot="{family}">` wrapping visx SVG — not the stub
`chart()` `<figure data-slot><figcaption>` + dummy polyline
`points="0,30 20,22 40,26 60,12 80,16 100,8 120,14"` or flex bar `height` px.

Branch: `feat/wave1f-area-bar-line`  
Base: `4630a19`

## Commits (one per family)

| Family | Commit | Renderer |
|---|---|---|
| area-chart | `e60821f` | `src/cronus_ui_area_chart.rs` |
| bar-chart | `71cac36` | `src/cronus_ui_bar_chart.rs` |
| line-chart | `0248702` | `src/cronus_ui_line_chart.rs` |

## DOM contract

Root is `<div data-slot="{family}" role="img">` with inner `<svg viewBox="0 0 200 100">`.

| Family | Series |
|---|---|
| area-chart | filled `<path>` + stroked `<polyline>` |
| bar-chart | `<rect>` per value |
| line-chart | stroked `<polyline>` |

Stroke/fill: `var(--cronus-primary)`. Token CSS lives in `COMPONENT_CHROME`.
No inline `BASE`/`SURF`/`CTRL`, no `<script>`, no voodoo attrs.

Values: numeric item text (or comma/whitespace lists). Fallback series `4,8,6,10,7`.

## Wiring

`PORTED_FAMILIES` appends `"area-chart", "bar-chart", "line-chart"`.

Also: `mod` in `src/main.rs`, `dedicated_render`, `dedicated_fn_name`,
`scripts/gen_cronus_ui_widgets.py`, `COMPONENT_CHROME`, stub-gate file list.

Shared series/plot helpers: `src/cronus_ui_kit.rs`.

Stub `chart()` remains for unported families (`pie-chart`, sparkline, …).
`all_fails_stub_chart` now uses `style:pie-chart`. Sparkline / sidebar / sonner untouched.

## Tests

```
cargo test cronus_ui_area_chart
cargo test cronus_ui_bar_chart
cargo test cronus_ui_line_chart
cargo test stub_renderer_gate
```

| Filter | Result |
|---|---|
| `cronus_ui_area_chart` | 6 passed |
| `cronus_ui_bar_chart` | 6 passed |
| `cronus_ui_line_chart` | 6 passed |
| `stub_renderer_gate` | 7 passed |

Family tests assert: no `<figure`, no `figcaption`, no stub polyline string,
no `style=`, no JS/voodoo. Stub gate: dedicated kind, no stub fingerprint,
no sidecar assets.
