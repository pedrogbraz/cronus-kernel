# R2 — Docs Edit Map: site-v2/app.cronus

**Source:** `/home/zedd/Documentos/CRONUS/docs/site-v2/app.cronus` (801 lines, ~1.2 MB)
**Mode:** READ ONLY — no modifications made.
**Note:** File is misnomered as "~20KB" in the brief — actual size is ~1.2 MB because every page inlines its sidebar HTML and all section templates. Line 36 (landing template) = 28,975 chars; line 542 (/dump template) = 25,462 chars.

---

## 0. Structural Overview

The file has a flat structure: one `app { }` block, one `style { }` block, one `tailwind_config` string, then ~40 top-level `page "/path" type:custom { }` blocks. Every page has the exact same shape:

```
page "/xxx" type:custom {
  section topbar {            # line N+2..N+5
    brand "CRONUS"
    nav "Guide, Components, API"
    template "<header...>"    # full topbar HTML, identical across pages
  }
  section layout {             # line N+8..N+10
    template "<div ...>"       # full sidebar + main + right-aside HTML
  }
}
```

**There is no `layout Main { sidebar { ... } }` block.** The sidebar is inlined as raw HTML inside every page's `section layout` template, duplicated ~40 times. Same for the topbar. This means any sidebar edit must be replicated across all pages, or done via find-and-replace.

**Landing page `/` is NOT a marketing landing page.** It is the docs "Introduction" page — functionally a getting-started doc with the same chrome as every other page. There is no hero, no CTA, no features tile grid.

---

## 1. Landing page `/` (lines 27–38)

### 1.1 Block structure

```
Line 27:  page "/" type:custom {
Line 28:  (blank)
Line 29:    section topbar {
Line 30:      brand "CRONUS"
Line 31:      nav "Guide, Components, API"
Line 32:      template "<header ... (full fixed topbar with search, github icon, /v0.1 badge)>"
Line 33:    }
Line 34:  (blank)
Line 35:    section layout {
Line 36:      template "<div class=\"flex max-w-[1440px]...\"> ... full content ... </div>"
Line 37:    }
Line 38:  }
```

Lines 32 and 36 are each a single giant string. Line 36 is 28,975 chars. To insert new content on the landing page you must edit inside line 36 — there is no multi-line block you can just append to.

### 1.2 Topbar (line 32) — abbreviated, identical across all pages

```html
<header class="fixed top-0 w-full z-50 backdrop-blur-md bg-[#020202]/90 border-b border-white/5 font-['Geist',system-ui,sans-serif]">
  <div class="flex h-14 items-center justify-between max-w-[1440px] mx-auto px-6">
    <div class="flex gap-8 items-center">
      <a href="/" class="flex items-center gap-2.5 text-white font-bold text-lg tracking-tight">
        <svg width="20" height="20" viewBox="0 0 24 24" ...>
          <path d="M12 2L2 7v10l10 5 10-5V7L12 2z" fill="#CC0000"/>
          <path d="M12 6l-5 2.5v5L12 16l5-2.5v-5L12 6z" fill="#020202"/>
        </svg>
        CRONUS<span class="text-[10px] font-mono text-neutral-600 ml-1">v0.1</span>
      </a>
      <nav class="hidden md:flex items-center gap-6">
        <a href="/" class="text-sm font-medium text-white border-b border-[#CC0000] pb-0.5">Guide</a>
        <a href="/components" class="text-sm text-neutral-500 hover:text-neutral-300 transition-colors">Components</a>
        <a href="/api-routes" class="text-sm text-neutral-500 hover:text-neutral-300 transition-colors">API</a>
      </nav>
    </div>
    <div class="flex items-center gap-3">
      <div class="relative"><input type="text" placeholder="Search docs..." .../>...</div>
      <a href="https://github.com/cronuslang/cronus" ...><svg .../></a>
    </div>
  </div>
</header>
<div class="h-14"></div>
```

Top-level nav links (global): **Guide**, **Components**, **API** — only three. No direct link to `/dump` from the topbar.

### 1.3 Layout template (line 36) — section-by-section breakdown

Line 36 is structured as:

```
<div class="flex max-w-[1440px] mx-auto font-['Geist',system-ui,sans-serif]">
  <aside class="hidden lg:block w-64 ...">        <!-- LEFT SIDEBAR -->
    <nav class="p-6 space-y-6">
      <div>Getting Started ul</div>
      <div>Core Language ul</div>
      <div>Components ul</div>
      <div>Reference ul</div>
      <div>Advanced ul</div>
      <div>Deploy ul</div>
      <div>Auto-Generated ul</div>
    </nav>
  </aside>
  <main class="flex-1 min-w-0">
    <div class="flex">
      <div class="flex-1 max-w-3xl px-10 py-10" id="doc-content">
        <nav breadcrumb>Docs > Introduction</nav>
        <h1>Introduction</h1>
        <p class="text-lg text-neutral-400 ...">CRONUS is a declarative language ...</p>
        <section id="philosophy">...</section>
        <section id="quick-start">...</section>
        <section id="design-principles">...</section>    <!-- 3-tile grid -->
        <section id="typography">...</section>
        <section id="color-palette">...</section>
        <section id="section-types">...</section>       <!-- 51 types list -->
        <section id="endpoints">...</section>           <!-- REST table -->
        <section id="dashboards">...</section>          <!-- runtime dashboards table -->
        <section id="related">...</section>             <!-- 3-tile icon links -->
        <footer>Prev/Next</footer>
      </div>
      <aside class="hidden xl:block w-48 ..."><h4>On This Page</h4>...</aside>
    </div>
  </main>
</div>
```

