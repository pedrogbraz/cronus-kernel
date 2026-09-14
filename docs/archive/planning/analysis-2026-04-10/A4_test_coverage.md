# A4 — Test Coverage Audit (2026-04-10)

**Scope**: CRONUS Rust kernel at `cronus-kernel/` (~14.5k LOC across the 8 critical files; 74k LOC total, 132 source files).
**Confirmed**: `cargo test --bin cronus -- --list` reports **203 tests, 0 benchmarks**. The claim checks out.

---

## 1. Executive Summary — Top Risks

1. **`src/server/router.rs` is DEAD CODE that was "fixed" this session.** `server/mod.rs` does NOT declare `pub(crate) mod router;`. The file is orphaned — Rust never compiles it, nothing imports it, the 22 new lines of auth+layout guard in `router.rs` will NEVER run. Fix was applied to a ghost. The real code path lives in `main.rs::handle_request_inner` (which *was* also patched — that part is live).
2. **Zero tests for UI rendering.** `ui/layout.rs`, `ui/page.rs`, `render.rs` — the three files that produce every HTML byte the user sees — have **0** `#[cfg(test)]` blocks combined. The 261-line rewrite of `render_layout_declarative` is covered only by manual browser checks.
3. **Zero tests for the request pipeline.** `main.rs::handle_request_inner` (the actual live dispatcher, 2000+ lines from ~line 325) has no tests. Every routing bug (including `/generations` 404) is a runtime discovery.
4. **Sidebar parser `nav`-optional fix** lives in `parser/mod.rs` but has no companion test among the 44 parser tests — none of them touch layout/sidebar parsing.
5. **The four bugs fixed this session are all in untested files.** Any of them can silently regress.

---

## 2. Test Inventory (module → count)

Counted via `cargo test --bin cronus -- --list`, grouped by top-level module.

| Module              | Tests | File(s)                                                                                             |
| ------------------- | ----: | --------------------------------------------------------------------------------------------------- |
| `parser`            |    44 | `src/parser/mod.rs` — schema/entity/transition/effect focus                                         |
| `lint`              |    35 | `src/lint.rs`                                                                                       |
| `scripting::parser` |    27 | `src/scripting/parser.rs`                                                                           |
| `database`          |    20 | `src/database.rs`                                                                                   |
| `resolve`           |    16 | `src/resolve.rs`                                                                                    |
| `hydra` (total)     |    12 | `src/hydra/compose.rs` (6), `microservices.rs` (5), `registry.rs` (1)                               |
| `audit`             |    10 | `src/audit.rs`                                                                                      |
| `vm::executor`      |     8 | `src/vm/executor.rs`                                                                                |
| `auth`              |     7 | `src/auth.rs`                                                                                       |
| `memory`            |     6 | `src/memory.rs`                                                                                     |
| `constitution_check`|     6 | `src/constitution_check.rs`                                                                         |
| `error`             |     5 | `src/error.rs`                                                                                      |
| `trust`             |     3 | `src/trust.rs`                                                                                      |
| `promote`           |     3 | `src/promote.rs`                                                                                    |
| `dump::detect`      |     1 | `src/dump/detect.rs`                                                                                |
| **TOTAL**           | **203** |                                                                                                  |

**Integration tests**: `tests/` directory does **not exist**. All 203 tests are unit tests living inline.

**Modules with zero tests** (notable): `ui/*` (9 files), `server/router.rs`, `server/api.rs`, `server/auth_pages.rs`, `server/docs*`, `main.rs`, `render.rs`, `dump/nextjs.rs`, `dump/emit.rs`, `dump/prisma.rs`, `dump/openapi.rs`, `dump/typescript.rs`, `runtime_js.rs`, `hmr.rs`, `sse.rs`, `realtime.rs`, `graphql.rs`, `tailwind.rs`, `theme.rs`, `actions.rs`, `binding.rs`, `reactive.rs`, `hardcode_lint.rs`, `rate_limit.rs`, `security.rs`, `payments.rs`, `feedback.rs`, `cache.rs`, `brain.rs`, `board.rs`, and every `section_*.rs`.

