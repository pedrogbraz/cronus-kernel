# SDD — CRONUS as Spec-Driven Language

> Especificação executável: o código É a spec. Zero ambiguidade, zero doc externo.
> Author: Zedd + Claude | Date: 2026-04-02 | Status: PROPOSED

---

## 0. Filtro Crítico — O Que Serve vs O Que Não Serve

O prompt original propõe uma linguagem general-purpose com spec embutida.
CRONUS é uma **DSL declarativa** que gera apps web. Nem tudo se aplica.

### ADOTAR (alto impacto, zero overhead):

| Conceito | Por que faz sentido | Como adaptar |
|----------|-------------------|-------------|
| **Invariants como first-class** | Já temos constitution + lint. Falta ser parte da gramática, não strings | `invariant` block com semântica real |
| **State transitions** | Entities com `enum` têm estados implícitos sem transições definidas | `transition` block define estado válido |
| **Contracts entre módulos** | Pages dependem de entities, APIs dependem de entities — sem verificação | `contract` verifica no compile-time |
| **Effects declarativos** | Webhooks são effects, mas limitados. Falta: notify, log, validate, transform | `effect` block com pipeline |
| **Constraints como semântica** | Validação é strings no constitution. Precisa ser compilável | `constraint` block com expressões |
| **Compile-time validation** | Lint roda pós-parse. Precisa rodar DURANTE o parse quando possível | Semantic analysis pass |

### REJEITAR (over-engineering para DSL):

| Conceito | Por que NÃO faz sentido |
|----------|------------------------|
| Ownership/lifetime model | CRONUS gera SQL + HTML, não gerencia memória |
| VM/bytecode/IR lowering | É um compilador para HTTP server, não um runtime |
| Dynamic dispatch | DSL declarativa, sem polimorfismo runtime |
| Reflection | Não existe execução dinâmica de .cronus |
| Async scheduler | Tokio já faz isso no Rust, .cronus não precisa expor |
| Capability system | Overkill para web apps — auth + roles é suficiente |

### ADAPTAR (bom conceito, implementação diferente):

| Conceito | Adaptação para CRONUS |
|----------|----------------------|
| Execution boundaries | → `scope` block (quais entities uma page pode acessar) |
| Effect tracking | → Effects são automáticos (audit, SSE, webhook) mas declaráveis |
| Deterministic execution | → Já é (SQLite + single-threaded queries). Documentar como garantia |

---

## 1. Core Semantic Model — CRONUS Spec-Driven

### 1.1 O que é cada coisa no CRONUS

```
MÓDULO     = Um arquivo .cronus (self-contained, um sistema)
ENTITY     = Unidade stateful (tabela SQL + CRUD API + tipo)  
TRANSITION = Mudança válida de estado em um entity field
EFFECT     = Ação automática disparada por uma mutation
CONTRACT   = Garantia entre dois blocos (entity↔page, entity↔api)
INVARIANT  = Regra que NUNCA pode ser violada (compile-time)
CONSTRAINT = Validação de dados (runtime, no INSERT/UPDATE)
SCOPE      = Quais entities uma page/api pode acessar
```

### 1.2 O que é validado quando

```
COMPILE-TIME (parse + semantic analysis, <10ms):
├── Gramática válida (parser)
├── Identificadores SQL-safe (P040/P041)
├── Referências resolvem (entity exists, page route exists)
├── Transitions são válidas (from→to declarados)
├── Contracts satisfeitos (page binds entity que existe)
├── Scopes respeitados (page não acessa entity fora do scope)
├── Invariants verificados (constitution rules)
└── Sensitive fields nunca expostos

RUNTIME (no HTTP request handler):
├── Constraints de campo (min/max/pattern)
├── Transition guards (estado atual permite transição?)
├── Auth enforcement (JWT + role + scope)
├── Rate limiting
├── Audit logging
└── Effect execution (webhook, notify, etc.)

PROIBIDO (impossible by design):
├── location.reload() (não existe no emitter)
├── Sensitive em SELECT (excluído do query builder)
├── SQL injection (parameterized + identifier validation)
└── Hardcoded data em sections de dados (lint ERROR)
```

---

## 2. Novos Constructs da Linguagem

### 2.1 `transition` — Estado válido de um field enum

**Problema atual:** `status enum ["Pending", "Rolling", "Live", "Failed"]` aceita qualquer transição. Nada impede `Failed → Pending` via PATCH.

