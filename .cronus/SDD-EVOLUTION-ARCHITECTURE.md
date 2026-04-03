# SDD — CRONUS Evolution: Architecture, Security & Code Quality

> A linguagem que GARANTE que regras não são quebradas — por design, não por disciplina.
> Author: Zedd + Claude | Date: 2026-04-02 | Status: PROPOSED

---

## 0. Princípio Central

**Regras não são quebradas porque não podem ser representadas no código.**

Não é "o lint avisa". Não é "a documentação diz pra não fazer". É "o compilador se recusa a gerar código que viola a regra". Como o borrow checker do Rust não "avisa" sobre use-after-free — ele torna impossível.

---

## 1. Estado Atual — O Que Já Garante

### 1.1 Regras Enforced (impossíveis de violar)

| Regra | Como é garantida | Nível |
|-------|-----------------|-------|
| SQL injection via identificadores | Parser rejeita chars inválidos (P040/P041) | **Parser** |
| Sensitive field no HTML | Lint ERROR (C002) bloqueia build | **Lint** |
| Sensitive field em SELECT | Lint ERROR (C031) bloqueia build | **Lint** |
| Hardcoded metrics em data sections | Lint ERROR (C001) bloqueia build | **Lint** |
| Data section sem bind | Lint ERROR (C003) bloqueia build | **Lint** |
| href morto (href="#") | Lint ERROR (C010) bloqueia build | **Lint** |
| location.reload() | Lint ERROR (C020) bloqueia build | **Lint** |
| Shared entity mutation sem auth | Lint ERROR (C030) bloqueia build | **Lint** |
| State transition inválida | Runtime 409 com transições permitidas | **Runtime** |
| Field constraint violada | Runtime 422 com mensagem clara | **Runtime** |
| Referência a entity inexistente | Resolve pass fatal error | **Semantic** |
| Coluna referencia campo inexistente | Resolve pass fatal error | **Semantic** |

### 1.2 Regras NÃO Garantidas (dependem de disciplina)

| Regra | Problema | Risco |
|-------|----------|-------|
| SQL injection via VALUES | Parameterized queries — mas não verificado estaticamente | Baixo (código correto) |
| XSS via template interpolation | html_escape() existe mas é opt-in em templates | **ALTO** |
| Race conditions em mutations | Mutex global serializa mas sem transações por request | Médio |
| Audit trail completude | Log é post-hoc, não transacional | Médio |
| Memory safety do Rust code | Rust garante, mas `unsafe` poderia ser adicionado | Baixo |
| Backward compat em migrations | Zero versionamento de schema | **ALTO** |
| Webhook delivery guarantee | Fire-and-forget, sem retry | Médio |
| Token expiry/rotation | JWT 7 dias, sem refresh token | Médio |

---

## 2. Evolução da Arquitetura — 5 Pilares

### Pilar 1: SAFETY BY CONSTRUCTION

**Objetivo:** Tornar mais categorias de bugs impossíveis no nível do compilador.

#### 1.1 Template Escaping Obrigatório

**Problema atual:** `{{field}}` em templates faz string interpolation sem escape. Se field contém `<script>alert('xss')</script>`, é XSS.

**Solução:**

```rust
// No template renderer, TODA interpolação passa por escape
fn interpolate_template(template: &str, data: &Value) -> String {
    // {{field}} → html_escape(value)  SEMPRE
    // {{{field}}} → raw value (triple braces = opt-in unsafe)
    // {{field|raw}} → raw value (pipe raw = opt-in unsafe)
}
```

**Regra:** `{{}}` SEMPRE escapa. `{{{}}}` ou `{{|raw}}` para raw (e o lint avisa).

**Enforcement:** Não é lint — é design do template engine. Impossível fazer XSS com `{{}}`.

#### 1.2 Transação Atômica por Request

**Problema atual:** Um POST que faz INSERT + audit_log pode falhar no audit_log após o INSERT ter commitado.

**Solução:**

```rust
// Em handle_request, TODA mutation é transacional:
async fn handle_mutation(db: &CronusDB, ...) -> Result<Response> {
    db.transaction(|conn| {
        let row = insert_record(conn, ...)?;
        insert_audit_log(conn, ...)?;
        fire_effects_sync(conn, ...)?;  // efeitos síncronos dentro da tx
        Ok(row)
    })?;
    
    // Efeitos assíncronos APÓS commit (webhooks, notify)
    tokio::spawn(fire_effects_async(...));
    
    Ok(response)
}
```

**Garantia:** Se qualquer step falha, TUDO faz rollback. Audit trail NUNCA tem holes.

#### 1.3 Immutable Audit Log

**Problema atual:** `_audit_log` é uma tabela SQLite normal. Pode ser deletada/alterada.

**Solução:**

