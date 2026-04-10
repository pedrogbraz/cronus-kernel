# W3 — Rendered docs: CLI / build / deploy / compiler / dump / templates

Source: `http://docs.cronus.test/{cli,docker,production,compiler,dump,templates}` (rendered HTML, fetched 2026-04-10).
Fetch method: curl (WebFetch force-upgrades to HTTPS and fails on local `.test` TLS cert, so the rendered pages were fetched plain-HTTP via the localhost entry in `/etc/hosts`, then parsed).

All six pages share a common nav shell (`CRONUS v0.1 / Guide / Components / API`). Sidebar advertises a single CLI reference page (`/cli`), so the docs self-present as authoritative.

---

## 1. `/cli` — CLI Commands

- **Page title (H1):** "CLI Commands"
- **Tagline:** "All 32 CLI commands available in the CRONUS binary. From scaffolding to deployment."

### Core Commands table

| Command | Description |
|---|---|
| `cronus run [dir] [port]` | Parse all .cronus files and start the dev server |
| `cronus build` | Validate syntax without starting server |
| `cronus parse [file]` | Show parsed AST statistics |
| `cronus new [name]` | Scaffold a new project |
| `cronus test` | Run declared test blocks |
| `cronus test --conformance` | Run full spec conformance suite |

### Generation

| Command | Description |
|---|---|
| `cronus compose --from <template>` | Generate from template (saas-billing, blog, crm, helpdesk, ecommerce) |
| `cronus compose --entity <Name>` | Generate pages from entity definition |
| `cronus dump [url]` | Convert HTML page to .cronus sections |
| `cronus seed [entity] [N]` | Generate N test records for an entity |
| `cronus generate` | AI-assisted generation |

### Deployment

| Command | Description |
|---|---|
| `cronus deploy` | Generate Dockerfile + docker-compose.yml |
| `cronus export --openapi` | Export OpenAPI specification |

### Diagnostics

| Command | Description |
|---|---|
| `cronus doctor` | Diagnose project issues |
| `cronus stats` | Show project statistics |
| `cronus validate` | Validate against specs |
| `cronus version` | Show kernel version |

### Spec Commands

| Command | Description |
|---|---|
| `cronus spec validate` | Validate .spec.toml files |
| `cronus spec codegen --structs` | Generate Rust contracts from specs |
| `cronus spec codegen --docs` | Generate documentation from specs |
| `cronus spec codegen --ai-protocol` | Generate JSON schema for AI agents |

### Advanced Commands (15 more, listed with table + example)

| Command | Description | Example |
|---|---|---|
| `doctor` | Health check (Rust, SQLite, ports) | `cronus doctor` |
| `context` | AI context capsule (JSON) | `cronus context` |
| `memory` | Semantic memory viewer | `cronus memory` |
| `verify-audit` | Verify audit hash chain integrity | `cronus verify-audit` |
| `review` | Semantic code review | `cronus review` |
| `timeline` | Project timeline history | `cronus timeline` |
| `status` | Project status overview | `cronus status` |
| `drift` | Detect strategic/scope drift | `cronus drift` |
| `handoff` | Complete task + update state | `cronus handoff` |
| `lease` | Task lease management | `cronus lease` |
| `segment` | Block isolation | `cronus segment` |
| `reconcile` | AST merge of two .cronus files | `cronus reconcile a.cronus b.cronus` |
| `spec` | Spec validate/list/codegen | `cronus spec list` |
| `debug` | Run with request tracing | `cronus debug` |
| `changelog` | Git-based changelog generator | `cronus changelog` |

### Shell blocks (key invocations only; outputs abridged)

