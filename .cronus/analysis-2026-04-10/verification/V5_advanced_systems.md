# V5 — Advanced Systems Verification

Scope: ground-truth verification of the "advanced" CRONUS kernel systems
against the Rust source (docs claims are not trusted; only code is).

File paths quoted are absolute under `/home/zedd/Documentos/CRONUS/cronus-kernel/src/`.

---

## 1. Constitution (`constitution_check.rs`, 635 lines)

**Verdict: SCAFFOLDED** (works but is pattern-matching, not semantic).

(a) Exists. (b) Partially working — only a closed set of rule phrases are
recognized; everything else is returned as "info" (no check available).
(c) Public surface: `check_constitution(nodes, constitution) -> Vec<ConstitutionViolation>`.
Runs against the parsed AST after parse. Called during `cronus run`/`cronus check`
pipeline (not observed in these files, but the function is the entry point).

**How rule matching actually works:** simple case-insensitive substring match
on English strings. From lines 62–160:

```rust
for rule in &constitution.must {
    let lower = rule.to_lowercase();
    if lower.contains("created_at") { /* passes silently */ }
    else if lower.contains("bind") && lower.contains("data") { /* data sections need bind */ }
    else if lower.contains("auth") { /* pages with data need requires */ }
    else if lower.contains("centavos") || lower.contains("cents") { /* trivially passes */ }
    else { /* push "info" — no check possible */ }
}
for rule in &constitution.never {
    let lower = rule.to_lowercase();
    if lower.contains("expose") && lower.contains("password") { ... }
    else if lower.contains("hardcode") { ... }
    else if lower.contains("location.reload") { ... }
    else if lower.contains("fake data") || lower.contains("mock") { ... }
    else { /* info */ }
}
```

No regex, no semantic analysis, no NLP. Violations are reported as runtime
warnings (return value); there is no compile-time hard failure wired in this
file — the caller decides. There is no enum for known rules — adding a new
recognized rule means editing this big `if/else if` chain.

Enforcement model: the function returns `Vec<ConstitutionViolation>` with
three rule_type values: "must", "never", "info". Callers presumably print
them and decide exit code. Unknown rules are politely stored as
informational, never fail the build.

**(d) Key types:**
- `ConstitutionViolation { rule_type, rule, violation, entity }`
- Uses `ConstitutionNode { must: Vec<String>, never: Vec<String> }` from the parser.

**(e) Test count: 6** (`test_must_bind_pass`, `test_must_bind_fail`,
`test_never_orphan_reload`, `test_never_expose_password`,
`test_unrecognized_rule_is_info`, `test_nova_core_constitution_passes`).

The helpers are real: `has_exposed_sensitive_field` does actual word-boundary
scanning and skips `<input type="password">` blocks and
`name=/type=/placeholder=` attribute values, which is surprisingly decent.
`has_orphan_reload` detects `location.reload()` not wrapped in
`CRONUS.reload`. `looks_like_hardcoded_metric` detects things like `$123,456`,
`99.99%`, `1.2M`, `4.5GB`.

**Biggest honesty note:** the `/* "created_at" must be present */` rule is
hardcoded to always-pass silently because the kernel "auto-adds" it — there
is no actual verification; the match just swallows it.

---

## 2. Audit (`audit.rs`, 643 lines)

**Verdict: REAL.**

(a) Exists and works. (b) Fully working — hash-chained SQLite audit log with
immutability triggers.

**Schema** (from lines 87–111):

```sql
CREATE TABLE IF NOT EXISTS _audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    action TEXT NOT NULL,
    entity TEXT NOT NULL,
    record_id TEXT NOT NULL,
    user_id TEXT,
    data TEXT,
    prev_data TEXT,
    prev_hash TEXT,
    hash TEXT NOT NULL
);

CREATE TRIGGER IF NOT EXISTS audit_no_delete
BEFORE DELETE ON _audit_log BEGIN
    SELECT RAISE(ABORT, 'Audit log entries cannot be deleted');
END;

CREATE TRIGGER IF NOT EXISTS audit_no_update
BEFORE UPDATE ON _audit_log BEGIN
    SELECT RAISE(ABORT, 'Audit log entries cannot be modified');
END;
```

