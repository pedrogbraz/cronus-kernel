# SDD — CRONUS NextGen: Superando Next.js e VINEXT

> Version: 1.0 | Data: 2026-04-09
> Autor: Zedd + CRONUS AI
> Status: APROVADO PARA IMPLEMENTACAO

---

## 1. VISAO

Uma unica linha `.cronus` faz o que Next.js precisa de 15 arquivos, 300MB de node_modules e 3 frameworks para fazer. E faz melhor — com audit trail, trust scoring, constitution enforcement, e zero dependencias externas.

**Objetivo**: Tornar .cronus o **compilador de intencao definitivo para web apps** — superior a Next.js, VINEXT, Astro, SvelteKit e qualquer framework baseado em JavaScript.

---

## 2. ESTADO ATUAL

### 2.1 O Que .cronus JA Tem (28 features)

| Categoria | Features |
|-----------|----------|
| **Routing** | File-based declarativo, dynamic `:id`, catch-all, API CRUD auto |
| **Auth** | JWT HS256 + Argon2id, roles, pages protegidas, auto login/signup |
| **UI** | 51 section types, data binding, forms, charts, kanban, tables |
| **Data** | SQLite auto-migration, binding queries, aggregation, GROUP BY |
| **Scripting** | .scriptcronus VM (7 namespaces: db, http, sse, log, format, env, auth) |
| **Quality** | 13 lint rules, constitution enforcement, resolve pass, trust engine |
| **Observability** | Zeus dashboard, SHA-256 audit trail, request tracing |
| **Evolution** | Hydra block promotion, registry, lineage tracking |
| **GraphQL** | Auto-gerado de entities, playground built-in |
| **Dump** | HTML, Prisma, OpenAPI, TypeScript, **Next.js projects** |

### 2.2 O Que .cronus NAO Tem (Gaps vs Next.js)

| Gap | Impacto | Dificuldade |
|-----|---------|-------------|
| Server Islands / Partial Prerendering | CRITICO | MEDIO |
| Caching declarativo (ISR, cache por bloco) | CRITICO | MEDIO |
| View Transitions (navegacao animada) | ALTO | FACIL |
| Speculation Rules (prefetch inteligente) | ALTO | FACIL |
| Middleware layer (pre-routing hooks) | ALTO | MEDIO |
| Server Actions (mutation colocada com UI) | ALTO | MEDIO |
| Streaming SSR (Suspense-like) | MEDIO | DIFICIL |
| Image/Font optimization | MEDIO | FACIL |
| i18n routing | BAIXO | MEDIO |
| React Server Components (RSC) | BAIXO | NAO NECESSARIO |

### 2.3 O Que .cronus Tem e Next.js NUNCA Tera

| Feature CRONUS | Next.js Equivalente | Por Que Next.js Nunca Tera |
|----------------|--------------------|-----------------------------|
| **Trust Engine** (6 eixos + 4 gates) | Nada | Precisa ser built-in na linguagem |
| **Audit Trail** (SHA-256 chain, tamper-proof) | Nada | Precisa de triggers no DB, compilador gera |
| **Constitution** (must/never compilaveis) | ESLint (ignoravel) | Precisa de parser customizado |
| **Hydra Evolution** (auto-promocao por evidencia) | Nada | Precisa de metrics + registry no runtime |
| **Auto GraphQL** (schema de entities) | Manual | Precisa de single-source-of-truth |
| **Semantic Memory** (decisoes + anti-patterns) | Nada | Precisa de DB persistente na linguagem |
| **Zero-Config Observability** (/zeus) | OpenTelemetry manual | Precisa de spans automaticos do runtime |
| **Ownership Isolation** (_owner_id enforcement) | Developer discipline | Precisa de enforcement no runtime |
| **7MB single binary** | 300MB node_modules | Rust vs Node.js |
| **Dump System** (absorve projetos inteiros) | Nada | Precisa de parser multi-formato |

---

## 3. ARQUITETURA NEXTGEN

### 3.1 O Modelo Mental

