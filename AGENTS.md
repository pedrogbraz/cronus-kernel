# CRONUS Kernel — Agent Guidelines

Rules for AI agents changing the Rust kernel. `CLAUDE.md` is a symlink to this file.

- Writing a `.cronus` app (not the kernel)? Read `llms-full.txt` instead.
- Language reference: `LANGUAGE.md`. Opt-in Voodoo.js runtime: `VOODOO.md`. Changes: `CHANGELOG.md`. Index: `docs/README.md`.
- Numbers below carry a date or the command that produces them. If a claim is wrong, trust the code and fix this file.

Last verified: 2026-09-14 (HEAD `f38e132` + docs pass).

## What it is

CRONUS is a declarative full-stack language. The `cronus` binary parses a `.cronus` file and serves SQLite, REST, GraphQL, SSR HTML, cookie sessions, SSE and a hash-chained audit trail.

- One binary crate: `[package] cronus-lang`, `[[bin]] cronus` at `src/main.rs`. No `[lib]`, no workspace.
- Do not add crates. Use what is in `Cargo.lock` (no TLS client, no tempfile, no chrono).
- All tests are inline `#[test]` in `src/`. There is no `tests/` or `specs/` directory.

## Build, test, lint

CI (`.github/workflows/test.yml`) runs exactly these:

```bash
cargo test --locked
cargo clippy --locked --all-targets -- -D clippy::correctness   # style warnings not yet enforced
cargo fmt --all -- --check
cargo build --release --locked                                  # binary must stay under 12 MB
```

In a git worktree, set `CARGO_TARGET_DIR="$PWD/target"` first. A shared target dir makes worktrees run each other's binaries.

Counts change on every commit. Get them with these commands:

```bash
cargo test 2>&1 | grep '^test result'                          # 2026-09-14: 1491 passed
grep -rc '#\[test\]' src | grep -v ':0$' | sort -t: -k2 -nr    # tests per file
find src -name '*.rs' | xargs cat | wc -l                      # 2026-09-14: ~140.6k lines
```

CLI verbs are matched in `src/main.rs` (`match cmd`); implementations live in `src/cli/`. Most used: `run [port] [--host] [--prod] [--audit-canvas]`, `build [--ai|--strict]`, `parse`, `new`, `test`, `dump`, `context [--for-claude]`, `audit <language|logic|visual|all|legacy>`, `doctor`.

## Layout (2026-09-14, from `ls` / `wc -l`)

```
src/main.rs               460  CLI verb dispatch (`match cmd`), global flags, find_cronus_file, open_memory_db
src/routes/               mod.rs = serve (SSE, audit canvas, panic isolation) → handle_request (guards, traces)
                          → handle_request_inner: ordered route groups (devtools, throttle, diagnostics, sessions,
                          billing, audit_log, introspection, gql, scripts, rest, forms, pages). Order is precedence.
src/effects.rs            transitions, outbound webhooks, entity effect blocks (called by api_crud/actions)
src/http_dispatch_tests.rs end-to-end HTTP tests over loopback against routes::serve
src/parser/              mod.rs 4724 (parser), ast.rs (AST + FieldType/HttpMethod), tokenizer.rs (KEYWORDS, METHODS)
src/cli/                 40 files, one per command (+ context_grammar.rs: grammar facts for AI context)
src/ui/                  14 files. mod.rs = section dispatcher (render_section_inner), page.rs, layout.rs, dashboard.rs, section_*.rs
src/cronus_ui*.rs        177 files: cronus-ui families. cronus_ui_widgets.rs = FAMILIES + PORTED_FAMILIES dispatch,
                         cronus_ui_<family>.rs = dedicated renderers, cronus_ui_kit.rs = shared helpers,
                         cronus_ui_output_gate.rs = safety gate tests (every ported family, zero JS/inline style)
src/dump/                12 files: HTML / Next.js / Prisma / OpenAPI → .cronus
src/server/              auth_pages, docs, docs_index, response, state (+ dead code, see below)
src/scripting/ src/vm/ src/hydra/   .scriptcronus interpreter, experimental bytecode VM, block evolution
```

Security and data path (top-level `src/`):

