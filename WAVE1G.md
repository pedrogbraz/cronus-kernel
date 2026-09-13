# Wave 1g — radar-chart / scatter-chart / ring-chart kernel ports

Dedicated CONTRACT renderers for `radar-chart`, `scatter-chart`, `ring-chart`.
Catalog `chart()` figure stubs are skipped. Zero JS, zero Voodoo. `meteors` not ported.

Remaining chart stub: `sankey-chart`.

`PORTED_FAMILIES` is now 70 (67 after wave 1f + these 3).

## DOM (React slots)

| Family | Contract | Forbidden |
| --- | --- | --- |
| radar-chart | `<div data-slot="radar-chart"><svg>` pentagon `<polygon>` + `<polyline>` (token stroke/fill, dummy series 4,8,6,10,7) | `<figure data-slot="radar-chart"><figcaption>` catalog `chart()` stub; dummy polyline `0,30 20,22…` |
| scatter-chart | `<div data-slot="scatter-chart"><svg>` `<circle>` points (token fill, dummy series 4,8,6,10,7) | `<figure data-slot="scatter-chart"><figcaption>` catalog `chart()` stub |
| ring-chart | `<div data-slot="ring-chart"><svg>` donut track `<circle>` + progress `<path>` arcs (token stroke) | `<figure data-slot="ring-chart"><figcaption>` catalog `chart()` stub |

Radar is a pentagon (5 axes). Scatter plots circles on the shared 200×100 chart viewBox. Ring is concentric donut arcs.

## Wiring

- modules: `src/cronus_ui_radar_chart.rs`, `src/cronus_ui_scatter_chart.rs`, `src/cronus_ui_ring_chart.rs`
- kit helpers in `src/cronus_ui_kit.rs` (`radar_*`, `chart_rings`)
- `mod` in `src/main.rs`
- `PORTED_FAMILIES` + `dedicated_render` in `src/cronus_ui_widgets.rs` and `scripts/gen_cronus_ui_widgets.py`
- `dedicated_fn_name` + interact fingerprints + sidecar file list in `src/cli/stub_renderer_gate.rs`
- token-only `COMPONENT_CHROME` in `src/cronus_ui.rs` (`var(--cronus-*)`, no zinc palette)

## Tests retargeted

- `stub_renderer_gate::radar_chart_is_stub` → `sankey_chart_is_stub` (family `sankey-chart`)
- `cronus_audit::all_fails_stub_chart` → `style:sankey-chart`
- `test_stub("radar-chart")` expecting `<figure` → `sankey-chart`

## Evidence

```
cargo test cronus_ui_radar_chart
# 6 passed (div+svg pentagon polygon/polyline, not figure/figcaption)

cargo test cronus_ui_scatter_chart
# 6 passed (div+svg circle points, token fill, not figure stub)

cargo test cronus_ui_ring_chart
# 6 passed (div+svg donut arcs, token stroke, not figure stub)

cargo test stub_renderer_gate
# 7 passed (Dedicated, skip interact, no sidecar assets; sankey-chart remains stub)

cargo test cronus_audit
# 6 passed (all_fails_stub_chart uses style:sankey-chart)
```

Commits on `feat/wave1g-radar-scatter-ring` (one per family + test retarget):

1. `feat(ui): dedicated RadarChart renderer`
2. `feat(ui): dedicated ScatterChart renderer`
3. `feat(ui): dedicated RingChart renderer`
4. `fix(test): remaining chart stub is sankey-chart, not radar`