**Solução:**

```cronus
entity Deployment shared {
  deploy_id string!
  service   string!
  status    enum ["Pending", "Rolling", "Live", "Failed"]! default:"Pending"

  transition status {
    Pending -> Rolling
    Rolling -> Live
    Rolling -> Failed
    Failed  -> Pending   # retry allowed
    Live    -> Rolling   # redeploy
  }
}
```

**Semântica:**
- No PATCH de `status`, o kernel verifica se `old_value → new_value` está no transition map
- Se não está: `409 Conflict { error: "Invalid transition: Live → Failed. Allowed: Live → Rolling" }`
- Compile-time: verifica que todos os enum values aparecem pelo menos uma vez no transition map
- Se `transition` não é declarado: qualquer transição é válida (backward compat)

**Custo runtime:** 1 hashmap lookup por PATCH em campo com transition. ~0.001ms.

### 2.2 `constraint` — Validação declarativa

**Problema atual:** Constitution rules são strings ("prices in centavos"). Não são compiláveis.

**Solução:**

```cronus
entity User {
  name     string! min:2 max:100
  email    email! unique
  age      number  min:0 max:150
  username string! min:3 max:30 match:"^[a-z0-9_]+$"
  role     enum ["admin", "operator", "viewer"]! default:"viewer"
  
  constraint "admin requires email verification" {
    when role == "admin"
    require email_verified == true
  }
}
```

**Semântica:**
- `min:N max:N match:"regex"` são constraints de campo — validados em INSERT/UPDATE
- `constraint "name" { when ... require ... }` são constraints cross-field
- Falha retorna: `422 Unprocessable Entity { error: "admin requires email verification", constraint: "..." }`
- Compile-time: verifica que fields referenciados existem na entity

**Custo:** Regex compile uma vez no startup (lazy_static). Check é ~0.01ms per constraint.

### 2.3 `effect` — Pipeline de ações automáticas

**Problema atual:** Webhooks são o único effect. Falta: notify, log, transform, validate.

**Solução:**

```cronus
entity Deployment shared {
  # ... fields ...

  effect on_create {
    audit                          # log no audit trail (já é automático, aqui é explícito)
    notify "slack" channel:"#deploys" message:"Deploy {{deploy_id}} started by {{deployed_by}}"
    validate service exists_in Endpoint.path
  }

  effect on_update status {
    when new.status == "Failed" {
      notify "pagerduty" severity:"critical"
      log "Deploy {{deploy_id}} FAILED on {{cluster}}"
    }
    when new.status == "Live" {
      notify "slack" channel:"#deploys" message:"{{deploy_id}} is LIVE"
    }
  }

  effect on_delete {
    log "Deploy {{deploy_id}} removed by {{_owner_id}}"
    cascade SecurityEvent where deploy_id == this.deploy_id
  }
}
```

**Semântica:**
- Effects rodam APÓS a mutation ser commitada no DB (não bloqueiam response)
- `notify` dispara webhook para provider configurado
- `validate ... exists_in` verifica integridade referencial customizada
- `cascade` propaga deletes para entities relacionadas
- `when` filtra por condição (acesso a `old.*` e `new.*`)
- `{{field}}` interpola valores da entity

**Compile-time:** Verifica que fields referenciados existem. Verifica que providers ("slack", "pagerduty") estão configurados no app block.

**Custo:** Effects rodam em tokio::spawn (async, não bloqueia). ~0ms no response time.

### 2.4 `contract` — Garantia entre blocos

**Problema atual:** Nada verifica que uma page bind a uma entity que realmente existe, ou que um API route serve uma entity com os campos certos.

**Solução:**

```cronus
# Implícito — o compilador infere contracts de bind/api:

page "/deployments" type:crud requires:auth {
  entity Deployment    # CONTRATO: Deployment deve existir, ser shared, e ter campos renderizáveis
}

# Explícito — para regras de negócio complexas:

contract "deploy-integrity" {
  entity Deployment
  entity Endpoint

  # Todo Deployment.service deve corresponder a um Endpoint.path
  require Deployment.service in Endpoint.path

  # Deployments "Live" devem ter duration > 0
  require Deployment.status == "Live" implies Deployment.duration != "0s"
}
```

