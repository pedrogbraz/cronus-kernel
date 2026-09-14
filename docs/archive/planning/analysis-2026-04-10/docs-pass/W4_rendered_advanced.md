# W4 — Rendered Advanced Docs Pass
Source: `http://docs.cronus.test/<page>` (HTTP only — HTTPS returns 502 via nginx)
Fetched: 2026-04-10. Method: `curl` (WebFetch force-upgrades to HTTPS which fails).
Scope: the 8 "Advanced" sidebar pages.
> **Truncation bug**: `/hydra-system` and `/contracts` are **server-side truncated**
> mid-code-block. HTML literally ends with `# e.g.` inside a `<code>` tag, then `</main>`
> with no closing tags. Everything after that (full `SectionContract` fields, full
> `SkillBlock` fields, "Related Topics" sections) is unreachable via the rendered docs.
## 1. Hydra System — `/hydra-system`
- **Title**: "Hydra System"
- **Tagline**: "The self-evolving block ecosystem that monitors, scores, and promotes high-quality UI patterns into reusable components."
- **Subsystem**: Block registry + trust scoring + block explorer UI
- **Block syntax**: NONE. Hydra is a kernel subsystem, not a `.cronus` block. No `hydra { ... }` shown.
- **Rust sources cited**: `trust.rs`, `block_explorer.rs`, `skill_block.rs`
### Philosophy
> "Think of it as natural selection for UI components."
> "A block earns trust through real usage data — not because someone says it works, but because the data proves it."
> "This is what makes CRONUS blocks different from npm packages — blocks carry their own production metrics."
### Evolution cycle (4 stages) — verbatim
```
# Stage 1: Collect
metrics = collect_runtime_metrics()  # render time, interactivity, errors
# Stage 2: Score
scores = score_blocks(metrics)  # 0.0 - 1.0 quality score
# Stage 3: Promote
promoted = promote_above_threshold(scores, 0.85)  # promote top blocks
# Stage 4: Persist
save_to_registry(promoted)  # .cronus/hydra/blocks/
```
> "Each cycle runs after every successful build."
> "Note: Hydra requires `--ai` or `--strict-ai` build mode to be active."
### TrustProfile — 6 weighted axes
```rust
struct TrustProfile {
    gates:         TrustGates,
    evidence:      EvidencePack,
    correctness:   f64,   // weight 0.25 -- test pass rate
    reliability:   f64,   // weight 0.20 -- 1.0 - error_rate
    security:      f64,   // weight 0.20 -- 0 CVEs = 1.0
    performance:   f64,   // weight 0.15 -- p95 latency vs 500ms
    reusability:   f64,   // weight 0.10 -- contexts / 10
    observability: f64,   // weight 0.10 -- production run count
}
```
### TrustGates — 4 binary gates
`contract_valid`, `isolation_clean`, `effects_declared`, `deps_clean`.
> "Any single gate failure zeroes the entire trust score."
### EvidencePack
```rust
struct EvidencePack {
    production_runs: u64, production_errors: u64,
    p95_latency_ms: f64, contexts_used: u32,
    tests_passed: u32, tests_total: u32,
    security_issues: u32, known_failures: Vec<String>,
    last_run: u64,
}
```
### Trust tiers
| Status | Range | Rule |
|---|---|---|
| Sandbox | < 0.3 | Testing only |
| Approved | 0.3–0.6 | Staging |
| Production | 0.6–0.8 | .scriptcronus → .cronus (100+ runs) |
| Official | 0.8–1.0 | Global registry (1000+ runs) |
Endpoint `/trust`. Lineage: Manual/Promoted/Distilled/Composed/HydraImport.
Block explorer at `/blocks` (source: `block_explorer.rs`).
### Block Registry — **PAGE TRUNCATED HERE**
```rust
struct SkillBlock {
    id:        Uuid,
    name:      String,         # e.g.
```
Rest of struct, file format, promotion CLI — all unreachable.
### Guarantees claimed
- Blocks "promoted automatically" after each successful build
- Gates binary — "any single failure zeroes the score"
- No immutability or cryptographic claim here
## 2. Block Composer — `/compose`
- **Title**: "Block Composer"
- **Tagline**: "Generate complete CRONUS applications from built-in templates with a single command."
- **Block syntax**: NONE. CLI-only (`cronus compose <template>`).
- **Templates**: saas-billing, blog, crm, helpdesk, ecommerce

