# D2 — Reference & Guides Pass

> **Scope:** `docs/reference/` (whole tree), `docs/guides/` (whole tree), `docs/DOCS-SPEC.md`
> **Date:** 2026-04-10
> **Agent:** docs-pass D2 (read-only)

---

## 1. Executive Summary (10 bullets)

1. **8 reference docs** (2,332 lines total) and **12 guides** (1,741 lines total) exist. Reference is numerically dominated by `section-types.md` (688 lines) which alone is ~30% of reference content.
2. All reference docs declare `Status: stable` except `runtime.md` which is `experimental`. All cite `parser.rs` / `ui.rs` as source of truth, consistent with DOCS-SPEC's contract.
3. Reference layer covers the "contract": grammar, AST, 16 field types, 52 section types (though text says 51), 13 CLI commands, renderer dispatch matrix, config keys, and the runtime lifecycle. This matches the DOCS-SPEC structure 1-to-1.
4. Guides cover task-oriented flows: entities, api, pages, sections, components, forms, auth, deploy, dump, responsive, security, testing. All but `responsive.md`, `dump.md`, `security.md`, and `testing.md` are `stable`; those four are `experimental` (or mixed).
5. `guides/testing.md` is a huge outlier (474 lines, dated 2026-04-10) — it was rewritten to absorb many 2026-04 features (VINEXT dump, View Transitions, SPA nav, declarative layout, dashboard type fix, AI tooling, tree-sitter, design-leak fix). It violates its own filename — it's really "runtime & 2026-04 changelog".
6. Discrepancy: `section-types.md` opens with "There are 52 section types" but then headers numbered 1-51, and the renderer matrix in `renderer-support.md` lists 52 rows. DOCS-SPEC claims "Section types: 51 (36 dedicated + 5 generic + 10 external)". These three counts are internally inconsistent.
7. The golden path for a new user through guides is: `entities -> api -> auth -> pages -> sections -> components -> forms -> deploy`. Secondary (advanced): `testing`, `security`, `dump`, `responsive`.
8. `DOCS-SPEC.md` is the internal contract defining the 3-layer docs structure (getting-started/guides/reference/internal), stability badges, header conventions, and parser-grounding rule. It lists expected files; the current `docs/` tree matches almost perfectly for `guides/` and `reference/`, but the `internal/` folder is not visible in this scope.
9. The 2026-04 testing guide leaks a large set of unreferenced features into the docs surface: declarative `layout` block, `requires:auth` / `requires:role`, `transition` blocks, `on click/submit/change` actions, `data-list` auto-fetch, `data-cronus-form`, `--conformance` flag, `cronus context --for-claude`, `/api/_context`, AI Context Protocol, VS Code extension, tree-sitter grammar. None of these are in `reference/grammar.md`, `reference/cli.md`, or `reference/ast.md`.
10. No stability-badges for deprecated, no `internal/change-policy.md` or `release-notes.md` visible here (DOCS-SPEC lists them as expected). Reference is uniformly "stable" with one "experimental" — no deprecated features flagged anywhere.

---

## 2. Reference Inventory

| File | Path | Lines | Status | Topic | One-line Summary |
|---|---|---|---|---|---|
| ast.md | `docs/reference/ast.md` | 291 | stable | AST node schema | Rust struct definitions for all 14 `AstNode` variants + public parser API (`parse`, `parse_with_imports`, `parse_directory`, `stats`). |
| cli.md | `docs/reference/cli.md` | 177 | stable | CLI commands | All 13 `cronus` commands with flags, templates, and behavior. |
| config.md | `docs/reference/config.md` | 257 | stable | Config keys | Exhaustive key:value table for app / style / page / section / component / entity / api / worker / field / env contexts. |
| field-types.md | `docs/reference/field-types.md` | 121 | stable | Field types | 16-type `FieldType` enum with DB mapping, parse rules for enum/relation/array, and 8 modifiers. |
| grammar.md | `docs/reference/grammar.md` | 310 | stable | Grammar | Token kinds (16), 17 keywords, EBNF-ish rules per top-level node, arrays, arrow, ColonPair, Price. |
| renderer-support.md | `docs/reference/renderer-support.md` | 163 | stable | Renderer matrix | Maps every section type to its render function and module (ui.rs / board.rs / data_table.rs / overlays.rs / command_palette.rs / layout_system.rs) with Dedicated/Shared/Generic/External classification. |
| runtime.md | `docs/reference/runtime.md` | 147 | experimental | Runtime model | Interpret-on-the-fly lifecycle, `hyper` server, request handling, SQLite schema generation, HMR via SSE, multi-file mode, output stack. |
| section-types.md | `docs/reference/section-types.md` | 688 | stable | Section catalog | One-entry-per-section reference for all 51/52 types with example `.cronus` per type, grouped by Marketing / Dashboard / Interactive / Data / State / Layout / Generic. |