- `cronus run` → `CRONUS v0.1.0`, parses `app.cronus`, reports entities/pages/sections, migrates sqlite `./data.db`, binds `http://localhost:3000`.
- `cronus run ./myapp --port 8080` → binds `http://localhost:8080`.
- `cronus new my-app` → scaffolds dir with `app.cronus`, `data.db`, `static/`; prints "Run: cd my-app && cronus run".
- `cronus build` → `Validating app.cronus... OK`, entity/page/section counts, `0 errors, 0 warnings`.
- `cronus parse app.cronus` → `AST: 847 nodes`, lists Entities, Pages, Sections histogram.
- `cronus seed Order 50` → `50 records created`; `cronus seed --all 100` → seeds every entity.
- `cronus compose --from saas-billing` → `Generated: app.cronus (4 entities, 8 pages, 22 sections)` entities `User, Plan, Subscription, Invoice`.
- `cronus compose --entity Product` → generates `/products`, `/products/:id`, `/products/new`, `/products/edit`.
- `cronus deploy` → writes `Dockerfile`, `docker-compose.yml`, `.env.example`; "Ready: docker compose up -d".
- `cronus export --openapi` → `Exported: openapi.json (5 endpoints, 3 schemas)`.
- `cronus doctor` → `[OK]` syntax/DB/migrations/pages, sample `[WARN] Entity "Order" has no search fields`.
- `cronus audit` → example output `Accessibility: 98/100`, `Performance: 95/100`, `SEO: 87/100`.

Behavior notes: `cronus run` runs migrations and starts dev server with hot reload; `cronus seed` is "field-type-aware random generation"; `cronus audit` appears only in the doctor/audit example and not in any command table.

---

## 2. `/docker` — Docker

- **Page title (H1):** "Docker"
- **Tagline:** "Deploy CRONUS apps with Docker. Auto-generated Dockerfile and docker-compose.yml from a single command."

### CLI commands mentioned
- `cronus deploy` (only CRONUS verb on this page)

### Shell blocks
```
cronus deploy
```
```
# Build and start
docker compose up -d
# View logs
docker compose logs -f app
# Stop
docker compose down
```

### Generated Dockerfile (summary)
Block 1 ("Generated by cronus deploy"): `FROM rust:1.75-slim AS builder` → `cargo build --release` → `FROM gcr.io/distroless/cc-debian12`, copies `/app/target/release/cronus`, `*.cronus`, `data.db`, `EXPOSE 5175`, `CMD ["cronus", "run", ".", "5175"]`.

Block 2 ("Multi-Stage Build"): `FROM rust:1.78-slim AS builder` (note: different Rust version) → `FROM debian:bookworm-slim` + `ca-certificates`, `EXPOSE 5175`, `CMD ["cronus", "run"]`. "The final image is ~25MB."

### Generated docker-compose.yml (summary)
Single-service variant: builds `.`, maps `5175:5175`, mounts `./data.db`, env `JWT_SECRET=${JWT_SECRET}`, `restart: unless-stopped`.

Postgres variant: adds `db` service on `postgres:16-alpine` with `POSTGRES_USER=cronus`, `POSTGRES_PASSWORD=secret`, `POSTGRES_DB=cronus_db`, healthcheck `pg_isready -U cronus`; app depends_on `db` (service_healthy). App env: `CRONUS_ENV=production`, `DATABASE_URL=postgres://cronus:secret@db:5432/cronus_db`, `JWT_SECRET=your-secret-key-here`.

### Environment variables (Env Configuration table)
| Variable | Default | Description |
|---|---|---|
| `CRONUS_ENV` | `development` | Active environment (development, production) |
| `DATABASE_URL` | `sqlite:./data.db` | Database connection string |
| `JWT_SECRET` | (none) | Secret for JWT signing (required in production) |
| `PORT` | `5175` | HTTP server port |
| `LOG_LEVEL` | `info` | Logging level (debug, info, warn, error) |
| `CORS_ORIGIN` | `*` | Allowed CORS origins |
| `MAX_BODY_SIZE` | `1mb` | Maximum request body size |

