# CRONUS Kernel Brief

> Read this file at the start of any session on cronus-kernel.
> Last updated: 2026-04-02

## What is CRONUS Kernel

Compiler for `.cronus` language -> full-stack web app (HTML + API + SQLite). One Rust binary (6.5MB release), zero external dependencies. 43 lines of `.cronus` = complete CRUD with DB + API + UI + auth + security + docs.

## Current State

- **Branch:** desenvolvimento
- **Last commit:** `a3f3427` -- extract docs + auth pages to server/
- **Build:** `cargo build` (3 warnings, 0 errors)
- **Tests:** 150/150 passing
- **Release binary:** 6.5MB (`target/release/cronus`)
- **Codebase:** 111 files, ~65,400 lines of Rust
- **SDDs:** 8 design documents
- **App demo:** `examples/nova-core/` port 5175

## Architecture

```
src/
  main.rs                 3,782 lines -- HTTP server (hyper), request routing, auth, all endpoints

  cli/                    7,522 lines -- 30 files
    mod.rs                  CLI dispatch + shared utils
    brief.rs                Project brief generator
    build.rs                Build + lint + AI-error protocol
    changelog.rs            AST diff changelog
    compose.rs              Multi-file composition
    context.rs              AI context protocol
    deploy_cmd.rs           Deploy to target
    doctor.rs               10-check health diagnostic
    drift.rs                Schema drift detection
    dump_cmd.rs             Dump + clone commands
    export_cmd.rs           Multi-format export dispatch
    generate.rs             Template-based .cronus generator (16 entity templates)
    graph_cmd.rs            Relationship graph (Mermaid)
    handoff.rs              Session handoff
    help.rs                 Help text
    lease.rs                Lease management
    memory_cmd.rs           Semantic memory interface
    new.rs                  New project scaffold
    objective_kernel.rs     Objective kernel commands
    parse_cmd.rs            Parse + AST dump
    reconcile.rs            State reconciliation
    review.rs               Code review
    seed.rs                 Data seeding
    segment.rs              Project segmentation
    spec.rs                 Spec-driven generation (994 lines)
    stats.rs                Project stats
    status_cmd.rs           Project status
    sync_cmd.rs             Sync operations
    test_cmd.rs             Test runner
    timeline.rs             Project timeline
    validate.rs             Multi-layer validation (--mission support)
    verify.rs               Audit chain verification

  server/                 2,325 lines -- 6 files
    mod.rs                  Request handling, routing, static files
    api.rs                  Entity CRUD API generation
    auth_pages.rs           Login/signup HTML pages
    docs.rs                 Auto-generated /docs, /docs/design, /docs/graph
    response.rs             HTTP response builders
    state.rs                Shared server state (AppState)

  ui/                     13,148 lines -- 13 files
    mod.rs                  Section dispatch, render orchestration
    component.rs            40+ UI components
    dashboard.rs            Dashboard layouts + widgets (4,179 lines)
    layout.rs               Page layout system
    page.rs                 Page renderer
    section_chart.rs        Chart sections
    section_extra.rs        Extra section types (1,844 lines)
    section_features.rs     Feature grid/list sections
    section_form.rs         Form sections
    section_hero.rs         Hero/banner sections
    section_kpi.rs          KPI card sections
    section_misc.rs         Misc section types
    util.rs                 Shared UI utilities

  parser/                 3,987 lines -- 3 files
    mod.rs                  Parser (.cronus -> AST) -- transitions, constraints, effects
    ast.rs                  AST node types + structures
    tokenizer.rs            Lexer / tokenizer

  dump/                   15,182 lines -- 12 files
    mod.rs                  Dump system entry point
    detect.rs               HTML/JS pattern detection (6,654 lines)
    emit.rs                 .cronus code emitter (2,136 lines)
    patterns.rs             UI pattern recognition (1,949 lines)
    dom.rs                  DOM tree parser
    clone_ir.rs             Clone intermediate representation
    project.rs              Project structure analysis
    routes.rs               Route extraction
    openapi.rs              OpenAPI spec generation
    prisma.rs               Prisma schema generation
    typescript.rs           TypeScript type generation
    style_extract.rs        CSS/style extraction

  -- Core modules --
  database.rs             1,314 lines -- SQLite (rusqlite), migration, CRUD, validation
  lint.rs                 1,585 lines -- 13 lint/compiler rules (Zero Hardcode Enforcement)
  resolve.rs                924 lines -- Resolve pass (symbol table, dead refs, type checks)
  render.rs                 648 lines -- SPA runtime + debug overlay
  export.rs                 590 lines -- Multi-format export (JSON, TS, SQL, OpenAPI)
  audit.rs                  643 lines -- SHA-256 hash-chained audit trail + diff tracking
  constitution_check.rs     634 lines -- Constitution enforcement (must/never rules)
  components.rs             634 lines -- 40+ UI components
  contracts.rs              553 lines -- Section contract validation
  memory.rs                 503 lines -- Semantic memory SQLite
  ast_diff.rs               400 lines -- AST-level semantic change tracking
  graph.rs                  200 lines -- Relationship graph
  auth.rs                   260 lines -- JWT HS256 + Argon2id
  security.rs               204 lines -- Rate limiter, CSP nonces, secure headers
  binding.rs                231 lines -- Data binding (section bind Entity -> query)
  sse.rs                    220 lines -- Server-Sent Events
  brain.rs                  163 lines -- Pattern learning engine
  runtime_js.rs             124 lines -- Action/effect execution
  error.rs                  138 lines -- Error types
  feedback.rs               137 lines -- Feedback system

  -- Additional modules --
  marketing_components.rs 2,419 lines -- Landing page components
  tailwind.rs             1,307 lines -- Tailwind CSS utilities
  graphql.rs                688 lines -- GraphQL engine
  data_table.rs             606 lines -- Table renderer (light + dark)
  deploy.rs                 402 lines -- Deploy system
  testing.rs                366 lines -- Test framework
  layout_system.rs          353 lines -- Layout engine
  hardcode_lint.rs          319 lines -- Extended hardcode detection
  reactive.rs               317 lines -- Reactive state system
  board.rs                  300 lines -- Board/kanban renderer
  orchestrator.rs           271 lines -- Build orchestrator
  theme.rs                  262 lines -- Design system tokens
  actions.rs                240 lines -- Action system
  realtime.rs               226 lines -- Realtime engine
  payments.rs               ~200 lines -- Payment integration
  rate_limit.rs             ~150 lines -- Rate limiting
  i18n.rs                   ~150 lines -- Internationalization
  navigation.rs             ~130 lines -- Navigation system
  animations.rs             ~100 lines -- Animation system
  cache.rs                  ~100 lines -- Caching layer
  command_palette.rs        ~100 lines -- Command palette UI
  hmr.rs                    ~100 lines -- Hot module replacement
  overlays.rs               ~100 lines -- Overlay system
  tabs.rs                   ~100 lines -- Tab system
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

## Language Features

- **Entities** with typed fields, `required`, `default:`, `sensitive`, `shared`
- **Inline constraints** with `!` syntax (e.g., `price integer required!`)
- **Transitions** (state machines): `transition status { draft -> review -> published }`
- **Effects**: `on create/update/delete { ... }` blocks in entities
- **Constitution**: `constitution { must "..." never "..." }` in app block
- **Doc-comments**: `///` with `@tags` parsed into AST, rendered in /docs
- **Resolve pass**: Symbol table, dead reference detection, type checking
- **AI-Error Protocol**: Machine-readable JSON errors for AI consumption (`--strict-ai`)
- **Templates**: 16 built-in entity templates (Member, Task, Project, Product, Order, etc.)