### Representative code per reference doc

**ast.md** — public API:
```rust
pub fn parse(source: &str) -> Result<Vec<AstNode>, String>
pub fn parse_with_imports(source: &str, base_dir: &str) -> Result<Vec<AstNode>, String>
pub fn parse_directory(dir: &str) -> Result<Vec<AstNode>, String>
pub fn stats(nodes: &[AstNode]) -> (usize, usize, usize)
```

**cli.md** — deploy:
```bash
cronus deploy            # Dockerfile + docker-compose.yml + .dockerignore
cronus deploy --fly      # + fly.toml
cronus deploy --railway  # + railway.json
cronus deploy --static   # prints instructions
```

**config.md** — page block:
```cronus
page "/users" type:list entity:User layout:light-app {
  title "Users"
  columns [name, email, role]
  search "Search users..."
  filters [role, status]
  actions [create, edit, delete]
  recent User limit:5
  use Sidebar
}
```

**field-types.md** — modifiers:
```cronus
email email required unique searchable
```

**grammar.md** — entity:
```cronus
entity User {
  name    string required
  email   email  required unique
  role    enum   [admin, member, viewer]
  team    -> Team
  tags    string[]
}
```

**renderer-support.md** — dispatch snippet:
```rust
"hero" => render_hero(section, accent, theme),
"features" => render_features(section, accent, theme),
...
"kanban" => crate::board::render_kanban(section),
_ => render_generic_section(section, accent),
```

**runtime.md** — lifecycle:
```
1. Parse .cronus file(s) -> Vec<AstNode>
2. Extract entities, pages, components, APIs, style
3. Create/migrate SQLite database
4. Start HTTP server (hyper)
5. On each request: render HTML or execute CRUD
6. Watch files for HMR
```

**section-types.md** — hero example:
```cronus
section hero style:dark {
  badge "New"
  title "Ship faster"
  cta "Start" -> "/signup" primary
}
```

### Reference-level vs tutorial-only coverage

**Covered at reference level** (contract-grade):
- 14 AST node variants — `ast.md`
- 16 field types + 8 modifiers — `field-types.md`, `ast.md`, `config.md`
- 51/52 section types with renderers — `section-types.md`, `renderer-support.md`
- 17 keywords + 16 token kinds + full EBNF — `grammar.md`
- 13 CLI commands + templates + deploy flags — `cli.md`
- App / Style / Page / Section / Component / Entity / API / Worker / Field / Env config keys — `config.md`
- SQLite schema generation, auto CRUD routes, HMR, multi-file mode — `runtime.md`
- 8 component layouts (`inline`, `stack`, `grid`, `table`, `hero`, `modal`, `sidebar`, `tabs`) — `config.md`, `ast.md`

**NOT in reference, only in guides** (or only in testing.md):
- `layout Main { sidebar {...} }` declarative layout block
- `requires:auth` / `requires:role(...)` page modifier
- `transition` blocks (state machines) — mentioned in `testing.md`
- `on click/submit/change` action blocks
- `webhook` top-level block — listed in tree-sitter coverage only
- `deploy` as a top-level block (distinct from the CLI command) — listed in tree-sitter coverage only
- `define` top-level block — listed in tree-sitter coverage only
- Bindings syntax (`query`, `aggregate`, `where`, `order`, `limit`) — tree-sitter coverage only
- Dynamic routes `[slug]`, `[...catch]`, `[[...optional]]`
- `cronus generate` / `cronus context` / `cronus test --conformance`
- View Transitions, Speculation Rules, SPA navigation runtime features

### API-reference-style pages

- `ast.md` — alphabetical by enum variant (AppNode, ApiNode, ComponentNode, ...) — the closest thing to an API reference.
- `field-types.md` — 16-row alphabetical-ish table of types.
- `section-types.md` — 51 numbered entries; not alphabetical but exhaustive catalog.
- `config.md` — categorical tables of keys with types + defaults.
- `renderer-support.md` — 52-row dispatch matrix.