### Cache System (on docker page)
- TTL 60s default, max 1000 entries, key `METHOD:PATH`, LRU eviction, auto-invalidates on `POST/PUT/PATCH/DELETE` per entity.

### Rate Limiting
- 100 req / 60s sliding window per IP, reads `X-Forwarded-For`, returns `429 Too Many Requests` with `Retry-After` header.

Default port stated by docker page: **5175** (conflicts with `cronus run` demo output in `/cli` which uses `3000`).

---

## 3. `/production` — Production

- **Page title (H1):** "Production"
- **Tagline:** "Deploy CRONUS apps to production. Guides for Fly.io, Railway, static export, and environment configuration."

### Production Checklist (verbatim 6 items)
1. Set `JWT_SECRET` to a strong random string (32+ characters)
2. Never use the fallback dev secret in production
3. Mark all password fields as `sensitive`
4. Use `auth:admin` for destructive operations
5. Test login flow after any auth-related changes
6. Back up data.db before deploying

### Shell blocks (summary)
- **Fly.io:** `curl -L https://fly.io/install.sh | sh` → `cronus deploy` → `fly launch --name my-cronus-app` → `fly secrets set JWT_SECRET=...` → `fly deploy`.
- **Railway:** `cronus deploy` → `railway init` → `railway up` → `railway variables set JWT_SECRET=...`.
- **Static export:** `cronus build --static` → outputs to `./dist/`. "Deploy the dist/ directory to any static host: Vercel, Netlify, Cloudflare Pages, or an S3 bucket."
- **Systemd:** `cp cronus.service /etc/systemd/system/` → `systemctl daemon-reload/enable/start/status cronus`.
- **Health check:** `curl -s localhost:5175/api/health` → JSON `{status, uptime, version: "0.1.4", database, entities, memory_mb}`. 200 when DB connected, 503 on DB failure.
- **Backup cron:** `sqlite3 $DB_PATH ".backup $BACKUP_DIR/data_$DATE.db"`, prune keeps last 30, crontab `0 2 * * * /opt/cronus/backup.sh`.

### `.cronus` example: env blocks (verbatim)
```
env development {
    DATABASE_URL "sqlite:./dev.db"
    JWT_SECRET "dev-secret-key"
}
env production {
    DATABASE_URL env(DATABASE_URL)
    JWT_SECRET env(JWT_SECRET)
}
```
"`env(VAR)` reads from system environment variables at runtime."

### Systemd unit `cronus.service`
`ExecStart=/usr/local/bin/cronus run`, `Restart=always`, `RestartSec=5`, env `CRONUS_ENV=production`, `DATABASE_URL=postgres://...`, `JWT_SECRET=...`, `After=network.target postgresql.service`.

### Nginx reverse proxy
Standard `proxy_pass http://127.0.0.1:5175` with `X-Real-IP`/`X-Forwarded-For` headers. Special `/api/sse/` block disables `proxy_buffering` and `proxy_cache` — "Without this, live bindings will not work through Nginx."

### Stripe / Payments
| Endpoint | Method | Description |
|---|---|---|
| `/api/checkout` | POST | Create Stripe Checkout session, returns redirect URL |
| `/api/webhooks/stripe` | POST | Handle Stripe webhook events |

Env var: `STRIPE_KEY` — without it, "CRONUS runs in mock mode for development".

Webhook events handled: `checkout.session.completed`, `customer.subscription.created`, `invoice.payment_failed`.

### i18n
HashMap-based, reads `Accept-Language`, fallback chain requested → default → key. Built-in locales: `en`, `pt-BR`. Example API:
```
i18n.add("en", "welcome", "Welcome")
i18n.add("pt-BR", "welcome", "Bem-vindo")
i18n.t("welcome", "pt-BR") → "Bem-vindo"
i18n.t("welcome", "fr") → "Welcome" (fallback)
i18n.t("unknown", "en") → "unknown" (key)
```

