# D3 — Live CRONUS Docs Sites Audit

**Date:** 2026-04-10
**Scope:** `/home/zedd/Documentos/CRONUS/docs/site-v2/app.cronus` (CURRENT)
and `/home/zedd/Documentos/CRONUS/docs/site/app.cronus` (REPLACED).
Only these two files were read; `.cronus/`, `backup/`, and `*.bak` were skipped.

---

## 1. Executive Summary

- **site/app.cronus** is a **pure-DSL** docs site: 1,942 lines, ~123 KB, 6 top-level
  pages, zero embedded HTML. It is the textbook "how a .cronus doc site should
  look" example, using only built-in sections (`topbar`, `hero`, `features`, `cta`,
  `footer`).
- **site-v2/app.cronus** is a **template-injection** docs site: 801 lines but
  **1.22 MB on disk**, 42 pages, 66 lines containing giant inline HTML blobs via
  a `template "..."` mechanism. The .cronus file is almost entirely a thin DSL
  shell around a pre-baked shadcn-style HTML documentation site.
- site-v2 is **strictly newer and much broader in scope** (42 pages vs 6), but
  it is NOT a pure-DSL rewrite — it is the same rendering stack driving a
  largely HTML-authored site. The two sites exercise the language VERY
  differently and are both worth keeping around as test corpora.
- No HEAD/OPTIONS, no middleware block, no cache:, no render:static|island|stream
  DSL usage was found. The words do appear in site-v2's HTML prose (describing
  those features to the reader), but never as real .cronus syntax.
- One non-obvious top-level keyword shows up in site-v2: **`tailwind_config`**
  (a string-valued top-level directive). Not present in site/. Worth confirming
  the parser supports it deliberately.

---

## 2. site-v2/app.cronus anatomy (CURRENT — docs.cronus.test)

### 2.1 File stats
- Line count: **801**
- Byte size: **1,217,055 bytes (~1.22 MB)**
- Longest line: **82,059 chars** (line 225, a giant HTML template string)
- Top 10 longest lines all in the 28 KB–82 KB range → these are all inline HTML
  payloads inside `template "..."` values

### 2.2 `app` declaration (lines 5–10)
```
app "CRONUS Design System" {
  stack react + tailwind
  port 4900
  database sqlite "./data.db"
  theme dark
}
```

### 2.3 `style` block (lines 12–18)
```
style {
  theme dark
  accent "#CC0000"
  font "Geist"
  mono "Geist Mono"
  radius md
}
```
Accent color = `#CC0000` (deep red). Both `mono` and `radius` subkeys are used
— consistent with the monochromatic + 1-accent house rule.

### 2.4 Other top-level blocks
- `entity` blocks: **0**
- `api` blocks: **0**
- `layout` blocks (top-level): **0**
- `auth` blocks: **0**
- `tailwind_config "..."`: **1** (line 20) — one-line directive whose string
  value is a raw JS object literal extending Tailwind with custom colors and
  fonts. See §6.

### 2.5 `page` blocks (42 total)
All declared as `page "/route" type:custom`. Grouped by intent:

**Getting Started / Guide (10 pages):**
`/`, `/project-structure`, `/first-app`, `/entities`, `/pages`,
`/data-binding`, `/actions`, `/auth`, `/api-routes`, `/scripting`

**Components (8 pages):**
`/components`, `/components/hero`, `/components/features`,
`/components/pricing`, `/components/table`, `/components/chart`,
`/components/kpi`, `/components/form`

**Ops / Runtime / Tools (7 pages):**
`/cli`, `/field-types`, `/docker`, `/production`, `/compiler`,
`/hydra-system`, `/compose`

**Advanced / System (8 pages):**
`/dump`, `/audit`, `/constitution`, `/sse-live`, `/graphql-api`,
`/contracts`, `/microservices`, `/templates`

**Live previews (8 pages) — the only pages that use real section DSL:**
`/preview/hero`, `/preview/features`, `/preview/pricing`, `/preview/kpi`,
`/preview/cta`, `/preview/faq`, `/preview/testimonial`, `/preview/stats`

### 2.6 Page internal shape
Every non-preview page follows the **same two-section skeleton**:

```cronus
page "/route" type:custom {
  section topbar {
    brand "CRONUS"
    nav "Guide, Components, API"
    template "<header class=\"fixed top-0 w-full z-50 ...\">...</header>"
  }
  section layout {
    template "<div class=\"flex max-w-[1440px] mx-auto ...\">...</div>"
  }
}
```

The `template` string on `section layout` is where **all** actual docs content
lives — sidebar nav, headings, copy, code samples, related-topics lists, the
whole page — rendered as one raw HTML string, shadcn-style. So the .cronus
file has 42 pages but only **2 unique section kinds** per page.