| File | Role |
|---|---|
| `http_guard.rs` | bind address, dev/prod mode, internal-route gating, body limit, panic isolation, timeouts, client IP, login backoff |
| `session.rs` | `cronus_token` HttpOnly cookie, CSRF Origin/Referer gate, `/api/auth/{signup,login,logout,me}` |
| `auth.rs` | JWT (HS256, `iat`+`jti` required), Argon2id `m=19456,t=2,p=1` |
| `access.rs` | **the** authorization source: viewer, read/write scopes, owner constraints, route-pattern matching, SSE visibility |
| `authz.rs` | `redact_sensitive`, `writable_body`, safe error bodies |
| `api_crud.rs` | REST `/api/<entity>` (tests: `api_security_tests.rs`) |
| `actions.rs` | `/_form` and `/_action` |
| `binding.rs` | `resolve_binding`: the only place sections read the DB |
| `graphql.rs`, `sse.rs` | GraphQL (session required) and live events |
| `security.rs` | escaping, CSP builder, nonces, `KERNEL_SCRIPT_URLS` |
| `webhook.rs` | outbound webhooks: validation, SSRF block, redaction, HMAC signature |
| `database.rs`, `audit.rs` | SQLite (UUIDv7 ids), hash-chained audit trail |
| `render.rs`, `runtime_js.rs` | client runtimes injected into pages |

## Security model (Sprint 1) — do not regress

- **Deny by default.** Auto CRUD, GraphQL, SSE, `/_action` and non-public `/_form` need a session. With an `api` block, only declared routes exist, and each route's `auth:` is enforced (401/403/404).
- **One authorization source.** Every surface asks `access.rs`. Owner scope goes in the SQL `WHERE` (never read-then-check). Non-admins get their own rows (`_owner_id`); `shared` entities are readable by anyone signed in; admins see all. Rows you may not see are `404`. `User`/auth-entity rows are self-only.
- **Anonymous bound data** only via `bind X { scope:public }`, never for the auth entity. `requires:` matches exact route patterns.
- **Fields.** Responses pass `authz::redact_sensitive`; writes pass `authz::writable_body` (no system, privileged or `sensitive` keys). Errors are `{"error":{"code","message"}}`, never DB text or paths.
- **Sessions.** HttpOnly cookie only (no JS-readable token). CSRF gate for cookie-authenticated mutations. `JWT_SECRET` must be ≥ 32 bytes; key files are 0600.
- **HTTP.** Binds `127.0.0.1` unless `--host`/`CRONUS_HOST`. `--prod`/`CRONUS_ENV=production` 404s internal routes (`/zeus`, `/api/_context`, `/docs*`, …). Body limit is 1 MiB. Rate limit keys on the socket peer.
- **CSP.** Nonces only on kernel-authored scripts (marked at generation), no `'unsafe-inline'`, exact CDN URLs. Never add a host-wide script source.
- **Escaping.** Every DB/user value interpolated into HTML is escaped. URLs go through `cronus_ui_kit::safe_url`.
- **Webhooks.** `http://` only; private targets blocked unless `CRONUS_WEBHOOK_ALLOW_PRIVATE=1`.

Changing any of the above is a breaking change: record it in `CHANGELOG.md` and add an HTTP-level test.

## UI renderer rules (cronus-ui families)

- **Zero JS.** Dedicated renderers emit no `<script>`, `<style>`, inline `style=`, `on*=`, `<canvas>` or executable URLs. Interaction uses native HTML and CSS (`:has(:checked)`, radios/checkboxes + labels, `popover`/`interestfor`). `cronus_ui_output_gate.rs` renders every `PORTED_FAMILIES` entry with hostile inputs and fails on violations. `KNOWN_JS_OFFENDERS` is empty; keep it empty.
- **JS-only controls** (steppers, reveal toggles, nav buttons) render as the React component's native control, `disabled`, with the idle look. Never omit them.
- **Use the kit.** `cronus_ui_kit::{esc, attr, attr_nonempty, attr_num, flag, flag_any, safe_url, content_texts}`. A gate test fails if another `src/cronus_ui_*.rs` defines `fn esc(`, `fn attr(` or `fn flag(`.
- **Data, not fixtures.** Values come from `.cronus` props/items/bindings. Never bake audit-fixture defaults into renderers (e.g. calendar/scheduler highlight only explicit `selected:`/`today:`).
- **Tokens only** (`var(--cronus-*)`), no palette classes. `style:primary` without `button+` is the legacy Obsidian button and must stay unchanged. Voodoo attributes only through `voodoo.rs` helpers, and only when the runtime is on.
- **CSS lives in `src/cronus_ui_css/`**: `<family>.css` per family, `shared.css` (multi-family and `[popover]`/`*-control` rules), `catalog.css` (kit catalog only), `theme.css`, `tokens.css` (vendored), `base.css`, `audit.css`. `MANIFEST` fixes the cascade order and owners; a new block needs a `new <file> <hash> <owners>` line (`cargo test cronus_ui_css` prints it). `src/cronus_ui_css.rs` emits per page only the families found in the rendered HTML (`data-slot` scan + `cronus_ui_widgets::render` registry), wrapped in `@layer cronus.tokens, cronus.base, cronus.components` in `render_layout` and the audit document; Tailwind-CDN layouts stay unlayered. A test fails on CSS for slots no renderer emits.
- **Parity audit.** The pixel/geometry source of truth is the Playwright audit in the **cooud-ui** repo (`e2e/audit/geometry.spec.ts`). It drives `cronus run --audit-canvas` (loopback-only, exclusive `/audit/*`, `src/cli/audit_http.rs`, `src/ui/audit_layout.rs`). In-kernel checks: `cronus audit language|logic|all`. `cronus audit visual` exits 2 and points to cooud-ui.
- Adding a family or section type: update `src/cli/context_grammar.rs` and `llms-full.txt`. `cargo test context_grammar` enforces it.

