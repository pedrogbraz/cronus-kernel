# R1 — Next.js / VINEXT Dumper Spec (Factual)

> Source of truth: `src/dump/nextjs.rs` (1366 lines), `src/dump/detect.rs`,
> `src/cli/dump_cmd.rs`, `src/dump/mod.rs`.
> Purpose: honest input for `docs.cronus.test` — no overpromising.
> Date: 2026-04-10.

---

## 1. Command Syntax

The Next.js dumper is one mode of the generic `cronus dump` command. There is
no dedicated `cronus dump-nextjs` subcommand — the router is in
`src/cli/dump_cmd.rs`.

Usage (from `dump_cmd.rs:11`):

```
cronus dump <file.html|.json|.prisma|dir/> [-o output.cronus] [--audit] [--nextjs]
```

Flags relevant to Next.js mode:

| Flag          | Effect                                                                 |
|---------------|------------------------------------------------------------------------|
| `<dir>`       | A directory argument triggers "Black Hole" mode (project dump).        |
| `--nextjs`    | Force Next.js mode even if autodetection would not fire.               |
| `-o <file>`   | Write output to `<file>`. Default: `<dirname>.cronus` in cwd.          |
| `--audit`     | Post-dump fidelity audit. **Only works for `.html` inputs**, not dirs. |

If no `-o` is given for a directory dump, the file is written next to the cwd
as `{dir.file_name()}.cronus` (`dump_cmd.rs:42-44`).

The `--audit` flag is explicitly gated to HTML input (`dump_cmd.rs:90,117-118`);
running it against a Next.js project directory prints:
`--audit only works with HTML input files`.

---

## 2. Detection Table (framework → signal)

Autodetection lives entirely in `dump_cmd.rs:18-28`. `detect.rs` is for HTML
section detection and contains no framework-detection helper used by the
Next.js dumper.

| Signal                                        | Triggers Next.js mode? |
|-----------------------------------------------|------------------------|
| `--nextjs` flag                               | Yes (forced)           |
| `<dir>/next.config.ts` exists                 | Yes                    |
| `<dir>/next.config.mjs` exists                | Yes                    |
| `<dir>/next.config.js` exists                 | Yes                    |
| `package.json` contains `"next"`              | Yes                    |
| `package.json` contains `"vinext"`            | Yes                    |
| None of the above                             | Falls through to `dump::project::dump_project` |

Code (`dump_cmd.rs:19-28`):

```rust
let is_nextjs = nextjs_mode
    || path.join("next.config.ts").exists()
    || path.join("next.config.mjs").exists()
    || path.join("next.config.js").exists()
    || {
        let pkg = path.join("package.json");
        pkg.exists() && fs::read_to_string(&pkg)
            .map(|c| c.contains("\"next\"") || c.contains("\"vinext\""))
            .unwrap_or(false)
    };
```

Ambiguity handling: there is none. Autodetection is a simple OR of file
existence and substring checks in `package.json`. A project with both Next.js
and other frameworks will be dumped as Next.js. A project with `"next"` as a
substring of another dep name (e.g. `"next-whatever"`) will also match — no
JSON parsing, just `String::contains`.

VINEXT vs Next.js: the same code path handles both. The only place VINEXT is
recognized is the `package.json` substring check. Once detected, the routing
scanner treats VINEXT exactly as Next.js — same file conventions, same router
layout. There is no fork-specific branch in `nextjs.rs`.

---

## 3. Extraction Table (source → .cronus output)

All row numbers reference `nextjs.rs`.

### 3.1 Router type

| Source                          | What is extracted                                         | Code |
|---------------------------------|-----------------------------------------------------------|------|
| `app/` or `src/app/`            | `has_app_router = true`                                   | 268-282 |
| `pages/` or `src/pages/`        | `has_pages_router = true`                                 | 268-282 |
| Both                            | Hybrid: both scanners run and pages are concatenated      | 161-166 |

### 3.2 App Router pages

File conventions recognized (`is_page_file`, line 341-343):
`page.tsx`, `page.ts`, `page.jsx`, `page.js`.

Directories skipped (`scan_app_dir_recursive`, line 306):
- `_*` (private folders, e.g. `_components`)
- `node_modules`, `.next`

Segment conversion (`file_to_app_route`, line 349-392):