```sql
-- Trigger que impede DELETE e UPDATE em _audit_log
CREATE TRIGGER IF NOT EXISTS audit_immutable_delete
BEFORE DELETE ON _audit_log
BEGIN
    SELECT RAISE(ABORT, 'Audit log entries cannot be deleted');
END;

CREATE TRIGGER IF NOT EXISTS audit_immutable_update
BEFORE UPDATE ON _audit_log
BEGIN
    SELECT RAISE(ABORT, 'Audit log entries cannot be modified');
END;
```

**Garantia:** Mesmo com SQL raw, não dá pra deletar/alterar audit entries.

### Pilar 2: SECURITY HARDENING

#### 2.1 Rate Limiter Proper (429, não TCP drop)

**Status:** Já corrigido — retorna 429 com Retry-After header.

#### 2.2 CORS Strict por Default

**Problema atual:** `CRONUS_CORS_ORIGIN` env var ou `same-origin` default. Mas em dev, `*` é comum.

**Solução:**

```cronus
app "My App" {
  security {
    cors ["https://myapp.com", "https://staging.myapp.com"]
    # cors * → compile warning: "Wildcard CORS is insecure"
  }
}
```

**Enforcement:** Se `cors *` em production mode, lint warning. Se `cors` não declarado, same-origin (seguro).

#### 2.3 Refresh Tokens

**Problema atual:** JWT 7 dias, sem refresh. Token roubado = acesso por 7 dias.

**Solução:**

```
POST /api/auth/login → { access_token (15min), refresh_token (7 days, HttpOnly) }
POST /api/auth/refresh → new access_token (se refresh_token válido)
POST /api/auth/logout → revoke refresh_token
```

**Implementação:** refresh_token em tabela `_refresh_tokens` com revogação.

#### 2.4 Input Sanitization Pipeline

**Problema atual:** Constraints (min/max/match) validam formato mas não sanitizam conteúdo.

**Solução:** Pipeline de sanitização automática:

```
INPUT → trim whitespace → normalize unicode → check constraints → html_escape → store
```

Nunca armazena input bruto. `<script>` vira `&lt;script&gt;` ANTES do INSERT.

### Pilar 3: CODE QUALITY ENFORCEMENT

#### 3.1 Compilador Multi-Pass Completo

**Atual:** Parse → Resolve → Lint → Constitution (4 passes)

**Evolução:**

```
Pass 1: PARSE         .cronus → AST                    ~3ms
Pass 2: RESOLVE       Tabela de símbolos + refs         ~1ms
Pass 3: TYPE CHECK    Tipos compatíveis em bind/filter  ~1ms  ← NOVO
Pass 4: FLOW          Template escape tracking          ~1ms  ← NOVO
Pass 5: LINT          Surface rules (13 regras)         ~2ms
Pass 6: CONSTITUTION  Business rules                    ~1ms
Pass 7: CODEGEN       AST → HTML + SQL + JS             ~5ms  ← NOVO (dry-run)
                                                  Total: ~15ms
```

**Pass 3 — Type Check:**
- `bind Task { where priority == 5 }` → priority é string, não number → tipo incompatível
- `order price desc` → price é money (INTEGER) → OK
- `columns "Name, Age"` → Name é string, Age é number → OK, mas formatar diferente

**Pass 4 — Flow Analysis:**
- Rastrear se `{{field}}` passa por escape antes de render
- Detectar se sensitive field é acessado em path que leva a HTML output
- Verificar se auth check acontece antes de data access em cada request path

#### 3.2 Error Messages com Spans (tipo Rust/Elm)

**Problema atual:** Erros são strings genéricas sem posição exata.

**Solução:**

```
error[C002]: sensitive field 'password' exposed in template
  --> app.cronus:42:15
   |
42 |     columns "Name, Email, Password, Role"
   |                          ^^^^^^^^
   |
   = field 'password' in entity User is marked 'sensitive'
   = sensitive fields can never appear in rendered output
   |
   = help: remove 'Password' from the columns list
```

**Implementação:**
- Cada Token no parser já tem `line: usize`. Adicionar `col: usize` e `span: (start, end)`.
- Cada erro retorna `Span` que aponta pro texto exato.
- Formatter gera output igual ao rustc.

#### 3.3 Deterministic Codegen

**Problema atual:** Código gerado (HTML/JS) pode variar entre builds (IDs aleatórios, timestamps).

**Solução:** Todo output determinístico:
- IDs de sections: hash do content, não random
- Nonces CSP: random (necessário para segurança) mas documentado
- Tailwind classes: ordenadas alfabeticamente
- Audit: timestamp do request, não do build

**Garantia:** Mesmo input `.cronus` → mesmo output HTML (exceto nonces).

### Pilar 4: SCHEMA EVOLUTION

