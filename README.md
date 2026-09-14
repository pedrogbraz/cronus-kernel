# CRONUS

> The declarative language that replaces your entire web stack.

One `.cronus` file + one binary = full-stack app with auth, CRUD, dashboard, charts, real-time.
No React. No Node. No npm. No config files.

---

## Quick Start

```bash
cargo install --path .

cronus new admin
cd admin

cronus run
# -> http://localhost:5175
# -> http://localhost:5175/api
# -> http://localhost:5175/graphql
```

---

## What Does It Look Like?

```cronus
app "Admin Panel" {
  port 5300
  database sqlite "./data.db"
  theme dark
}

entity Order {
  customer    string    required
  amount      money     required
  status      enum      [pending, approved, shipped, cancelled]
  created_at  date
}

auth {
  entity User
  login email
  session jwt
  roles [admin, member]
}

api /orders {
  list    GET    /       auth:jwt
  create  POST   /       auth:jwt
  update  PATCH  /:id    auth:jwt
  delete  DELETE /:id    auth:jwt
}

page "/" type:dashboard requires:auth {
  title "Admin Dashboard"

  section stats cols:4 {
    bind entity:Order { query count }
    item "Total Orders" value:"count" icon:shopping_cart
    item "Revenue" value:"$48,290.00" icon:attach_money
  }

  section recent-orders {
    bind entity:Order { query all; order created_at desc; limit 10 }
    columns "Customer, Amount, Status, Date"
    on click {
      set status "approved"
      toast "Order approved"
      refresh self
    }
  }
}
```

These ~50 lines give you: HTTP server, SQLite database, JWT auth with roles, CRUD API, server-rendered dashboard with KPIs, a data table with real database binding, and click actions that mutate data.

---

## What You Get

- **Full HTTP server** -- Rust, starts in microseconds
- **SQLite database** -- auto-created from entity declarations
- **JWT authentication** -- signup, login, protected pages, role-based access
- **REST API** -- auto-generated CRUD for every entity, with validation (400/409/422)
- **GraphQL endpoint** -- auto-generated schema + playground
- **SSR rendering** -- 48 section types, dark/light themes
- **Data binding** -- sections show real database data via `bind entity:X`
- **Actions** -- declarative mutations: `set`, `toast`, `navigate`, `refresh`, `create`, `delete`
- **Search, filter, pagination** -- built into tables, with `X-Total-Count` headers
- **Charts** -- real aggregation (GROUP BY + SUM/COUNT) from bound data
- **Hot module reload** -- edit `.cronus`, browser updates
- **Multi-file compose** -- multiple `.cronus` files merge at runtime, zero conflicts
- **Single binary, zero dependencies** -- 6.7MB

---

## The Numbers

| | Traditional Stack | CRONUS |
|---|---|---|
| Admin panel (auth + CRUD + dashboard) | ~3,000 lines + 200MB node_modules | ~80 lines |
| 108-entity commerce platform | ~48,000 lines (TS + React) | ~1,700 lines |
| Install size | 200MB+ | 6.7MB binary |
| Config files | package.json, tsconfig, vite.config, .env, ... | 0 |
| Build step | 30-120 seconds | None (interpreted) |

67 language specs. 89 conformance tests. 28,912 lines of Rust powering the kernel.

---

## Demos

| File | Description |
|---|---|
| `demos/admin-panel.cronus` | Order management with auth, KPIs, tables, forms (166 lines) |
| `demos/saas-dashboard.cronus` | Multi-entity analytics with charts and team management (245 lines) |
| `demos/landing-page.cronus` | Marketing site with hero, features, pricing, lead capture (107 lines) |

```bash
cronus run demos/admin-panel.cronus
```

---

## CLI

