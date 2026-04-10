# A1 — Kernel Architecture Audit

> Scope: `/home/zedd/Documentos/CRONUS/cronus-kernel/`
> Date: 2026-04-10
> Method: static analysis — `mod` declarations, `fn` signatures, call-site grep, cross-file reference search. No build was run.
> Bin: single `cronus` binary from `src/main.rs` (Cargo.toml). No `lib.rs`.

---

## 1. Executive Summary

1. **`src/server/router.rs` (1542 LOC) and `src/server/api.rs` (511 LOC) are NOT compiled into the binary.** They are never declared as submodules in `src/server/mod.rs`, which only declares `auth_pages, docs, docs_index, response, state`. Every edit made to them over the last ~week is wasted work. The session handoff dated 2026-04-10 still lists them as "modified files" — the author believes they are live.
2. **`src/server/mod.rs` itself contains a completely separate, older `CronusServer` / `handle_request` / `handle_api` implementation (448 LOC)** that is also dead: no code constructs `CronusServer`, and the private `async fn handle_request` in that file is shadowed by the one in `main.rs`. This is a third parallel HTTP server living in the tree.
3. **The live request flow is entirely inside `src/main.rs`**: `main` (129) → spawn loop (2760) → `handle_request` (243) → `handle_request_inner` (325–~1651, **~1326 LOC monolith**) → `ui::render_page` + `ui::render_layout_*` → response. This matches what `.cronus/SDD-QUALITY-BLOCK1.md` flagged as "1113 lines — impossible to test or debug". Since that SDD was written (April 3), the function has grown, not shrunk.
4. **Compiler dead-code detection is globally silenced.** 54 of ~95 top-level source files start with `#![allow(dead_code, unused_imports, unused_variables)]`, including `main.rs`, `ui/mod.rs`, all `server/*`, `parser/mod.rs`, and the entire `dump/` tree. This is why the 2 dead parallel servers were never noticed by `cargo build`.
5. **Dead/orphaned code budget (conservative): ~3,200+ LOC in `src/server/` alone** (`router.rs` 1542 + `api.rs` 511 + `mod.rs` ~400 of 448 + `ui.rs.bak` 659 + `ui/page.rs::render_dashboard` ~220). Plus the unused dashboard templates (`render_billing_dashboard`, `render_payouts_dashboard`, etc.) that are only reachable from dead code in `router.rs` — see §2.

---

## 2. Live vs Dead Code Inventory

### 2.1 Module declarations — ground truth

Only these files are reachable from `main.rs`/sub-mod.rs files (via `mod` / `pub mod`):

| Top-level (from `src/main.rs` lines 2–57) |
|---|
| actions, animations, audit, auth, binding, block_explorer, board, brain, cache, error, command_palette, components, constitution_check, contracts, data_table, database, deploy, dump, export, feedback, graph, graphql, hardcode_lint, hmr, hydra, i18n, layout_system, lint, marketing_components, orchestrator, overlays, parser, payments, promote, rate_limit, reactive, realtime, render, runtime_js, scripting, server, trust, zeus, sse, tabs, tailwind, testing, theme, navigation, security, ui, ast_diff, memory, resolve, vm, cli |

`server` is a folder module. `src/server/mod.rs` (line 7–11) declares **only**:

```rust
pub(crate) mod auth_pages;
pub(crate) mod docs;
pub(crate) mod docs_index;
pub(crate) mod response;
pub(crate) mod state;
```

`router.rs`, `api.rs`, and the upper ~440 LOC of `mod.rs` itself (the `CronusServer` impl + its private `handle_request`) are **not referenced by any `mod` statement, `use`, or function call anywhere in the tree**. Grep result for `\brouter::`, `server::router`, `mod router`, `CronusServer` outside these files: zero matches in `src/`.

### 2.2 Dead-code inventory (confirmed unreachable from `fn main`)