```
    DECLARAR  ──────────────────>  OBTER TUDO
    
    50 linhas .cronus              = Database + API + UI + Auth
                                    + GraphQL + Cache + SSR
                                    + Audit + Trust + Observability
                                    + View Transitions + Prefetch
                                    + Server Islands + Streaming
                                    + Edge Deploy
```

### 3.2 Pipeline de Compilacao NextGen

```
.cronus source
    |
    v
[1] Parse (tokenizer -> AST)              ~3ms
    |
    v
[2] Resolve (symbol table, refs)           ~1ms
    |
    v
[3] Lint (13 rules + constitution)         ~2ms
    |
    v
[4] Plan (render strategy per section)     ~1ms   <-- NOVO
    |   - static: pre-render no build
    |   - stream: SSR streaming com chunks
    |   - island: renderiza independente com cache
    |   - dynamic: renderiza por request
    |
    v
[5] Emit (HTML + JS runtime + cache headers)
    |
    v
[6] Serve (hyper HTTP + middleware pipeline)
```

### 3.3 Novo Modelo de Rendering

```cronus
page "/products" {
  section hero render:static {
    title "Our Products"
    subtitle "Browse our catalog"
  }

  section table render:island cache:5m {
    bind Product { query all order created_at desc }
    columns "name, price, status"
  }

  section chart render:stream {
    bind Order { aggregate sum field:total group_by:created_at interval:month }
  }
}
```

**4 estrategias de render por section**:

| Estrategia | Quando | Cache | Latencia |
|------------|--------|-------|----------|
| `static` | Conteudo que nunca muda (hero, footer, marketing) | Infinito | 0ms (pre-built) |
| `island` | Dados que mudam a cada minutos (tabelas, KPIs) | `cache:Xm` | ~5ms (cache hit) |
| `stream` | Dados que mudam sempre (charts real-time, feeds) | Nenhum | ~50ms (DB query) |
| `dynamic` | Default — renderiza por request | Request-scoped | ~10ms |

---

## 4. FEATURES NEXTGEN — IMPLEMENTACAO

### Fase 1: FOUNDATIONS (1 semana)

#### 4.1 Render Strategy per Section

**AST**: Adicionar `render` ao SectionNode:
```rust
pub struct SectionNode {
    // ... existente ...
    pub render_strategy: RenderStrategy,  // NOVO
    pub cache_ttl: Option<u32>,           // NOVO (segundos)
}

pub enum RenderStrategy {
    Dynamic,  // default
    Static,
    Island,
    Stream,
}
```

**Parser**: Reconhecer `render:static|island|stream|dynamic` e `cache:5m|1h|30s`.

**Runtime**: O render engine checa `render_strategy` e decide:
- `Static`: pre-renderiza HTML no build, serve do cache
- `Island`: renderiza independente, cacheia com TTL
- `Stream`: inicia render, flush chunks via chunked transfer encoding
- `Dynamic`: comportamento atual

#### 4.2 Middleware Layer

**AST**: Novo node `MiddlewareNode`:
```rust
pub struct MiddlewareNode {
    pub name: String,
    pub matchers: Vec<String>,    // ["/api/*", "/dashboard/*"]
    pub actions: Vec<MiddlewareAction>,
}

pub enum MiddlewareAction {
    RequireAuth { role: Option<String> },
    AddHeader { key: String, value: String },
    Redirect { to: String, status: u16 },
    RateLimit { max: u32, window: u32 },
    Rewrite { to: String },
}
```

**Sintaxe .cronus**:
```cronus
middleware auth {
  match "/dashboard/*" "/api/*"
  require auth role:admin
  header "X-Frame-Options" "DENY"
  rate_limit 100 per:60s
}

middleware redirects {
  match "/old-blog/*"
  rewrite "/blog/*"
}
```

**Runtime**: Pipeline de middleware executa ANTES do routing:
```
Request -> Middleware Chain -> Router -> Handler -> Response
```

#### 4.3 Declarative Caching

