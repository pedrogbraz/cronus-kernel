# V4 — ScriptCronus Verification

Date: 2026-04-10
Scope: `src/scripting/{mod,ast,parser,vm}.rs` + `src/promote.rs`
Verdict: partially real. Core AST/parser/tree-walking VM exist and are wired into the kernel. Namespace count, fuel limit, and trust-promotion claims in docs are mostly inflated or misrepresented.

---

## 1. Module structure

| File | LOC | Purpose |
|---|---|---|
| `src/scripting/mod.rs` | 270 | Registry, `fire_scripts` / `execute_webhook` / `execute_endpoint` entry points, `parse_interval` helper. |
| `src/scripting/ast.rs` | 189 | AST node definitions: `ScriptFile`, `ScriptBlock`, `Statement`, `Expr`, `Filter`, `BinOperator`. |
| `src/scripting/parser.rs` | 1153 | Hand-rolled tokenizer + recursive-descent parser. Contains the 27-test suite. |
| `src/scripting/vm.rs` | 457 | Tree-walking interpreter: `ScriptContext`, `execute_statements`, `eval_expr`, templating, filters. **No test module.** |
| `src/promote.rs` | 416 | `promote_to_text(ScriptFile) -> String`: text emitter that rewrites script blocks as `.cronus` source. 3 unit tests. |

---

## 2. AST node enums (verbatim)

### 2.1 Top-level blocks

```rust
pub enum ScriptBlock {
    OnEvent(OnEventBlock),
    OnWebhook(OnWebhookBlock),
    Schedule(ScheduleBlock),
    Endpoint(EndpointBlock),
}
```

Four block kinds. Real.

### 2.2 Statements

```rust
pub enum Statement {
    Let        { name: String, value: Expr },
    DbCreate   { entity: String, fields: HashMap<String, Expr> },
    DbUpdate   { entity: String, id: Expr, fields: HashMap<String, Expr> },
    DbDelete   { entity: String, id: Expr },
    Log        { message: Expr },
    SseBroadcast { event: String, data: HashMap<String, Expr> },
    For        { var: String, iter: Expr, body: Vec<Statement> },
    If         { condition: Expr, then_body: Vec<Statement>, else_body: Vec<Statement> },
    Respond    { status: u16, body: Expr, headers: HashMap<String, String> },
    ExprStatement(Expr),
}
```

10 statement kinds. No `while`, no `break`/`continue`, no function definitions, no try/catch, no return.

### 2.3 Expressions

```rust
pub enum Expr {
    StringLit(String),
    NumberLit(f64),
    BoolLit(bool),
    Path(Vec<String>),
    DbQuery   { entity: String, filters: Vec<Filter>, order: Option<String>, limit: Option<u64> },
    DbCount   { entity: String, filters: Vec<Filter> },
    HttpCall  { method: String, url: Box<Expr>, headers: HashMap<String, Expr>,
                body: Option<Box<Expr>>, json: Option<HashMap<String, Expr>> },
    FormatCsv { data: Box<Expr>, fields: Vec<String> },
    FormatJson{ data: Box<Expr> },
    Now,
    EnvVar(String),
    BinOp     { left: Box<Expr>, op: BinOperator, right: Box<Expr> },
    AuthCheckRole(String),
    AuthGetUser,
}
```

14 expression kinds. No array literal, no object literal, no closures, no user functions.

### 2.4 Operators

```rust
pub enum BinOperator { Eq, Ne, Lt, Gt, Lte, Gte, And, Or, Contains }
```

Note: `Contains` is present in the enum and evaluated in `vm.rs`, but the parser does NOT emit it (no `contains` keyword/token). Dead code path unless built programmatically. **docs-gap**.

---

## 3. Top-level blocks supported

Docs claim: `on Entity.event {}`, `schedule every:1h {}`, `endpoint GET /x {}`, `on webhook {}`.

All four are **real**, parsed in `parse_file()` (parser.rs:194-307):