### 1.4 Left sidebar verbatim (left aside, lines within line 36)

Sidebar groups and items (quoted from the raw template):

**Getting Started**
- `/` Introduction  [ACTIVE on landing: `class="... text-[#CC0000] font-medium ... bg-[#CC0000]/5 border-l-2 border-[#CC0000]"`]
- `/project-structure` Installation
- `/first-app` Your First App

**Core Language**
- `/entities` Entities
- `/pages` Pages
- `/data-binding` Data Binding
- `/actions` Actions
- `/auth` Auth
- `/api-routes` API Routes
- `/scripting` ScriptCronus

**Components**
- `/components` Overview
- `/components/hero` Hero
- `/components/features` Features
- `/components/pricing` Pricing
- `/components/table` Table
- `/components/chart` Chart
- `/components/kpi` KPI
- `/components/form` Form

**Reference**
- `/cli` CLI Commands
- `/field-types` Field Types
- `/contracts` Contracts
- `/templates` Templates

**Advanced**
- `/compiler` Compiler
- `/hydra-system` Hydra System
- `/compose` Block Composer
- **`/dump` HTML Dump**   ← this is the dump page entry
- `/audit` Audit System
- `/constitution` Constitution
- `/sse-live` Live Updates
- `/graphql-api` GraphQL
- `/microservices` Microservices

**Deploy**
- `/docker` Docker
- `/production` Production

**Auto-Generated**
- `/docs` API Docs (ext)
- `/docs/design` Design System (ext)
- `/docs/graph` Entity Graph (ext)

Every item uses this exact class pattern (inactive):
```html
<li><a href="/xxx" class="block text-sm text-neutral-400 hover:text-white py-1 px-2 transition-colors">Label</a></li>
```
Active item (per-page) swaps the classes:
```html
<a href="/xxx" class="block text-sm text-[#CC0000] font-medium py-1 px-2 rounded bg-[#CC0000]/5 border-l-2 border-[#CC0000]">Label</a>
```

### 1.5 Hero / "what CRONUS is" copy (verbatim)

Breadcrumb → H1 → lead paragraph:

```html
<nav class="flex items-center gap-2 text-sm text-neutral-500 mb-8">
  <a href="/" class="hover:text-neutral-300">Docs</a>
  <svg class="w-3.5 h-3.5" .../>
  <span class="text-neutral-300">Introduction</span>
</nav>
<h1 class="text-4xl font-bold text-white tracking-tight mb-4">Introduction</h1>
<p class="text-lg text-neutral-400 leading-relaxed mb-12">
  CRONUS is a declarative language that compiles to full-stack web applications.
  One <code class="bg-[#111] text-[#CC0000] px-1.5 py-0.5 rounded text-xs font-['Geist_Mono',monospace]">.cronus</code>
  file produces an HTTP server, database, API, and server-rendered UI with 51 built-in section types.
</p>
```

Philosophy (3 numbered items):
1. **Declarative over imperative** — Describe what you want, not how to build it...
2. **Zero-config defaults** — Every section type has sensible defaults...
3. **Semantic composition** — Sections compose by meaning, not by pixels...

### 1.6 "Design Principles" 3-tile grid (the closest thing to a features section)

This is the grid where new "Dump Next.js" feature tile can plug in. Verbatim template structure:

```html
<section id="design-principles" class="mb-14">
  <h2 class="text-2xl font-bold text-white mb-6 flex items-center gap-2 group">Design Principles
    <a href="#design-principles" class="opacity-0 group-hover:opacity-100 text-[#CC0000] transition-opacity text-lg">#</a>
  </h2>
  <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
    <div class="bg-[#0A0A0A] border border-white/5 rounded-xl p-6 hover:border-[#CC0000]/20 transition-colors">
      <div class="text-[#CC0000] text-2xl mb-4">&#9889;</div>
      <h3 class="text-white font-semibold mb-2">Speed</h3>
      <p class="text-sm text-neutral-400">Rust-compiled binary starts in microseconds. Server-rendered HTML with zero JavaScript by default. No build step, no bundler.</p>
    </div>
    <div class="bg-[#0A0A0A] border border-white/5 rounded-xl p-6 hover:border-[#CC0000]/20 transition-colors">
      <div class="text-[#CC0000] text-2xl mb-4">&#9855;</div>
      <h3 class="text-white font-semibold mb-2">Accessibility</h3>
      <p class="text-sm text-neutral-400">Semantic HTML output. All interactive sections include ARIA attributes, keyboard navigation, and proper focus management.</p>
    </div>
    <div class="bg-[#0A0A0A] border border-white/5 rounded-xl p-6 hover:border-[#CC0000]/20 transition-colors">
      <div class="text-[#CC0000] text-2xl mb-4">&#10024;</div>
      <h3 class="text-white font-semibold mb-2">Elegance</h3>
      <p class="text-sm text-neutral-400">Typography-first design. Monochrome palette with a single accent. Every section type produces refined, production-ready HTML.</p>
    </div>
  </div>
</section>
```

