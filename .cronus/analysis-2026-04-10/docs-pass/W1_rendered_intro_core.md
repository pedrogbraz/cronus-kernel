# W1 — Rendered Docs Pass: Introduction & Core Language

**Source:** `http://docs.cronus.test` (local nginx — WebFetch force-upgrades to HTTPS and fails on the dev cert, so fetched via `curl`)
**Date:** 2026-04-10
**Pages:** 11 (Introduction, Project Structure, First App, Entities, Pages, Data Binding, Actions, Auth, API Routes, Field Types, ScriptCronus)

---

## 1. Homepage `/` — Introduction (full transcription)

**H1:** Introduction
**Lead:** "CRONUS is a declarative language that compiles to full-stack web applications. One `.cronus` file produces an HTTP server, database, API, and server-rendered UI with **51 built-in section types**."

**Philosophy (3 principles):**
1. Declarative over imperative — a section declaration is enough to produce a complete UI component with data binding.
2. Zero-config defaults — write `section table` and get sorting, pagination, search, and CRUD out of the box.
3. Semantic composition — `hero` above `features` = landing page; `sidebar` + `table` = dashboard.

**Quick Start — "A complete CRONUS application in 12 lines":**
```cronus
app "My App" {
  port 3000
  database sqlite "./data.db"
}

entity Task {
  title    string  required
  done     boolean
}

page "/" type:dashboard {
  section table {
    title "Tasks"
    bind entity:Task { query all }
  }
}
```
"Run `cronus run` and you get an HTTP server, SQLite database, REST API, and a fully functional data table."

**Design Principles:** Speed ("Rust-compiled binary starts in microseconds. Server-rendered HTML with zero JavaScript by default. No build step, no bundler."), Accessibility ("All interactive sections include ARIA attributes, keyboard navigation, and proper focus management."), Elegance ("Typography-first design. Monochrome palette with a single accent.").
**Typography / palette:** Geist family; `#020202 / #0A0A0A / #111111 / #0D0D0D` + accent `#CC0000`.

**51 Section Types (verbatim list):**
```
# Marketing    hero features pricing cta faq testimonial trusted
# Navigation   topbar sidebar breadcrumb tabs footer page-header links
# Data         table chart kpi stat-cards timeline progress kanban
# Forms        form checkout
# Cards        card product-grid bento team-list status-card activity-table
# Feedback     alert modal sheet toast skeleton empty error not-found command
# Layout       grid layout edge accordion pagination filters dark-mode
# Aliases      404 notifications dropdown quick-links info-bar promo
                features-split policies loading stats
```

**Built-in API endpoints (verbatim table):** `/api/health` (GET, 200 OK), `/api/_health` (GET, extended health w/ uptime, memory, entity count), `/api/schema` (GET, full entity schema as JSON), `/api/_seed` (POST, demo data), `/api/_context` (GET, AI Context Protocol for LLM integration), `/api/docs/index` (GET), `/api/docs/search?q=` (GET, full-text), `/api/sse` (GET, SSE stream), `/api/auth/signup` (POST), `/api/auth/login` (POST, returns JWT), `/api/auth/me` (GET), and CRUD quintet `/api/{entity}s` (GET list, POST create), `/api/{entity}s/:id` (GET/PATCH/DELETE). **Note:** entity endpoints auto-pluralize (`Task` → `/api/tasks`); auth endpoints only exist when an `auth` block is defined.

**Built-in dashboards (verbatim):** `/docs` (API Docs from AST), `/docs/design` (Design System, live showcase of all 51 section types), `/docs/graph` (Mermaid entity graph), `/blocks` (Block Explorer — "3D visual browser for compiled blocks"), `/zeus` (Zeus Tracer — request timeline w/ spans and SQL queries), `/trust` (block trust profiles and confidence scores), `/hydra` (evolution cycle status and mutation log). "All dashboards … require no configuration. They are disabled in production mode (`cronus run --production`)."

**Commands shown:** `cronus run`, `cronus run --production`. **Outbound links:** `/first-app`, `/entities`, `/components`.

---

## 2. `/project-structure` — Project Structure

**Summary:** Flat project layout using numbered-prefix `.cronus` files. All files are merged alphabetically into a single AST regardless of boundaries. Also covers single-file projects, the hidden `.cronus/` cache dir, multi-file, imports, and env blocks.

```
my-app/
  00-app.cronus        # App config + auth + style
  01-entities.cronus   # Data models
  02-api.cronus        # REST routes
  03-pages.cronus      # UI pages
  04-portal.cronus     # User-facing pages
  05-components.cronus # Reusable components
  06-scripts.scriptcronus  # Server-side automation
  data.db              # SQLite (auto-generated)
  .cronus/
    ast-snapshot.json  # AST cache
    block-registry.json # Promoted blocks
```

```cronus
# 00-app.cronus
app "My SaaS" {
  port 5175
  database sqlite "./data.db"
  theme dark
}
auth {
  provider jwt
  secret env:JWT_SECRET
  expire "24h"
}
```

```cronus
import "./shared/auth.cronus"
import "./shared/common-entities.cronus"
app "My App" { port 5175 }
```

