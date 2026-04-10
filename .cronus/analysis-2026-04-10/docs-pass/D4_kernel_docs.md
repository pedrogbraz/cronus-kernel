# D4 — Kernel Docs Pass

**Date:** 2026-04-10
**Scope:** `/home/zedd/Documentos/CRONUS/cronus-kernel/` kernel-side docs only
**Files audited:**
- `README.md`
- `CHANGELOG.md`
- `docs/LANGUAGE-REFERENCE.md`
- `docs/scriptcronus.md`

---

## 1. Executive Summary

The kernel ships four doc files that are internally consistent with each other but collectively frozen around **2026-04-03 → 2026-04-07**. The README is the most up-to-date (dated `2026-04-07`, 196 tests, 51 section types, 74 specs, 74,130 LOC) and matches `docs/LANGUAGE-REFERENCE.md` exactly on those numbers.

**Critical gap:** `CHANGELOG.md` stops at version **0.1.0 — 2026-04-03** with only one entry. None of the 2026-04-10 session work is recorded: Next.js dump, View Transitions, layout rewrite, optional `nav` keyword, removal of `#CC0000`, 8 ecosystem templates, or the HEAD/OPTIONS dumper fix. The README mentions "8 demos" and "51 section types" and `cronus dump [url]` — features that post-date the 0.1.0 changelog entry — so README advanced without CHANGELOG following.

`docs/scriptcronus.md` is the cleanest file: self-contained, well-structured, no staleness indicators.

`docs/LANGUAGE-REFERENCE.md` has one concrete staleness issue: it claims **"Compose templates | 3"** in the numbers table while the README says **"Compose templates | 5"** and lists 5 templates (blog, crm, ecommerce, helpdesk, saas). Internal contradiction across kernel docs.

---

## 2. README.md Summary

### Elevator pitch (exact quote)

> The declarative language that replaces your entire web stack.

Followed by:
> One `.cronus` file + one binary = full-stack app with auth, CRUD, dashboard, charts, real-time.
> No React. No Node. No npm. No config files.

### Metadata
- **Status:** Experimental
- **Version:** 0.1.0
- **Verified:** 2026-04-07
- **Closing line:** "Built in Rust. Designed for AI. Made in Brazil."

### Installation / Quick Start

```bash
cargo install --path .
cronus new my-app
cd my-app
cronus run
# -> http://localhost:5175
```

### Quick-start example
A ~50-line `.cronus` snippet showing an "Admin Panel" with `app`, `entity Order`, `auth`, `api /orders`, and `page "/"` with `kpi` + `table` sections (including an inline `on click` action). Demonstrates the full declarative stack in one screen.

### Badges / links / license / contributing
- **No badges** (no CI, no version shield, no license badge).
- **No LICENSE mention** anywhere in README.
- **No CONTRIBUTING section**, no code of conduct, no contact.
- Documentation pointers:
  - `docs/LANGUAGE-REFERENCE.md`
  - `docs/scriptcronus.md`
  - `../../docs/`
  - `../../specs/` (74 specs)

### Verified numbers table (claimed 2026-04-07)
| Metric | Value |
|---|---|
| Rust LOC | 74,130 |
| Rust files | 128 |
| Tests passing | 196/196 |
| AST nodes | 18 |
| Section types | 51 |
| Field types | 16 |
| Spec files | 74 |
| CLI commands | 32 |
| VM opcodes | 30 |
| Compose templates | 5 |
| Demo apps | 8 |

### Runtime features table
HTTP server (hyper), SQLite auto-migration, JWT+Argon2id, auto-CRUD REST, GraphQL + playground, SSR with 51 sections, data binding, 10 action verbs, search/filter + `X-Total-Count`, charts with GROUP BY, HMR, SSE, multi-file merge, tree-walk + bytecode VM (30 opcodes), Trust Engine (6-axis + 4 gates), Zeus observability, Hydra auto-promotion, ~7MB single binary.

### Section categorisation (51 total)
Data (11), Forms (4), Navigation (6), Feedback (10), Marketing (17), Layout (6).

### Demos table (8 apps)
`saas-billing` (NovaPay, reference), `stitch-dashboard`, `ecommerce`, `crm`, `blog`, `helpdesk`, `saas-dashboard`, `component-test`.

