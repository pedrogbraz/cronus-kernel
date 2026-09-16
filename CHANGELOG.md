# Changelog

All notable changes to CRONUS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
No release has been tagged since 0.1.0; everything below `[Unreleased]` is on
`main`. Commit hashes are given so each entry can be checked.

## [Unreleased]

### Language (define)

- **`define Name { section … }` is real.** `page { use Name }` splices those
  sections ahead of the page's own (same expansion `cronus run` already did).
  `cronus build` no longer reports `LANG_001`. Duplicate define names are
  `COMPOSE_001`. `use` of a kit `component` is unchanged.

### UI (overlays)

- **Modal overlays trap focus.** Closed-mode dialog, alert-dialog, confirmation,
  invite-dialog, sheet and drawer use native `<dialog>` with
  `command="show-modal"` / `command="close"` (inert backdrop, Esc, no JS).
  Alert/confirm use `closedby="closerequest"` so an outside click does not
  dismiss them. Menus stay `popover="auto"`.

### Language (pages)

- **`type:form` / `type:detail` / `type:list` with sections render those sections.**
  They used to ignore the body and emit a generic entity form or JS table
  (`Carregando...`). Empty pages of those types still use the generic renderer.
- **`query one` on a `section table` is a one-row table**, not an empty state.

### Language (GraphQL)

- **`update<Entity>(id, input)`** is generated. Partial update, same validation,
  uniqueness, transitions and M2M rules as REST. Other users' rows return `null`.
  The auth entity stays admin-only.
- **`app { graphql false }`** unmounts `/graphql` and `/graphql/schema` (404).
  Default remains on.

### Language (types)

- **`datetime` is not `date`.** `date` is `YYYY-MM-DD`. `datetime` (alias `timestamp`) is
  `YYYY-MM-DDTHH:MM` with optional seconds and `Z`/`±HH:MM`. Forms use `datetime-local`.
- **`file` is a real type.** Forms send a data URL; the kernel stores `/_files/<id>.<ext>`
  next to the database (512KiB). REST/GraphQL also accept an `https://` URL. `GET /_files/…`
  serves the bytes. Traversal names are rejected.
- **Reverse relations.** `Order { customer -> Customer }` yields `orders` on Customer.
  `bind Customer { expand:orders }` / REST `?expand=orders` loads the related rows
  (owner-scoped). A name that collides with a field on Customer becomes `{source}_{field}`.

### Language (composition)

- **`import` and `compose { use }` are a load graph.** Nested imports resolve relative
  to the importing file. Each path is loaded once (cycles skip, they do not error).
  Missing files are `COMPOSE_002` — they used to `eprintln` a warning and continue.
- **Union, not last-wins.** Duplicate entity name, page route, `app`, `auth`, `style`,
  layout name, api prefix, component name, webhook entity, or env variable is
  `COMPOSE_001`. The first declaration is kept; every later collision is reported.
  Exactly one `app {}`.
- **`parse_directory` no longer concatenates sources** (that double-loaded a file that
  was also `import`ed). `cronus run` with several `*.cronus` files unions the directory
  through the same graph. `cronus build path.cronus` follows that file's imports;
  `cronus build` with no path and several files in cwd unions like `run`.
- **`compose` is not `LANG_001`.** `compose App { use entities }` loads `entities.cronus`.
  `cronus compose --from` is still hydra template generation, not this primitive.

### Language (query)

- **`ends_with` and `in:[…]`** are real `where` operators (parameterized `LIKE` / `IN`).
  `in` requires a list; `in:"paid"` is still `BIND_001`. An empty list matches nothing
  (`1=0`), it is not dropped. Unknown operators still error; they never become `eq`.
- **`bind { expand:tags }`** (also `expand:tags,author`) loads related rows for SSR,
  one query per field, same redact/scope as REST `?expand=`. Without `expand`, M2M
  fields attach as id arrays so `columns "tags"` is no longer blank. A to-one
  `-> Entity` listed in `expand` is replaced with the related object.

### Language (mutation)

- **Forms honour `on submit`.** `POST`/`PATCH /_form` still write the row once.
  `create`/`update` in the block are declarations (must match `bind`). The response
  `effects` are that block's `toast`/`navigate`/`refresh` instead of a generic
  "Created successfully". `/_action` returns 403 for `submit` blocks, so they cannot
  double-insert. (`src/actions.rs`)
