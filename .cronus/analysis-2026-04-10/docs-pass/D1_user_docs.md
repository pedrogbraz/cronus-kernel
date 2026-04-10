# D1 — User Docs Pass (CRONUS)

> Scope: user-facing docs under `/home/zedd/Documentos/CRONUS/docs/` (index, README, getting-started, language-reference, architecture, actions, auth, data-binding, sections, ai-generation). Report is strictly descriptive — no claims about whether the kernel actually implements any of this.

## 1. Executive summary

- CRONUS is pitched as "a full-stack declarative language": one `.cronus` file + one Rust binary = running web app (SQLite + REST + HTML/Tailwind SSR + JWT auth).
- Version advertised: `0.1.0` (docs updated 2026-04-07, index rule: "Parser is law").
- One root `app` block defines port/database/stack/theme; all other blocks (`entity`, `api`, `page`, `section`, etc.) hang off it.
- Verified-numbers tables claim: 74,130 kernel LOC, 196 passing tests, 18 AST nodes, 51 section types, 16 field types, 74 specs (index.md) / 64 specs (README.md), 32 CLI commands, 30 VM opcodes, 5 compose templates.
- `sections.md` opens with "52 section types" but index/README claim 51 — internal contradiction.
- Field-type lists in language-reference and ai-generation match (16 types), but `form` section field types add `password`/`select`/`textarea`/`checkbox`/`tel` that are NOT in the entity field list — those are form-input types, not entity types.
- Auth modes listed vary: language-reference lists `public|jwt|admin|editor|internal`; auth.md lists only `public|jwt|admin|jwt [roles]`.
- `cronus new` is stated to scaffold a directory with a starter `.cronus` file; `cronus run` (optionally `cronus run 5175`) starts dev server with hot reload.
- Binding DSL supports `query all|one|count`, operators `eq/ne/gt/lt/gte/lte/contains/starts_with`, `route.id`, group intervals `day|week|month|year`, aggregates `sum|count|avg|min|max`.
- Actions DSL lists ~14 verbs (`set`, `toast`, `navigate`, `refresh`, `create`, `update`, `delete`, `validate`, `confirm`, `open`, `close`, `send`, `notify`, `log`) executed server-side, returning an "effects envelope" consumed by a ~120-line JS client runtime.

## 2. Language pitch + positioning

**index.md (line 6):**
> "One `.cronus` file + one binary = complete web application."

**README.md (line 6):**
> "CRONUS is a full-stack declarative language. One `.cronus` file defines your app, database, API, and UI. The Rust kernel parses it and serves HTTP directly — no build step, no Node.js, no React."

**architecture.md (line 3):**
> "CRONUS is a single Rust binary that compiles `.cronus` files into running web applications. No Node.js, no bundler, no framework runtime. One file in, full-stack app out."

Target audience: developers who want full-stack prototypes / internal tools from a single declarative spec. The ai-generation doc additionally positions CRONUS as an LLM-friendly target via `cronus-schema.json` + `--strict-ai`.

**Tech stack (from docs):** Rust 2021, Tokio async, Hyper 1.x HTTP, rusqlite (SQLite WAL), `jsonwebtoken` HS256, Argon2id (`argon2` + `rand_core`) for password hashing. Frontend is server-rendered HTML + Tailwind CDN + a ~120-line vanilla JS runtime (`runtime_js.rs`). The `app` block's `stack` key can declare e.g. `react + tailwind + elysia` but architecture.md explicitly says "no React, no bundler."

## 3. Getting-started flow

From `getting-started.md` + `getting-started/installation.md` + `getting-started/first-app.md`:

1. Install Rust 1.75+ and SQLite3 (installation.md).
2. Build kernel: `cd cronus-kernel && cargo build --release`. Binary at `target/release/cronus`. (installation.md)
3. Alternative install shown in top-level getting-started.md: `cargo install cronus`.
4. Verify: `cronus version` (prints `v0.1.0`), optionally `cronus doctor`.
5. Scaffold: `cronus new myapp` → creates directory + starter `app.cronus`.
6. Edit `app.cronus` (example shows `app`, `entity Task`, `page "/" type:dashboard` with `bind entity:Task { query all }`).
7. Run: `cronus run` or `cronus run 5175`, opens at `http://localhost:5175`, "Hot reload included."
8. Validate w/o running: `cronus parse hello.cronus`, `cronus stats`.