| Path | LOC | Kind | Evidence |
|---|---|---|---|
| `src/server/router.rs` | 1542 | **Dead extraction draft** | Not declared as module. File header: `//! HTTP request routing — extracted from main.rs handle_request_inner.` |
| `src/server/api.rs` | 511 | **Dead extraction draft** | Not declared. Header: `//! Entity CRUD handler and related helpers extracted from main.rs.` |
| `src/server/mod.rs` — `CronusServer` + `handle_request` + `handle_api` + `serve_static` + `json_response` + `percent_decode` + `hex_val` (lines 34–447) | ~414 | **Dead legacy server** | `CronusServer` never constructed anywhere; `handle_request` here is a private `async fn` in a module whose file-level decls expose only submods — its existence is silent. |
| `src/ui.rs.bak` | 659 | **Backup file** | `.bak` extension, not a valid Rust module name; never referenced. |
| `src/ui/page.rs::render_dashboard` (lines 43–259) | ~217 | **Dead after dashboard→custom fix** | Dispatch at `page.rs:15` routes `"dashboard" \| "custom"` → `render_custom`. Session handoff 2026-04-10 line 308 explicitly flags this. |
| `src/ui/dashboard.rs` — `render_billing_dashboard`, `render_payouts_dashboard`, `render_security_dashboard`, `render_settings_dashboard`, `render_checkout_dashboard`, `render_payment_links_dashboard`, `render_order_detail_dashboard`, `render_unified_dashboard`, `render_generic_dashboard` | ~3900 of 4221 | **Partially dead** | These are called from **both** `main.rs` (live) **and** `server/router.rs` (dead). From live path: `main.rs:1466–1623` does call them, so they ARE live. **Correction noted — keep.** Only `render_dashboard_page` (dashboard.rs:11) has zero call sites in the live path (exported via `pub use` at `ui/mod.rs:22` and then imported nowhere) and appears dead. |
| `src/server/router.rs::render_matched_page` (1175) | ~345 | **Dead** | Only called from the dead `handle_request_inner` inside the same dead file. |
| `src/server/router.rs::handle_api_dispatch` (904) | ~270 | **Dead** | Same. |

### 2.3 Live core

| Path | LOC | Role |
|---|---|---|
| `src/main.rs` | 4213 | CLI dispatch, HTTP server, `handle_request`, `handle_request_inner`, `handle_api`, `fire_webhooks`, `fire_effects`, `cmd_run`, `cmd_build`, etc. The god-file. |
| `src/parser/mod.rs` | 3520 | Parser — ~30 `parse_*` methods on `Parser`. Submods: `ast`, `tokenizer`. Grammar centralized here, not split. |
| `src/parser/tokenizer.rs` | — | Token types + tokenizer (re-exported via `pub(crate) use tokenizer::*`). |
| `src/parser/ast.rs` | — | AST types. |
| `src/ui/mod.rs` | 954 | Section dispatch (`render_section`), `CRONUS_ANIMATIONS_CSS`, re-exports. |
| `src/ui/page.rs` | 1120 | `render_page`, `render_auth_page`, `render_custom`, `render_list`, `render_form`, `render_detail`, `render_checkout`. `render_dashboard` dead. |
| `src/ui/layout.rs` | 1638 | `render_layout`, `render_layout_declarative`, `render_layout_landing`, `render_layout_landing_ex`, `render_layout_dashboard`. |
| `src/ui/dashboard.rs` | 4221 | Dashboard template variants (still reachable from main.rs). |
| `src/ui/section_*.rs` | ~3k | Section renderers (hero/features/chart/kpi/form/misc/extra). `section_extra.rs` IS declared (`pub(crate) mod section_extra` at ui/mod.rs:18). |
| `src/ui/component.rs`, `dashboard.rs` | — | Component + dashboard rendering. |
| `src/server/auth_pages.rs` | 302 | `generate_login_page`, `generate_register_page`. Called from main.rs:239. |
| `src/server/docs.rs` | 821 | `render_auto_docs`, `render_design_system`, `render_graph_page`. Called from main.rs:240. |
| `src/server/docs_index.rs` | 671 | `generate_docs_index`, `search_docs_index`, `get_docs_by_tag`. Called from main.rs:1037–1045. |
| `src/server/response.rs` | 320 | `cors_origin`, `json_response`, `html_response`, `forbidden_response`. main.rs:238. |
| `src/server/state.rs` | 98 | `AppState`, `RequestTrace`, `TraceBuffer`, `generate_request_id`, `current_time_hms`, `iso_timestamp`. main.rs:122, 236. Also used by `zeus.rs:110`. |
| `src/cli/*` | ~15k | 32 CLI subcommand modules, each `cmd_*` function. All declared in `cli/mod.rs` and dispatched from `main.rs::main` match. |
| `src/dump/*` | ~5k | Dump subsystem — see §5. |
| `src/vm/*`, `src/scripting/*`, `src/hydra/*` | — | Script/block/hydra subsystems. Submods properly declared. |