Preview pages are the exception: they use real DSL sections (`hero`, `features
style:bento`, `pricing` with `plan`, `kpi cols:4`, `cta`, `faq`, `testimonial`,
`stats cols:4`) and zero templates. These are the "here's what the real CRONUS
renderer outputs" comparison pages.

### 2.7 DSL vs embedded HTML breakdown (site-v2)
Counting bytes per line classification:

| Category              | Lines | Bytes      | % of file |
|-----------------------|------:|-----------:|----------:|
| `template "..."` HTML |    66 | 1,198,733  | **98.6 %**|
| DSL (everything else) |   735 |    17,221  |   1.4 %   |

So only ~17 KB of the 1.2 MB file is actually CRONUS DSL. Roughly 260 DSL
lines would be the "real" size if you stripped the HTML payloads.

### 2.8 What documentation content is embedded
From sidebar / section headings parsed out of the templates:

- Introduction (CRONUS at a glance, installation summary)
- Project Structure (layout of a .cronus project)
- First App walkthrough
- Entities, Pages, Data Binding, Actions, Auth, API Routes, Scripting
- 51-section component library with dedicated pages for hero, features,
  pricing, table, chart, kpi, form
- CLI reference
- Field types reference
- Docker / Production deployment
- Compiler internals
- Hydra System (mentioned — relates to multi-terminal orchestration)
- Compose (modular .cronus files)
- Dump, Audit, Constitution (Objective Kernel / governance)
- SSE live updates
- GraphQL API
- Contracts
- Microservices
- Templates

### 2.9 `layout` block? `auth` block?
- **No top-level `layout` block.** The `section layout { template "..." }` used
  per-page is a regular section named `layout`, not a top-level layout
  primitive. Every page re-declares it individually.
- **No `auth` block** anywhere. Auth is discussed in docs prose only.

---

## 3. site/app.cronus anatomy (REPLACED)

### 3.1 File stats
- Line count: **1,942**
- Byte size: **123,319 bytes (~123 KB)**
- 100 % DSL — **zero `template` strings**

### 3.2 `app` declaration (lines 7–12)
```
app "CRONUS Docs" {
  stack react + tailwind
  port 4900
  database sqlite "./data.db"
  theme dark
}
```
Same port (4900) as site-v2 — confirming site-v2 replaced site/ on the same
domain, not running side-by-side.

### 3.3 `style` block (lines 14–18)
```
style {
  theme dark
  accent "#f59e0b"   # amber
  font "Inter"
}
```
No `mono`, no `radius`, no `tailwind_config`. Different accent color (amber)
vs site-v2's red.

### 3.4 `page` blocks (6 total)
| Route             | Purpose                                       |
|-------------------|-----------------------------------------------|
| `/`               | Home / overview + core concepts               |
| `/getting-started`| Install + first app walkthrough + CLI quickref|
| `/language`       | Full grammar reference (app, entity, page, style, auth, api, action, service, compose, Objective Kernel files, additional blocks, quick-ref table) |
| `/sections`       | Every section type with syntax and examples  |
| `/examples`       | Todo, SaaS landing, blog, e-commerce, admin…  |
| `/tools`          | CLI reference for all 27 commands             |