```cronus
env development {
  DATABASE_URL "sqlite:./data.db"
  LOG_LEVEL "debug"
  SEED_ON_START true
}
env production {
  DATABASE_URL "postgres://user:pass@db:5432/app"
  LOG_LEVEL "warn"
  SEED_ON_START false
}
```

**Commands:** `CRONUS_ENV=production cronus run`, `cronus run --env production`.
**Claims:** "Files are loaded alphabetically and merged into a single AST." "Circular imports are detected and rejected at parse time." "Import paths are relative; subdirectories ok; parent/absolute paths rejected." "`development` environment is used by default." "Env block variables **override** system environment variables." "`data.db` is auto-created from entity declarations; the kernel handles migrations automatically."
**Links:** `/first-app`, `/entities`, `/cli`.

---

## 3. `/first-app` — Your First App

**Summary:** Tutorial that builds a task manager in ~35 lines. Walks install → create → entity → API → UI → run → inspect → build/validate → testing → seeding → hot reload.

```cronus
app "Task Manager" { port 5175; database sqlite "./data.db"; theme dark }

entity Task {
  title       string  required
  description text
  done        boolean
  priority    enum    [low, medium, high]
  due_date    date
}

api /tasks {
  list   GET    /     auth:public
  create POST   /     auth:public
  detail GET    /:id  auth:public
  update PATCH  /:id  auth:public
  delete DELETE /:id  auth:public
}

page "/" type:dashboard {
  section kpi { bind entity:Task { query count }; item "Total Tasks" }
  section table {
    title "All Tasks"
    bind entity:Task { query all; order due_date asc }
    item "Title" column:true; item "Priority" column:true; item "Done" column:true
    on click { set done true; toast "Task completed" style:success; refresh self }
  }
}

test "create and list tasks" {
  create Task { title: "Buy milk" } -> expect 201
  create Task { title: "Read book" } -> expect 201
  list Task -> expect 200 count:2
  delete Task where title eq "Buy milk" -> expect 200
  list Task -> expect 200 count:1
}
```

**Commands shown:** `cargo install cronus` (Rust 1.75+, ~7MB binary, zero runtime deps); `cronus new taskman`; `cronus run`; `cronus brief` (app summary); `cronus doctor` (env check); `cronus build`; `cronus build --strict`; `cronus build --ai` (JSON errors for AI tooling); `cronus validate`; `cronus test`; `cronus seed`; `cronus compose "SaaS billing dashboard with Stripe integration"`; `curl localhost:5175/.cronus/version`.

**Claims:** "Tests use a declarative syntax that maps directly to API calls." "`cronus seed` generates realistic fake data per field type; respects modifiers." "Hot reload preserves database state. Schema changes trigger automatic migrations without losing data." `/.cronus/version` returns `{ version, entities, pages, uptime }`; browser polls and reloads on hash change. `cronus compose` can scaffold a full multi-file app from one NL prompt.
**Links:** `/entities`, `/pages`, `/data-binding`, `/auth`, `/components`.

---

## 4. `/entities` — Entities

**Summary:** Data models that map to SQLite tables. Documents 16 field types, modifiers, relations, enums, advanced constraints, lifecycle hooks, and `shared` (non-scoped) entities.

```cronus
entity User {
  name     string  required
  email    email   required unique
  password string  required sensitive
  role     enum    [admin, member, viewer]
  avatar   url     optional
  team     -> Team
  tags     string[]
  bio      text    searchable
  score    percentage
  balance  money   formatted
  ip       ip      index
  metadata json
  pid      ulid    unique
  phone    phone
  handle   slug    unique
  joinedAt date
}
```

```cronus
entity Product {
  name        string  required searchable
  sku         string  required unique pattern:"[A-Z]{3}-[0-9]{4}"
  price       money   required min:100 max:9999900
  description text    optional min_length:10 max_length:5000
  tags        string[] searchable
  ssn         string  sensitive
  metadata    json    optional
}
```

```cronus
entity AuditLog {
  action string required; actor string required; payload json
  @lifecycle retain 90 days
}

entity Order {
  total  money required
  status enum  [pending, paid, shipped]
  on create { notify admin "New order received"; log "order.created" }
  on update where status eq "paid" { notify customer "Payment confirmed"; log "order.paid" }
  on delete { log "order.deleted" }
}

entity Config shared { key string required unique; value json required }
```

**16 field types:** `string, text, email, url, slug, phone, number, money, percentage, boolean, date, ulid, json, enum, ip, relation (->)`.
**Basic modifiers:** `required, unique, sensitive, optional, searchable, index, featured, formatted`, plus array suffix `[]`.
**Advanced modifiers:** `pattern:"regex"`, `min:N`, `max:N`, `min_length:N`, `max_length:N`.
**Claims:** `sensitive` excludes from list responses (still in detail with auth); `searchable` creates a full-text index and is included in `?q=` queries; `money` stored as INTEGER centavos; `->` creates `customer_id` FK column; enums validated on API input AND form submission (CHECK constraint). Entity default scoping: implicit `owner_id` filter from the JWT. `shared` keyword removes `owner_id` and opts out. `@lifecycle retain N days` auto-deletes records older than N days. Entity blocks can declare `on create`, `on update where ...`, `on delete` hooks that run server-side after the DB operation completes.
**Links:** `/field-types`, `/data-binding`, `/auth`.

