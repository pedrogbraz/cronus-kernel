# SDD — CRONUS Spec-Driven Language v2 (Deep Review)

> Revisão profunda do SDD v1 com olhar de engenheiro que vai mudar uma geração.
> Nada pode ser destruído. Tudo deve ser adicionado com propósito.
> Author: Zedd + Claude | Date: 2026-04-02 | Status: PROPOSED (refined)

---

## 0. Princípio Fundamental

**CRONUS não é uma linguagem de programação. É um compilador de intenção.**

O desenvolvedor declara O QUE quer. O CRONUS decide COMO implementar.
Cada novo construct só entra se satisfaz TODOS estes critérios:

1. **Resolve um problema real** que existe hoje no Nova Core ou em apps reais
2. **Não pode ser resolvido** pelo que já existe (lint, constitution, etc.)
3. **Performance zero-cost** em produção (compile-time ou O(1) runtime)
4. **IA consegue gerar** sem ambiguidade (vocabulário fechado)
5. **Backward compatible** — apps existentes continuam funcionando
6. **Não é prematuro** — só implementar quando tiver 3+ casos de uso reais

---

## 1. Review Profundo de Cada Construct

### 1.1 `transition` — APROVADO (com ajustes)

**Problema real:** O Nova Core tem `status enum ["Pending", "Rolling", "Live", "Failed"]`. Nada impede um PATCH de `Live → Pending` que não faz sentido no domínio.

**Análise:**
- É um state machine — conceito bem estabelecido em engenharia de software
- Prisma não tem isso. Django não tem isso. Rails não tem isso nativamente
- **CRONUS seria a primeira DSL web que traz state machines como primitiva**
- Custo: 1 HashMap<(String,String), bool> por entity com transitions
- Compile-time: O(E × T) onde E = enum values, T = transitions. Microsegundos.
- Runtime: 1 lookup por PATCH em campo com transition. ~0.001ms

**Ajuste necessário:**
- Não exigir que TODOS os enum values tenham transição — alguns são terminais (ex: "archived")
- Adicionar `terminal` keyword: `terminal Archived` = não pode sair desse estado
- Adicionar `initial` keyword: `initial Pending` = estado padrão em INSERT

**Sintaxe refinada:**
```cronus
entity Order {
  status enum ["draft", "pending", "paid", "shipped", "delivered", "cancelled"]! {
    initial draft
    terminal delivered, cancelled
    
    draft     -> pending
    pending   -> paid | cancelled
    paid      -> shipped | cancelled
    shipped   -> delivered
  }
}
```

**Por que isso muda o jogo:**
- Qualquer app com workflow (e-commerce, SaaS, ticketing) precisa disso
- Hoje desenvolvedores implementam isso em middleware, validators, ou business logic
- CRONUS faz em 5 linhas e o compilador garante que é correto
- IA gera transitions corretas porque o compilador rejeita as incorretas

**Veredicto: IMPLEMENTAR (Phase 1)**

---

### 1.2 `constraint` inline (min/max/match/default) — APROVADO

**Problema real:** Entity fields não têm validação. `age number` aceita -999 ou 99999999.

**Análise:**
- Prisma tem `@default()`. SQL tem `CHECK`. GraphQL tem directives.
- CRONUS não tem NENHUMA validação de campo além de `required`
- Isso significa que toda validação acontece no frontend (JS) ou não acontece

**O que existe vs o que falta:**

```
EXISTE:    required, unique, sensitive
FALTA:     default, min, max, match (regex), minlength, maxlength
```

**Ajuste:** NÃO criar um `constraint` block separado no v1. Manter inline:

```cronus
entity Product {
  name        string! min:1 max:200
  price       money! min:0
  sku         string! match:"^[A-Z]{2}-[0-9]{4}$"
  stock       number default:0 min:0
  category    string! default:"general"
}
```

**Cross-field constraints** (o `constraint "name" { when ... require ... }`) é poderoso MAS:
- Adiciona complexidade na gramática
- Poucos apps reais precisam disso no v1
- Pode ser implementado via constitution rules pattern-matched (já existe)