**Sintaxe**:
```cronus
page "/products" cache:5m {
  section kpi cache:1m { ... }    # cache por section
  section hero cache:forever { ... }  # nunca expira
}

api /products {
  list GET / auth:public cache:30s
}
```

**Runtime**: Cache em memoria com HashMap<CacheKey, (HTML, Expiry)>:
- Key = route + query params + owner_id (se auth)
- Stale-while-revalidate: serve stale, regenera em background
- Tag-based invalidation: `revalidate tag:products` no ScriptCronus

**ScriptCronus extension**:
```
on Product.create {
  cache.invalidate tag:"products"
  cache.invalidate path:"/products"
}
```

---

### Fase 2: MODERN WEB (1 semana)

#### 4.4 View Transitions API

**Sintaxe**:
```cronus
app "MyApp" {
  transitions true       # habilita View Transitions globalmente
}

page "/about" transition:slide {
  # transicao especifica desta pagina
}
```

**Emissao**: O compiler gera:
```html
<meta name="view-transition" content="same-origin">
```

E para cada section com `transition:*`:
```css
.section-hero { view-transition-name: hero; }
```

**Client Runtime** (~20 linhas de JS):
```js
// SPA navigation com View Transitions
document.addEventListener('click', e => {
  const a = e.target.closest('a[href^="/"]');
  if (!a) return;
  e.preventDefault();
  if (document.startViewTransition) {
    document.startViewTransition(() => cronusNavigate(a.href));
  } else {
    cronusNavigate(a.href);
  }
});
```

#### 4.5 Speculation Rules (Prefetch)

**Sintaxe**:
```cronus
page "/products" prefetch:eager {
  # esta pagina sera pre-carregada automaticamente
}

layout Main {
  sidebar {
    "Dashboard" -> "/" prefetch:hover
    "Products" -> "/products" prefetch:eager
    "Settings" -> "/settings"   # sem prefetch
  }
}
```

**Emissao**: Gera `<script type="speculationrules">`:
```json
{
  "prerender": [{"where": {"href_matches": "/products"}}],
  "prefetch": [{"where": {"selector_matches": "a[data-prefetch='hover']"}, "eagerness": "moderate"}]
}
```

#### 4.6 Image Optimization

**Sintaxe**:
```cronus
section hero {
  image "/hero.jpg" width:1200 height:600 priority
  image "/team.jpg" width:400 lazy
}
```

**Emissao**:
```html
<img src="/hero.jpg" width="1200" height="600" 
     fetchpriority="high" decoding="async"
     srcset="/hero.jpg?w=640 640w, /hero.jpg?w=1200 1200w"
     sizes="(max-width: 640px) 640px, 1200px">
```

**Runtime**: Endpoint `/_cronus/image?url=X&w=Y&q=Z` que redimensiona on-the-fly (usando a crate `image` do Rust).

#### 4.7 Font Optimization

**Sintaxe**:
```cronus
style {
  font "Inter" provider:google weight:[400, 600, 700]
  font "JetBrains Mono" provider:google weight:[400] subset:latin
}
```

**Emissao**: Gera CSS inline + preconnect:
```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<style>
@font-face { font-family: 'Inter'; src: url(...); font-display: swap; }
</style>
```

---

### Fase 3: SERVER POWER (1 semana)

#### 4.8 Server Actions

**Sintaxe**:
```cronus
page "/products/new" requires:auth {
  section form {
    bind Product
    
    action submit {
      validate
      create Product
      toast "Product created!" success
      navigate "/products"
    }
    
    action delete confirm:"Are you sure?" {
      delete Product route.id
      toast "Deleted" success
      navigate "/products"
    }
  }
}
```

Ja existe parcialmente via `on submit`. A melhoria: tornar `action` um bloco first-class que:
1. Valida campos automaticamente (usa entity constraints)
2. Executa operacao no server (sem client-side JS)
3. Retorna resultado + redirect/toast
4. Suporta `confirm:` para delete destrutivo

#### 4.9 Server Functions (ScriptCronus Extension)

