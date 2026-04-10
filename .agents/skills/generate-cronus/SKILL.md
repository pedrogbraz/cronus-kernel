---
name: generate-cronus
description: "Generate .cronus files — the declarative full-stack language. Use when the user asks to create a web app, dashboard, API, landing page, or any CRUD application. Replaces React/Next.js/Vite with 50 lines of .cronus that compile to a 7MB Rust binary."
license: MIT
---

# Generate .cronus Applications

You are an expert in the CRONUS language — a declarative full-stack DSL that compiles to a single Rust binary. One .cronus file replaces React + Next.js + Prisma + Express + 300MB of node_modules.

## When to Use

- User asks to create a web app, dashboard, admin panel, landing page, SaaS, e-commerce, blog, CRM, or any CRUD app
- User asks to migrate from Next.js, React, or any JS framework
- User mentions ".cronus" or "CRONUS"

## Language Quick Reference

### App Block (REQUIRED — always first)
```cronus
app "App Name" {
  stack react + tailwind
  port 5175
  database sqlite "./data.db"
}
```

### Entity (data model — auto-creates DB table + REST API)
```cronus
entity Product {
  name        string!  searchable
  slug        slug!    unique
  description text
  price       money!
  stock       number   default:0
  active      boolean  default:true
  category    -> Category
  status      enum [draft, published, archived]
  
  transition status {
    draft -> published
    published -> archived
    archived -> draft
  }
  
  on create {
    log "Product created: {{name}}"
  }
}
```

**16 field types**: `string`, `text`, `email`, `url`, `slug`, `phone`, `number`, `money`, `percentage`, `boolean`, `date`, `ulid`, `json`, `enum`, `ip`, `-> Relation`

**Modifiers** (after type):
- `!` or `required` — NOT NULL
- `unique` — unique constraint
- `sensitive` — never exposed in HTML/API responses
- `searchable` — indexed for search
- `default:value` — default value
- `min:N max:N` — range validation
- `match:"regex"` — pattern validation

### Auth
```cronus
auth {
  entity User
  login email + password
  session jwt expires:24h
  roles [admin, user, viewer]
}
```

### API Routes (auto-generated CRUD, customize auth per route)
```cronus
api /products {
  list    GET    /           auth:public
  create  POST   /           auth:jwt
  detail  GET    /:id        auth:public
  update  PATCH  /:id        auth:jwt
  delete  DELETE /:id        auth:role(admin)
}
```

### Pages
```cronus
page "/" type:dashboard requires:auth {
  title "Dashboard"
  
  section kpi {
    bind Order { aggregate count }
    item "Total Orders" value:bind icon:shopping_cart
  }
  
  section table {
    bind Product { query all order created_at desc limit 20 }
    columns "name, price, status, created_at"
    search true
  }
  
  section chart {
    bind Order { aggregate sum field:total group_by:created_at interval:month }
    chart_type bar
  }
}

page "/products/:id" type:detail requires:auth {
  section card {
    bind Product { query one where id eq:route.id }
  }
}

page "/products/new" type:form requires:auth {
  section form {
    bind Product
    on submit {
      create Product
      toast "Created!" success
      navigate "/products"
    }
  }
}
```

### Layout (sidebar navigation)
```cronus
layout Main {
  brand "MyApp"
  sidebar {
    "Dashboard" -> "/" icon:dashboard
    "Products"  -> "/products" icon:inventory
    "Orders"    -> "/orders" icon:receipt
    "Settings"  -> "/settings" icon:settings
  }
}
```

### Style
```cronus
style {
  theme dark
  accent blue
  font "Inter"
  radius 8px
}
```

### 51 Section Types

**Data**: `table`, `kpi`, `chart`, `kanban`, `timeline`, `progress`, `stat-cards`
**Forms**: `form`, `modal`, `sheet`, `filters`
**Navigation**: `tabs`, `breadcrumb`, `sidebar`, `topbar`, `command`
**Feedback**: `alert`, `toast`, `accordion`, `dropdown`, `notifications`, `skeleton`, `empty`, `error`
**Marketing**: `hero`, `features`, `pricing`, `cta`, `faq`, `testimonial`, `footer`, `trusted`, `product-grid`, `team-list`
**Layout**: `card`, `page-header`, `dark-mode`

### Binding (connects sections to database)
```cronus
bind Entity { query all }                              # all records
bind Entity { query all order name asc limit 10 }      # sorted + limited
bind Entity { query one where id eq:route.id }         # single record by URL param
bind Entity { query all where status eq:"active" }     # filtered
bind Entity { aggregate count }                        # count
bind Entity { aggregate sum field:price }              # sum
bind Entity { aggregate sum field:total group_by:created_at interval:month }  # chart data
```

### Actions (button/form event handlers)
```cronus
on submit {
  create Entity
  toast "Saved!" success
  navigate "/list"
}

on click confirm:"Delete this?" {
  delete Entity route.id
  toast "Deleted" success
  navigate "/list"
}
```