### Animations
"50+ built-in CSS animations with zero dependencies." Categories: Fades / Slides / Scale / Bounce / Rotate / Shake / Attention / Specials / Loading / Complex. Usage classes: `animate-fadeIn`, speed `anim-fast` (150ms) / `anim-slow` (600ms), delays `anim-d1..anim-d10`, scroll trigger `data-animate="fadeInUp"`, parent `stagger` class.

---

## 4. `/compiler` — Compiler Pipeline

- **Page title (H1):** "Compiler Pipeline"
- **Tagline:** "How CRONUS transforms declarative source code into running full-stack applications through tokenization, parsing, contract validation, and rendering."

### Claims about the compiler

Claims: single-pass Rust compiler doing **tokenization → parsing → contract validation → section dispatch → HTML rendering**. Not a classical IR/optimizer — it's a recursive-descent parser feeding a renderer dispatch table.

- **Tokenizer:** 30 token types. Keywords (verbatim): `app entity page section bind query auth style env script api template constitution must never on action required unique searchable sensitive default type tailwind_config true false`. Non-keyword tokens: `Ident, String, Number, Bool, LBrace, RBrace, Colon, Dot, Comma, Arrow (=>), Plus, Pipe, Newline, Comment, EOF`.
- **Parser & AST:** 17 node variants: `AppDecl, EntityDecl, PageDecl, SectionDecl, BindExpr, FieldDef, StyleDecl, EnvDecl, ScriptDecl, ApiRoute, AuthDecl, ConstitutionDecl, TemplateExpr, ActionDecl, TailwindConfig, QueryExpr, ConfigPair`.
- **Section dispatch:** "51 section types (plus aliases)". Aliases: `stats → kpi`, `404 → not-found`.
- **Render pipeline:** `tokenize → parse → resolve_contracts → validate (fails in --strict) → render_all_sections → build_server → listen(port)`.

### Build Modes table

| Flag | Contracts | AI Assist | Use Case |
|---|---|---|---|
| (none) | Warnings only | Off | Development |
| `--strict` | Hard errors | Off | CI / Production |
| `--ai` | Warnings only | On | AI-assisted development |
| `--strict-ai` | Hard errors | On | AI with full validation |

Shell:
```
cronus run
cronus run --strict
cronus run --ai
cronus run --strict-ai
```

### VM / bytecode sub-page (second half of /compiler)

- VM is a **stack machine** for `.scriptcronus` files. Modules: `opcodes`, `compiler` (AST → `Vec<OpCode>`), `executor` (stack machine + fuel metering).
- Page says "35 opcodes in 8 categories" but the module comment says `// 33 bytecode instructions`. The opcode table below lists **35 opcodes**: `PushStr, PushNum, PushBool, PushNull, Pop, LoadLocal, StoreLocal, Jump, JumpIfFalse, Halt, Equal, NotEqual, LessThan, GreaterThan, LessOrEqual, GreaterOrEqual, And, Or, Contains, GetField, Log, DbQuery, DbInsert, DbUpdate, DbDelete, IterBegin, IterNext, IterEnd, Respond, Now, EnvVar, AuthGetUser, AuthCheckRole, PushMap, MapInsert`.
- **Fuel:** default budget 10,000 / execution. All ops cost 1 fuel except `DbQuery, DbInsert, DbUpdate, DbDelete` which cost 10.
- Block kinds compiled: `OnEvent`, `OnWebhook`, `Schedule`, `Endpoint`.
- Example compilation:
  ```
  endpoint GET /api/users {
      let users = db.query User
      respond 200 users
  }
  →
  0 DbQuery("User")
  1 StoreLocal(0)
  2 LoadLocal(0)
  3 Respond(200)
  4 Halt
  ```