### Things that look stale in README
1. **No LICENSE / no contributing info** — standard OSS hygiene missing.
2. **"Verified: 2026-04-07"** — 3 days stale versus today (2026-04-10).
3. **Port inconsistency:** Quick Start shows `http://localhost:5175` but all demos and the reference `saas-billing` use 5220/5210/5200. The 5175 default doesn't match any demo.
4. **Action verbs count mismatch:** README runtime table says "10 verbs: set, toast, navigate, refresh, create, delete, validate, confirm, open, close" while CHANGELOG 0.1.0 still lists "6 instruction verbs: `set`, `toast`, `navigate`, `refresh`, `create`, `confirm`". README advanced, CHANGELOG did not.
5. **Section count drift:** README / LANGUAGE-REFERENCE say **51**, CHANGELOG 0.1.0 still says **48**.
6. **No mention of Next.js dump, View Transitions, layout rewrite** (2026-04-10 work) — README is still a 2026-04-07 snapshot.
7. **`cronus dump [url]`** appears in README CLI list but has no CHANGELOG entry (no "HTML → .cronus" shipping record), and the 2026-04-10 HEAD/OPTIONS dumper fix is invisible.

---

## 3. CHANGELOG.md Summary

### Latest version / date
**`[0.1.0] — 2026-04-03`** — and that is the ONLY version present.

### Format
Declares "The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)." Technically follows the shape (`## [version] - date` + `### Added` subsections), but:
- **No `## [Unreleased]` section** — a Keep-a-Changelog core convention.
- **Only `### Added`** — no `Changed`, `Fixed`, `Deprecated`, `Removed`, `Security`.
- **No links section** at bottom (no `[0.1.0]: <compare URL>`).
- **No entries after 0.1.0.**

### Last 3–5 entries
There is only **one** entry. The full 0.1.0 release from 2026-04-03 lists:
1. **Language:** `app` block, 16 field types, modifiers/constraints, state machines (`transition`), auth, api, page with **48 section types** (stale — now 51), `bind`, 6 action verbs (stale — now 10), effects, style, layout, component, `show:when`, constitution rules (`must`/`never`).
2. **Runtime:** hyper HTTP, SQLite auto-migration, JWT+Argon2id, auto-CRUD with 400/409/422, GraphQL + playground, SSR with 48 sections, HMR, SSE, search/filter/pagination, chart aggregation.
3. **CLI (32 commands):** `run`, `new <template>` with templates (blog/crm/ecommerce/helpdesk/saas), `build --strict`, `compose`, `seed`, `test`, `test --conformance`, `generate`, `parse`, `spec validate|list|codegen`, `deploy` (Fly.io/Railway/static), `doctor`, `stats`, `export`, `dump` (here described as "Import from OpenAPI/Prisma schemas" — NOT the HTML→.cronus dump the README advertises).
4. **Multi-Agent:** multi-file compose, zero merge conflicts, AI protocol via JSON schema.
5. **Templates:** 5 production templates.
6. **Security:** XSS auto-escape, JWT+Argon2id, input validation, audit logging.
7. **Quality:** 65,000+ LOC Rust (stale — now 74,130), **150 tests** (stale — now 196), 68 specs (stale — now 74), 90 conformance tests, 13 lint rules, semantic analysis, AI-Error Protocol, 6.6MB binary.

### Are there unreleased entries?
**No.** No `[Unreleased]` header. Changelog is effectively abandoned after the 0.1.0 initial dump.

### Consistency with known 2026-04-10 shipped work
Compared against the work the caller says landed today, the CHANGELOG is **completely silent** on all of it:

| 2026-04-10 shipped | In CHANGELOG? |
|---|---|
| Next.js dump | NO |
| View Transitions | NO |
| Layout rewrite | NO |
| `nav` keyword made optional | NO |
| Removal of `#CC0000` hardcoded color | NO |
| 8 ecosystem templates | NO (still says "5 production templates") |
| HEAD/OPTIONS dumper fix | NO |

Even the delta that is already visible in the README (51 sections, 196 tests, 74,130 LOC, 10 action verbs, 8 demos, `cronus dump [url]` as HTML→.cronus) has never been written back into CHANGELOG. There is a **~1 week gap** between changelog and reality.

---

## 4. LANGUAGE-REFERENCE.md Structure

### Line count
**759 lines.**

### Header metadata (line 3–4)
> Versão: 0.1.0 | Atualizado: 2026-04-07
> 74,130 LOC | 128 arquivos Rust | 196 testes | 74 specs | 51 section types

Matches README exactly.

### Table of contents (implicit — numbered top-level sections, in order)
The file has no explicit TOC block but uses numbered `##` headers:

0. Visão Geral (architecture ASCII diagram)
1. **Arquivo .cronus — Linguagem Declarativa**
2. **Arquivo .scriptcronus — Linguagem Imperativa**
3. **Block Composer**
4. **Trust Engine**
5. **Hydra Evolution**
6. **Zeus Observability**
7. **Bytecode VM**
8. **Security**
9. **CLI (32 comandos)**
10. **Estrutura de Projeto**
11. **Números**