The `first-app.md` flow is literally one file (`hello.cronus`) containing `app + style + entity Lead + api /leads + page "/" type:custom + page "/signup" type:form entity:Lead`, then `cronus run 5175`. That's the "5-minute quick start."

## 4. Top-level blocks

Table assembled from `language-reference.md` unless noted.

| Block | Purpose | Key documented syntax |
|---|---|---|
| `app` | Root app config (exactly one) | `app "Name" { stack react+tailwind+elysia; port 5175; database sqlite "./data.db"; theme dark }` |
| `entity` | Data model → DB table | `entity User { name string!; email email! unique; role enum [admin,viewer]; team -> Team }` |
| `api` | REST route group | `api /users { list GET / auth:jwt; create POST / auth:admin [admin] }` |
| `page` | Route + UI | `page "/dashboard" type:dashboard requires:auth { use Topbar; section ... }` |
| `section` | UI block inside page | `section hero style:dark { title "..."; cta "Go" -> "/" primary }` |
| `style` | Global design tokens | `style { theme dark; accent indigo; cards glass; radius xl; font "Inter"; animations true }` |
| `auth` | Auth config | `auth { entity User; login email+password; session jwt expires:7d; roles [admin,editor,viewer] }` |
| `layout` | Navigation layout wrapper (docs show it implemented via `component`) | `component Sidebar { layout sidebar; items [...] }` then `use Sidebar` in pages |
| `component` | Reusable UI block | `component Navbar { layout horizontal; items [link ... , button ...]; style sticky+glass }` |
| `import` | Split across files | `import "./entities.cronus"` |
| `test` | Conformance tests | `test "User Registration" { create User { ... } -> expect 201 }` |
| `service` | Backend service config | `service api port:3001 { cors origins:[...]; rate_limit 100/min }` |
| `middleware` | Request middleware | `middleware rateLimit { applies_to ["/api/*"]; limit 100/min; per user }` |
| `on <event>` | Entity lifecycle handler | `on user.registered { send email "welcome"; create Invoice { plan:"starter" } }` |
| `env` | Env-specific vars | `env production { DATABASE_URL env(DATABASE_URL) }` |

**Blocks NOT documented in the scoped files** (the prompt asks about these — they are absent from user-facing docs):

- `webhook` — not documented as a top-level block (only a `webhooks` **section** renderer, sections.md ~L858).
- `constitution` — absent.
- `transition` — absent.
- `deploy` — absent as a block (only `cronus deploy` CLI command).
- `worker` — absent.
- `compose` — absent as a block (only `cronus compose` CLI command; README verified-numbers mentions "Compose templates 5").
- `use` — not a top-level block; it's an inline directive inside `page` to inject components (`use Topbar`).
- `merge` — absent.

## 5. Section types

`sections.md` documents **the following sections** (it opens "52 section types" but index/README say 51). List extracted from section headings in sections.md:

