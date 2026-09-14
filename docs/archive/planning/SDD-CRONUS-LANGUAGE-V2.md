# SDD: CRONUS Language v2 — Blueprint para a Linguagem #1 do Mundo

> Documento fundacional. Cada decisao aqui define o futuro do CRONUS.
> Data: 2026-04-07

---

## TESE CENTRAL

A vantagem do CRONUS que nenhuma linguagem tem: **o compilador ve a aplicacao inteira como um grafo semantico** — entidades, rotas, auth, data flow, UI, state machines, efeitos. React/Svelte compilam componentes isolados. CRONUS compila a *aplicacao*.

---

## 1. PRINCÍPIOS DA LINGUAGEM

### 1.1 Filosofia de Sintaxe

1. **Se pode ser declarado, DEVE ser declarado** (entidades, rotas, auth, layouts)
2. **Se precisa de logica, use expressoes** (tudo retorna valor)
3. **Se precisa de codigo imperativo, quarentena** (blocos `do {}` explicitos)

```cronus
// Declaracao — 80% dos casos
entity Order {
  customer: User
  items: [Product]
  total: money = sum(items.price)
  status: pending -> paid -> shipped -> delivered | cancelled
  
  on status:paid {
    notify customer "Order confirmed: {id}"
    decrement items.stock
  }
}

// Expressao — 15% dos casos
component PriceTag(amount: money, discount?: percent) {
  let final = discount ? amount * (1 - discount) : amount
  
  <span class="price {if discount: 'discounted'}">
    {format(final, currency: user.locale)}
    {if discount: <s class="original">{format(amount)}</s>}
  </span>
}

// Escape imperativo — 5% dos casos
api POST "/webhooks/stripe" {
  do {
    let event = verify_signature(request.body, env.STRIPE_SECRET)
    match event.type {
      "payment_intent.succeeded" -> Order.transition(event.data.order_id, :paid)
      "charge.disputed" -> alert_admin(event)
      _ -> log.warn("Unhandled: {event.type}")
    }
  }
}
```

### 1.2 Sistema de Tipos: Estrutural + Domain-Aware

```
text, integer, float, boolean, date, datetime
money     -> armazenado em centavos, formatado por locale, aritmetica currency-aware
email     -> validado no parse, normalizado lowercase, indexavel
slug      -> auto-gerado de campos text, unique-enforced, URL-safe
image     -> gera srcset, lazy loading, blur placeholder em compile time
phone     -> validado, formatado por pais
url       -> validado, preview metadata extraido
color     -> validado hex/rgb/oklch, gera paleta automatica
```

### 1.3 Mensagens de Erro (nivel Rust)

```
error[E0201]: Transicao de estado inalcancavel
  --> store.cronus:14:3
   |
14 |   status: draft -> published -> archived
   |                                 ^^^^^^^^
   |
   = Nenhuma page ou API endpoint dispara a transicao para `archived`
   = Este estado existe mas nada no seu app move uma entidade pra la
   
fix: Adicione um trigger:
   |
   |  on status:published after 90.days {
   |    transition status -> archived
   |  }

warning[W0450]: Performance — N+1 query detectado
  --> app.cronus:15:3
   |
15 |   list Order {
16 |     show: [customer.name, items.count]
   |            ^^^^^^^^^^^^^
   |
   = Cada Order vai disparar uma query separada para customer.name
   
fix: O compilador vai fazer batch automaticamente em producao.
```

---

## 2. PIPELINE DE COMPILACAO

```
.cronus source
    |
    v
 PARSE (2-5ms) -----> AST
    |
    v
 SEMANTIC ANALYSIS (10-20ms) -----> Resolve entidades, valida state machines,
    |                                 checa conflitos de rota, type-check
    v
 INTENT INFERENCE (5-10ms) -----> Detecta padroes: "auth", "CRUD",
    |                               "e-commerce", aplica otimizacoes de dominio
    v
 IR GENERATION -----> Representacao intermediaria platform-agnostic
    |
    v
 SPLIT COMPILATION
    |         |          |
    v         v          v
 SERVER    CLIENT     QUERIES
 (Rust)    (JS min)   (SQL opt)
```

### Multi-Target do Mesmo IR

| Target | Output | Caso |
|--------|--------|------|
| `web` | HTTP server Rust + HTML/CSS/JS minimo | Default |
| `static` | HTML/CSS puro, zero JS | Landing pages |
| `spa` | Client-side app + API server | Dashboards |
| `edge` | Cloudflare Workers / Deno Deploy | Distribuicao global |
| `native` | WebView shell + APIs nativas | Desktop/mobile |

### Otimizacoes Whole-Program

- **Dead route elimination**: page existe mas nenhum link aponta pra ela? Warning
- **Query fusion**: 2 componentes na mesma page fazem query na mesma entidade? 1 SQL query
- **CSS atomic**: so classes usadas sao enviadas, zero runtime
- **Precomputed joins**: relationships conhecidas em compile time -> views materializadas
- **Code splitting automatico**: compilador sabe quais componentes em quais pages -> bundles por rota

---

## 3. RUNTIME: ZERO A MICRO

```
STATIC PAGE          INTERACTIVE PAGE         REAL-TIME PAGE
-------------------------------------------------------------
0 KB runtime         ~1.2 KB runtime          ~3 KB runtime
HTML/CSS puro        DOM patches cirurgicos   + EventSource client
Zero JS              Sem VDOM, sem diffing    Server-pushed state
```

### Porque NAO VDOM