- `on Ident.Ident { … }` → `OnEventBlock`
- `on webhook "/path" { … }` → `OnWebhookBlock`
- `schedule "name" every:1h { … }` → `ScheduleBlock` (interval ends up glued to the identifier because the tokenizer allows `:` inside idents and parses `every:1h` as one token, then strips the prefix)
- `endpoint METHOD /path auth:role { … }` → `EndpointBlock` (same glue trick for `auth:role`)

There is also a `script "Name" { version "x" }` header block (not in docs list) that just sets `ScriptFile::{name, version}`.

---

## 4. Namespaces — real vs docs

Docs claim 8: `db`, `http`, `sse`, `cache`, `memory`, `log`, `env`, `secrets`.

What the **tokenizer** (parser.rs:127-149) actually recognizes as reserved namespace keywords:

```
Db, Http, Sse, Log, Auth, Format, Env, Respond, Webhook
```

What the **VM** actually evaluates (vm.rs):

| Namespace | In AST? | Executed in VM? | Notes |
|---|---|---|---|
| `db` | yes | yes (create/update/delete/query/count) | owner-isolated; `query` filters only evaluate Eq/Ne in vm.rs:289-293 |
| `http` | yes | **stubbed** | `eval_expr` returns `{status:200, json:{}, body:""}` — no reqwest call. vm.rs:312-324 comment: *"actual reqwest integration in Phase 4"* |
| `sse` | yes | **logged only** | comment vm.rs:184: *"Actual SSE broadcast happens via the SseHub passed from the caller"* — no hub is passed |
| `log` | yes | yes | prints + captures up to 50 entries |
| `env` | yes | yes | reads `ctx.env_vars` |
| `auth` | yes | yes | `check_role`, `get_user` |
| `format` | yes | yes | `csv`, `json` |
| `cache` | NO | NO | not a token, not a node, not in VM |
| `memory` | NO | NO | not a token, not a node, not in VM |
| `secrets` | NO | NO | not a token, not a node, not in VM |

**Real namespace count: 7** (`db`, `http`*, `sse`*, `log`, `env`, `auth`, `format`). Starred two are parsed but not truly executed. The docs' 8-namespace claim is wrong in two ways: count (8 vs 7) and composition (`cache`/`memory`/`secrets` don't exist; `auth`/`format` do but are omitted from the doc list).

---

## 5. Fuel limit — real

```rust
// vm.rs:11
const MAX_STATEMENTS: usize = 1000;

// vm.rs:57-64
fn check_limit(&mut self) -> Result<(), String> {
    self.statement_count += 1;
    if self.statement_count > MAX_STATEMENTS {
        Err("script exceeded max statement limit (1000)".into())
    } else {
        Ok(())
    }
}
```

`check_limit()` is called before each statement in `execute_statements` and once per `for`-loop iteration. **Real, value = 1000, matches docs.**

---

## 6. Scope limit — real

```rust
// vm.rs:12
const MAX_SCOPE_VARS: usize = 100;

// vm.rs:88-92
Statement::Let { name, value } => {
    if ctx.scope.len() >= MAX_SCOPE_VARS && !ctx.scope.contains_key(name) {
        return Err(format!("scope limit exceeded ({} vars max)", MAX_SCOPE_VARS));
    }
    …
}
```

Also `MAX_LOG_ENTRIES: usize = 50` and a 2048-byte truncation for log messages, plus a 1 MB webhook body cap in `mod.rs:181`. **Real, value = 100, matches docs.**

---

## 7. Trust promotion — real but **NOT automatic**

`src/promote.rs` defines exactly one public function:

```rust
pub fn promote_to_text(script: &ScriptFile) -> String {
    let trust_profile = trust::compute_trust(&script.name);
    let (score_str, status_str) = match &trust_profile {
        Some(tp) => (format!("{:.3}", tp.score()), format!("{:?}", tp.status())),
        None => ("0.000".to_string(), "Sandbox".to_string()),
    };
    …
}
```

What it does:
- Takes a parsed `ScriptFile`, reads its trust score from `trust::compute_trust(name)`, and emits a `.cronus` **text string** with header comments: `# Promoted from "…"`, `# Trust score: 0.xxx (Status)`, date.
- Groups `OnEvent` blocks by entity into `entity Name { on create { … } }` pseudo-`.cronus`.
- Emits `endpoint` → `api { … }`, `schedule` → `worker name { … }`, `on webhook` → `endpoint POST /hooks/x { … }`.

