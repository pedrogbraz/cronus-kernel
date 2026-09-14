# SDD — CRONUS Language Constitution

> A linguagem que se documenta, se protege, e da memoria a IA.
> Author: Zedd + Claude | Date: 2026-04-02 | Status: PROPOSED

---

## Vision

CRONUS deve ser a linguagem onde:
1. **Regras sao inquebraveis** — estados invalidos sao irrepresentaveis no AST
2. **Codigo se documenta** — `///` doc-comments entram no AST e geram /docs automaticamente
3. **IA tem memoria perfeita** — `/api/_context` entrega tudo que a IA precisa em 1 chamada

---

## Part 1: Unbreakable Rules (13 regras)

### Filosofia
Inspirado no borrow checker do Rust: se a struct nao pode representar o estado invalido, ele nao existe. Regras nao sao lint warnings — sao erros fatais que impedem compilacao.

### Category 1: Data Integrity

| Code | Rule | Enforcement | Escape |
|------|------|-------------|--------|
| C001 | Dado numerico em section de dados sem `bind` | Parser classifica texto como Label ou SuspectFakeData. Compiler rejeita se sem bind | `static:true` |
| C002 | Campo `sensitive` nunca no HTML/API response | Compiler verifica todas refs. Runtime strip no SELECT | NENHUM |
| C003 | Section de dados (kpi/table/chart) DEVE ter source | AST: `DataSection.source: DataSource` (nao-Optional) | NENHUM |

### Category 2: Connectivity

| Code | Rule | Enforcement | Escape |
|------|------|-------------|--------|
| C010 | Todo `href` interno resolve para rota existente | Compiler constroi RouteTable e valida. Sugere via Levenshtein | NENHUM (interno) |
| C011 | Todo `<button>` deve ter acao | AST: `CtaButton.action: ButtonAction` (nao-Optional) | NENHUM |
| C012 | Form deve ter handler de submit | Compiler verifica `on submit` ou action no escopo | NENHUM |

### Category 3: Realtime Contract

| Code | Rule | Enforcement | Escape |
|------|------|-------------|--------|
| C020 | Zero `location.reload()` em codigo gerado | Impossibilitado pela arquitetura do emitter + post-build verify | NENHUM |
| C021 | Toda mutacao gera soft-reload automatico | Compiler computa invalidacoes automaticamente | NENHUM |
| C022 | Links internos SEMPRE usam SPA navigation | Emitter gera cronusNavigate(), nunca `<a href>` para rota interna | NENHUM |

### Category 4: Security Invariants

| Code | Rule | Enforcement | Escape |
|------|------|-------------|--------|
| C030 | Mutacao em entity shared requer auth | Compiler rejeita `auth:public` em POST/PATCH/DELETE de shared | NENHUM |
| C031 | Sensitive nunca em SELECT/response | Gerador exclui automaticamente do SQL e do JSON | NENHUM |
| P040 | Identificadores SQL validados | Parser: apenas `[a-zA-Z][a-zA-Z0-9_]{0,63}` | NENHUM |
| P041 | Palavras reservadas SQL proibidas como nomes | Parser verifica contra lista (SELECT, DROP, etc.) | NENHUM |

### Hierarquia de Severidade

```
FATAL  (C0xx, P0xx) — Compilacao abortada. Irrecuperavel.
ERROR  (lint --strict) — Lint warnings promovidas. Build bloqueado.
WARNING (lint) — Mostra no terminal, nao bloqueia.
INFO   — Sugestoes. Nunca bloqueia.
```

---

## Part 2: Self-Documenting Code (`///` doc-comments)

### Sintaxe