### Full list of documented language constructs (exact order)

**Section 1 — .cronus declarative constructs:**
1. `app`
2. `entity` (with field types, modifiers, `transition`, `on create` effect)
3. `auth`
4. `api`
5. `page` (with `layout`, `requires`, `type`)
6. `section` — split into four subsections:
   - Data & Tables: `table`, `kpi`, `chart`, `kanban`
   - Forms: `form` (with `on submit`)
   - UI Components: `hero`, `pricing`, `modal`, `sheet`, `tabs`, `accordion`, `alert`, `timeline`, `progress`, `empty`, `skeleton`, `toast`, `breadcrumb`, `command`
   - Full list of 51 (verified against `ui/mod.rs`): accordion, activity-table, alert, bento, breadcrumb, chart, checkout, command, cta, dark-mode, dropdown, edge, empty, error, faq, features, features-split, filters, footer, form, grid, hero, info-bar, kanban, kpi, layout, loading, modal, notifications, page-header, pagination, policies, pricing, product-grid, progress, promo, quick-links, sheet, sidebar, stat-cards, stats, status-card, table, tabs, team-list, testimonial, timeline, toast, topbar, trusted, webhooks
7. `component` (params + `state` + `template`)
8. `bind` (query all/one/count, where, order, limit, offset, group, aggregate)
9. `layout`
10. `style`
11. `import` / `compose`
12. `test`
13. `webhook`
14. `worker`

**Section 2 — .scriptcronus imperative constructs:**
1. `script` (version block)
2. `on Entity.event { }` (create/update/delete)
3. `schedule "name" every:Xh { }`
4. `endpoint METHOD /path { }` (+ `auth:admin|public`)
5. `on webhook "/hooks/path" { }`
6. Builtins: `db`, `http`, `sse`, `log`, `format`, `env`, `auth`
7. Statements: `let`, `for`, `if`/`else`, `respond`
8. Operators: `==`, `!=`, `<`, `>`, `<=`, `>=`, `and`, `or`

### Code-block sanity check
Scanned all code fences — no mismatched braces, no obviously broken indentation, no references to non-existent keywords. A few minor stylistic points:

- The `entity` example uses `field name type:text required` syntax (prefixed with `field`), whereas the README's Quick-Start example uses bare `customer string required`. Two different declaration styles documented in two places. Not broken, but inconsistent. This may be the prefixed-`field` form surviving from an older parser dialect.
- The `pricing` example uses `item "Starter" { price $9/month, features [...] }` — the `$9/month` literal and trailing comma are non-standard token shapes worth verifying against the tokenizer; may be illustrative pseudocode rather than exact syntax.
- The `auth` example contains `jwt secret:"my-secret" expires:"7d"` — not shown in README (which uses `session jwt`). Two dialects for the same block.

### Keyword coverage probe
Requested terms and whether LANGUAGE-REFERENCE mentions them:

| Term | Mentioned? |
|---|---|
| `scriptcronus` | YES (section 2, also file-level banner) |
| `hydra` | YES (section 5, dedicated) |
| `trust` | YES (section 4, dedicated) |
| `constitution` | NO — the doc does NOT mention `constitution` / `must` / `never` even though CHANGELOG 0.1.0 ships them |
| `audit` | NO — not mentioned (CHANGELOG mentions "Audit logging" under Security, LANGUAGE-REFERENCE does not) |
| `zeus` | YES (section 6, dedicated) |
| `vm` | YES (section 7, Bytecode VM dedicated) |

**Gap:** Constitution rules (`must` / `never`) are listed in CHANGELOG 0.1.0 as shipped language features but have **zero** presence in `docs/LANGUAGE-REFERENCE.md`. If they exist, they are undocumented; if removed, the CHANGELOG is wrong.

### Other internal contradictions inside LANGUAGE-REFERENCE
- **Compose templates count:** Section 3 lists 3 templates (`saas-billing`, `blog`, `crm`) and the `## 11. Números` table says "Compose templates | 3". README says 5 (blog, crm, ecommerce, helpdesk, saas). Two authoritative docs in the same folder disagree by 2.
- **Field types count:** The doc text says "`text`, `email`, `password`, `number`, `money`, `boolean`, `date`, `json`, `enum`" = **9 types**. README and CHANGELOG claim **16** field types. LANGUAGE-REFERENCE is missing `string`, `url`, `slug`, `phone`, `percentage`, `ulid`, plus (possibly) `text` variations.
- **Modificadores count:** LANGUAGE-REFERENCE lists only `required`, `unique`, `default`. CHANGELOG lists 7 modifiers (`required`/`!`, `unique`, `sensitive`, `searchable`, `index`, `featured`, `optional`) plus 4 constraints (`min`, `max`, `match`, `default`).
- **Page types:** Listed as `custom`, `checkout`, `landing` — README Quick-Start uses `type:dashboard`, which is not in this list.
- **Compose templates adapted:** `Product → Service` adapt example is good, but the ad-hoc entity example `cronus compose --entity Product name:text price:money! status:text` uses the `money!` required-shorthand, which is not documented under Modificadores.
- **Hydra blocks | 300** in the numbers table — unsourced magic number, no corroboration elsewhere.