---

## 3. Request Lifecycle Trace (Live Path)

```
main.rs:128  #[tokio::main] async fn main()
  └─ main.rs:140  "run" => cmd_run(&args).await
        └─ main.rs:2164  async fn cmd_run(args: &[String])
              ├─ parses .cronus file, builds AppState
              ├─ main.rs:2757  tokio::signal::ctrl_c() (graceful shutdown)
              └─ main.rs:2760  loop { tokio::select! { listener.accept() ... } }
                    └─ main.rs:2767  tokio::task::spawn(async move { … })
                          └─ main.rs:2768  service_fn(|req| { … })
                                ├─ main.rs:2772  if path == "/api/sse" → sse_hub.subscribe() (streaming bypass)
                                └─ main.rs:2780  handle_request(req, state, remote_addr).await
                                      ↓
main.rs:243  async fn handle_request(req, state, remote_addr) -> Response
  ├─ main.rs:254  early exit for /api/debug/traces
  ├─ main.rs:259  handle_request_inner(req, state, remote_addr).await  ← THE MONOLITH
  ├─ main.rs:263–273  attach x-response-time, x-request-id, x-query-count headers
  ├─ main.rs:280–292  state.zeus.push(ZeusTrace{…}) for tracing
  └─ main.rs:295–320  if DEBUG_MODE → trace_buffer.push + sse broadcast + stderr log
      ↓
main.rs:325–~1651  async fn handle_request_inner(req, state, remote_addr)  [~1326 LOC]
  ├─ 340  CORS preflight short-circuit
  ├─ 346  /.cronus/version → hmr::current_version()
  ├─ 351  /blocks → block_explorer::render_explorer
  ├─ 362  /zeus → zeus::render_dashboard
  ├─ … hundreds of path matches …
  ├─ 1037–1045  /api/docs/* → server::docs_index::*
  ├─ 1267  ui::render_layout_declarative(…)        ← layout dispatch site 1
  ├─ 1269  ui::render_layout(…)                    ← legacy layout fallback
  ├─ 1397  ui::render_layout_declarative(…)        ← site 2
  ├─ 1466  ui::render_order_detail_dashboard(…)
  ├─ 1477  ui::render_settings_dashboard(…)
  ├─ 1489  ui::render_page(…) → render_layout_landing_ex(…)
  ├─ 1503  ui::render_billing_dashboard(…)
  ├─ 1508  ui::render_page(…) → render_layout_dashboard(…)
  ├─ 1513  ui::render_page(…) → (layout selected by state.layout)
  ├─ 1589–1630  layout selection cascade (declarative / landing / landing_ex / dashboard)
  ├─ 1641–1643  404 path: render_layout_declarative / render_layout
  └─ 1652  fn handle_api(…)                        ← inlined CRUD handler
        ├─ 1957 fn fire_webhooks(…)                ← inlined
        └─ 2011 fn fire_effects(…)                 ← inlined
              └─ 2079 fn execute_effect_action(…)
```

**Key observation**: `handle_request_inner` has grown from 1113 LOC (documented in SDD-QUALITY-BLOCK1.md on April 3) to ~1326 LOC in 7 days. The extraction work in `server/router.rs` was an attempt to split it — the attempt stalled and was never wired up.

---

## 4. UI Render Pipeline Map

