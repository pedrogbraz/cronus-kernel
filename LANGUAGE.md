# CRONUS — Language Reference

> **This is the north star.** Every claim here is verified against the Rust source at `src/` as of **2026-04-10**. When this file disagrees with `docs/`, `docs.cronus.test`, `.cronus/` SDDs, or any other documentation, **trust this file** — it is the reality check.
>
> Status markers used throughout:
> - **REAL** — fully implemented, tested, callable from user `.cronus` files
> - **SCAFFOLDED** — code exists, parses, executes, but has known holes or is partially wired
> - **STUB** — type/function exists but is a placeholder (mock data, print-only, etc.)
> - **DOCS-ONLY** — mentioned in documentation but no corresponding code

---

## 1. What CRONUS Is

**One-sentence pitch**: CRONUS is a declarative full-stack language — one `.cronus` file compiles into a single ~7MB Rust binary that serves SQLite + REST API + server-rendered HTML + auto-generated GraphQL + JWT auth + SSE + audit trail.

**Crate shape** (`Cargo.toml`):
- Name: `cronus-lang`
- Version: `0.1.0` (Experimental)
- **One binary** (`cronus`, `src/main.rs`). No `[lib]`. No workspace. No nested crates.
- Dependencies: `tokio`, `hyper 1.x`, `rusqlite` (bundled), `jsonwebtoken`, `argon2`, `serde`, `regex`, `scraper`, `base64`.

**Target**: Rust 1.78+, release profile uses `opt-level=z`, LTO, strip, single codegen unit, panic=abort.