#### 4.1 Migration Automática

**Problema atual:** Mudou o .cronus → recompilou → schema antigo no SQLite. Sem ALTER TABLE automático, sem rollback.

**Solução:**

```
$ cronus migrate

  Analyzing schema changes...

  Entity User:
    + field 'avatar' (url, optional)       → ALTER TABLE User ADD COLUMN avatar TEXT
    ~ field 'role' default changed          → UPDATE pragmas
    - field 'phone' removed                 → Column preserved (soft remove)

  Entity Product (NEW):
    → CREATE TABLE Product (...)

  Apply? [y/N]
```

**Implementação:**
- `cronus migrate` compara AST atual com `ast-snapshot.json`
- Gera SQL de migração (ALTER TABLE ADD COLUMN, CREATE TABLE)
- Nunca faz DROP COLUMN (soft remove — coluna fica, não é usada)
- Reversível: `cronus migrate --rollback` (se snapshot anterior existe)

#### 4.2 Schema Versioning

```cronus
app "My App" {
  version "2.1.0"  # Semver — mudanças no schema incrementam minor
}
```

Cada build salva a versão em `_schema_versions` table. Migrations são rastreáveis.

### Pilar 5: TESTING BUILT-IN

#### 5.1 `cronus test` — Testes Declarativos

**Problema atual:** `cronus test` gera CRUD tests automáticos mas são básicos.

**Evolução:**

```cronus
test "deployment lifecycle" {
  # Setup
  create Deployment {
    deploy_id: "DPL-TEST"
    service: "api-gateway"
    cluster: "us-east-1"
    status: "Pending"
  }
  
  # Valid transition
  update Deployment.status -> "Rolling"
  assert status == "Rolling"
  
  # Invalid transition
  update Deployment.status -> "Delivered"
  assert error 409
  assert error.message contains "Invalid transition"
  
  # Valid to terminal
  update Deployment.status -> "Live"
  assert status == "Live"
  
  # Cleanup
  delete Deployment where deploy_id == "DPL-TEST"
}
```

**Implementação:** Test blocks no parser → test runner que executa sequencialmente contra o DB.

#### 5.2 `cronus bench` — Performance Benchmark

```bash
$ cronus bench

  CRONUS Benchmark — Nova Core

  Build time:        12ms  (parse 3ms + resolve 1ms + lint 2ms + codegen 5ms)
  Startup time:      45ms  (DB open + migration + brain init)
  Cold request:      14ms  (first page render after start)
  Warm request:       3ms  (cached page render)
  API latency:        2ms  (simple SELECT)
  Complex query:     12ms  (JOIN + filter + sort + paginate)
  Throughput:       450 req/s  (100 concurrent connections)
  Memory:            28MB  (idle after startup)
  Binary size:      7.2MB  (release build)
```

---

## 3. Código — Standards de Qualidade

### 3.1 Regras de Código para o Kernel Rust

```
1. ZERO unsafe — se precisar, justificar em comentário + issue
2. ZERO unwrap() em paths de request — sempre handle errors
3. ZERO panic! em runtime — catch_unwind no handler
4. Cada módulo tem testes — mínimo 5 por módulo
5. Cada feature tem SDD — sem código sem plano
6. Cada commit tem 'cargo test' verde — CI obrigatório
7. Cada novo field no AST atualiza TODOS os construtores — verificado pelo compiler
8. Error messages incluem sugestão — nunca "error: failed"
9. Performance budget: build <20ms, request <10ms, startup <100ms
10. Backward compatibility obrigatória — apps existentes NUNCA quebram
```

### 3.2 Regras de Código para a Linguagem .cronus

```
1. ZERO ambiguidade sintática — cada construct tem 1 forma
2. ZERO magic — tudo que o kernel faz está declarado ou documentado
3. ZERO hidden state — se existe, aparece no /api/_context
4. ZERO breaking changes — nova syntax é additive, nunca remove
5. Cada keyword tem propósito único — sem overload semântico
6. Defaults são seguros — opt-out de segurança, nunca opt-in
7. Templates são escaped por default — raw é opt-in explícito
8. Auth é fail-closed — sem auth = sem acesso (não o contrário)
9. Errors são structured — humano e máquina entendem
10. A spec vive no código — não em docs externas
```

### 3.3 Arquitetura de Módulos

