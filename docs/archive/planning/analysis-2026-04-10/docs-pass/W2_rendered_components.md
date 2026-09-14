# W2 — Rendered Components Catalog (live docs.cronus.test)

**Source:** `http://docs.cronus.test/components/*` (nginx/1.28.3, HTTP only, 127.0.0.1)
**Fetched:** 2026-04-10
**Tool note:** `WebFetch` force-upgrades HTTP→HTTPS and `docs.cronus.test` has no TLS listener (`ERR_TLS_CERT_ALTNAME_INVALID`). All 8 pages were pulled via `curl http://docs.cronus.test/...` and parsed. Content below is the live rendered HTML as nginx served it.

---

## 1. `/components` — Index

**Page title:** *Components*. **Opening claim (verbatim):** *"51 semantic section types organized by purpose. Each section is a self-contained building block that produces complete, styled HTML with zero configuration."* **H2 at the full catalog:** *"All 51 Section Types"*.

Index is **grouped by category**:

- **Marketing (7):** hero, features, pricing, cta, faq, testimonial, trusted
- **Navigation (7):** topbar, sidebar, breadcrumb, tabs, footer, page-header, links
- **Data Display (7):** table, chart, kpi, stat-cards, timeline, progress, kanban
- **Forms (2):** form, checkout
- **Cards & Lists (6):** card, product-grid, bento, team-list, status-card, activity-table
- **Feedback (9):** alert, modal, sheet, toast, skeleton, empty, error, not-found, command
- **Layout (7):** grid, layout, edge, accordion, pagination, filters, dark-mode
- **Aliases & Variants (10):** `404`→not-found, `stats`→kpi, `loading`→skeleton, `notifications`→toast, `dropdown`→command (also "select menu"), `quick-links`→links, `info-bar`→alert, `promo`→cta, `features-split`→features style:split, `policies`→legal accordion.

**Count reality:** 7+7+7+2+6+9+7 = **45 unique base types**; +10 aliases = **55**. The *"51"* number is asserted 3× but doesn't match either count. Discrepancy flagged.

**Also on the index page:**
- "Live Demos — Rendered by CRONUS" strip with 8 embedded previews: hero, features (bento), pricing, kpi/stats, cta, faq, testimonial, stats. **Chart is NOT in this strip.**
- Composed `landing.cronus` example (5 sections) and `dashboard.cronus` (sidebar + kpi + chart + table).
- A **"widgets Component v2"** block documenting a SECOND syntax surface (flagged §4.A).

---

## 2. Detail pages

### 2.1 `/components/hero` — "Hero"

*Full-width landing page hero: title, subtitle, CTA buttons, optional background image.*
Type: Pattern · Layer: Pattern · Status: Stable · Requires Items: **No** · Tags: hero, landing, marketing, cta, badge.

```
section hero {
  badge "Introducing CRONUS v0.1"
  title "Build full-stack apps with one file"
  subtitle "A declarative language that compiles to HTTP servers,
            databases, and server-rendered UIs."
  cta "Get Started" -> "/docs" primary
  cta "Documentation" -> "/reference" secondary
}
```

**Variants shown:** centered-dark w/ `image "bg.jpg" role:background`; two-column `style:ultima` w/ `image role:child` + `item role:terminal`; developer w/ terminal items; `variant:terminal` w/ `item "..." type:prompt|output|success|line`; background image form using `action "..." href: "..." style:primary|ghost` (alt CTA syntax); `variant:split` w/ `image "..." role:right`.

**Props:** `badge` (string), `title` (string, required), `subtitle` (string), `cta` (`string -> url`, modifier `primary|secondary`), `cta2` (alt 2nd CTA), `image role:background` (path), `item role:terminal` (string), `style:ultima` (modifier).
**Variants table:** `centered`, `split`, `terminal`, `light`, `minimal`.
**Terminal line types:** `prompt` (green $), `output` (gray), `success` (green), `line` (white).
**Notes:** Hero should be the first section; `role:background` triggers auto dark overlay; max 2 CTAs (first `primary` = filled).
**Binding:** Not mentioned. No `bind` block shown.