- **`/_action` `create`/`update` execute AST field literals.**
  `on click { create Task { title "hello" } }` inserts; the client still posts only
  `{action_id, entity, id}`. A `create` with no fields is `400 INVALID`.
- **`set` on many-to-many fields** (`set tags "id1,id2"`) replaces join rows with the
  same id-scope rules as REST/`/_form`.
- **Entity `on create/update/delete` runs after form and action writes**, not only REST.
  `log` interpolates and records; `notify` broadcasts SSE. Outbound HTTP remains `webhook`.

### Breaking changes (language honesty)

A `.cronus` file that used to parse and no-op now fails `cronus build`. Codes are in
LANGUAGE.md §15.9.

- **Unknown `where` operators are `BIND_001`.** `where status in:["paid"]` and
  `where title ends_with "x"` used to run as `eq`. Supported: `eq`, `ne`/`neq`,
  `gt`, `gte`, `lt`, `lte`, `contains`, `starts_with`.
- **Unknown `query` kinds are `BIND_002`.** `query every` used to mean `query all`.
- **Unknown field modifiers are `FIELD_004`.** `indexed`, `computed` and `onupdate:`
  used to be ignored; write `index` or drop them.
- **Unknown action verbs and `validate` are `ACTION_001`.** They used to be skipped.
  `create`/`update` still parse (templates / `/_form`); the executor still does not
  run them as separate steps.
- **Hollow top-level blocks are `LANG_001`:** `service`, `worker`, `middleware`,
  `deploy`, `test`, `define`, file-scope `on`. They parse so the rest of
  the file can be diagnosed; the app is invalid until they are removed. `webhook`,
  `import` and `compose { use }` are real. Two `app {}` or two `entity Task` across
  files used to last-win; they now fail (`COMPOSE_001`).

### Breaking changes

Apps that worked before may now return 401/403/404 or refuse to start. Each item is a
deliberate security change.

- **Data is owner-scoped on every surface.** REST, GraphQL, SSR bindings, `/_form`,
  `/_action` and SSE share one rule set (`src/access.rs`): non-admins see and change only
  rows whose `_owner_id` is theirs, `shared` entities are readable by signed-in users, admins
  see everything. Other users' rows are `404`. (d9ce111, 8dc8c98, fd75d70)
- **Anonymous visitors see no bound data** unless the binding says `scope:public`; the auth
  entity is never public. (8dc8c98)
- **Declared routes only.** When an entity has an `api` block, only its declared method +
  path shapes are served (`404` otherwise, also for admins); `auth:public|jwt|role(x)` is
  enforced. Entities are resolved by exact path segment. (d9ce111)
- **Authentication required** for auto CRUD without an `api` block, for GraphQL
  (`/graphql`, `/graphql/schema`), `/api/sse`, `/_action` and non-public `/_form`. Users are
  never created through generic REST; use `POST /api/auth/signup`. (d9ce111, 8dc8c98)
- **`requires:` matches the exact route pattern**, no longer a prefix: `/admin` does not
  protect `/admin/users`. (8dc8c98)
- **Cookie-only sessions + CSRF.** Login/signup set an `HttpOnly; SameSite=Lax` cookie
  (`Secure` in production); the JWT is in the response body only with `?token=1` or
  `X-Cronus-Session-Token: 1`. Client JS no longer stores or sends the token. Cookie-authenticated
  mutations must send a same-host `Origin`/`Referer`. (1fd49b2)
- **Old tokens are invalid.** Session JWTs now require `iat` and `jti`; tokens minted before
  do not verify and users must log in again. Cookie `Max-Age` and JWT `exp` follow
  `session jwt expires:<dur>`. (cbc198b, 1fd49b2)
- **Legacy password hashes no longer log in.** Only Argon2id PHC strings verify
  (`m=19456,t=2,p=1`); SHA-256 hashes were removed. Signup requires at least 15 characters. (d9ce111)
- **`cronus run` binds `127.0.0.1` by default.** Use `--host 0.0.0.0` / `CRONUS_HOST` to
  expose the server. (bf1fcf9)