```
src/
├── CORE (não muda frequentemente)
│   ├── parser.rs         # Gramática — mudanças precisam de RFC
│   ├── database.rs       # SQLite wrapper — performance-critical
│   ├── auth.rs           # JWT + Argon2id — security-critical
│   └── security.rs       # Rate limit, CSP, headers
│
├── ANALYSIS (evolui com features)
│   ├── resolve.rs        # Tabela de símbolos + referências
│   ├── lint.rs           # 20 regras de enforcement
│   ├── constitution_check.rs  # Business rules
│   └── contracts.rs      # Section validation
│
├── GENERATION (output)
│   ├── ui.rs             # SSR renderers (12K linhas — candidato a split)
│   ├── render.rs         # SPA runtime + debug overlay
│   ├── export.rs         # JSON/TS/SQL/OpenAPI
│   └── deploy.rs         # Dockerfile, fly.toml
│
├── INTELLIGENCE
│   ├── brain.rs          # Pattern learning
│   ├── memory.rs         # Semantic memory
│   ├── ast_diff.rs       # Change tracking
│   └── graph.rs          # Relationship graph
│
├── INFRASTRUCTURE
│   ├── audit.rs          # Hash-chained audit trail
│   ├── sse.rs            # Server-Sent Events
│   ├── rate_limit.rs     # Token bucket
│   └── cache.rs          # LRU cache
│
└── FUTURE (roadmap)
    ├── migrate.rs        # Schema evolution     ← NOVO
    ├── test_runner.rs    # Declarative tests    ← NOVO
    ├── type_check.rs     # Type compatibility   ← NOVO
    └── flow.rs           # Escape flow analysis ← NOVO
```

---

## 4. Roadmap de Evolução

### Q2 2026 (Já feito + próximo mês)

| Item | Status | Impacto |
|------|--------|---------|
| 13 lint rules | ✅ DONE | Impossível gerar código inseguro |
| Resolve pass | ✅ DONE | Referências validadas |
| Transitions | ✅ DONE | State machines nativas |
| Constraints | ✅ DONE | Validação declarativa |
| Effects | ✅ DONE | Side-effects declarativos |
| AI-Error Protocol | ✅ DONE | IA corrige erros sozinha |
| Templates escaping | 🔜 NEXT | XSS impossível por design |
| Transação atômica | 🔜 NEXT | Audit trail sem holes |
| Immutable audit | 🔜 NEXT | Compliance real |
| Error spans | 🔜 NEXT | DX nível Rust |

### Q3 2026 (3 meses)

| Item | Impacto |
|------|---------|
| Type check pass | Tipos incompatíveis em bind = compile error |
| Flow analysis | Escape tracking automático |
| Schema migration | ALTER TABLE automático |
| Refresh tokens | Segurança de auth real |
| `cronus test` declarativo | Testing built-in |
| Postgres support | Enterprise-ready |

### Q4 2026 (6 meses)

| Item | Impacto |
|------|---------|
| LSP (Language Server) | Autocomplete em VSCode/Neovim |
| Playground web | Experimenta sem instalar |
| Plugin system (cuidadoso) | Extensibilidade governada |
| `cronus bench` | Performance tracking |
| Multi-tenancy | SaaS real |
| File uploads (S3) | Apps completos |

---

## 5. Métricas de Qualidade

### Build Deve Passar TODOS:

```
✓ Parse válido (0 syntax errors)
✓ Resolve válido (0 reference errors)
✓ Type check válido (0 type errors)           ← FUTURO
✓ Flow analysis limpo (0 escape gaps)         ← FUTURO
✓ Lint passed (0 errors, N warnings)
✓ Constitution passed (0 violations)
✓ Hardcode lint clean (0 false data)
✓ Tests green (N/N passed)
✓ Performance budget met (build <20ms)
✓ AST snapshot saved
```

### Cada Request Deve:

```
✓ Transação atômica (INSERT + audit em 1 tx)
✓ Input sanitizado (trim + escape)
✓ Auth verificado (JWT + role + scope)
✓ Rate limit checked (429 se excedido)
✓ Constraints validados (min/max/match)
✓ Transitions validadas (409 se inválida)
✓ Audit logged (hash chain)
✓ Effects dispatched (async)
✓ SSE broadcast (data_change)
✓ Performance headers (X-Response-Time)
```

---

## 6. O Que NUNCA Fazer

| Tentação | Por que destruiria o CRONUS |
|----------|---------------------------|
| Adicionar `unsafe` ao Rust | Perde a garantia de memory safety |
| Aceitar `Record<string, any>` | Buraco negro pra bugs + alucinação IA |
| Plugin system sem sandbox | Um plugin malicioso compromete tudo |
| Turing-complete scripting | Vira JavaScript. Perde declaratividade |
| `--force` flag que bypassa lint | Se existe escape, ninguém respeita a regra |
| Backward-incompatible syntax change | Apps existentes quebram = morte da linguagem |
| Dynamic require/import | Perde análise estática, impossível resolve |
| Implicit side effects | Cada mutation deve ser rastreável |
| Default inseguro | Segurança é opt-out, nunca opt-in |
| Código sem testes | 1 módulo sem teste = 1 fonte de regressão |