Three tiles only: **Speed**, **Accessibility**, **Elegance**. These are design principles, not product features. **Next.js / VINEXT dumping is NOT mentioned anywhere on `/`.**

### 1.7 Other landing sections (condensed)

- `#philosophy` — 3 numbered principles (Declarative, Zero-config, Semantic composition).
- `#quick-start` — 12-line code sample inside a `<div class="bg-[#0D0D0D] border border-white/5 rounded-xl p-5">` terminal card with fake window dots, showing `app "My App" { ... } entity Task { ... } page "/" type:dashboard { section table { ... } }`.
- `#design-principles` — the 3-tile grid above.
- `#typography` — Geist Display / Body / Label / Mono rows.
- `#color-palette` — 8-color swatch grid (2 col sm, 4 col).
- `#section-types` — `<div class="bg-[#0D0D0D] border border-white/5 rounded-xl p-5">` terminal card listing 51 sections by category: Marketing, Navigation, Data, Forms, Cards, Feedback, Layout, Aliases. **This is where "dump" as a capability is implicitly hidden; it is NOT listed here, because this section is about section types not compiler features.**
- `#endpoints` — REST API table: `/api/health`, `/api/_health`, `/api/schema`, `/api/_seed`, `/api/_context`, `/api/docs/index`, `/api/docs/search?q=`, `/api/sse`, `/api/auth/signup`, `/api/auth/login`, `/api/auth/me`, `/api/{entity}s` GET/POST, `/api/{entity}s/:id` GET/PATCH/DELETE.
- `#dashboards` — runtime dashboards table: `/docs`, `/docs/design`, `/docs/graph`, `/blocks`, `/zeus`, `/trust`, `/hydra`.
- `#related` — 3 icon tiles: Your First App (`rocket_launch`), Entities (`database`), Components (`widgets`). Each tile uses pattern:
  ```html
  <a href="/xxx" class="flex items-center gap-3 p-3 rounded-lg border border-white/5 hover:border-[#CC0000]/20 transition-colors group">
    <span class="material-symbols-outlined text-neutral-600 group-hover:text-[#CC0000] transition-colors" style="font-size:20px">icon_name</span>
    <div>
      <div class="text-sm font-medium text-white">Title</div>
      <div class="text-xs text-neutral-500">Subtitle</div>
    </div>
  </a>
  ```
- Footer: `<a href="/components">Next → Components Overview</a>` only (no prev).
- Right aside "On This Page": `#philosophy`, `#quick-start`, `#design-principles`, `#typography`, `#color-palette`, `#section-types`, `#endpoints`, `#dashboards`, `#related`.

---

## 2. `/dump` page (lines 533–544)

### 2.1 Block structure

```
Line 533: page "/dump" type:custom {
Line 534: (blank)
Line 535:   section topbar {
Line 536:     brand "CRONUS"
Line 537:     nav "Guide, Components, API"
Line 538:     template "<header ... (identical to landing topbar)>"
Line 539:   }
Line 540: (blank)
Line 541:   section layout {
Line 542:     template "<div class=\"flex max-w-[1440px] ...\"> ... </div>"   # 25,462 chars
Line 543:   }
Line 544: }
```

Line 542 holds the entire page. Like the landing page, it includes its own full sidebar HTML (with the `/dump` item marked active: `class="... text-[#CC0000] font-medium ... bg-[#CC0000]/5 border-l-2 border-[#CC0000]"`).

### 2.2 Main content (quoted, elided where repetitive)

**Breadcrumb + H1 + lead:**

```html
<nav class="flex items-center gap-2 text-sm text-neutral-500 mb-8">
  <a href="/" class="hover:text-neutral-300">Docs</a>
  <svg .../>
  <span class="text-neutral-300">HTML Dump</span>
</nav>
<h1 class="text-4xl font-bold text-white tracking-tight mb-4">HTML Dump</h1>
<p class="text-lg text-neutral-400 leading-relaxed mb-12">
  Reverse-engineer any HTML page into clean CRONUS source code with automatic
  section detection, template extraction, and fidelity auditing.
</p>
```

**Sections in order:**

1. `#overview`
   > The dump command reverse-engineers any HTML page into CRONUS source code. Feed it a premium landing page, dashboard, or marketing site, and it produces a semantically equivalent `.cronus` file with proper section types, templates, and data bindings.