---

## 5. `/pages` — Pages

**Summary:** Page declarations define route, type, and section composition. Covers page types, inline modifiers, list/form shorthand, dynamic routes, guards, section composition, and the top-level `layout` block.

```cronus
# Dashboard page with modifiers
page "/dashboard" type:dashboard requires:auth {
  use Topbar
  title "Dashboard"
  section hero { title "Welcome back" }
  section kpi cols:4 {
    item "Revenue" { "$45,000" }
    item "Users"   { "1,200" }
  }
}

# List shorthand
page "/products" type:list entity:Product {
  title   "Products"
  columns [name, price, stock, status]
  search  [name, description]
  actions [create, edit, delete]
  filters [status, category]
}

# Form shorthand
page "/signup" type:form entity:User {
  title "Create Account"
  fields [name, email, password]
}

# Dynamic routes + guards
page "/users/:id" type:detail {
  bind entity:User { query one; where id eq route.id }
}
page "/stores/:storeId/products/:productId" type:detail {
  bind entity:Product {
    query one
    where id eq route.productId
    where store eq route.storeId
  }
}
page "/admin" type:dashboard requires:auth { }
page "/admin/settings" type:form requires:role(admin) { }
page "/pricing" type:custom { }

# Composed dashboard with topbar + kpi (with compare) + chart + table
page "/dashboard2" type:dashboard requires:auth {
  section topbar { brand "My SaaS" nav "Dashboard, Users, Settings" }
  section kpi {
    bind entity:Order { query count; compare period:"30d" }
    item "Revenue"   field:"sum:total"
    item "Avg Order" field:"avg:total"
  }
  section chart { bind entity:Order { group total by:day; limit 30 }; type area }
  section table {
    bind entity:Order { query all; order created_at desc; limit 20 }
    item "Customer" column:true; item "Total" column:true; item "Status" column:true
  }
}
```

```cronus
layout {
  sidebar {
    brand "MyApp"
    position left
    width 250
    style dark
    item "Dashboard" icon:dashboard -> "/"
    item "Products"  icon:inventory -> "/products"
    item "Orders"    icon:receipt   -> "/orders"
    item "Settings"  icon:settings  -> "/settings" position:bottom
  }
  topbar { search true; notifications true; avatar true }
}
```

**Page types (6):** `dashboard` (sidebar + topbar + metric slots), `list` (auto-table w/ pagination + search), `form` (auto-form w/ validation + submit), `detail` (single record view), `custom` (empty canvas), `checkout` (payment flow with steps).
**Page modifiers:** `type:<name>`, `requires:auth`, `requires:role(admin)`, `entity:Name`, `layout:<variant>`.
**Claims:** Nested route params work at any depth; unused params → build warning. Guarded pages return 302 → `/login` on unauth, then redirect back after login. `layout` block applies to all pages unless overridden; sidebar items accept `position:bottom` to pin.
**Links:** `/components`, `/actions`, `/data-binding`, `/entities`.

---

## 6. `/data-binding` — Data Binding

**Summary:** Connect sections to entity queries. Covers bind syntax, query types (all/one/count), filters, ordering/pagination, grouping/aggregation, route params, live binding, auth-scoped queries, and conditional visibility.

```cronus
# where + order + pagination
section table {
  bind entity:Order {
    query all
    where status eq "active"
    where amount gt 1000
    order created_at desc
    limit 20; offset 0
  }
  item "Customer" column:true; item "Total" column:true
}

# Grouping + aggregation
bind entity:Order { query all; group created_at by:month; aggregate sum(amount) }

# KPI with multiple aggregates
section kpi {
  bind entity:Order { aggregate sum(total); aggregate count; aggregate avg(total) }
  item "Revenue"   field:"sum_total"
  item "Orders"    field:"count"
  item "Avg Order" field:"avg_total"
}

# Live binding via SSE
section table {
  bind entity:Order { query all; order created_at desc; live true }
  item "Customer" column:true
}

# Auth-scoped + conditional visibility (NOTE different operator syntax from where)
section table {
  bind entity:Order { query all; where owner eq auth.user_id; order created_at desc }
}
section kpi { show:when status == "active"; title "Active Users" }
```

**Query types:** `query all` → array; `query one` → single row or null; `query count` → integer.
**Filter operators (where):** `eq, ne, gt, lt, gte, lte, contains (LIKE '%v%'), starts_with (LIKE 'v%')`.
**Visibility operators (show:when):** `==, !=, <, >, <=, >=` (different!)
**Aggregates:** `sum(field), count, avg(field), min(field), max(field)`.
**Group intervals:** `day, week, month, year`.
**Claims:** Multiple `where` clauses are AND-combined. Only single-field ordering is supported. Default order is ascending. `live true` opens `/api/sse/:entity` and re-renders the section without full page reload (~2KB client cost per section). `auth.user_id` and `auth.role` are injected from JWT and resolved server-side — cannot be spoofed.
**Links:** `/entities`, `/actions`, `/sse-live`.