What it does **not** do:
- **No threshold check.** `promote_to_text` runs unconditionally on whatever `ScriptFile` you hand it. There is no `if trust >= 0.800 { promote }` anywhere in the file.
- **No trigger.** A grep across `src/` shows `promote_to_text` is never called except in `promote.rs`'s own tests. It is NOT invoked by `fire_scripts`, `main.rs`, or any scheduler. There is no pipeline that auto-converts `.scriptcronus` → `.cronus` on disk.
- **No file write.** The function returns a `String`; nothing writes it back.
- **No AST-level canonicalization.** `ExprStatement` is rendered as `# expr: …` (a comment), `SseBroadcast` is rendered as `# sse.broadcast …` (a comment), `http.*` is just echoed. The output is not a faithful `.cronus` re-parse target — it is lossy text.

Status of `0.800` threshold: the only 0.8 comparison in the whole kernel lives in `trust.rs:229` and is used to pick a display label, not to gate promotion:

```rust
// src/trust.rs:229
else if s < 0.8 { TrustStatus::Production }
```

**Verdict: "canonical promotion at trust ≥ 0.800" is docs-only.** What's real is a one-way text emitter that bakes trust score into a comment header.

---

## 8. Call-syntax consistency

Docs mention both `db.query X { … }` and `db.query(X, { … })`.

The parser only accepts the **space-separated / brace** form. Both `parse_statement` (`db.create`, `db.update`, `db.delete`) and `parse_primary` (`db.query`, `db.count`) go:

```rust
// parser.rs:506-517 — db.query entity { … }
Tok::Db => {
    self.advance();
    self.expect(&Tok::Dot)?;
    let op = match self.advance() {
        Tok::Ident(s) => s,
        t => return Err(format!("expected db op, got {:?}", t)),
    };
    let entity = match self.advance() {
        Tok::Ident(s) => s,
        t => return Err(format!("expected entity, got {:?}", t)),
    };
    …
}
```

There is no paren-based call form anywhere. `(`/`)` tokens exist only for `now()`. **Docs-gap: the parenthesized call syntax is fiction.**

---

## 9. User-defined functions / closures / recursion

**None.** There is no `fn`/`function`/`def` keyword, no `Statement::FnDef`, no `Expr::Call` (aside from `Now` which is hardcoded), no call stack, no return statement. `for` loops and `if`/`else` are the only control flow. Recursion is impossible because you can't name a callable.

---

## 10. Error handling

No `try`/`catch`/`throw`/`rescue`. The execution model is:

- `execute_statement` returns `Result<(), String>`.
- `execute_statements` early-returns on the first `Err`.
- Specific statements swallow their own errors and print instead of propagating: `DbCreate` (vm.rs:127), `DbUpdate` (vm.rs:157), `DbDelete` (vm.rs:175 — uses `let _ = …`), `DbQuery` (vm.rs:299 returns empty array on error). So a failing `db.*` call silently continues.
- Only scope-limit violations, statement-limit violations, and some field evaluation errors actually abort the script.
- Webhook/endpoint/event handlers surface the final `Result` to `trust::track_execution` in `mod.rs`, which is the only "error reporting" hook.

No user-visible propagation model. **docs-gap if docs imply try/catch.**

---

## 11. How scripts are invoked in the kernel

Grep of `scripting::` across `src/` (call sites only — not read):

- `src/main.rs:2526` — `scripting::ScriptRegistry::load_from_directory(".")` at startup.
- `src/main.rs:1767, 1796, 1835, 1866` — `scripting::fire_scripts(…)` on entity create/update/delete.
- `src/main.rs:1110` — `scripting::execute_endpoint(…)` for custom endpoints.
- `src/main.rs:1139` — `scripting::execute_webhook(…)` for `/hooks/*` paths.
- `src/main.rs:2692-2711` — scheduler that calls `parse_interval` + `execute_statements` for `schedule` blocks.
- `src/server/api.rs:133, 200, 231` — parallel CRUD path that also calls `fire_scripts`.
- `src/server/router.rs:1481, 1494, 1506` — HTTP router dispatches endpoint + webhook execution.
- `src/server/state.rs:96` — `script_registry: ScriptRegistry` held in server state.
- `src/block_explorer.rs:9-34` — explorer UI introspects `ScriptBlock::*` for display.
- `src/hydra/extract.rs:6`, `src/hydra/mod.rs:13`, `src/vm/compiler.rs:6` — Hydra + the other VM reuse `scripting::ast`.
- `src/promote.rs:6` — promoter imports AST.