2. `#pipeline` — "10-Step Pipeline" table (`<table class="w-full text-sm">`):
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

3. `#section-detection` — "Section Detection" table of auto-detected types and confidence:
   - `hero` — h1 + subtitle + CTA button in first viewport — &gt; 95%
   - `features` — Grid of cards with icons/titles/descriptions — &gt; 90%
   - `pricing` — Cards with price, period, feature list — &gt; 92%
   - `testimonial` — Blockquotes with avatars and names — &gt; 88%
   - `table` — thead + tbody with structured data — &gt; 97%
   - `chart` — canvas or SVG with data visualization — &gt; 85%
   - `faq` — Accordion or dt/dd pairs — &gt; 90%
   - `footer` — Bottom section with links and copyright — &gt; 95%

   Followed by a `<div class="bg-[#CC0000]/5 border border-[#CC0000]/20 rounded-xl p-4 mb-6">` callout: "Sections with confidence below 70% are emitted as `type:custom` with the raw HTML preserved in a template block."

4. `#template-extraction` — Before/after code card (terminal-style `<div class="bg-[#0D0D0D] border border-white/5 rounded-xl p-5 mb-6">` with dots + `template_vars` label). Before: raw `<h1>Welcome to Acme</h1>...`. After: `<h1>{{title}}</h1><p>{{subtitle}}</p><a href="{{cta.url}}">{{cta.text}}</a>`.

5. `#unicorn-studio` — "WebGL & Unicorn Studio"
   > The dump engine detects WebGL-based animations from Unicorn Studio and other canvas-based libraries. These are preserved as `template` blocks with the original embed code.

6. `#chartjs-detection` — "Chart.js Detection"
   > Canvas elements with Chart.js data attributes are converted into native `section chart` blocks with the appropriate chart type, labels, and datasets extracted from the JavaScript initialization code.

7. `#cli-usage` — Terminal code card:
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

8. `#export-formats` — Table of other export commands:
   - `cronus export` → `cronus-project.ir.json` — Full IR as JSON
   - `cronus dump page.html` → `.cronus source` — Reverse-engineer HTML
   - `cronus spec codegen --docs` → Markdown
   - `cronus spec codegen --ai-protocol` → JSON schema
   - `cronus spec codegen --structs` → Rust source (`contracts_generated.rs`)

   Followed by a second terminal card with example commands.

9. `#related` — 3 icon tiles: Audit System (`fact_check`), Compiler (`build`), Templates (`code`).

10. Footer: `<a href="/compose">Prev Block Composer</a>` and `<a href="/audit">Next Audit System</a>`.

**Right "On This Page" aside** lists: Overview, Pipeline, Section Detection, Template Extraction, WebGL, Chart.js, CLI Usage, Export Formats, Related Topics.

### 2.3 What is MISSING from /dump (relevant to Next.js/VINEXT)

- **No Next.js mention** anywhere on the page.
- **No VINEXT mention.**
- **No framework-specific dumping** — the entire page assumes a single raw HTML file or URL.
- **No `cronus dump` flags for framework detection** (only `--audit`, `--strict`).
- **No mention of JSX/TSX, React components, Next.js App Router, pages/, server components, or `.next/` build output.**
- **Pipeline step 1 says "Parse DOM"** — not "Parse JSX" or "Parse React tree."
- **Section detection table lists only HTML-element signals**, not React component signals.

---

## 3. Navigation + sidebar structure

### 3.1 Top nav

Only 3 links site-wide, hard-coded into every `section topbar` template: **Guide** (`/`), **Components** (`/components`), **API** (`/api-routes`). No dropdown, no "Tools" menu, no direct `/dump` link.

### 3.2 Sidebar

There is **no `layout Main { sidebar { ... } }` block** in the file. The sidebar is an inlined HTML `<aside class="hidden lg:block w-64 shrink-0 border-r border-white/5 bg-[#0A0A0A] sticky top-14 ...">` that appears literally at the start of every page's `section layout` template string.

`/dump` IS already in the sidebar, under the **Advanced** group, between `/compose` (Block Composer) and `/audit` (Audit System). Label: "HTML Dump".

Because the sidebar is duplicated in every page template (~40 times), adding a new sidebar item or renaming the existing `/dump` label must be done as a global find-and-replace across all ~40 templates. There is no single source of truth to edit.

---

## 4. Proposed edit zones

### Edit A — New feature tile on landing "Design Principles" grid

**Location:** inside line 36, inside the `<section id="design-principles">` block, inside its `<div class="grid grid-cols-1 md:grid-cols-3 gap-4">`.

**Problem:** The grid is `md:grid-cols-3` with exactly 3 tiles (Speed, Accessibility, Elegance). Adding a 4th tile as-is creates a 3+1 layout on md screens. Two options:
- Change `md:grid-cols-3` → `md:grid-cols-2 lg:grid-cols-4` (clean 1×4 on lg, 2×2 on md).
- Keep 3 tiles and replace one of them. NOT recommended — these are core design principles.