---

## 7. `/actions` — Actions

**Summary:** Declarative mutations triggered by user interaction. Kernel executes them server-side and returns a JSON effects envelope the client processes in order.

```cronus
section form {
  bind entity:Order { query all }
  item "Customer" required:true; item "Amount" required:true

  on submit {
    validate all
    create Order { total: form.total, status: "pending" }
    toast "Created" style:success
    navigate "/orders"
  }
  on error { toast "Failed to create" style:error }
}

on click {
  confirm "Are you sure you want to delete?"
  delete Order where id eq row.id
  toast "Record deleted" style:success
  refresh self
}
```

**Effects envelope (response JSON):**
```json
{ "ok": true,
  "effects": [
    { "type": "toast", "target": "Order approved", "style": "success" },
    { "type": "refresh", "target": "", "style": "" }
  ]
}
```

**Action verbs (complete list):** `set <field> <value>`, `create <Entity> [{ ... }]`, `update <Entity> where ...`, `delete <Entity> [where ...]`, `toast "msg" style:type` (success|error|info|warning), `navigate "/path"`, `refresh self|section_id`, `validate all|<field>`, `confirm "message"`, `open <modal_id|url>`, `close <modal_id>`.
**Contexts:** `form.<field>` (inside form blocks), `row.<field>` (inside list/table click handlers).
**Events:** `on submit`, `on click`, `on error`.
**Claims:** Actions execute sequentially; if one fails, the chain stops. `confirm` cancel skips all subsequent actions. `on error` runs on network errors, validation failures, or constraint violations.
**Links:** `/data-binding`, `/components/form`, `/scripting`.

---

## 8. `/auth` — Auth

**Summary:** Built-in JWT auth with Argon2id hashing, RBAC, auto-generated endpoints, page/route guards, and PIN login for admin panels.

```cronus
# Short form
auth {
  entity User
  login email + password
  session jwt expires:7d
  roles [admin, editor, viewer]
}
```

```cronus
# Long form (same page, different style)
auth {
  provider jwt
  secret env:JWT_SECRET
  expire "24h"
  refresh_expire "30d"
  entity User
  login_field email
  password_field password
}
```

```cronus
entity User {
  name     string required
  email    email  required unique
  password string required sensitive
  role     enum   [admin, editor, viewer]
}
```

```cronus
page "/dashboard" requires:auth { }
page "/admin" requires:role(admin) { }

api /users {
  list   GET    /     auth:jwt
  create POST   /     auth:admin
  update PATCH  /:id  auth:admin
  delete DELETE /:id  auth:admin
}
```

```cronus
# PIN login (admin kiosk)
auth {
  provider jwt
  pin_login true
  pin_length 6
  entity Admin
}
```

**Auth block keys observed:** `entity, login, session, roles, provider, secret, expire, refresh_expire, login_field, password_field, pin_login, pin_length`.
**Auto-generated endpoints (when auth block exists):** `POST /api/auth/signup` (homepage) / `POST /api/auth/register` (this page — contradicts!), `POST /api/auth/login` (returns access+refresh), `GET /api/auth/me`, `POST /api/auth/logout`, `POST /api/auth/refresh`, `POST /api/auth/pin-login` (when enabled).
**Route auth modes:** `auth:public`, `auth:jwt`, `auth:admin`, `auth:editor`, `auth:jwt [role1, role2]`. Also `auth:internal` (see API Routes page) — internal only, not exposed.
**Claims:** "Passwords hashed with Argon2id." — but the very next table says register "automatically hashes with **bcrypt**." (contradiction). Tokens use HS256 (jsonwebtoken crate). Middleware extracts Bearer token, verifies, attaches Claims. **"Role hierarchy: admin satisfies any role check."** Secret must use env var. PIN stored hashed on the entity.
**JWT flow (verbatim):** 1) client POST login → 2) server returns `{ token, refreshToken, user }` → 3) client stores in localStorage → 4) sends `Authorization: Bearer <token>` → 5) server verifies JWT on each request → 6) on expiry, POST refresh → 7) server returns new token pair.
**Links:** `/api-routes`, `/entities`, `/scripting`.

---

## 9. `/api-routes` — API Routes

**Summary:** REST route groups with auth levels, role guards, auto-CRUD, validation, middleware, custom/nested routes, response envelope, query params, and declarative webhook blocks.

```cronus
api /users {
  list   GET    /     auth:jwt
  create POST   /     auth:admin [admin]
  detail GET    /:id  auth:jwt
  update PATCH  /:id  auth:jwt
  delete DELETE /:id  auth:jwt
}
```

```cronus
middleware rateLimit {
  applies_to ["/api/*"]
  limit 100/min
  per user
}
middleware cors {
  origins ["http://localhost:5175", "https://myapp.com"]
}
```

