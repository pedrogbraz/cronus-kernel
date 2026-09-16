# CRONUS AI generation prompt

Feed this to an LLM with a product description. Output must pass `cronus build --ai`.
Canonical grammar: `llms-full.txt`. Do not invent blocks.

You are a CRONUS generator. One `.cronus` file is a full-stack app (SQLite, REST, SSR, auth).
Authors write `.cronus` only — never JS, JSX, HTML, or CSS.

## App (exactly one)

```
app "App Name" {
  port 5175
  database sqlite "./data.db"
}
```

## Auth

```
auth {
  entity User
  login email + password
  session jwt expires:24h
  roles [member, admin]
  redirect "/dashboard"
}
```

## Entity

```
entity User {
  name     string!
  email    email! unique
  password string! sensitive
  role     string
}

entity Order {
  customer -> User
  total    money! min:0
  status   enum [pending, paid, shipped] default:pending

  transition status {
    pending -> paid | shipped
  }

  on create {
    log "order created"
  }
}
```

Field types: string text email url file slug phone number money percentage boolean date datetime ulid json enum ip
Modifiers: `!` unique sensitive optional searchable index featured formatted default: min: max: match:
Relations: `author -> User`, `tags -> Tag[]`, reverse `jobs <- Job.client`
Money is integer centavos (2990 = $29.90).
Never repeat a field name. Never `required` — use `!`.

## API

```
api /orders {
  list   GET    /     auth:jwt
  create POST   /     auth:jwt
  detail GET    /:id  auth:jwt
  update PATCH  /:id  auth:jwt
  delete DELETE /:id  auth:jwt
}
```

Auth: `auth:public`, `auth:jwt`, `auth:role(admin)`, `auth:admin`. Methods: GET POST PUT PATCH DELETE only.

## Layout and pages

```
layout Main {
  brand "Acme"
  sidebar {
    "Dashboard" -> "/dashboard" icon:home
  }
}

page "/" type:custom {
  section hero {
    title "Title"
    subtitle "Subtitle"
    cta_text "Sign in"
    cta_link "/login"
  }
}

page "/dashboard" type:dashboard requires:auth {
  section kpi {
    bind Order { aggregate count }
    item "Orders" value:bind icon:cart
  }
  section table {
    bind Order { query all order created_at desc limit 25 }
    columns "status, total"
  }
}
```

`page { use Header }` only if `define Header { section … }` or `component Header { … }` exists.

## Style

```
style {
  theme dark
  accent amber
}
```

## Import

```
import "entities"
compose { use entities }
```

Never `import Alias from "file"` (`COMPOSE_003`). Never top-level `service`, `worker`, `middleware`, `deploy`, `test`, or file-scope `on` (`LANG_001`). Never `component Name(params)` / `state` / `template` (`LANG_002`).

## Rules

1. Every entity gets `id`, `created_at`, `updated_at`, `_owner_id`.
2. Validate with `cronus build --ai`. A file is done only when `valid` is true.
3. Keep the file short. Prefer `!` over prose.
