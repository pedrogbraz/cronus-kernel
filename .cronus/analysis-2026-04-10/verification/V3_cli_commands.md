# V3 — CRONUS CLI Surface Verification

**Source of truth**: `/home/zedd/Documentos/CRONUS/cronus-kernel/src/main.rs` (match arms at `async fn main()`, lines 129–182) + `/home/zedd/Documentos/CRONUS/cronus-kernel/src/cli/*.rs`.

**Method**: Read the argv dispatcher in `main.rs`, walked `src/cli/mod.rs`, and opened each referenced cli module to confirm exported `cmd_*` functions and their parsed flags. Did not deep-read implementation bodies.

---

## 1. CLI entry point

- File: `src/main.rs`
- Function: `async fn main()` (line 129)
- Argv is collected at line 130; the first positional (`args[1]`) becomes `cmd`. Global flags `--strict` / `--strict-ai` are scanned before dispatch.
- Dispatch is a single `match cmd { … }` block, lines **139–181**.

### Match arms (verbatim → handler)

| arm | handler (fully-qualified) |
|---|---|
| `"run"` | `cmd_run` (in-file, `main.rs`) |
| `"debug"` | `cmd_debug` (in-file, `main.rs` → toggles `DEBUG_MODE` + calls `cmd_run`; `debug audit` subcmd → `cli::verify::cmd_debug_audit`) |
| `"build"` | `cli::build::cmd_build` |
| `"parse"` | `cli::parse_cmd::cmd_parse` |
| `"new"` | `cli::new::cmd_new` |
| `"seed"` | `cli::seed::cmd_seed` |
| `"deploy"` | `cli::deploy_cmd::cmd_deploy` |
| `"doctor"` | `cli::doctor::cmd_doctor` |
| `"stats"` | `cli::stats::cmd_stats` |
| `"export"` | `cli::export_cmd::cmd_export` |
| `"test"` | `cli::test_cmd::cmd_test` |
| `"compose"` | `cli::compose::cmd_compose` |
| `"generate"` / `"gen"` | `cli::generate::cmd_generate` |
| `"dump"` | `cli::dump_cmd::cmd_dump` |
| `"clone"` | `cli::dump_cmd::cmd_dump` (alias, comment: "clone is an alias for dump") |
| `"validate"` | `cli::validate::cmd_validate` (or `cmd_validate_mission` if `--mission` present) |
| `"graph"` | `cli::graph_cmd::cmd_graph` |
| `"brief"` | `cli::brief::cmd_brief` |
| `"context"` | `cli::context::cmd_context` |
| `"sync"` | `cli::sync_cmd::cmd_sync` |
| `"handoff"` | `cli::handoff::cmd_handoff` |
| `"lease"` | `cli::lease::cmd_lease` |
| `"drift"` | `cli::drift::cmd_drift` |
| `"spec"` | `cli::spec::cmd_spec` |
| `"segment"` | `cli::segment::cmd_segment` |
| `"reconcile"` | `cli::reconcile::cmd_reconcile` |
| `"review"` | `cli::review::cmd_review` |
| `"timeline"` | `cli::timeline::cmd_timeline` |
| `"status"` | `cli::status_cmd::cmd_status` |
| `"changelog"` | `cli::changelog::cmd_changelog` |
| `"memory"` | `cli::memory_cmd::cmd_memory` |
| `"verify-audit"` | `cli::verify::cmd_verify_audit` |
| `"audit"` | `cli::audit_fidelity::cmd_audit_fidelity` |
| `"version"` / `"-v"` / `"--version"` | inline `println!("cronus v0.1.0")` |
| `"help"` / `"--help"` / `"-h"` / `_` | `cli::help::print_help` (default arm) |

**Total distinct verbs wired in argv (excluding aliases & help/version):** 33
(`gen` aliases `generate`, `clone` aliases `dump`, `-v/--version` alias `version`, `-h/--help` alias `help`.)

Counting **all callable top-level tokens** a user can type: 33 real commands + 1 help + 1 version = 35 tokens; with aliases (`gen`, `clone`, `-v`, `--version`, `-h`, `--help`) → 41 accepted tokens.

---

## 2. Commands table