```cronus
/// Rastreia deploys em producao e staging.
/// Usado pelo time de SRE para monitorar rollouts.
/// @owner sre-team
/// @lifecycle critico
entity Deployment shared {
  /// Identificador unico (formato: dep_xxxx)
  /// @example "dep_a8f3k2"
  deploy_id string required

  /// Estado do pipeline
  /// @business Quando "Failed", dispara PagerDuty
  status enum ["Live", "Rolling", "Failed"] required
}

/// API de deploys. Rate limit: 100/min.
/// @version v1
api /deployments {
  /// Lista deploys com paginacao.
  /// @param page int "Numero da pagina"
  /// @returns Deployment[] "Array paginado"
  list GET /deployments auth:jwt
}

/// Dashboard de deploys em tempo real.
/// @audience SRE, DevOps
/// @refresh 5s
page "/deployments" type:custom {
  /// Metricas de alto nivel
  section kpi cols:4 {
    bind Deployment { query all }
  }
}
```

### Tags Estruturadas

**Entities/Fields:**
`@owner`, `@lifecycle`, `@example`, `@default`, `@business`, `@format`, `@source`, `@metric`, `@deprecated`, `@see`

**APIs:**
`@param nome tipo "desc"`, `@body nome tipo "desc"`, `@returns Tipo "desc"`, `@example`, `@error code "desc"`, `@version`, `@rate-limit`

**Pages:**
`@audience`, `@refresh`, `@requires`, `@data-source`, `@interactive`

**App (global):**
`@team`, `@environment`, `@oncall`, `@repo`, `@status`, `@since`, `@rule`

**AI-specific:**
`@ai` — contexto livre que so a IA le (nao aparece na /docs publica)

### AST Changes

```rust
#[derive(Debug, Clone)]
pub struct DocComment {
    pub summary: String,
    pub description: String,
    pub tags: Vec<DocTag>,
}

#[derive(Debug, Clone)]
pub struct DocTag {
    pub name: String,
    pub value: String,
}
```

Nodes que recebem `doc: Option<DocComment>`:
AppNode, EntityNode, FieldNode, PageNode, SectionNode, ApiNode, RouteNode

### Tokenizer Change

```rust
// parser.rs — detectar /// antes de #
if i + 2 < chars.len() && chars[i] == '/' && chars[i+1] == '/' && chars[i+2] == '/' {
    let doc_text: String = chars[i+3..].iter().collect();
    tokens.push(Token {
        kind: TokenKind::DocComment,
        value: doc_text.trim().to_string(),
        line: current_line,
    });
    break;
}
```

### /docs Upgrade (Before → After)

**Before:** Apenas nomes de campos e tipos
**After:** Summary, description, @tags, exemplos, regras de negocio, params de API

---

## Part 3: AI Context Protocol (ACP)

### `GET /api/_context` — Tudo que a IA precisa em 1 chamada

```json
{
  "acp_version": "1.0.0",
  "project": { "name", "purpose", "category", "version" },
  "stack": { "frontend", "backend", "database", "port" },
  "entities": [{
    "name": "Deployment",
    "fields": [{ "name", "type", "required", "doc_comment" }],
    "relationships": { "belongs_to", "has_many", "referenced_by" },
    "row_count": 42,
    "api_endpoints": ["GET /api/deployments", "POST /api/deployments"]
  }],
  "pages": [{ "route", "type", "bound_entity", "sections", "requires_auth" }],
  "api_routes": [{ "method", "path", "entity", "auth" }],
  "auth": { "enabled", "entity", "provider", "roles", "protected_routes" },
  "constitution": {
    "invariants": ["prices in centavos", "never expose passwords"],
    "forbidden": ["no fake data", "no location.reload()"]
  },
  "memory": {
    "completed_tasks": [],
    "recent_changes": [],
    "known_issues": [],
    "anti_patterns": [],
    "decisions": [{ "date", "decision", "reason" }]
  },
  "relationship_graph": {
    "entity_relations": [{ "from", "to", "type", "field" }],
    "page_bindings": [{ "page", "entity", "type" }],
    "event_flows": [{ "trigger", "actions" }]
  },
  "health": {
    "build_status": "passing",
    "lint_violations": 0,
    "brain_suggestions": []
  }
}
```

### CLI: `cronus context`

```bash
cronus context              # JSON completo → stdout
cronus context --compact    # Sem whitespace (pipe)
cronus context --for-claude # Otimizado para system prompt
cronus context --section entities  # So entidades
```