- Executor stores: value stack (init capacity 64), locals array (u16-indexed, auto-extends with null), iterator stack (for nested for-loops), single response slot.
- `DbInsert` auto-injects `_owner_id`; `DbUpdate`/`DbDelete` enforce owner checks.
- Truthiness: `false, null, "", 0, []` are falsy; objects always truthy. "Matches JavaScript semantics."

---

## 5. `/dump` — HTML Dump

- **Page title (H1):** "HTML Dump"
- **Tagline:** "Reverse-engineer any HTML page into clean CRONUS source code with automatic section detection, template extraction, and fidelity auditing."

### 10-Step Pipeline (verbatim)
1. Parse DOM — Load HTML and build DOM tree
2. Detect Theme — Extract colors, fonts, spacing from CSS
3. Identify Sections — Map DOM regions to CRONUS section types
4. Extract Content — Pull text, images, links from each section
5. Templatize — Replace dynamic content with template variables
6. Detect Bindings — Infer data sources from repeated patterns
7. Generate Entities — Create entity definitions from data shapes
8. Build Pages — Assemble page blocks with sections
9. Emit Source — Write .cronus source code
10. Audit — Compare output against reference HTML

### Section detection confidences
| Section type | Signal | Confidence |
|---|---|---|
| hero | h1 + subtitle + CTA button in first viewport | >95% |
| features | Grid of cards with icons/titles/descriptions | >90% |
| pricing | Cards with price, period, feature list | >92% |
| testimonial | Blockquotes with avatars and names | >88% |
| table | thead + tbody with structured data | >97% |
| chart | canvas or SVG with data visualization | >85% |
| faq | Accordion or dt/dd pairs | >90% |
| footer | Bottom section with links and copyright | >95% |

"Sections with confidence below 70% are emitted as `type:custom` with the raw HTML preserved in a template block." Docs claim "20+ section types".

### WebGL / Chart.js handling
- Unicorn Studio and canvas libs preserved as `template` blocks with original embed code.
- Chart.js canvases with data attributes converted to native `section chart` blocks (type, labels, datasets extracted from JS init).

### CLI Usage (verbatim)
```
# Basic dump
cronus dump page.html -o output.cronus

# Dump with audit against reference
cronus dump page.html -o output.cronus --audit

# Dump from URL
cronus dump https://example.com -o output.cronus

# Dump with strict section detection
cronus dump page.html -o output.cronus --strict
```

### Export Formats table (on /dump)

| Command | Output | Description |
|---|---|---|
| `cronus export` | `cronus-project.ir.json` | Full IR as JSON (entities, pages, sections, bindings) |
| `cronus dump page.html` | `.cronus` source | Reverse-engineer HTML into CRONUS source |
| `cronus spec codegen --docs` | Markdown | Generate docs from spec files |
| `cronus spec codegen --ai-protocol` | JSON schema | Export contracts as JSON schema for AI/LLM |
| `cronus spec codegen --structs` | Rust source | Generate `contracts_generated.rs` |

Extra shell examples on the page:
```
cronus export -o cronus-project.ir.json
cronus spec codegen --ai-protocol -o schema.json
cronus spec codegen --structs -o src/contracts_generated.rs
```

### Dump targets documented on `/dump`
Only the HTML → `.cronus` pipeline is described on this page. The extended format matrix (prisma/openapi/typescript) is **not** on `/dump` — it appears on `/templates` (see next section). **No mention of `next.js` or `vinext` as a dump target.** No limitations section, no admissions of parse failures, no mention of `HEAD`/`OPTIONS` HTTP methods.

---

## 6. `/templates` — Templates

- **Page title (H1):** "Templates"
- **Tagline:** "Override any section output with custom HTML templates while keeping the declarative CRONUS structure."

**Important:** this page is about **template overrides** (inline HTML inside section blocks), not about the 5 base / 8 ecosystem project templates. The only mention of project templates is on `/cli` in the `cronus compose --from` description: `saas-billing, blog, crm, helpdesk, ecommerce` (5 templates — the "5 base templates"). There is **no page in this crawl that lists 8 ecosystem templates**, and there is **no admission that any ecosystem template fails to parse**.