| Next.js segment   | .cronus route         | Notes |
|-------------------|-----------------------|-------|
| `foo`             | `/foo`                | Static |
| `[slug]`          | `/:slug`              | Dynamic |
| `[...slug]`       | `/:slug`              | Catch-all — collapsed, no marker preserved |
| `[[...slug]]`     | `/:slug`              | Optional catch-all — collapsed, no marker preserved |
| `(marketing)`     | *(removed)*           | Route group — transparent |
| `@modal`          | *(removed)*           | Parallel slot — stripped |

Per-page fields extracted (`scan_app_dir_recursive`, line 311-337):
- `has_loading` — `true` if sibling `loading.tsx|js` exists.
- `has_error` — `true` if sibling `error.tsx|js` exists.
- `requires_auth` — substring match for `auth`, `session`, or
  `getServerSession` in file content.
- `data_fetching` — one of `server-action` / `fetch` / `database` / `None`
  (line 1045-1055). Detection is substring-based (`"use server"`, `fetch(`,
  `prisma`, `drizzle`, `db.`).
- `page_type` — inferred from route + content (line 1099-1117): `dashboard` /
  `detail` (if route has `:`) / `form` (has `<form` or `onSubmit`) / `list`
  (has `<table` or `DataTable`) / `custom`.

### 3.3 Pages Router pages

Extensions: `.tsx`, `.ts`, `.jsx`, `.js` (line 449-452).

Skipped: files starting with `_` (e.g. `_app.tsx`, `_document.tsx`), dirs
`api`, `node_modules`, `_app`, `_document` (line 415-417).

Route conversion (`file_to_pages_route`, line 454-497):
- `index.tsx` → `/`
- `foo/index.tsx` → `/foo`
- `[slug].tsx` → `/:slug`
- `[...slug].tsx` → `/:slug`

Data-fetching detection for Pages Router (`detect_pages_data_fetching`,
1057-1065): `getServerSideProps` or `getStaticProps`.

### 3.4 API routes

**App Router API** (`scan_app_api_routes`, line 503-546):
- Scans `app/api/**/route.{tsx,ts,js}` (no `.jsx` — `is_route_file` line 345-347).
- Route groups `(group)` are traversed transparently without adding a segment.
- Dynamic segments `[id]` become `:id`; `[...rest]` also becomes `:rest`
  (`...` is stripped).
- Methods: filtered by `detect_exported_methods` (see §4).

**Pages Router API** (`scan_pages_api_routes`, line 548-596):
- Every file under `pages/api/**` is treated as a catch-all handler.
- Emitted methods: **hardcoded to `[GET, POST, PUT, PATCH, DELETE]`** for
  every Pages Router API file (line 586-589). There is no AST inspection —
  Pages Router APIs are `export default handler` so the dumper cannot tell
  which methods the handler actually branches on.

Per-route fields: `has_auth` is a substring check for `auth`, `token`,
`session`, or (App Router only) `getServerSession`.

### 3.5 Layouts

`scan_layouts` (line 622-667) recurses the app dir looking for
`layout.{tsx,ts,jsx,js}`. For each layout file:

- `has_sidebar` — content contains `sidebar`, `Sidebar`, `aside`, or `nav-sidebar`.
- `has_topbar` — contains `header`, `Header`, `navbar`, `Navbar`, `topbar`.
- `brand` — substring near `href="/"` (line 720-737).
- `nav_items` — extracted from `href="/path"` / `href='/path'` /
  `href={`/path`}` patterns (line 679-718). Filters: href must start with `/`,
  be < 80 chars, and not contain `api/`. Deduplicated by href.

At emit time, **only the first layout with both `has_sidebar` and at least one
nav item is written** (line 1187-1207, `break` on line 1205). Layouts without
sidebars, or nested layouts, are discarded.

### 3.6 Middleware

`detect_middleware` (line 743-794) checks for:
- `middleware.ts|js` (project root)
- `src/middleware.ts|js`

Extracted:
- `has_auth_check` — substring match for `auth`, `token`, `session`,
  `getToken`, or `NextAuth`.
- `has_redirect` — substring match for `redirect` or `NextResponse.redirect`.
- `matchers` — parsed from the first `matcher` occurrence by walking a 500-char
  window and collecting quoted strings that start with `/`. Naive — no JSON or
  AST parsing. If no matchers are found, defaults to `["/"]`.

Middleware doesn't get its own block in the output. Instead, if
`has_auth_check` is true, the scanner back-propagates `requires_auth = true`
onto any page whose route starts with a matcher (line 205-214).

### 3.7 Auth

`detect_auth` (line 861-899) inspects `package.json` as a string, in order:

| Substring in `package.json` | Provider emitted |
|-----------------------------|------------------|
| `next-auth` or `@auth/`     | `next-auth`      |
| `@clerk`                    | `clerk`          |
| `lucia`                     | `lucia`          |
| `supabase` AND `auth`       | `supabase`       |
| `firebase`                  | `firebase`       |
| else, if any of `lib/auth.ts`, `src/lib/auth.ts`, `utils/auth.ts`, `app/api/auth` exists | `custom` |
| else                        | none, `auth` block omitted |

When auth is detected, roles are **always hardcoded to `[admin, user]`** (line
893-897). There is no role extraction from code.

The emitted `auth` block is also hardcoded (line 1154-1166):

```cronus
auth {
  entity User
  login email + password
  session jwt expires:24h
  roles [admin, user]
}
```

This is emitted regardless of the actual provider or login method. If the
source project uses OAuth, magic links, or Clerk's hosted UI, the emitted
block still says `email + password`.

### 3.8 Prisma → Entities

`detect_prisma` (line 990-1005) reads `prisma/schema.prisma` or
`./schema.prisma`, delegates to `super::prisma::dump_prisma` (not in scope for
this pass), and then strips the `app { ... }` block from the result via
`remove_app_block` (line 1007-1039) so only the entities survive. The entity
block is emitted under the header `# ── Entities (from Prisma schema) ──`.

Field-type mapping lives in `src/dump/prisma.rs` which was **not read** per the
scope restriction — so this spec does not document which Prisma types map to
which .cronus field types.

### 3.9 Tailwind / style

`detect_style` (line 905-984):

- `has_tailwind`: true if any of `tailwind.config.{ts,js,mjs}` exist, OR if
  `app/globals.css` contains `@tailwind` / `@import "tailwindcss"`.
- `theme`: defaults to `light`. Switches to `dark` if `app/globals.css` (or
  `src/app/globals.css` or `styles/globals.css`) contains both `dark` and
  `:root` on any line. This is a very naive check.
- `accent`: first line in globals.css containing `primary` or `accent` and a
  `:`, value after the colon (stripped). Returned raw — no hex→name mapping.
- `font`: first line with `font-family` or `--font`, first comma-separated
  value, stripped of quotes. Filters `sans-serif`, `inherit`, `var`.

The `tailwind.config.*` file itself is **not parsed** for the Next.js dumper
(that parser exists for HTML dumps in `mod.rs`, not here).

### 3.10 Port and name

- `detect_name` (line 235-248): reads `package.json` `name` field (real JSON
  parse). Falls back to directory basename.
- `detect_port` (line 250-266): string scan for `--port` in `package.json`,
  parses the following digits. Defaults to `3000`.

### 3.11 next.config.*

`detect_next_config` (line 800-820) extracts:
- `basePath` — first string value via substring scan.
- `trailingSlash` — substring `trailingSlash: true` or `trailingSlash:true`.
- `i18n` — if the file contains `i18n`, emits a single-locale config using
  `defaultLocale` (or `"en"`). Locales array is **not parsed**; it's set to a
  one-element array with just the default (line 851-854).
- `redirects` and `images_domains` — always empty. Comment on line 817:
  `// Complex to parse, handled at runtime`.

At emit time, only `basePath` is written to the `app` block (line 1148-1150).
The rest of `NextConfig` is collected but never emitted.

---

## 4. Supported HTTP Methods (post-2026-04-10 fix)

`detect_exported_methods` (line 598-616):

```rust
for method in &["GET", "POST", "PUT", "PATCH", "DELETE"] {
    if content.contains(&format!("export async function {}", method))
        || content.contains(&format!("export function {}", method))
        || content.contains(&format!("export const {} =", method))
        || content.contains(&format!("export const {}", method))
    {
        methods.push(method.to_string());
    }
}
```