```
ui/page.rs:10  pub fn render_page(page, entities, accent, theme, db, route_params, owner_id) -> String
  └─ match page.page_type {
       "list"                  → render_list        (page.rs:266)
       "form" / "create"       → render_form        (page.rs:624)
       "detail"                → render_detail      (page.rs:780)
       "checkout"              → render_checkout    (page.rs:943)
       "dashboard" | "custom"  → render_custom      (page.rs:829)  ← live
       _                       → render_custom
     }

ui/page.rs:43  fn render_dashboard(…)         ← UNREACHABLE after 2026-04-10 fix (dead)
ui/page.rs:484 pub fn render_auth_page(…)    ← called from main.rs auth routes

ui/mod.rs:680–758  render_section()  ← section type dispatcher (huge match)
  → section_hero / section_features / section_chart / section_kpi
    / section_form / section_misc / section_extra  (all private submods, reached here)

ui/layout.rs:
  render_layout                 (line 10, legacy)       ← still called as fallback
  render_layout_declarative     (line 140, modern)      ← preferred when state.layout.is_some()
  render_layout_landing         (line 529)
  render_layout_landing_ex      (line 534)              ← used for auth/landing
  render_layout_dashboard       (line 1503)             ← used for fixed dashboard shell
  (private helpers: accent_to_hex, hex_to_rgb, generate_css_vars)

ui/dashboard.rs:
  render_dashboard_page         (line 11)    ← exported via pub use, NO call sites (orphan)
  render_generic_dashboard      (line 143)   ← called main.rs:1623
  render_billing_dashboard      (line 728)   ← main.rs:1503, 1614
  render_payouts_dashboard      (line 1283)  ← main.rs:1612
  render_unified_dashboard      (line 1902)  ← main.rs:1610
  render_payment_links_dashboard(line 2055)  ← main.rs:1620
  render_checkout_dashboard     (line 2473)  ← main.rs:1602
  render_security_dashboard     (line 2916)  ← main.rs:1618
  render_settings_dashboard     (line 3587)  ← main.rs:1477, 1616
  render_order_detail_dashboard (line 3914)  ← main.rs:1466
```

**Orphans in UI pipeline**:
- `ui/page.rs::render_dashboard` — dead by dispatch change.
- `ui/dashboard.rs::render_dashboard_page` — re-exported but no call site.
- (`ui.rs.bak` — backup, delete.)

**Nothing else in ui/ is orphaned in the live path.** All `section_*` modules are reached via `render_section` dispatch.

---

## 5. Module Dependency Observations

### 5.1 `src/dump/`
Entry point: `main.rs:153 "dump" => cmd_dump(&args)` → `cli/dump_cmd.rs:4 pub fn cmd_dump(args)` which dispatches to:

| Trigger | Callee |
|---|---|
| `--nextjs` flag OR Next.js auto-detect | `dump::nextjs::dump_nextjs(path)` (dump_cmd.rs:32) |
| Generic project dump | `dump::project::dump_project(path)` (dump_cmd.rs:34) |
| Prisma schema input | `dump::prisma::dump_prisma(&html)` (dump_cmd.rs:61) |
| OpenAPI input | `dump::openapi::dump_openapi(&html)` (dump_cmd.rs:66) |

Submodules declared in `dump/mod.rs:7–17`: `dom, detect, patterns, emit, nextjs, openapi, prisma, project, routes, style_extract, typescript`. All declared, but many carry `#![allow(dead_code)]` and not every helper is verified live — would need a second pass to confirm internal dead code within dump. For this audit: the dump subsystem is **wired correctly** at the module level.

### 5.2 `src/parser/`
Single `Parser` struct in `parser/mod.rs`. ~30 `parse_*` methods between lines 189 and 2600+. Grammar **is centralized** in `mod.rs`, not split by construct. Submods are only for TOKEN TYPES (`tokenizer.rs`) and AST TYPES (`ast.rs`). At 3520 LOC this is the second-largest file in the kernel. Tests live inline at `mod parser_tests { … }` (line 2788).

### 5.3 `src/cli/`
32 command modules, 1:1 mapping with match arms in `main.rs::main` (lines 140–178). Well structured. `cli/mod.rs` declares all 32 explicitly. No orphans.

### 5.4 Cross-module observation
The following modules are declared in `main.rs` and have their file-level `#![allow(dead_code)]` AND are only referenced in a handful of places — candidates for a second-pass audit to verify they are not themselves substantially dead: `ast_diff`, `layout_system`, `lint`, `hardcode_lint`, `overlays`, `realtime`, `reactive`, `i18n`, `animations` (the CSS const is used, but the rest?), `resolve`, `memory`, `marketing_components`, `components`. Not analyzed here — just flagged.

---

## 6. Red Flags / Duplicated Implementations

### 6.1 THREE parallel HTTP server implementations

