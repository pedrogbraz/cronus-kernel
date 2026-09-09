# CRONUS Kernel — Agent Guidelines

Instructions for AI agents working on this codebase. **Read the whole file before touching anything.**

> **Authoritative language spec: `LANGUAGE.md` at kernel root.** This file is the short cheatsheet; `LANGUAGE.md` is the full, verified, code-cross-referenced reference. When in doubt, trust `LANGUAGE.md`.
>
> **Voodoo.js runtime:** paste **`llms.txt`** first (compact). Full contract: `VOODOO.md`. Voodoo is opt-in HTML runtime, never the authoring language. Do not emit JSX into `.cronus`.
>
> Last verified: 2026-09-09. If a claim here doesn't match reality, trust the code, fix this file, and update the "Last verified" line.

## What is CRONUS

CRONUS is a **declarative full-stack language** (`.cronus`) that compiles to a single ~7MB Rust binary. One `.cronus` file replaces React + Next.js + Prisma + Express. No `node_modules`, no config files, no JavaScript build step.

```
50 lines .cronus = Database + REST API + GraphQL + Auth + UI + SSR + Audit Trail
```

The kernel is a **single binary crate**:
- `[package] name = "cronus-lang"`
- `[[bin]] name = "cronus" path = "src/main.rs"`
- No `[lib]`. No workspace. No nested crates.

---

## Quick Reference — CLI

```bash
cronus run [port]              # Dev server with HMR (default: 5175)
cronus build [--ai]            # Validate (--ai for JSON errors)
cronus test                    # Auto-generated CRUD tests against running server
cronus parse app.cronus        # Show AST
cronus dump ./project          # Convert HTML/Next.js/Prisma/OpenAPI → .cronus
cronus context --for-claude    # Export project context for AI
cronus doctor                  # Health check
cronus new my-app              # Scaffold new project
cargo build --release          # Build kernel
cargo test                     # Run 203 kernel tests
```

---

## Project Structure (verified 2026-04-10)

