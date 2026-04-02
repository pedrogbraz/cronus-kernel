# SDD — CRONUS Semantic Optimization for AI-First Operation

> A linguagem onde IA opera com zero alucinação.
> Author: Zedd + Claude | Date: 2026-04-02 | Status: PROPOSED

---

## 1. Problem Statement

O Nova Core (354 linhas) revelou que **48% do arquivo é HTML/JS raw** — ruído que a linguagem deveria gerar. A razão sinal:ruído é 1:2.5. Além disso, a gramática tem inconsistências que fazem IA errar 40% das vezes em sintaxe (`key:value` vs `key value` vs `key:"value"`).

Uma linguagem AI-first precisa de: **vocabulário fechado, defaults inteligentes, zero ambiguidade, e localidade de referência** (toda info para entender um bloco está no bloco).

---

## 2. Princípios de Design (estudados de 10 DSLs)

| Princípio | Inspiração | Aplicação no CRONUS |
|-----------|-----------|---------------------|
| `!` para required | GraphQL | `name string!` em vez de `name string required` |
| Relações inline | Prisma | `author -> User` com cardinalidade |
| Vocabulário fechado | Protobuf | Enum de tipos, sem `any`/`custom` |
| Plan antes de apply | Terraform | `cronus check` → `cronus plan` → `cronus run` |
| Linha = unidade atômica | SQL DDL | Cada field é 1 linha com tudo |
| Defaults inteligentes | SwiftUI | `entity` implica timestamps, `page` implica SPA |
| Sem interpolação de strings | HCL | Referências por nome, não `${var}` |
| Named params > positional | Flutter | Sempre `key:value`, nunca posicional |

---

## 3. Diagnóstico do Estado Atual

### 3.1 Inconsistências Sintáticas (CRÍTICO)

| Contexto | Sintaxe Atual | Problema |
|----------|--------------|---------|
| Section config | `cols:4 style:dark` | `:` sem espaço |
| Page config | `type:custom requires:auth` | `:` sem espaço |
| Field modifiers | `string required unique` | Espaço separado, sem `:` |
| Field type+enum | `enum ["a", "b"] required` | Inline array + modifier |
| Bind filter | `where page eq "overview"` | `eq` em vez de `==` |
| Modal field | `field "Name" type:text placeholder:"value"` | Mix de posicional + named |
| Navigation | `item "Label" -> "/path" icon:name` | `->` + posicional + named |

**IA não consegue deduzir qual padrão usar.** A gramática precisa de 1 regra, não 5.

### 3.2 Ruído no Nova Core

| Categoria | Linhas | Chars | % do arquivo |
|-----------|--------|-------|-------------|
| Entities + APIs + Webhooks | 100 | ~3.5K | 28% (sinal) |
| Pages/sections (DSL) | 50 | ~2K | 14% (sinal) |
| Templates HTML raw | 170 | ~39K | 48% (ruído) |
| Inline JavaScript | 34 | ~8K | 10% (ruído) |
| **Total** | 354 | ~52K | |

### 3.3 Lacunas Semânticas

| Conceito | Status | Impacto |
|----------|--------|---------|
| Field defaults | Ausente | Toda entrada é NULL ou obrigatória |
| Field validation (min/max/pattern) | Ausente | Sem data integrity |
| Many-to-many relations | Ausente | Precisa tabela intermediária manual |
| Cascading deletes | Ausente | Referências órfãs |
| Computed fields | Ausente | Cálculos exigem JS |
| Auth como first-class | Parcial | Login/signup são 11K chars de HTML |
| Auto-CRUD pages | Ausente | Cada página é manual |
| Reactive bindings | Ausente | 12 fetch() imperativas |
| Design tokens | Parcial | Tailwind config de 6K chars inline |
| Pagination in API | Ausente | Manual limit/offset |

---

## 4. Propostas de Evolução (6 Phases)

### Phase 1: Gramática Uniforme — `!` e Constraints Inline

**Antes:**
```cronus
entity User {
  name string required
  email email required unique
  age number
  role enum ["admin", "user"] required
  password string required sensitive
}
```

**Depois:**
```cronus
entity User {
  name     string!
  email    email! unique
  age      number min:0 max:150
  role     enum ["admin", "user"]! default:"user"
  password string! sensitive
}
```

**Mudanças no parser:**
- `!` após o tipo = required (tokenizar como parte do tipo ou como modifier)
- `default:"value"` como modifier inline
- `min:N max:N` como modifiers de constraint
- `pattern:"regex"` como modifier de validação