- **Production mode** (`--prod` or `CRONUS_ENV=production`): internal/diagnostic routes
  (`/zeus`, `/blocks`, `/trust`, `/hydra`, `/api/_context`, `/docs*`, …) return `404`; in dev
  they need an admin unless bound to loopback. `/api/audit/trail*` always needs an admin. (bf1fcf9)
- **`JWT_SECRET` shorter than 32 bytes refuses to start.** (7dfb451)
- **Webhooks:** only `http://` URLs are delivered (`https://` is refused and logged, the
  kernel has no TLS client); loopback/private/link-local/metadata/CGNAT targets are blocked
  unless `CRONUS_WEBHOOK_ALLOW_PRIVATE=1`; payloads are redacted and signed with
  `X-Cronus-Signature`. (85964d0)
- **Request bodies over 1 MiB get `413`** (`CRONUS_MAX_BODY_BYTES`). (bf1fcf9)
- **Record ids are UUIDv7** instead of nanosecond hex. (4b6aeb3)
- **`/_action` runs only AST-declared actions by `action_id`**; client-sent instruction JSON
  is ignored. (8dc8c98, fd75d70)
- **cronus-ui renderers are zero-JS.** `tabs`, `accordion`, `dialog`, `select`, `tooltip`,
  `table`, `pagination`, `breadcrumb`, `number-input` and `password-input` no longer emit
  `<select>`/`<dialog>`/`<details>`/`onclick`; JS-only controls render disabled. `calendar`
  and `scheduler` highlight a day only from explicit `selected:`/`today:` attrs. (37596d4, f38e132)
- **Validation failures are `422` with per-field errors.** REST, `/_form` and GraphQL
  return `{"error":{"code":"VALIDATION_FAILED","message","fields":{"title":["must be at least
  3 characters"]}}}`; a taken `unique` value is `409` with `fields`. REST used `400` with one
  message and forms `400` with only `errors` (still present). The rules are now the same on
  every surface: `url` fields need `http(s)://` on REST too, `enum` values match exactly on
  forms, lengths count characters. (01ee808, a88cecd)
- **`-> Entity[]` is a many-to-many relation.** It used to be a TEXT column; it is now a join
  table `<Entity>_<field>` and an existing column of that name is ignored. (01ee808)

### Security (Sprint 1)

- Shared authorization contract: `authz::redact_sensitive`, `authz::writable_body` and
  safe `{"error":{"code","message"}}` bodies; database text never reaches clients. (56c550d, d9ce111)
- `src/api_crud.rs` replaces the REST handler in `main.rs`, with owner scope in the SQL
  `WHERE` of every SELECT/UPDATE/DELETE and 17 HTTP-level regression tests
  (`src/api_security_tests.rs`). (d9ce111)
- GraphQL, forms/actions, SSR bindings and SSE authorized through `src/access.rs`; sensitive
  fields removed from GraphQL SDL and responses. (8dc8c98, fd75d70)
- HTTP hardening in `src/http_guard.rs`: per-request panic isolation (`panic = "unwind"`),
  HTTP/1 header read timeout, rate limit keyed on the socket peer (`X-Forwarded-For` only from
  `CRONUS_TRUSTED_PROXIES`), per-account login backoff, bounded maps, poison-recovering
  locks. (bf1fcf9)
- CSP with per-request nonces for kernel-authored scripts only, no `'unsafe-inline'`,
  exact CDN URLs, `object-src 'none'`, `base-uri 'self'`. (cbc198b, 076a332)
- Bound data escaped in SSR sections and client HTML sinks (kanban, timeline, KPI,
  charts, form values, data lists, SSE cells). (234cdf1)
- Secrets: `.cronus/jwt.key` and `.cronus/webhook.key` created with mode 0600; `cronus new`
  gitignores key files and databases; `cronus generate` passes the API key on stdin. (7dfb451)
- Voodoo CDN script pinned with Subresource Integrity. (ed25676)
- Output safety gate (`src/cronus_ui_output_gate.rs`) renders every ported cronus-ui family
  with hostile inputs and fails on scripts, inline styles, `on*` attributes or executable
  URLs; `cronus_ui_kit::safe_url` filters `href`/`src`. (3e425e8)
- SSE clients stop reconnecting without a session and back off exponentially. (575c3bd)

### UI parity and audit

- Waves 1a–1r: dedicated kernel renderers for the cronus-ui families (about nine per wave),
  replacing generic fallbacks. Reports: `docs/archive/waves/`.