```cronus
api /products {
  list   GET    /     auth:"public"
  create POST   /     auth:"jwt"
  detail GET    /:id  auth:"public"
  update PATCH  /:id  auth:"jwt"
  delete DELETE /:id  auth:"jwt" ["admin"]
}

api /analytics {
  dashboard GET  /dashboard    auth:jwt [admin, analyst]
  export    GET  /export/:type auth:"jwt"
  webhook   POST /webhook      auth:"internal"
}

# Nested resource routes
api /orders/:order_id/items {
  list   GET    /     auth:"jwt"
  add    POST   /     auth:"jwt"
  remove DELETE /:id  auth:"jwt"
}
```

```cronus
# Declarative, entity-level webhook block (.cronus — NOT ScriptCronus)
webhook Order {
  on create -> POST "https://hooks.slack.com/xxx" {
    body { text: "New order: {{name}} — {{amount}}" }
  }
  on update status {
    when "shipped" -> POST "https://api.shipping.com/notify" {
      body { order_id: "{{id}}", status: "shipped" }
    }
  }
  on delete -> POST "https://audit.internal/log" {
    body { event: "order_deleted", id: "{{id}}" }
  }
}
```

**Response envelope:**
```json
// Success
{ "data": [ {"id":1,"name":"Widget"} ],
  "meta": { "total": 42, "page": 1, "per_page": 20 } }

// Error
{ "error": { "code": 422, "message": "Validation failed",
             "fields": { "name": "required" } } }
```

**Auth levels:** `auth:public | auth:jwt | auth:admin | auth:editor | auth:internal` + role brackets `[admin, finance]`.
**Auto-CRUD:** declaring an `entity` auto-registers the 5-endpoint CRUD. Declaring an `api` block overrides auth levels and adds role guards.
**Validation codes:** 400 wrong type, 409 unique conflict, 422 required missing.
**Query params on list endpoints:** `?page=2&per_page=10`, `?sort="price"&order="desc"`, filter like `?status="active"&category="electronics"`, search `?q="widget"`, relation include `?include="customer,items"`.
**Webhook block events:** `on create`, `on update [field]` (with optional nested `when "value" ->`), `on delete`. Template vars `{{field}}` resolved from entity. Explicitly "separate from ScriptCronus webhooks — webhook blocks are declarative, scripts are imperative."
**Links:** `/auth`, `/entities`, `/graphql-api`.

---

## 10. `/field-types` — Field Types

**Summary:** The 16 built-in field types with DB mappings for SQLite + PostgreSQL, auto-validation, input rendering, and advanced attributes.

**Type table (verbatim):**

| Type | Keyword | SQLite | PostgreSQL | Input | Auto-validation |
|---|---|---|---|---|---|
| String | `string` | VARCHAR(255) | VARCHAR(255) | text | — |
| Text | `text` | TEXT | TEXT | textarea | — |
| Email | `email` | VARCHAR(255) | VARCHAR(255) | email | RFC 5322 |
| URL | `url` | VARCHAR(2048) | VARCHAR(2048) | url | URL w/ protocol |
| Slug | `slug` | VARCHAR(255) | VARCHAR(255) | text (auto-slugify from `name`) | — |
| Phone | `phone` | VARCHAR(20) | VARCHAR(20) | tel | E.164 |
| Number | `number` | INTEGER | INTEGER | number | range |
| Money | `money` | INTEGER | INTEGER | number + format | integer centavos ≥0 |
| Percentage | `percentage` | REAL | DOUBLE PRECISION | number+% | 0–100 |
| Boolean | `boolean` | INTEGER (0/1) | BOOLEAN | checkbox | coerce true/false |
| Date | `date` | TEXT (ISO 8601) | DATE | datetime-local | ISO 8601 |
| ULID | `ulid` | TEXT (26) | CHAR(26) | auto | auto on create |
| JSON | `json` | TEXT | JSONB | textarea | valid JSON |
| Enum | `enum` | TEXT + CHECK | TEXT + CHECK | select | in declared list |
| IP | `ip` | TEXT | INET | text | IPv4/IPv6 |
| Relation | `->` | TEXT + FK | TEXT + FK | select | FK constraint |

```cronus
entity Product {
  name  string required min_length:2 max_length:100
  price money  required formatted featured min:0
  sku   string required pattern:"^[A-Z]{3}-[0-9]{4}$" index
  tags  string array
}
```

**Advanced attributes:** `featured`, `formatted`, `array` (or `[]`), `min`, `max`, `min_length`, `max_length`, `pattern`, `index`.
**Claims:** slug auto-generates from `name` (e.g., "John Doe" → `john-doe`); ulid auto-generates 26-char string, never changes; money stored ALWAYS as integer centavos — use `format:money` in KPI/table items, **never divide by 100 in client code**; enum creates SQL CHECK constraint. Validation runs server-side on every API write, and client forms validate with the same rules.
**Links:** `/entities`, `/components/form`, `/data-binding`.

---

## 11. `/scripting` — ScriptCronus

**Summary:** Imperative layer (`.scriptcronus` files) running in a hardened VM. Defines entity event handlers, cron schedules, custom HTTP endpoints, webhooks. Sandboxed with hard limits. Successful scripts can be promoted to canonical `.cronus` blocks via the Trust Engine. Every file starts with `script "Name" { version "1.0" }`.