Legend: ✓ live (wired in `main.rs` dispatcher) · ✗ dead (defined in `cli/*` but never dispatched).

| # | verb | source file | one-line purpose | positional args | flags |
|---|---|---|---|---|---|
| ✓ 1 | `run` | `src/main.rs` (`cmd_run`) | Parse `.cronus` and serve the app | `[port]` | `--strict`, `--strict-ai` (global) |
| ✓ 2 | `debug` | `src/main.rs` (`cmd_debug`) | Run with request tracing + `/api/debug/traces`; sub-route `debug audit` | `[port]` / `audit …` | inherits `run` flags |
| ✓ 3 | `build` | `cli/build.rs` | Validate `.cronus` file (AI-error protocol optional) | `[file]` | `--strict`, `--strict-ai`, `--ai`, `--machine`, `--json-errors` |
| ✓ 4 | `parse` | `cli/parse_cmd.rs` | Parse and print AST stats | `<file>` | `--strict` |
| ✓ 5 | `new` | `cli/new.rs` | Scaffold a new project from a template | `<template>` | (none) |
| ✓ 6 | `seed` | `cli/seed.rs` | Populate the SQLite DB with fake rows | `[count]` | (none) |
| ✓ 7 | `deploy` | `cli/deploy_cmd.rs` | Emit deploy artifacts | — | `--fly`, `--railway`, `--static` (per help.rs) |
| ✓ 8 | `doctor` | `cli/doctor.rs` | 10-check project health report | — | (none — ignores `args`) |
| ✓ 9 | `stats` | `cli/stats.rs` | Entity/page/DB stats | — | (none) |
| ✓ 10 | `export` | `cli/export_cmd.rs` | Export to `cronus-project.ir.json` | — | (none) |
| ✓ 11 | `test` | `cli/test_cmd.rs` | Auto-CRUD tests; conformance runner | `[port]` | `--conformance`, `--dir <path>` |
| ✓ 12 | `compose` | `cli/compose.rs` | Compose all `.cronus` files and print result | — | (none) |
| ✓ 13 | `generate` / `gen` | `cli/generate.rs` | Generate `.cronus` from description (template or Anthropic API) | `<description…>` | `--dry-run`, `--go`, `--from-file <path>`, `--output` / `-o` / `--save <file>` |
| ✓ 14 | `dump` / `clone` | `cli/dump_cmd.rs` | Reverse-engineer HTML / JSON / Prisma / project dir → `.cronus` | `<file or dir>` | `--audit`, `--nextjs`, `-o <out>` |
| ✓ 15 | `validate` | `cli/validate.rs` | AST + semantic validator (or `--mission` variant) | `[file]` | `--json`, `--strict`, `--strict-ai`, `--mission` |
| ✓ 16 | `graph` | `cli/graph_cmd.rs` | Emit entity/route graph | — | (none) |
| ✓ 17 | `brief` | `cli/brief.rs` | AI context capsule (~500 words) | — | (none) |
| ✓ 18 | `context` | `cli/context.rs` | JSON context pack (for agents) | — | varies inside module |
| ✓ 19 | `sync` | `cli/sync_cmd.rs` | Write `.cronus/state-digest.json` | — | (none — signature `cmd_sync()`) |
| ✓ 20 | `handoff` | `cli/handoff.rs` | Close active task + refresh state digest | — | (internal) |
| ✓ 21 | `lease` | `cli/lease.rs` | Task-lease mgmt (`check`/`list`/`create`) | `<sub>` | (internal) |
| ✓ 22 | `drift` | `cli/drift.rs` | Detect strategic/scope/semantic drift | — | `--explain` (per help.rs) |
| ✓ 23 | `spec` | `cli/spec.rs` | `.spec.toml` validator / lister / codegen | `<validate\|list\|codegen>` | `--structs`, `--docs`, `--ai-protocol` |
| ✓ 24 | `segment` | `cli/segment.rs` | Semantic block isolation (`create`/`list`/`show`/`check`) | `<sub>` | (internal) |
| ✓ 25 | `reconcile` | `cli/reconcile.rs` | AST-level merge of two `.cronus` files | `<a> <b>` | `--output <file>` |
| ✓ 26 | `review` | `cli/review.rs` | Semantic task-change review | `[task-id]` | (internal) |
| ✓ 27 | `timeline` | `cli/timeline.rs` | Task-based project history | — | (none — `cmd_timeline()`) |
| ✓ 28 | `status` | `cli/status_cmd.rs` | Git-status-like semantic overview | — | (none — `cmd_status()`) |
| ✓ 29 | `changelog` | `cli/changelog.rs` | AST-diff-driven changelog | — | (none — `cmd_changelog()`) |
| ✓ 30 | `memory` | `cli/memory_cmd.rs` | Semantic memory (`sessions`/`decisions`/`log`/`decide`) | `<sub>` | (internal) |
| ✓ 31 | `verify-audit` | `cli/verify.rs` (`cmd_verify_audit`) | Verify audit-trail hash chain | — | (internal) |
| ✓ 32 | `audit` | `cli/audit_fidelity.rs` | Dump-fidelity audit (compare `.cronus` render vs reference HTML) | `<ref>` | (internal) |
| ✓ 33 | `version` / `-v` / `--version` | inline in `main.rs` | Print `cronus v0.1.0` | — | — |
| ✓ 34 | `help` / `--help` / `-h` / (default) | `cli/help.rs` | Print banner + command index | — | — |