- Waves 1s–1t: geometry parity with the React components, checked by the cooud-ui
  Playwright audit (`e2e/audit/geometry.spec.ts`) against `cronus run --audit-canvas`.
- Sprint 2: zero-JS rewrites of ten renderers (37596d4), multi-select search row (762de83),
  choropleth/sunburst pixel parity (3c475d6), style fixes from the extended geometry audit
  (dd6082d), `cronus_ui_kit` attr/flag/esc helpers with a gate test against local copies (20e6e2b).
- Sprint 4: per-page cronus-ui CSS. The 220 KB component stylesheet is split into
  `src/cronus_ui_css/` and pages ship tokens plus only the families they render (audit
  button canvas 275 KB → 51 KB of CSS; a page without families 347 KB → 117 KB). Kit catalog
  CSS ships only with the catalog, audit preflight only in the audit document. `render_layout`
  and the audit document use `@layer cronus.tokens, cronus.base, cronus.components`, so
  unlayered author CSS wins without `!important`. Dead `combobox-content`/`combobox-item`
  rules and one duplicate block removed; reduced motion now also covers descendants and
  pseudo-elements of `[data-slot]`. SPA navigation does a full load when the target page
  needs a different cronus-ui stylesheet.
- Sprint 5 (agent A): zero-JS interactivity for overlay and menu families with native
  popovers instead of `disabled` triggers. workspace-switcher, split-button, select,
  combobox, color-picker and navigation-menu items with `content:` get a working
  `popovertarget` trigger and a `popover="auto"` panel (select/combobox: radio options whose
  label the trigger shows via CSS `attr()`; color-picker: preset radio swatches). hover-card
  opens with `interestfor` (Chromium) plus a `popovertarget` click fallback; its CSS
  `:hover` reveal is dropped. time-picker Done closes its panel. dialog, alert-dialog,
  confirmation-dialog, invite-dialog, sheet, drawer, morphing-popover and context-menu keep
  the audited open specimen by default and render closed with a trigger and working
  close/cancel buttons for `trigger:"…"` or `open:false`. Audit fixtures render unchanged.

### Language

- `where x eq:auth.id` / `eq:"v"` / `gt:5` colon forms equal the space form; `neq` aliases `ne`;
  `field:`, `group_by:`, `interval:` parsed in bindings. `aggregate` without `group_by` feeds
  KPI `value:bind`. (fd75d70)
- `PATCH /_form/<section>/<id>` edits through the declared form. (fd75d70)
- P041 (SQL reserved words) no longer applies to enum/array values; a test parses every
  `.cronus` under `templates/` and `demos/`. (e98d8ae)
- `style { theme light | dark | system }` (alias `mode`) now reaches every layout:
  `data-cronus-mode` follows it (it was hard-coded `dark`), `color-scheme` is set, and
  `system` ships the preset's other-mode tokens under `prefers-color-scheme`. Default stays
  `dark`; the audit canvas is byte-identical. Dark legacy pages now also declare
  `color-scheme: dark`. Section renderers with hard-coded dark colors are not converted
  (LANGUAGE.md §12).
- Field constraints `!`, `min:`, `max:`, `match:"…"`, `unique` and types are enforced on REST
  create/update, `/_form`, `/_action set` and GraphQL create, with field-level errors
  (LANGUAGE.md §3.6). `build` rejects an invalid `match:` regex (`FIELD_001`), `min` greater
  than `max` (`FIELD_002`) and non-numeric bounds (`FIELD_003`). (01ee808, a88cecd)
- Many-to-many relations `tags -> Tag[]`: join table with `ON DELETE CASCADE` on both sides
  and an index per id column, parent `_owner_id` on join rows, ids checked against the
  caller's read scope, id arrays in REST and GraphQL (`[String!]!`), `?expand=` for linked
  rows (LANGUAGE.md §3.7). (01ee808, a88cecd)
- `env { APP_STRIPE_KEY string! sensitive  APP_FEATURE_X boolean default:false }`: `cronus run`
  refuses to start on missing or mistyped variables without printing values; `ENV_001` /
  `ENV_002` build errors and an `ENV_003` prefix warning (LANGUAGE.md §3.8). (a88cecd)

### Tooling and docs