| Implementation | File | Status | `handle_request` at |
|---|---|---|---|
| **Live** | `src/main.rs` | Compiled + running | line 243 (+ inner at 325) |
| **Dead extraction draft** | `src/server/router.rs` (+ `server/api.rs`) | Never compiled (not declared as module) | router.rs:39 (+ inner at 127, render_matched_page at 1175, handle_api_dispatch at 904) |
| **Dead legacy server** | `src/server/mod.rs` lines 34–447 | Compiled but never invoked (`CronusServer::new` called nowhere) | mod.rs:139 |

Grep confirmation commands (reproducible):
```
rg "\brouter::|server::router|use\s+crate::server::router|mod\s+router" cronus-kernel/  # → 0 hits
rg "CronusServer"                                                                         # → 3 hits, all in server/mod.rs
```

Each of these three implementations has its own `handle_request`, its own `handle_api`, its own CORS/JSON helpers. The session handoff dated today (2026-04-10) explicitly lists edits to `src/server/router.rs` as "Cache headers + auth-with-layout routing" and "Same auth-with-layout routing" in `src/main.rs` — the author was duplicating every bug fix across both files believing both were live. **Fixes made only to `router.rs` since its creation did not affect runtime behaviour at all.**

### 6.2 Two `fire_webhooks` / `fire_effects`
- Live: `main.rs:1957` (`fire_webhooks`), `main.rs:2011` (`fire_effects`)
- Dead: `server/api.rs:322` and `server/api.rs:376`

### 6.3 File-level `#![allow(dead_code)]` blanket
**54** top-level files start with `#![allow(dead_code, unused_imports, unused_variables)]` (including `main.rs`, every `ui/*`, every `server/*`, `parser/mod.rs`, every `dump/*`, and most standalone modules). This silences the single most useful warning for detecting exactly the kind of duplication documented above. Full list in §2.3 grep output (see investigation trace). The compiler would have flagged `CronusServer`, `handle_request` in `server/mod.rs`, and likely both dead extraction files the moment they became unreachable.

### 6.4 `ui.rs.bak`
`src/ui.rs.bak` — 659 LOC, last modified 2026-03-29. Literal backup file sitting in source tree. Delete.

### 6.5 `handle_request_inner` has grown, not shrunk
SDD-QUALITY-BLOCK1.md (2026-04-03) measured the function at 1113 LOC and set D3 = "extract router". The extraction was started in `server/router.rs` but never wired. The function is now ~1326 LOC (325 → ~1651). Entropy is increasing.

### 6.6 CRUD handler also inlined in main.rs
`handle_api` sits at `main.rs:1652` (242 LOC), followed by `fire_webhooks`, `fire_effects`, `interpolate_effect_message`, `execute_effect_action`. These were also targets in SDD D3 for extraction to `server/api_handler.rs`, `server/auth_handler.rs`, `server/page_handler.rs`, `server/debug_handler.rs` — none of which exist. Only `server/api.rs` (the dead draft) was started.

### 6.7 No `TODO: delete` / `FIXME` hygiene
Grep for `TODO|FIXME|XXX|HACK` in `main.rs`: 1 hit total (`// TODO: add spans from TraceBuilder` at line 288). Either the code is perfect or markers aren't being used. Given the state uncovered here, it's the latter.

### 6.8 Everything runs as one binary with zero integration tests
No `tests/` directory. 203 "tests" live inline (e.g. `parser_tests` at parser/mod.rs:2788). Nothing exercises the live `handle_request` path end-to-end. This is why #6.1 has been invisible — the only way to notice the dead servers would be to grep.

---

## 7. Recommendations (Ordered by ROI)

### R1 — Delete `src/server/router.rs`, `src/server/api.rs`, and the body of `src/server/mod.rs`. (~2,050 LOC, 1h)
- Zero runtime impact (proven dead by grep).
- Instantly ends the two-places-to-fix-every-bug situation.
- Before deletion, diff `router.rs::handle_request_inner` vs `main.rs::handle_request_inner` and cherry-pick any bug fixes the author made only in router.rs over the last week (the SESSION-HANDOFF-2026-04-10 describes "auth-with-layout routing" and "cache headers" edits — verify they also exist in main.rs).
- Keep `server/mod.rs` as a pure module-container: the only code left should be `pub(crate) mod auth_pages; pub(crate) mod docs; pub(crate) mod docs_index; pub(crate) mod response; pub(crate) mod state;`.