---

## 5. scriptcronus.md Summary

### What is `.scriptcronus`?
Yes — a mini-language. Quote from line 2:

> Camada imperativa do CRONUS para integrar apps externas, automações e webhooks.
> Arquivos `.scriptcronus` vivem ao lado dos `.cronus` e são carregados automaticamente no boot.

Positioned explicitly as the **imperative counterpart** to the declarative `.cronus`:
> `.cronus` = declarativo (o quê). `.scriptcronus` = imperativo (como integrar).

### Stability
**Not labeled experimental.** The README as a whole flags the project as "Experimental | Version 0.1.0", but this doc reads as if the scripting layer is a stable, shipped feature (no caveats, no "preview", no "unstable" warnings). It documents hard limits, owner isolation, and a boot output that treats scripts as first-class.

### Syntax cheatsheet (all 4 block types)

```
# Event handler
on Entity.create | update | delete { ... }

# Cron
schedule "name" every:30s|5m|1h|6h|1d { ... }

# Custom HTTP route (auth default = JWT required)
endpoint GET|POST|PUT|PATCH|DELETE /path [auth:admin|billing|public] { ... }

# Webhook receiver (path must start with /hooks/)
on webhook "/hooks/name" { ... }
```

**Statements:** `let`, `for item in collection`, `if / else`, `respond status body [headers]`.

**Operators:** `==`, `!=`, `<`, `>`, `<=`, `>=`, `and`, `or`.

**Builtins (7 namespaces):**
- `db.query`, `db.create`, `db.update`, `db.delete`, `db.count` (owner-isolated for non-admin)
- `http.get|post|put|patch|delete` with headers/body/json
- `sse.broadcast channel { payload }`
- `log "msg with {{interpolation}}"` (max 2048 chars, 50 entries/exec)
- `format.csv data [cols]`, `format.json data`
- `env.VAR_NAME` (only project-declared env, never `std::env`)
- `auth.check_role "admin"`, `auth.get_user`

**Automatic variables in `on Entity.event`:** `event.id`, `event.entity`, `event.event`, `event.record`, `event.record.field`, `event.prev` (update only).

**Automatic variables in `on webhook`:** `event.body`, `event.body.field`.

### Execution limits
- 1000 statements per run
- 100 variables in scope
- 50 log entries
- 2048 chars per log message
- 1 MB webhook body cap
- Whitelisted response headers only: `Content-Type`, `Content-Disposition`, `Cache-Control`, `X-Request-Id`, `X-Total-Count`
- Newlines stripped from headers (injection prevention)
- Webhooks run as role `webhook` (not admin)

### Use cases / positioning
Three worked examples at the bottom:
1. **Stripe integration** — `on Customer.create` posts to Stripe + stores `stripe_id`, plus `on webhook "/hooks/stripe"` for `invoice.paid`.
2. **Admin tools / Export API** — `endpoint GET /api/export/customers auth:admin` returning CSV via `format.csv`.
3. **Notifications** — `schedule "overdue_invoices" every:6h` posting to Slack + `on Order.create` broadcasting SSE.

**Fit vs `.cronus`:** `.cronus` handles declarative app shape (data, pages, API, auth). `.scriptcronus` handles side effects and integrations: reacting to CRUD, cron jobs, custom HTTP routes, webhooks from external services. They coexist in the same folder and boot together.

### Quality
Cleanest of the four files. No staleness flags, no contradictions with itself, fully worked boot-output example showing `2 (3 events, 1 schedules, 2 endpoints, 1 webhooks)`. No TODO markers.

---

## 6. Staleness Findings (cross-doc)

