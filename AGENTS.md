# CRONUS Kernel — Agent Guidelines

Instructions for AI agents working on this codebase.

## What is CRONUS

CRONUS is a **declarative full-stack language** (.cronus) that compiles to a single 7MB Rust binary. One .cronus file replaces React + Next.js + Prisma + Express. No node_modules, no config files, no JavaScript.

```
50 lines .cronus = Database + REST API + GraphQL + Auth + UI + SSR + Audit Trail
```

## Quick Reference

### Commands
```bash
cronus run [port]              # Dev server with HMR (default: 5175)
cronus build [--ai]            # Validate (--ai for JSON errors)
cronus test                    # Auto-generated CRUD tests
cronus parse app.cronus        # Show AST
cronus dump ./project          # Convert HTML/Next.js/Prisma → .cronus
cronus context --for-claude    # Export project context for AI
cronus doctor                  # Health check (10 diagnostics)
cronus new my-app              # Scaffold new project
cargo build --release          # Build kernel (6.5MB binary)
cargo test                     # Run 203 kernel tests
```

### Project Structure
```
src/
  main.rs              # HTTP server + CLI dispatch (hyper 1.x)
  parser/              # Tokenizer → Parser → AST (18 node types)
  server/              # Router, API auto-gen, auth pages, docs
  ui/                  # 51 section renderers
  dump/                # HTML/Prisma/OpenAPI/Next.js → .cronus converters
  scripting/           # .scriptcronus VM (tree-walk interpreter)
  vm/                  # Bytecode VM (30 opcodes, experimental)
  hydra/               # Block evolution + registry
  database.rs          # SQLite engine (rusqlite)
  auth.rs              # JWT HS256 + Argon2id
  binding.rs           # Section → DB query resolution
  render.rs            # SPA runtime (~2KB vanilla JS)
  lint.rs              # 13 Zero Hardcode Enforcement rules
  trust.rs             # 6-axis trust scoring + 4 binary gates
  audit.rs             # SHA-256 hash-chained audit trail
  constitution_check.rs # Compilable must/never rules
  graphql.rs           # Auto-generated GraphQL engine
  zeus.rs              # Observability dashboard

demos/                 # 8 working demo apps
templates/             # 5 base templates (saas, blog, crm, ecommerce, helpdesk)
tests/conformance/     # 90 parse conformance tests
specs/                 # 74 section specification files (.spec.toml)
```

## .cronus Language Cheatsheet

### Minimal Working App (7 lines)
```cronus
app "Hello" {
  port 5175
}
entity Task {
  title string!
  done  boolean default:false
}
```
This gives you: SQLite DB + REST API + auto-docs + GraphQL. Zero config.

### Full App Pattern
```cronus
app "Name" { stack react + tailwind  port 5175  database sqlite "./data.db" }
auth { entity User  login email + password  session jwt expires:24h  roles [admin, user] }
style { theme dark  accent blue  font "Inter" }

entity Name {
  field type! modifiers        # 16 types: string text email url slug phone number money
  relation -> OtherEntity      #           percentage boolean date ulid json enum ip
  status enum [a, b, c]
  transition status { a -> b  b -> c }
  on create { log "created" }
}

api /entities {
  list GET / auth:public       # Auto-CRUD with auth per route
  create POST / auth:jwt
}

layout Main {
  brand "App"
  sidebar { "Page" -> "/route" icon:name }
}

page "/route" type:dashboard requires:auth {
  section kpi  { bind Entity { aggregate count }  item "Label" value:bind icon:name }
  section table { bind Entity { query all order created_at desc }  columns "f1, f2, f3" }
  section chart { bind Entity { aggregate sum field:amount group_by:date interval:month } }
  section form  { bind Entity  on submit { create Entity  toast "Done" success  navigate "/" } }
}
```

### Section Types (51)
**Data**: table, kpi, stat-cards, chart, kanban, timeline, progress
**Forms**: form, modal, sheet, filters
**Nav**: tabs, breadcrumb, sidebar, topbar, command
**Feedback**: alert, toast, accordion, dropdown, notifications, skeleton, empty, error
**Marketing**: hero, features, pricing, cta, faq, testimonial, footer, trusted, product-grid, team-list
**Layout**: card, page-header, dark-mode

### Binding (MANDATORY for data sections)
```cronus
bind Entity { query all }                                    # all records
bind Entity { query one where id eq:route.id }               # by URL param
bind Entity { query all where status eq:"active" order name } # filtered + sorted
bind Entity { aggregate count }                              # count
bind Entity { aggregate sum field:price }                    # sum
bind Entity { aggregate sum field:x group_by:date interval:month } # chart
```

### Actions
```cronus
on submit { create Entity  toast "Saved" success  navigate "/list" }
on click confirm:"Sure?" { delete Entity route.id  refresh }
```

## Development Rules

### Adding a Feature
1. Read existing code before modifying — `Read` tool first
2. Run `cargo test` after changes — must stay at 203+ passing
3. Add conformance tests in `tests/conformance/` for parser changes
4. Update `specs/` for new section types
5. Keep `main.rs` clean — extract to modules

### Key Invariants
- **Money = centavos** — 2990 = R$29.90. Entity uses `money` type
- **`!` = required** — `name string!` not `name string required`
- **`bind` mandatory** — data sections without `bind` = lint error C003
- **sensitive blocked** — `sensitive` fields never in HTML/API (lint C002/C031)
- **SQL safe** — identifiers validated at parse time (P040/P041)
- **Owner isolation** — `_owner_id` auto-injected, non-admin filtered

### Testing
```bash
cargo test                               # All 203 tests
cargo test parser                        # Parser tests only
cargo test -- test_name                  # Specific test
cargo run -- test                        # CRUD tests against running server
cargo run -- test --conformance          # Parse conformance suite
```

### Architecture Gotchas
- `main.rs` is huge (~3800 LOC) — HTTP server + all routes + CLI dispatch
- `render.rs` has the 2KB client-side JS runtime (vanilla, no React)
- `binding.rs:resolve_binding()` is THE ONLY place sections touch the DB
- Entity `shared` flag = multi-tenant (all users see records)
- Constitution rules block compilation — not just warnings

## Debugging
- Dev server: `cargo run -- run` in a demo dir
- AST inspection: `cargo run -- parse app.cronus`
- Route issues: check `server/router.rs` dispatch order
- Binding issues: add `eprintln!` in `binding.rs:resolve_binding()`
- Section rendering: check `ui/mod.rs` dispatcher (51-way match)