## Security Stack

| Feature | Implementation |
|---------|---------------|
| Passwords | Argon2id (SHA-256 fallback for migration) |
| Auth | JWT HS256, secure cookies |
| Rate limit | 10/60s auth, 100/60s API |
| CSP | Per-request nonces |
| Data isolation | _owner_id + shared keyword |
| SQL injection | Parser validation P040/P041 |
| XSS | html_escape() on all output, {{}} escapes by design |
| Audit | SHA-256 hash chain, immutable diff tracking |
| Cookies | HttpOnly, Secure, SameSite=Strict |

## AI Context Protocol

```bash
cronus context                    # Full project JSON
cronus context --for-claude       # AI-optimized Markdown
cronus context --section entities # Filter by section
GET /api/_context                 # HTTP endpoint (runtime)
```

## CLI Commands (34)

```
run           Run HTTP server
debug         Run with debug overlay + colored terminal logs
build         Lint + constitution + resolve validation (--strict, --strict-ai)
parse         Parse .cronus and dump AST
new           Scaffold new project
seed          Seed database with sample data
deploy        Deploy to target
doctor        10-check health diagnostic
stats         Project statistics
export        Export to JSON/TypeScript/SQL/OpenAPI
test          Run test suite
compose       Multi-file composition
generate/gen  Generate .cronus from description (AI or 16 templates)
dump          Dump project analysis
clone         Clone existing web app to .cronus
validate      Multi-layer validation (--mission support)
graph         Relationship graph (Mermaid diagram)
brief         Generate project brief
context       AI context protocol
sync          Sync operations
handoff       Session handoff (--summary "...")
lease         Lease management
drift         Schema drift detection
spec          Spec-driven generation (994 lines)
segment       Project segmentation analysis
reconcile     State reconciliation
review        Code review
timeline      Project timeline
status        Project status
changelog     AST diff semantic changelog
memory        Semantic memory interface
verify-audit  Offline hash chain verification
help          Show all commands
```