### 2.2 `/components/features` — "Features"

*Flexible feature showcase with grid, bento, and split layouts. Supports icons, images, chip children.*
Type: Pattern · Layer: Pattern · Status: Stable · Requires Items: **Yes** · Tags: features, bento, grid, cards, icons.

```
section features {
  title "Why CRONUS?"
  item "Zero Config" icon:bolt {
    "No webpack, no vite, no config files."
  }
  item "Type Safe" icon:shield {
    "Entity schemas generate typed APIs."
  }
  item "Single Binary" icon:package {
    "7MB compiled. Deploy anywhere."
  }
}
```

**Bento (excerpt):**
```
section features style:bento {
  title "Platform Features"
  item "Database" span: 2 icon:database {
    "SQLite with auto-migrations."
    image "db-diagram.png" role:child
  }
  item "51 Sections" span: 2 icon:grid {
    "Server-rendered UI components."
    chip "table" chip "chart" chip "form" chip "hero"
  }
}
```
**Split:** `section features style:split { title "..." image "..." role:child item "..." icon:x { "..." } }`. A second example uses `variant:split` + `image "..." side:right` — `style:` and `variant:` used interchangeably on the same page. A third passage uses `layout:bento`.

**Props:** `style:bento` (mod), `style:split` (mod), `cols` (number, default 3), `span` (number, bento), `icon` (Material Symbols string), `image role:child` (path), `chip` (string).
**Bento spans:** 4 (1/3), 6 (1/2), 8 (2/3), 12 (full). Mobile → full width. (First bento example above uses `span: 2` which is NOT in the allowed list.)
**Item children:** `description`, `chip`, `code`, `meta`, `label`, `action "..." href: "..."`, `image`.
**Common icons:** shield, bolt, code, rocket_launch, speed, security, visibility, analytics, palette, tune, star, check_circle, storage, api, terminal.
**Binding:** Not mentioned. No `bind` block shown.

### 2.3 `/components/pricing` — "Pricing"

*Pricing cards with plan tiers, feature lists, CTAs. Monthly/yearly toggle and highlighted recommended plans.*
Type: Pattern · Layer: Pattern · Status: Stable · Requires Items: **Yes (plans)** · Tags: pricing, plans, saas, billing.

```
section pricing {
  title "Simple pricing"
  subtitle "Start free, scale when ready."

  plan "Starter" price: 0 {
    feature "1 project"
    feature "SQLite database"
    feature "Community support"
    cta "Get Started" -> "/signup"
  }
  plan "Pro" price: 29 recommended {
    feature "Unlimited projects"
    feature "PostgreSQL + SQLite"
    feature "Priority support"
    feature "Custom domains"
    cta "Start Free Trial" -> "/signup?plan=pro"
  }
  plan "Enterprise" price: "custom" {
    feature "Everything in Pro"
    feature "SLA guarantee"
    feature "Dedicated support"
    feature "On-premise deploy"
    cta "Contact Sales" -> "/contact"
  }
}
```

**Base props:** `plan` (string, card title), `price` (number | `"custom"`), `recommended` (flag, accent highlight), `feature` (string, check icon), `cta` (`string -> url`).
**Additional props:** `description` (plan subtitle), `toggle "monthly, yearly"` (period toggle csv), `yearly_discount` (number %), `badge` (string), `footnote` (string).
**Binding:** Not mentioned. No `bind` block shown.

### 2.4 `/components/table` — "Table"

*Data table with auto column generation from entity fields. Sorting, search, pagination, inline editing.*
Type: Stdlib · Layer: Stdlib · Status: Stable · Requires Items: **Yes** · Tags: table, data, crud, sorting, pagination.

```
entity Order {
  customer string required
  product string required
  amount integer required
  status string default: "pending"
  created_at datetime auto
}

section table {
  title "Orders"
  bind entity:Order { query all }
  columns "customer, product, amount, status, created_at"
  search true
  pagination 20
  actions "edit, delete"
}
```