**Live top-level commands (real verbs, excluding help/version and alias duplicates):** **32**
(`run, debug, build, parse, new, seed, deploy, doctor, stats, export, test, compose, generate, dump, validate, graph, brief, context, sync, handoff, lease, drift, spec, segment, reconcile, review, timeline, status, changelog, memory, verify-audit, audit`).

If you also count `help` and `version` as user-facing commands → **34**.
If you also count `gen` and `clone` aliases → **36** (this matches the W3 "36 in rendered docs" count).

---

## 3. Subcommands

| parent | subcommand(s) | source |
|---|---|---|
| `debug` | `debug audit` → `cli::verify::cmd_debug_audit` | `main.rs` `cmd_debug` (lines 2148–2160) |
| `spec` | `validate`, `list`, `codegen` | `cli/spec.rs` lines 3–23 |
| `spec codegen` | `--structs` / `--docs` / `--ai-protocol` (flag-gated dispatch) | `cli/spec.rs` lines 8–17 |
| `test` | `--conformance` (not a subverb, a flag-mode) + `--dir <path>` | `cli/test_cmd.rs` lines 6–28 |
| `lease` | `check` / `list` / `create` (per `help.rs`) | `cli/lease.rs` |
| `segment` | `create` / `list` / `show` / `check` (per `help.rs`) | `cli/segment.rs` |
| `memory` | `sessions` / `decisions` / `log` / `decide` (per `help.rs`) | `cli/memory_cmd.rs` |
| `validate` | `--mission` re-routes to `cmd_validate_mission` | `main.rs` 155–161 |
| `dump` | path auto-detects `.prisma` / `.json`(openapi|swagger) / `.html` / directory (Next.js sniffing) | `cli/dump_cmd.rs` |

---

## 4. Flag inventory

| flag | commands using it |
|---|---|
| `--strict` | `run`, `build`, `parse`, `validate`, (global: `main.rs` 131) |
| `--strict-ai` | `build`, `validate`, (global: `main.rs` 132) |
| `--ai` / `--machine` / `--json-errors` | `build` (per `help.rs`) |
| `--json` | `validate` (per `help.rs`) |
| `--mission` | `validate` (`main.rs` 156) |
| `--conformance` | `test` (`test_cmd.rs` 6) |
| `--dir <path>` | `test --conformance` (`test_cmd.rs` 8–12) |
| `--audit` | `dump` (`dump_cmd.rs` 5) |
| `--nextjs` | `dump` (`dump_cmd.rs` 6) |
| `-o <file>` | `dump` (`dump_cmd.rs` 38, 78) |
| `--dry-run` | `generate` (`generate.rs` 298) |
| `--go` | `generate` (`generate.rs` 299) |
| `--from-file <path>` | `generate` (`generate.rs` 300–306) |
| `--output` / `-o` / `--save <file>` | `generate` (`generate.rs` 308–314) |
| `--structs` / `--docs` / `--ai-protocol` | `spec codegen` (`spec.rs` 9–14) |
| `--output <file>` | `reconcile` (per `help.rs`) |
| `--explain` | `drift` (per `help.rs`) |
| `--fly` / `--railway` / `--static` | `deploy` (per `help.rs`) |
| `--version` / `-v` | top-level alias for `version` |
| `--help` / `-h` | top-level alias for `help` |

