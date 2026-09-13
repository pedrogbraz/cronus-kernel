# Wave 1j — 3 dedicated kernel ports

expandable-tabs, live-line-chart, sunburst-chart.
97 PORTED_FAMILIES. meteors and sankey-chart remain stubs. heatmap / segmented-control untouched.

## Evidence

```
cargo test cronus_ui_expandable_tabs   # 7 passed
cargo test cronus_ui_live_line_chart   # 7 passed
cargo test cronus_ui_sunburst_chart    # 7 passed
cargo test stub_renderer_gate          # 7 passed (sankey_chart_is_stub = Stub("chart"))
```

- expandable-tabs: `div[data-slot=expandable-tabs]` + `expandable-tabs-item` buttons from texts; first current. Skips interact `tabs()` generic tablist (no nested tablist, no tabpanel).
- live-line-chart: `div[data-slot=live-line-chart]` + SVG polyline from `numeric_series` (fallback 4,8,6,10,7). Static snapshot, no JS ticker. Not stub `points="0,30 20,22…"`. Not `chart()` figure/figcaption.
- sunburst-chart: `div[data-slot=sunburst-chart]` + SVG annulus arcs/rings with token fills. Not `chart()` figure stub.