### CLI examples
```bash
cronus compose saas-billing --name "My SaaS"
cronus compose blog --name "Dev Blog"
cronus compose crm --name "Sales CRM" --port 4000
cronus compose saas-billing \
  --name "Acme Billing" --port 4800 \
  --auth --auth-config local \
  --admin-pages --scripts --constitution --style dark
```

### FieldDef builder
```rust
FieldDef::new("email", string)
    .req()      # required
    .uniq()     # unique
    .search()   # searchable in table views
    .sens()     # sensitive — hidden in logs/exports
FieldDef::new("price", number).req().default(0)
```
> ".sens() fields excluded from API responses, logs, and data exports."

### Guarantees claimed
- `--constitution` flag adds "default rules" (no definition here)
- `.sens()` excluded from API/logs/exports (strong, unverified)
- Generated app "ready to run"

## 3. Audit System — `/audit`
- **Title**: "Audit System"
- **Tagline**: "Measure how faithfully CRONUS output reproduces reference HTML with automated fidelity scoring."
- **Subsystems bundled**: (a) fidelity audit (b) SHA-256 hash-chain DB trail (c) lint engine
- **Block syntax**: NONE.
- **Rust sources**: `audit.rs`, `lint.rs`

### Fidelity algorithm
```
ref_tokens = extract_visible_text(reference_html)
gen_tokens = extract_visible_text(generated_html)
matched = ref_tokens.intersection(gen_tokens)
score = len(matched) / len(ref_tokens) * 100
```
Build flags: `--strict-audit` (fail <90%), `CRONUS_AUDIT_REF` env var,
`cronus dump --audit`, `cronus audit --threshold 95`.

### Results file
```json
{ "runs": [{ "timestamp": "2026-04-08T14:32:00Z",
  "reference": "landing.html", "source": "app.cronus",
  "score": 94.7, "missing_tokens": ["Free trial", "Enterprise"],
  "extra_tokens": [] }] }
```

### Audit Trail (Hash Chain) — THE CRYPTOGRAPHIC CLAIM
> "Every INSERT, UPDATE, and DELETE on entity tables is recorded in a **tamper-proof**,
> SHA-256 hash-chained audit log backed by SQLite. Entries **cannot be deleted or
> modified** (enforced by database triggers)."
Schema `_audit_log`:
```
id INTEGER PRIMARY KEY,  timestamp TEXT,  action TEXT (INSERT|UPDATE|DELETE),
entity TEXT, record_id TEXT, user_id TEXT,
data TEXT, prev_data TEXT,  prev_hash TEXT,
hash TEXT  // SHA-256(timestamp|action|entity|record_id|data|prev_hash)
```
Endpoints: `/api/audit/trigger`, `/api/audit/results`, `/api/audit/trail/verify`.
CLI: `cronus verify-audit ./data.db [--verbose] [--entity User]`.
> "SQLite triggers prevent DELETE and UPDATE on `_audit_log`. Any attempt raises ABORT."

### Lint — 9 rules (<5ms at parse time)
`no-dead-text`, `no-dead-links`, `no-dead-ui`, `no-fake-state`, `no-orphan-reload`,
`no-hardcode-user`, `no-sensitive-render`, `form-submit-handler`, `shared-entity-auth`.
`--strict` promotes warnings to errors.