Flags such as `--structs`/`--docs`/`--ai-protocol` are only inspected as a presence test — they are not positional.

---

## 5. Undocumented vs `AGENTS.md` list

`AGENTS.md` claims: `run, build, test, parse, dump, context, doctor, new` (8 verbs).

**Wired in code but NOT in AGENTS.md list:** 24

```
debug, seed, deploy, stats, export, compose, generate (gen),
clone, validate, graph, brief, sync, handoff, lease, drift,
spec, segment, reconcile, review, timeline, status, changelog,
memory, verify-audit, audit
```

(Plus `help` and `version`, which `AGENTS.md` also omits.)

---

## 6. Dead commands (defined in `cli/*` but not reachable via argv)

Scanned every `pub fn cmd_*` referenced in `src/cli/mod.rs` against the `match cmd` arms in `main.rs`:

| symbol | file | status |
|---|---|---|
| `cli::verify::cmd_verify` | `cli/verify.rs` | **✗ DEAD** — `use`-imported at `main.rs:74` but never called in the `match`. There is no `"verify"` arm. |
| `cli::verify::cmd_verify_audit` | `cli/verify.rs` | ✓ live via `"verify-audit"` |
| `cli::verify::cmd_debug_audit` | `cli/verify.rs` | ✓ live via `"debug audit"` sub-route |
| `cli::objective_kernel::*` | `cli/objective_kernel.rs` | Helper module — no `cmd_*` entry point of its own; invoked by `drift`/`lease`/`reconcile`. Not dead, but also not a command. |

Every other `cmd_*` in the `cli` modules is wired. So the **only orphan command symbol is `cmd_verify`** (1 dead).

---

## 7. `cronus dump` targets (real support)

From `cli/dump_cmd.rs` flag parser (lines 5–88) and `src/dump/`:

| target | how triggered | backend | supported? |
|---|---|---|---|
| generic HTML → `.cronus` | file path ends otherwise / falls through | `dump::dump_html` | ✓ |
| Prisma schema → `.cronus` | filename ends with `.prisma` | `dump::prisma::dump_prisma` | ✓ |
| OpenAPI/Swagger JSON → `.cronus` | `.json` file whose root has `openapi` or `swagger` key | `dump::openapi::dump_openapi` | ✓ |
| Project directory (generic) → `.cronus` | positional is a directory | `dump::project::dump_project` | ✓ |
| Next.js / VINEXT project → `.cronus` | directory + `--nextjs` flag OR auto-detected (`next.config.*`, `"next"` or `"vinext"` in `package.json`) | `dump::nextjs::dump_nextjs` | ✓ |
| TypeScript → `.cronus` | `src/dump/typescript.rs` exists in the module tree | not wired from `cmd_dump` | ✗ dead path |
| `--audit` post-dump fidelity check | only with `.html` inputs | `cli::audit_fidelity` | ✓ |

Comparing to the docs-claimed list `{.cronus, html, prisma, openapi, typescript, nextjs, vinext}`:

- `html` ✓
- `prisma` ✓
- `openapi` ✓
- `nextjs` ✓ (also covers `vinext` by auto-detection — same branch)
- `vinext` ✓ (same as `nextjs`)
- `typescript` ✗ — the module file `src/dump/typescript.rs` exists but `cmd_dump` has no branch that calls it
- `.cronus` (self → self) — not a dump target; `.cronus` is the **output**, not an input

Real supported inputs: **HTML, Prisma, OpenAPI JSON, generic project dir, Next.js/VINEXT project dir.** `typescript` module is present but orphaned from the CLI.

Output is always a `.cronus` file (stdout or `-o <file>`).

---

## 8. `generate` and `new` options

### `new` (`cli/new.rs`)