**Alternative:** Insert a brand-new section before `#section-types` or after `#quick-start`, e.g. `<section id="dump-features">` with its own `grid grid-cols-1 md:grid-cols-3 gap-4` — cleaner.

**Recommended edit zone:** Line 36, character range approximately 13600–13660 (between the closing `</div></section>` of `#typography`... actually safer: insert a new section **between `#quick-start` and `#design-principles`** or **between `#section-types` and `#endpoints`**. The latter makes semantic sense: after listing 51 section types, introduce "How to get CRONUS source: write it or dump it."

**Surrounding context (anchor before):**
The `#section-types` block ends with `...stats</code></div></section>` (found at ~char 17050). Immediately after that the `#endpoints` section begins: `<section id="endpoints" class="mb-14">...`.

**Insertion anchor string (unique in file, safe for Edit tool):**
```
</code></div></section><section id="endpoints" class="mb-14">
```

**Proposed insertion** (inserts between the two):
```html
</code></div></section>
<section id="dumping" class="mb-14">
  <h2 class="text-2xl font-bold text-white mb-4 flex items-center gap-2 group">Dump to .cronus
    <a href="#dumping" class="opacity-0 group-hover:opacity-100 text-[#CC0000] transition-opacity text-lg">#</a>
  </h2>
  <p class="text-neutral-400 leading-relaxed mb-6">Reverse-engineer existing projects into CRONUS source. Works with raw HTML, Next.js apps, and React component trees.</p>
  <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
    <a href="/dump" class="bg-[#0A0A0A] border border-white/5 rounded-xl p-6 hover:border-[#CC0000]/20 transition-colors block">
      <div class="text-[#CC0000] text-2xl mb-4">&#60;/&#62;</div>
      <h3 class="text-white font-semibold mb-2">HTML → .cronus</h3>
      <p class="text-sm text-neutral-400">Feed the dump engine a premium landing page or marketing site. 10-step pipeline auto-detects 20+ section types.</p>
    </a>
    <a href="/dump#nextjs" class="bg-[#0A0A0A] border border-white/5 rounded-xl p-6 hover:border-[#CC0000]/20 transition-colors block">
      <div class="text-[#CC0000] text-2xl mb-4">&#9650;</div>
      <h3 class="text-white font-semibold mb-2">Next.js → .cronus</h3>
      <p class="text-sm text-neutral-400">Point VINEXT at an <code class="bg-[#111] text-[#CC0000] px-1 py-0.5 rounded text-[10px] font-['Geist_Mono',monospace]">app/</code> directory to convert server components and routes into semantic sections.</p>
    </a>
    <a href="/audit" class="bg-[#0A0A0A] border border-white/5 rounded-xl p-6 hover:border-[#CC0000]/20 transition-colors block">
      <div class="text-[#CC0000] text-2xl mb-4">&#10003;</div>
      <h3 class="text-white font-semibold mb-2">Fidelity Audit</h3>
      <p class="text-sm text-neutral-400">Every dump runs a pixel-and-DOM diff against the reference to report what was lost in translation.</p>
    </a>
  </div>
</section>
<section id="endpoints" class="mb-14">
```

**CSS safety:** Uses the exact same tile classes as `#design-principles` (`bg-[#0A0A0A] border border-white/5 rounded-xl p-6 hover:border-[#CC0000]/20 transition-colors`), same `grid grid-cols-1 md:grid-cols-3 gap-4`, same H2/H3 typography. Respects the `mb-14` section rhythm. Will not break the flex/grid tree because it's a sibling `<section>` of the same depth.

**Also update** the right "On This Page" aside (same line 36, near the end ~char 26600). Anchor string:
```
<li><a href="#section-types" class="text-xs text-neutral-500 hover:text-white transition-colors">51 Section Types</a></li>
```
Insert after:
```html
<li><a href="#dumping" class="text-xs text-neutral-500 hover:text-white transition-colors">Dump to .cronus</a></li>
```

---

### Edit B — Next.js/VINEXT content on `/dump` (line 542)

**Location:** inside line 542, inside the `<main>` content area, adding a new `<section>` block that covers framework-specific dumping.

**Best anchor:** between `#section-detection` and `#template-extraction`, OR between `#chartjs-detection` and `#cli-usage`. The second is safer — it groups "special integrations" (Unicorn Studio WebGL, Chart.js, **Next.js/VINEXT**) together before the CLI usage reference.

**Anchor string (unique, safe for Edit tool):**
```
</p></section><section id="cli-usage" class="mb-14">
```

Wait — that may appear in multiple templates. Safer anchor specific to `/dump` line 542:
```
section chart</code> blocks with the appropriate chart type, labels, and datasets extracted from the JavaScript initialization code.</p></section><section id="cli-usage"
```

**Proposed insertion:**
```html
...JavaScript initialization code.</p></section>
<section id="nextjs" class="mb-14">
  <h2 class="text-2xl font-bold text-white mb-4 flex items-center gap-2 group">Next.js &amp; VINEXT
    <a href="#nextjs" class="opacity-0 group-hover:opacity-100 text-[#CC0000] transition-opacity text-lg">#</a>
  </h2>
  <p class="text-neutral-400 leading-relaxed mb-6">VINEXT is the Next.js-aware mode of the dump engine. Point it at an <code class="bg-[#111] text-[#CC0000] px-1.5 py-0.5 rounded text-xs font-['Geist_Mono',monospace]">app/</code> directory and it walks the App Router, resolves server/client components, and emits one <code class="bg-[#111] text-[#CC0000] px-1.5 py-0.5 rounded text-xs font-['Geist_Mono',monospace]">.cronus</code> file per route.</p>
  <div class="overflow-x-auto mb-6">
    <table class="w-full text-sm">
      <thead>
        <tr class="border-b border-white/5">
          <th class="text-left py-3 px-4 text-neutral-400 font-medium">Next.js Concept</th>
          <th class="text-left py-3 px-4 text-neutral-400 font-medium">CRONUS Output</th>
        </tr>
      </thead>
      <tbody class="text-neutral-300">
        <tr class="border-b border-white/5"><td class="py-2 px-4 font-mono text-xs text-[#CC0000]">app/page.tsx</td><td class="py-2 px-4 text-neutral-400 text-xs">page "/" type:custom { sections... }</td></tr>
        <tr class="border-b border-white/5"><td class="py-2 px-4 font-mono text-xs text-[#CC0000]">app/dashboard/page.tsx</td><td class="py-2 px-4 text-neutral-400 text-xs">page "/dashboard" type:dashboard</td></tr>
        <tr class="border-b border-white/5"><td class="py-2 px-4 font-mono text-xs text-[#CC0000]">Server Component</td><td class="py-2 px-4 text-neutral-400 text-xs">Rendered to HTML, templatized</td></tr>
        <tr class="border-b border-white/5"><td class="py-2 px-4 font-mono text-xs text-[#CC0000]">&lt;Hero /&gt; component</td><td class="py-2 px-4 text-neutral-400 text-xs">section hero with extracted props</td></tr>
        <tr class="border-b border-white/5"><td class="py-2 px-4 font-mono text-xs text-[#CC0000]">fetch() in RSC</td><td class="py-2 px-4 text-neutral-400 text-xs">bind entity or api_route</td></tr>
      </tbody>
    </table>
  </div>
  <div class="bg-[#0D0D0D] border border-white/5 rounded-xl p-5 mb-6">
    <div class="flex items-center justify-between mb-3"><div class="flex gap-1.5"><div class="w-2.5 h-2.5 rounded-full bg-white/10"></div><div class="w-2.5 h-2.5 rounded-full bg-white/10"></div><div class="w-2.5 h-2.5 rounded-full bg-white/10"></div></div><span class="text-[10px] font-mono text-neutral-600">terminal</span></div>
    <code class="block font-['Geist_Mono',monospace] text-sm text-neutral-300 whitespace-pre leading-relaxed"><span class="text-neutral-600"># Dump a Next.js project</span><br>cronus dump ./my-next-app --next -o out/<br><br><span class="text-neutral-600"># Dump a single route</span><br>cronus dump ./my-next-app/app/dashboard --next -o dashboard.cronus<br><br><span class="text-neutral-600"># With audit against the live site</span><br>cronus dump ./my-next-app --next --audit https://my-site.com</code>
  </div>
  <div class="bg-[#CC0000]/5 border border-[#CC0000]/20 rounded-xl p-4 mb-6">
    <p class="text-sm text-neutral-300"><strong class="text-[#CC0000]">Note:</strong> VINEXT requires the Next.js project to build successfully. It reads the compiled RSC payloads from <code class="bg-[#111] text-[#CC0000] px-1.5 py-0.5 rounded text-xs font-['Geist_Mono',monospace]">.next/</code>, not the raw TSX source, so dynamic data is resolved at dump time.</p>
  </div>
</section>
<section id="cli-usage" class="mb-14">
```

**Also update** the right aside in line 542 (same template) to include `#nextjs`. Anchor:
```
<li><a href="#chartjs-detection" class="text-xs text-neutral-500 hover:text-white transition-colors">Chart.js</a></li>
```
Insert after:
```html
<li><a href="#nextjs" class="text-xs text-neutral-500 hover:text-white transition-colors">Next.js &amp; VINEXT</a></li>
```

**CSS safety:** Matches exactly the table/terminal-card/callout patterns used in the rest of `/dump`. `<section class="mb-14">` is a sibling of `#template-extraction`, `#unicorn-studio`, `#chartjs-detection`, `#cli-usage` — all at the same depth in `#doc-content`. Will not break layout.

---

### Edit C — Sidebar prominence (optional)

`/dump` already has a sidebar entry under **Advanced** labeled "HTML Dump". Since VINEXT makes dumping broader than HTML-only, consider renaming the label to **"Dump / VINEXT"** or **"Project Dump"**.

**Problem:** The sidebar is inlined in **every page's** `section layout` template (~40 copies). A rename requires global find-and-replace of:
```
<a href="/dump" class="block text-sm text-neutral-400 hover:text-white py-1 px-2 transition-colors">HTML Dump</a>
```
And the active variant in `/dump` itself (line 542):
```
<a href="/dump" class="block text-sm text-[#CC0000] font-medium py-1 px-2 rounded bg-[#CC0000]/5 border-l-2 border-[#CC0000]">HTML Dump</a>
```

Use the `replace_all` flag on the Edit tool for the inactive variant. Do a separate targeted edit for the active variant.

**Alternative — add topbar link.** Add a 4th top-level nav item "Dump" pointing to `/dump`. This also requires editing **every page's `section topbar` template**. Anchor:
```
<a href="/api-routes" class="text-sm text-neutral-500 hover:text-neutral-300 transition-colors">API</a></nav>
```
Insert after (and before `</nav>`):
```html
<a href="/dump" class="text-sm text-neutral-500 hover:text-neutral-300 transition-colors">Dump</a>
```
Again ~40 replacements; use `replace_all`. **Not recommended** unless dumping is a headline feature — it would clutter the minimal 3-item nav.

---

## 5. Style guide — what new content must match

### 5.1 Tokens (from `style { }` block lines 12–18 and `tailwind_config` line 20)

| Token | Value |
|---|---|
| Theme | `dark` |
| Accent | `#CC0000` (Tailwind: `accent`) |
| Accent dim | `#991111` |
| Background root | `#020202` (`bg-root`) |
| Sidebar bg | `#0A0A0A` (`bg-sidebar`) |
| Code block bg | `#0D0D0D` (`bg-code`) |
| Card bg | `#111111` (`bg-card`) |
| Border subtle | `rgba(255,255,255,0.06)` |
| Text muted | `#888888` |
| Text body | `#A3A3A3` |
| Font sans | `Geist`, `system-ui`, `sans-serif` |
| Font mono | `Geist Mono`, `monospace` |
| Radius default | `md` |

### 5.2 Typography classes

- **H1:** `text-4xl font-bold text-white tracking-tight mb-4`
- **Page lead:** `text-lg text-neutral-400 leading-relaxed mb-12`
- **H2:** `text-2xl font-bold text-white mb-4 flex items-center gap-2 group` with a hover anchor `<a href="#slug" class="opacity-0 group-hover:opacity-100 text-[#CC0000] transition-opacity text-lg">#</a>`
- **H3 (card title):** `text-white font-semibold mb-2`
- **Body:** `text-neutral-400 leading-relaxed mb-6`
- **Small body (card):** `text-sm text-neutral-400`
- **Inline code:** `bg-[#111] text-[#CC0000] px-1.5 py-0.5 rounded text-xs font-['Geist_Mono',monospace]`
- **Monospace font enforcement:** `font-['Geist_Mono',monospace]` (note underscore because of quote escaping).
- **Root font for page:** `font-['Geist',system-ui,sans-serif]` on outer `<div>`.

### 5.3 Structural primitives

**Section wrapper:**
```html
<section id="slug" class="mb-14">
  <h2 class="text-2xl font-bold text-white mb-4 flex items-center gap-2 group">Title<a href="#slug" class="opacity-0 group-hover:opacity-100 text-[#CC0000] transition-opacity text-lg">#</a></h2>
  <p class="text-neutral-400 leading-relaxed mb-6">Intro paragraph.</p>
  ...
</section>
```

**3-tile card grid:**
```html
<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
  <div class="bg-[#0A0A0A] border border-white/5 rounded-xl p-6 hover:border-[#CC0000]/20 transition-colors">
    <div class="text-[#CC0000] text-2xl mb-4">&#9889;</div>
    <h3 class="text-white font-semibold mb-2">Title</h3>
    <p class="text-sm text-neutral-400">Body.</p>
  </div>
  ...
</div>
```

**Terminal / code card:**
```html
<div class="bg-[#0D0D0D] border border-white/5 rounded-xl p-5 mb-6">
  <div class="flex items-center justify-between mb-3">
    <div class="flex gap-1.5">
      <div class="w-2.5 h-2.5 rounded-full bg-white/10"></div>
      <div class="w-2.5 h-2.5 rounded-full bg-white/10"></div>
      <div class="w-2.5 h-2.5 rounded-full bg-white/10"></div>
    </div>
    <span class="text-[10px] font-mono text-neutral-600">label</span>
  </div>
  <code class="block font-['Geist_Mono',monospace] text-sm text-neutral-300 whitespace-pre leading-relaxed">
    code with <br> for newlines and <span class="text-[#CC0000]">keyword</span> spans
  </code>
</div>
```

**Table:**
```html
<div class="overflow-x-auto mb-6">
  <table class="w-full text-sm">
    <thead>
      <tr class="border-b border-white/5">
        <th class="text-left py-3 px-4 text-neutral-400 font-medium">Col</th>
      </tr>
    </thead>
    <tbody class="text-neutral-300">
      <tr class="border-b border-white/5">
        <td class="py-2 px-4 font-mono text-xs text-[#CC0000]">mono</td>
        <td class="py-2 px-4 text-neutral-400 text-xs">body</td>
      </tr>
    </tbody>
  </table>
</div>
```

**Callout (accent):**
```html
<div class="bg-[#CC0000]/5 border border-[#CC0000]/20 rounded-xl p-4 mb-6">
  <p class="text-sm text-neutral-300"><strong class="text-[#CC0000]">Note:</strong> Body.</p>
</div>
```

**Related-topics icon tile:**
```html
<a href="/xxx" class="flex items-center gap-3 p-3 rounded-lg border border-white/5 hover:border-[#CC0000]/20 transition-colors group">
  <span class="material-symbols-outlined text-neutral-600 group-hover:text-[#CC0000] transition-colors" style="font-size:20px">icon_name</span>
  <div>
    <div class="text-sm font-medium text-white">Title</div>
    <div class="text-xs text-neutral-500">Subtitle</div>
  </div>
</a>
```

### 5.4 Hard rules

- **One accent only.** `#CC0000` for highlights, keyword spans, active state, hover borders (`/20` opacity), callouts (`/5` bg, `/20` border). Never introduce a second color.
- **Monochrome greys** for text: `text-white` (h1/h2/h3/strong), `text-neutral-300` (body-strong in dark cards), `text-neutral-400` (body), `text-neutral-500` (muted), `text-neutral-600` (ultra-muted labels).
- **Borders are always** `border-white/5` (subtle) or `border-[#CC0000]/20` (hover/callout). Never solid grey or colored borders.
- **Card radius:** `rounded-xl` for content cards, `rounded-lg` for small buttons/tiles, `rounded` for inline-code pill.
- **Spacing rhythm:** sections use `mb-14`, block elements inside `mb-6`, headings `mb-4` or `mb-2`.
- **Font:** everything uses the root `font-['Geist',system-ui,sans-serif]` declared on the outer `<div>`; code uses `font-['Geist_Mono',monospace]` (underscore, not space, because it's inside an escaped attribute).
- **No emojis in content.** Use HTML entities like `&#9889;` (lightning), `&#9855;` (accessibility), `&#10024;` (sparkles), `&#10003;` (check), or `<svg>` icons. Also uses Google `material-symbols-outlined` for related-topic tiles.
- **Quote escaping:** every `"` in the embedded HTML is written as `\"` because the template is a single CRONUS string literal. Every new HTML snippet must be inserted with all quotes escaped.
- **No line breaks inside the template string.** The whole `template "..."` is one physical line. New content must be flattened to a single line before insertion (the report samples above are pretty-printed for readability — flatten them).

---

## 6. Summary of exact insertion points

| Edit | File line | Approx. char offset in that line | Anchor string (unique) |
|---|---|---|---|
| A1 — feature section on `/` | 36 | ~17050 | `...stats</code></div></section><section id="endpoints"` |
| A2 — `/` right aside ToC entry | 36 | ~26700 | `>51 Section Types</a></li>` |
| B1 — Next.js section on `/dump` | 542 | ~16300 | `...initialization code.</p></section><section id="cli-usage"` |
| B2 — `/dump` right aside ToC entry | 542 | ~24700 | `>Chart.js</a></li>` |
| C1 — sidebar rename (optional, global) | 36, 57, 78, 99, 120, 141, 162, 183, 204, 225, 246, 267, 288, 309, 330, 351, 372, 393, 411, 432, 453, 474, 491, 508, 525, 542, 559, 576, 593, 610, 627, 646, 663 | each ~5200 | `hover:text-white py-1 px-2 transition-colors\">HTML Dump</a>` — use `replace_all` |

All insertion points land at closing-tag boundaries in the HTML tree, are siblings of existing same-depth siblings, and do not require changing any parent grid/flex containers. Inserting at A and B is fully additive.

---

## 7. Caveats

1. The file is **1.2 MB**, not ~20 KB as the task brief stated. This is because every page inlines the entire sidebar + topbar HTML rather than referencing a shared layout block.
2. There is **no `layout Main { sidebar { ... } }`** block at all in this file. If the CRONUS DSL supports it, a refactor to pull topbar/sidebar into shared blocks would prevent the need for global find-and-replace — but that is out of scope for this mapping pass.
3. Because each page template is a single long string, **never use multi-line insertions** — flatten your HTML to one line before editing.
4. When using the `Edit` tool on line 36 or line 542, anchor strings must be long enough to be unique **within the single line**. Prefer 60+ characters of context.
5. Line numbers reference `/home/zedd/Documentos/CRONUS/docs/site-v2/app.cronus` as it stood on 2026-04-10. If the file is edited in between, re-run `awk '{print NR": "length($0)}'` to find the new long-line offsets before editing.