**Decisão:** 
- Inline constraints (min/max/match/default): **IMPLEMENTAR (Phase 1)**
- Cross-field constraint blocks: **ADIAR para v2** — resolver via constitution enforcement

---

### 1.3 `effect` — APROVADO COM CAUTELA

**Problema real:** Webhooks são o único mecanismo de side-effects. E são limitados (só POST para URL).

**Análise honesta:**
- Effects são poderosos MAS adicionam complexidade significativa
- `notify "slack"` exige um sistema de providers (config de slack, pagerduty, etc.)
- `validate service exists_in Endpoint.path` é um cross-entity check — complexo
- `cascade` em deletes é perigoso (cascade em produção é como rm -rf)

**O que realmente precisa existir vs o que é prematuro:**

| Effect | Precisa? | Justificativa |
|--------|----------|---------------|
| `audit` | **Já existe** (automático) | Manter implícito, não precisa declarar |
| `notify "provider"` | **Sim** | Evolução natural dos webhooks |
| `log "message"` | **Sim** | Brain já faz tracking, log explícito é útil |
| `cascade Entity` | **Perigoso** | Deletes em cascata devem ser muito explícitos |
| `validate field exists_in` | **Prematuro** | Cross-entity validation é complexo, adirar |
| `when old/new` | **Sim** | Condicional é essencial para effects úteis |

**Sintaxe refinada (simplificada):**
```cronus
entity Deployment shared {
  # ... fields ...

  on create {
    notify "slack" "#deploys" "Deploy {{deploy_id}} started"
  }

  on update status {
    when "Failed" {
      notify "pagerduty" "critical" "Deploy {{deploy_id}} FAILED"
    }
  }

  on delete {
    log "Deploy {{deploy_id}} removed"
  }
}
```

**Mudança de nome:** `effect` → `on` (mais natural, menos acadêmico)

**Providers configurados no app block:**
```cronus
app "Nova Core" {
  # ... existing config ...
  
  providers {
    slack webhook:"https://hooks.slack.com/services/..."
    pagerduty webhook:"https://events.pagerduty.com/..."
    email smtp:"smtp://..."
  }
}
```

**Veredicto: IMPLEMENTAR `on` SIMPLES (Phase 2). Adiar `cascade` e `validate cross-entity`.**

---

### 1.4 `contract` — PARCIALMENTE APROVADO

**Problema real:** O compilador não verifica que um `bind Deployment` numa page realmente referencia uma entity que existe.

**Análise honesta:**

Contracts **implícitos** (inferidos pelo compilador) são valiosos:
- `bind Entity` → compilador verifica que Entity existe
- `-> Entity` em field → compilador verifica referência
- `columns "Name, Email"` → compilador verifica que campos existem na entity bound

Contracts **explícitos** (`contract "name" { require ... }`) são:
- Complexos de implementar
- Difíceis para IA gerar corretamente
- Cross-entity requires exigem SQL queries no compile time (sem DB!)
- Poucos apps reais precisam no v1

**Decisão:**
- Contracts implícitos: **IMPLEMENTAR na semantic analysis (Phase 3)**
  - Verificar bind references
  - Verificar field references em columns
  - Verificar relation targets
- Contracts explícitos: **ADIAR para v2**

---

### 1.5 `scope` — ADIADO

**Análise honesta:**
- O conceito é bom mas resolve um problema que POUCOS apps têm
- Apps pequenos/médios (80% do target CRONUS) têm 3-10 entities — tudo no mesmo scope
- O sistema de `requires:auth` + `auth:jwt` + roles já cobre segurança
- Scope adiciona complexidade na gramática sem ROI claro

**Decisão: ADIAR para v2.** Reavaliar quando apps CRONUS tiverem 20+ entities.

---

### 1.6 Semantic Analysis Pipeline — APROVADO