WAL mode is enabled. Additive migration (`ALTER TABLE ... ADD COLUMN prev_data`).
No index is declared — queries are `ORDER BY id DESC LIMIT ?` and
`WHERE entity = ?`. No index on entity column — potential issue at scale, but
correct.

**What gets hashed** (lines 128–146):

```rust
fn compute_hash(timestamp, action, entity, record_id, data, prev_hash) -> String {
    let input = format!("{}|{}|{}|{}|{}|{}", timestamp, action, entity, record_id, data, prev_hash);
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}
```

`prev_data` is deliberately NOT in the hash (comment says "to preserve
chain integrity"). `verify()` (lines 273–337) re-walks the chain from id ASC,
checking both `prev_hash` linkage and re-computing each row's hash; returns
`{valid, entries, broken_at, reason}`.

**Write sites (`audit_trail.log(...)`)** — only 4 real write sites, both
split between `main.rs` and `server/api.rs`:

- `server/api.rs:135` — INSERT via POST to entity collection
- `server/api.rs:163` — INSERT variant
- `server/api.rs:201` — UPDATE (with `prev_record`)
- `server/api.rs:232` — DELETE (with `prev_record`)
- The same four calls are mirrored in `main.rs:1768/1797/1836/1867` (looks
  like two server paths exist — legacy in `main.rs`, new in `server/api.rs`).

Audit log is **only** written on entity CRUD — it does NOT log auth events,
script executions, or admin actions.

**Public surface on the kernel:**
- CLI: `cronus verify` calls `verify_from_file(path)`, `cronus audit` calls
  `debug_from_file(...)` (see `cli/verify.rs`).
- HTTP: `server/router.rs:595` — GET `/api/audit/verify`;
  `router.rs:611` — GET `/api/audit` (with optional entity filter).
- Same endpoints mirrored in `main.rs:710` and `main.rs:726`.

`debug_display()` is a nice CLI formatter that prints field-level diffs per
UPDATE with ANSI colors and hash-valid / hash-INVALID markers.

**(d) Key types:** `AuditTrail { conn: Mutex<Connection> }`, `FieldDiff`,
`compute_diff(prev, next) -> Vec<FieldDiff>`.

**(e) Test count: 10** — `test_compute_diff_detects_changes`,
`test_compute_diff_added_field`, `test_compute_diff_removed_field`,
`test_compute_diff_no_changes`, `test_audit_log_with_prev_data`,
`test_audit_chain_integrity_with_prev_data`, `test_query_filtered_by_entity`,
`test_audit_immutable_no_delete`, `test_audit_immutable_no_update`,
`test_debug_display_format`.

This is the most rigorously-built system in the file set.

---

## 3. Trust (`trust.rs`, 463 lines)

**Verdict: REAL for scoring, SCAFFOLDED for gate evaluation.**

(a) Exists. (b) Scoring + in-memory metrics work; gates are always
`new_clean()` (no actual gate checker logic).

**The 6 axes** are not "6 axes"; they are 6 weighted components, not
orthogonal axes. From `TrustProfile` (lines 157–168):

```rust
pub correctness: f64,   // = test_pass_rate
pub reliability: f64,   // = 1 - error_rate
pub security: f64,      // = 1.0 if security_issues == 0 else 0.0
pub performance: f64,   // = (1 - p95/500ms).clamp(0..1)
pub reusability: f64,   // = contexts_used / 10 (capped at 1)
pub observability: f64, // tiered: >100 runs -> 1.0, >10 -> 0.5, else 0.1
```

**The 4 binary gates** (`TrustGates`, lines 117–151):

```rust
pub contract_valid: bool,
pub isolation_clean: bool,
pub effects_declared: bool,
pub deps_clean: bool,
```

**Trust formula (lines 174–185):**

```rust
pub fn score(&self) -> f64 {
    if !self.gates.all_pass() { return 0.0; }
    self.correctness   * 0.25 +
    self.reliability   * 0.20 +
    self.security      * 0.20 +
    self.performance   * 0.15 +
    self.reusability   * 0.10 +
    self.observability * 0.10
}
```

Weights sum to 1.0. Gates are binary AND; any failure zeroes the entire score.

**Thresholds** (`TrustStatus`): `< 0.3` Sandbox, `< 0.6` Approved, `< 0.8`
Production, `>= 0.8` Official. Promotable requires `>= 0.6 && all_gates &&
production_runs >= 100`. Official requires `>= 0.8 && all_gates &&
production_runs >= 1000`.

**Runtime integration:**
- `METRICS` global (`LazyLock<Mutex<HashMap<block_id, BlockMetrics>>>`)
  records every script execution.
- `track_execution(block_id, latency_ms, error)` is called from
  `scripting/mod.rs:155,158,204,208,239,243` (six call sites) inside
  `fire_scripts` and endpoint/webhook executors.
- `compute_trust(block_id)` is called from `promote.rs:15` during the
  promotion pipeline.

**Critical gap:** gates are *only* constructed via `TrustGates::new_clean()`
everywhere in the kernel. There is no code that sets `contract_valid=false`
or `deps_clean=false`. The gate system is fully coded but effectively
always-open. There is no CVE scanner, no contract validator, no side-effect
declaration check.

**(d) Types:** `EvidencePack`, `TrustGates`, `TrustProfile`, `TrustStatus`
(enum), `Lineage`, `BlockOrigin`, `BlockMetrics`.

**(e) Test count: 3** — `test_trust_gates_block`, `test_trust_thresholds`,
`test_lineage`.

**Honesty note:** p95 is approximated as `max_latency_ms * 0.95`, not a real
percentile. `tests_passed`/`tests_total` are hardcoded to 0 in
`BlockMetrics::to_evidence()` — so `correctness` is always 0 at runtime,
which means no real block will ever hit Official status from live data
(requires 0.25 * correctness = 0.25 of score).

---

## 4. Binding (`binding.rs`, 232 lines)

**Verdict: REAL** (for the documented cases). AGENTS.md claim that it's
the ONLY place sections touch DB is accurate for this file — the comment at
line 75 literally says: *"This is the ONLY place where binding -> database
query happens. Renderers never touch the database directly."*

I verified the claim by grepping `binding::` usage — only `ui/*`,
`data_table.rs`, and `board.rs` import it, and the only callsite type that
queries DB is `resolve_binding()`. There's no renderer calling `db.find_*`
directly that I can see from imports.

**Binding forms accepted:**
- `query all` → `QueryType::All` → `db.find_many(...)` → `ResolvedData::Rows`
- `query one where ...` → `QueryType::One` → `db.find_one(...)` → `ResolvedData::Record`
- `query count where ...` → `QueryType::Count` → `db.count_where(...)` → `ResolvedData::Count`
- `group_by <field> [interval]` → triggers `resolve_aggregation()` which
  builds raw SQL with `SUM/COUNT/AVG/MIN/MAX`, `strftime` for
  `month|week|day|year`, and parameterized WHERE clauses.
- `aggregate sum/count/avg/min/max [field]` — only used inside aggregation.
- `live true` — stored on `BindingNode.live: bool` but NOT consulted in
  `binding.rs` itself.

**Owner isolation:** line 89 — filters unconditionally get
`("_owner_id", "=", owner_id)` appended if `owner_id` is non-empty. Good.

**SQL safety in aggregation:** field names (`group.field`, `agg.field`,
`table`, filter fields) are validated via
`crate::security::is_safe_identifier(...)` before being interpolated into
SQL. Values use `?1..?N` parameters via `db.query_raw_params(sql, params)`.
This is correct.

**Route param substitution:** `BindingValue::AuthRef("route.id")` is resolved
from `route_params` HashMap (line 43); `auth.*` refs are left as literal
`$auth.*` placeholders — TODO comment at line 46 says so.

**Live (SSE) wiring — where is it?**
Not in `binding.rs`. The `live: bool` flag is read in `ui/mod.rs:784`:

```rust
let is_live = section.binding.as_ref().map(|b| b.live).unwrap_or(false);
let output = if is_live {
    let entity = section.binding...;
    format!(r#"<div id="{live_id}" data-live-entity="{entity}">{output}</div>
<script>
(function(){{
  var es=new EventSource('/api/sse');
  es.addEventListener('data_change',function(e){{
    var d=JSON.parse(e.data);
    if(d.entity==='{entity}'){{
      if(window.__cronusNavigate){{window.__cronusNavigate(location.href,false);}}
      else{{location.reload();}}
    }}
  }});
  ...
}})();
</script>"#, ...)
```

So `live true` DOES wire up SSE — but by injecting client-side JS into the
section output, not by registering any server-side subscription. Each live
section opens its own `EventSource('/api/sse')` and filters events by
entity name client-side. This works, but it's a fairly heavy pattern: N
live sections = N EventSources per page. And the reconnect loop at lines
808–814 has a broken closure (`arguments.callee.caller` referenced but not
invoked — the reconnect never actually fires).

**(d) Types:** `ResolvedData { Rows, Record, Count, None }`.
`resolve_binding(section, db, route_params, owner_id) -> ResolvedData`.

**(e) Test count: 0.** No tests in this file. The allow-dead-code at the top
(`#![allow(dead_code, unused_imports)]`) is a minor yellow flag.

---

## 5. Actions / effects envelope (`actions.rs`, 240 lines)

**Verdict: SCAFFOLDED** (works for the 7 verbs it knows).

**Effects envelope** exists as `ActionEffect` struct (line 10) and
`effects_to_json(effects) -> Value` at line 227, emitting:

```rust
json!({ "ok": true, "effects": [ {type, target, value, style}, ... ] })
```

That matches the `{ ok, effects: [...] }` shape the docs describe. There is
no `ok: false` path in `effects_to_json` — error cases return
`(false, vec![ActionEffect{type:"toast", style:"error", ...}])` from
`execute_action`, but the caller must construct the envelope manually in
that case (which they do in router.rs — not verified here).

**Supported verbs** (from the `match verb` at line 148):

| Verb | Effect |
|---|---|
| `set` | DB update + field validation against entity schema |
| `toast` | emits toast effect |
| `navigate` | emits navigate effect |
| `refresh` | emits refresh effect |
| `delete` | calls `db.delete` + success toast |
| `open` | emits open effect |
| `close` | emits close effect |

**Missing compared to doc claims:** no `create`, no `update` (only `set` for
single-field updates), no explicit `log`, no `confirm`. `delete` only works
by using the current `record_id` — can't delete a different entity.

The `effect_type` values emitted are "toast", "navigate", "refresh", "open",
"close". No "log", no "confirm", no "navigate_back". Lines 226–239 are the
serializer.

**Validation** (lines 20–98) is real: `validate_field_value` type-checks
against `FieldType::Number|Money|Percentage|Boolean|Email|Enum|Url`;
`validate_form_data` checks required + types across all fields. Good.

**(d) Types:** `ActionEffect { effect_type, target, value, style }`.

**(e) Test count: 0.** No tests in this file — concerning given it mutates
the database.

---

## 6. GraphQL (`graphql.rs`, 688 lines)

**Verdict: SCAFFOLDED** (auto-gen works, query parser is hand-rolled &
limited, mutations are incomplete).

**Schema auto-generation** (lines 26–68) walks `EntityNode[]` and emits SDL:

- One `type EntityName { id: String! ... created_at updated_at }` per entity
- One `input CreateEntityNameInput { ... }` per entity
- `Query` with `{entity}s(limit: Int): [Entity!]!` and `{entity}(id: String!): Entity`
- `Mutation` with `create{Entity}(input: ...): Entity!` and `delete{Entity}(id: String!): Boolean!`

**What's NOT generated:** no `update{Entity}`, no filter arguments, no
pagination offset, no enum types (enums become plain String), no
relationships / nested fetches, no subscriptions. Field type mapping:
`Number|Money|Percentage -> Int`, `Boolean -> Boolean`, `Ulid -> ID`,
everything else (including Date, Email, URL, Text) -> `String`.

**Query parser** is hand-rolled (`parse_query` at 140, `parse_fields` at
207). It handles:
- `query { ... }` / `mutation { ... }` / implicit query
- field names, `(args)`, `{ subfields }` (one level only)
- arg values: string `"..."`, int, bool, `$variable`

It does NOT handle: fragments, aliases, directives, inline fragments, nested
selection sets beyond one level, multiline strings, list args, object args.
`filter_fields_array` and `filter_fields_object` (lines 544–566) then
project the requested subfields out of the DB row.

**Routes** (from `server/router.rs:261/264/279` and `main.rs:1049/1052/1063`):
- GET `/graphql` → playground HTML
- POST `/graphql` → execute query
- GET `/graphql/schema` → SDL text

There's a duplicate registration in `main.rs` and `server/router.rs` — the
same dual path as audit, suggesting a mid-refactor state.

**(d) Types:** `GraphQLSchema { sdl, entities }`, private `Operation`,
`ParsedField`, `ArgValue`.

**(e) Test count: 0.** No tests in the file — odd for a parser.

The playground HTML is 116 lines of hand-written dark-theme HTML at the
bottom — nicely polished aesthetically.

---

## 7. SSE Live (`sse.rs`, 220 lines)

**Verdict: REAL.**

**Public surface:** exactly one route — `/api/sse` — registered in two
places:
- `main.rs:2772` — the live one: delegates to `state.sse_hub.subscribe()`
  which returns a `StreamBody<...>` with `content-type: text/event-stream`.
- `server/mod.rs:176` — a STUB that returns a single
  `": connected to CRONUS SSE\n\n"` string via `Full<Bytes>` with no
  streaming. Comment: *"Return a simple keepalive for now — full SSE handled
  by main.rs"*. The real implementation only works when `main.rs`'s hyper
  service is used.

**Events** (from `DataChangeEvent` at line 20 and `DebugEvent` at line 28):

- `event: data_change` — `{entity, action: "created"|"deleted"|"updated", id}`
- `event: debug` — `{type, method, path, status, ms, queries}` (broadcast
  alongside data_change when `DEBUG_MODE` is active)
- `event: warning` — `{message: "missed N events"}` on broadcast lag
- Initial `: connected to CRONUS SSE\n\n` comment
- Every 30s `: heartbeat\n\n` comment

**Broadcast mechanism:** `tokio::sync::broadcast::channel(256)` — push-only.
No polling fallback; clients reconnect via the JS client's `onerror` handler
in `SSE_CLIENT_JS` (line 199). Connection counter is maintained via an RAII
`SseConnectionGuard` (lines 49–55) — clean.

**Who broadcasts:**
- `server/router.rs:105` — `broadcast_debug(...)` inside a request hook
- `server/router.rs:992` — `broadcast(DataChangeEvent{...})` on entity mutation
- Only these two sites. The `main.rs` legacy write path does NOT call
  broadcast — meaning SSE events only fire via the `server/router.rs` path.

**(d) Types:** `SseHub { tx: broadcast::Sender<DataChangeEvent>, debug_tx,
active_connections }`, `DataChangeEvent { entity, action, id }`, `DebugEvent`.

**(e) Test count: 0.**

The `SSE_CLIENT_JS` constant at line 173 is a self-contained reconnecting
EventSource wrapper with a status dot indicator — not used by the live
binding in ui/mod.rs (which injects its own inline script).

---

## 8. Hydra (`hydra/`, 1877 lines total)

**Verdict: SCAFFOLDED** — hangs together internally but mostly nothing
user-facing triggers it; the `evolve()` pipeline runs only when the HTTP
endpoint is hit.

**What hydra is, from code:** a block evolution / promotion pipeline. Tracks
script blocks via `trust::all_metrics()`, scores them with `TrustProfile`,
auto-promotes the ones that pass threshold to "canonical blocks" in a JSON
registry, and has a `compose` module that can synthesize `.cronus` source
text from declarative `EntityRemap` + `ComposeOptions` structures (used by
the `cronus compose` CLI command).

**`mod.rs` (133 lines)** — `evolve(registry, scripts) -> EvolutionReport`
iterates all tracked metrics, builds `TrustProfile::from_evidence` with
`TrustGates::new_clean()`, auto-adds promotable blocks to the registry via
`crate::promote::promote_to_text(script)`. Returns `{blocks_analyzed,
blocks_promoted, message, scores}`.

**`registry.rs` (195 lines)** — `BlockRegistry { blocks: HashMap<String,
CanonicalBlock>, file_path: Option<String> }`. Persists to JSON
(`.cronus/block-registry.json`). `CanonicalBlock` fields: id, name, version,
hash, tier (Atomic/Composed/Feature/Application), status (Sandbox/Approved/
Production/Official), intent, cronus_text, trust (TrustProfile), lineage,
tags. Methods: `add_block`, `has_block`, `get_block`, `all_blocks`, `count`,
`blocks_by_status`, `blocks_by_tier`, `search(query)`, `to_json`. Clean.

**`extract.rs` (135 lines)** — `extract_candidates(scripts) -> Vec<BlockCandidate>`
walks each script's blocks (OnEvent/Endpoint/Schedule/OnWebhook), looks up
metrics by synthesized block_id, and produces candidate entries with
trust_score, executions, promotable, and a human reason string ("Needs more
runs (42/100 minimum)", etc.). Also has `find_common_patterns(scripts)`
which detects `Entity.event` combinations that appear in 2+ scripts.

**`compose.rs` (942 lines — the biggest)** — the template engine. Takes
`SkillBlock` definitions loaded from `hydra/skill-blocks.json` (external file
not read here) and composes them into a full `.cronus` app. Has `EntityRemap`,
`FieldDef`, `Transition`, `EntityEffect`, `ComposeOptions`, `StyleConfig`,
`AuthConfig`. Four doc-stated modes: Translate / Adapt / Compose / Distill
(Distill is marked "future" in the header comment). Used by the `cronus
compose` CLI.

**`microservices.rs` (472 lines)** — splits a composed `.cronus` monolith
into per-service `.cronus` files + API gateway stub. Has `ServiceSplit`,
`GatewayDef`, `ServiceDef`, and `split_services(...)` that emits per-service
source strings including remote entity stubs for cross-service references.
Completely internal tooling.

**Does it affect user-written `.cronus`?** Only indirectly:
1. `/api/hydra/evolve` HTTP endpoint in `main.rs:392` triggers
   `hydra::evolve`, which may promote script blocks to the registry — this
   runs against the user's running scripts.
2. `cli/compose.rs` uses `hydra::compose` to generate `.cronus` files from
   templates — this is a code generator, not something user `.cronus` files
   trigger.
3. Nothing in user-written `.cronus` syntax invokes hydra. User apps never
   "call hydra" at runtime.

**Test counts per file:**
- `mod.rs`: 0
- `registry.rs`: 1 (`test_registry_add_and_search`)
- `extract.rs`: 0
- `compose.rs`: 6
- `microservices.rs`: 5
- **Total: 12**

---

## 9. Contracts (`contracts.rs`, 614 lines + `contracts_generated.rs`, 702 lines)

**Verdict: REAL.**

Yes, `SectionContract` is a real Rust struct (line 35):

```rust
pub struct SectionContract {
    pub name: &'static str,
    pub layer: Layer,               // Core/Stdlib/Pattern
    pub stability: Stability,       // Stable/Experimental/Internal/Deprecated
    pub requires_title: bool,
    pub requires_items: bool,
    pub min_items: usize,
    pub config_keys: &'static [&'static str],
    pub structural_keys: &'static [KeyDef],   // KeyDef { name, required }
    pub entity_binding: bool,
    pub on_unknown_key: Fallback,             // Warn/Error/Ignore
    pub on_missing_required: Fallback,
}
```

**Hardcoded contracts in `contracts.rs`**: 20 (`TABLE`, `FORM`, `CARD`, etc.)
**Generated contracts in `contracts_generated.rs`**: 44 (auto-gen header:
*"Auto-generated by `cronus spec codegen --structs`. Do not edit manually.
Edit .spec.toml files instead. Source: specs/"*).

**Resolution order** (`ContractRegistry::get`, line ~405): with feature
`generated-contracts`, generated takes precedence; fallback to hardcoded.
Unknown sections produce `ParseWarning::UnknownSection`.

**Aliases** (line ~423): hardcoded map `stats → kpi`, `stat-cards → kpi`,
`status-card → card`, `activity-table → table`, `team-list → table`,
`policies → form`, `live-keys → card`, `test-keys → card`, `webhooks →
card`, `quick-links → links`, `promo → card`, `info-bar → alert`, `edge →
features`, `bento → features`, `features-split → features`, `product-grid
→ card`.

**`validate_section(section, entity_fields) -> Vec<ParseWarning>`** (line 500):
- Resolves aliases first (emits `AliasUsed`)
- Checks `min_items` (skipped when binding is present — data comes from DB)
- Validates structural keys vs item keys; unknown keys hit `on_unknown_key`
  fallback; missing required keys hit `on_missing_required`
- Has two-namespace validation: structural keys from contract + entity
  fields from binding; skips unknown-key check when entity fields aren't
  threaded through (TODO comment at line 534)
- Checks `config_keys` and allows common universal keys (`title`, `subtitle`,
  `style`, `entity`)

`ParseWarning` enum has 6 variants: `UnknownSection`, `UnknownKey`,
`MissingRequired`, `AliasUsed`, `MinItemsViolation`, `UnknownConfig`.

**Who reads contracts?** grep of `crate::contracts::`:
- `ui/mod.rs` — uses them during render
- `cli/spec.rs` — the codegen CLI (`cronus spec codegen`)
- `testing.rs` — spec tests
- `contracts_generated.rs` — itself

Line numbers unimportant here — the import is low-traffic, which means the
contracts system is less wired-in than e.g. binding.rs. No reference from
parser — the parser does not enforce contracts during parse; they're
enforced during a later UI-render pass or via `spec` CLI only.

**`contracts_generated.rs` generator:** `cli/spec.rs` reads `specs/*.spec.toml`
and emits `contracts_generated.rs` via `cronus spec codegen --structs`. Not
verified further here.

**(e) Test count: 0** in `contracts.rs` itself.

---

## 10. AI Context Protocol — `/api/_context`

**Verdict: REAL.**

Endpoint is registered in `server/router.rs:224`:

```rust
if path == "/api/_context" && method == Method::GET {
    return Ok(handle_context_endpoint(&state));
}
```

Handler `handle_context_endpoint` is at `server/router.rs:696` (lines 696–…).
Returns a JSON bundle containing:

- **entities**: for each non-underscore entity: `name`, `shared`, `fields[]`
  (each with name/type/required/unique/sensitive and optional `doc` from
  tags), optional `transitions`, `row_count` (live from DB)
- **pages**: route, type, title, sections_count, requires, doc
- **apis**: route name/method/... (truncated in my view)
- **memory**: from `state.memory.get_context_data()` (per line 817)
- **brain stats**: from `state.brain.stats()`

`SemanticMemory::get_context_data()` lives in `memory.rs:297` and is
documented as: *"Get context data for /api/_context (decisions + changelog +
business_rules)."* It's backed by SQLite.

This is a substantial endpoint — not a stub. It's actually wired to live DB
data (entity row counts are computed per-request via `state.db.count(name)`).

---

## 11. Zeus / Brain / Memory — quick names + purposes

**`zeus.rs` (403 lines)** — "Zeus Mode — Deep observability for CRONUS.
Every request generates a trace with granular spans showing exactly what
happened: auth → script → db queries → render → response." Exposes
`GET /zeus` (HTML dashboard) and `GET /zeus/api` (JSON). `Span { name, kind:
SpanKind, start_us, duration_us, metadata }` with `SpanKind: Auth/Db/Script/...`.

**`brain.rs` (163 lines)** — "CRONUS Hydra Brain — Pattern Learning Engine.
Tracks usage patterns from HTTP requests and provides suggestions for
optimization based on observed behavior." Wraps `CronusDB`, table
`_brain_patterns` (init is a near-no-op in this file — comment says
"We'll use a workaround … For now, we'll create a separate connection").
`suggest(_context) -> Vec<String>` takes an unused `_context` param.

**`memory.rs` (503 lines)** — "CRONUS Semantic Memory — Persistent SQLite
storage for AI sessions, decisions, anti-patterns, and changelog entries
across sessions." `SemanticMemory { conn: Mutex<Connection> }`. Used by the
`/api/_context` endpoint (`get_context_data()`), also has 6 tests.

---

## Test count summary (files in scope)

| File | Tests |
|---|---|
| constitution_check.rs | 6 |
| audit.rs | 10 |
| trust.rs | 3 |
| binding.rs | 0 |
| actions.rs | 0 |
| graphql.rs | 0 |
| sse.rs | 0 |
| contracts.rs | 0 |
| hydra/mod.rs | 0 |
| hydra/registry.rs | 1 |
| hydra/extract.rs | 0 |
| hydra/compose.rs | 6 |
| hydra/microservices.rs | 5 |
| **TOTAL** | **31** |

For reference: the broader kernel has 205 `#[test]` occurrences across 18+
files; the advanced-systems files contribute 31 of those.

---

## Final verdicts (one-line)

1. **Constitution** — SCAFFOLDED (substring matcher, not semantic; unknown rules become "info")
2. **Audit** — REAL (hash chain + SQL triggers + real write sites; no entity index)
3. **Trust** — SCAFFOLDED (scoring works; gates are always `new_clean()`; correctness is always 0 at runtime)
4. **Binding** — REAL (the AGENTS.md "only place sections touch DB" claim is accurate; group_by + parameterized SQL work)
5. **Actions** — SCAFFOLDED (7 verbs; missing update/create/log/confirm; 0 tests)
6. **GraphQL** — SCAFFOLDED (auto-gen + hand-rolled parser; no update mutation, no nesting, 0 tests)
7. **SSE Live** — REAL (push-only broadcast; real hyper streaming path in main.rs; duplicate stub in server/mod.rs)
8. **Hydra** — SCAFFOLDED (evolve/extract/compose all coded; auto-promotion hinges on gates that never fail)
9. **Contracts** — REAL (20 hardcoded + 44 generated; validator + alias map; low callsite count)
10. **AI Context Protocol** — REAL (`/api/_context` returns live entity/page/API/memory/brain data)

**Does `live true` binding really wire up SSE?** Yes — but not in
`binding.rs`. The wiring happens in `ui/mod.rs` (line 784) which injects an
inline `<script>` that opens a client-side `EventSource('/api/sse')` and
filters by entity name. Each live section = one client connection. The
inline reconnect loop has a broken closure and probably doesn't actually
reconnect, but the first connection works.