- Usage: `cronus new <template>`
- Embedded templates (`cli/new.rs` lines 5–7, 29–37): **`landing`, `admin`, `saas`, `api`, `ecommerce`, `blog`, `helpdesk`, `crm`** (8 total).
- Loader order: first tries `$(dirname binary)/templates/<name>.cronus`, falls back to an embedded `TEMPLATE_*` const.
- Side effects: creates `./<template>/app.cronus`, parses it to print stats, suggests `seed` + `run` as next steps.
- No flags are parsed.

### `generate` / `gen` (`cli/generate.rs`, `cmd_generate` at line 286)

- Positional: free-form `<description…>` (joined across non-flag tokens).
- Flags:
  - `--dry-run` — write prompt to file, no LLM call
  - `--go` — skip interactive confirmation
  - `--from-file <path>` — ingest LLM output from a pre-written file
  - `--output <file>` / `-o <file>` / `--save <file>` — output path (default `app.cronus`)
- Modes resolved (lines 331–370):
  1. `--from-file` present → `cmd_generate_from_file`
  2. `--dry-run` → `cmd_generate_dry_run` (saves prompt)
  3. `ANTHROPIC_API_KEY` unset → `cmd_generate_template` (built-in templates)
  4. Otherwise → `cmd_generate_api` (calls Anthropic API)
- The embedded system prompt lists 16 primitive types and a Task-manager example (`generate.rs` 4–180).

---

## 9. `doctor` checks (`cli/doctor.rs`)

Hard-coded 10 checks (`total = 10u32`, line 12). Passed counter increments on each success:

| # | check | how |
|---|---|---|
| 1 | Syntax | `parser::parse` against the discovered `.cronus` file |
| 2 | Port availability | `TcpListener::bind("0.0.0.0:{app_port}")` (from `app { port … }`) |
| 3 | Database | Open SQLite read-only at `db.path` (fallback `data.db`), count tables + rows |
| 4 | Zero-hardcode lint | `lint::lint_ast` over 8 rules: `no-dead-text`, `no-dead-links`, `no-dead-ui`, `no-fake-state`, `no-orphan-reload`, `no-hardcode-user`, `bind-or-empty`, `no-sensitive-render` |
| 5 | Constitution | AST `constitution` block OR `.cronus/constitution.toml` (counts `must`/`never` rules) |
| 6 | Dead links | Subset of lint: rules where `rule == "no-dead-links"` |
| 7 | Sensitive exposure | Subset of lint: `rule == "no-sensitive-render"` |
| 8 | SQL identifier safety | Local `is_sql_safe_ident` against entity/field names; blocks SQL reserved words |
| 9 | AST snapshot | `.cronus/ast-snapshot.json` presence + mtime formatted as YYYY-MM-DD |
| 10 | Memory DB | `.cronus/memory.db` readable, counts rows in `sessions` table |

Summary line prints `CLEAN` only when `passed == total`.

---

## 10. `spec` subcommand (`cli/spec.rs`)

Dispatcher at `cli/spec.rs` lines 3–23:

```rust
match subcmd {
    "validate" => spec_validate(args),
    "list"     => spec_list(args),
    "codegen"  => {
        if --structs      → spec_codegen_structs(args);
        else if --docs    → spec_codegen_docs(args);
        else if --ai-protocol → spec_codegen_ai_protocol(args);
        else prints usage;
    }
    _ => prints usage;
}
```

**Confirmed subcommands**: `spec validate`, `spec list`, `spec codegen --structs|--docs|--ai-protocol`. Docs claim matches the code exactly.

Validator (lines 51+) enforces: `[meta]` section with `name`, `version`, `layer ∈ {core,stdlib,pattern}`, `stability ∈ {draft,experimental,stable,deprecated}`; `pattern` layer must include `[alias].canonical`.

---

## 11. Inventory of `src/cli/*`

One-line purposes derived from `mod.rs`, `use` statements, and file tops (no deep reads):

