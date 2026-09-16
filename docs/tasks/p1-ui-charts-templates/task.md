# p1-ui-charts-templates

Risco: normal

## Pedido

Portar os dois stubs (`meteors`, `sankey-chart`) para renderers dedicados zero-JS, fazer as famílias de chart lerem `cronus_ui_data::rows()` (bind), e atualizar templates: landing `scope:public`, saas `on click` delete, `cronus-ui.cronus` com metric/bar-chart/data-table.

Escopo: novos `cronus_ui_meteors.rs` / `cronus_ui_sankey_chart.rs`, registry, CSS+MANIFEST, `values_for`/`categories_or`, três templates, testes de stub/gate, CHANGELOG/AGENTS. Sem crates. Sem Playwright.

## Análise

Fatos:
- `FAMILY_TABLE` ainda tem `Renderer::Stub` só em meteors (fx) e sankey-chart (chart). Dezenas de testes usam `test_stub("meteors")` como o genérico inline-style.
- `light-rays` / `particles` são o modelo CSS+divs. Charts cartesianos já têm SVG em `cronus_ui_chart`.
- `data-table` já lê `cronus_ui_data::rows()`. `values_for` só olha `numeric_items(comp)`.
- `landing.cronus` form bind sem `scope:public` → submit 401. `saas.cronus` não tem `on click`. `cronus-ui.cronus` usa kpi/table canônicos, não families.

Decisões:
- Meteors: DOM como particles (root + aria-hidden streaks + label). CSS `@keyframes`, zero `style=`.
- Sankey: `data-slot="sankey-chart"` wrapping chart SVG; fluxos de items (`source`/`target`/`value`) ou rows bound.
- Charts: `bound_series()` em `cronus_ui_chart` alimenta categories/values.
- Templates: mudanças mínimas e válidas em `cronus build --ai` (parse_all já cobre templates).

## Plano

1. `cronus_ui_meteors.rs` + CSS + `dedicated!` + `mod` + FILES.
2. `cronus_ui_sankey_chart.rs` + CSS + registry.
3. Atualizar testes que exigiam Stub/FX_BOX em meteors/sankey.
4. `categories_or` / `values_for` leem rows bound; teste with_binding.
5. Templates + CHANGELOG + AGENTS (zero stubs).
6. `cargo test cronus_ui_css` para a linha `new` do MANIFEST.
7. harness quick então full.

## Revisão

Não desambigua `section tabs` vs `style:tabs`. Não preenche 39 contratos. Adequado para a fatia P1 UI prometida.

## Validação

`family_table_derives_lists` stubs vazio; output gate; `every_template_and_demo_parses`; bind test no chart.

## Ajustes

Nenhum ainda.

## Entrega

Critérios de aceitação:
- FAMILY_TABLE has zero stubs; meteors and sankey-chart are Dedicated and pass the output gate
- bar-chart/line-chart/area-chart/pie-chart use bound rows for categories and values when bind data is present
- landing form bind is scope:public; saas ships an on click delete; cronus-ui.cronus uses metric, bar-chart and data-table sections
