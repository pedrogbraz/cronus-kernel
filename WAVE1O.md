# Wave 1o — 3 dedicated kernel ports

gauge-chart, funnel-chart, candlestick-chart.
sankey-chart and meteors remain stubs. Not ported. aspect-ratio and countdown untouched.

Geometry lives in `src/cronus_ui_kit.rs` (same pattern as ring-chart / bar-chart). Dedicated modules, not catalog `chart()` `<figure><figcaption>` + dummy polyline `0,30 20,22 40,26 60,12 80,16 100,8 120,14`. No voodoo.

## DOM

- **gauge-chart** — `<div data-slot="gauge-chart">` + SVG horseshoe track/fill arcs. Token stroke (`--cronus-border` / `--cronus-primary`). Static value 60.
- **funnel-chart** — `<div data-slot="funnel-chart">` + SVG trapezoids shrinking (fallback 10,8,6,4,2). Token fills.
- **candlestick-chart** — `<div data-slot="candlestick-chart">` + SVG 5 candles (rect + wick). Success/error tokens.

## Evidence

```
$ cargo test cronus_ui_gauge_chart
test cronus_ui_gauge_chart::tests::chrome_is_token_only ... ok
test cronus_ui_gauge_chart::tests::default_value_is_sixty ... ok
test cronus_ui_gauge_chart::tests::no_voodoo_even_when_runtime_on ... ok
test cronus_ui_gauge_chart::tests::numeric_items_drive_value ... ok
test cronus_ui_gauge_chart::tests::root_is_div_with_svg_arc_not_figure ... ok
test cronus_ui_gauge_chart::tests::skips_chart_figure_stub ... ok
test result: ok. 6 passed; 0 failed

$ cargo test cronus_ui_funnel_chart
test cronus_ui_funnel_chart::tests::chrome_is_token_only ... ok
test cronus_ui_funnel_chart::tests::comma_list_item_is_series ... ok
test cronus_ui_funnel_chart::tests::default_series_shrinks ... ok
test cronus_ui_funnel_chart::tests::no_voodoo_even_when_runtime_on ... ok
test cronus_ui_funnel_chart::tests::numeric_items_drive_series ... ok
test cronus_ui_funnel_chart::tests::root_is_div_with_svg_trapezoids_not_figure ... ok
test cronus_ui_funnel_chart::tests::skips_chart_figure_stub ... ok
test result: ok. 7 passed; 0 failed

$ cargo test cronus_ui_candlestick_chart
test cronus_ui_candlestick_chart::tests::chrome_is_token_only ... ok
test cronus_ui_candlestick_chart::tests::comma_list_item_is_series ... ok
test cronus_ui_candlestick_chart::tests::default_series_when_only_label ... ok
test cronus_ui_candlestick_chart::tests::no_voodoo_even_when_runtime_on ... ok
test cronus_ui_candlestick_chart::tests::numeric_items_drive_series ... ok
test cronus_ui_candlestick_chart::tests::root_is_div_with_svg_candles_not_figure ... ok
test cronus_ui_candlestick_chart::tests::skips_chart_figure_stub ... ok
test result: ok. 7 passed; 0 failed

$ cargo test stub_renderer_gate
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
142 PORTED_FAMILIES.