**Exactly 5 methods are recognized**: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`.

`HEAD` and `OPTIONS` are deliberately excluded. The reasoning is pinned in a
comment at line 599-604:

> The .cronus language only supports GET/POST/PUT/PATCH/DELETE.
> HEAD is auto-handled by the runtime (returns GET headers). OPTIONS is
> auto-handled as CORS preflight. Emitting them here would produce .cronus
> files that fail to parse (tokenizer rejects them as non-Method identifiers).
> So we intentionally ignore HEAD/OPTIONS exports in the scanned Next.js code.

Two tests pin this behavior (line 1336-1365):
- `detect_exported_methods_skips_head_and_options` — verifies HEAD/OPTIONS are
  filtered even when exported.
- `detect_exported_methods_returns_empty_for_non_route` — verifies a page
  component returns an empty method list.

Caveat: **Pages Router API routes bypass this filter entirely** (§3.4). Any
file under `pages/api/**` gets `[GET, POST, PUT, PATCH, DELETE]` hardcoded
(line 586-589), regardless of what the handler actually does.

---

## 5. Output Format

Emission is done by `emit_cronus` (line 1123-1320). Order:

1. **Header comments**
   ```
   # Generated by cronus dump --nextjs
   # Source: Next.js App Router project           (or "Pages Router")
   # Pages: N | API Routes: M | Layouts: L
   ```
2. **`app "<name>" { ... }`** — stack (`react` or `react + tailwind`), port,
   hardcoded `database sqlite "./data.db"`, optional `basepath`.
3. **`auth { ... }`** if detected — hardcoded block (see §3.7).
4. **`style { ... }`** — theme, optional accent, optional font.
5. **Entities** — from Prisma, if any. Preceded by
   `# ── Entities (from Prisma schema) ──`.
6. **Layout Main** — first layout with sidebar + nav items, if any.
7. **`# ── Pages ──`** then one `page "..." { ... }` block per detected page.
   - Type modifier `type:list|detail|form|dashboard` appended inline.
   - `requires:auth` appended if true.
   - `title "..."` inside, inferred from route via `route_to_title` (line
     1067-1090) — last segment, `-`/`_` replaced with space, title-cased.
   - `# data: <source>` comment if data-fetching was detected.
   - Placeholder section per page type (`table`, `card`, `form`) with **commented
     `# bind Entity` lines** — not real bindings, just hints.
8. **`# ── API Routes ──`** then grouped `api <prefix> { ... }` blocks. Routes
   are grouped by the first three path segments (line 1264-1273). Within each
   block, each method becomes a `name METHOD path auth:<jwt|public>` line,
   where name is `list` (GET+static), `detail` (GET+dynamic), `create` (POST),
   `update` (PUT/PATCH), `delete` (DELETE).
9. **Summary** printed to stderr (not written to file) — "Generated N blocks",
   "N → .cronus (Nx fewer files)".

There is **no middleware block** in the output; middleware is only used to
back-propagate `requires_auth` onto pages.

There is **no i18n block** in the output; `NextConfig.i18n` is collected but
never emitted.

There is **no `redirects` block** and **no `images.domains`** in the output.

---

## 6. Example Output Snippet

Real sample from `templates/ecosystem/rallly-polls.cronus:1-30`:

```cronus
# Generated by cronus dump --nextjs
# Source: Next.js App Router project
# Pages: 35 | API Routes: 16 | Layouts: 8

app "@rallly/web" {
  stack react
  port 3000
  database sqlite "./data.db"
}

auth {
  entity User
  login email + password
  session jwt expires:24h
  roles [admin, user]
}

style {
  theme light
}

layout Main {
  sidebar {
    "Home" -> "/"
  }
}

# ── Pages ──

page "/:locale/setup" type:detail {
```

Note the hardcoded `sqlite "./data.db"`, the email+password auth block (Rallly
actually uses OAuth + email magic links), the single-item sidebar, and the
theme defaulting to `light`.

---

## 7. Known Limitations (from code)

Drawn from code comments and observed behavior — no TODO/FIXME markers were
found in `nextjs.rs` itself, but the gaps are clear:

1. **Auth block is a lie.** Always `email + password` + `jwt 24h` + roles
   `[admin, user]`, regardless of actual auth (1154-1166).
2. **Route groups and parallel slots are stripped but not preserved.** A user
   reading the output cannot tell which pages were under which group.
3. **Catch-all marker lost.** `[...slug]` and `[[...slug]]` both become
   `:slug`, indistinguishable from a single dynamic segment.
4. **Pages Router API methods are fake.** Hardcoded `[GET, POST, PUT, PATCH,
   DELETE]` (586-589). Only App Router `route.ts` files get real methods.
5. **Only the first sidebar layout is emitted.** Nested layouts, multi-layout
   shells, and non-sidebar layouts (e.g. marketing topbar) are dropped (1205).
6. **`next.config.*` redirects and image domains are ignored.** Comment on
   817: `// Complex to parse, handled at runtime`.
7. **i18n is detected but not emitted.** Locales list is not parsed — only the
   default locale (852-854).
8. **Style detection is single-pass heuristic.** Theme defaults to `light`
   unless globals.css has `dark` and `:root` on the *same* line as a pattern
   match. Accent is raw value from the first matching line — no hex→name
   mapping, no deduping.
9. **All "auth-required" detection is substring-based.** Any page that
   mentions `auth` / `session` / `token` in a comment will be marked
   `requires:auth` (319-321, 754-758). False positives are expected.
10. **Middleware is not emitted as its own block.** Only the auth effect is
    back-propagated onto pages (205-214).
11. **No tailwind.config parsing.** Style extraction is 100% from globals.css.
12. **Prisma field-type mapping not verified in this pass** (`prisma.rs` out
    of scope) — the emitted entity block depends entirely on `dump_prisma`.
13. **Page sections are comment placeholders.** For list/detail/form pages,
    the emitted section has `# bind Entity { ... }` as a comment, not a real
    binding. Entity name is literally `Entity`, not the actual entity.
14. **`--audit` does not work for Next.js projects.** Only HTML
    (`dump_cmd.rs:90`).
15. **Test coverage is minimal.** Only the two tests added on 2026-04-10 exist
    in `nextjs.rs::tests` — no scanner tests, no emitter tests, no router
    discrimination tests, no fixture-based integration tests.

---

## 8. Ecosystem Templates

Output of `wc -l templates/ecosystem/*.cronus`:

| Template                            | Lines |
|-------------------------------------|-------|
| `calcom-scheduling.cronus`          | 1520  |
| `dub-analytics.cronus`              | 1497  |
| `formbricks-surveys.cronus`         |  469  |
| `rallly-polls.cronus`               |  350  |
| `plane-projects.cronus`             |  241  |
| `taxonomy-blog.cronus`              |  194  |
| `documenso-signing.cronus`          |   98  |
| `vercel-commerce.cronus`            |   51  |
| **Total**                           | **4420** |

All 8 are dump outputs of real OSS Next.js apps. Note the wide spread: Cal.com
and Dub produce ~1500-line dumps while Vercel Commerce lands at 51 lines. The
small ones are mostly the hardcoded `app`/`auth`/`style` block plus a handful
of pages — i.e., they show the dumper picked up the minimum but missed the
component-level content entirely.

---

## 9. Suggested Doc Copy (honest)

Three paragraphs, no marketing fluff. This is what could go on
`docs.cronus.test` under a "Next.js → .cronus" section.

---

### Dumping a Next.js project

```bash
cronus dump path/to/your/nextjs-app -o app.cronus
```

CRONUS detects Next.js automatically when any of `next.config.{ts,mjs,js}`
exists, or when `package.json` lists `next` or `vinext` as a dependency. You
can force the mode with `--nextjs`. The command writes a single `.cronus` file
that captures the project's routes, API endpoints, auth presence, Prisma
entities, and a minimal style block. Both the App Router and the legacy Pages
Router are supported, including hybrid projects that use both.

### What actually gets extracted

The dumper is a **structural scanner**, not a semantic compiler. It walks
`app/` and `pages/` following standard Next.js file conventions — page files,
`route.ts` handlers, `layout.tsx`, and `middleware.ts` — and extracts a flat
list of routes (with dynamic segments turned into `:param`), the HTTP methods
actually exported by each App Router `route.ts` (only GET/POST/PUT/PATCH/DELETE
are emitted; HEAD and OPTIONS are handled by the runtime), a single "Main"
layout built from the first sidebar-bearing `layout.tsx`, and a skeleton `auth`
block if it finds one of next-auth, Clerk, Lucia, Supabase, Firebase, or a
custom `lib/auth.ts`. Prisma schemas under `prisma/schema.prisma` are
converted into .cronus entities via the shared Prisma dumper.

### Known limits — read before relying on it

The output is a **starting point, not a lossless translation**. Route groups
`(group)` and parallel slots `@slot` are stripped. Catch-all segments
`[...slug]` collapse to `:slug`, losing the catch-all marker. The `auth` block
is a stub: it always declares email+password login, JWT sessions, and
`[admin, user]` roles regardless of the real configuration. For Pages Router
API files (`pages/api/**/*.ts`), the dumper assumes every method is handled
because it cannot introspect `export default` handlers without an AST. Only
the first layout with a sidebar is emitted — nested or topbar-only layouts are
dropped. Page section bodies are comment placeholders (`# bind Entity { ... }`)
rather than real bindings. For mission-critical conversions, treat the dumped
`.cronus` as a scaffold to edit, not a finished app.