O VDOM existe porque React nao conhece a component tree em build time. CRONUS conhece. Cada binding e uma referencia direta ao no DOM:

```cronus
component Counter {
  state count: integer = 0
  
  <button on click: count += 1>
    Clicked {count} times
  </button>
}
```

Compila para:
```js
const btn = document.getElementById('c_0');
const txt = document.getElementById('c_0_t');
let count = 0;
btn.onclick = () => { count++; txt.data = `Clicked ${count} times`; };
```

Zero framework. Zero runtime. Direto no DOM.

### Real-Time como Primitiva

```cronus
page "/chat/{room.slug}" {
  live list Message where room = params.room {
    order: -created
    limit: 50
  }
  
  form Message {
    fields: [body]
    defaults: { room: params.room, author: current_user }
  }
}
```

`live` = compilador gera SSE endpoint + trigger no DB + client que prepend no DOM + reconnection.

---

## 4. DEVELOPER EXPERIENCE

### 4.1 Testes Co-localizados

```cronus
component LoginForm {
  state email: email
  state password: text
  state error: text?
  
  <form on submit: authenticate(email, password)>
    <input bind: email />
    <input bind: password type="password" />
    {if error: <p class="error">{error}</p>}
    <button>Login</button>
  </form>
  
  test "mostra erro com credenciais invalidas" {
    fill email "bad@test.com"
    fill password "wrong"
    click "Login"
    expect visible ".error" with "Invalid credentials"
  }
}
```

### 4.2 Deploy como Feature da Linguagem

```cronus
deploy {
  target: docker
  database: postgres
  assets: cdn("r2")
  domain: "app.example.com"
}
```

`cronus deploy` le esse bloco e gera o artefato.

### 4.3 Dev Server com Time Travel

```bash
$ cronus dev
  > http://localhost:4800
  > 3 entities, 7 pages, 12 components
  > compiled in 34ms
```

Overlay de dev grava cada mudanca de estado. Voltar no tempo, inspecionar DB, replay de interacoes.

---

## 5. ECOSISTEMA: BLOCKS

```cronus
use @cronus/auth-magic-link
use @cronus/payments-stripe
use @cronus/blog
```

Blocks sao **composicao source-level** — o compilador inline as declaracoes e otimiza tudo junto. Sem node_modules. Sem dependency tree.

---

## 6. STATE MACHINES TEMPORAIS

```cronus
entity Subscription {
  status: trial -> active -> past_due -> cancelled | expired
  
  on status:trial after 14.days {
    if payment_method: transition -> active
    else: transition -> expired
  }
  
  on status:active every 30.days {
    charge(customer, plan.price)
    on failure: transition -> past_due
  }
  
  on status:past_due after 7.days {
    transition -> cancelled
    notify customer "subscription_cancelled"
  }
}
```

Compilador gera cron jobs, retry logic, notifications. Zero worker separado.

---

## 7. ESTRATÉGIA DE ADOCAO

### Hook de 3 Minutos

```bash
curl -fsSL https://cronus.dev/install | sh
cronus new myapp && cd myapp
cronus dev
```

O `app.cronus` gerado:
```cronus
app "myapp" { auth: email }

entity Task {
  title: text
  done: boolean = false
  owner: User
}

page "/" {
  list Task where owner = current_user { toggle: done }
  form Task { fields: [title] }
}
```

**15 linhas = full-stack app com auth, DB, SSR, CRUD. Rodando em 80ms.**

### Migracao Gradual

- `cronus export react` -> gera componentes React do CRONUS
- `cronus import react ./Button.tsx` -> gera .cronus equivalente
- API REST/GraphQL padrao -> qualquer frontend consome

### AI-Native

CRONUS e estruturalmente vantajoso pra LLMs: cada programa valido e composicao de declaracoes bem-definidas. O compilador valida tudo. LLM nao consegue gerar runtime error porque nao ha runtime code pra 80% dos casos.

---

## 8. PERFORMANCE vs REACT

| Metrica | React/Next | CRONUS |
|---------|-----------|--------|
| Bundle JS | 80-200KB | 1-5KB |
| Hydration | 100-500ms | 0ms |
| TTFB | ~200ms | <50ms (edge) |
| TTI | Apos hydration | First paint |
| Rebuild | 2-10s | <50ms |
| Binary | node_modules 500MB | 6MB single binary |

---

## 9. PROXIMAS PHASES DO KERNEL

| # | Feature | Impacto | Esforco |
|---|---------|---------|---------|
| 1 | Component com reactive state (signals) | Fundacao de tudo | Grande |
| 2 | SPA Router (client-side navigation) | UX sem reload | Medio |
| 3 | `live` keyword (SSE real-time) | Feature "wow" | Medio |
| 4 | `test` blocks co-localizados | DX signal | Medio |
| 5 | `use` blocks (ecosystem) | Composicao | Medio |
| 6 | N+1 detection + query fusion | Whole-program advantage | Grande |
| 7 | Multi-target IR | Platform expansion | Grande |
| 8 | `deploy` block | DX final | Medio |

---

## O MOAT

CRONUS ganha em 3 eixos simultaneamente:

- **Simplicidade**: 15 linhas = full-stack app (bate Rails scaffolding)
- **Performance**: zero-to-micro runtime, binary compilado, whole-program optimization (bate React/Next)
- **Corretude**: compilador enforca best practices, detecta estados inalcancaveis, previne N+1 (bate tudo)

Nenhuma linguagem/framework existente consegue os 3 ao mesmo tempo.