| Section | Category (inferred) | Notes |
|---|---|---|
| `table` | data | Columns via `item ... column:true`, supports `bind entity:` |
| `form` | forms | `entity:`, `action:"/path"`, `method:POST`, `field` blocks, `action` buttons |
| `kpi` | data | Cards, alias `stats` renders identically |
| `stat-cards` | data | "Similar to kpi but with dedicated renderer" |
| `chart` | data | `type:bar|line|donut`; note says chart "renders via generic section … not a graphical chart" |
| `kanban` | data | `column:true` marks column headers, items fall under preceding column |
| `hero` | marketing | `badge`, `title`, `subtitle`, `bullets`, `cta` with `-> "/x" primary` |
| `features` | marketing | `cols`, `style:cards|bento`, `icon` |
| `pricing` | marketing | `plan "Name" $29/mo [features]`, `featured` keyword |
| `timeline` | data | Vertical entries |
| `progress` | feedback | Percentage steps |
| `modal` | layout | `title:`, `field`, `action` |
| `sheet` | layout | Slide-in panel |
| `card` | layout | Generic container |
| `alert` | feedback | `style:success|warning|error|info`; note "renders via generic section. Styled as a card, not a colored banner." |
| `cta` | marketing | Title + buttons |
| `faq` | marketing | Alias `accordion` |
| `testimonial` | marketing | Quote + attribution |
| `footer` | nav | `nav`, `copyright`, `footnote` |
| `topbar` | nav | `brand`, `nav`, `cta` |
| `sidebar` | nav | `item "X" -> "/x" icon:...` |
| `page-header` | layout | Title + subtitle + action |
| `bento` | marketing | `cols:12`, `span:N` per item |
| `features-split` | marketing | Two-column |
| `checkout` | forms | Card/expiry fields |
| `product-grid` | data | Products with `price`/`interval` |
| `team-list` | data | Member list w/ roles |
| `activity-table` | data | Audit log with `columns`/`row`/`status` |
| `status-card` | data | Status indicator |
| `policies` | data | Toggles `value:` / `toggle:on|off` |
| `toast` | feedback | `variant:success|error|info` |
| `command` | nav | Command palette |
| `dropdown` | nav | Trigger + items |
| `notifications` | feedback | Notification center |
| `trusted` | marketing | "Trusted by" logo bar |
| `promo` | marketing | Promotional banner |
| `info-bar` | feedback | Icon + title + link |
| `links` / `quick-links` | nav | Link list |
| `empty` | feedback | Empty state with CTA |
| `error` | feedback | Error state |
| `not-found` / `404` | feedback | 404 content |
| `skeleton` / `loading` | feedback | Loading placeholder |
| `edge` | marketing | Infra section |
| `live-keys` / `test-keys` | data | API-key display |
| `webhooks` | data | Webhook config card |
| `pagination` | data | Page controls |
| `filters` | data | Filter toolbar |
| `dark-mode` | feedback | Theme toggle |

`first-dashboard.md` additionally uses these section names which are NOT present as headings in sections.md:

- `current-plan`
- `usage-status`
- `billing-stats`
- `recent-invoices`

These are used in an example without being catalogued, which is worth flagging.

Section types grouped by the prompt's categories (docs don't explicitly taxonomize, this is inferred):

- **Data**: `table`, `kpi`, `stat-cards`, `chart`, `kanban`, `timeline`, `product-grid`, `team-list`, `activity-table`, `status-card`, `policies`, `live-keys`, `test-keys`, `webhooks`, `pagination`, `filters`
- **Forms**: `form`, `checkout`, `modal` (dual-purpose)
- **Nav**: `sidebar`, `topbar`, `footer`, `command`, `dropdown`, `links`/`quick-links`
- **Feedback**: `alert`, `toast`, `progress`, `notifications`, `info-bar`, `empty`, `error`, `not-found`/`404`, `skeleton`/`loading`, `dark-mode`
- **Marketing**: `hero`, `features`, `pricing`, `cta`, `faq`/`accordion`, `testimonial`, `bento`, `features-split`, `trusted`, `promo`, `edge`
- **Layout**: `card`, `sheet`, `page-header` (+ modal if treated as layout)

## 6. Field types

From `language-reference.md` §entity (line 58, "16 field types"), reconfirmed by `ai-generation.md` §Field Types and `architecture.md` DB mapping:

| Type | Keyword | DB mapping (architecture.md) |
|---|---|---|
| String | `string` | TEXT |
| Text | `text` | TEXT |
| Email | `email` | TEXT |
| Url | `url` | TEXT |
| Slug | `slug` | TEXT |
| Phone | `phone` | TEXT |
| Number | `number` | INTEGER |
| Money | `money` | INTEGER (centavos) |
| Percentage | `percentage` | REAL (architecture.md says INTEGER — see §12) |
| Boolean | `boolean` | INTEGER (0/1) |
| Date | `date` | TEXT |
| Ulid | `ulid` | TEXT |
| Json | `json` | TEXT (JSON string) |
| Enum | `enum` | TEXT |
| Ip | `ip` | TEXT |
| Relation | `->` | TEXT (FK) |

Total: 16. Matches the README "verified numbers" count.

Array suffix `[]` (e.g. `string[]`) is documented as a modifier, not a distinct type.

**Field-input types in `form` sections** (sections.md §form, line 74) — these are form UI widgets, not entity types: `text`, `email`, `password`, `select`, `textarea`, `checkbox`, `number`, `date`, `url`, `tel`. Note: `password` and `select`/`textarea`/`checkbox`/`tel` are **form inputs only** — they do not appear in the entity field-type list.

## 7. Modifiers