```
cronus-kernel/
├── Cargo.toml              # Single binary crate. 1 [[bin]], no [lib], no [workspace].
├── AGENTS.md               # This file. CLAUDE.md is a symlink to it.
├── LANGUAGE.md             # Verified language spec.
├── VOODOO.md               # Cronus × Voodoo.js agent contract (opt-in runtime).
├── llms.txt                # Compact LLM ingest for Cronus × Voodoo (paste into Claude/Grok).
├── src/
│   ├── main.rs             # ~4213 LOC. HTTP server (hyper 1.x) + CLI dispatch.
│   │                       # handle_request_inner starts at ~line 325. THIS IS THE LIVE DISPATCHER.
│   ├── parser/
│   │   ├── mod.rs          # ~3520 LOC. Parser + 44 inline tests.
│   │   ├── ast.rs          # AST node types. HttpMethod enum = GET|POST|PATCH|PUT|DELETE only.
│   │   └── tokenizer.rs
│   ├── ui/
│   │   ├── mod.rs          # 51-way section dispatcher
│   │   ├── layout.rs       # ~1638 LOC. render_layout_declarative (rewritten 2026-04-10).
│   │   ├── page.rs         # ~1120 LOC. page_type dispatch. dashboard|custom → render_custom.
│   │   ├── dashboard.rs
│   │   ├── section_kpi.rs, section_chart.rs, section_form.rs,
│   │   ├── section_hero.rs, section_features.rs, section_extra.rs, section_misc.rs
│   │   ├── component.rs, util.rs
│   ├── dump/
│   │   ├── mod.rs
│   │   ├── nextjs.rs       # ~1314 LOC. Next.js/VINEXT → .cronus scanner.
│   │   ├── prisma.rs, openapi.rs, typescript.rs, dom.rs, emit.rs
│   │   ├── routes.rs, patterns.rs, project.rs, detect.rs, style_extract.rs
│   ├── cli/                # 33 command files: build, run, dump_cmd, parse_cmd, new, doctor, …
│   ├── server/
│   │   ├── mod.rs          # Declares: auth_pages, docs, docs_index, response, state.
│   │   │                   # ⚠️ mod.rs ALSO contains CronusServer/handle_request/handle_api (lines 34-447)
│   │   │                   #    which are DEAD CODE. CronusServer::new has ZERO call sites.
│   │   ├── auth_pages.rs   # Login/register HTML
│   │   ├── docs.rs         # /.cronus/docs auto-docs
│   │   ├── docs_index.rs
│   │   ├── response.rs     # Response helpers
│   │   ├── state.rs        # AppState struct
│   │   ├── router.rs       # ⚠️⚠️ DEAD CODE — not declared in mod.rs, never compiled. 1542 LOC.
│   │   ├── api.rs          # ⚠️⚠️ DEAD CODE — not declared in mod.rs, never compiled. 511 LOC.
│   │   └── cronus-dump-audit.js
│   ├── scripting/          # .scriptcronus mini-language (tree-walk interpreter, 4 files, 27 tests)
│   ├── vm/                 # Bytecode VM experimental (4 files, 8 tests)
│   ├── hydra/              # Block evolution + registry (5 files, 12 tests)
│   ├── parser.rs           ⚠️ N/A — parser is `src/parser/` directory
│   ├── voodoo.rs           # Opt-in Voodoo.js runtime (task-local, gated attrs, CDN). See VOODOO.md.
│   ├── cronus_ui.rs        # cronus-ui tokens + CONTRACT Button (opt-in).
│   ├── cronus_ui_widgets.rs  # 173 family dispatcher. Calls interact first.
│   ├── cronus_ui_interact.rs # Native HTML controls + gated v-data/v-model.
│   ├── render.rs           # ~863 LOC. SPA client-side JS runtime (vanilla, ~2KB).
│   ├── binding.rs          # resolve_binding() = THE ONLY place sections touch DB. Zero tests.
│   ├── database.rs         # SQLite engine (rusqlite). 20 tests.
│   ├── auth.rs             # JWT HS256 + Argon2id. 7 tests.
│   ├── lint.rs             # 13 Zero Hardcode Enforcement rules. 35 tests.
│   ├── trust.rs            # 6-axis trust scoring + 4 binary gates. 3 tests.
│   ├── audit.rs            # SHA-256 hash-chained audit trail. 10 tests.
│   ├── constitution_check.rs  # Compilable must/never rules. 6 tests.
│   ├── graphql.rs          # Auto-generated GraphQL engine. Zero tests.
│   ├── zeus.rs             # Observability dashboard. Zero tests.
│   └── ... (~50 more top-level .rs files, most untested)
├── demos/                  # 11 demo directories (landing-test, vinext-dumps, blog, crm, …)
├── templates/
│   ├── saas.cronus, blog.cronus, crm.cronus, ecommerce.cronus, helpdesk.cronus
│   └── ecosystem/          # 8 files dumped from open-source Next.js apps (2026-04-09).
│                           # ⚠️ Some fail to parse — see "Known bugs".
├── docs/
│   ├── LANGUAGE-REFERENCE.md
│   └── scriptcronus.md
├── examples/nova-core/
└── .cronus/
    ├── PLAN-2026-04-10.md  ← active plan (source of truth for current work)
    ├── SESSION-HANDOFF-2026-04-10.md
    ├── SDD-CRONUS-NEXTGEN.md  ← future roadmap (4 phases)
    └── ... (~23 older SDDs, most stale — archive candidates)
```

**Directories `AGENTS.md` previously claimed existed BUT DON'T**:
- ❌ `tests/conformance/` — does not exist. All tests are inline `#[test]` in `src/`.
- ❌ `specs/` — does not exist. Contracts are in `src/contracts.rs` / `src/contracts_generated.rs`.

---

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
Gives you: SQLite DB + REST API + auto-docs + GraphQL. Zero config.

### Full App Pattern
```cronus
app "Name" { stack react + tailwind  port 5175  database sqlite "./data.db" }
auth { entity User  login email + password  session jwt expires:24h  roles [admin, user] }
style { theme dark  accent blue  font "Inter" }

entity Name {
  field type! modifiers        # Types: string text email url slug phone number money
  relation -> OtherEntity      #        percentage boolean date ulid json enum ip
  status enum [a, b, c]
  transition status { a -> b  b -> c }
  on create { log "created" }
}

api /entities {
  list   GET    / auth:public
  create POST   / auth:jwt
  detail GET    /:id auth:jwt
  update PATCH  /:id auth:jwt
  delete DELETE /:id auth:jwt
  # ⚠️ HEAD and OPTIONS are NOT supported. Parser currently coerces them to GET silently — bug tracked in PLAN-2026-04-10.md.
}

layout Main {
  brand "App"
  sidebar {
    "Dashboard" -> "/dashboard" icon:home   # `nav` keyword is optional (since 2026-04-10)
    nav "Reports" -> "/reports" icon:chart  # both forms work
  }
}

page "/dashboard" type:dashboard requires:auth {
  section kpi  { bind Entity { aggregate count }  item "Label" value:bind icon:name }
  section table { bind Entity { query all order created_at desc }  columns "f1, f2, f3" }
  section chart { bind Entity { aggregate sum field:amount group_by:date interval:month } }
  section form  { bind Entity  on submit { create Entity  toast "Done" success  navigate "/" } }
}
```