No alphabetical "function index" page exists. There is no `functions.md` or `api.md` at the reference level. All reference pages are organized by concept/category.

---

## 3. Guides Inventory

| File | Path | Lines | Status | 1-line Purpose |
|---|---|---|---|---|
| api.md | `docs/guides/api.md` | 169 | stable | How to declare REST routes, auth modes, built-in auth endpoints, auto-generated GraphQL, and consuming APIs from list/form pages. |
| auth.md | `docs/guides/auth.md` | 129 | stable | 4-step JWT auth walkthrough: define User entity, declare auth routes, build login/signup pages, protect routes. |
| components.md | `docs/guides/components.md` | 126 | stable | Reusable components with 8 layouts, `use` syntax, style chains, and a full topbar+sidenav example. |
| deploy.md | `docs/guides/deploy.md` | 130 | stable | `cronus deploy` flows for Docker, Fly.io, Railway, and static export; health check at `/api/health`. |
| dump.md | `docs/guides/dump.md` | 77 | experimental | `cronus dump` reverse-engineers HTML pages into `.cronus`; lists detection patterns and limitations. |
| entities.md | `docs/guides/entities.md` | 158 | stable | Defining entities with 16 field types + 8 modifiers; enums, relations, arrays, common patterns (auth user, billing, team). |
| forms.md | `docs/guides/forms.md` | 155 | stable | `section form` with 17 field input types, properties, select/checkbox, login/signup pattern. |
| pages.md | `docs/guides/pages.md` | 182 | stable | 6 page types (`dashboard`, `list`, `form`, `detail`, `custom`, `checkout`), page-level helpers, using components via `use`. |
| responsive.md | `docs/guides/responsive.md` | 77 | experimental | `cols-md` / `cols-sm` / `hide-on:mobile` / `responsive:scroll` config keys and their limitations. |
| sections.md | `docs/guides/sections.md` | 138 | stable | Section syntax, config keys, content primitives, items, rows, common patterns (stats grid, promo, pricing). |
| security.md | `docs/guides/security.md` | 120 | experimental | Security model overview, JWT defaults, `sensitive` modifier, stability classification for security-critical features, production checklist. |
| testing.md | `docs/guides/testing.md` | 474 | stable+experimental | 12-section omnibus: testing, VINEXT dump, view transitions, prefetch, SPA nav, cache headers, declarative layout, dashboard type fix, AI tooling, VS Code ext, tree-sitter, design-leak fix. |

---

## 4. Golden Path for New Users

Based on cross-references (`See X Guide`) and logical dependency order:

```
1. entities.md       — data model (entities + fields)
2. api.md            — wire CRUD routes to entities
3. auth.md           — protect those routes + define User entity
4. pages.md          — create UI pages bound to entities
5. sections.md       — fill pages with sections
6. components.md     — extract reusable topbar/sidebar
7. forms.md          — build signup/login/edit forms
8. deploy.md         — ship it (Docker / Fly / Railway)
```

**Secondary / advanced:**
```
9.  testing.md       — testing + all 2026-04 runtime features (very heavy)
10. security.md      — production checklist and stability for auth
11. dump.md          — migration from existing HTML/Next.js apps
12. responsive.md    — mobile breakpoint overrides
```

Cross-reference links confirming golden path (quoted from the files):
- `entities.md` -> "See [Field Types Reference](../reference/field-types.md)"
- `api.md` -> "See [Auth Guide](auth.md) for the full authentication walkthrough."
- `pages.md` -> "See [Sections Guide](sections.md) for details" + "See [Section Types Reference](../reference/section-types.md)"
- `forms.md` -> "See [Auth Guide](auth.md) for the complete login/signup flow."
- `components.md` -> self-contained, referenced from `pages.md`
- `auth.md` -> "See [API Guide](api.md) for more on route configuration."
- `security.md` -> "See [Auth Guide](auth.md) for the full setup" + "See [Section Types Reference](../reference/section-types.md)"

---

## 5. DOCS-SPEC.md Summary

**Purpose:** The internal docs contract — an SDD (Spec Driven Development) document stating that "esta spec é o contrato. Docs devem refletir ela." It is the source of truth for how CRONUS docs are organized and what counts as "real".

**Prescribed structure (quoted):**