```
# novo bloco: server function callable de pages
function calculate_shipping auth:jwt {
  let weight = event.body.weight
  let country = event.body.country
  
  if country == "BR" {
    let base = 15.00
    let per_kg = weight * 2.50
    respond 200 { price base + per_kg currency "BRL" }
  } else {
    let rate = http.get "https://api.shipping.com/rate" {
      headers { Authorization "Bearer {{env.SHIPPING_KEY}}" }
    }
    respond 200 rate.json
  }
}
```

Callable de qualquer page:
```cronus
page "/checkout" {
  section form {
    action calculate {
      call calculate_shipping { weight field.weight country field.country }
      set shipping_cost result.price
    }
  }
}
```

#### 4.10 Streaming SSR (Chunked Sections)

Para sections com `render:stream`:

```rust
// Em render.rs
fn render_section_stream(section: &SectionNode, db: &CronusDB) -> impl Stream<Item = String> {
    // 1. Emit section shell (titulo, container)
    yield format!("<section id='{}'><h2>{}</h2><div class='loading'>", id, title);
    
    // 2. Query DB (pode ser lento)
    let data = resolve_binding(section, db);
    
    // 3. Emit section content
    yield render_section_content(section, &data);
    
    // 4. Close section
    yield "</div></section>";
}
```

O HTTP response usa `Transfer-Encoding: chunked`:
```
HTTP/1.1 200 OK
Transfer-Encoding: chunked
Content-Type: text/html

<html><body>
[hero section - instant]
[loading placeholder for table]
[loading placeholder for chart]

... later ...
[table data arrives - replaces placeholder]
[chart data arrives - replaces placeholder]
</body></html>
```

---

### Fase 4: EDGE & AI (1 semana)

#### 4.11 Cloudflare Workers Target

**Sintaxe**:
```cronus
app "MyApp" {
  stack react + tailwind
  port 5175
  deploy cloudflare {
    workers true
    kv "CACHE"
    d1 "main-db"
    r2 "assets"
  }
}
```

**Compilacao**: `cronus build --target cloudflare` gera:
- `worker/index.ts` (entry point)
- `wrangler.jsonc` (config)
- `dist/` (assets)

#### 4.12 Agent Block (Cloudflare Agents SDK)

**Sintaxe**:
```cronus
agent support_bot {
  model "claude-sonnet"
  tools [search_docs, create_ticket, check_status]
  state durable
  schedule "0 */6 * * *"
  
  tool search_docs {
    endpoint GET /api/docs/search
    param query string!
  }
  
  tool create_ticket {
    endpoint POST /api/tickets
    param title string!
    param description text!
    param priority enum [low, medium, high]
  }
}
```

**Runtime**: Compila para Cloudflare Agent com Durable Object:
- Estado persistente entre invocacoes
- WebSocket para comunicacao real-time
- Scheduling integrado
- Tools mapeiam para API routes existentes

#### 4.13 Workflow Block

**Sintaxe**:
```cronus
workflow user_onboarding {
  step validate_email {
    call verify_email { email event.user.email }
    on_error retry:3 delay:5s
  }
  
  step create_account {
    create User { email event.user.email role "user" }
  }
  
  step send_welcome {
    call send_email { template "welcome" to event.user.email }
    wait 1h
  }
  
  step check_activation {
    let user = db.query User { filter email eq:event.user.email }
    if user.activated == false {
      call send_reminder { to event.user.email }
    }
  }
}
```

---

## 5. TABELA COMPARATIVA FINAL