### Section Types (~51)
- **Data**: table, kpi, stat-cards, chart, kanban, timeline, progress
- **Forms**: form, modal, sheet, filters
- **Nav**: tabs, breadcrumb, sidebar, topbar, command
- **Feedback**: alert, toast, accordion, dropdown, notifications, skeleton, empty, error
- **Marketing**: hero, features, pricing, cta, faq, testimonial, footer, trusted, product-grid, team-list
- **Layout**: card, page-header, dark-mode

### Binding (MANDATORY for data sections — lint rule C003)
```cronus
bind Entity { query all }
bind Entity { query one where id eq:route.id }
bind Entity { query all where status eq:"active" order name }
bind Entity { aggregate count }
bind Entity { aggregate sum field:price }
bind Entity { aggregate sum field:x group_by:date interval:month }
```

### Actions
```cronus
on submit { create Entity  toast "Saved" success  navigate "/list" }
on click confirm:"Sure?" { delete Entity route.id  refresh }
```

---

## Testing — 203 inline tests, no `tests/` dir

Find tests:
```bash
grep -rn '#\[test\]' src/
```

Distribution (verified 2026-04-10):

| Module | Tests |
|---|---|
| `parser/mod.rs` | 44 |
| `lint.rs` | 35 |
| `scripting/parser.rs` | 27 |
| `database.rs` | 20 |
| `resolve.rs` | 16 |
| `audit.rs` | 10 |
| `vm/executor.rs` | 8 |
| `auth.rs` | 7 |
| `memory.rs` | 6 |
| `constitution_check.rs` | 6 |
| `hydra/compose.rs` | 6 |
| `error.rs` | 5 |
| `hydra/microservices.rs` | 5 |
| `promote.rs`, `trust.rs` | 3 each |
| `dump/detect.rs`, `hydra/registry.rs` | 1 each |
| **Total** | **203** |

**Zero coverage** in (critical modules):
- `src/ui/*` — layout, page, dashboard, all sections (rewritten 2026-04-10, still uncovered)
- `src/main.rs` — handle_request_inner (the live HTTP dispatcher)
- `src/render.rs` — client runtime + HTML
- `src/binding.rs` — THE ONLY place sections touch DB
- `src/dump/nextjs.rs` — emits .cronus that goes straight to users
- `src/server/*` — all live server submodules
- ~30 other top-level modules

**Rule**: if you change any of the above, add a regression test in the SAME file before committing.

### Commands
```bash
cargo test                               # All 203 tests
cargo test parser                        # Parser tests only
cargo test -- test_name                  # Specific test
cargo run -- test                        # CRUD tests against running server
```

---

## Known Language Bugs (2026-04-10)

### HTTP methods: GET/POST/PUT/PATCH/DELETE only
The language supports exactly **5 HTTP methods**. They're hardcoded in two places that must stay in sync:
- `src/parser/tokenizer.rs:45` — `METHODS = &["GET", "POST", "PATCH", "PUT", "DELETE"]` (tokenizer classifies these as `TokenKind::Method`)
- `src/parser/ast.rs:181-196` — `HttpMethod` enum with 5 variants

Writing `api /x { list HEAD / }` in a `.cronus` file produces:
```
Parse error: Linha N: esperava Method, encontrou 'HEAD' (Identifier)
```

Because `HEAD`/`OPTIONS` aren't in the tokenizer's `METHODS` list, they get tokenized as `Identifier`, and `expect(TokenKind::Method)` fails cleanly. The `.unwrap_or(HttpMethod::GET)` in `src/parser/mod.rs:701` is **dead defensive code** — it never fires because the tokenizer filters first. (Do not rely on its presence, but also no need to remove it urgently.)

**Rationale**: HEAD is auto-handled by the runtime (returns GET headers with empty body). OPTIONS is auto-handled as CORS preflight. Exposing them as user-declarable would be a language feature addition, not a bug fix — deliberately out of scope.

**Dumper correctness (fixed 2026-04-10)**: `src/dump/nextjs.rs::detect_exported_methods` used to scan for all 7 methods, including HEAD/OPTIONS. When a Next.js route exported them, the emitted `.cronus` contained unparseable lines. The function now only detects the 5 supported methods. Test coverage: `src/dump/nextjs.rs::tests::detect_exported_methods_skips_head_and_options`.