| file | role | wired as verb? |
|---|---|---|
| `mod.rs` | Re-exports every `cli/*` module (32 `pub mod` lines) | — |
| `help.rs` | Banner + command listing printer | ✓ default arm |
| `stats.rs` | `cronus stats` (tiny, 1 KB) | ✓ `stats` |
| `parse_cmd.rs` | `cronus parse <file>` → prints AST stats | ✓ `parse` |
| `doctor.rs` | `cronus doctor` health report (10 checks) | ✓ `doctor` |
| `brief.rs` | `cronus brief` — AI context capsule + shared TOML helpers | ✓ `brief` |
| `graph_cmd.rs` | `cronus graph` — entity/route graph via `crate::graph` | ✓ `graph` |
| `context.rs` | `cronus context` — JSON context pack for agents | ✓ `context` |
| `memory_cmd.rs` | `cronus memory` — semantic memory SQLite wrapper | ✓ `memory` |
| `export_cmd.rs` | `cronus export` — emits `cronus-project.ir.json` | ✓ `export` |
| `generate.rs` | `cronus generate` / `gen` — LLM or template scaffolder | ✓ `generate`, `gen` |
| `handoff.rs` | `cronus handoff` — finish task, refresh state digest | ✓ `handoff` |
| `changelog.rs` | `cronus changelog` — AST-diff changelog | ✓ `changelog` |
| `build.rs` | `cronus build` — validate `.cronus` (AI-error protocol) | ✓ `build` |
| `dump_cmd.rs` | `cronus dump` / `clone` — HTML/Prisma/OpenAPI/dir → `.cronus` | ✓ `dump`, `clone` |
| `validate.rs` | `cronus validate [--mission]` — semantic validator | ✓ `validate` |
| `verify.rs` | `cmd_verify` (DEAD), `cmd_verify_audit`, `cmd_debug_audit` | partial (`verify-audit`, `debug audit`) |
| `new.rs` | `cronus new <template>` | ✓ `new` |
| `seed.rs` | `cronus seed [count]` — fake rows | ✓ `seed` |
| `deploy_cmd.rs` | `cronus deploy` — fly/railway/static | ✓ `deploy` |
| `test_cmd.rs` | `cronus test [port]` / `--conformance` | ✓ `test` |
| `compose.rs` | `cronus compose` — compose all `.cronus` files | ✓ `compose` |
| `objective_kernel.rs` | helpers for drift/lease/reconcile (no own verb) | — (helper) |
| `sync_cmd.rs` | `cronus sync` — writes state-digest | ✓ `sync` |
| `drift.rs` | `cronus drift [--explain]` | ✓ `drift` |
| `lease.rs` | `cronus lease <check\|list\|create>` | ✓ `lease` |
| `review.rs` | `cronus review [task]` | ✓ `review` |
| `timeline.rs` | `cronus timeline` | ✓ `timeline` |
| `status_cmd.rs` | `cronus status` | ✓ `status` |
| `segment.rs` | `cronus segment <create\|list\|show\|check>` | ✓ `segment` |
| `reconcile.rs` | `cronus reconcile <a> <b> [--output]` | ✓ `reconcile` |
| `spec.rs` | `cronus spec <validate\|list\|codegen>` | ✓ `spec` |
| `audit_fidelity.rs` | `cronus audit` — dump-vs-HTML fidelity | ✓ `audit` |

**Total files in `src/cli/`:** 34 (`mod.rs` + 33 command/helper modules).

---

## 12. Summary

- **Live top-level commands:** **32** real verbs (+ `help` + `version` → 34 if you count those; + `gen` and `clone` aliases → 36, matching the "W3 agent counted 36" figure from rendered docs).
- **AGENTS.md coverage:** 8/32 — **24 commands are undocumented** there.
- **Dead symbols:** 1 — `cli::verify::cmd_verify` is imported in `main.rs` but has no argv arm (`cmd_verify_audit` and `cmd_debug_audit` from the same file are wired).
- **Dead dump backend:** `src/dump/typescript.rs` exists but `cmd_dump` never calls it.
- **Real `dump` inputs:** HTML, Prisma, OpenAPI/Swagger JSON, generic project dir, Next.js/VINEXT dir (auto-detected). `.cronus` is the output, not an input; `typescript` is orphaned.
- **`new` templates:** 8 (`landing, admin, saas, api, ecommerce, blog, helpdesk, crm`).
- **`doctor` runs 10 hard-coded checks** (syntax, port, DB, lint-8rules, constitution, dead-links, sensitive-render, SQL-ident safety, AST snapshot, memory.db).
- **`spec` subcommands confirmed:** `validate`, `list`, `codegen --structs|--docs|--ai-protocol`. Matches docs claim verbatim.