Documented field modifiers (language-reference.md lines 79-90 + ai-generation.md lines 109-115):

| Modifier | Effect |
|---|---|
| `required` | Must have a value |
| `unique` | Unique constraint |
| `sensitive` | Excluded from API list responses / SSR output |
| `optional` | Nullable |
| `searchable` | Included in search queries |
| `index` | Database index |
| `featured` | Shown in list/card views |
| `formatted` | Apply display formatting |
| `[]` suffix | Array field |
| `!` (bang after type) | Shorthand seen in `index.md` and README (`name string!`, `email email!`) — documented by example but not in the modifier table |

The prompt mentions `default:`, `min:`, `max:`, `match:` — **none of these appear in any of the scoped docs**. They are not documented as field modifiers.

## 8. CLI commands

Three lists exist across docs — they do not agree on count.

**getting-started.md (line 55, 9 commands):**
`cronus run`, `cronus build`, `cronus parse`, `cronus test`, `cronus dump <file>`, `cronus new <name>`, `cronus seed`, `cronus spec validate`, `cronus generate`.

**getting-started/installation.md (lines 66-80, "32 total", lists 13):**
`run`, `build`, `parse`, `new`, `deploy`, `doctor`, `stats`, `export`, `test`, `compose`, `generate`, `dump`, `version`.

**architecture.md (lines 143-159, 16 commands listed):**
```
cronus run <file>     — Parse, migrate DB, start server
cronus build <file>   — Parse + validate (no server)
cronus parse <file>   — Dump AST as JSON
cronus test <file>    — Run auto-generated CRUD tests
cronus seed <file>    — Insert sample data
cronus new <name>     — Scaffold new project
cronus doctor <file>  — Check project health
cronus stats <file>   — Show entity/page/route counts
cronus validate <file>— Run contract validation
cronus spec <name>    — Show spec for a section type
cronus deploy <file>  — Build + deploy
cronus dump <url>     — Dump external site to .cronus
cronus generate <file>— Generate code from .cronus
cronus compose <file> — Run compose blocks
cronus export <file>  — Export project
cronus version        — Print version
```
Flags: `--strict`, `--strict-ai`.

Aggregate unique commands across docs: **run, build, parse, test, seed, new, doctor, stats, validate, spec, deploy, dump, generate, compose, export, version, spec validate** (17 unique). README & index both claim 32 commands "verified against main.rs dispatch," but only ~16-17 are actually enumerated anywhere in the scoped docs.

## 9. Auth / HTTP / routing

**HTTP methods documented** (first-api.md line 61, language-reference.md §api):
`GET`, `POST`, `PATCH`, `DELETE`. No `PUT`, `OPTIONS`, `HEAD` in the scoped docs.

**Route syntax** (language-reference.md):
```
api /prefix {
  name  METHOD  /path  auth:level [role1, role2]
}
```
`:id` is a path parameter (first-api.md).

**Auth modes — inconsistent across docs:**

- `language-reference.md` line 124: "Auth levels: `public`, `jwt`, `admin`, `editor`, `internal`. Optional role guard with `[role1, role2]`."
- `auth.md` §Auth Modes: `auth:public`, `auth:jwt`, `auth:admin`, `auth:jwt [role1, role2]`. No mention of `editor` or `internal` as first-class modes.
- `first-app.md` line 115: "Auth levels: `auth:public`, `auth:jwt`, `auth:admin`."
- `first-api.md` line 63: "`auth:public`, `auth:jwt`, or `auth:admin`."
- `auth.md` also uses `requires:auth` and `requires:role(admin)` on **pages** (not the same as API `auth:` modifier).

`auth.md` claims JWT = HS256, default expiry 7 days, secret from `JWT_SECRET` env (fallback `"cronus-dev-secret-change-me"`), transport via `Authorization: Bearer` header AND HttpOnly cookie. Argon2id for password hashing (`argon2` + `rand_core`).

