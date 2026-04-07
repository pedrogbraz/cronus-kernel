# Changelog

All notable changes to CRONUS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

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