**Block types:** `on <Entity>.<event>`, `on webhook "/hooks/..."`, `webhook <name> path:"..." secret:env:VAR { ... }` (typed w/ signature), `schedule <name> every:<interval>`, `endpoint <METHOD> <path> [auth:<mode>] { ... }`.

```scriptcronus
# integrations.scriptcronus
script "My Integration" { version "1.0" }

on Customer.create {
  log "New customer: {{event.record.name}}"
  db.update Customer event.id { synced true }
}

on Invoice.update {
  if event.record.status == "paid" {
    log "Invoice {{event.id}} paid"
    sse.broadcast "invoice_paid" {
      invoice_id event.id
      amount event.record.amount
    }
  }
}

on Order.delete { log "Order removed: {{event.id}}" }
```

```scriptcronus
schedule "check_overdue" every:1h {
  let overdue = db.query Invoice { filter status == "open" }
  for inv in overdue { log "Overdue: {{inv.invoice_number}}" }
}

schedule "daily_cleanup" every:1d {
  let old = db.query Session { filter expired == true }
  for s in old { db.delete Session s.id }
}
```

```scriptcronus
endpoint GET /api/export/customers {
  let data = db.query Customer { query all }
  respond 200 data
}
endpoint POST /api/admin/reset auth:admin {
  db.delete TempData "all"
  respond 200 "done"
}
endpoint GET /api/status auth:public { respond 200 "ok" }
```

```scriptcronus
on webhook "/hooks/stripe" {
  log "Stripe event: {{event.body.type}}"
  if event.body.type == "payment_intent.succeeded" {
    db.create Payment {
      external_id event.body.data.object.id
      amount      event.body.data.object.amount
      status      "paid"
    }
  }
}
```

```scriptcronus
# Typed webhook with signature verification — NEW syntax on same page
webhook stripe_payment path:"/webhooks/stripe" secret:env:STRIPE_WEBHOOK_SECRET {
  let event = payload
  if event.type == "payment_intent.succeeded" {
    let pi = event.data.object
    db.update(Order, { stripe_id: pi.id }, { status: "paid" })
  }
  if event.type == "charge.refunded" {
    let charge = event.data.object
    db.update(Order, { stripe_id: charge.payment_intent }, { status: "refunded" })
  }
}
```

```scriptcronus
# Custom endpoint with JS-like call syntax (also on same page)
endpoint GET "/api/dashboard/stats" auth:jwt {
  let user = auth.get_user()
  let orders = db.count(Order, { owner: user.id })
  let revenue = db.query(Order, { owner: user.id, status: "paid" })
    .reduce(fn(sum, o) => sum + o.total, 0)
  let recent = db.query(Order, { owner: user.id },
                        { order: "created_at desc", limit: 5 })
  return format.json({ orders: orders, revenue: revenue, recent: recent })
}
```

```scriptcronus
# Control flow
let x = 42
let items = db.query Product { filter active == true }
for item in items { log "Product: {{item.name}} - ${{item.price}}" }
if x > 10 { log "big" } else { log "small" }
```

```scriptcronus
# Promotion pipeline example (trust_score = 0.950 → promoted to .cronus)
# Before: integrations.scriptcronus
on Customer.create {
  log "New customer"
  db.update Customer event.id { synced true }
}
# After promotion: app.cronus (auto-generated)
entity Customer {
  on create { log "New customer"; update self { synced true } }
}
```

**Event variables:** `event.id`, `event.entity`, `event.event` (create/update/delete), `event.record` (current), `event.prev` (update only), `event.body` (alias for webhook compatibility).
**Built-in namespaces (overview list = 7):** `db, http, sse, log, format, env, auth`.
**Built-in namespaces (detailed table on same page adds an 8th):** `db (query/create/update/delete/count)`, `http (get/post/put/patch/delete)`, `sse (broadcast)`, `log (info/warn/error)`, `format (csv/json)`, `env (env.KEY access)`, `auth (check_role/get_user/hash_password)`, `cache (get/set/delete/clear)`.
**Security model (9 protections):** 1000-statement fuel system, 100-variable scope limit, 50-log cap, 2048-char log truncation, owner isolation (non-admin sees only own records), forced `_owner_id` on create (cannot be overridden), 1MB webhook body cap, webhook role (NOT admin), no filesystem/eval/exec/raw SQL. Constants from `vm.rs`: `MAX_STATEMENTS=1000`, `MAX_SCOPE_VARS=100`, `MAX_LOG_ENTRIES=50`.
**Schedule intervals:** `30s, 5m, 30m, 1h, 6h, 1d` (one example also uses `every:"1w"` but the parser docstring says only `s/m/h/d` suffixes — drift).
**CRUD pipeline order:** DB op → `fire_effects()` (declarative .cronus effects) → `fire_webhooks()` (outbound) → `fire_scripts()` (ScriptCronus handlers). `ScriptRegistry` indexes blocks at startup: `get_event_handlers()`, `get_webhook_handlers()`, `get_endpoints()`, `get_schedules()`.
**Trust Engine / Promotion:** every execution tracked by `trust::track_execution()`. Status ranges: `Sandbox 0.000–0.499` (no promote), `Probation 0.500–0.799` (no promote), `Trusted 0.800–1.000` (eligible). Promotion pipeline at `src/promote.rs` groups script blocks by entity, emits valid `.cronus` syntax, embeds trust metadata as comments.
**Operators & control flow:** `==, !=, <, >, <=, >=`, `and`, `or`, `contains`. `let` bindings, `for…in`, `if/else`, `{{path}}` interpolation. Truthiness: `false`, `null`, empty string, `0` are falsy (empty objects are truthy).
**Links:** `/actions`, `/api-routes`, `/microservices`.