### R2 — Remove the blanket `#![allow(dead_code, unused_imports, unused_variables)]` from every file. (30 min to remove, then 2–4h to fix the fallout)
- Do it in tranches: start with `src/server/*.rs`, then `src/ui/*.rs`, then `main.rs`, then the rest.
- Replace each blanket allow with surgical `#[allow(dead_code)]` on the specific item if truly intentional (rare).
- This is the **only** durable fix for R1 class of bugs. Without it, every future extraction attempt risks becoming another shadow file.

### R3 — Delete `src/ui.rs.bak`, `src/ui/page.rs::render_dashboard` (lines 43–259), and `src/ui/dashboard.rs::render_dashboard_page`. (~1,100 LOC, 15 min)
- `ui.rs.bak` is a literal `.bak` file.
- `render_dashboard` is dead by the `page.rs:15` dispatch rewrite documented in today's session handoff.
- `render_dashboard_page` has no call sites in the live path.

### R4 — Extract `handle_request_inner` into a real `server/router.rs` — **this time wired up**. (4–8h)
- Do it after R1 + R2. The allow(dead_code) stripping in R2 will refuse to compile a file that isn't declared as `pub(crate) mod router;` in `server/mod.rs`, so the class of bug self-heals.
- Follow the plan in `.cronus/SDD-QUALITY-BLOCK1.md §D3`:
  - `server/router.rs` — dispatch
  - `server/api_handler.rs` — CRUD
  - `server/auth_handler.rs` — login/signup/me/logout
  - `server/page_handler.rs` — SSR
  - `server/debug_handler.rs` — /api/debug/*, /api/brain/*
- Important: **move code, don't copy it**, and delete the original immediately so there's never two versions.

### R5 — Add one end-to-end smoke test that boots the server and hits `/api/health`, `/`, `/login`, `/dashboard`. (2h)
- This would catch "I edited the wrong file" class bugs immediately.
- Put it in a new `tests/http_smoke.rs`.

### R6 — Document the live module map in `AGENTS.md` / `CLAUDE.md`. (30 min)
- A 20-line table: "If you want to change request routing, edit main.rs:325–1651 (not server/router.rs, which does not exist yet)."
- This helps future AI-assisted edits not get confused by the graveyard.

### R7 — Audit the other likely-orphan modules flagged in §5.4. (second pass, not in scope here)
- `layout_system`, `lint`, `hardcode_lint`, `overlays`, `realtime`, `reactive`, `i18n`, `ast_diff`, `resolve`, `memory`, `marketing_components`, `components` — each with blanket `#![allow(dead_code)]` and uncertain reachability. May find thousands more dead LOC.

---

## Appendix A — Investigation Trace (reproducible)

```
# 1. Module declarations in main.rs
rg '^\s*(pub\s+)?mod\s+\w+' src/main.rs

# 2. Submodule declarations in server/mod.rs (authoritative dead-code proof)
sed -n '1,15p' src/server/mod.rs
# → only auth_pages, docs, docs_index, response, state

# 3. Any reference to router/api module from anywhere
rg '\brouter::|server::router|use\s+crate::server::router|mod\s+router' .
# → zero matches

# 4. CronusServer call sites
rg 'CronusServer'
# → only 3 hits, all inside server/mod.rs (definition only)

# 5. Spawn chain in main.rs
rg -n 'tokio::spawn|spawn\s*\(|serve_connection|handle_request' src/main.rs

# 6. Parallel implementations
rg '^\s*(pub(\(\w+\))?\s+)?(async\s+)?fn\s+(fire_webhooks|fire_effects|handle_api|handle_request|handle_request_inner|render_matched_page)' src

# 7. File-level dead_code allows
rg '^#!\[allow\(dead_code' src
```

## Appendix B — File size top 10 (LOC)
```
4221  src/ui/dashboard.rs
4213  src/main.rs
3520  src/parser/mod.rs
1638  src/ui/layout.rs
1542  src/server/router.rs      ← DEAD
1120  src/ui/page.rs
 954  src/ui/mod.rs
 821  src/server/docs.rs
 671  src/server/docs_index.rs
 659  src/ui.rs.bak             ← DEAD (backup)
```