| Feature | Next.js 16 | VINEXT | CRONUS NextGen |
|---------|-----------|--------|----------------|
| **Setup** | 300MB node_modules | 50MB vite + deps | **7MB single binary** |
| **Config files** | 5-10 (next.config, tsconfig, postcss, etc) | 3-5 | **1 arquivo .cronus** |
| **Lines for CRUD app** | 500-2000 | 400-1500 | **30-50** |
| **Auth** | Biblioteca externa (NextAuth) | Shim do NextAuth | **Built-in JWT + Argon2id** |
| **Database** | Prisma + migrate | Prisma + migrate | **Auto-migration de entities** |
| **GraphQL** | Apollo/Urql + schema manual | Nao tem | **Auto-gerado** |
| **Audit trail** | Nao tem | Nao tem | **SHA-256 chain built-in** |
| **Trust scoring** | Nao tem | Nao tem | **6-axis + 4 gates** |
| **Constitution** | ESLint (ignoravel) | ESLint | **Compiler-enforced** |
| **Observability** | OpenTelemetry manual | Nao tem | **Zero-config /zeus** |
| **Cache** | ISR + fetch cache | ISR + fetch cache | **Declarativo: cache:5m** |
| **View Transitions** | Manual | Manual | **transitions true** |
| **Prefetch** | Link prefetch | Link prefetch | **prefetch:eager/hover** |
| **Render Strategy** | PPR (Next.js 16 only) | Nao tem | **render:static/island/stream** |
| **Middleware** | middleware.ts | middleware.ts | **middleware block declarativo** |
| **Server Actions** | "use server" + form | "use server" + form | **action block declarativo** |
| **AI Agents** | Nao tem | Nao tem | **agent block + Durable Objects** |
| **Workflows** | Nao tem | Nao tem | **workflow block durable** |
| **Dump/Migration** | Manual rewrite | vinext init | **cronus dump --nextjs** |
| **Edge deploy** | Vercel only | Cloudflare Workers | **cronus build --target cloudflare** |
| **Evolution** | Nao tem | Nao tem | **Hydra auto-promotion** |
| **Memory** | Nao tem | Nao tem | **Semantic memory built-in** |
| **Startup** | 2-5s (Node.js) | 1-3s (Vite) | **<100ms (Rust)** |
| **Binary size** | 300MB+ | 50MB+ | **7MB** |
| **Runtime deps** | Node.js + npm | Node.js + Vite | **Zero** |

---

## 6. ROADMAP DE IMPLEMENTACAO

| Fase | Semana | Features | Testes |
|------|--------|----------|--------|
| **1: Foundations** | 1 | render strategy, middleware, cache declarativo | +30 testes |
| **2: Modern Web** | 2 | view transitions, speculation rules, image/font opt | +20 testes |
| **3: Server Power** | 3 | server actions, server functions, streaming SSR | +25 testes |
| **4: Edge & AI** | 4 | cloudflare target, agent block, workflow block | +20 testes |

**Total: 4 semanas, ~95 novos testes, zero breaking changes.**

---

## 7. CRITERIOS DE SUCESSO

1. `cronus dump --nextjs ./any-nextjs-app` gera .cronus funcional em <5s
2. App .cronus com 50 linhas faz o mesmo que Next.js app com 2000 linhas
3. `cronus build` detecta 95% dos bugs pre-runtime
4. Startup em <100ms (vs 2-5s do Next.js)
5. View Transitions funcionam sem 1 linha de JS do developer
6. Cache declarativo reduz latencia de tabelas em 90%+ (cache hit)
7. Trust scoring impede deploy de codigo com error rate >5%
8. Constitution enforcement bloqueia sensitive fields em templates
9. 7MB binary vs 300MB node_modules
10. IA gera .cronus valido em 1 shot com 95%+ success rate

---

## 8. A TESE FINAL

Next.js e um **framework de execucao** — voce escreve codigo que roda.
CRONUS e um **compilador de intencao** — voce declara o que quer e o compilador prova que funciona.

A diferenca nao e velocidade ou features. E que CRONUS torna impossivel:
- Deploys com dados sensiveis expostos (constitution)
- Endpoints sem metricas de confiabilidade (trust)
- Mudancas de dados sem rastro (audit)
- Codigo morto sem deteccao (lint)
- Blocos nao testados em producao (hydra gates)

**Next.js pede que voce seja cuidadoso. CRONUS garante que voce seja.**

Isso nao e um framework. E uma linguagem que compila intencao em realidade verificavel.
