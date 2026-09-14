# CRONUS

> The declarative language that replaces your entire web stack.

One `.cronus` file + one binary = full-stack app with auth, CRUD, dashboard, charts, real-time.
No React. No Node. No npm. No config files.

---

## Quick Start

```bash
cargo install --path .

cronus new my-app                 # default template: saas
cd my-app

cronus run
# -> http://127.0.0.1:5175            home page
# -> http://127.0.0.1:5175/register   create an account (15+ char password)
# -> http://127.0.0.1:5175/dashboard  your data
```

Pick another starting point with `cronus new my-app --template <t>`:
`saas`, `api`, `landing`, `admin`, `blog`, `crm`, `ecommerce`, `helpdesk`, `cronus-ui`.
Every template passes `cronus build --ai` with zero errors.

---

## What Does It Look Like?

```cronus
app "Admin Panel" {
  port 5175
  database sqlite "./data.db"
}

auth {
  entity User
  login email + password
  session jwt expires:24h
  roles [member, admin]
  redirect "/dashboard"
}

entity Order {
  customer string!
  amount   money!
  status   enum [pending, approved, shipped, cancelled] default:pending
}

api /orders {
  list   GET    /    auth:jwt
  create POST   /    auth:jwt
  update PATCH  /:id auth:jwt
  delete DELETE /:id auth:jwt
}

page "/" type:custom {
  section hero {
    title "Admin Panel"
    subtitle "Orders and revenue for your team."
    cta_text "Sign in"
    cta_link "/login"
  }
}

page "/dashboard" type:dashboard requires:auth {
  section kpi {
    bind Order { aggregate count }
    item "Orders" value:bind icon:shopping_cart
  }
  section kpi {
    bind Order { aggregate sum field:amount }
    item "Revenue" value:bind icon:attach_money
  }
  section table {
    title "Recent orders"
    bind Order { query all order created_at desc limit 10 }
    columns "Customer, Amount, Status, Created At"
  }
}
```

These ~50 lines give you: HTTP server, SQLite database, signup/login (the `User` table is created for you when you don't declare it), a CRUD API, and a dashboard whose KPIs and table read each signed-in user's own orders.

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

Start from `templates/` (via `cronus new`) for idiomatic code: every template is
checked by `cargo test` to pass `cronus build --ai` with zero errors.

The demos are showcases. They must all **parse** (`every_template_and_demo_parses`),
but only some pass `build --ai`:

| Path | `build --ai` | Why |
|---|---|---|
| `demos/component-test/`, `demos/cronus-ui/` | valid | |
| `demos/cronus-ui-catalog/` | parse-only | widget catalog uses experimental section keys |
| `demos/saas-billing/`, `demos/stitch-dashboard/` | parse-only | multi-file showcases with custom sections |
| `demos/vinext-dumps/`, `templates/ecosystem/` | parse-only | raw `cronus dump` output from Next.js apps, kept as-is |

```bash
cd demos/saas-billing && cronus run 5950
```

---

## CLI

| Command | Description |
|---|---|
| `cronus run [port] [--strict]` | Parse `.cronus`, create DB, serve on `127.0.0.1` (dev mode) |
| `cronus run --host 0.0.0.0` | Expose on all interfaces (or `CRONUS_HOST`); internal dev routes then require an admin |
| `cronus run --prod` | Production mode (or `CRONUS_ENV=production`): `/zeus`, `/api/_context`, `/api/_seed`, debug routes → 404. Env: `CRONUS_MAX_BODY_BYTES`, `CRONUS_TRUSTED_PROXIES`, `CRONUS_HEADER_READ_TIMEOUT_SECS` |
| `cronus new <name> [--template t]` | Create project `<name>/` (default template `saas`; `cronus new --list` shows all) |
| `cronus build [--ai]` | Validate `.cronus` file (`--ai`: JSON errors with fix hints) |
| `cronus parse <file>` | Parse and show AST |
| `cronus seed [count]` | Seed database with realistic test data |
| `cronus test [port]` | Auto-generate and run CRUD tests |
| `cronus test --conformance` | Run conformance test suite |
| `cronus compose` | Compose all `.cronus` files into one app |
| `cronus generate <desc> [-o file] [--force]` | Generate `.cronus` from a description (never overwrites `app.cronus` without `--force`) |
| `cronus dump <path>` | Convert HTML / Next.js / Prisma / OpenAPI into `.cronus` |
| `cronus context --for-claude` | Export project context for an AI assistant |
| `cronus spec <validate\|list\|codegen>` | Validate, list, or generate from `.spec.toml` |
| `cronus deploy` | Generate deploy artifacts (`--fly`, `--railway`, `--static`) |
| `cronus doctor` | Required checks (parse, build, port, database writable) + informational ones + next commands |
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

Field modifiers: `!` (required, e.g. `name string!`), `unique`, `sensitive`, `searchable`, `index`, `featured`, `optional`.

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
