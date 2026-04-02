# CRONUS Kernel Brief

> Leia este arquivo ao iniciar qualquer sessao no cronus-kernel.
> Ultima atualizacao: 2026-04-02

## O que e o CRONUS Kernel

Compilador de linguagem `.cronus` → full-stack web app (HTML + API + SQLite). Um binario Rust, zero dependencias externas. 43 linhas de `.cronus` = CRUD completo com DB + API + UI + auth + seguranca.

## Estado Atual

- **Branch:** desenvolvimento
- **Ultimo commit:** `27ac601` — security hardening + data isolation
- **Build:** `cargo build` (3 warnings, 0 errors)
- **Testes:** 17/18 passam (1 pre-existente em dump detector)
- **App demo:** `/tmp/nova-core/` porta 5175 (precisa mover pra permanente)

## Arquitetura

```
src/
├── main.rs          # HTTP server (hyper), request routing, auth endpoints
├── parser.rs        # Tokenizer + parser (.cronus → AST)
├── ui.rs            # Renderers (KPI, table, chart, modal, page-header, etc.)
├── database.rs      # SQLite (rusqlite) — migration, CRUD, validation
├── binding.rs       # Data binding (section bind Entity → DB query)
├── auth.rs          # JWT signing/verification, password hashing
├── security.rs      # HTML escape, SQL identifier validation, rate limiter, secure cookies
├── data_table.rs    # Table renderer (light + dark, static + DB-driven)
├── theme.rs         # Design system token extraction
├── contracts.rs     # Section contract validation
├── hardcode_lint.rs # Build-time hardcode detection
├── sse.rs           # Server-Sent Events (infra pronta, nao integrada)
├── server.rs        # Alternative API handler
└── ...              # 20+ outros modulos
```

## Regras Inviolaveis

1. **ZERO hardcode** — nenhum dado visivel sem existir no DB
2. **ZERO dados fake** — DB comeca vazio, seed proibido
3. **Conta nova = zero dados** — `_owner_id` isola tudo
4. **HTML escape obrigatorio** — `security::html_escape()` em todo output
5. **SQL parameterizado** — nenhuma string interpolation
6. **Cookies seguros** — HttpOnly + SameSite=Strict + Secure
7. **Signup nunca e admin** — roles privilegiados bloqueados

## Seguranca Implementada

| Camada | Status |
|--------|--------|
| Input validation (entity types) | DONE |
| SQL parameterization | DONE |
| Authentication (JWT) | DONE |
| Authorization (_owner_id) | DONE |
| HTML escaping (XSS prevention) | DONE |
| Secure cookies + CORS | DONE |
| Security headers | DONE |
| Rate limiting (struct pronta) | PARTIAL |
| Cryptographic integrity (row hashes) | PLANNED |
| Audit trail (hash chain) | PLANNED |

## Como a Linguagem Funciona

```cronus
app "My App" {
  stack fullstack
  port 5175
  database sqlite "./data.db"
  theme dark
}

entity Deployment {
  deploy_id string required
  service string required
  cluster string required
  status enum ["Live", "Rolling", "Failed"] required
}

api /deployments {
  list   GET    /deployments
  create POST   /deployments
}

define SharedChrome {
  section topbar { ... }
  section sidebar { ... }
  section modal id:"new-deploy" entity:"Deployment" { ... }
}

page "/deployments" type:custom {
  use SharedChrome
  section kpi cols:4 {
    bind KpiSnapshot { where page eq "deployments" }
  }
  section table style:dark {
    columns "Deploy ID, Service, Status"
    bind Deployment { query all order created_at desc }
  }
}

page "/login" type:custom {
  section hero {
    template "..." # glassmorphic login form
  }
}
```

## Conceitos Chave

- **entity** → tabela SQLite auto-migrada + CRUD API
- **bind** → conecta section ao DB (query all/one/count + where/order/limit)
- **define/use** → componentes reutilizaveis (sidebar, topbar, modal, footer)
- **section modal** → modal glassmorphic com entity binding + form submit
- **_owner_id** → injetado automaticamente, filtra por user autenticado
- **section kpi/table/chart** → renderers smart que aceitam bind
- **style { glow-1/2/3 }** → gradient backgrounds customizaveis
- **type:custom** com template → tem prioridade sobre auto-detect

## Portas

| Servico | Porta |
|---------|-------|
| Nova Core (demo) | 5175 |
| CRONUS Daemon | 4800 |
| CRONUS Dashboard | 4803 |

## Build/Test

```bash
cd /home/zedd/Documentos/CRONUS/cronus-kernel
cargo build        # Compilar
cargo test         # Rodar testes
cronus run         # Rodar app no diretorio atual
cronus build --strict  # Detectar hardcode
```

## Sessoes

| Data | Arquivo | Resumo |
|------|---------|--------|
| 2026-04-01 tarde | SESSION-2026-04-01-EVENING.md | 12 features, dump renderers, define/use, theme tokens |
| 2026-04-01 noite | SESSION-2026-04-01-NIGHT.md | DB-driven pages, animations, modals, responsive |
| 2026-04-02 | SESSION-2026-04-02.md | Security hardening, data isolation, login, zero hardcode |
| 2026-04-02 | SECURITY-AUDIT-2026-04-02.md | 12 vulnerabilities found + fixed |

## Proximos Passos

1. Argon2id pra password hashing
2. Audit trail com hash chain (_audit_log)
3. Row hashes (HMAC-SHA256) pra tamper detection
4. CSP com nonces per-request
5. Rate limiting ativo nos endpoints
6. `cronus verify` CLI pra verificar integridade offline
7. `/api/_integrity` endpoint
8. Mover /tmp/nova-core pra path permanente
9. SSE real-time (infra pronta em sse.rs)