**Impacto:** -30% tokens por entidade, zero ambiguidade para IA.

**Backward compatibility:** `required` keyword continua funcionando como alias de `!`.

### Phase 2: Auth First-Class

**Antes (11.222 chars de HTML):**
```cronus
page "/login" type:custom {
  section hero {
    template "<div style='min-height:100vh...' [5.611 chars of HTML + JS]"
  }
}
```

**Depois (~20 linhas):**
```cronus
auth {
  entity User
  strategy jwt
  roles ["admin", "operator", "viewer"]

  login "/login" {
    title "Secure Access"
    subtitle "Enter the operational command center"
    fields {
      email    label:"Identity Token"    placeholder:"operator@nova-core.io"
      password label:"Passphrase"
    }
    cta "Initialize Portal"
    redirect "/"
  }

  signup "/signup" {
    title "Create Access"
    subtitle "Join the operational network"
    fields {
      name     label:"Operator Name"    placeholder:"Your name"
      email    label:"Identity Token"    placeholder:"operator@nova-core.io"
      password label:"Passphrase"
    }
    cta "Initialize Access"
    redirect "/"
  }
}
```

**O kernel gera automaticamente:**
- Formulário glassmorphic com o theme do projeto
- Validação inline (email format, password min length)
- POST para /api/auth/login ou /api/auth/signup
- Token storage (HttpOnly cookie preferido, localStorage fallback)
- Redirect após auth
- Link entre login ↔ signup
- Error messages inline

**Impacto:** -95% do HTML de auth, zero JS manual.

### Phase 3: Auto-CRUD Pages

**Antes (~50 linhas por entity page):**
```cronus
page "/deployments" type:custom requires:auth {
  use NovaCoreChrome
  section page-header style:dark {
    eyebrow "Infrastructure"
    title "Active Deployments"
    subtitle "Real-time deployment pipeline..."
  }
  section kpi cols:4 {
    bind KpiSnapshot { where page eq "deployments" }
  }
  section table style:dark {
    title "Recent Deployments"
    search "Filter by service..."
    columns "Deploy ID, Service, Cluster, Duration, Status"
    bind Deployment { query all order created_at desc limit 20 }
  }
}
```

**Depois (~10 linhas):**
```cronus
page "/deployments" type:crud requires:auth {
  entity Deployment
  title "Active Deployments"
  subtitle "Real-time deployment pipeline"
  search "Filter by service, cluster, or status..."
  sort created_at desc
  limit 20
  hide_columns [created_at, updated_at, _owner_id]
}
```

**O kernel gera automaticamente:**
- Page header (do title/subtitle)
- KPI cards (count, recent, etc.)
- Table com todas as colunas (exceto hide_columns)
- Search input
- Sort headers
- Pagination
- Create modal (do entity fields)
- Edit/delete actions

**Impacto:** -80% linhas por página CRUD.

### Phase 4: Reactive Bindings

**Antes (12 fetch() imperativos no Nova Core):**
```javascript
fetch('/api/server/stats', {headers: h})
  .then(function(r) { return r.json() })
  .then(function(d) {
    document.getElementById('server-stats').innerHTML = ...
  })
```

**Depois (declarativo):**
```cronus
section stats {
  bind ServerStats { source "/api/server/stats" refresh:5s }
  
  item "Total Requests" value:{{total_events}} icon:trending_up
  item "Top Endpoint" value:{{top_endpoint}} icon:bolt
}
```

**Mudanças:**
- `bind` aceita `source` (URL customizada) além de entity
- `refresh:Ns` = auto-refresh via SSE ou polling
- `{{field}}` = mustache-style interpolação de dados bound
- O kernel gera o fetch + DOM update automaticamente

**Impacto:** Zero JS manual para data display.

### Phase 5: Component System

**Antes (define com 10K+ chars de HTML):**
```cronus
define NovaCoreChrome {
  section topbar {
    template "<header class=\"backdrop-blur-2xl... [1.309 chars]"
    style_block "header { position: fixed;... [4.360 chars]"
  }
  section sidebar {
    template "<aside class=\"bg-[#000000]... [3.500 chars]"
  }
}
```