### `.cronus` examples (verbatim)

Basic override:
```
page "/" type:custom {
    section hero {
        title "Welcome"
        subtitle "Build faster with CRONUS"
        template "<div class='custom-hero'><h1>{{title}}</h1><p>{{subtitle}}</p></div>"
    }
}
```
"Template strings must be on a single line. Use `<br>` for line breaks within the template. All double quotes inside the template must be escaped."

Scoped CSS via inline `<style>`:
```
section hero {
    title "Styled Hero"
    template "<style>.hero-custom { background: linear-gradient(135deg, #0a0a0a, #1a1a1a); padding: 4rem 2rem; } .hero-custom h1 { font-size: 3rem; color: white; }</style><div class='hero-custom'><h1>{{title}}</h1></div>"
}
```
"Scoped CSS is extracted at compile time and injected into the page head."

Mustache-like list iteration:
```
section features {
    title "Features"
    items "Fast, Secure, Simple"
    template "<div><h2>{{title}}</h2><ul>{{#items}}<li>{{.}}</li>{{/items}}</ul></div>"
}
```

`style_block` key:
```
section hero {
    template "<div class='custom'>{{title}}</div>"
    style_block "body { background: #000 }"
}
```

### Template variables
| Variable | Source | Example |
|---|---|---|
| `{{title}}` | Section config | The section title |
| `{{subtitle}}` | Section config | The section subtitle |
| `{{items}}` | Bind query results | Array of entity records |
| `{{count}}` | Bind aggregation | Record count |
| `{{cta.text}}` | Nested config | CTA button label |
| `{{cta.url}}` | Nested config | CTA button URL |

### Dump format matrix (appears here, not on /dump)
```
cronus dump page.html -o output.cronus
```
| Format | Command | Output |
|---|---|---|
| CRONUS | `cronus dump page.html` | .cronus source with sections and templates |
| HTML | `cronus dump --format html` | Clean HTML with extracted styles |
| Prisma | `cronus dump --format prisma` | Prisma schema from detected entities |
| OpenAPI | `cronus dump --format openapi` | OpenAPI spec from API patterns |
| TypeScript | `cronus dump --format typescript` | TypeScript interfaces from entities |

### "When to use templates" guidelines (verbatim)
- "You need a layout not covered by the 51 built-in section types"
- "You are embedding third-party widgets (chat, analytics, maps)"
- "You need pixel-perfect control over specific sections"
- "You are converting an existing HTML design to CRONUS"
- "You need custom animations or interactions"
- "Templates bypass contract validation for the section content."
- "Use `cronus audit` to verify fidelity against a reference design."

---

## Cross-cutting flags: NOT in AGENTS.md cheatsheet

AGENTS.md cheatsheet lists: `cronus run/build/test/parse/dump/context/doctor/new` + `cargo build/test`. Everything below is documented on the rendered docs but NOT in the cheatsheet:

**Verbs not in cheatsheet:**
- `cronus compose` (`--from <template>`, `--entity <Name>`) — `/cli`
- `cronus seed` (`[entity] [N]`, `--all N`) — `/cli`
- `cronus generate` — `/cli`
- `cronus deploy` — `/cli`, `/docker`, `/production`
- `cronus export` (`--openapi`, `-o file`) — `/cli`, `/dump`
- `cronus stats` — `/cli`
- `cronus validate` — `/cli`
- `cronus version` — `/cli`
- `cronus spec` (+ `validate`, `list`, `codegen --structs`, `codegen --docs`, `codegen --ai-protocol`) — `/cli`, `/dump`
- `cronus audit` — `/cli` (only shown in example, not listed in any command table)
- `cronus memory` — `/cli`
- `cronus verify-audit` — `/cli`
- `cronus review` — `/cli`
- `cronus timeline` — `/cli`
- `cronus status` — `/cli`
- `cronus drift` — `/cli`
- `cronus handoff` — `/cli`
- `cronus lease` — `/cli`
- `cronus segment` — `/cli`
- `cronus reconcile a.cronus b.cronus` — `/cli`
- `cronus debug` — `/cli`
- `cronus changelog` — `/cli`