### Guarantees claimed
- **tamper-proof** hash-chained log (cryptographic)
- **immutable** via DB triggers
- Lint runs in **<5ms** (specific perf claim)
- `.sens()` fields blocked from templates by `no-sensitive-render`

## 4. Constitution — `/constitution`
- **Title**: "Constitution"
- **Tagline**: "Define unbreakable rules enforced at compile time to guarantee application integrity and security."
- **Block syntax introduced**: ✅ **`constitution { must "..." never "..." }`** — first-class block
- **Subsystems on page**: constitution + objective kernel + Brain + orchestrator (4 bundled)
- **Rust sources**: `brain.rs`, `orchestrator.rs` (constitution_check.rs implied)

### Definition (this is what was missing in flat docs)
> "The constitution is a set of unbreakable rules enforced at compile time. When you define
> a constitution block, the compiler validates every entity, page, and section against these
> rules before emitting any output. Violations cause build failures."

### Syntax — verbatim
```
constitution {
  must "all price fields use centavos"
  must "all forms have validation"
  must "all pages have auth"
  never "hardcode user data"
  never "expose sensitive fields in API"
  never "use inline styles"
}
```

### Rule → check mapping
| Rule pattern | Validation check |
|---|---|
| prices in centavos | Number fields named `*price*` need centavos comment |
| forms have validation | Form sections must have required fields |
| pages have auth | Every page must have an `auth` block |
| hardcode user data | No string literals matching user data patterns |
| expose sensitive | `.sens()` fields excluded from API routes |
| inline styles | No `style=` attrs in template HTML |
Error format:
```
ERROR: Constitution violation at line 42
  Rule: "never hardcode user data"
  Found: hardcoded email "admin@example.com" in section login
  Fix: Use entity binding or environment variable
```

### Objective Kernel (mission-driven)
```
constitution {
  must "optimize for conversion rate"
  must "minimize time to first interaction"
  never "add friction to checkout flow"
  never "require registration before browsing"
}
```
> "Objective kernel rules are advisory in normal mode and enforced only in `--strict` builds."

### Real-app examples
```
# E-commerce
constitution {
  must "prices in centavos"
  must "all products have images"
  must "cart persists across sessions"
  never "delete order history"
  never "expose payment tokens"
}
# SaaS
constitution {
  must "all API routes require auth"
  must "rate limit all endpoints"
  must "log all admin actions"
  never "store passwords in plaintext"
  never "allow cross-tenant data access"
}
```

### Brain — bolted onto same page
```rust
struct CronusBrain { db: Arc<CronusDB> }
// init, track, track_request, suggest, stats
// Endpoints: /api/brain/stats, /api/brain/suggest
// Hot (>10), cold (1), error rate, slow (>100ms)
```

### Service Orchestrator — also on this page
```rust
struct ServiceConfig {
    name: String, port: u16,
    service_type: ServiceType,  // Api|Auth|Worker|Frontend|Gateway
    entities: Vec<String>, config: HashMap<String,String>,
}
// Circuit breaker: 5 consecutive failures = open
```

### Guarantees claimed
- **"Unbreakable"** rules, **"enforced at compile time"**, build fails on violation
- Objective kernel **advisory by default** (honest)
- Circuit breaker opens at exactly 5 failures
- **Caveat**: rules are **fuzzy-matched English strings**, not a formal rule grammar

## 5. Live Updates — `/sse-live`
- **Title**: "Live Updates"
- **Tagline**: "Real-time data synchronization via Server-Sent Events with **zero configuration**."
- **Block syntax introduced**: Adds `live:true` flag inside `bind`. No new top-level block.
- **Subsystems on page**: SSE hub + HMR + Reactive engine
- **Rust sources**: `sse.rs` (docs say `sse_hub.rs`), `hmr.rs`, `realtime.rs`, `reactive.rs`

### Live binding
```
page "/" type:dashboard {
  section table {
    title "Live Orders"
    bind entity:Order { query all  live:true }
  }
}
```