### Semantic Memory: `.cronus/memory.db` (SQLite)

**Tabelas:**
- `sessions` — historico de sessoes AI (quando, quem, resultado)
- `decisions` — decisoes arquiteturais (o que, porque, quando)
- `anti_patterns` — o que falhou e como foi resolvido
- `business_rules` — regras extraidas de doc-comments
- `changes` — changelog AST-level (field added, entity removed, etc.)

### AST Change Tracking

```rust
pub enum AstChange {
    EntityAdded { name: String },
    EntityRemoved { name: String },
    FieldAdded { entity: String, field: String, field_type: String },
    FieldRemoved { entity: String, field: String },
    FieldModified { entity: String, field: String, what_changed: String },
    PageAdded { route: String },
    PageRemoved { route: String },
    ApiRouteAdded { method: String, path: String },
    StyleChanged { key: String, old_value: String, new_value: String },
}
```

### Relationship Graph

Auto-detectado de:
- `FieldNode.reference` → belongs_to
- Inversao automatica → has_many
- `SectionNode.binding` → page binds to entity
- `WebhookNode` → event flows

CLI: `cronus graph` → Mermaid diagram
Endpoint: `/docs/graph` → visual interativo

### Constitution Inline

```cronus
app "Nova Core" {
  stack fullstack
  port 5175

  constitution {
    must "prices in centavos — formatPrice()"
    must "all entities need created_at"
    never "expose passwords in API"
    never "use float for money"
    never "hardcode user data in templates"
  }
}
```

Parsed into AST → validado pelo compiler → incluido no ACP endpoint.

---

## Part 4: Implementation Roadmap

### Phase 1: Doc-Comments (1 semana)
1. Add `TokenKind::DocComment` ao lexer
2. Add `DocComment` struct e `doc: Option<DocComment>` a todos os nodes
3. Parser coleta `///` consecutivos e associa ao proximo bloco
4. `/docs` usa doc-comments para gerar documentacao rica
5. Nova Core: adicionar doc-comments ao app.cronus

### Phase 2: ACP Endpoint (1 semana)
1. Implementar `/api/_context` que retorna JSON completo
2. Implementar `cronus context` CLI
3. Incluir relationship graph auto-detectado
4. Incluir constitution rules no output

### Phase 3: Unbreakable Rules — Parser Level (1 semana)
1. P040/P041: validacao de identificadores SQL no parser
2. C001: classificacao de texto (Label vs SuspectFakeData)
3. C003: `DataSection.source` nao-Optional na AST
4. C011: `CtaButton.action` nao-Optional na AST

### Phase 4: Unbreakable Rules — Compiler Level (1 semana)
1. C002: SensitiveFieldSet + varredura de referencias
2. C010: RouteTable + validacao cruzada + Levenshtein suggestions
3. C012: Form submit handler verification
4. C030: shared entity auth validation
5. C031: sensitive exclusion in SELECT/response

### Phase 5: Semantic Memory (1 semana)
1. `.cronus/memory.db` SQLite schema
2. AST diff engine (`ast_diff.rs`)
3. `cronus changelog` CLI
4. Session tracking no `cronus handoff`
5. Memory no ACP endpoint

### Phase 6: Constitution Enforcement (1 semana)
1. Parser: bloco `constitution {}` no app
2. `constitution_check.rs` — valida regras contra AST
3. Integrar no `cronus validate` e `cronus build --strict`
4. AI: constitution rules injetadas no context

---

## Success Metrics

1. **Zero hardcoded data** pode existir em apps CRONUS — verificado por 13 regras fatais
2. **100% dos elementos** do .cronus aparecem na /docs — medido por cobertura
3. **1 chamada** (`/api/_context`) da a IA todo o contexto — medido por completude
4. **< 10ms** overhead total de todas as verificacoes — medido por benchmark
5. **AI working on CRONUS** nunca produz codigo invalido — medido por lint pass rate
