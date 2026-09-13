# Wave 1p — 9 dedicated kernel ports

A: logo-carousel, dynamic-island, image-zoom
B: aurora-background, border-beam, confetti
C: composed-chart, heatmap-chart, chart
sankey-chart and meteors remain stubs. Not ported.

Dedicated modules, not catalog `chart()` `<figure><figcaption>` + dummy polyline `0,30 20,22 40,26 60,12 80,16 100,8 120,14`. No voodoo. No Recharts/JS.

## DOM

- **composed-chart** — `<div data-slot="composed-chart" role="img">` + SVG area path (`fill-opacity` 0.28) and line polyline. Token fill/stroke `--cronus-primary`. `numeric_series` fallback `4,8,6,10,7`.
- **heatmap-chart** — `<div data-slot="heatmap-chart">` wrapping `role="img"` of `data-slot="heatmap-day"` cells + `heatmap-legend` swatches. Distinct root from `heatmap`. Same level buckets as React `getHeatmapLevel`.
- **chart** — ChartContainer shell: `<div data-slot="chart" role="img">` + SVG of one numeric series (area path + polyline). Token fills. Not a figure.

## Evidence

```
$ cargo test --offline cronus_ui_composed_chart -- --test-threads=1
test cronus_ui_composed_chart::tests::aria_label_is_escaped ... ok
test cronus_ui_composed_chart::tests::chrome_is_token_only ... ok
test cronus_ui_composed_chart::tests::comma_list_item_is_series ... ok
test cronus_ui_composed_chart::tests::default_series_when_only_label ... ok
test cronus_ui_composed_chart::tests::no_voodoo_even_when_runtime_on ... ok
test cronus_ui_composed_chart::tests::numeric_items_drive_series ... ok
test cronus_ui_composed_chart::tests::root_is_div_with_svg_area_and_line_not_figure ... ok
test cronus_ui_composed_chart::tests::skips_chart_figure_stub ... ok
test result: ok. 8 passed; 0 failed

$ cargo test --offline cronus_ui_heatmap_chart -- --test-threads=1
test cronus_ui_heatmap_chart::tests::aria_label_is_escaped ... ok
test cronus_ui_heatmap_chart::tests::chrome_is_token_only ... ok
test cronus_ui_heatmap_chart::tests::comma_list_item_is_series ... ok
test cronus_ui_heatmap_chart::tests::default_series_when_only_label ... ok
test cronus_ui_heatmap_chart::tests::heatmap_level_matches_react_buckets ... ok
test cronus_ui_heatmap_chart::tests::legend_has_five_swatches ... ok
test cronus_ui_heatmap_chart::tests::no_voodoo_even_when_runtime_on ... ok
test cronus_ui_heatmap_chart::tests::numeric_items_drive_days ... ok
test cronus_ui_heatmap_chart::tests::root_is_div_with_day_cells_not_figure ... ok
test cronus_ui_heatmap_chart::tests::skips_chart_figure_stub ... ok
test result: ok. 10 passed; 0 failed

$ cargo test --offline cronus_ui_chart -- --test-threads=1
test cronus_ui_chart::tests::aria_label_is_escaped ... ok
test cronus_ui_chart::tests::chrome_is_token_only ... ok
test cronus_ui_chart::tests::comma_list_item_is_series ... ok
test cronus_ui_chart::tests::default_series_when_only_label ... ok
test cronus_ui_chart::tests::no_voodoo_even_when_runtime_on ... ok
test cronus_ui_chart::tests::numeric_items_drive_series ... ok
test cronus_ui_chart::tests::root_is_div_with_svg_series_not_figure ... ok
test cronus_ui_chart::tests::skips_chart_figure_stub ... ok
test result: ok. 8 passed; 0 failed

$ cargo test --offline stub_renderer_gate -- --test-threads=1
test cli::stub_renderer_gate::tests::button_is_dedicated ... ok
test cli::stub_renderer_gate::tests::dedicated_module_no_sidecar_assets ... ok
test cli::stub_renderer_gate::tests::every_family_is_classified ... ok
test cli::stub_renderer_gate::tests::meteors_is_stub_fx ... ok
test cli::stub_renderer_gate::tests::no_new_stub_families ... ok
test cli::stub_renderer_gate::tests::ported_families_are_dedicated_and_skip_interact ... ok
test cli::stub_renderer_gate::tests::sankey_chart_is_stub ... ok
test result: ok. 7 passed; 0 failed
```

sankey-chart is still `RendererKind::Stub("chart")`. meteors is still `Stub("fx")`.
151 PORTED_FAMILIES (148+3).