### Full dashboard
```
page "/dashboard" type:dashboard {
  section kpi  { title "Revenue" bind entity:Order { query count | live:true } }
  section table { title "Recent Orders"
    bind entity:Order { query last:10  live:true } }
  section chart { title "Orders Over Time" type line
    bind entity:Order { query group_by:created_at | live:true } }
}
```

### SSE Hub
```rust
struct SseHub { clients: HashMap<EntityName, Vec<Sender>> }
// subscribe(entity) -> Receiver
// broadcast(entity, event)
```
> "Zero-copy broadcasting. Each entity gets its own channel."

### HMR
Watches files every 500ms → `bump_version()` → browser polls `/.cronus/version`
every 500ms → reload. Paused during SPA nav via `window.__hmrPaused`.

### Reactive engine (`CRONUS_REACTIVE_JS`, "~5KB")
> "Replaces React with HTML data attributes."
API: `CRONUS.state/get/set/update/watch`. Attrs: `data-bind`, `data-model`,
`data-show/hide`, `data-for`, `data-click`, `data-submit`, `data-fetch`, `data-poll`, `data-into`.

### Guarantees claimed
- **"Zero configuration"**
- **"Auto-init"** on DOMContentLoaded, **"no manual setup needed"**
- "No WebSocket, no polling" — then contradicted by **"polling fallback if SSE disconnects"** on same page

## 6. GraphQL — `/graphql-api`
- **Title**: "GraphQL"
- **Tagline**: "**Auto-generated** GraphQL API with queries, mutations, and a built-in playground from your entity definitions."
- **Block syntax**: NONE. Derived from `entity` blocks.
- **Endpoint**: `/api/graphql` AND `/graphql` (docs give both inconsistently)
- **Rust source**: `graphql.rs`

### Queries (auto)
```graphql
query { tasks { id title done created_at } }
query { task(id: 1) { id title done } }
query { tasksCount }
query { tasks(where: { done: true }) { id title } }
```

### Mutations (auto)
```graphql
mutation { createTask(input: { title: "Learn CRONUS", done: false }) { id title } }
mutation { updateTask(id: 1, input: { done: true }) { id done } }
mutation { deleteTask(id: 1) { success } }
```
(Second section of page contradicts this — says only create/delete generated.)

### Type mapping
| Cronus | GraphQL |
|---|---|
| number, money, percentage | Int |
| boolean | Boolean |
| ulid | ID |
| string, email, url | String |

### With variables
```graphql
mutation($input: CreateUserInput!) {
  createUser(input: $input) { id name email }
}
# Vars: { "input": { "name":"John", "email":"john@test.com" } }
mutation { deleteUser(id: "01HXYZ...") }
```

### Playground
`/api/graphql/playground` in one section, `GET /graphql` in another. Two different URLs.
Disabled in prod unless `CRONUS_GRAPHQL_PLAYGROUND=true`.

### Guarantees claimed
- **"Auto-generated"** (said 4 times)
- **"No additional configuration"**
- Entity-level perms **"enforced automatically"** when auth is on
- **Internal contradiction**: updateTask shown in Mutations but Schema Generation says "create and delete" only

## 7. Contracts — `/contracts`
- **Title**: "Contracts"
- **Tagline**: "Type-safe section contracts that validate configuration keys, enforce required fields, and catch errors at compile time."
- **Block syntax**: NONE. Contracts are **internal Rust structs**, not user blocks.
- **Rust source**: `contract.rs` (actual file: `contracts.rs` + `contracts_generated.rs`)

### What contracts actually are — verbatim
> "Section contracts define the type safety rules for each section type. A contract
> specifies which configuration keys are valid, their types, defaults, and validation
> rules. The contract system ensures that `.cronus` files are correct before rendering."

### Struct — **PAGE TRUNCATED HERE**
```rust
struct SectionContract {
    name:       String,          # e.g.
```
Rest of struct, examples, CLI, enforcement details — all unreachable.