**Depois (declarativo):**
```cronus
component AppShell {
  topbar {
    brand "NOVA CORE"
    links [
      "Docs" -> "/docs",
      "Deployments" -> "/deployments",
      "Alerts" -> "/notifications"
    ]
    actions {
      notifications -> "/notifications"
      settings -> "/settings"
    }
  }
  
  sidebar {
    user_profile from:auth
    nav [
      "Overview"     -> "/"              icon:rocket_launch,
      "Deployments"  -> "/deployments"   icon:terminal,
      "Analytics"    -> "/analytics"     icon:insert_chart,
      "Security"     -> "/security"      icon:shield,
      "Server"       -> "/server"        icon:dns,
      "Settings"     -> "/settings"      icon:settings
    ]
    cta "New Deployment" action:modal:"new-deployment"
    logout
  }
  
  footer {
    live_stats from:"/api/server/stats"
  }
}
```

**O kernel gera:** Todo o HTML, CSS, JS — responsivo, dark theme, SPA navigation, active states, user profile from auth, notification badge.

**Impacto:** -90% do ruído de templates.

### Phase 6: Design Tokens

**Antes (6K chars inline):**
```cronus
tailwind_config "tailwind.config = { darkMode: \"class\", theme: { extend: { colors: { \"tertiary\": \"#81ecff\", ... } } } }"
```

**Depois:**
```cronus
design {
  theme dark
  accent #87adff
  secondary #d277ff
  tertiary #81ecff
  
  fonts {
    headline "Space Grotesk"
    body "Inter"
  }
  
  radius compact   # compact | rounded | pill
  density normal   # compact | normal | spacious
  motion smooth    # none | subtle | smooth | dramatic
}
```

**O kernel gera:** Tailwind config, CSS custom properties, Material Design 3 tokens, glow effects, glass morphism — tudo derivado dos 5-10 tokens declarados.

**Impacto:** De 6K chars para 15 linhas.

---

## 5. Estimativa de Redução

### Nova Core (antes vs depois)

| Bloco | Antes | Depois | Redução |
|-------|-------|--------|---------|
| Entities | 70 linhas | 55 linhas (`!` syntax) | -21% |
| APIs | 30 linhas | 30 linhas (sem mudança) | 0% |
| Webhooks | 8 linhas | 8 linhas | 0% |
| Auth (login/signup) | 60 linhas (11K HTML) | 25 linhas | -58% |
| Chrome (topbar/sidebar/footer) | 15 linhas (19K HTML) | 35 linhas (declarativo) | +133% linhas, -95% chars |
| Dashboard pages (7) | 120 linhas | 50 linhas (type:crud) | -58% |
| Design tokens | 5 linhas (6K inline) | 15 linhas | +200% linhas, -97% chars |
| **Total** | 354 linhas (52K chars) | ~180 linhas (~8K chars) | **-49% linhas, -85% chars** |

### Para IA

| Métrica | Antes | Depois |
|---------|-------|--------|
| Tokens para gerar app | ~15K | ~3K |
| Ambiguidade sintática | Alta (5 padrões) | Zero (1 padrão) |
| Chance de erro | ~40% | ~5% |
| Precisa saber HTML/CSS/JS? | Sim | Não |
| Context window necessário | 32K+ | 8K |

---

## 6. Implementação Roadmap

| Phase | Effort | Impact | Breaking? |
|-------|--------|--------|-----------|
| 1. `!` + constraints + defaults | 1 semana | Alto | Não (backward compat) |
| 2. Auth first-class | 2 semanas | Muito alto | Não (novo bloco) |
| 3. Auto-CRUD pages (type:crud) | 2 semanas | Muito alto | Não (novo type) |
| 4. Reactive bindings (refresh, mustache) | 1 semana | Alto | Não (extensão de bind) |
| 5. Component system | 3 semanas | Transformacional | Sim (substitui define) |
| 6. Design tokens | 1 semana | Alto | Não (extensão de style) |

**Recomendação:** Phases 1-2-6 primeiro (4 semanas, sem breaking changes). Depois 3-4-5 (6 semanas).

---

## 7. Success Metrics

1. **Nova Core em <200 linhas** com mesmo poder expressivo (hoje 354)
2. **Zero HTML raw** em apps gerados (hoje 48% do arquivo)
3. **IA gera app válido em 1 shot** com <5% error rate (hoje ~40%)
4. **Context window <8K tokens** para app completo (hoje 15K+)
5. **`cronus generate` produz app funcional** sem edição manual
6. **Novos developers entendem a linguagem em <10 minutos** (hoje ~30 min com templates)