**Auto-generated endpoints** (when User entity + auth block present):
- `POST /api/auth/signup`
- `POST /api/auth/login`
- `POST /api/auth/logout` (mentioned in language-reference.md, absent from auth.md's table)
- `GET /api/auth/me`
- `POST /api/auth/register` (mentioned in language-reference.md line 322 — note this is a **second name** for signup, see contradictions)

## 10. Bindings

From `data-binding.md`:

```
bind entity:<Name> {
  query <all|one|count>
  where <field> <op> <value>
  order <field> <asc|desc>
  limit <N>
  offset <N>
  group <field> by:<interval>
  aggregate <fn(field)>
}
```

**Query types**: `all` → rows, `one` → single record, `count` → integer.

**Operators**: `eq`, `ne`, `gt`, `lt`, `gte`, `lte`, `contains`, `starts_with`.

**Value types**: string literal, number, boolean, `route.id`, `route.slug`, `auth.id` (marked **"planned"**).

**Group intervals**: `day`, `week`, `month`, `year`, or raw (no interval).

**Aggregate functions**: `sum(field)`, `count`, `avg(field)`, `min(field)`, `max(field)`.

**ResolvedData variants** (per renderer): `Rows(Vec<Value>)`, `Record(Option<Value>)`, `Count(u64)`, `None`.

`language-reference.md` has a DIFFERENT binding keyword list (line 219-241): uses `query all|recent|custom`, `order`, `limit`, `where`, `group`, `aggregate`. Note `recent` and `custom` are not in data-binding.md's query-type list. Also uses `group category` with no `by:` interval. See §12.

## 11. Actions

From `actions.md` §Verbs + `language-reference.md` §Action syntax:

| Verb | Syntax | Source |
|---|---|---|
| `set` | `set field "value"` | both |
| `toast` | `toast "msg" style:success` | both |
| `navigate` | `navigate "/path"` | both |
| `refresh` | `refresh self` (or `page` / entity name) | both |
| `create` | `create Entity { field: "v" }` | both |
| `update` | `update Entity/:id { field: "v" }` | lang-ref; `update Entity` in actions.md |
| `delete` | `delete Entity/:id` / `delete Entity` | both |
| `validate` | `validate all` / `validate field "rule"` | both |
| `confirm` | `confirm "Are you sure?"` | both |
| `open` | `open "modal-id"` / `open <target>` | both |
| `close` | `close "modal-id"` / `close <target>` | both |
| `send` | `send email "template"` | lang-ref only |
| `notify` | `notify admin "event"` | lang-ref only |
| `log` | `log "message"` | lang-ref only |

**Events**: `click`, `submit`, `change` (actions.md §Events).

**Toast styles**: `success` (green), `error` (red), `warning` (amber), `info` (blue, default).

**Flow** (actions.md §How It Works): renderer adds `data-cronus-action` / `data-cronus-section` attrs → JS micro-runtime (~120 lines) intercepts → optional `window.confirm()` → POST `/_action/{section}` with action JSON → kernel runs `execute_action_validated()` → validates against entity schema → returns `{ ok: true, effects: [...] }` envelope → client applies toast/navigate/refresh/modal.

**Effects envelope example** (actions.md lines 115-123):
```json
{ "ok": true, "effects": [
  { "type": "toast", "target": "Order approved", "style": "success" },
  { "type": "refresh", "target": "", "style": "" }
]}
```

## 12. Contradictions and suspicious claims

1. **51 vs 52 section types.** README.md/index.md/sections reference = 51. `sections.md` line 6 literally says "52 section types." Internal mismatch.
2. **74 vs 64 spec files.** `index.md` "Verified Numbers" says 74 specs. `README.md` same table says 74, but `architecture.md` §Spec System says "Total: 64 spec files" (13 core + 31 stdlib + 20 patterns). `ai-generation.md` also says "64 `.spec.toml` contracts."
3. **Auth modes disagree.** language-reference.md lists `public|jwt|admin|editor|internal`; auth.md/first-api.md/first-app.md list only `public|jwt|admin` (+ `[roles]`). `editor` and `internal` are orphaned.
4. **`register` vs `signup`.** language-reference.md line 322 says the auth block auto-generates `POST /api/auth/register`. auth.md §Built-in Endpoints lists `POST /api/auth/signup` instead (no `/register`). Two different names for the same endpoint.
5. **`percentage` DB mapping.** language-reference.md table says `percentage` → REAL; architecture.md §Database says `percentage` → INTEGER. Directly contradictory.
6. **`query` vocabulary.** language-reference.md says query options are `all | recent | custom`. data-binding.md says query types are `all | one | count`. `recent`, `custom`, `one`, `count` are fragmented across docs.
7. **`group` syntax.** language-reference.md shows `group category` (bare field). data-binding.md shows `group created_at by:month` (field + interval). Minor but inconsistent.
8. **Chart & alert "render via generic section".** sections.md explicitly notes (line 149 for chart, line 392 for alert) that these sections fall back to generic card rendering — i.e. the docs openly admit two of the 51 sections don't render distinctively. Surprising given the "51 section types" claim.
9. **Architecture LOC vs verified numbers.** architecture.md says "~30,000 lines across 37 modules"; README/index "Verified Numbers" table says 74,130 kernel LOC. ~2.5x mismatch.
10. **Rust files.** README.md says "Rust files: 128." architecture.md says "37 modules." Not exactly the same thing, but the wording doesn't reconcile these.
11. **32 CLI commands claim.** README/index/installation all claim 32 commands verified against `main.rs`. Only ~16 are enumerated anywhere in the docs. Half are undocumented by name.
12. **`cronus new` output is unclear.** getting-started.md says it "generates `app.cronus` with a starter template." installation.md says it "scaffolds a directory with a starter `.cronus` file." Close but slightly different.
13. **`cronus run` arg.** getting-started.md shows `cronus run` with no args (uses port from file). first-app.md/first-api.md show `cronus run 5175` (positional port arg). index.md shows `cronus run 5175`. Two call conventions documented.
14. **`cargo install cronus` vs build-from-source.** getting-started.md (line 8) says `cargo install cronus` (implying it's on crates.io). installation.md only covers `cargo build --release` from a local clone. Either the crate exists or it doesn't — docs are ambiguous.
15. **`first-dashboard.md` uses undocumented sections.** `current-plan`, `usage-status`, `billing-stats`, `recent-invoices` are demoed as if they're real section types but don't appear in sections.md's catalog.
16. **"No React" vs `stack react`.** architecture.md line 3: "No Node.js, no bundler, no framework runtime." But app blocks in examples routinely declare `stack react + tailwind + elysia`. Docs never explain what the `stack` key actually does if React/Elysia aren't used.
17. **`auth.id` "planned".** data-binding.md explicitly flags `auth.id` as "(planned)" — i.e. openly documented as not-yet-implemented. Everything else in the same table is presented without a stability marker, so this is the one documented gap.
18. **`send email`, `notify admin`.** These action verbs appear only in language-reference.md, not in actions.md's verb table. May or may not exist.
19. **`layout` as a top-level block.** language-reference.md §layout (line 344) claims `layout` is a top-level block, but the example then immediately shows the syntax as `component Sidebar { layout sidebar; ... }` — i.e. `layout` is actually a keyword INSIDE `component`, not a top-level block. The heading is misleading.
20. **`webhook` block absent.** Prompt asks about it; docs only expose `webhooks` as a section renderer, not as a top-level block. Same for `constitution`, `transition`, `worker`, `merge`, `compose` (block form) — all asked-about but absent.

## 13. Files read

| File | Lines |
|---|---|
| `/home/zedd/Documentos/CRONUS/docs/index.md` | 96 |
| `/home/zedd/Documentos/CRONUS/docs/README.md` | 133 |
| `/home/zedd/Documentos/CRONUS/docs/getting-started.md` | 72 |
| `/home/zedd/Documentos/CRONUS/docs/getting-started/installation.md` | 88 |
| `/home/zedd/Documentos/CRONUS/docs/getting-started/first-app.md` | 150 |
| `/home/zedd/Documentos/CRONUS/docs/getting-started/first-api.md` | 156 |
| `/home/zedd/Documentos/CRONUS/docs/getting-started/first-dashboard.md` | 157 |
| `/home/zedd/Documentos/CRONUS/docs/language-reference.md` | 577 |
| `/home/zedd/Documentos/CRONUS/docs/architecture.md` | 236 |
| `/home/zedd/Documentos/CRONUS/docs/actions.md` | 132 |
| `/home/zedd/Documentos/CRONUS/docs/auth.md` | 229 |
| `/home/zedd/Documentos/CRONUS/docs/data-binding.md` | 286 |
| `/home/zedd/Documentos/CRONUS/docs/sections.md` | 902 |
| `/home/zedd/Documentos/CRONUS/docs/ai-generation.md` | 317 |
| **Total** | **3531** |