### What contracts are NOT
- **NOT** `.spec.toml` files (contradicting AGENTS.md)
- **NOT** API/HTTP contracts between services
- **NOT** a user-writable `contract { ... }` block
- **ARE** kernel-internal schemas for built-in section types (kpi, table, form, chart…)

### Guarantees claimed (from visible fragment)
- "Catch errors at compile time"
- "Type-safe"
- Ensures `.cronus` files "are correct before rendering"

## 8. Microservices — `/microservices`
- **Title**: "Microservices"
- **Tagline**: "CRONUS generates microservices architectures **by default**. One `.cronus` file defines the full system — the kernel splits it into isolated services automatically."
- **Block syntax introduced**: ✅ **`deploy microservices { gateway / service }`**
- **Rust source**: `hydra/microservices.rs`, `deploy.rs`

### Deploy block — verbatim
```
deploy microservices {
  gateway port:5220 {
    provider "cronus"
    cors "*"
  }
  service "auth" port:5221 db:"./auth.db" {
    entities [User]
    apis [/auth, /users]
    pages ["/login", "/register"]
  }
  service "billing" port:5222 db:"./billing.db" {
    entities [Customer, Plan, Subscription, Invoice]
    apis [/customers, /plans, /subscriptions, /invoices]
    pages ["/", "/customers", "/invoices"]
  }
}
```

### Gateway block (generated)
```
app "api-gateway" { port 5220  mode gateway }
gateway {
  service "auth" port:5221 { route "/api/auth/*"  route "/api/users/*" }
  service "billing" port:5222 { route "/api/customers/*"  route "/api/invoices/*" }
}
```

### Remote entities
```
# Auto-generated in billing.cronus
entity User remote:"http://auth:5221" {
  name  string
  email email
}
# No local table — CRUD proxied to auth service
```

### Generated files
```
my-app/
  saas-billing.cronus        # Monolith source (single truth)
  docker-compose.yml         # Auto-generated orchestration
  Dockerfile                 # Shared binary image
  services/{gateway,auth,billing}.cronus
```

### docker-compose.yml (auto-generated) — verbatim
```yaml
version: "3.9"
services:
  gateway:
    build: .
    command: [cronus, run, services/gateway.cronus, 5220]
    ports: ["5220:5220"]
    depends_on: [auth, billing]
  auth:
    build: .
    command: [cronus, run, services/auth.cronus, 5221]
    volumes: [auth-data:/data]
  billing:
    build: .
    command: [cronus, run, services/billing.cronus, 5222]
    volumes: [billing-data:/data]
volumes: { auth-data: , billing-data: }
```

### --mono flag
```
cronus compose --from saas-billing --mono
# Single-process monolith, no services/ dir, no docker-compose
```

### Guarantees claimed
- **"By default"** — microservices, not monolith
- **"Automatically"** split from one source
- `docker-compose.yml` + `Dockerfile` **"auto-generated"**
- Remote entities auto-stubbed with HTTP-proxy CRUD

## Glossary of advanced concepts
- **hydra** — Kernel subsystem (not a block). 4-stage cycle after every build:
  collect → score → promote above 0.85 → persist to `.cronus/hydra/blocks/`.
  Backed by `TrustProfile` (6 weighted axes) + `TrustGates` (4 binary) + `EvidencePack`.
  Needs `--ai`/`--strict-ai`.
- **compose** — CLI `cronus compose <template>`; scaffolds from 5 built-in templates. Not a block.
- **audit trail** — SHA-256 hash-chained SQLite `_audit_log`. DB triggers block
  DELETE/UPDATE on log. Verified via `cronus verify-audit` or `/api/audit/trail/verify`.
- **fidelity audit** — Unrelated subsystem sharing the name; token-intersection score of
  generated vs reference HTML. `--strict-audit` fails <90%.