---

## 3. Coverage Verdict — Critical Modules

| Module                                 | LOC  | Tests | Verdict     | Notes                                                                                                         |
| -------------------------------------- | ---: | ----: | ----------- | ------------------------------------------------------------------------------------------------------------- |
| `src/parser/mod.rs`                    | 3520 |    44 | **PARTIAL** | Covers schema/entity/transition/effect/identifiers. **Nothing** touches `page`, `layout`, `sidebar`, routes.  |
| `src/ui/layout.rs` (`render_layout_declarative`) | 1638 | **0** | **NO TESTS** | Complete rewrite this session, covered only by eyeball.                                               |
| `src/ui/page.rs` (`render_custom`)     | 1120 | **0** | **NO TESTS** | `dashboard → render_custom` redirect and new `skip_wrapper` heuristic untested.                               |
| `src/server/router.rs`                 | 1542 | **0** | **NO TESTS** | **Also dead code** — not declared in `server/mod.rs`. See §7.                                                 |
| `src/main.rs` (`handle_request_inner`) | 4213 | **0** | **NO TESTS** | The real live dispatcher. Auth+layout guard change (+19 lines) runs in production but has no regression test. |
| `src/dump/nextjs.rs`                   |    — | **0** | **NO TESTS** | Only `dump::detect` has 1 test.                                                                               |
| `src/render.rs`                        |  863 | **0** | **NO TESTS** | HTML renderer — zero coverage.                                                                                |
| `src/auth.rs`                          |  277 |     7 | **PARTIAL** | Covers hashing/sessions in isolation. Does not exercise the new `requires: auth` + layout guard path.         |
| `src/database.rs`                      | 1316 |    20 | **HAS TESTS** | Strong unit coverage for CRUD, schema, migrations.                                                           |

Other modules referenced by the prompt:
- `src/auth/` directory — **does not exist**. Auth is a single file `src/auth.rs`.
- `src/database/` directory — **does not exist**. Database is a single file `src/database.rs`.

---

## 4. Risk Heat Map

| Area                                                   | Change This Session                   | Test Coverage | Blast Radius      | Risk     |
| ------------------------------------------------------ | ------------------------------------- | ------------- | ----------------- | -------- |
| `server/router.rs` auth+layout guard (+22 lines)       | Code added                            | 0             | **None (dead!)**  | **Trap** — looks fixed, isn't live |
| `main.rs` auth+layout guard (+19 lines)                | Same guard duplicated here, **live**  | 0             | Every auth page   | **High** |
| `ui/layout.rs` rewrite of `render_layout_declarative`  | +261 lines, complete rewrite          | 0             | Every page shell  | **High** |
| `ui/page.rs` `dashboard → render_custom` redirect      | +22 lines, behavior change            | 0             | Every dashboard page | **High** |
| `ui/page.rs` `skip_wrapper` heuristic                  | New `has_any_template` branch         | 0             | Landing + dashboards | **High** |
| `parser/mod.rs` sidebar `nav` keyword optional         | +9 lines                              | 0 relevant    | Every `.cronus` file with sidebar | **Medium** |
| `render.rs` changes (-32 lines net)                    | Mostly removals                       | 0             | Secondary renderer | **Medium** |
| `server/auth_pages.rs` changes                         | Not reviewed in detail                | 0             | Login/signup HTML | **Medium** |

**Primary danger**: all four bugs fixed this session live in files with zero unit coverage. A future refactor has nothing to catch a regression except a human opening a browser.

---

## 5. Proposed Regression Tests