**Guiding invariants** (enforced by lint + parser):
- Money is stored and transferred as **centavos** (integer). `money` field type = integer cents.
- `!` is the idiomatic way to mark a field required (parser also accepts legacy `required`).
- Data sections without `bind` = lint error **C003**.
- Sensitive fields never appear in HTML or auto-API (lint **C002**/**C031**).
- Owner isolation: `_owner_id` auto-injected; non-admin requests filter by it.

---

## 2. Parser Grammar — Verified From Source

### 2.1 Tokenizer reserved words

**Exactly 23 keywords** (`src/parser/tokenizer.rs:38-43`, const `KEYWORDS`):

```
app       entity    api       page      style
service   section   import    compose   use
merge     on        worker    component middleware
env       test      webhook   constitution
must      never     transition deploy
```

> **Note**: `auth`, `layout`, `define`, `style`, and `tailwind_config` are dispatched in the parser via **`TokenKind::Identifier`** checks, not via `KEYWORDS`. They work, but they are not tokenizer keywords. If you add a field or enum value named `auth` / `layout` / `define`, you can cause ambiguity.

**HTTP methods** (`src/parser/tokenizer.rs:45`, const `METHODS`):

```
GET  POST  PATCH  PUT  DELETE
```

**HEAD and OPTIONS are NOT supported.** Writing them in a `.cronus` file produces a parse error:
```
PARSE_001: expected an HTTP method (GET, POST, PUT, PATCH, DELETE), found 'HEAD' (line N, col C)
```
A lowercase method (`get`) gets the same error with `fix.replacement: "GET"`.

Rationale: HEAD is auto-handled by the runtime (returns GET headers). OPTIONS is auto-handled as CORS preflight. Exposing them to user syntax is a language-feature decision, not a bug. (The Next.js dumper previously emitted them by mistake — fixed 2026-04-10.)

### 2.2 SQL reserved word blocklist

`P041` rejects these as entity names, field names, or enum values (`src/parser/mod.rs:2312`):

```
SELECT  DROP  INSERT  DELETE  UPDATE  TABLE  FROM
```

Plus 15 more in the full list — grep `validate_identifier` for the canonical set. `P040` enforces identifier shape (64-char cap, starts with letter/underscore, no dashes, no special chars).

**Caveat**: the validator runs ONLY for entity names, field names, and enum values. Route paths, component names, and page names are NOT validated.

### 2.3 Field types — 16 verified variants

`src/parser/ast.rs` `FieldType` enum:

| Keyword          | Storage      | Rendering hint                  | Notes |
|------------------|--------------|----------------------------------|-------|
| `string`         | TEXT         | `<input type=text>`              | |
| `text`           | TEXT         | `<textarea>`                     | |
| `email`          | TEXT         | `<input type=email>`             | Pattern validated |
| `url`            | TEXT         | `<input type=url>`               | |
| `slug`           | TEXT         | slug field                       | Auto-kebab |
| `phone`          | TEXT         | `<input type=tel>`               | |
| `number`         | INTEGER      | `<input type=number>`            | |
| `money`          | INTEGER      | money formatter                  | **Stored as centavos** |
| `percentage`     | INTEGER/REAL | percent formatter                | |
| `boolean`        | INTEGER      | checkbox                         | |
| `date`           | TEXT         | `<input type=date>`              | ISO string |
| `ulid`           | TEXT         | ULID pill                        | |
| `json`           | TEXT         | JSON viewer                      | |
| `enum`           | TEXT         | select/badge                     | Requires `[a, b, c]` list |
| `ip`             | TEXT         | monospace                        | |
| **relation**     | FK column    | linked reference                 | Produced by `-> OtherEntity` arrow syntax, not a keyword |

Canonical keyword list: `FIELD_TYPE_KEYWORDS` in `src/parser/ast.rs`. Keywords are case-sensitive (`String` is not a type).

**Aliases** (`FIELD_TYPE_ALIASES`, `src/parser/ast.rs`) — accepted and stored as the canonical type:

| Alias | Canonical |
|-------|-----------|
| `int`, `integer`, `float`, `decimal` | `number` |
| `bool` | `boolean` |
| `datetime`, `timestamp` | `date` |

No other spelling is accepted (since 2026-09-14; before that, every unknown type silently became `string`):

- **`TYPE_001`** — unknown type. The fix carries the closest canonical type by edit distance (adjacent swaps count as one edit) when one is near enough: `title strin!` → `string`, `qty numbr` → `number`, `String` → `string`. Otherwise the hint lists the valid types.
- **`TYPE_002`** — field with no type on its line (`title` alone).

Both are recoverable: the parser keeps going, so one build reports every type error in the file (plus the first syntax error, if any). Relation fields accept same-line modifiers: `owner -> User required`, `tags -> Tag[] unique`.

### 2.4 Field modifiers — verified parser acceptance

**Bare modifiers** (no value):
- `!` — shorthand for required
- `required` — legacy, still accepted (test `parse_bang_backward_compat_required`)
- `unique`
- `sensitive` — hides field from auto-API and HTML
- `optional` — inverse of required
- `searchable` — tags for search index
- `index` — adds DB index (NOT `indexed`, the `d` version is not recognized)
- `featured` — UI hint
- `formatted` — UI hint

**Colon-pair modifiers** (have a value):
- `default:<lit>`
- `min:<num>`
- `max:<num>`
- `match:"<regex>"`

**NOT recognized** (despite some doc claims):
- `onupdate:` — does nothing
- `indexed` — use `index`
- `computed` — not implemented

### 2.5 Top-level block parsers — 29 helpers

`src/parser/mod.rs` exposes `fn parse_*` helpers. The top-level dispatch matches on the first token of each block and routes to the corresponding helper:

| Block syntax              | Parser function      | AST node                  | Status |
|---------------------------|----------------------|---------------------------|--------|
| `app { ... }`             | `parse_app`          | `AstNode::App`            | REAL |
| `entity Name { ... }`     | `parse_entity`       | `AstNode::Entity`         | REAL |
| `api /prefix { ... }`     | `parse_api`          | `AstNode::Api`            | REAL |
| `page "/route" { ... }`   | `parse_page`         | `AstNode::Page`           | REAL |
| `auth { ... }`            | `parse_auth`         | `AstNode::Auth`           | REAL |
| `style { ... }`           | `parse_style`        | `AstNode::Style`          | REAL |
| `layout Name { ... }`     | `parse_layout`       | `AstNode::Layout`         | REAL |
| `service Name { ... }`    | `parse_service`      | `AstNode::Service`        | SCAFFOLDED |
| `component Name { ... }`  | `parse_component`    | `AstNode::Component`      | **LIMITED** — see §6 |
| `webhook Name { ... }`    | `parse_webhook`      | `AstNode::Webhook`        | SCAFFOLDED |
| `worker Name { ... }`     | `parse_worker`       | `AstNode::Worker`         | SCAFFOLDED |
| `deploy { ... }`          | `parse_deploy`       | `AstNode::Deploy`         | SCAFFOLDED |
| `middleware { ... }`      | `parse_middleware`   | `AstNode::Middleware`     | SCAFFOLDED |
| `env { ... }`             | `parse_env`          | `AstNode::Env`            | REAL |
| `test { ... }`            | `parse_test`         | `AstNode::Test`           | SCAFFOLDED |
| `import "path"`           | `parse_import`       | `AstNode::Import`         | REAL |
| `compose { ... }`         | `parse_compose`      | `AstNode::Compose`        | SCAFFOLDED |
| `use Name { ... }`        | `parse_use`          | `AstNode::Use`            | SCAFFOLDED |
| `merge Name`              | `parse_merge`        | `AstNode::Merge`          | SCAFFOLDED |
| `define Name { ... }`     | `parse_define`       | `AstNode::Define`         | SCAFFOLDED |
| `on Name { ... }`         | `parse_event`        | `AstNode::Event`          | SCAFFOLDED |
| `tailwind_config "..."`   | inline                | Stored on `App` node      | REAL (undocumented) |

**`constitution`**: has a parser (`parse_constitution`) but NO top-level AST variant. It lives only inline inside `app { constitution { must "..." never "..." } }`. See §13.

**NO `parse_hydra`** — hydra is an internal block-evolution subsystem (§14), not a user-facing block.

### 2.6 Nested parsers inside blocks

- `parse_section` / `parse_section_item` — for `section <type> { ... }` inside pages
- `parse_transition` — for entity state machines
- `parse_effect_block` — for `on create/update/delete { ... }` inside entities
- `parse_action_block` — for action bodies (`on submit { ... }`, `on click { ... }`)
- `parse_binding` — for `bind Entity { ... }` blocks
- `parse_plan` — for deploy plans
- `parse_array` / `parse_string_array` — helpers

---

## 3. Entity System — REAL

### 3.1 Syntax

```cronus
entity Order {
  number       string!  unique
  customer     -> Customer
  items        number    default:0
  total        money!    min:0
  status       enum [pending, paid, shipped, cancelled]
  notes        text
  priority     number    min:1 max:5 default:3
  created_at   date

  transition status {
    pending -> paid | cancelled
    paid    -> shipped
  }

  on create {
    log "Order {{number}} created"
  }
  on update when status changed to paid {
    notify "payments@example.com" template "order-paid"
  }
}
```

### 3.2 Auto-generated per entity

- SQLite table with all columns, foreign keys, indexes from `index` modifier
- CRUD REST endpoints (can be overridden via explicit `api /prefix` block)
- GraphQL type, queries (`allOrders`, `order(id)`), mutations (`createOrder`, `deleteOrder` — **no `updateOrder`**; see §15)
- Owner isolation via injected `_owner_id`
- Audit log rows on INSERT/UPDATE/DELETE (§13.2)
- SSE broadcast when modified (if any section `bind`s with `live true`)

### 3.3 Transitions — REAL

State machine validation at both compile time (state names must match enum values) and runtime (prevents invalid transitions).

### 3.4 Effects (`on create/update/delete`) — SCAFFOLDED

Parser accepts the block. Runtime fires the actions but **supported action verbs are limited** (see §8). Test coverage: effect parsing has 10 unit tests in `parser/mod.rs`; effect execution has 0 tests.

### 3.5 Entity features **NOT** advertised by docs but present in code

- `EntityNode.remote_url` — field exists in the AST but is never populated by the parser. Dead.
- `EntityNode.shared` — multi-tenant flag (all users see records) — parsed and respected by binding SQL.

---

## 4. API Routes — REAL

### 4.1 Syntax

```cronus
api /orders {
  list   GET    /        auth:jwt
  detail GET    /:id     auth:jwt
  create POST   /        auth:jwt
  update PATCH  /:id     auth:jwt
  delete DELETE /:id     auth:role(admin)
}
```

### 4.2 Auth modes

Parsed in `src/parser/mod.rs::parse_api`, **enforced** in `src/api_crud.rs::authorize` (verified 2026-09-14, tests in `src/api_security_tests.rs`):

- `auth:public` — no session required (401 is never returned for this route)
- `auth:jwt` — valid session required → `401 UNAUTHORIZED` otherwise. Any unknown or empty `auth:` value behaves like `jwt` (deny by default).
- `auth:role(name)` — session + role → `403 FORBIDDEN` for other roles. Multiple roles: `auth:role(admin|manager)`. A trailing `[roles]` array on the route adds roles the same way.
- `auth:admin` — alias for `auth:role(admin)`.
- The `admin` role always satisfies role requirements. The session comes from `Authorization: Bearer <jwt>` or the `cronus_token` cookie; the role is the JWT `role` claim.

### 4.3 HTTP methods — 5 only

GET, POST, PUT, PATCH, DELETE. See §2.1. HEAD and OPTIONS are handled by the runtime automatically.

### 4.4 `api` block vs auto-generated CRUD

Both are served by `src/api_crud.rs::handle_api` (called from `main.rs::handle_request_inner` for `/api/*`).

- **Entity resolution is by exact path segment.** `/api/notes` and `/api/note` resolve `Note` (also `-es`, `y → ies`, and `-`/`_` removed: `/api/blog-posts` → `BlogPost`). `/api/notesarchive` is `404`, never `Note`.
- **With an `api` block** whose prefix resolves to the entity (e.g. `api /notes`, or `api /api/notes`): only the declared method + path shapes are served; everything else is `404 NOT_FOUND`, even for admins. `PUT` is not an alias of a declared `PATCH`. The block applies to every alias path of that entity. CRUD shapes: `GET /` list, `GET /:x` detail, `POST /` create, `PATCH|PUT /:x` update, `DELETE /:x` delete.
- **Without an `api` block** (auto CRUD): all five operations exist and every one requires a session (`401` otherwise).

### 4.5 Owner scope (enforced in SQL)

Every SELECT / UPDATE / DELETE carries the scope in its `WHERE` clause; there is no read-then-check. The rules are decided in one place, `src/access.rs` (`rest_read_scope`, `account_write_scope`, `create_owner`, `read_scope`, `write_scope`), shared by REST, GraphQL, SSR bindings, `/_form`, `/_action` and SSE. The session is always read by `access::Access::from_headers`. `User`/`Users` count as account tables on every surface, even when `auth { entity X }` names a different entity.

| Caller | Reads (list/detail) | Update / delete |
|---|---|---|
| `admin` role | all rows | all rows |
| any caller on an `auth:public` route | all rows | — (see below) |
| user, entity declared `shared` | all rows | own rows only |
| user, normal entity | rows with `_owner_id` = user id | own rows only |

- Rows owned by someone else, or with an empty `_owner_id`, are `404 NOT_FOUND` for non-admins (not `403`, so existence doesn't leak) unless the entity is `shared` or the route is public.
- Update and delete always need a session, even on a route declared `auth:public` (`401`).
- Create sets `_owner_id` to the caller's id server-side (empty for anonymous creates on a public route).
- **User entity** (`User`/`Users` or the `auth { entity X }` entity): list is admin-only (`403`); detail/update only on the caller's own record (`404` otherwise); delete is admin-only; create through generic REST is always refused (`401`/`403`) — accounts are created with `POST /api/auth/signup`. `auth:public` does not relax these rules.

### 4.6 Writes, responses, errors

- Request bodies are filtered by `authz::writable_body`: only declared fields that are not `sensitive`, not system (`id`, `_owner_id`, `created_at`, `updated_at`, …) and not privileged (`role`, `password`, `password_hash`). Other keys are silently dropped. Defaults (`default:`) are then applied server-side.
- Create validates with the entity rules (required, enum, email, min/max, pattern, unique); update validates only the fields being written, plus `transition` rules (`409`).
- Every row leaving REST passes `authz::redact_sensitive`: `sensitive` fields and `password`/`password_hash` never appear. `?search=` never matches sensitive or privileged columns.
- List: `?limit=` (default 100, max 1000), `?offset=`, `?search=` / `?q=` (SQL `LIKE` over all visible rows, wildcards escaped). `X-Total-Count` is the total number of matching rows in scope, not the page size.
- Errors are `{"error": {"code", "message"}}` with codes `UNAUTHORIZED` (401), `FORBIDDEN` (403), `NOT_FOUND` (404), `VALIDATION_FAILED` (400, or 409 for uniqueness), `INTERNAL` (500). Database driver text is logged once server-side and never returned.

### 4.7 Passwords

- Hashing: Argon2id with explicit `m=19456, t=2, p=1` (`src/auth.rs::hash_password`).
- Verification accepts only Argon2 PHC strings. Legacy SHA-256 hex / `salt:sha256` hashes were removed (2026-09-14) and no longer log in.
- Signup (`POST /api/auth/signup`) requires at least 15 characters (Unicode characters, not bytes), with no composition rules and no expiry → `400 VALIDATION_FAILED` otherwise.

---

## 5. Pages — REAL

### 5.1 Page types

```cronus
page "/dashboard" type:dashboard requires:auth { ... }
page "/"          type:custom { ... }
page "/admin"     type:custom requires:role(admin) { ... }
```

Supported `type:` values in `src/ui/page.rs`:
- `dashboard` — sidebar layout + grid (**rewritten 2026-04-10**: now delegates to `render_custom`, respects user sections, no longer rendering hardcoded Cooud template)
- `list` — default list view
- `form` — single form view
- `detail` — detail view
- `custom` — freeform, sections handle their own layout
- `checkout` — checkout flow
- `components` — component library preview

### 5.2 `requires:` clauses

- `requires:auth` — any JWT
- `requires:role(admin)` — specific role
- `requires:role(admin|editor)` — any of listed roles

Matching is by **exact route pattern** (`access::route_pattern_matches`): `requires` on `/admin` protects `/admin` only, not `/admin/users`; `/orders/:id` matches `/orders/42` but not `/orders/42/edit`. Declare `requires:` on every protected page. (Before Sprint 1 this was a prefix match.)

### 5.3 Pages + layout integration — 2026-04-10 fix

When a page has `requires:auth` AND the app has a top-level `layout Main { sidebar { ... } }` block, the renderer uses `render_layout_declarative` to wrap the page in the declarative sidebar shell — **even if the page has template sections**. This ensures every protected page shares the same design system. The routing guard lives in `src/main.rs::handle_request_inner` at ~line 1477 (look for `auth_with_layout`).

---

## 6. Components — LIMITED (docs overstate)

**What docs (`docs.cronus.test/components` index) claim**:
```cronus
component Counter(initial: number) {
  state x = initial
  template "<button @click='x++'>{{x}}</button>"
}
```
This syntax — parameters, `state`, `template` with signals and `@click`/mustache — **is not implemented**. `src/ui/component.rs` is a layout-driven preset dispatcher keyed on `comp.layout` + `comp.style` substrings. No params, no state, no template block, no signals.

**What DOES work**: reusable section presets via the `component Name { ... }` block (parser exists, AST node `AstNode::Component`), referenced from pages via `use Component` or inline. It's useful for shared card/layout snippets, not for reactive widgets.

**Button + tokens (REAL, 2026-09, dual theme)**:
- **Legacy (default):** existing Obsidian Button (`uppercase`, `--foreground`) — `style:primary` without `button+`. Demos do not change.
- **Cronus UI (opt-in):** `style { preset aurora }` + `style:button+primary+md` → `--cronus-*` from `@cronus-ui/tokens`, CONTRACT Button (`data-slot`, `destructive` alias `danger`).
- Pages without `preset` only get fallback aliases (`--cronus-primary: var(--primary)`). They do not steal `--background`. Authoring stays `.cronus`.
- **173 families (REAL opt-in):** `src/cronus_ui_widgets.rs` renders every cronus-ui family when `style` starts with that slug (`dialog`, `input`, `area-chart`, …). Legacy `style:primary` / `style:metric` is unchanged.
- **Native controls (REAL):** interactive families emit real HTML (`<input type=checkbox>`, `<dialog>`, `<details>`, `<progress>`, `<table>`, tablist). They work without a JS framework.
- **Voodoo runtime (REAL opt-in, not authoring):** `app { stack voodoo }` or `style { runtime voodoo }` injects the pinned CDN `https://cdn.jsdelivr.net/npm/voodoojs@0.13.0/dist/voodoo.full.min.js` into the **emitted HTML** and adds `v-data` / `v-model` / `@click` / `{ expr }` on those controls. `.cronus` source stays `.cronus` — no JSX, no HTML, no CSS in authoring. Off by default so Obsidian demos do not load it. Interpolations are gated: `{ count }` is never written unless the runtime is on. **LLM ingest: `llms.txt`. Full contract: `VOODOO.md`.**

If you need React-like reactivity without Voodoo, use `.scriptcronus` event handlers + `live true` binding instead (§10).

---

## 7. Sections — 39 Canonical Types (REAL)

### 7.1 The real number

Docs headline: "51 section types".
AGENTS.md historical claim: "51-way dispatcher".
`docs.cronus.test/components` index: "45 base + 10 aliases".

**Verified** via reading `src/ui/mod.rs::render_section` match arms:
- 51 explicit `=>` arms + 1 `_` default = 52 arms
- 58 distinct section type strings (aliases inlined via `|` pipes)
- After collapsing aliases via `ContractRegistry::resolve_alias()`: **39 canonical section types**

### 7.2 Canonical catalog (39)

**Marketing (7)** — no binding, pure presentation
`hero`, `features`, `pricing`, `cta`, `testimonial`, `faq`, `footer`

**Data (8)** — binding-capable, consume `bound_data`
`table`, `kpi`, `chart`, `timeline`, `progress`, `kanban`, `stat-cards`, `stats`

**Feedback (8)**
`alert`, `toast`, `skeleton`, `empty`, `error`, `notifications`, `accordion`, `dropdown`

**Navigation (5)**
`tabs`, `breadcrumb`, `sidebar`, `topbar`, `command`

**Overlay (4)**
`modal`, `sheet`, `drawer`, `popover`

**Layout (3)**
`card`, `page-header`, `divider`

**Form (2)**
`form`, `filters`

**Card (1)**
`stats-card`

**Control (1)**
`dark-mode`

**= 39 total**

### 7.3 Chart and alert — REAL, not fallbacks

Earlier analysis suspected `chart` and `alert` were falling back to generic cards. **This is wrong.**

- **`chart`** — `src/ui/section_chart.rs::render_chart_section` produces real SVG. Four subtypes via `config.type`: `bar`, `line`, `area`, `donut`. Uses Bézier paths, stroke-dasharray for donut, linear gradients.
- **`alert`** — `src/feedback.rs::render_alert` produces a real Tailwind-styled banner with tone-aware colors (info/success/warning/error), icon, title, subtitle, close button.

### 7.4 Dead match arms — 14 shadowed sections

The following section type strings are in the dispatcher match but **never reached** because `ContractRegistry::resolve_alias()` rewrites them before the match runs (`src/contracts.rs:415-443`):

`stats`→`kpi`, `stat-cards`→`kpi`, `bento`→`features`, `edge`→`features`, `features-split`→`features`, `promo`→`card`, `info-bar`→`alert`, `team-list`→`table`, `status-card`→`card`, `policies`→`card`, `activity-table`→`table`, `live-keys`→`table`, `test-keys`→`table`, `webhooks`→`links` (partial), `quick-links`→`links` (partial), `product-grid`→`features`

**Effect**: these strings work from the user's perspective (they render as their alias) but cleaning up the dead match arms would shrink `mod.rs`.

### 7.5 Bound data consumers

Only **8 renderers** actually read `bound_data`: `form`, `chart`, `kpi`, `stat-cards` (dead arm), `timeline`, `progress`, `table`, `kanban`. Everything else ignores bindings even if declared.

However: the outer wrapper at `src/ui/mod.rs:762-781` adds `data-entity="..."` and `data-bound-rows="N"` HTML attributes to every section that declares a binding, so client-side code can still discover the relationship.

---

## 8. Actions — SCAFFOLDED

### 8.1 Syntax

```cronus
on submit {
  set status "paid"
  toast "Order confirmed" success
  navigate "/orders"
}

on click confirm:"Delete this order?" {
  delete Entity route.id
  refresh
}
```

### 8.2 Supported verbs (verified in `src/actions.rs`)

**7 action verbs** are implemented:
- `set <field> "<value>"`
- `toast "<message>" <style>`
- `navigate "<path>"`
- `refresh`
- `delete <Entity> <id_expr>`
- `open <modal_name>`
- `close <modal_name>`

**Docs claim these that DO NOT exist**:
- `create Entity { ... }` — parser accepts it but no executor handles it yet
- `update Entity ...` — same
- `log "..."` — not in the action executor
- `confirm:"..."` — **only the `confirm:` MODIFIER on `on click`** works; there's no standalone `confirm` action

### 8.2.1 Server enforcement (Sprint 1)

`POST /_action/*`:
- Requires a session (401 otherwise).
- The client can only reference an action **declared in the AST**, by `action_id`. Kernel-rendered action buttons carry `data-action-id` (sha256 of the section entity and the canonical block, first 16 hex chars), and the runtime posts `{action_id, entity, id}`. The server executes **its own copy** of the instructions. A body without a matching `action_id` is 403, and client-sent instruction JSON (`action`) is ignored.
- `entity` must equal the declared section's entity (`bind X` or `entity:`).
- `set`/`delete` run on that entity only, constrained by `_owner_id` in SQL for non-admins (404 if the row is not yours), never on the auth entity for non-admins. `set` targets must be writable (`authz::writable_body`: no `id`, `_owner_id`, timestamps, `role`, `password`, or `sensitive` fields).

`POST /_form/<section_type>`:
- `entity` must be bound by a declared section of that type.
- Requires a session unless the form is explicitly public: `bind X { scope:public }` on a page without `requires:`. Anonymous rows get no `_owner_id`. The auth entity is never writable here by non-admins.
- The body goes through `authz::writable_body`; `_owner_id` is set by the server. DB errors are logged, and the client receives `{"error":{"code","message"}}`.
`PATCH /_form/<section_type>/<id>` (edit-mode forms, sent by `runtime_js`):
- Same declared-form check (403 `UNKNOWN_FORM`). Always requires a session (401), never public.
- Update is constrained by `_owner_id` in SQL for non-admins (404 if the row is not yours). Non-admins can never edit the auth entity (403).
- Only writable fields are applied (`authz::writable_body`), and only the fields sent are validated. Privileged/sensitive/system keys are ignored, and a body with nothing writable is 400. `transition` rules apply (409).
- Response: `{ok, id, record, effects}` with `record` redacted.
- Kernel forms are submitted once, by `runtime_js`. The older `render.rs`/animation handlers skip `data-cronus-form` forms when that runtime is present.

Test coverage in `actions.rs`: declared actions/forms, owner/writable checks, HTTP-level `handle_form` (PATCH owner/other/anonymous/undeclared/privileged, POST) and `handle_action` (renderer id equals server id, id-only execution, JSON rejected).

### 8.3 Effects envelope — REAL contract

When an action block runs, the API returns:
```json
{
  "ok": true,
  "effects": [
    { "type": "set",      "target": "status",  "value": "paid" },
    { "type": "toast",    "target": "root",    "style": "success", "message": "Order confirmed" },
    { "type": "navigate", "target": "/orders" }
  ]
}
```
The client-side runtime in `src/render.rs::CRONUS_RUNTIME_JS` replays the effects in order. This is the contract that lets `on submit { ... }` work without custom JS.

---

## 9. Bindings — REAL

`src/binding.rs::resolve_binding()` is **the only place** sections touch the database. Verified.

### 9.1 Query forms

```cronus
# List
bind Order { query all }
bind Order { query all where status eq:"paid" order created_at desc }

# Single
bind Order { query one where id eq:route.id }

# Filter by route param
bind Item { query all where order_id eq:route.id }

# Aggregations
bind Order { aggregate count }
bind Order { aggregate sum  field:total }
bind Order { aggregate avg  field:total }
bind Order { aggregate max  field:total }
bind Order { aggregate min  field:total }

# Grouped aggregation (for charts)
bind Order { aggregate sum field:total group_by:created_at interval:month }

# Live (SSE)
bind Order { query all live:true }
```

Aggregates without `group_by` return one value: `aggregate count` → a count, `aggregate sum|avg|min|max field:x` → a number. A KPI item with `value:bind` shows that value. It shows `0` when there is nothing to count and never renders the literal `bind`. Aggregates use the same owner scope as every other binding. `field:`, `group_by:` and `interval:` are parsed in colon form.

### 9.2 Supported operators in `where`

`eq`, `ne` (alias `neq`), `gt`, `gte`, `lt`, `lte`, `contains`, `starts_with`. Both forms are equivalent: `where status eq "active"` and `where status eq:"active"` (also `eq:auth.id`, `eq:route.id`, `gt:5`, `eq:true`). `ends_with` and `in:[...]` are **not** implemented. An unknown operator currently falls back to `eq`.

### 9.3 `group_by` intervals

For chart aggregations: `day`, `week`, `month`, `quarter`, `year`.

### 9.4 `live:true` — REAL (with caveat)

Verified in `src/ui/mod.rs:784-821`: when a section's binding has `live == true`, the renderer wraps the section in `<div id="live_{entity}" data-live-entity="{entity}">` and injects an inline `<script>` that opens `new EventSource('/api/sse')`, filters `data_change` events by entity name, and triggers a full SPA re-render via `window.__cronusNavigate(location.href, false)`.

Server side: `src/sse.rs` uses a tokio broadcast channel; `main.rs::handle_request_inner` serves `/api/sse` as a long-lived EventStream.

**Caveats**:
- One live section per page = one EventSource (no multiplexing)
- Strategy is full SPA re-render, not targeted DOM patching
- The inline reconnect loop has a broken closure (`arguments.callee.caller.toString()`) — initial connection works; reconnect after disconnect is dead

### 9.5 Who sees bound data (Sprint 1 security)

Rules live in `src/access.rs::read_scope` and are applied in `resolve_binding`:

| Viewer | Normal entity | `shared` entity | `bind X { scope:public }` | Auth entity (`auth { entity User }`) |
|---|---|---|---|---|
| No session | **nothing** | **nothing** | all rows | **nothing** |
| Signed-in user | own rows (`_owner_id`) | all rows | all rows | own row only (`id = auth.id`) |
| `role: admin` | all rows | all rows | all rows | all rows |

- A public page without a session renders empty tables and `0` counts unless the binding says `scope:public`. That is the explicit author opt-in for marketing KPIs and catalogs. It exposes every row of that entity to anyone, so use it only for data that is public by nature.
- `sensitive` fields and `password`/`password_hash` are stripped from every bound row. Aggregations over them return no data.
- `auth.id`, `auth.role` and `auth.email` in `where` resolve from the session (`auth.email` is read from the auth entity row). Without a session, or with an unknown `auth.*` ref, the binding returns no data instead of matching a literal placeholder.
- `where owner_id eq auth.id` and `where owner_id eq:auth.id` produce the same filter (fixed in Sprint 2; before, the colon form lost its value).
- Aggregations bind every filter (including the owner scope) as a parameter. There is no unparameterized fallback anymore.

Test coverage in `binding.rs`: 6 tests. Coverage in `sse.rs`: the visibility filter is tested in `access.rs`.

---

## 10. Layout — REAL (rewritten 2026-04-10)

```cronus
layout Main {
  brand "Acme"
  sidebar {
    "Dashboard" -> "/dashboard" icon:home
    "Orders"    -> "/orders"    icon:shopping_cart
    nav "Admin" -> "/admin"     icon:settings requires:role(admin)
  }
  topbar {
    search true
    notifications true
  }
}
```

### 10.1 `nav` keyword — optional since 2026-04-10

Both forms work:
- `nav "Label" -> "/route" icon:foo`
- `"Label" -> "/route" icon:foo`

Confirmed in `src/parser/mod.rs:838-844`.

### 10.2 `requires:` on sidebar items

Sidebar items can have `requires:auth` or `requires:role(x)` — non-authorized users see a filtered menu.

### 10.3 Rendering

`src/ui/layout.rs::render_layout_declarative` (~1638 LOC, rewritten this session) produces:
- Gradient sidebar with brand + auto-generated initial
- Auto-styled content area (`h1`, tables, forms styled via `.cronus-decl-main > main ...` selectors)
- 5-strategy SPA active-state management: initial load, pushState/replaceState override, popstate listener, 200ms interval poll, click handler
- Sign Out button in footer
- Responsive: hamburger below 768px, 240px fixed sidebar desktop
- **Zero hardcoded brand colors** (the runtime JS only toggles semantic `active` class; colors come from the user's style block)

Test coverage in `layout.rs`: **0 tests**. Adding regression tests is Pillar 3 of `.cronus/PLAN-2026-04-10.md`.

---

## 11. Auth — REAL

### 11.1 Syntax

```cronus
auth {
  entity User
  login email + password
  session jwt expires:24h
  roles [admin, editor, user]
}
```

### 11.2 Implementation facts

- Password hashing: **Argon2id** (`argon2` crate) — `src/auth.rs`, 7 tests
- JWT: `jsonwebtoken` crate, HS256, configurable expiry
- Auto-generated routes: `POST /auth/signup`, `POST /auth/login`, `GET /auth/me`
- Post-login redirect: `/dashboard` for non-admin, `/admin` for admin (hardcoded in `src/server/auth_pages.rs` as of 2026-04-10)
- Auto-generated HTML login/register pages (templated by `src/server/auth_pages.rs`)
- Session storage: JWT in `localStorage` + injected into `Authorization: Bearer` header by the runtime
- Sensitive fields in the `User` entity (like `password`) are blocked from auto-API responses by lint rule **C002**

### 11.3 Docs contradictions (resolved here)

The `docs.cronus.test/auth` page contradicts itself on Argon2id vs bcrypt. **The real answer is Argon2id.** `src/auth.rs` only imports `argon2`.

---

## 12. Style — REAL

```cronus
style {
  theme dark
  accent "#3b82f6"
  font "Inter"
  mono "JetBrains Mono"
  radius md
}
```

Accepted values:
- `theme` — `dark` | `light`
- `accent` — any hex string
- `font`, `mono` — any string (shipped via Google Fonts include)
- `radius` — `none` | `sm` | `md` | `lg` | `xl` | `full`

The style block feeds into `render_layout_declarative` and the auto-API docs page. It does NOT override template sections with inline `style=""` attributes.

**`tailwind_config "<JS literal>"`** is a separate top-level directive (not inside `style`) that stores a raw Tailwind config string for the layout renderer to inject. Undocumented in the flat docs but actively used by `docs/site-v2/app.cronus`.

---

## 13. ScriptCronus — Second Language, SCAFFOLDED

ScriptCronus is a small imperative language that lives in `.scriptcronus` files (or inline `script { ... }` blocks, depending on invocation). It's for event handlers, schedulers, and webhook endpoints.

### 13.1 Module layout

`src/scripting/`:
- `mod.rs` — public entry points
- `ast.rs` — AST node definitions
- `parser.rs` — hand-rolled parser, **27 inline tests**
- `vm.rs` — tree-walk interpreter, **0 tests** (prior reports said 8; verified zero)

### 13.2 Top-level blocks

- `on Entity.event { ... }` — reacts to entity lifecycle (`created`, `updated`, `deleted`)
- `schedule every:1h { ... }` — cron-like periodic task
- `endpoint GET /x { ... }` — custom HTTP endpoint outside of auto-CRUD
- `on webhook { ... }` — webhook receiver

### 13.3 Namespaces — 7 real, not 8

Verified in `vm.rs`:

| Namespace | Status    | Methods |
|-----------|-----------|---------|
| `db`      | REAL      | `query`, `insert`, `update`, `delete` |
| `log`     | REAL      | `info`, `warn`, `error` |
| `env`     | REAL      | `get` |
| `auth`    | REAL      | `current_user`, `has_role` |
| `format`  | REAL      | `money`, `date`, `percent` |
| `http`    | STUB      | `get`, `post` — returns mock JSON with Phase-4 TODO |
| `sse`     | STUB      | `broadcast` — only prints to stderr, not wired to `SseHub` |

**Docs claim 3 more namespaces that don't exist**: `cache`, `memory`, `secrets`. These are not in the tokenizer, AST, or VM. `auth` and `format` are real but missing from the docs.

### 13.4 Sandbox — REAL

`src/scripting/vm.rs:11`:
```rust
const MAX_STATEMENTS: usize = 1000;
```

Enforced before each statement via `check_limit()` and before each for-loop iteration. Running over 1000 statements kills the script with a fuel-exhausted error. Variable scope is capped at 100 entries.

### 13.5 Call syntax

Only `db.query X { ... }` works. The docs also show `db.query(X, {...})` (parenthesized form) — **the parser does not accept it**. Docs-only.

### 13.6 No user-defined functions

Scripts have no `fn`, no closures, no recursion, no try/catch. They are straight-line imperative with variables, conditionals, loops, and namespace calls.

### 13.7 Trust promotion — DOCS-ONLY

`docs.cronus.test/scripting` claims scripts reaching trust score ≥ 0.800 are "auto-promoted to canonical `.cronus`". **This is fiction.**

Verified in `src/promote.rs`:
- `promote_to_text()` exists and converts a subset of the script AST back to `.cronus` text
- It is called **only from its own 3 unit tests**
- There is no threshold check, no automatic trigger, and no pipeline hook (grepped `fire_scripts`, `main.rs`, schedulers)
- `promote_to_text` is lossy: `ExprStatement`, `sse.broadcast`, and `http.*` calls are emitted as comments
- The `0.8` constant in `trust.rs:229` is only a display label (`TrustStatus::Production`), not a gate

If you want a script to become canonical, you must manually copy the logic into a `.cronus` block. No automation exists.

---

## 14. Advanced Subsystems — Honest Verdicts

### 14.1 Audit — REAL

`src/audit.rs`, 10 tests. Every INSERT/UPDATE/DELETE on audited tables writes an audit row via SQL triggers. Each row is hash-chained: `sha256(prev_hash + row_data)`. The chain detects tampering (delete or edit an old row and the chain breaks).

Schema (simplified):
```sql
CREATE TABLE audit_log (
  id INTEGER PRIMARY KEY,
  entity TEXT, operation TEXT, row_id TEXT,
  old_data TEXT, new_data TEXT,
  actor TEXT, timestamp TEXT,
  prev_hash TEXT, hash TEXT
);
```

Surface: `cronus verify-audit` CLI verifies the chain. `cronus audit <query>` reads log rows (command exists but is not listed in the `cronus --help` dispatcher table — leaked in code only).

### 14.2 Constitution — SCAFFOLDED

`src/constitution_check.rs`, 6 tests.

Syntax (inside `app { }`):
```cronus
constitution {
  must "todo order precisa ter owner_id"
  must "nenhuma api pública pode ler campos sensitive"
  never "SELECT * em entity User"
}
```

**Enforcement**: substring match against the source code at compile time. English-string rules are NOT parsed semantically. If the rule is `"must have auth"`, the check looks for the substring `auth` in the code; it does not understand what "auth" means in context.

Good enough to catch careless changes; not good enough to be a formal spec language.

There is also an "objective-kernel" advisory variant that produces warnings rather than blocking compilation.

### 14.3 Trust — SCAFFOLDED (effectively castrated)

`src/trust.rs`, 3 tests.

**On paper**: 6-axis scoring (correctness, stability, security, clarity, adoption, dependency), 4 binary gates (builds, tests, lint, types), weighted formula producing a score 0.0-1.0.

**In runtime**:
- Gates are always constructed via `TrustGates::new_clean()` — every callsite in the kernel passes all 4 gates as true, never false
- `BlockMetrics::to_evidence()` hardcodes `tests_passed: 0, tests_total: 0`, which means the **correctness axis is always 0.0**
- Consequence: the maximum achievable runtime score is **0.75** (25 percentage points permanently locked out)

If you see trust scores in the UI, they are computed but the inputs are rigged. The system is **architecturally sound but never actually consulted** to gate anything.

### 14.4 Hydra — SCAFFOLDED

`src/hydra/` — 5 files: `mod.rs`, `compose.rs` (6 tests), `extract.rs`, `microservices.rs` (5 tests), `registry.rs` (1 test).

Purpose: block evolution — take user code blocks, extract them, score them, compose them into reusable units, optionally split into microservices.

Status: all four files have executable logic. `auto-promotion` rides on the Trust gates that never fire (§14.3), so the auto path is never exercised. Manual invocation via the CLI and via `cronus compose --from <template>` works.

**No top-level `hydra { }` block exists**. Hydra is internal tooling, not user syntax.

### 14.5 Contracts — REAL

`src/contracts.rs` + `src/contracts_generated.rs` (generated by `cargo build` when the `generated-contracts` feature is on).

- 20 hardcoded `SectionContract` structs + 44 generated = 64 total
- Each contract defines: section type name, allowed fields, required fields, default values, alias list
- `ContractRegistry::resolve_alias()` rewrites alias strings to canonical names before the dispatcher sees them
- `ContractRegistry::validate()` is called by the parser to enforce field whitelist per section type

**NOT `.spec.toml` files.** `specs/` directory does not exist. This was a historical misconception propagated by stale AGENTS.md.

### 14.6 GraphQL — SCAFFOLDED

`src/graphql.rs`, 5 tests.

- **Auth (Sprint 1):** `POST /graphql` and `GET /graphql/schema` return 401 without a session. Reads use the same owner scope as bindings (§9.5, without `scope:public`). `create<Entity>` keeps only writable fields and sets `_owner_id` on the server. `delete<Entity>` is constrained by `_owner_id` in SQL (it returns `false` for other users' rows). The auth entity is admin-only for mutations. `sensitive`/`password` fields are absent from responses, output types and create inputs. DB errors are logged and never returned.
- There is no language flag to disable GraphQL yet. It is always mounted, but authenticated.
- Auto-generates an SDL schema from entities at server startup
- Serves `POST /graphql` with a hand-rolled query parser
- Query playground at `GET /graphql` (hardcoded HTML)
- Generates: `all<Entity>`, `<entity>(id)` queries
- Generates mutations: `create<Entity>`, `delete<Entity>` — **no `update<Entity>`**
- No nested resolvers; related entities return IDs only
- No subscriptions (SSE is separate)

### 14.7 SSE Live — REAL

`src/sse.rs` + `main.rs::handle_request_inner` path for `/api/sse`.

- **Auth (Sprint 1):** `/api/sse` requires a session (401). Each `data_change` event is filtered per subscriber by `access::can_see_event`: admins get everything; signed-in users get `shared` entities and rows they own. Deletes of owner-scoped rows can no longer be looked up, so only admins receive those. `debug` events are streamed to admins only.
- tokio broadcast channel (`SseHub::subscribe_filtered`)
- Real streaming long-lived `Content-Type: text/event-stream` response
- Events: `data_change` (entity write), `debug` (request trace in DEBUG_MODE)
- Zero polling on server side — push-only
- Client reconnect is broken (see §9.4)

The stub version of SSE inside `src/server/mod.rs::CronusServer` is DEAD code (see §16).

### 14.8 AI Context Protocol — REAL

`GET /api/_context` returns structured JSON with:
- Entities (name, fields, relationships)
- Pages (routes, requires, sections)
- API blocks (prefix, routes, auth)
- Auth config
- Constitution rules
- Relationship graph (edges between entities)
- Memory (brain.rs snapshots if available)

Intended consumer: LLM agents that need to reason about the current project without re-parsing source. Called by `cronus context --for-claude`.

### 14.9 Internal dev dashboards (7 routes, all REAL)

Served by the kernel on the running app's port:

| Route           | Purpose |
|-----------------|---------|
| `/zeus`         | Request tracer with SQL spans (`zeus.rs`) |
| `/blocks`       | 3D visual block explorer (`block_explorer.rs`) |
| `/trust`        | Trust scoring dashboard (`trust.rs`) |
| `/hydra`        | Evolution cycle viewer (`hydra/mod.rs`) |
| `/docs/graph`   | Mermaid entity relationship graph (`graph.rs`) |
| `/api/_context` | AI Context Protocol (see §14.8) |
| `/.cronus/docs` | Auto-generated API docs (`server/docs.rs`) |

These are for developer use during `cronus run`, NOT exposed to end users of the deployed app.
Enforced by `src/http_guard.rs` (Sprint 1 security):

- **Production** (`cronus run --prod` or `CRONUS_ENV=production`): `/zeus*`, `/blocks`, `/trust`,
  `/hydra`, `/api/hydra/*`, `/api/debug/*`, `/api/_context`, `/api/_seed`, `/api/_health`,
  `/api/server/*`, `/api/brain/*`, `/api/schema`, `/graphql/schema`, `/docs*`, `/api/docs/*`,
  `/api/audit/trigger`, `/api/audit/results` and `source`-backed pages return **404**.
- **Dev** (`cronus run`): served without auth only when bound to loopback; with `--host 0.0.0.0`
  they require an authenticated `admin` (401/403 otherwise).
- `/api/audit/trail*` requires `admin` in every mode, and sensitive fields are redacted from
  `data`, `prev_data` and `diff`.
- `POST /api/audit/results` writes `.cronus/audit-widget-results.json` (never `/tmp`).

---

## 15. CLI — 32 Verified Commands

Verified by reading `src/main.rs` argv dispatch and `src/cli/`. There are **32 top-level verbs**.

### 15.1 Core workflow (6)

| Verb | Purpose | Key flags |
|------|---------|-----------|
| `run [port]` | Dev server with HMR (default port 5175, binds `127.0.0.1`) | `--host <ip>`, `--prod`, `--strict`, `--audit-canvas [port]` |
| `build [file]` | Parse + validate `.cronus`; one verdict, exit 0/1/2 (§15.9) | `--ai` (aliases `--machine`, `--json-errors`, `--strict-ai`), `--strict`, `--strict-audit` |
| `test` | Run auto-generated CRUD tests against running server | `--conformance` |
| `parse <file>` | Show AST | |
| `new <name>` | Scaffold new project | `--template <name>` |
| `doctor` | Health check diagnostics | |

`cronus run` network/security environment (see `src/http_guard.rs`):

| Flag / env | Default | Effect |
|---|---|---|
| `--host <ip>` / `CRONUS_HOST` | `127.0.0.1` | Bind address. `0.0.0.0` exposes the server and prints a warning. `--audit-canvas` always binds loopback. |
| `--prod` / `CRONUS_ENV=production` | dev | Production mode: internal/diagnostic routes return 404 (§14.9). |
| `CRONUS_MAX_BODY_BYTES` | `1048576` | Max request body; larger bodies get `413`. |
| `CRONUS_TRUSTED_PROXIES` | empty | Comma list of IPs/CIDRs whose `X-Forwarded-For` is honored for rate limiting. Otherwise the socket peer IP is used. |
| `CRONUS_HEADER_READ_TIMEOUT_SECS` | `15` | HTTP/1 header read timeout. |

Login is also limited per account (normalized email): after 5 failures, exponential backoff (1s, 2s, 4s… capped at 15 min) with `429` + `Retry-After`.

### 15.2 Generation (4)

| Verb | Purpose |
|------|---------|
| `compose` | Compose from template — `--from saas-billing\|blog\|crm\|helpdesk\|ecommerce` |
| `dump <path>` | Convert source project to `.cronus` (see §15.6) |
| `seed` | Seed database with fixture data |
| `generate` | (alias for compose in some paths) |

### 15.3 Deploy (2)

| Verb | Purpose |
|------|---------|
| `deploy` | Deploy to target platform (docker, cloudflare) |
| `export` | Export project IR to `cronus-project.ir.json`, OpenAPI, etc. |

### 15.4 Diagnostics (4)

| Verb | Purpose |
|------|---------|
| `stats` | Project statistics — blocks, LOC, coverage |
| `validate` | Validate without build |
| `version` | Print version |
| `graph` | Dump Mermaid entity graph to stdout |

### 15.5 Spec subcommand (4)

`cronus spec <subcommand>`:
- `spec validate`
- `spec codegen --structs`
- `spec codegen --docs`
- `spec codegen --ai-protocol`
- `spec list`

### 15.6 Advanced (16)

`debug`, `brief`, `context`, `sync`, `handoff`, `lease`, `drift`, `segment`, `reconcile`, `review`, `timeline`, `status`, `changelog`, `memory`, `verify-audit`, `audit`

### 15.7 Dump targets — verified

`cronus dump` auto-detects the source type and emits `.cronus`:

| Target             | Auto-detect | Status |
|--------------------|-------------|--------|
| Next.js project    | `next.config.*` or `package.json` contains `"next"` | REAL (fixed HEAD/OPTIONS on 2026-04-10) |
| VINEXT project     | `package.json` contains `"vinext"` | REAL |
| Prisma schema      | `.prisma` file | REAL |
| OpenAPI spec       | `.json` or `.yaml` with `openapi:` | REAL |
| HTML page          | `.html` file | REAL |
| Generic project dir| fallback | REAL |
| **TypeScript**     | `src/dump/typescript.rs` exists | **ORPHANED** — no match arm in `dump_cmd.rs` calls it |

Flags: `--audit`, `--nextjs`, `-o <file>`.

### 15.8 Dead / inconsistent CLI

- `cli::verify::cmd_verify` is imported in `main.rs:74` but has no match arm — **dead**
- `cronus audit` is referenced in code examples but not listed in the `cronus --help` table
- Some dispatch paths use short aliases (`gen` → `generate`, `clone` → some compose variant) inconsistently

### 15.9 `cronus build` diagnostics — codes, `--ai` schema, exit codes

Source: `src/cli/build.rs` (CLI), `src/cli/build_report.rs` (report + JSON), `src/cli/build_locate.rs` (name → position), `src/parser/diagnostic.rs` (parser errors). Contract tests: `tests/build_cli.rs` (runs the real binary).

**Exit codes** (both modes):

| Code | Meaning |
|------|---------|
| `0` | Valid — no errors (warnings allowed) |
| `1` | Invalid — the file was read and has at least one error |
| `2` | Usage / I/O — no `.cronus` file found, or the file cannot be read (`IO_001`) |

**Profiles.** `--ai` (and `--strict`) validate with the *strict* profile, mirroring the runtime's strict mode: contract violations (`CONTRACT_001/002/003/005/006`), lint findings, hardcoded-content findings (`LINT_020`, strict only) and constitution violations are **errors**. Plain `cronus build` uses the default profile: those contract, lint-warning and constitution findings are **warnings**. Parse, type and resolve errors are errors in both. `CONTRACT_004` (alias section name) is always a warning.

**Human mode** (`cronus build [file]`): each diagnostic goes to stderr as `file:line:col: error[CODE]: message`, followed by `    fix: replace 'x' with 'y'` or `    hint: ...`. Then exactly one verdict line: `✓ app.cronus is valid — …` on stdout, or `✗ app.cronus — build blocked: N error(s), M warning(s)` / `✗ build failed: …` on stderr.

**AI mode** (`cronus build --ai [file]`): stdout is **only** one JSON document; nothing else is printed to stdout (renderer warnings, if any, go to stderr). Aliases: `--machine`, `--json-errors`, `--strict-ai`.

```json
{
  "schema_version": 1,
  "valid": false,
  "exit_code": 1,
  "file": "app.cronus",
  "errors": [
    {
      "code": "TYPE_001",
      "severity": "error",
      "category": "type",
      "message": "unknown field type 'strin' for field 'title'",
      "location": {
        "file": "app.cronus", "line": 5, "col": 9,
        "span": { "start": { "line": 5, "col": 9 }, "end": { "line": 5, "col": 14 } }
      },
      "fix": { "action": "replace", "target": "strin", "replacement": "string", "hint": "did you mean 'string'?" }
    }
  ],
  "warnings": [
    {
      "code": "CONTRACT_004",
      "severity": "warning",
      "category": "contract",
      "message": "section type 'stats' is an alias; use the canonical name 'kpi'",
      "location": { "file": "app.cronus", "line": 9, "col": 11, "span": { "start": { "line": 9, "col": 11 }, "end": { "line": 9, "col": 16 } }, "page": "/", "section": "stats" },
      "fix": { "action": "replace", "target": "stats", "replacement": "kpi" }
    }
  ],
  "context": { "entities": 1, "pages": 1, "routes": 0, "error_count": 1, "warning_count": 1 }
}
```

Field rules:
- `location.line` / `location.col` are 1-based and always ≥ 1 (never `0`). `col` counts characters; a tab is one column. `span.end.col` is exclusive. `span` is omitted for point locations. Validators that only know names (resolve, lint, contracts, constitution) are mapped back to the referenced token, else to the enclosing section/page/entity, else `1:1`. Extra keys `page`, `section`, `entity` give context.
- `message` is English, without code or location.
- `fix.action` ∈ `replace | remove | add | edit | add_bind | add_auth | wrap_in_dynamic`. `fix.replacement` is present only when a concrete replacement for `fix.target` is derivable (type typo, alias, section-type typo, lowercase HTTP method, transition state typo, unresolved name with a close match, invalid identifier chars); otherwise `fix.hint` explains what to do.
- `rule` (optional) names the originating lint or constitution rule.
- The string API `parser::parse` renders parser errors as `CODE: message (line L, col C)`, one per line.

**Error codes:**

| Code | Severity | Meaning |
|------|----------|---------|
| `IO_001` | error (exit 2) | No `.cronus` file found / file unreadable |
| `PARSE_001` | error | Unexpected token (`expected X, found 'Y'`) |
| `PARSE_002` | error | Invalid identifier shape (P040) |
| `PARSE_003` | error | SQL reserved word as entity/field name (P041) |
| `PARSE_004` | error | `transition` on a missing or non-enum field |
| `PARSE_005` | error | `transition` state/target not in the enum |
| `PARSE_006` | error | `on <event>` in an entity is not create/update/delete |
| `TYPE_001` | error | Unknown field type (§2.3) |
| `TYPE_002` | error | Field without a type |
| `STRUCTURE_001` | error | A `page "…"` / `entity Name {` declared in the source is missing from the parsed app (an earlier statement consumed a `}`) |
| `RESOLVE_001` | error | Unresolved reference (entity, field, column, route) |
| `RESOLVE_002` | error | State-machine reference error (transition field missing / not enum) |
| `CONTRACT_001` | strict: error / default: warning | Unknown section type (fix suggests closest canonical type) |
| `CONTRACT_002` | strict: error / default: warning | Unknown key on a section item |
| `CONTRACT_003` | strict: error / default: warning | Missing required key on a section item |
| `CONTRACT_004` | warning | Alias section name (`stats` → `kpi`) |
| `CONTRACT_005` | strict: error / default: warning | Too few items |
| `CONTRACT_006` | strict: error / default: warning | Unknown section config key |
| `LINT_001`…`LINT_011` | from the rule (strict promotes warnings) | `no-dead-text`, `no-dead-links`, `bind-or-empty`, `no-sensitive-render`, `no-sensitive-select`, `no-hardcode-user`, `no-fake-state`, `no-dead-ui`, `no-orphan-reload`, `form-submit-handler`, `shared-entity-auth` (in that order); `LINT_099` = any other rule, see `rule` |
| `LINT_020` | from the finding (strict only) | Hardcoded text in rendered HTML |
| `CONSTITUTION_001` | strict: error / default: warning | Constitution `must`/`never` violation, see `rule` |

Parser diagnostics are collected in one pass: every recoverable error (`TYPE_*`) plus the first fatal syntax error. If parsing fails, validation passes do not run.

---

## 16. Dead Code You Should Not Touch

Three parallel HTTP server implementations exist in `src/server/`, two of which are **not compiled** and one of which compiles but has zero call sites:

| File | LOC   | State |
|------|-------|-------|
| `src/server/router.rs` | 1542 | **Not declared in `server/mod.rs`. Never compiled.** |
| `src/server/api.rs`    | 511  | **Not declared in `server/mod.rs`. Never compiled.** |
| `src/server/mod.rs` lines 34-447 (`CronusServer`, `handle_request`, `handle_api`) | ~414 | Compiles; `CronusServer::new` has zero callers. Dead. |

The live HTTP dispatcher is **`src/main.rs::handle_request_inner`** at line ~325 (currently ~1326 LOC inside that single function).

**Do not edit `router.rs`, `api.rs`, or `CronusServer`.** Fixes applied there never run.

**54 source files** carry `#![allow(dead_code, unused_imports, unused_variables)]` at the top. Do not add more of these — prefer actual cleanup. Removing them en masse is planned backlog work (not a language concern).

---

## 17. Testing — 205 Inline Tests, No `tests/` Directory

`cargo test` prints `205 passed; 0 failed` as of 2026-04-10.

Tests are `#[test]` functions scattered across 18 source files:

| File | Tests |
|------|-------|
| `src/parser/mod.rs` | 44 |
| `src/lint.rs` | 35 |
| `src/scripting/parser.rs` | 27 |
| `src/database.rs` | 20 |
| `src/resolve.rs` | 16 |
| `src/audit.rs` | 10 |
| `src/vm/executor.rs` | 8 |
| `src/auth.rs` | 7 |
| `src/memory.rs` | 6 |
| `src/constitution_check.rs` | 6 |
| `src/hydra/compose.rs` | 6 |
| `src/error.rs` | 5 |
| `src/hydra/microservices.rs` | 5 |
| `src/promote.rs` | 3 |
| `src/trust.rs` | 3 |
| `src/dump/detect.rs` | 1 |
| `src/hydra/registry.rs` | 1 |
| `src/dump/nextjs.rs` | **2 (NEW 2026-04-10)** |
| **Total** | **205** |

### 17.1 Modules with ZERO coverage (the danger zones)

- All of `src/ui/` (layout, page, dashboard, all sections, all of `section_*.rs`)
- `src/main.rs` — including the live HTTP dispatcher
- `src/render.rs` — the client runtime JS
- `src/binding.rs` — THE ONLY place sections touch the DB (§9)
- `src/actions.rs` — action executor
- `src/graphql.rs`
- `src/sse.rs`
- `src/scripting/vm.rs`
- `src/zeus.rs`, `src/ast_diff.rs`, `src/cli/*`
- `src/server/*` (except auth_pages.rs which has no tests either)

Rule of thumb: if you modify anything in the above list, add a regression test in the same file before committing. See `.cronus/PLAN-2026-04-10.md` Pillar 3 for the 4 pending regression tests.

---

## 18. Known Gaps — Where Docs Lie

This section is important because every other doc in the repo has at least one inaccuracy. The items below are **docs claims that are false or overstated**, verified against source.

| Claim                                             | Reality |
|---------------------------------------------------|---------|
| `specs/` directory with 74 `.spec.toml` files    | Directory does not exist. Contracts are Rust structs in `src/contracts.rs`. |
| `tests/conformance/` with 90 tests               | Directory does not exist. All tests are inline. |
| 51 section types                                  | 39 canonical; dispatcher has 52 match arms counting aliases and default. |
| 8 ScriptCronus namespaces                         | 7 implemented; docs list 3 that don't exist and miss 2 that do. |
| Trust auto-promotion at ≥ 0.800                  | No threshold gate, no auto trigger. `promote_to_text` only called in unit tests. |
| `component Name(param: type) { state ... }`      | Not implemented. `component.rs` is a layout-preset dispatcher. |
| 6-axis trust scoring used at runtime             | Gates are always `new_clean()`; correctness axis hardcoded to 0. Scores are cosmetic. |
| `update<Entity>` GraphQL mutation                 | Not generated. Only create/delete. |
| `create Entity`, `update Entity`, `log "..."` actions | Parser accepts them; executor ignores. Only 7 verbs work. |
| HEAD/OPTIONS HTTP methods                         | Tokenizer rejects them. Silent fallback never fires. |
| `src/dump/typescript.rs` as dump target          | Orphaned. No CLI flag calls it. |
| Next.js dump target                               | Not in `docs.cronus.test/dump` page. Is in code, is in CLI. |
| `src/server/router.rs` as the live dispatcher    | Not compiled. Live dispatcher is in `main.rs`. |
| `bcrypt` password hashing (in one auth page)     | It's Argon2id. |
| Port 3000 default                                 | Actual default is 5175. |
| 32 CLI commands vs 36 CLI commands                | 32 is correct. |
| `specs/` contracts as `.spec.toml`                | Rust structs. Some are auto-generated into `contracts_generated.rs`. |

---

## 19. Language Surfaces Summary

CRONUS is not a single language — it's a coordinated set of surfaces. This is the complete picture:

1. **`.cronus` declarative** — blocks, entities, pages, API, auth, layout, style, constitution. The primary surface. 90% of user code.
2. **`.scriptcronus` imperative** — event handlers, schedulers, webhooks, custom endpoints. Sandboxed, fueled, 7 namespaces. Second surface, scaffolded.
3. **Section renderers** — 39 canonical types + dead-arm aliases. Some consume bindings, most don't.
4. **Effects envelope** — JSON contract between server actions and client runtime. 7 effect types.
5. **Bindings + SSE** — `bind Entity { ... live:true }` wires up real-time without custom JS.
6. **Dev dashboards** — 7 internal routes (`/zeus`, `/blocks`, `/trust`, `/hydra`, `/docs/graph`, `/api/_context`, `/.cronus/docs`) for inspection and AI integration.
7. **CLI** — 32 verbs, including `dump` (5 real source formats + 1 orphaned), `compose`, `spec codegen`, `verify-audit`.
8. **Constitution + audit + trust** — compile-time + runtime guardrails. Audit is the only one that runs unrigged; the other two are scaffolded.
9. **Hydra block evolution** — internal tooling. No user-facing block.
10. **AI Context Protocol** — `/api/_context` endpoint for LLM integration.

---

## 20. Maintenance — How to Keep This File True

1. **If you change the parser** (`src/parser/`), verify sections §2, §3, §4, §5 are still accurate. If you add a new top-level block, add it to §2.5.
2. **If you add a section renderer** (`src/ui/mod.rs`), update §7.2. Count canonical sections with: `grep '=>' src/ui/mod.rs | wc -l` and subtract aliases.
3. **If you touch CLI** (`src/cli/` or `src/main.rs` argv), update §15.
4. **If you fix one of the "known gaps" in §18**, move the row from §18 to the body of the spec.
5. **Never add a claim here without verifying against source.** If you can't quote a file:line, don't write it.
6. **Test count in §17** should match `grep -rn '#\[test\]' src/ | wc -l`. Update after every test addition.

**Last verified**: 2026-04-10 after reading `src/parser/*`, `src/ui/*`, `src/cli/*`, `src/scripting/*`, `src/hydra/*`, `src/actions.rs`, `src/binding.rs`, `src/audit.rs`, `src/trust.rs`, `src/constitution_check.rs`, `src/graphql.rs`, `src/sse.rs`, `src/promote.rs`, `src/contracts.rs`, `src/dump/nextjs.rs`.

**Cross-references**:
- `AGENTS.md` — short cheatsheet for AI agents (this file is the long reference)
- `.cronus/PLAN-2026-04-10.md` — current work plan
- `.cronus/analysis-2026-04-10/verification/V1-V5_*.md` — raw verification reports backing this file
