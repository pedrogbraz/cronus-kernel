# CRONUS

**Compile full-stack software from a single spec file.**

1 file. 1 binary. Zero dependencies. 108 database tables. 41 pages. 175 API routes.

---

## What it does

```cronus
entity User {
  email     email     required unique
  name      string    required
  role      enum      [admin, member]
}

api /users {
  list    GET    /     auth:public
  create  POST   /     auth:jwt
}

page "/users" type:list entity:User {
  title "Users"
  columns [name, email, role]
}
```

Run `cronus run`. You get:

- SQLite database with `users` table (auto-migrated)
- REST API with CRUD endpoints + validation (400/409/422)
- Server-rendered HTML page with search, pagination, status badges
- JWT authentication with signup/login
- GraphQL endpoint (auto-generated from entities)
- Hot module reload (edit .cronus, browser updates)
- Hydra Brain tracking usage patterns

From 15 lines of spec. No React. No Vite. No node_modules. One 4.2MB Rust binary.

---

## Quickstart

```bash
# Install
cargo install --path .

# Create project
cronus new saas
cd saas

# Run
cronus run
# Server running -> http://localhost:5175
# API -> http://localhost:5175/api
# GraphQL -> http://localhost:5175/graphql
```

---

## Commands

```
cronus run [port]       Parse .cronus -> create DB -> serve
cronus new <template>   Create project (landing/saas/api)
cronus build            Validate .cronus file
cronus parse <file>     Parse and show AST stats
cronus compose          Compose multiple .cronus files
cronus generate         Generate code from .cronus
cronus deploy           Generate Dockerfile + docker-compose
cronus doctor           Check syntax, DB, ports
cronus stats            Project statistics
cronus export           Export to cronus-project.ir.json
cronus test             Run API endpoint tests
```

Templates: `landing` (hero + pricing), `saas` (auth + dashboard + billing), `api` (backend only).

---

## Features

- **Spec to Full Stack** — Entities, API routes, pages, auth, GraphQL. All from one file.
- **Rust Native** — 4.2MB binary. Zero runtime dependencies. Instant cold start.
- **Multi-Agent Mode** — Multiple AI agents write separate .cronus files. Kernel composes automatically. Zero merge conflicts.
- **Validated API** — Required fields (400), unique constraints (409), email validation (422). No code needed.
- **Pagination** — `?limit=20&offset=40` with `X-Total-Count` headers. Built in.
- **Smart Seed** — `POST /api/_seed` generates realistic test data with FK resolution.
- **SSR Pages** — Hero, features, pricing, dashboard, lists, forms. Dark theme. Tailwind CDN.
- **Auth Built-in** — Signup, login, JWT tokens, role-based access control.
- **GraphQL Free** — Auto-generated schema from entity definitions. Playground included.
- **Hot Reload** — Edit .cronus file, server detects change, browser reloads.
- **Hydra Brain** — Tracks API usage patterns. Suggests optimizations at `/api/brain/stats`.
- **Deploy Ready** — `cronus deploy` generates Dockerfile, docker-compose.yml, .dockerignore.

---

## Multi-Agent Mode

The killer feature. Multiple AI agents work on the same project simultaneously with zero conflicts:

```
project/
  00-app.cronus          # App config + style
  01-auth.cronus         # Agent 1: auth entities + pages
  02-products.cronus     # Agent 2: product catalog
  03-payments.cronus     # Agent 3: orders + billing
  04-marketing.cronus    # Agent 4: landing page sections
  05-analytics.cronus    # Agent 5: analytics entities
```

```bash
cronus compose
# Composing 6 files:
#   + 00-app.cronus (41 lines)
#   + 01-auth.cronus (45 lines)
#   + 02-products.cronus (70 lines)
#   + 03-payments.cronus (79 lines)
#   + 04-marketing.cronus (82 lines)
#   + 05-analytics.cronus (29 lines)
#
# Composed: 16 entities, 16 pages, 36 routes
# Total: 48 nodes from 346 lines

cronus run
# All 6 files composed into 1 running app
```

Each agent owns a file. No imports between files. The kernel merges at runtime. Conflict-free by design.

---

## Size comparison

A real commerce platform (Cooud) with 108 entities:

| | TypeScript + React | .cronus |
|---|---|---|
| 108 entities + CRUD | ~21,600 lines | 890 lines |
| 41 pages (SSR) | ~12,300 lines | 350 lines |
| Auth + roles | ~2,000 lines | 20 lines |
| API routes (175) | ~8,750 lines | 200 lines |
| Infrastructure (17 services) | ~3,400 lines | 238 lines |
| **Total** | **~48,000 lines** | **1,698 lines** |

~40x compression ratio. Same functionality.

---

## Language reference

```cronus
# App config
app "My App" {
  stack react + tailwind
  port 5175
  database sqlite "./data.db"
  theme dark
}

# Entity (becomes DB table + API type)
entity Product {
  name        string    required
  price       money     required
  status      enum      [active, draft, archived]
  category    string
  featured    boolean
  createdAt   date
}

# API routes
api /products {
  list    GET    /        auth:public
  detail  GET    /:id     auth:jwt
  create  POST   /        auth:jwt
  update  PATCH  /:id     auth:jwt
  delete  DELETE /:id     auth:jwt
}

# Pages
page "/" type:custom {
  section hero {
    title "My App"
    subtitle "Built with CRONUS"
    cta "Get Started" -> "/signup" primary
  }
  section pricing cols:3 {
    plan "Starter" $29/mo [
      "5 projects",
      "Email support"
    ]
    plan "Pro" $79/mo featured [
      "Unlimited projects",
      "Priority support"
    ]
  }
}

page "/products" type:list entity:Product {
  title "Products"
  columns [name, price, status, createdAt]
  filters [status]
  search name
}

page "/login" type:form entity:User {
  title "Sign In"
  fields [email, password]
}

# Style
style {
  theme dark
  accent amber
  font "Inter"
  radius xl
}

# Services
service api port:3001 {
  cors origins:["*"]
  rate_limit 100/min
}

# Events
on order.created {
  notify user "order-confirmed"
  send email "receipt"
}

# Workers
worker email-sender queue:emails {
  concurrency 10
  retry 5
}
```

Field types: `string`, `text`, `email`, `url`, `slug`, `phone`, `number`, `money`, `percentage`, `boolean`, `date`, `enum`, `json`, `ulid`.

Field modifiers: `required`, `unique`, `sensitive`, `searchable`, `index`, `featured`, `optional`.

---

## Architecture

```
.cronus file
    |
    v
[Parser] ── Rust native, zero-copy tokenizer
    |
    v
[AST] ── Vec<AstNode> (Entity, Page, Api, Style, Service, Event, Worker...)
    |
    v
[Database] ── SQLite auto-migration from entities
[Server] ── hyper 1.x HTTP server
[UI] ── SSR HTML with Tailwind CDN
[Auth] ── JWT + argon2 password hashing
[Brain] ── Pattern learning from API usage
[HMR] ── File watcher with version polling
[GraphQL] ── Auto-generated schema + playground
```

Single binary. No runtime. No garbage collector. No cold starts.

---

## Status

**Alpha** — actively developing. Parser, database, server, auth, and SSR are production-quality. GraphQL and multi-agent mode are stable. 17 unit tests passing.

---

## Roadmap

- [ ] PostgreSQL support
- [ ] VS Code extension with syntax highlighting
- [ ] Plugin system for custom field types
- [ ] Cloud deploy (fly.io, Railway)
- [ ] Real-time SSE push to connected clients
- [ ] Import from existing TypeScript projects

---

## Stats

```
cronus-kernel/src/
  main.rs              1,280 lines   CLI + HTTP server + request handler
  parser.rs            1,300 lines   Tokenizer + parser + AST
  database.rs            660 lines   SQLite CRUD + validation + seed
  ui.rs                  440 lines   SSR HTML renderer
  server.rs              340 lines   Standalone HTTP server
  auth.rs                350 lines   JWT + password + RBAC
  marketing_components  2,400 lines   Premium section library
  + 12 more modules
  ─────────────────────────────────
  Total:              ~12,500 lines of Rust
  Binary:              4.2MB (release, stripped)
  Dependencies:        14 crates
  Tests:               17 passing
```

---

Built in Rust. Designed for AI. Made in Brazil.