Rust doesn't allow `#[cfg(test)]` blocks in binary crates unless the test has access to the module — so the cheapest path is either (a) add `#[cfg(test)] mod tests` inline to each target file, or (b) carve out a `lib.rs` and add a `tests/` directory. The tests below assume option (a), which matches the existing pattern (all 17 test-carrying files use inline `#[cfg(test)] mod tests`).

### Bug 1 — `type:dashboard` rendered hardcoded Cooud template instead of user sections

- **Test name**: `render_dashboard_page_uses_user_sections`
- **File**: `src/ui/page.rs` (inline `#[cfg(test)] mod tests`)
- **Assertion**: Build a `PageNode { page_type: "dashboard", sections: [SectionNode { section_type: "kpi", config: {"title": "Revenue"} }] }`, call `render_page(...)`, assert the output HTML contains `"Revenue"` and does NOT contain the string `"Cooud"` or any Cooud-template marker (e.g., the original hardcoded brand).
- **Why it catches**: the old code path routed `"dashboard"` to `render_dashboard()`, which ignored user sections. This test fails loudly if someone reverts the match arm.

### Bug 2 — Auth pages with templates + layout block fell through to landing layout

- **Test name**: `auth_page_with_declarative_layout_renders_sidebar_shell`
- **File**: `src/main.rs` (needs a testable extraction — see note below) or a new `src/ui/shell.rs` helper that can be tested in isolation.
- **Assertion**: Given `PageNode { requires: Some("auth"), sections: [{template: Some("<div>hi</div>")}] }` and `AppState { layout: Some(LayoutNode { sidebar: ... }) }`, the final HTML MUST contain the sidebar markup emitted by `render_layout_declarative`, not the Tailwind CDN landing shell.
- **Implementation note**: extract the routing decision (`if auth_with_layout { declarative } else if has_templates { landing } else { ... }`) into a pure function like `fn pick_shell(page: &PageNode, state: &AppState) -> ShellKind` so it can be unit-tested without hyper.
- **Where to live**: new helper in `src/ui/shell.rs` + inline `#[cfg(test)] mod tests`.

### Bug 3 — Runtime JS injected `#CC0000` class names into user apps

- **Test name**: `render_layout_declarative_injects_no_brand_colors`
- **File**: `src/ui/layout.rs` (inline `#[cfg(test)] mod tests`)
- **Assertion**: Call `render_layout_declarative("MyApp", &empty_layout, "/home", "<main>body</main>")`. Assert the returned String does NOT contain (case-insensitively): `"#CC0000"`, `"#cc0000"`, `"cc0000"`, or any hardcoded hex Cooud brand color. Also assert it does not contain the literal class name `"cooud-brand"` or similar markers.
- **Why it catches**: this is exactly the class of bug where a refactor accidentally reintroduces branded constants. Pair with a grep-style lint in `hardcode_lint.rs`.

### Bug 4 — Sidebar parser required `nav` keyword

- **Test name**: `parse_sidebar_nav_keyword_is_optional`
- **File**: `src/parser/mod.rs` (already has `parser_tests` mod)
- **Assertion**: Parse this snippet:
  ```cronus
  layout {
    sidebar {
      brand "MyApp"
      "Dashboard" -> "/dashboard"
      nav "Reports" -> "/reports"
    }
  }
  ```
  Assert both nav items land in the parsed sidebar config with correct labels and routes. A second test `parse_sidebar_nav_keyword_still_works` keeps backward compatibility.
- **Why it catches**: precisely the `|| self.peek().kind == TokenKind::StringLit` branch added this session.

### Bonus — Router.rs guard was applied to dead code

- **Test name**: `router_module_is_registered_or_removed`
- **File**: `src/server/mod.rs` (inline `#[cfg(test)] mod tests`)
- **Assertion**: A compile-time test that asserts `crate::server::router` is reachable (e.g., a dummy `let _ = crate::server::router::handle_request;`). If the module isn't declared, the test fails to compile → CI catches the dead-code situation.
- **Why it catches**: prevents the current situation where 22 lines of "auth+layout guard" silently do nothing. Either register the module, or delete the file.