### Some `templates/ecosystem/` files fail to parse
Of the 8 files dumped from open-source Next.js apps on 2026-04-09, some fail at `cronus parse`. Initial agent reports attributed the failures to various causes (reserved words, method keywords, `required` vs `!`), but **none of those claims were empirically verified** — treat them as hypotheses, not facts. Re-dump after the HEAD/OPTIONS fix and investigate each remaining failure manually with `cronus parse <file>`.

---

## Dead Code Warnings

**Do NOT patch these files — they don't compile:**
- `src/server/router.rs` (1542 LOC) — not declared in `server/mod.rs`, never compiled
- `src/server/api.rs` (511 LOC) — not declared in `server/mod.rs`, never compiled

**Do NOT call these symbols — they compile but have zero callers:**
- `CronusServer::new`, `CronusServer::add_crud_routes`, `CronusServer::start` in `src/server/mod.rs` lines 41-123
- Internal `handle_request`, `handle_api`, `serve_static`, `json_response` in the same file (only reachable from `CronusServer::start`)

**54 files have `#![allow(dead_code, unused_imports, unused_variables)]` at the top.** This silences warnings that would otherwise expose the dead code above. Do not add more of these `allow` attributes; prefer actual cleanup.

If you find yourself editing `router.rs` or `api.rs` or `CronusServer`, **stop** — you're editing code that never runs.

---

## Development Rules

### Adding a feature
1. **Read existing code** before modifying — `Read` tool first, no speculative edits
2. **Check if the module already has `#[test]`s** — if yes, add to the same block; if no, add `#[cfg(test)]\nmod tests { use super::*; ... }` at the end of the file
3. **Run `cargo test` after changes** — must stay at ≥203 passing (plus any new tests you added)
4. **Do NOT add code to `src/server/router.rs`, `src/server/api.rs`, or `CronusServer`** — they don't run
5. **Keep `main.rs` clean** — extract new routing logic to its own module under `src/`

### Key invariants
- **Money = centavos** — `2990` in the DB = `R$29.90` in the UI. Entity uses `money` type.
- **`!` = required** — prefer `name string!` over `name string required` (both work, `!` is idiomatic)
- **`bind` mandatory** — data sections without `bind` = lint error C003
- **`sensitive` blocked** — `sensitive` fields never appear in HTML/API (lint C002/C031)
- **SQL safe** — identifiers validated at parse time (P040 rejects non-identifiers, P041 rejects reserved words `SELECT, DROP, INSERT, DELETE, UPDATE, TABLE, FROM`)
- **Owner isolation** — `_owner_id` auto-injected, non-admin requests filter by it
- **Voodoo is runtime, not language** — `.cronus` never contains JSX/HTML/CSS. Opt-in via `stack voodoo` or `style { runtime voodoo }`. Full contract: `VOODOO.md`.

### Architecture gotchas
- `main.rs` is **4213 LOC** — HTTP server + all routes + CLI dispatch. `handle_request_inner` at ~line 325 is the live dispatcher. It is too big; split carefully and only under a test net.
- `render.rs` has the ~2KB client-side JS runtime (vanilla, no React, no build step)
- `binding.rs::resolve_binding()` is **THE ONLY** place sections touch the DB — zero tests, any change here needs one added
- Entity `shared` flag = multi-tenant (all users see records)
- Constitution rules block compilation — not just warnings
- HMR lives in `src/hmr.rs` and re-parses on file change

---

## Debugging

- **Dev server**: `cargo run -- run` from inside a demo dir
- **AST inspection**: `cargo run -- parse app.cronus`
- **Route issues**: the live dispatcher is `main.rs::handle_request_inner` (~line 325). **Ignore `src/server/router.rs` — it's dead code.**
- **Binding issues**: add `eprintln!` in `binding.rs::resolve_binding()`
- **Section rendering**: check `ui/mod.rs` dispatcher (51-way match on section_type)
- **Parser errors**: `src/parser/mod.rs` is 3520 LOC — grep for the error message; most errors carry line/column

---

## Current Work (2026-04-10)

Active plan: **`.cronus/PLAN-2026-04-10.md`**

Three pillars being executed:
1. **Fix this file** (AGENTS.md) — this update
2. **Fix HEAD/OPTIONS parser bug** — parser rejects + dumper filters
3. **Regression tests** for the 4 fixes the previous session made (layout rewrite, dashboard type, CC0000 removal, nav optional)

Until all three pillars ship with `cargo test` green at 208+, do NOT start work from `SDD-CRONUS-NEXTGEN.md` phases or touch the dead-code cleanup in `src/server/`.