**Análise:**
- Hoje: Parse → Lint (2 passes)
- Proposto: Parse → Resolve → Check → Lint (4 passes)
- O Resolve pass (tabela de símbolos) é essencial para QUALQUER feature nova
- Sem resolve, não dá pra verificar que `bind Deployment` existe
- +5ms é aceitável (hoje build demora ~5ms, ficaria ~10ms)

**Decisão: IMPLEMENTAR (Phase 3) — pré-requisito para transitions, contracts implícitos, e tudo mais.**

---

## 2. O Que Realmente Muda Uma Geração

Os 5 constructs propostos são bons, mas não são suficientes para "dominar uma era". 
O que domina uma era é o MODELO MENTAL, não os features individuais.

### 2.1 O Modelo Mental do CRONUS

**Rails dominou porque:** "Convention over Configuration" — um modelo mental.
**React dominou porque:** "UI = f(state)" — um modelo mental.
**Docker dominou porque:** "Build once, run anywhere" — um modelo mental.

**CRONUS deve dominar porque:**

> **"Declare once, get everything."**

Um arquivo `.cronus` declara a intenção. O compilador gera:
- ✅ Banco de dados (schema + migration + CRUD)
- ✅ API REST (endpoints + auth + validation)
- ✅ API GraphQL (schema + resolvers)
- ✅ UI (pages + components + SPA navigation)
- ✅ Docs (API reference + design system + graph)
- ✅ Auth (JWT + login/signup + roles)
- ✅ Security (CSP + rate limit + audit trail)
- ✅ Real-time (SSE + live reload)
- 🆕 State machines (transitions)
- 🆕 Validação (constraints inline)
- 🆕 Side-effects (on create/update/delete)
- 🆕 Verificação formal (semantic analysis)

**Nenhuma outra linguagem/framework faz TUDO isso de um arquivo.**

### 2.2 O Que Falta Para Dominar

O gap não é em features. É em **experiência do desenvolvedor (DX)**:

| Gap | Impacto | Solução |
|-----|---------|---------|
| **Onboarding** | Novo dev leva 30min+ para entender .cronus | `cronus init` interativo + tutorial inline |
| **Error messages** | Erros do parser são genéricos | Rust-quality errors com sugestões e spans |
| **LSP (Language Server)** | Sem autocomplete em editors | `cronus-lsp` para VSCode/Neovim |
| **Playground** | Sem forma de experimentar online | `cronus playground` web-based |
| **Ecosystem** | Sem plugins/extensões | `cronus install plugin-name` |

### 2.3 Priorização Final

**O que implementar AGORA (muda o jogo imediatamente):**

| Prioridade | Feature | Por que muda o jogo |
|-----------|---------|---------------------|
| **P0** | Inline constraints (min/max/match/default) | Zero apps web existem sem validação |
| **P0** | `!` syntax para required | -30% tokens, zero ambiguidade |
| **P1** | `transition` blocks | **Primeira DSL web com state machines** |
| **P1** | Semantic analysis (resolve pass) | Pré-requisito para tudo que vem depois |
| **P2** | `on create/update/delete` effects | Substitui webhooks + notificações |
| **P2** | Auth first-class (do SDD anterior) | -95% HTML de login/signup |
| **P3** | Contracts implícitos | Verificação automática de referências |
| **P3** | Error messages com spans | DX que compete com Rust/Elm |

**O que ADIAR (bom mas prematuro):**

| Feature | Por que adiar |
|---------|---------------|
| Cross-field constraints | Complexidade alta, poucos casos de uso reais |
| `scope` blocks | Apps pequenos não precisam |
| `cascade` deletes | Perigoso, precisa de UX muito cuidadosa |
| `validate exists_in` | Cross-entity query sem DB é impossível em compile-time |
| Explicit contracts | Poucos apps reais precisam no v1 |
| Effect algebras | Over-engineering acadêmico |
| Plugins/ecosystem | Prematuro antes de ter 100+ users |

---

## 3. A Mudança Técnica Mais Profunda

### 3.1 De "String-Based Rules" para "Semantic-Based Rules"