**10 action verbs**: `set`, `toast`, `navigate`, `refresh`, `create`, `delete`, `validate`, `confirm`, `open`, `close`

### ScriptCronus (.scriptcronus — imperative logic)
```
on Order.create {
  let customer = db.query Customer { filter id == event.record.customer_id }
  log "New order for {{customer.name}}"
  
  http.post "https://api.stripe.com/v1/charges" {
    headers { Authorization "Bearer {{env.STRIPE_KEY}}" }
    json { amount event.record.total currency "brl" }
  }
  
  sse.broadcast "order-created" { id event.record.id }
}

schedule "daily-report" every:1d {
  let count = db.count Order { filter created_at > "today" }
  log "Orders today: {{count}}"
}

endpoint GET /api/health auth:public {
  respond 200 { status "ok" timestamp now() }
}
```

## Generation Rules

1. **ALWAYS start with `app` block** — name, stack, port, database
2. **Entities before pages** — pages bind to entities
3. **Money = centavos** — R$29.90 is `2990`. Use `money` type
4. **Use `!` for required fields** — `name string!` not `name string required`
5. **`bind` is mandatory for data sections** — table, kpi, chart MUST have `bind`
6. **Don't hardcode data** — use `bind` to query the database
7. **Auth pages use `requires:auth`** — dashboard, admin, settings
8. **Public pages have no requires** — landing, pricing, about
9. **One file is enough** — put everything in one .cronus file
10. **Keep it short** — 30-80 lines for a typical app. Less is better

## NEVER Do This

- NEVER hardcode numbers in KPI/stats sections (use `bind` + `aggregate`)
- NEVER put `password` or `sensitive` fields in `columns` config
- NEVER create a form without `on submit` action
- NEVER use `location.reload()` — use `refresh` action
- NEVER write JavaScript — .cronus handles everything
- NEVER add `id`, `created_at`, `updated_at` fields — auto-generated
- NEVER use React/JSX syntax — this is .cronus, not React

## Complete Example: SaaS Billing App

```cronus
app "NovaPay" {
  stack react + tailwind
  port 5175
  database sqlite "./data.db"
  constitution {
    must "prices in centavos"
    never "expose payment tokens in API"
  }
}

auth {
  entity User
  login email + password
  session jwt expires:24h
  roles [admin, user]
}

style {
  theme dark
  accent blue
  font "Inter"
}

entity Customer {
  name    string!  searchable
  email   email!   unique
  plan    -> Plan
  status  enum [trial, active, churned] default:trial
  
  transition status {
    trial -> active | churned
    active -> churned
  }
}

entity Plan {
  name     string!
  price    money!
  interval enum [monthly, yearly]
  features json
}

entity Invoice {
  customer -> Customer
  amount   money!
  status   enum [draft, open, paid, void]
  due_date date
  
  transition status {
    draft -> open
    open -> paid | void
  }
}

api /customers {
  list   GET    / auth:jwt
  create POST   / auth:jwt
  detail GET    /:id auth:jwt
  update PATCH  /:id auth:jwt
}

api /plans {
  list GET / auth:public
}

api /invoices {
  list   GET    / auth:jwt
  create POST   / auth:role(admin)
  detail GET    /:id auth:jwt
}

layout Main {
  brand "NovaPay"
  sidebar {
    "Dashboard"  -> "/" icon:dashboard
    "Customers"  -> "/customers" icon:people
    "Plans"      -> "/plans" icon:payments
    "Invoices"   -> "/invoices" icon:receipt
  }
}

page "/" type:dashboard requires:auth {
  section kpi {
    bind Customer { aggregate count }
    item "Customers" value:bind icon:people
  }
  section kpi {
    bind Invoice { aggregate sum field:amount where status eq:"paid" }
    item "Revenue" value:bind icon:attach_money
  }
  section table {
    bind Invoice { query all order due_date desc limit 10 }
    columns "customer, amount, status, due_date"
  }
  section chart {
    bind Invoice { aggregate sum field:amount group_by:due_date interval:month }
    chart_type bar
  }
}

page "/customers" requires:auth {
  section table {
    bind Customer { query all order name asc }
    columns "name, email, plan, status"
    search true
  }
}

page "/customers/new" requires:auth {
  section form {
    bind Customer
    on submit {
      create Customer
      toast "Customer created" success
      navigate "/customers"
    }
  }
}

page "/invoices" requires:auth {
  section table {
    bind Invoice { query all order due_date desc }
    columns "customer, amount, status, due_date"
  }
}
```

## CLI Commands
```bash
cronus run                 # Dev server with HMR
cronus build               # Validate without serving
cronus build --ai          # JSON errors for AI consumption
cronus test                # Auto-generated CRUD tests
cronus dump ./nextjs-app   # Convert Next.js project to .cronus
cronus context --for-claude # Export project context for AI
```