```
docs/
  README.md                          # Índice + visão geral
  getting-started/
    installation.md                  # stable
    first-app.md                     # stable
    first-dashboard.md               # stable
    first-api.md                     # stable
  guides/
    entities.md                      # stable
    api.md                           # stable
    pages.md                         # stable
    sections.md                      # stable
    components.md                    # stable
    responsive.md                    # experimental
    forms.md                         # stable
    auth.md                          # stable
    deploy.md                        # stable
    dump.md                          # experimental
    testing.md                       # experimental
    security.md                      # experimental
  reference/
    grammar.md                       # stable
    ast.md                           # stable
    field-types.md                   # stable
    section-types.md                 # stable
    cli.md                           # stable
    renderer-support.md              # stable
    runtime.md                       # experimental
    config.md                        # stable
  internal/
    change-policy.md                 # internal
    stability-levels.md              # internal
    rfc-template.md                  # internal
    release-notes.md                 # internal
```

**Key principles (Next.js-inspired):**
1. Three fixed layers: getting-started (tutorial) / guides (problem-solving) / reference (exact contract)
2. Separate "how to use" from "what exists"
3. Source of truth = `parser.rs` + `ui.rs` — no doc can lie about the parser
4. Stability badges mandatory: `stable`, `experimental`, `internal`, `deprecated`
5. No example is accepted unless it passes the real parser
6. No primitive in reference without a real renderer
7. Do not document the future as if it were the present

**Numbers stated as source of truth (from DOCS-SPEC):**
- Top-level nodes: **14** (app, entity, api, page, style, component, service, import, compose, on, worker, middleware, env, test)
- Field types: **16**
- Section types: **51** (36 dedicated + 5 generic + 10 external)
- CLI commands: **13**
- Component layouts: **8** (inline, stack, grid, table, hero, modal, sidebar, tabs)

**Does current `docs/` tree match?**

- **`guides/`: MATCH.** All 12 files exist with matching names and matching stability labels per DOCS-SPEC, except `testing.md` is labeled as stable+experimental in its header while DOCS-SPEC says pure `experimental`.
- **`reference/`: MATCH.** All 8 files exist with matching names and stabilities. `section-types.md` is labeled stable and contains 51-52 types as promised.
- **`internal/`: NOT VISIBLE IN SCOPE.** Cannot confirm whether `change-policy.md`, `stability-levels.md`, `rfc-template.md`, `release-notes.md` exist — scope excluded `internal/`.
- **`getting-started/`: NOT IN SCOPE.** Excluded per task.
- **`README.md`: NOT IN SCOPE.**