- **constitution** — Top-level `.cronus` block of `must "..."` / `never "..."` string
  clauses. Compile-time enforced by fuzzy pattern-match to validators. Build fails on violation.
- **must / never** — The two clause verbs. `must` = required behavior, `never` = prohibited
  pattern. Matched fuzzily, not a formal grammar.
- **contracts** — Internal `SectionContract` Rust structs. NOT `.spec.toml`, NOT HTTP
  contracts, NOT a user block. Define schemas for built-in section types.
- **SSE Live** — `live:true` flag in `bind`. Kernel runs `SseHub` with per-entity channels;
  injects <2KB client with polling fallback.
- **microservices** — Default output of `cronus compose`. `deploy microservices { }` block
  splits entities/apis/pages into `service` children. Kernel generates `services/*.cronus`,
  `docker-compose.yml`, `Dockerfile`, gateway. `--mono` to opt out.
- **GraphQL auto-gen** — Every `entity` → GraphQL type, input, list/get query,
  create/delete mutations. No config. Schema at `/graphql/schema`.

## Cross-reference: docs → Rust modules in `cronus-kernel/src/`

| Docs page / concept | Rust module | Exists? |
|---|---|---|
| Hydra subsystem | `src/hydra/` (mod, compose, extract, microservices, registry) | ✅ |
| TrustProfile/gates | `src/trust.rs` | ✅ |
| SkillBlock struct | `src/hydra/compose.rs` (docs said `skill_block.rs`) | ✅ (wrong filename in docs) |
| Block explorer | `src/block_explorer.rs` | ✅ |
| Fidelity audit + hash chain | `src/audit.rs` | ✅ |
| Lint rules | `src/lint.rs` + `src/hardcode_lint.rs` | ✅ |
| Constitution enforcement | `src/constitution_check.rs` | ✅ |
| Brain pattern engine | `src/brain.rs` | ✅ |
| Service orchestrator | `src/orchestrator.rs` | ✅ |
| SSE hub | `src/sse.rs` (docs: `sse_hub.rs`) | ✅ (wrong filename) |
| HMR | `src/hmr.rs` | ✅ |
| Realtime / live sections | `src/realtime.rs` | ✅ |
| Reactive engine JS | `src/reactive.rs` | ✅ |
| GraphQL auto-gen | `src/graphql.rs` | ✅ |
| Section contracts | `src/contracts.rs` + `src/contracts_generated.rs` (docs: `contract.rs`) | ✅ (wrong filename) |
| Microservices deploy | `src/hydra/microservices.rs` + `src/deploy.rs` | ✅ |
**Every advanced feature documented maps to a real Rust module.** No docs-only phantoms at
the module level. Wrong filenames in docs (3 cases): `sse_hub.rs`, `contract.rs`,
`skill_block.rs` don't exist — real files are `sse.rs`, `contracts.rs`, `hydra/compose.rs`.

## FLAG — "automatic" / "zero config" claims needing verification

| Claim | Page | Check |
|---|---|---|
| Hydra promotes blocks automatically after every build | hydra-system | Does build pipeline really call promote? |
| SSE Live = "zero configuration" | sse-live | Does `live:true` truly need nothing else? |
| Reactive engine auto-inits on DOMContentLoaded | sse-live | Does `reactive.rs` inject unconditionally? |
| GraphQL "auto-generated" from entities | graphql-api | Does `graphql.rs` scan all entities with no extra block? |
| Microservices split "by default" | microservices | Really default, or does `compose` need `--services`? |
| `docker-compose.yml` "auto-generated" | microservices | Does compose.rs write it without a flag? |
| Entity-level GraphQL perms "enforced automatically" | graphql-api | Real permission layer in auth.rs? |
| `.sens()` fields excluded from API/logs/exports | compose | Cross-check database.rs + export.rs |
| Audit log "tamper-proof" + DB triggers exist | audit | Verify SQL triggers in `audit.rs` init |
| Lint runs in <5ms | audit | Perf claim — needs benchmark |
| Circuit breaker opens at exactly 5 failures | constitution | Check constant in `orchestrator.rs` |