## Working rules

1. Read the file and a neighbour before editing. Keep `main.rs` thin: new routing logic goes in its own module.
2. Every behaviour change or bug fix gets a regression test in the same file (`#[cfg(test)] mod tests`).
3. Run `cargo fmt` and `cargo test` before committing; clippy must pass `-D clippy::correctness`.
4. Do not add `#![allow(dead_code, …)]` (53 files already have it, 2026-09-14). Do not add `as any`-style suppressions without a comment.
5. Language changes: update `LANGUAGE.md`, `llms-full.txt` and, if user-visible, `CHANGELOG.md`.
6. Money is integer centavos (`money` type). `!` marks a required field. Data sections without `bind` or items fail `build`.

## Dead code (verified 2026-09-14)

Deleted in Sprint 4 (2026-09-14): `src/server/router.rs`, `src/server/api.rs`, `CronusServer` and its private handlers in `src/server/mod.rs`, the duplicate `RateLimiter` in `src/security.rs` (the live one is `src/rate_limit.rs`), and `src/cronus_ui_interact.rs` (unreachable fallback).

- The live HTTP path is `routes::serve` → `routes::handle_request` → `handle_request_inner` (route groups in `src/routes/`) → `api_crud::handle_api` / `actions` / `graphql` / `sse` / `ui`. `src/server/mod.rs` only declares its live submodules.
- cronus-ui families have one table: `cronus_ui_widgets::FAMILY_TABLE`. `FAMILIES`, `PORTED_FAMILIES` and `cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind}` derive from it. The only stubs are `meteors` (fx) and `sankey-chart` (chart).
- `stub_renderer_gate::looks_like_interact_generic` is a fingerprint of the retired generic renderers, kept as a test oracle for dedicated output.

## Known gaps (2026-09-14)

- No tests: `src/runtime_js.rs`, `src/ui/mod.rs`, `src/ui/dashboard.rs`, `src/scripting/vm.rs`, `src/zeus.rs`, `src/hmr.rs`, `src/server/{docs,response,auth_pages}.rs`. Thin: `render.rs`, `sse.rs`, `ui/layout.rs` (1 each).
- The dispatcher is split into `src/routes/`; `http_dispatch_tests.rs` is its net. Add a case there when you add or reorder a route group.
- Clippy has ~291 non-correctness warnings (per the CI comment); only `clippy::correctness` is enforced.
- Webhooks cannot deliver `https://` (no TLS client in dependencies).
- GraphQL has no `update<Entity>` mutation and cannot be disabled.
- An unknown field type silently becomes `string`, and an unknown `where` operator becomes `eq`.
- `create`/`update` in action blocks parse, but `/_action` does not execute them. Bound forms create through `/_form`.
- `#[cfg(feature = "generated-contracts")]` references a feature that is not declared in `Cargo.toml` (compiler warning).

## Docs map

| Path | Status |
|---|---|
| `AGENTS.md` (this), `LANGUAGE.md`, `VOODOO.md`, `CHANGELOG.md` | current |
| `llms.txt`, `llms-full.txt`, `docs/voodoo-llms.md` | current, for LLMs |
| `docs/archive/waves/`, `docs/archive/planning/` | historical, not maintained |