| # | Finding | Where | Severity |
|---|---|---|---|
| 1 | CHANGELOG frozen at 0.1.0 / 2026-04-03 — no entry for 2026-04-04..10 work | CHANGELOG.md | **Critical** |
| 2 | CHANGELOG says "48 section types" — reality is 51 | CHANGELOG.md | High |
| 3 | CHANGELOG says "6 action verbs" — README says 10 | CHANGELOG.md | High |
| 4 | CHANGELOG says "150 tests / 65,000 LOC" — README says 196 / 74,130 | CHANGELOG.md | High |
| 5 | CHANGELOG says "5 production templates" — if today's work shipped 8 ecosystem templates, neither CHANGELOG nor README reflects it | CHANGELOG.md, README.md | **Critical** |
| 6 | `cronus dump` described as "OpenAPI/Prisma" in CHANGELOG vs "HTML → .cronus" in README | CHANGELOG.md vs README.md | High — semantic drift of a CLI command |
| 7 | No `## [Unreleased]` section | CHANGELOG.md | Medium (Keep-a-Changelog violation) |
| 8 | No LICENSE / CONTRIBUTING / badges | README.md | Medium (OSS hygiene) |
| 9 | Quick Start shows port 5175, no demo uses 5175 | README.md | Low (cosmetic) |
| 10 | LANGUAGE-REFERENCE.md says "Compose templates | 3" | docs/LANGUAGE-REFERENCE.md | High (contradicts README) |
| 11 | LANGUAGE-REFERENCE lists only 9 field types vs CHANGELOG's 16 | docs/LANGUAGE-REFERENCE.md | High (feature under-documented) |
| 12 | LANGUAGE-REFERENCE lists only 3 modifiers vs CHANGELOG's 7 | docs/LANGUAGE-REFERENCE.md | High |
| 13 | LANGUAGE-REFERENCE never mentions `constitution` / `must` / `never` (shipped per CHANGELOG) | docs/LANGUAGE-REFERENCE.md | High |
| 14 | Page type `dashboard` used in README but not listed in LANGUAGE-REFERENCE's page-types enum | README vs docs/LANGUAGE-REFERENCE | Medium |
| 15 | Two declaration dialects for entity fields coexist (`field name type:text required` vs bare `customer string required`) | README vs docs/LANGUAGE-REFERENCE | Low — but confusing for new users |
| 16 | No mention of Next.js dump / View Transitions / layout rewrite / optional `nav` / `#CC0000` removal / HEAD/OPTIONS dumper fix anywhere in kernel docs | all four files | **Critical** (all of 2026-04-10 is invisible) |
| 17 | LANGUAGE-REFERENCE "Hydra blocks | 300" unsourced, no cross-reference | docs/LANGUAGE-REFERENCE.md | Low |

---

## 7. CHANGELOG Gap vs Reality (2026-04-10)

**Expected session output (per caller):**
1. Next.js dump
2. View Transitions
3. Layout rewrite
4. `nav` keyword made optional
5. Removal of hardcoded `#CC0000`
6. 8 ecosystem templates
7. HEAD/OPTIONS dumper fix

**Actually recorded in CHANGELOG:** 0 of 7.

**Recorded elsewhere in kernel docs:**
- 8 demos: YES, README demos table lists 8 rows (`saas-billing`, `stitch-dashboard`, `ecommerce`, `crm`, `blog`, `helpdesk`, `saas-dashboard`, `component-test`). README "Verified numbers" says `Compose templates | 5 | templates/*.cronus` AND `Demo apps | 8 | demos/`. So the "8" is demos/, not compose templates — compose templates are still 5 per README, 3 per LANGUAGE-REFERENCE. If the "8 ecosystem templates" the caller refers to are in `templates/`, README says there are still 5 there. If they are in `demos/`, README already reflects 8. Ambiguity not resolvable from docs alone.
- Next.js dump / View Transitions / layout rewrite / nav-optional / `#CC0000` removal / HEAD/OPTIONS dumper fix: **none are mentioned in README, CHANGELOG, LANGUAGE-REFERENCE, or scriptcronus.md.**
- `cronus dump [url]` exists in README CLI list with signature "HTML -> .cronus converter" — this is the only trace that the dumper family exists at all in kernel docs. HEAD/OPTIONS behavior is entirely undocumented.

**Bottom line:** The kernel's CHANGELOG is effectively 7 days behind the kernel's own README, and the README itself is 3 days behind the codebase. None of the 2026-04-10 work is visible in any kernel doc file. A CHANGELOG rewrite pass and a README bump to `Verified: 2026-04-10` are needed.

---

## Sources
- `/home/zedd/Documentos/CRONUS/cronus-kernel/README.md`
- `/home/zedd/Documentos/CRONUS/cronus-kernel/CHANGELOG.md`
- `/home/zedd/Documentos/CRONUS/cronus-kernel/docs/LANGUAGE-REFERENCE.md`
- `/home/zedd/Documentos/CRONUS/cronus-kernel/docs/scriptcronus.md`
