# CRONUS Kernel Brief

> Read this file at the start of any session on cronus-kernel.
> Last updated: 2026-04-02

## What is CRONUS Kernel

Compiler for `.cronus` language → full-stack web app (HTML + API + SQLite). One Rust binary, zero external dependencies. 43 lines of `.cronus` = complete CRUD with DB + API + UI + auth + security + docs.

## Current State

- **Branch:** desenvolvimento
- **Last commit:** `5d5941e` — fix bugs from massive testing
- **Build:** `cargo build` (3 warnings, 0 errors)
- **Tests:** 87/87 passing
- **App demo:** `examples/nova-core/` port 5175

## Architecture

```
src/
├── main.rs              # HTTP server (hyper), request routing, auth, all endpoints
├── parser.rs            # Tokenizer + parser (.cronus → AST) with /// doc-comments
├── ui.rs                # Renderers (KPI, table, chart, modal, page-header, etc.)
├── database.rs          # SQLite (rusqlite) — migration, CRUD, validation, query counter
├── binding.rs           # Data binding (section bind Entity → DB query)
├── auth.rs              # JWT signing/verification, Argon2id password hashing
├── security.rs          # HTML escape, SQL validation, rate limiter, CSP nonces, secure cookies
├── lint.rs              # Zero Hardcode Enforcement — 13 lint/compiler rules
├── constitution_check.rs # Constitution enforcement (must/never rules)
├── audit.rs             # Hash-chained audit trail with diff tracking
├── memory.rs            # Semantic memory SQLite (sessions, decisions, changelog)
├── ast_diff.rs          # AST-level semantic change tracking
├── graph.rs             # Relationship graph (entities, pages, webhooks)
├── export.rs            # Multi-format export (JSON, TypeScript, SQL, OpenAPI)
├── sse.rs               # Server-Sent Events (data changes + debug stream)
├── render.rs            # SPA runtime + debug overlay
├── data_table.rs        # Table renderer (light + dark, static + DB-driven)
├── theme.rs             # Design system token extraction
├── contracts.rs         # Section contract validation
├── brain.rs             # Pattern learning engine (request tracking)
├── components.rs        # 40+ UI components
├── runtime_js.rs        # Action/effect execution
└── ...                  # 10+ other modules
```

## Unbreakable Rules (13)

| Code | Rule | Severity |
|------|------|----------|
| C001 | No hardcoded metrics in templates | ERROR |
| C002 | Sensitive fields never in HTML/API | ERROR |
| C003 | Data sections must have bind/items | ERROR |
| C010 | All links resolve to existing routes | ERROR |
| C011 | Buttons must have action handlers | WARNING |
| C012 | Forms must have submit handlers | WARNING |
| C020 | Zero location.reload() | ERROR |
| C030 | Shared entity mutations require auth | ERROR |
| C031 | Sensitive fields blocked in columns/bind | ERROR |
| P040 | SQL identifier validation | FATAL (parser) |
| P041 | SQL reserved words blocked | FATAL (parser) |
| no-fake-state | Static "Loading" text detected | WARNING |
| no-hardcode-user | "System Admin" text detected | WARNING |

## Security Stack

| Feature | Implementation |
|---------|---------------|
| Passwords | Argon2id (SHA-256 fallback) |
| Auth | JWT HS256, secure cookies |
| Rate limit | 10/60s auth, 100/60s API |
| CSP | Per-request nonces |
| Data isolation | _owner_id + shared keyword |
| SQL injection | Parser validation P040/P041 |
| XSS | html_escape() on all output |
| Audit | SHA-256 hash chain, diff tracking |

## Self-Documenting Code

```cronus
/// Tracks deploys in production.
/// @owner sre-team
/// @business "Failed" triggers PagerDuty
entity Deployment shared {
  /// Unique ID (format: DPL-XXXX)
  /// @example "DPL-5001"
  deploy_id string required
}

app "Nova Core" {
  constitution {
    must "all data sections require bind"
    never "expose passwords in API responses"
  }
}
```

## AI Context Protocol

```bash
cronus context                    # Full project JSON
cronus context --for-claude       # AI-optimized Markdown
cronus context --section entities # Filter
GET /api/_context                 # HTTP endpoint
```

## CLI Commands

`run, debug, build, context, changelog, graph, doctor, memory, handoff, export, generate, verify-audit, brief, parse, stats, new, deploy, test`

## Auto-Generated Endpoints

`/docs, /docs/design, /docs/graph, /graphql, /api/_context, /api/_health, /api/audit/trail, /api/audit/trail/verify, /api/debug/traces, /api/debug/stream, /api/sse, /api/auth/*, /api/{entity}`

## Build/Test

```bash
cd /home/zedd/Documentos/CRONUS/cronus-kernel
cargo build                    # Compile
cargo test                     # 87 tests
cd examples/nova-core
../../target/debug/cronus run  # Run Nova Core on :5175
../../target/debug/cronus build --strict  # Full validation
../../target/debug/cronus debug  # Run with debug overlay
../../target/debug/cronus doctor # 10-check health diagnostic
../../target/debug/cronus context --for-claude  # AI context
```

## Sessions

| Date | File | Summary |
|------|------|---------|
| 2026-04-01 tarde | SESSION-2026-04-01-EVENING.md | 12 features, dump renderers, define/use |
| 2026-04-01 noite | SESSION-2026-04-01-NIGHT.md | DB-driven pages, animations, modals |
| 2026-04-02 early | SESSION-2026-04-02.md | Security hardening, data isolation |
| 2026-04-02 full | SESSION-2026-04-02-FULL.md | **11 commits, 3 SDDs, 10.8k lines** |
