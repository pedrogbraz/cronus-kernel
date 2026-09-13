# Wave 1f — sparkline / pie-chart / data-table kernel ports

Dedicated CONTRACT renderers for `sparkline`, `pie-chart`, `data-table`.
Catalog `chart()` figure stubs and interact `table("data-table")` are skipped.
Zero JS, zero Voodoo. `area-chart` and `sidebar` were not touched.

`PORTED_FAMILIES` is now 61 (58 after wave 1e merge + these 3).

## DOM (React slots)

| Family | Contract | Forbidden |
| --- | --- | --- |
| sparkline | `<svg data-slot="sparkline">` + `<path data-slot="sparkline-line">` (line type). Optional `<path data-slot="sparkline-area">`. | `<figure data-slot="sparkline"><figcaption>` catalog `chart()` stub |
| pie-chart | `<div data-slot="pie-chart"><svg>` with 3–4 `<path>` slices (fills `var(--cronus-primary\|success\|warning\|info)`) | `<figure data-slot="pie-chart"><figcaption>` catalog `chart()` stub |
| data-table | `<div data-slot="data-table"><table><thead><tbody>` — first text items as headers, remaining as cells (or 2-col Name/Value dummy). Token CSS, no inline SURF on `th`. | interact `table("data-table")` `th` with `style="text-align:left;padding:0.5rem 0.75rem;..."` |

Sparkline is an SVG root, not a figure. Pie-chart is a div wrapping SVG slices. Data-table never emits inline `style=` on `th`.

## Wiring

- modules: `src/cronus_ui_sparkline.rs`, `src/cronus_ui_pie_chart.rs`, `src/cronus_ui_data_table.rs`
- `mod` in `src/main.rs`
- `PORTED_FAMILIES` + `dedicated_render` in `src/cronus_ui_widgets.rs` and `scripts/gen_cronus_ui_widgets.py`
- `dedicated_fn_name` + interact fingerprints + sidecar file list in `src/cli/stub_renderer_gate.rs`
- token-only `COMPONENT_CHROME` in `src/cronus_ui.rs` (`var(--cronus-*)`, no zinc palette)

## Evidence

```
cargo test cronus_ui_sparkline
# 8 passed (svg + sparkline-line, not figure/figcaption)

cargo test cronus_ui_pie_chart
# 5 passed (div+svg slices, token fills, not figure stub)

cargo test cronus_ui_data_table
# 7 passed (<table and data-slot=data-table, no inline th SURF)

cargo test stub_renderer_gate
# 7 passed (Dedicated, skip interact, no sidecar assets; area-chart remains stub)
```

Commits on `feat/wave1f-spark-pie-table` (one per family):

1. `feat(ui): dedicated Sparkline renderer`
2. `feat(ui): dedicated PieChart renderer`
3. `feat(ui): dedicated DataTable renderer`