**Mismatch with the parser:**
- DOCS-SPEC says 14 top-level nodes and names them, but the list in DOCS-SPEC omits `section` (section is not top-level — it's nested in page). The 14 listed are correct for top-level.
- DOCS-SPEC says "Component layouts: 8" but `config.md` lists 9: `inline, stack, grid, table, hero, modal, sidebar, tabs, menu`. The `ast.md` comment also lists 9 (adds `menu`). So DOCS-SPEC's count is slightly out-of-date.
- DOCS-SPEC says "Section types: 51" but `section-types.md` text says "There are 52" and `renderer-support.md` matrix has 52 rows (row 52 = `layout`). `dark-mode` is listed as the 52nd entry but "counted within the 51".

---

## 6. Advanced / New Concepts Found in Guides (not yet in reference)

These features are mentioned in guides (mostly `testing.md`) but have no dedicated reference doc coverage. Flagged for main conversation context:

### Declarative layout system (new in 2026-04-10)
```cronus
layout Main {
  brand "Kronos"
  sidebar {
    "Dashboard"   -> "/dashboard" icon:dashboard
    "Projects"    -> "/projects" icon:folder
  }
}
```
Also: the `nav` keyword becomes optional — bare `"Label" -> "/route"` is accepted.

### Page modifiers: `requires:auth`, `requires:role(...)`
```cronus
page "/admin" type:dashboard requires:auth { ... }
```
Applied automatically to gate access and trigger `render_layout_declarative`.

### `transition` blocks (state machines)
Listed in tree-sitter coverage. No syntax shown in guides, no reference doc.

### `on click/submit/change` action blocks
Mentioned in tree-sitter coverage as "Actions (on click/submit/change)". Only the event-level `on` (for worker-style events) is in `grammar.md`.

### Bindings
Mentioned in tree-sitter coverage: "Bindings (query, aggregate, where, order, limit)". No reference docs. Templates use `data-list="entity"` for auto-fetched tables.

### New top-level blocks mentioned but not in reference grammar
From the tree-sitter coverage list in `testing.md`:
- `auth { ... }` block (distinct from `api /auth { ... }`)
- `webhook`
- `deploy` as a top-level block
- `define`
- `layout`
- `transition`

### CLI features not in `cli.md`
- `cronus test --conformance [--dir tests/conformance]` — conformance suite with parse-positive/parse-negative/warnings
- `cronus context --for-claude` — export project context as markdown
- `cronus generate "..." --dry-run` — save system prompt to file
- `cronus dump /path --nextjs` — explicit Next.js flag
- Description-based `cronus generate "..."` using `ANTHROPIC_API_KEY`

### Runtime features not in `runtime.md`
- View Transitions API auto-injection (`<meta name="view-transition">`)
- Hover prefetch + Speculation Rules (`data-prefetch="eager"`)
- SPA navigation (global link interception, `window.CRONUS.navigate/transition/prefetch/reload`)
- Cache-Control headers by content type
- `/api/health` health endpoint (mentioned in `deploy.md`)
- `/api/_context` HTTP endpoint for AI project state
- `/api/brain/stats` brain analytics (mentioned briefly in `runtime.md`)
- `data-cronus-form` auto-submission attribute
- `data-list="entity"` auto-fetched tables
- `.scriptcronus` file extension (VS Code extension highlights it)

### Tooling not in reference
- VS Code extension (`cronus-vscode/`) with 18 snippets
- Tree-sitter grammar (`tree-sitter-cronus/`, grammar.js + highlights.scm)
- Agent Skill for AI tools (`.agents/skills/generate-cronus/SKILL.md`)
- `AGENTS.md` / `CLAUDE.md` at kernel root

### Concepts from task prompt that DO NOT appear in reference/guides scope
Listed for completeness — the following terms were pre-flagged and I searched: none appear in the scoped files.
- `hydra`, `trust`, `constitution`, `scripting`, `vm`, `zeus`, `audit`, `brain` (only `CronusBrain` mentioned once in `runtime.md`), `block evolution`, `trust scoring`, multi-tenant `shared` flag, `AI Error Protocol`
- `brain` appears once in `runtime.md` ("Optional `CronusBrain` for analytics") and once referenced by `/api/brain/stats` — no dedicated doc.
- `constitution rules` is mentioned once inside `testing.md` under AI Context Protocol output ("Returns entities, pages, API routes, auth config, constitution rules, relationship graph, memory") — no definition, no syntax, no reference.

---

## 7. Stale / Suspicious Claims

Items I can assert with high confidence from comparing files within scope. I avoid speculating about code behavior.

1. **`cli.md` says `cronus version` outputs `cronus v0.1.0`** — but `testing.md` says `window.CRONUS.version = "0.6.0"`. One of these is stale.
2. **`cli.md` describes `cronus dump page.html`** (HTML reverse-engineering only) — but `testing.md` and `guides/dump.md` also describe `cronus dump /path/to/nextjs-app [--nextjs]` which is missing from `cli.md`.
3. **`cli.md` template list is `landing | saas | api | ecommerce | blog`** — `testing.md` mentions `cronus generate "..."` descriptions and "TaskFlow + E-Commerce" canonical examples, but `cli.md`'s template list is probably still accurate.
4. **Section count inconsistency.** `DOCS-SPEC.md` says 51, `section-types.md` overview says 52, `renderer-support.md` matrix has 52 rows numbered. The header in `section-types.md` line 8 says "52 section types recognized" but sub-entries are numbered 1-51 with `dark-mode` unnumbered. One of these is out-of-date; I cannot say which without reading `ui.rs`.
5. **Component layouts count.** `DOCS-SPEC.md` says 8, `components.md` says "8 types" and lists 8, but `config.md` and `ast.md` both list 9 (`inline, stack, grid, table, hero, modal, sidebar, tabs, menu` — adding `menu`). Either `DOCS-SPEC`+`components.md` are stale, or `menu` is a recent addition.
6. **`guides/testing.md` claims sidebar layout "5 strategies"** (initial, pushState/replaceState override, popstate, interval polling, click handler) — cannot verify without reading render.rs. Not flagging as stale, just noting for D3/D4.
7. **`runtime.md` route list is incomplete.** It lists `/__cronus/version`, `/api/brain/stats`, `/api/{entity}s`, `/{route}`. It omits `/api/auth/*` (mentioned in `auth.md` and `api.md`), `/api/health` (mentioned in `deploy.md`), `/api/_context` (mentioned in `testing.md`), `/graphql` (mentioned in `api.md`).
8. **`testing.md` says "0.6.0"** version but `cli.md` says "v0.1.0". Definite version skew.
9. **`cli.md` does NOT document `cronus test --conformance`** — only `cronus test [port]`. `testing.md` documents `--conformance --dir tests/conformance`.
10. **`grammar.md` keywords list is 17** — the listed 17 are `app entity api page style service section import compose use merge on worker component middleware env test`. Missing from this list but used in guides: `layout`, `auth`, `webhook`, `deploy`, `define`, `transition`, `requires`. Either these are not real top-level keywords (testing.md overpromises), or grammar.md is stale.

---

## 8. Gaps — Topics That Should Have Reference/Guides But Don't

### Missing reference pages

- **`layout.md`** — The declarative layout block (added 2026-04-10) is a first-class primitive in `testing.md` but has zero reference coverage. No `LayoutNode` in `ast.md`, no grammar rule in `grammar.md`, no config in `config.md`.
- **`requires.md` / Page modifiers** — `requires:auth`, `requires:role(...)` are gating primitives with no reference coverage.
- **`transitions.md`** — `transition` blocks (state machines) are mentioned in tree-sitter coverage and snippet list but have no syntax or reference.
- **`actions.md`** — `on click/submit/change` event actions (the form/UI flavor, not the `on order.created` worker flavor) have no reference.
- **`bindings.md`** — `query`, `aggregate`, `where`, `order`, `limit` bindings are in tree-sitter coverage with no reference or guide.
- **`http-routes.md` / runtime endpoints** — No single reference for all auto-generated HTTP routes (`/api/*`, `/graphql`, `/__cronus/version`, `/api/brain/stats`, `/api/health`, `/api/_context`, `/api/auth/*`).
- **`auth-block.md`** — `auth { ... }` block (mentioned in tree-sitter coverage) vs `api /auth { ... }` — unclear which is canonical. No reference.
- **`webhook.md`, `deploy-block.md`, `define.md`** — Listed in tree-sitter coverage as top-level blocks, not in grammar.md.
- **Dynamic route reference** — `[slug]`, `[...catch]`, `[[...optional]]`, parallel slots `@slot/`, route groups `(group)/` are only mentioned in dump context, not as first-class CRONUS routing primitives.

### Missing guides

- **`layout.md` guide** — How to build dashboard shells with the declarative layout block. Currently buried in section 7 of `testing.md`.
- **`navigation.md` / SPA guide** — View Transitions, prefetch, SPA navigation. Buried in `testing.md`.
- **`generate.md` / AI tooling guide** — `cronus generate`, `cronus context`, AI Context Protocol, Agent Skill usage. Buried in `testing.md`.
- **`migration.md`** — Next.js/VINEXT migration via `cronus dump --nextjs`. Currently split between `dump.md` (HTML-only) and `testing.md` (Next.js-only).
- **`editor-support.md`** — VS Code extension, tree-sitter grammar, snippets. Currently buried in `testing.md`.

### Missing CLI documentation

- `cronus test --conformance [--dir]`
- `cronus context --for-claude`
- `cronus generate --dry-run`
- `cronus dump --nextjs`
- `cronus generate "<description>"` via `ANTHROPIC_API_KEY`

### Missing stability / lifecycle docs

- No `internal/stability-levels.md` visible in scope
- No `internal/change-policy.md` visible
- No `internal/release-notes.md` visible — the 2026-04-10 changelog is effectively inside `guides/testing.md`, which is wrong per DOCS-SPEC principle "Do not document the future as the present" and the three-layer separation rule.

### Structural concern

The guide `testing.md` (474 lines) has absorbed what should be ~7 separate documents. This is a clear organizational debt: a single experimental file holds stable declarative layout, stable SPA runtime, stable cache headers, experimental tests, stable AI tooling, stable VS Code ext, and a 2026-04-10 changelog. Splitting it would cut reference/guide gaps significantly.

---

## End of D2 report