- CI runs `cargo test`, `cargo clippy -D clippy::correctness`, `cargo fmt --check` and a
  12 MB release-size budget. (bdd6c45, 14209e3)
- `llms.txt` is now an llmstxt.org index; `llms-full.txt` holds the language grammar with
  section/field type lists checked against the code; the Voodoo guide moved to
  `docs/voodoo-llms.md`.
- `cronus context --for-claude` prints the grammar, valid section and field types, the
  project's entities/APIs/pages and the current `build --ai` errors; `cronus brief` no
  longer prints placeholders when metadata is missing.
- Historical `WAVE*.md` and `.cronus/` planning docs moved to `docs/archive/`.

## [0.1.0] - 2026-04-03

### Added

#### Language Features
- Declarative app configuration (`app` block with port, database, theme).
- Entity declarations with 16 field types: `string`, `text`, `email`, `url`, `slug`, `phone`, `number`, `money`, `percentage`, `boolean`, `date`, `enum`, `json`, `ulid`.
- Field modifiers: required (`!`), `unique`, `sensitive`, `searchable`, `index`, `featured`, `optional`.
- Field constraints: `min:N`, `max:N`, `match:"regex"`, `default:"value"`.
- State machines via `transition` blocks on enum fields.
- Auth declaration (`auth` block with entity, login, session jwt, roles).
- API route declaration (`api` block with REST routes and auth levels).
- Page declaration with 48 section types (data, forms, navigation, feedback, marketing, layout).
- Data binding (`bind entity:X { query all/count/one; where/order/limit }`).
- Action model with 6 instruction verbs: `set`, `toast`, `navigate`, `refresh`, `create`, `confirm`.
- Effects (`on create/update/delete` blocks with `notify`).
- Style declaration (accent color, theme dark/light).
- Layout declaration (sidebar + topbar).
- Component declaration (reusable section templates).
- Conditional visibility (`show:when`).
- Constitution rules (`must` / `never`).

#### Runtime
- Full HTTP server (hyper 1.x).
- SQLite database with auto-migration from entity declarations.
- JWT authentication with Argon2id password hashing.
- REST API auto-generated for every entity with validation (400/409/422).
- GraphQL endpoint with auto-generated schema and playground.
- Server-side rendering with 48 section types.
- Hot module reload (file watcher + version polling).
- SSE live updates.
- Search, filter, and pagination built into tables.
- Chart rendering with real data aggregation.

#### CLI (32 commands)
- `cronus run` — Parse `.cronus`, create DB, serve HTTP.
- `cronus new <template>` — Create project from template (blog, crm, ecommerce, helpdesk, saas).
- `cronus build` — Validate `.cronus` file (with `--strict` mode).
- `cronus compose` — Merge multiple `.cronus` files into one app.
- `cronus seed` — Seed database with realistic test data.
- `cronus test` — Auto-generate and run CRUD tests.
- `cronus test --conformance` — Run conformance test suite.
- `cronus generate` — Generate `.cronus` from natural language description.
- `cronus parse` — Parse and show AST.
- `cronus spec validate|list|codegen` — Spec management and AI protocol generation.
- `cronus deploy` — Generate deploy artifacts (Fly.io, Railway, static).
- `cronus doctor` — Check syntax, DB, ports.
- `cronus stats` — Project statistics.
- `cronus export` — Export to IR JSON.
- `cronus dump` — Import from OpenAPI/Prisma schemas.

#### Multi-Agent Support
- Multi-file compose (`cronus compose`) merges separate `.cronus` files at runtime.
- Zero merge conflicts by design (declarative = no execution order dependencies).
- AI generation protocol via JSON schema (`cronus spec codegen --ai-protocol`).

#### Templates
- 5 production templates: blog, crm, ecommerce, helpdesk, saas.

#### Security
- XSS protection by design (`{{}}` auto-escapes, `{{{}}}` for raw).
- JWT with Argon2id.
- Input validation via constraints.
- Audit logging.

#### Quality
- 65,000+ lines of Rust.
- 150 tests, all passing.
- 68 language specs (`.spec.toml`).
- 90 conformance tests.
- 13 lint rules (zero hardcode enforcement).
- Semantic analysis resolve pass.
- AI-Error Protocol (`--ai` flag for JSON error output).
- 6.6MB release binary, zero external runtime dependencies.