### 3.5 Section usage
- Total `section` instances: **87** (vs site-v2's 74)
- Section kinds used: `topbar`, `hero`, `features` (with variants `cols:1`,
  `cols:3`, `cols:12 gap:4`, `style:cards`), `cta`, `footer`
- `nav` directive appears 12 times (every topbar + every footer) using the
  CSV-string form: `nav "Getting Started, Language, Sections, Examples, Tools"`
- Zero top-level `entity`, `api`, `auth`, `layout`, `tailwind_config`

### 3.6 Documentation content embedded (site/)
- Home: CRONUS pitch, 8 core concept cards (One File Architecture, Declarative
  UI, Built-in DB, Auth, API Routes, Deploy, Objective Kernel, Control Plane,
  Anti-Drift AI, Site Dumping)
- Getting Started: System requirements, install via cargo, install via GitHub
  release, verify, build a task manager step-by-step, full starter file, CLI
  quick reference, project structure explanation, AI coordination (Objective
  Kernel + constitution + leases)
- Language Reference: 10 numbered sections — app, entity, page, style, auth,
  api, action, service, compose/import, Objective Kernel files, plus
  "Additional Blocks" (including a `middleware` rate-limit example) and a
  quick-reference table
- Sections: 9 detailed section-type walkthroughs (topbar, hero, features, cta,
  footer, pricing, testimonial, faq, form) plus a quick-ref of all section
  types
- Examples: 4 featured example apps
- Tools: 1-by-1 CLI reference for all 27 commands (with flags)

### 3.7 DSL vs embedded HTML breakdown (site/)
| Category              | Lines | Bytes    | % of file |
|-----------------------|------:|---------:|----------:|
| `template "..."` HTML |     0 |       0  |     0 %   |
| DSL                   | 1,942 | 112,000  |   100 %   |

---

## 4. Content comparison — what topics are in each

| Topic                     | site/ | site-v2 |
|---------------------------|:-----:|:-------:|
| Home / introduction       |  Yes  |   Yes   |
| Install / first app       |  Yes  |   Yes   |
| Project structure         |  Yes  |   Yes   |
| Entity reference          |  Yes  |   Yes   |
| Page reference            |  Yes  |   Yes   |
| Data binding              |   —   |   Yes   |
| Actions                   |  Yes  |   Yes   |
| Auth                      |  Yes  |   Yes   |
| API routes                |  Yes  |   Yes   |
| Scripting                 |   —   |   Yes   |
| Field types               |   —   |   Yes   |
| Components overview       |   —   |   Yes   |
| Per-component pages (7)   |   —   |   Yes   |
| Section types (full list) |  Yes  |  partial|
| CLI reference             |  Yes  |   Yes   |
| Docker                    |   —   |   Yes   |
| Production deployment     |   —   |   Yes   |
| Compiler internals        |   —   |   Yes   |
| Hydra System              |   —   |   Yes   |
| Compose / modular imports |  Yes  |   Yes   |
| Dump command              |  Yes  |   Yes   |
| Audit                     |   —   |   Yes   |
| Constitution              |   —   |   Yes   |
| Objective Kernel          |  Yes  |   Yes   |
| SSE live                  |   —   |   Yes   |
| GraphQL API               |   —   |   Yes   |
| Contracts                 |   —   |   Yes   |
| Microservices             |   —   |   Yes   |
| Templates                 |   —   |   Yes   |
| Examples (Todo, SaaS…)    |  Yes  |   —     |
| Live component previews   |   —   |   Yes   |

### 4.1 What site-v2 added
- **+36 pages** beyond what site/ covered
- Full component library (one dedicated page per major section type)
- Docker + production deployment walkthroughs
- Compiler internals, Hydra multi-terminal system, Audit, Constitution
- SSE, GraphQL, Contracts, Microservices, Templates
- Live `/preview/*` pages that render real DSL sections as visual samples

### 4.2 What site-v2 removed
- **`/examples` page** — site/ has a "Run Any Example." page listing Todo,
  SaaS landing, blog, e-commerce, admin dashboard. site-v2 has no equivalent
  top-level Examples page. If marketing/docs relies on showing complete
  reference apps, this is a real regression and should be ported to site-v2.
- **`/language` single-page grammar reference** — site/ consolidates the
  entire grammar on one scrollable page (`/language`) with numbered sections
  1–10. site-v2 splits this into many per-page topics, which is more
  discoverable but loses the printable "one page that explains everything"
  experience.
- **`/sections` gallery** — site/ has a single section-types index page with
  every section kind. site-v2's equivalent is split across /components and
  /preview/* and does not cover all 51 section types.
- **`section footer`** with tagline and copyright — site/ ends every page with
  a real `footer` section; site-v2 bakes the footer into its HTML templates.

---

## 5. site → site-v2 verdict

| Dimension                       | site/ wins                 | site-v2 wins                |
|---------------------------------|----------------------------|-----------------------------|
| Breadth of topics               |                            | **Yes** (42 vs 6 pages)     |
| Visual polish                   |                            | **Yes** (shadcn-styled HTML)|
| Pure-DSL / exercises compiler   | **Yes** (100 % DSL)        |                             |
| Small + easy to review          | **Yes** (123 KB)           |                             |
| Grammar reference completeness  | **Yes** (covers all blocks)|                             |
| Examples gallery                | **Yes**                    |                             |
| Live component previews         |                            | **Yes** (/preview/*)        |

**site-v2 is NOT a strict superset** of site/ — it dropped the grammar
reference, the examples gallery, and the section-type gallery that site/ had.
The work it added is substantially larger in surface area, but the
removed content is real and should be migrated forward (see §7).

---

## 6. Syntax surprises found

1. **`tailwind_config "<JS object literal>"`** (site-v2 line 20) — a
   **top-level directive** outside any block. Not something a basic .cronus
   cheatsheet would mention. It takes a single string whose contents are
   interpreted (presumably) by the renderer as a Tailwind config fragment.
   Not used in site/. **Action:** confirm the parser intentionally supports
   this, add it to the cheatsheet, or deprecate it if it was accidental.

2. **`template "<huge raw HTML>"`** as a section child — used 66 times in
   site-v2, zero times in site/. This is the dominant authoring pattern in
   site-v2. Needs to be documented explicitly: when does a section accept a
   `template` override vs rendering its native layout? It appears `topbar`
   and `layout` both tolerate it in site-v2, but the preview pages prove the
   same sections render fine without it.

3. **`section layout { ... }`** — site-v2 uses `layout` as a **section name**.
   The SDDs talk about a top-level `layout` block (for shared chrome across
   pages). Here it is a per-page section that just holds the main template.
   Easy source of confusion: is `layout` reserved, or just another arbitrary
   section identifier? Should probably not be both.

4. **`style { radius md mono "Geist Mono" }`** — site/ uses only
   `theme/accent/font`; site-v2 adds `mono` and `radius`. These look fine but
   should be documented as supported style keys.

5. **`features style:bento`** (site-v2 `/preview/features`) — a `style:bento`
   modifier on `features`. site/ only uses `style:cards`. Worth confirming
   the compiler supports `bento` and that it's in the section catalog.

6. **`stats cols:4`** with plain string item bodies (`item "Section Types"
   icon:widgets { "51" }`) — simple `{ "literal" }` one-liner body. Looks
   valid, noted for the cheatsheet.

7. **`plan "Pro" $29/mo featured [...]`** — the `pricing` section uses a
   `plan` keyword with a `$29/mo` literal token and a `featured` modifier,
   followed by a bracketed string array. This is a bespoke mini-grammar
   inside pricing and worth documenting explicitly.

### 6.1 Things NOT found (good news)
- **Zero HEAD method** usage. Good — this was a recent bug area.
- **Zero OPTIONS method** usage. Good.
- **Zero real `middleware` block** usage. The word "middleware" appears 16×
  inside site-v2 template strings (as prose describing the feature in the HTML
  docs) and 2× in site/ prose. No .cronus file actually declares one.
- **Zero `cache:` directives**.
- **Zero `render:static` / `render:island` / `render:stream`**.
- **Zero bare sidebar `"Label" -> "/route"`** shortcut syntax. Both files use
  the explicit `nav "csv, list"` form (site/) or inline HTML `<a>` tags
  (site-v2). The optional no-`nav` shortcut is not exercised here.
- **Zero GET/POST/PUT/PATCH/DELETE** route lines — neither file has an `api`
  block at all, so HTTP methods aren't represented.

---

## 7. Which to keep, which to archive

### Recommendation: **KEEP BOTH, MOVE site/ TO AN ARCHIVE FOLDER**

**Keep site-v2/** as the production docs (it's already what docs.cronus.test
serves, and it has 7× more content):
- It's 42 pages of real documentation content
- The visual treatment is what users actually see
- Port 4900 collision would force one to be chosen anyway

**Keep site/** as a **language fixture / compiler test corpus**:
- 100 % DSL, no HTML escape hatch → stresses the actual CRONUS compiler
- Covers grammar sections site-v2 doesn't (full Language Reference page, all
  51 section types, Examples gallery)
- Tiny (123 KB) → fast to parse in CI
- Archive it at `docs/archived-site-pure-dsl/app.cronus` (or similar) with a
  README noting it is NOT served, it's a compiler test fixture

**Before archiving site/, port these missing pieces to site-v2:**
1. `/examples` top-level page (Todo, SaaS landing, Blog with Auth, E-commerce,
   Admin dashboard)
2. `/language` — single-page grammar reference covering all top-level blocks
3. `/sections` — complete gallery of all 51 section types (site-v2 only has
   7 component pages + 8 preview pages)
4. Footer section with tagline, copyright, and link groups

**Action items for the language team:**
- Decide whether `tailwind_config` is a first-class directive. If yes,
  document; if no, refactor site-v2 to use `style { ... }` subkeys.
- Decide whether `layout` is a reserved section name or not. Don't let
  site-v2's `section layout { template "..." }` pattern coexist with a
  future top-level `layout { ... }` block.
- Document `style:bento` variant of `features`.
- Document `section template "..."` as the HTML escape hatch, including
  which sections accept it.

---

## 8. Quick stats reference

|                           | site/         | site-v2          |
|---------------------------|---------------|------------------|
| Lines                     | 1,942         | 801              |
| Bytes                     | 123,319       | 1,217,055        |
| Longest line              | ~1,800 chars  | **82,059 chars** |
| `page` blocks             | 6             | 42               |
| `section` instances       | 87            | 74               |
| `template "..."` blobs    | 0             | 66               |
| `entity` blocks           | 0             | 0                |
| `api` blocks              | 0             | 0                |
| `auth` block              | no            | no               |
| top-level `layout` block  | no            | no               |
| `tailwind_config`         | no            | yes (1)          |
| Port                      | 4900          | 4900             |
| Accent color              | `#f59e0b`     | `#CC0000`        |
| Font                      | Inter         | Geist / Geist Mono|
| HEAD / OPTIONS methods    | none          | none             |
| Phase-1 features in DSL   | none          | none             |