**Props:** `bind entity:Name` (entity, auto-columns from fields), `columns` (csv), `search` (bool), `pagination` (rows/page), `actions` ("edit,delete,view"), `sort` (string).
**Column overrides:** `column "name" label: "..." width: "..." format: currency|relative|date|number badge: bool sortable: bool`.
**Search/filter:** `search true`, `search_fields "a, b"`, `filter "status" options: "a||b||c"`, `filter "created_at" type: "date_range"`, `sort "created_at" direction: "desc"`.
**Bind query forms:**
```
bind entity:Order { query all }
bind entity:Order { query where status = "active" }
bind entity:Order { query all order_by: "created_at" limit: 10 }
bind entity:Order { query all include: "customer, products" }
```
**Binding: REQUIRED** — every example opens with `bind entity:Name { ... }`; Props row for `bind` says *"Entity to display. Columns auto-generated from fields."*

### 2.5 `/components/chart` — "Chart"

*Auto-detected chart visualization from entity data. CRONUS selects best chart type (bar, line, pie, area) automatically.*
Type: Stdlib · Layer: Stdlib · Status: Stable · Requires Items: **Yes** · Tags: chart, bar, line, area, donut, visualization.

```
entity Revenue {
  month    string  required
  amount   integer required
  category string
}

section chart {
  title "Revenue Overview"
  bind entity:Revenue { query all }
  x "month"
  y "amount"
  group "category"
}
```

Narrative (verbatim): *"When `x` is a date/string and `y` is numeric, CRONUS renders a bar chart. With a `group` field, it produces grouped/stacked bars. For time-series data, it auto-selects a line chart."*

**"Chart Types" section has four headings (Bar / Line / Pie / Area) with ONE-SENTENCE text descriptions and NO rendered previews / canvas / screenshots.**

**Props table:** `bind` (entity:Name binding), `x` (string, x-axis field), `y` (string, y numeric), `group` (string, series field), `type` (`bar|line|pie|area`, default auto).

**Per-type examples use DIFFERENT prop names that conflict with the Props table:**
```
section chart {
  title "Monthly Revenue"
  type "bar"                          # quoted string vs. enum
  bind entity:Revenue { query all }
  x_axis "month"                      # NOT in Props table (table says "x")
  y_axis "amount" format: "currency"  # NOT in Props table
  color "#CC0000"
}

section chart {
  title "User Growth"
  type "line"
  bind entity:Metric { query all }
  x_axis "date"
  y_axis "users"
  periods "7d, 30d, 90d, 1y"
}

section chart {
  title "Traffic Overview"
  type "area"
  bind entity:Traffic { query all }
  x_axis "date"
  y_axis "visits"
  group "source"            # stacked
  fill_opacity 0.3
}

section chart {
  title "Revenue by Category"
  type "donut"              # "donut" NOT in Props table (only bar|line|pie|area)
  bind entity:Revenue { query all }
  group "category"
  y_axis "amount"
  show_legend true
}
```
**Config props:** `periods` (csv time ranges), `color` (hex), `fill_opacity` (0–1), `show_legend` (bool), `height` (string, default `300px`).
**Binding: REQUIRED** — `bind entity:Name { query ... }` in every example.
**Fallback / generic-card rendering: NOT admitted.** See flag §4.C.

### 2.6 `/components/kpi` — "KPI"

*KPI cards with metrics, trend indicators, delta values. Alias: `stats`.*
Type: Stdlib · Layer: Stdlib · Status: Stable · Requires Items: **Yes** · Tags: kpi, metrics, dashboard, stats, trends.

```
section kpi {
  item "Total Revenue" value: "$48,230" delta: "+12.5%" trend:up
  item "Active Users"  value: "2,847"   delta: "+4.2%"  trend:up
  item "Conversion"    value: "3.6%"    delta: "-0.8%"  trend:down
  item "Churn Rate"    value: "1.2%"    delta: "-0.3%"  trend:up
}
```