**Semântica:**
- Contracts implícitos: inferidos de `bind`, `entity`, `->` references
- Contracts explícitos: regras de negócio verificáveis
- Compile-time: verifica que entities/fields existem e tipos são compatíveis
- Runtime: `require` é verificado em INSERT/UPDATE (cross-entity)
- Violação: `409 Conflict { error: "contract 'deploy-integrity' violated: ..." }`

**Custo:** Compile-time check = 0 runtime cost. Runtime cross-entity check = 1 SQL query.

### 2.5 `scope` — Isolamento de acesso

**Problema atual:** Qualquer page pode bind qualquer entity. Não há isolamento.

**Solução:**

```cronus
# Scopes definem quais entities cada contexto pode acessar

scope admin {
  entities [Deployment, SecurityEvent, Endpoint, KpiSnapshot, User]
  pages ["/", "/deployments", "/analytics", "/security", "/server", "/settings"]
  apis ["/deployments", "/securityevents", "/endpoints", "/kpisnapshots"]
}

scope public {
  entities [Notification]
  pages ["/notifications"]
  apis ["/notifications"]
}
```

**Semântica:**
- Compile-time: se uma page no scope `public` tenta bind `Deployment`, erro
- Runtime: API routes fora do scope retornam 403
- Opcional — se não declarado, tudo é acessível (backward compat)

**Custo:** Compile-time only. Zero runtime overhead quando não usado.

---

## 3. Semantic Analysis Pipeline

### Atual (2 passes):
```
Pass 1: Parse (.cronus → AST)           ~3ms
Pass 2: Lint (13 regras pós-parse)       ~2ms
```

### Proposto (4 passes):
```
Pass 1: Parse (.cronus → AST)           ~3ms
Pass 2: Resolve (referências, types)     ~1ms  ← NOVO
Pass 3: Check (contracts, transitions,   ~2ms  ← NOVO
         constraints, scopes)
Pass 4: Lint (surface rules)             ~2ms
                                   Total: ~8ms
```

**Pass 2 — Resolve:**
- Constrói tabela de símbolos: entity names, field names+types, page routes, API prefixes
- Resolve todas as referências: `bind Entity`, `-> Entity`, `field ... in Entity.field`
- Detecta: entity não encontrada, campo não existe, tipo incompatível

**Pass 3 — Check:**
- Verifica transitions: todos os enum values têm pelo menos uma transição
- Verifica contracts: campos referenciados existem, tipos compatíveis
- Verifica scopes: pages/APIs não acessam entities fora do scope
- Verifica constraints: campos em `when`/`require` existem

**Impacto em performance:** +5ms no build. Zero no runtime (exceto constraint/transition checks que são O(1)).

---

## 4. Como Isso Muda o Código

### Entity ANTES:
```cronus
entity Deployment shared {
  deploy_id string required
  service string required
  status enum ["Live", "Rolling", "Failed", "Pending"] required
  deployed_by string
}

webhook /deployments {
  on create -> POST "https://hooks.slack.com/..."
}
```

### Entity DEPOIS (spec-driven):
```cronus
/// Tracks deployment pipeline across all clusters.
/// @owner sre-team
entity Deployment shared {
  deploy_id   string!
  service     string! min:1 max:100
  cluster     string!
  status      enum ["Pending", "Rolling", "Live", "Failed"]! default:"Pending"
  duration    string default:"0s"
  deployed_by string!

  transition status {
    Pending -> Rolling
    Rolling -> Live | Failed
    Failed  -> Pending
    Live    -> Rolling
  }

  effect on_create {
    notify "slack" channel:"#deploys"
    audit
  }

  effect on_update status {
    when new.status == "Failed" {
      notify "pagerduty" severity:"critical"
    }
  }

  constraint "deployer-required" {
    require deployed_by != ""
  }
}
```

**Diferença:**
- Transitions previnem estados inválidos (antes: qualquer PATCH passava)
- Effects substituem webhooks manuais + são condicionais
- Constraints são compiláveis (antes: strings no constitution)
- `!` + defaults reduzem boilerplate
- TUDO no mesmo bloco (localidade de referência)

---

## 5. Impacto em Performance