**Flags not in cheatsheet:**
- `cronus run [dir] [port]` (positional dir + positional port)
- `cronus run --port <N>`
- `cronus run --strict`, `--ai`, `--strict-ai` (`/compiler`)
- `cronus build --static` (`/production`) — static export flag
- `cronus test --conformance` (`/cli`)
- `cronus dump --audit`, `--strict`, `-o <file>`, `--format <html|prisma|openapi|typescript>` (`/dump`, `/templates`)

**Env vars not in cheatsheet:** `CRONUS_ENV, DATABASE_URL, JWT_SECRET, PORT, LOG_LEVEL, CORS_ORIGIN, MAX_BODY_SIZE, STRIPE_KEY`.

---

## Dump targets — observed vs. expected

Expected set (per the task): `html, next.js, prisma, openapi, vinext`.

**Docs actually advertise (union of `/dump` + `/templates`):**
- `cronus` (`.cronus` source) — primary output
- `html` (via `--format html`)
- `prisma` (via `--format prisma`)
- `openapi` (via `--format openapi`)
- `typescript` (via `--format typescript`) — **surprising, not in expected set**

**Missing from docs:**
- `next.js` — not mentioned on either page
- `vinext` — not mentioned anywhere in the 6 pages

`cronus export --openapi` (from `/cli`) is a separate command from `cronus dump --format openapi`, so OpenAPI has two distinct emit paths.

---

## HEAD / OPTIONS HTTP methods

No mention of `HEAD` or `OPTIONS` on any of the 6 rendered pages. The `/docker` Rate Limiting section only lists the write verbs `POST/PUT/PATCH/DELETE` as triggers for cache invalidation.

---

## Internal inconsistencies to flag

1. **Default port conflict:** `/cli` example shows `Server: http://localhost:3000` but `/docker` hardcodes `PORT=5175` everywhere (Dockerfile `EXPOSE 5175`, compose `5175:5175`, health check `curl localhost:5175/api/health`). `/docker` env table also lists `PORT` default = `5175`.
2. **Rust base image versions differ between the two Dockerfiles on `/docker`:** `rust:1.75-slim` (first block, "Generated by cronus deploy") vs `rust:1.78-slim` (second block, "Multi-Stage Build"). No explanation.
3. **Opcode count:** `/compiler` prose says "35 opcodes in 8 categories" but the module comment inside the same page says `// 33 bytecode instructions`. Counting the opcode table gives 35.
4. **Section type count:** `/compiler` says "51 section types (plus aliases)", `/templates` says "51 built-in section types", `/dump` says "20+ section types". Either the dump engine only recognizes a subset (likely), or the docs are inconsistent.
5. **"32 CLI commands"** claim on `/cli` — counting all command tables yields: Core (6) + Generation (5) + Deployment (2) + Diagnostics (4) + Spec (4) + Advanced (15) = **36** unique verbs listed. Overcounted or undercounted, depending on how you treat `cronus spec` as one vs. many.
6. **`cronus audit`** is used in example output but never appears in any of the command tables.
7. **`cronus test --conformance`** is advertised but no further docs exist on the other 5 pages.
8. **`cronus compose --from helpdesk`** is listed, but the `/templates` page never mentions helpdesk, crm, blog, ecommerce, or saas-billing as a unit — it's about HTML template overrides, not project scaffolds. The "5 base templates" claim lives only in the `/cli` table cell.
9. **No `/templates` page lists 8 ecosystem templates.** Not present. No admission of parse failures anywhere in the rendered docs.