---

## 12. Discrepancies & contradictions across pages

1. **Password hashing contradiction (`/auth`)** — same page says "Argon2id" and then "register … hashes with **bcrypt**." Only one is true.
2. **Register vs signup endpoint** — homepage: `/api/auth/signup`; `/auth`: `/api/auth/register`. Both names appear inside `/auth` itself.
3. **Auth block has two syntaxes on the same page** — short form (`login email + password`, `session jwt expires:7d`) and long form (`provider jwt`, `secret env:...`, `login_field email`, `password_field password`). No explanation whether both work or one is legacy.
4. **ScriptCronus call syntax drift** — early examples use space-block form (`db.query Customer { filter ... }`, `db.update Customer event.id { ... }`); later same-page examples use JS-paren form (`db.query(Order, { status: "paid" })`, `db.delete(Session, expired)`). Unexplained.
5. **ScriptCronus namespaces listed twice with conflicts** — overview says 7; expanded table adds an 8th (`cache`), changes `format` from `money/date/number` to `csv/json/xml`, changes `auth` from `hash_password/verify_password` to `check_role/get_user/hash_password`.
6. **Schedule interval syntax drift** — `every:1h`, `every:"5m"`, `every:1d`, and `every:"1w"` all on the same page; parser doc says suffixes are only `s/m/h/d`.
7. **Section type count** — "51 built-in section types" but the verbatim list contains ~57 names (some "Aliases"). 51 is a promise, not a count.
8. **Page types count** — `/pages` says "six types" and lists `dashboard/list/form/detail/custom/checkout`, but the inline modifier table on the same page only mentions four (`dashboard/custom/form/list`).
9. **Webhook blocks documented in three different forms** — (a) declarative `webhook Order { on create -> POST ... }` in `/api-routes`, (b) imperative `on webhook "/hooks/..."` in `/scripting`, (c) typed `webhook stripe_payment path:"..." secret:env:... { }` also in `/scripting`. `/api-routes` flags (a) as separate from ScriptCronus, but (b)/(c) relationship is never explained.
10. **`route.id` vs `route.<paramName>`** — both forms appear; param naming rule never spelled out.
11. **Database support** — homepage only mentions SQLite; `/entities` and `/field-types` show full Postgres mappings; `/project-structure` hints at `DATABASE_URL "postgres://..."` in env blocks; no page documents the full Postgres selection flow.
12. **Homepage lists 7 dev dashboards** (`/docs`, `/docs/design`, `/docs/graph`, `/blocks`, `/zeus`, `/trust`, `/hydra`); **none of the 10 sub-pages reference any of them** — complete gap.
13. **Marketing sections** (`hero, features, pricing, cta`, etc.) listed on the homepage, but no crawled page shows authoring syntax — all focus on `table/kpi/chart/form`. Presumably on `/components/*` pages outside this sweep.
14. **`/pages` "layout" subsection** rendered with visible double code-fence artifacts (`code.cronus[[CODE_START]][[CODE_START]]layout { ...`), suggesting a recent edit the template didn't clean up.

---

## 13. Features/syntax an agent reading the source would NOT easily infer

**Top-level blocks:** `app {port,database,theme}`; `entity Name {...}` and `entity Name shared {...}` (shared = opt out of owner_id scoping); `page "/path" type:X requires:Y entity:Z {...}`; `api /prefix { name METHOD /path auth:mode [roles] ... }`; `auth {...}` (two alt syntaxes documented); `middleware <name> {applies_to [...] limit N/min per <subject>}` and `middleware cors {origins [...]}`; `webhook <EntityOrName> {...}` declarative entity-level webhook in .cronus; `layout {sidebar{...} topbar{...}}` global page shell (`position left|right`, `width`, `style dark|light`, `item ... position:bottom`); `import "./path.cronus"` file-relative only (parent/absolute rejected; circular imports rejected at parse time); `env <name> {KEY "val" ...}` with `CRONUS_ENV=<name>` / `--env` and env-block vars overriding system env; `test "name" {...}` built-in test DSL; `@lifecycle retain N days` auto-delete; entity hooks `on create {...}`, `on update where X eq Y {...}`, `on delete {...}` (server-side, post-DB).