| Feature | Compile-time | Runtime | Overhead |
|---------|-------------|---------|----------|
| `!` syntax | 0ms | 0ms | Zero (sugar) |
| `default` | 0ms | 0.001ms (set if null) | Negligível |
| `min/max/match` | Regex compile 1x | 0.01ms per check | Negligível |
| `transition` | Map build 1x | 0.001ms lookup | Negligível |
| `effect` | Validate refs | Async spawn | Zero (non-blocking) |
| `contract` (compile) | 1-2ms | 0ms | Zero runtime |
| `contract` (runtime) | 0ms | 1 SQL query | Aceável |
| `scope` | 1ms | 0ms | Zero runtime |
| **Semantic analysis** | **+5ms total** | **0ms** | **Zero runtime overhead** |

**A spec vive no código e custa <10ms no compile. Zero no runtime para features estáticas.**

---

## 6. O Que Isso Significa Para IA

### IA gerando .cronus com spec-driven constructs:

```
# Prompt: "Create a task management app with projects and tasks"

# IA gera:
entity Project {
  name        string! min:1 max:200
  status      enum ["active", "archived"]! default:"active"
  
  transition status {
    active   -> archived
    archived -> active
  }
}

entity Task {
  title       string! min:1
  priority    enum ["low", "medium", "high"]! default:"medium"
  status      enum ["todo", "in_progress", "done"]! default:"todo"
  project     -> Project
  assigned_to -> User

  transition status {
    todo        -> in_progress
    in_progress -> done | todo
    done        -> todo
  }

  effect on_update status {
    when new.status == "done" {
      notify "slack" message:"Task '{{title}}' completed"
    }
  }

  constraint "assigned-before-progress" {
    when status == "in_progress"
    require assigned_to != null
  }
}
```

**Por que é melhor para IA:**
1. **Transitions são enumeráveis** — IA não pode gerar transição inválida (compilador rejeita)
2. **Constraints são declarativas** — IA não precisa inventar JS de validação
3. **Effects são pipelines** — IA não precisa escrever fetch() imperativo
4. **Tudo no entity block** — localidade de referência (IA não precisa de contexto global)
5. **Vocabulário fechado** — `notify`, `audit`, `cascade`, `log` são keywords, não strings livres

---

## 7. Implementation Roadmap

| Phase | O que | Esforço | Breaking? |
|-------|-------|---------|-----------|
| 1 | `!` + `default:` + `min/max/match` no parser | 1 semana | Não |
| 2 | `transition` block no parser + runtime enforcement | 1 semana | Não |
| 3 | `effect` block (substitui webhook, adiciona notify/log) | 2 semanas | Não (webhook continua) |
| 4 | `constraint` block (cross-field validation) | 1 semana | Não |
| 5 | `contract` (compile-time verification) | 1 semana | Não |
| 6 | `scope` (access isolation) | 1 semana | Não |
| 7 | Semantic analysis pipeline (4 passes) | 2 semanas | Não |

**Total: ~9 semanas. Zero breaking changes. Tudo additive.**

---

## 8. O Que NÃO Fazer

| Sugestão do ChatGPT | Por que NÃO para CRONUS |
|---------------------|------------------------|
| Ownership/lifetime | CRONUS gera SQL, não gerencia memória |
| Bytecode/IR lowering | Overengineering — .cronus compila direto para HTTP handler |
| Capability system | Auth + roles + scopes é suficiente para web apps |
| Reflection | DSL declarativa não tem execução dinâmica |
| Custom scheduler | Tokio no Rust já resolve. .cronus não precisa expor |
| Effect algebras | Teoria demais, prática de menos. Pipeline simples é melhor |
| Execution boundaries | `scope` block cobre isso de forma mais simples |

**Princípio:** CRONUS é uma DSL que gera apps, não uma linguagem general-purpose. Cada feature adicionada deve justificar seu custo em complexidade. Se o Rust já resolve no runtime, o .cronus não precisa reinventar.

---

## 9. Success Metrics

1. **Entity com transitions rejeita PATCH inválido** — testável com curl
2. **Constraints validam antes de INSERT** — testável com dados inválidos
3. **Effects disparam automaticamente** — verificável no audit trail
4. **Contracts detectam inconsistências no build** — `cronus build` mostra erro
5. **Scopes isolam acesso no compile-time** — page fora do scope = build error
6. **Semantic analysis em <10ms** — benchmark no CI
7. **Zero runtime overhead para features estáticas** — benchmark request latency
8. **IA gera .cronus válido com transitions/effects** — medir lint pass rate