## FLAG — Manifesto / philosophy language (vs technical spec)

| Statement | Page | Classification |
|---|---|---|
| "Natural selection for UI components" | hydra-system | Metaphor, not spec |
| "Blocks earn trust through real usage data — not because someone says it works, but because the data proves it" | hydra-system | Manifesto |
| "Different from npm packages" | hydra-system | Marketing |
| "Unbreakable rules" | constitution | Manifesto (rules are string-matched) |
| "Guarantee application integrity and security" | constitution | Marketing — no formal semantics |
| "Mission-driven validation" (objective kernel) | constitution | Manifesto — own page admits advisory |
| "Must / never" as English strings | constitution | Pretend-formal — fuzzy match, not rule engine |
| "Real-time ... zero configuration" | sse-live | Marketing tagline |
| "CRONUS generates microservices by default" | microservices | Marketing framing of CLI default |

## Key findings (specific questions)

### Q1: Does constitution page actually define `must / never`?
**YES.** Unlike the flat docs where it was mentioned once without definition, `/constitution`
gives: full block syntax, 6 example clauses per category, compile-time enforcement contract,
error message format, pattern→validator mapping table, objective-kernel variant, and two
real-app examples (ecommerce + saas). **Confirmed defined.** Caveat: enforcement is
**pattern-matched English**, not a formal rule grammar.

### Q2: Does hydra-system actually explain "block evolution + registry"?
**PARTIALLY.** Defines: 4-stage cycle, 0.85 threshold, 4 trust tiers with run-count gates,
6 trust axes + 4 binary gates, `/trust` endpoint, lineage variants. But the **Block Registry
section is server-side truncated** mid-`SkillBlock` struct, so the promotion mechanism,
file format, and registry CLI are unreachable from the rendered docs.

### Q3: Are "contracts" the `.spec.toml` files AGENTS.md claimed?
**NO.** Contracts are **internal `SectionContract` Rust structs** (in `contracts.rs`) that
define the schema of each built-in section type (kpi, table, form, chart, etc.) so the
parser can validate `.cronus` files at compile time. They are: not TOML, not user-authored,
not HTTP/API contracts, not a user `.cronus` block. **The AGENTS.md `.spec.toml` claim is
false** and contradicted by this page. The contracts page is also truncated mid-struct,
so even the full field list is unreachable.

## Summary: well-documented vs hand-wavy
**Well-documented** (concrete syntax + algorithm + Rust source):
- Audit trail hash chain — exact schema, SQL triggers, endpoints, CLI
- Constitution — exact block syntax, rule→check mapping, error format, examples
- Microservices — full `deploy microservices { }` block, generated files, docker-compose
- GraphQL auto-gen — exact queries/mutations, type mapping table
- SSE Live — exact `live:true` syntax, SseHub struct, client behavior
- Lint — all 9 rules named with severity and catch condition
**Hand-wavy / incomplete:**
- Hydra evolution cycle — pseudocode only, no real algorithm, **truncated at SkillBlock**
- Trust scoring formula — weights listed but no actual computation shown
- Contracts — **truncated** before any real field list or example
- Objective-kernel rules — admitted advisory, no enforcement shown
- Constitution rule matcher — claims pattern match but no grammar/matcher examples
**Internal contradictions on single pages:**
- GraphQL: `updateTask` shown in Mutations, but Schema Generation section says only create/delete auto-gen
- GraphQL: two different endpoint URLs (`/api/graphql` vs `/graphql`) on same page
- SSE Live: "no polling" then describes "polling fallback"
**Server-side truncation bugs** (docs renderer, not my parser):
- `/hydra-system` — ends mid-SkillBlock struct at `# e.g.`
- `/contracts` — ends mid-SectionContract struct at `# e.g.`