**Field syntax:** relation arrow `fieldname -> Entity` → FK column `entity_id`; array as `string[]` or `tags string array`; modifiers `required, optional, unique, sensitive, searchable, index, featured, formatted`; advanced `pattern:"regex"`, `min:N`, `max:N`, `min_length:N`, `max_length:N`. `sensitive` excluded from list responses (in detail with auth). `searchable` = FTS index + `?q=` support. `money` ALWAYS stored as INTEGER centavos — renderer formats, never divide client-side. `slug` auto-generates from sibling `name` field. `ulid` auto-generates 26-char on create and never changes. `enum` creates SQL CHECK constraint.

**16 field types:** `string, text, email, url, slug, phone, number, money, percentage, boolean, date, ulid, json, enum, ip, relation (->)`.

**Page types & modifiers:** 6 types (`dashboard, list, form, detail, custom, checkout`); modifiers `type:`, `requires:auth`, `requires:role(admin)`, `entity:Name`, `layout:<variant>`. Dynamic params `:id`/`:paramName` → `route.id`/`route.<name>`. Unused params → build warning. Guarded pages 302→`/login` with round-trip.

**Data-binding DSL inside `bind entity:X { ... }`:** `query all|one|count`; `where <field> <op> <value>` with named ops `eq, ne, gt, lt, gte, lte, contains, starts_with`; `order <field> asc|desc` (single-field only, default asc); `limit N`, `offset N`; `group <field> by:day|week|month|year`; `aggregate sum(f)|count|avg(f)|min(f)|max(f)`; `live true` (opens `/api/sse/:entity`); `compare period:"30d"`. **Section-level `show:when <expr>` uses DIFFERENT operators (`==, !=, >, <, >=, <=`)** from where's named ops. References: `route.id`, `route.<paramName>`, `auth.user_id`, `auth.role` (server-only, not spoofable). Item shorthand `item "Label" column:true`, `item "Revenue" field:"sum:total"`.

**Actions DSL verbs:** `set, create, update, delete, toast, navigate, refresh, validate, confirm, open, close`. Contexts `form.<field>`, `row.<field>`. Events `on submit, on click, on error`. Server returns JSON effects envelope `{ok, effects: [{type, target, style}, ...]}` processed in order.

**Auth configuration keys:** `entity, login, session, roles, provider, secret, expire, refresh_expire, login_field, password_field, pin_login, pin_length`. `login email + password` joins creds with `+`. Route modes: `auth:public|jwt|admin|editor|internal|jwt [roles]`. **Admin satisfies any role check.** PIN login → `POST /api/auth/pin-login`.

**Auto-endpoints:** `/api/health`, `/api/_health`, `/api/schema`, `/api/_seed`, `/api/_context` (AI Context Protocol for LLM integration), `/api/docs/index`, `/api/docs/search?q=`, `/api/sse`, `/api/sse/:entity` (live bindings), `/api/auth/*` (when auth block present), CRUD under `/api/{entity}s`. List params: `?page`, `?per_page`, `?sort`, `?order`, `?q=`, `?include=`, plus arbitrary field filters.

**ScriptCronus (`.scriptcronus`) — separate language layer:** required header `script "Name" {version "1.0"}`; block types `on <Entity>.<event>`, `on webhook "/hooks/..."`, `webhook <name> path:"..." secret:env:VAR {}`, `schedule <name> every:<interval>`, `endpoint <METHOD> <path> [auth:mode] {}`. Control flow `let`, `for…in`, `if/else`, `{{path}}` interpolation; operators `==, !=, <, >, <=, >=, and, or, contains`; falsy = `false/null/""/0` (empty objects truthy). Sandboxed namespaces (8): `db, http, sse, log, format, env, auth, cache`. Hard limits: 1000 statements, 100 vars, 50 logs, 2KB log msg, 1MB webhook body. Scripts run as user; webhooks as `webhook` role (NOT admin). `_owner_id` force-set on create; owner isolation on query/update/delete. No filesystem, no eval, no exec, no raw SQL. Pipeline after CRUD: `fire_effects()` → `fire_webhooks()` → `fire_scripts()`. Trust Engine scores every execution; ≥0.800 eligible for promotion to canonical `.cronus` via `src/promote.rs`. Schedule intervals: `30s, 5m, 30m, 1h, 6h, 1d` (and undocumented `1w` in examples).

**CLI:** `cargo install cronus`, `cronus new <name>`, `cronus run`, `cronus run --production`, `cronus run --env <name>`, `cronus build`, `cronus build --strict`, `cronus build --ai`, `cronus validate`, `cronus brief`, `cronus doctor`, `cronus test`, `cronus seed`, `cronus compose "<description>"`. Env: `CRONUS_ENV=<name>`.

**Runtime endpoints:** `/.cronus/version` (HMR — returns `{version, entities, pages, uptime}`; browser polls, soft-reloads on hash change).

**Dev-only dashboards (off in production):** `/docs`, `/docs/design`, `/docs/graph`, `/blocks`, `/zeus`, `/trust`, `/hydra`.

**Declarative test DSL:** `test "name" { create X { ... } -> expect 201; list X -> expect 200 count:N; delete X where title eq "x" -> expect 200 }`.
