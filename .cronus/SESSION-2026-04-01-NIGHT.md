# Session 2026-04-01 Night — DB-Driven Pages + Animations + Modals + Responsive

## Resumo
Continuação da sessão anterior. Backend + DB conectados na app 5175, animações em todos os componentes, modais glassmorphic na linguagem, responsivo mobile, gradient backgrounds.

---

## Bugs fixados nesta sessão

### 1. Parser keyword bug (parser.rs:841)
Campos de entity cujo nome é um CRONUS keyword (ex: "page", "style", "service") eram silenciosamente pulados. A entity `KpiSnapshot` perdia o campo `page`.
**Fix:** entity loop aceita `TokenKind::Keyword` além de `Identifier`.

### 2. Binding table name mismatch (binding.rs:84)
`binding.rs` fazia `to_lowercase() + "s"` (pluralização) mas migrate cria tabelas com nome exato.
**Fix:** usa `binding.entity.clone()` sem pluralizar.

### 3. MinItemsViolation warning (contracts.rs:466)
Warning falso pra sections sem items estáticos mas com binding.
**Fix:** skip quando `section.binding.is_some()`.

### 4. Sidebar navigation na home (app.cronus)
Home page tinha sidebar com `<div>` sem links. Sub-pages usavam `NovaCoreChrome` com `<a href>`.
**Fix:** home agora usa `use NovaCoreChrome` — mesma sidebar em todas as pages.

### 5. FOUC (Flash of Unstyled Content)
Topbar/sidebar apareciam unstyled por 1s antes do Tailwind CDN carregar.
**Fix:** CSS crítico inline no `style_block` do NovaCoreChrome (`header`, `aside`, `#cronus-main` com position:fixed).

---

## Features implementadas

### 1. DB-Driven Renderers
- `render_kpi_dashboard_dark` — KPI cards de DB rows
- `render_kpi_section` (light) — idem
- `render_stat_cards` — theme-aware (dark/light), DB rows com badge/icon
- `render_data_table_dark` — table rows do DB com status badges animados

### 2. Nova Core Full-Stack (porta 5175)
- 4 entities: Deployment, SecurityEvent, Endpoint, KpiSnapshot
- 27 rows seedados via API
- 3 sub-pages 100% DB-driven: /deployments, /analytics, /security
- Home (/) mantém visual dump original
- Todas usam `use NovaCoreChrome` — sidebar unificada

### 3. Animações
- **11 novas keyframes:** barGrow, drawLine, donutDraw, glowPulse, shimmer, countUp, rowSlide, gradientShift, breathe
- **Counter animation:** números contam de 0 ao valor real (suporta "847", "99.7%", "42s", "12,847", "$142,804")
- **Chart bars:** crescem de baixo pra cima com stagger
- **Line chart:** SVG draw animation
- **Donut chart:** segmentos se desenham sequencialmente
- **Area chart (analytics):** 24 barras com grow stagger + breathing light
- **Table rows:** cascade slide-in
- **Status badges:** dot pulsante em Live/Rolling/Processing + cores por severity

### 4. Modais Glassmorphic (`section modal`)
- Glass panel: `backdrop-filter:blur(40px)` + gradient overlay
- Edge lighting: `box-shadow: inset 0 0.5px 0 rgba(135,173,255,0.2)`
- Header: título + subtitle + ícone badge
- Fields: text, select, textarea, checkbox com focus glow
- Summary rows: label + value com cor customizável
- Actions: Cancel (secondary) + Primary (gradient + glow)
- Gradient bar decorativo na base
- Theme-aware: dark (obsidian) e light
- Trigger: `onclick="cronusModal.open('id')"` ou `trigger:auto`

Sintaxe:
```cronus
section modal id:"new-deployment" icon:"rocket_launch" {
  title "New Deployment"
  subtitle "Configure and launch."
  field "Service" type:text placeholder:"api-gateway" required:true
  field "Cluster" type:select options:"us-east-1||eu-west-2"
  summary "Build Time" value:"~45s"
  action "Cancel" style:secondary
  action "Deploy" style:primary
}
```

### 5. Gradient Backgrounds (`glow-1/2/3`)
- 3 radial gradient orbs no `body::before` (fixed, pointer-events:none)
- CSS vars: `--cronus-glow-1`, `--cronus-glow-2`, `--cronus-glow-3`
- Auto-computed da cor accent, ou customizável no `style` block
- `hex_to_rgb()` helper pra converter accent hex → rgba

Sintaxe:
```cronus
style {
  theme dark
  accent #87adff
  glow-1 "rgba(135,173,255,0.08)"
  glow-2 "rgba(210,119,255,0.05)"
  glow-3 "rgba(129,236,255,0.04)"
}
```

### 6. Responsivo Mobile
- `@media (max-width: 768px)` — sidebar esconde, bottom nav aparece
- Bottom nav auto-gerada via JS (extrai links da sidebar)
- KPIs: 4 cols → 2 cols no mobile
- Tables: scroll horizontal
- Topbar: nav links escondem
- Content: padding adapta
- JS resize handler pra margin-left dinâmico

---

## Arquivos modificados

| Arquivo | Mudanças |
|---|---|
| `src/parser.rs` | Entity field aceita Keyword como nome |
| `src/binding.rs` | Table name sem pluralização |
| `src/contracts.rs` | Skip MinItems com binding |
| `src/ui.rs` | KPI/stat-card DB-driven, counter animation, gradient chart, modal glassmorphic, responsive CSS, bottom nav, glow vars, hex_to_rgb |
| `src/data_table.rs` | Dark table bound_data, row cascade anim, status badges com dots |
| `/tmp/nova-core/app.cronus` | Entities + APIs + bind + modal + glow style + NovaCoreChrome unificado |

## Apps rodando

| Porta | App | Status |
|---|---|---|
| 5175 | Nova Core (fullstack + dump) | 4 entities, 27 rows, 4 pages, modal |

## Próximos passos
1. Mover /tmp/nova-core pra path permanente no repo
2. Mais modais (edit deployment, confirm delete)
3. SSE real-time updates nas tables
4. `cronus dump` absorver modais de templates HTML
5. Deploy story (static export)