## Auto-Generated Endpoints (20+)

```
GET  /docs                     Auto-generated documentation
GET  /docs/design              Design system documentation
GET  /docs/graph               Relationship graph (Mermaid)
GET  /graphql                  GraphQL Playground
POST /graphql                  GraphQL queries
GET  /graphql/schema           GraphQL schema
GET  /api/_context             Full project context (JSON)
GET  /api/_health              Health check (detailed)
GET  /api/health               Health check (simple)
GET  /api/schema               Database schema
POST /api/_seed                Seed database
GET  /api/sse                  Server-Sent Events stream
GET  /api/audit/trail          Audit trail entries
GET  /api/audit/trail/verify   Verify audit hash chain
GET  /api/audit/trigger        Trigger audit
GET  /api/audit/results        Audit results
GET  /api/debug/traces         Debug request traces
GET  /api/debug/stream         Debug SSE stream
GET  /api/brain/stats          Pattern learning stats
GET  /api/brain/suggest        AI suggestions
GET  /api/server/logs          Server logs
GET  /api/server/stats         Server stats
GET  /api/_errors              Error list (AI protocol)
POST /api/auth/signup          User registration
POST /api/auth/login           User authentication
GET  /api/auth/me              Current user info
*    /api/{entity}             Auto-generated CRUD (GET/POST/PUT/DELETE)
POST /_action/{name}           Action handlers
POST /_form/{name}             Form submissions
```

## Build/Test

```bash
cd /home/zedd/Documentos/CRONUS/cronus-kernel

# Compile
cargo build                         # Debug build
cargo build --release               # Release build (6.5MB binary)

# Tests
cargo test                          # 150 tests

# Run Nova Core demo
cd examples/nova-core
../../target/debug/cronus run 5175  # http://localhost:5175/login
../../target/debug/cronus debug     # Run with debug overlay (Cmd+Shift+D)

# Validate
../../target/debug/cronus build           # Lint + constitution + resolve
../../target/debug/cronus build --strict   # Promote warnings to errors
../../target/debug/cronus build --strict-ai # AI-readable error JSON
../../target/debug/cronus doctor           # 10-check health diagnostic

# AI Context
../../target/debug/cronus context --for-claude
```

## SDDs (8)

| File | Topic |
|------|-------|
| SDD-ZERO-HARDCODE-ENFORCEMENT.md | Zero Hardcode lint system |
| SDD-LANGUAGE-CONSTITUTION.md | Constitution block design |
| SDD-DEBUG-SYSTEM.md | Debug overlay + traces |
| SDD-CODE-QUALITY.md | Rust-grade architecture |
| SDD-EVOLUTION-ARCHITECTURE.md | Safety by construction |
| SDD-SEMANTIC-OPTIMIZATION.md | Semantic optimization |
| SDD-SPEC-DRIVEN-LANGUAGE.md | Executable specs in .cronus |
| SDD-SPEC-DRIVEN-LANGUAGE-v2.md | Deep engineering review |

## Sessions

| Date | File | Summary |
|------|------|---------|
| 2026-04-01 tarde | SESSION-2026-04-01-EVENING.md | 12 features, dump renderers, define/use |
| 2026-04-01 noite | SESSION-2026-04-01-NIGHT.md | DB-driven pages, animations, modals |
| 2026-04-02 early | SESSION-2026-04-02.md | Security hardening, data isolation |
| 2026-04-02 full | SESSION-2026-04-02-FULL.md | 11 commits, 3 SDDs, 10.8k lines |
| 2026-04-02 arch | (commits a83a0c8..a3f3427) | Major refactor: cli/, server/, ui/, parser/ splits |