**Scripts are really fired from the CRUD pipeline, the router, and the scheduler.** This is the strongest part of the module.

---

## 12. Test coverage

### parser.rs — 27 tests (confirmed via grep, matches prior claim)

High-level coverage:
1. `test_parse_basic_script` — full multi-block smoke test.
2. `test_parse_webhook` — basic `on webhook "/path"`.
3. `test_parse_on_event` / `test_parse_on_update` / `test_parse_on_delete` — CRUD event blocks.
4. `test_parse_schedule` — schedule with `every:1h`.
5. `test_parse_endpoint_get` / `test_parse_endpoint_with_auth` — endpoint form, auth flag.
6. `test_parse_webhook_stripe` — webhook with body.
7. `test_parse_let_variable` — `let` binding.
8. `test_parse_for_loop` — `for … in …`.
9. `test_parse_if_else` — conditional.
10. `test_parse_db_query_with_filters` / `test_parse_db_create` / `test_parse_db_delete` — db operations.
11. `test_parse_http_post` — http with headers.
12. `test_parse_sse_broadcast` — sse.
13. `test_parse_env_var` — env.KEY.
14. `test_parse_format_csv` — format.csv + field list.
15. `test_parse_multiple_blocks` — mixed file.
16. `test_parse_script_header` / `test_parse_empty_script` — header block.
17. `test_parse_missing_entity` / `test_parse_missing_brace` / `test_parse_empty_file` / `test_parse_unknown_keyword` / `test_parse_nested_error` — 5 negative/error tests.

Net: mostly happy-path AST-shape assertions. Semantic correctness of filters, interpolation, http, and env is NOT tested at parser level.

### vm.rs — 0 tests

`grep "#\[test\]" vm.rs` returns zero matches. There is no `#[cfg(test)]` module. The earlier analysis claim of "8 tests in vm.rs" is **wrong — VM has no unit tests**. Execution is only exercised indirectly via integration tests (if any) and at runtime.

### promote.rs — 3 tests

- `test_promote_to_text_generates_valid_output` — full round-trip of a fabricated `ScriptFile`, asserts header + entity + api + worker + webhook strings appear.
- `test_sanitize_name` — name slugification.
- `test_emit_expr_coverage` — StringLit/NumberLit/BoolLit/Now/EnvVar/AuthGetUser/Path rendering.

No negative tests, no trust-threshold tests (because there is no threshold).

---

## Summary of real-vs-docs

| Claim | Status |
|---|---|
| Sandboxed VM per user | real — `ScriptContext::new(user_id, role, env_vars)` fresh per invocation |
| 1000-statement fuel | real — `MAX_STATEMENTS = 1000` |
| 100-variable scope cap | real — `MAX_SCOPE_VARS = 100` |
| 8 namespaces | **false** — 7 real, 3 docs-only (`cache`, `memory`, `secrets`), 2 docs omit (`auth`, `format`) |
| `http.*` real calls | **false** — stubbed, returns mock |
| `sse.broadcast` real | **half** — logged only, no SseHub wired |
| 4 top-level blocks | real |
| `db.query X {}` syntax | real |
| `db.query(X, {})` syntax | **false** — parser has no paren form |
| User functions / closures / recursion | **false** — not supported |
| try/catch | **false** — not supported |
| Trust-promotion auto at ≥0.800 | **false** — `promote_to_text` exists, no threshold, no trigger, no file write |
| `.cronus` canonical output | **half** — text emitter only; lossy (`sse`, `ExprStatement`, `http` become comments) |
| parser.rs 27 tests | real |
| vm.rs 8 tests | **false** — zero tests in vm.rs |
| Wired into CRUD + router + scheduler | real |