---

## 6. Bugs Lacking Coverage

### `/generations` HTTP 404 (Kronos demo routing edge case)

- **Root cause hypothesis**: `handle_request_inner` matches routes by exact path; a trailing slash or parameterized segment may fall through.
- **Test to add**: `handle_request_routes_generations_path`
- **File**: new `src/router_match.rs` helper extracted from `main.rs`, or inline test on a pure `fn match_route(path: &str, pages: &[PageNode]) -> Option<usize>` extracted for testability.
- **Assertion**: given pages with routes `"/generations"` and `"/generations/:id"`, `match_route("/generations", ...)` returns the correct page; so does `/generations/`; so does `/generations/123`.

### False positive `sensitive-render` warning on `/api-keys/new`

- **Location**: currently lives in `src/constitution_check.rs` + `src/lint.rs` + `src/cli/doctor.rs`.
- **Test to add**: `sensitive_render_allows_new_route`
- **File**: `src/lint.rs` (inline `tests` mod — there are already 35 tests here).
- **Assertion**: linting a page with route `"/api-keys/new"` and a `form` section with a `text` input for "name" produces zero `sensitive-render` warnings. The detector should only fire when the form renders pre-existing secret values, not when creating new ones.

### Lint warnings for hardcoded metrics in pricing templates

- **Test to add**: `hardcoded_metrics_allowed_in_pricing_templates` and `hardcoded_metrics_flagged_in_dashboards`
- **File**: `src/hardcode_lint.rs` (needs a new `#[cfg(test)] mod tests`).
- **Assertion**: pricing/marketing section types (`pricing`, `hero`, `cta`, `testimonial`) get a free pass for hardcoded numbers like `"$29/mo"` or `"99.9% uptime"`. Dashboard/KPI sections still flag hardcoded metrics.

---

## 7. Is `server/router.rs` Actually Running?

**No.** Evidence:

1. `src/server/mod.rs` declares only `auth_pages`, `docs`, `docs_index`, `response`, `state`. It does **not** declare `pub(crate) mod router;`.
2. `grep -r "mod router" src/` returns zero hits. The file is not referenced by any `mod` statement anywhere in the crate.
3. `grep -r "server::router\|router::handle" src/` returns zero hits. Nothing imports symbols from it.
4. `router.rs` starts with `#![allow(dead_code, unused_imports, unused_variables)]` inherited via `server/mod.rs` — but since the module isn't declared, the file isn't even compiled. It's literally a text file Rust never sees.
5. `main.rs::handle_request_inner` is the live dispatcher (4213 LOC file, handler at line 325). It was ALSO patched this session with the same +19-line auth+layout guard — that change is live.

**Implication**: the 22-line auth+layout fix in `router.rs` is cosmetic. If someone thought the session "fixed it in two places for safety," they actually only fixed it in one. If someone later deletes the `main.rs` copy thinking `router.rs` is the source of truth, the bug comes back instantly.

**Recommendation**:

- Short term: delete `server/router.rs` entirely — it's a 1542-line footgun hiding in the repo.
- Or: declare it in `server/mod.rs`, delete the duplicated logic from `main.rs`, and have `handle_request_inner` delegate. This is the refactor direction router.rs *pretends* to be.
- Add the `router_module_is_registered_or_removed` test from §5 as a tripwire.

---

## Appendix — How I Ran This

```bash
cd /home/zedd/Documentos/CRONUS/cronus-kernel
cargo test --bin cronus -- --list   # 203 tests, 0 benchmarks
```

Counted `#[test]` occurrences via ripgrep across `src/` — came to 203 exactly, matching the cargo output.

Per-module grouping via `cargo test --bin cronus -- --list | awk -F'::' '{print $1}' | sort | uniq -c`.

No code was modified. No destructive commands were run.