| Command | Description |
|---|---|
| `cronus run [port] [--strict]` | Parse `.cronus`, create DB, serve on `127.0.0.1` (dev mode) |
| `cronus run --host 0.0.0.0` | Expose on all interfaces (or `CRONUS_HOST`); internal dev routes then require an admin |
| `cronus run --prod` | Production mode (or `CRONUS_ENV=production`): `/zeus`, `/api/_context`, `/api/_seed`, debug routes → 404. Env: `CRONUS_MAX_BODY_BYTES`, `CRONUS_TRUSTED_PROXIES`, `CRONUS_HEADER_READ_TIMEOUT_SECS` |
| `cronus new <template>` | Create project: `landing`, `admin`, `saas`, `api`, `ecommerce`, `blog` |
| `cronus build [--strict]` | Validate `.cronus` file |
| `cronus parse <file>` | Parse and show AST |
| `cronus seed [count]` | Seed database with realistic test data |
| `cronus test [port]` | Auto-generate and run CRUD tests |
| `cronus test --conformance` | Run conformance test suite |
| `cronus compose` | Compose all `.cronus` files into one app |
| `cronus generate <desc>` | Generate `.cronus` from natural language description |
| `cronus spec <validate\|list\|codegen>` | Validate, list, or generate from `.spec.toml` |
| `cronus deploy` | Generate deploy artifacts (`--fly`, `--railway`, `--static`) |
| `cronus doctor` | Check syntax, DB, ports |
| `cronus stats` | Project statistics |
| `cronus export` | Export to `cronus-project.ir.json` |

---

## Section Types (48)

**Data** -- `table`, `kpi`, `stats`, `chart`, `kanban`, `timeline`, `progress`, `pagination`, `filters`

**Forms** -- `form`, `modal`, `sheet`, `checkout`

**Navigation** -- `tabs`, `breadcrumb`, `sidebar`, `topbar`, `command`

**Feedback** -- `alert`, `toast`, `accordion`, `dropdown`, `notifications`, `skeleton`, `loading`, `empty`, `error`, `not-found`

**Marketing** -- `hero`, `features`, `features-split`, `bento`, `edge`, `pricing`, `cta`, `faq`, `testimonial`, `trusted`, `footer`, `promo`

**Layout** -- `layout`, `columns`, `card`, `links`, `dark-mode`

Plus aliases: `stats` -> `kpi`, `activity-table` -> `table`, `info-bar` -> `alert`, and more.

---

## The Language

```cronus
app "Name" { ... }          # App config: port, database, theme
entity Name { ... }          # Data model -> DB table + API type
api /path { ... }            # REST routes with auth levels
auth { ... }                 # JWT auth config
page "/path" type:X { ... }  # Pages with sections
  section type { ... }       # UI sections with data binding
    bind entity:X { ... }   # Bind section to database queries
    on event { ... }         # Declarative actions
style { ... }                # Theme: accent, font, radius
layout { ... }               # App shell: sidebar, topbar
component Name { ... }       # Reusable section templates
```

Field types: `string`, `text`, `email`, `url`, `slug`, `phone`, `number`, `money`, `percentage`, `boolean`, `date`, `enum`, `json`, `ulid`.

Field modifiers: `required`, `unique`, `sensitive`, `searchable`, `index`, `featured`, `optional`.

---

## Multi-Agent Mode

Multiple AI agents write separate `.cronus` files. The kernel composes them at runtime. Zero merge conflicts.

```
project/
  00-app.cronus          # App config + style
  01-auth.cronus         # Agent 1: auth entities + pages
  02-products.cronus     # Agent 2: product catalog
  03-payments.cronus     # Agent 3: orders + billing
  04-marketing.cronus    # Agent 4: landing page sections
```

```bash
cronus compose   # Merges all files
cronus run       # Serves composed app
```

---

## AI Integration

```bash
cronus spec codegen --ai-protocol
```

Generates JSON schema from `.spec.toml` files, enabling LLMs to generate valid `.cronus` code with full type awareness.

---

## Architecture

```
.cronus file
    |
    v
[Parser] -- Rust, zero-copy tokenizer
    |
    v
[AST] -- Vec<AstNode>
    |
    +---> [Database] -- SQLite auto-migration
    +---> [Server]   -- hyper 1.x HTTP
    +---> [UI]       -- SSR HTML + Tailwind CDN
    +---> [Auth]     -- JWT + argon2
    +---> [GraphQL]  -- auto-generated schema
    +---> [HMR]      -- file watcher + version polling
```

---

## Status

Experimental. The language is evolving. Breaking changes may occur.

67 specs. 89 conformance tests. 28,912 lines of Rust. Actively developed.

---

Built in Rust. Designed for AI. Made in Brazil.