**Hoje a constitution é:**
```cronus
constitution {
  must "prices in centavos — use formatPrice()"
  must "all data sections require bind"
  never "expose passwords in API responses"
}
```

**Isso é documentation, não code.** O compilador faz pattern-matching em strings.

**A mudança profunda:**
```cronus
constitution {
  # Compilável — o compilador ENTENDE isso
  must every field(type:money) uses format:centavos
  must every section(type:data) has bind
  never field(sensitive:true) in render_output
  never function("location.reload") in generated_code
  
  # Business rules — domínio específico
  must entity(Deployment).status has transition
  must api(method:POST) requires auth:jwt
}
```

**A diferença:** `must "string"` é grep. `must every field(type:money)` é uma QUERY no AST.

**Isso é o que muda uma geração:**
- Constitution rules são executáveis, não decorativas
- O compilador PROVA que o app satisfaz as regras
- IA pode VERIFICAR seu output contra a constitution antes de entregar
- É como type checking, mas para regras de negócio

### 3.2 Implementação Realista

Essa constitution semântica NÃO precisa de uma linguagem de query complexa. Pode ser implementada como:

```rust
// No compilador, cada rule é uma função:
fn check_must_rule(rule: &ConstitutionRule, ast: &[AstNode]) -> Vec<Violation> {
    match rule.pattern {
        "every field(type:money) uses format:centavos" => {
            // Verifica que todo campo money tem format constraint
        }
        "every section(type:data) has bind" => {
            // Já existe como lint C003
        }
        // ... pattern matching para N regras conhecidas
    }
}
```

**Ou seja:** não é uma linguagem de query genérica. São **patterns pré-compilados** que o compilador reconhece. Vocabulário fechado, extensível pela Cooud.

---

## 4. Roadmap Revisado

### Phase 1: Foundations (2 semanas)
- `!` syntax no parser (backward compat com `required`)
- `default:"value"` inline modifier
- `min:N max:N` inline constraints
- `match:"regex"` inline constraint
- Runtime enforcement em INSERT/UPDATE (422 on violation)
- 20+ testes

### Phase 2: State Machines (2 semanas)
- `transition` block parsing (dentro de entity)
- `initial` e `terminal` keywords
- Transition map build em compile-time
- Runtime enforcement em PATCH (409 on invalid transition)
- Transition graph in /docs auto-generated
- 15+ testes

### Phase 3: Semantic Analysis (2 semanas)
- Symbol table (entity names, field names+types, page routes, API prefixes)
- Reference resolution (bind targets, -> targets, column names)
- Implicit contract checking (bind references valid entity, columns match fields)
- Error messages com span (line number + suggestion)
- 20+ testes

### Phase 4: Effects (2 semanas)
- `on create/update/delete` block parsing (dentro de entity)
- `when` conditional (acessa old/new values)
- `notify "provider"` action (usa providers do app block)
- `log "message"` action (escrito no brain)
- Provider config no app block
- Async execution (tokio::spawn)
- 15+ testes

### Phase 5: Constitution Semântica (1 semana)
- Substituir pattern-matching de strings por rules compiláveis
- 10 rules pré-compilados que o compilador reconhece
- Incluir resultado no /api/_context
- 10+ testes

**Total: 9 semanas. Zero breaking changes. ~80+ novos testes.**

---

## 5. Critérios de Sucesso — Como Saber Que Mudamos Uma Geração

1. **Um dev cria um app CRUD completo com state machines em 30 linhas** — impossível em qualquer outro framework
2. **`cronus build` detecta 95% dos bugs antes de rodar** — compile-time verification
3. **IA gera app válido em 1 shot com 95%+ success rate** — vocabulário fechado + compilador rigoroso
4. **0 linhas de JavaScript escritas pelo dev** — tudo declarativo
5. **Transition violation retorna erro claro** — UX de erro melhor que qualquer API framework
6. **Constitution rules são COMPILÁVEIS, não strings** — a spec É o código
7. **De ideia a app em produção em 5 minutos** — `cronus generate "..." → cronus run`