**Entity-bound:**
```
section kpi {
  bind entity:Order {
    query count
    compare period: "30d"
  }
  item "Orders"  field: "count"
  item "Revenue" field: "sum:amount"
}
```

**Aggregate-bound:**
```
section kpi {
  bind entity:Order {
    aggregate count
    aggregate sum(total)
    aggregate avg(total)
    compare period: "30d"
  }
  item "Total Orders" field: "count"
  item "Revenue"      field: "sum_total" format:money
  item "Avg Order"    field: "avg_total" format:money
}
```

**Props:** `value` (static string), `delta` (change indicator), `trend` (`up|down|flat`), `field` (entity field / `count` / `sum:field`), `compare period` (string, `7d|30d|90d`).
**Trend badges:** up=green+up arrow, down=red+down arrow, flat=neutral. Bound `compare period` auto-computes delta+trend; manual values override.
**Dashboard variant:** `section kpi cols: N { ... }` where N ∈ {2,3,4} (default 4). Obsidian theme = dark cards.
**Binding: OPTIONAL.** Both static and `bind entity:Name { ... }` forms shown.

### 2.7 `/components/form` — "Form"

*Auto-generated forms from entity field declarations. Reads entity schema → form with validation, labels, proper input types.*
Type: Stdlib · Layer: Stdlib · Status: Stable · Requires Items: **Yes** · Tags: form, input, validation, fields, submit.

```
entity Contact {
  name     string required
  email    email  required
  phone    string
  message  text   required
  category string enum: "sales,support,other"
}

section form {
  title "Contact Us"
  bind entity:Contact { action create }
  submit "Send Message"
}
```

**Entity → input mapping:** `string`→text, `email`→email, `text`→textarea, `integer|float`→number, `boolean`→checkbox, `datetime`→datetime-local, `enum`→select.
**Props:** `bind entity:Name` (binding), `action` (`create|edit`), `submit` (string), `fields` (csv subset), `redirect` (url).

**Explicit field form (15 field types):**
```
section form {
  title "User Profile"
  field "name"     type: "text"     required placeholder: "Full name"
  field "email"    type: "email"    required
  field "password" type: "password" required min: 8
  field "website"  type: "url"      placeholder: "https://..."
  field "phone"    type: "tel"
  field "age"      type: "number"   min: 1 max: 120
  field "role"     type: "select"   options: "admin||editor||viewer"
  field "bio"      type: "textarea" rows: 4
  field "active"   type: "checkbox"
  field "plan"     type: "radio"    options: "free||pro||enterprise"
  field "avatar"   type: "file"     accept: "image/*"
  field "birthday" type: "date"
  field "salary"   type: "money"    currency: "USD"
  field "color"    type: "color"    default: "#CC0000"
  field "tags"     type: "tags"
}
```

**Field types:** text, email, password, url, tel, number, select, textarea, checkbox, radio, file, date, money, color, tags (options use `||` separator). Per-type sub-props: min/max/pattern/rows/accept/currency/max_tags/default/min_date/max_date/strength.
**Validation:** `required`, `unique`, `min`, `match: "otherField"` — enforced on client AND server.
**Edit mode:** `bind entity:User { action edit id:params.id }` + individual `readonly` fields.
**Binding:** Optional in the explicit-`field` mode; primary example and edit example BOTH use `bind entity:Name { action ... }`. Mixing auto-bound and explicit fields is NOT demonstrated.

---

## 3. Cross-reference: live doc claims vs. user reality

| Claim / Feature | Doc says | User reality |
|---|---|---|
| "51 section types" | Stated 3× | 45 unique enumerated; +10 aliases = 55. Marketing constant, not a count. |
| `section <type> { ... }` THE syntax | Every detail page | Index also documents `component Name(params) { state...; template "..." }` with `@click` signals — second language surface. |
| Hero CTA shape | `cta "text" -> "/url" primary` | Same page also uses `action "text" href: "/url" style:primary` — two coexisting forms, no reconciliation. |
| Features layout modifier | `style:bento` / `style:split` | Same page also uses `layout:bento` AND `variant:split`. Three keyword families. |
| Chart axis props | Props table: `x`, `y`, `type` enum | Examples: `x_axis`, `y_axis`, `type "bar"` string, `type "donut"` (not in enum). |
| Bento `span` values | "4, 6, 8, 12" | First bento example uses `span: 2` (not in allowed list). |
| Table requires entity | `bind` Props row | Confirmed — all examples use `bind`; user MUST define entity first. |
| KPI binding | Dynamic via `bind` | Both static and bound supported; static works without any entity. |
| Form mixing modes | — | Auto-from-`bind` and explicit `field` modes shown separately; mixing not demonstrated, not forbidden. |
| Chart rendering | "rendered by CRONUS" | Zero rendered chart previews on the page. See §4.C. |

---

## 4. Flags

### Flag A — Non-`section <type> { ... }` syntax on the live index page

The `/components` index documents a second component form (NOT `section`):

```
component Counter(initial: int) {
  state count = initial
  template "<button @click='count++'>Clicked {{count}} times</button>"
}

component UserCard(user_id: string, show_avatar: boolean) {
  state loading = true
  section card {
    title "{{user.name}}"
    subtitle "{{user.email}}"
    icon "person"
  }
}
```

Features table on that page: `component Name(param: type)` params, `state name = default` local reactive state, `template "html"`, `test "name" { ... }`, `@click` / `@change` signals, `{{...}}` mustache interpolation inside templates. **None** of this is `section <type> { ... }`.

### Flag B — Sections claiming binding without a `bind Entity { ... }` block

**None.** Across all 7 detail pages, every section that claims binding support shows a `bind entity:Name { ... }` block:
- `table` — REQUIRED, shown in every example.
- `chart` — REQUIRED, shown in every example.
- `kpi` — optional; bound form shows a real `bind entity:Order { ... }` block.
- `form` — optional; bound form shows `bind entity:Contact { action create }`.
- `hero`, `features`, `pricing` — do NOT claim binding support and do NOT show `bind` blocks. (Clean — no mismatch.)

### Flag C — `/components/chart` and the known generic-card-fallback issue

The chart detail page:
1. **Claims** auto-detection: *"CRONUS analyzes the bound entity fields and selects the best chart type (bar, line, pie, area) automatically."*
2. **Shows four chart-type subsections** (Bar/Line/Pie/Area) each with one-sentence prose and NO rendered preview, canvas, SVG, or screenshot. Contrast with hero/features/kpi/pricing/form which have "Live Preview — Rendered by CRONUS engine" banners + visible previews.
3. **Index "Live Demos" strip** renders 8 components live (hero, features-bento, pricing, kpi/stats, cta, faq, testimonial, stats). **`chart` is NOT among the 8.**
4. **Copy never admits** fallback-to-card, incomplete rendering, or why previews are absent.

**Conclusion:** The chart doc does NOT admit the generic-card fallback. It sidesteps the topic by omitting live previews on the chart page and by excluding chart from the index's rendered-live strip, while still asserting auto-detected chart rendering in prose. A user reading only this page learns nothing about the fallback.

### Flag D — Inconsistent modifier namespaces

`style:*`, `layout:*`, `variant:*` used interchangeably for the same concept (bento/split/terminal variants on hero and features). Doc-cleanup signal, not blocking.

### Flag E — Props-table-vs-examples divergence on `chart`

Props table documents `x` / `y` / `type` (enum `bar|line|pie|area`). Examples below use `x_axis` / `y_axis` / quoted `type "bar"` / `type "donut"` (not in enum). Two parallel APIs on the same page with zero note.

---

## Appendix — URLs

- http://docs.cronus.test/components
- http://docs.cronus.test/components/hero
- http://docs.cronus.test/components/features
- http://docs.cronus.test/components/pricing
- http://docs.cronus.test/components/table
- http://docs.cronus.test/components/chart
- http://docs.cronus.test/components/kpi
- http://docs.cronus.test/components/form
