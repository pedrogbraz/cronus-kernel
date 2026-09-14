# SDD — Zero Hardcode Enforcement System

> Software Design Document — CRONUS Kernel
> Author: Zedd + Claude
> Date: 2026-04-02
> Status: PROPOSED

---

## 1. Problem Statement

The CRONUS language generated UI with hardcoded values that masqueraded as live data: fake metrics in footers ("Uptime: 99.99%"), dead UI elements without actions, `location.reload()` calls that broke the SPA contract, and placeholder data that looked real. This violates the core promise of CRONUS: **one .cronus file = a real, working, live application**.

A language that allows its users to ship fake data is a language that will never be trusted. This must be enforced at the **language level**, not by code review.

---

## 2. Design Principles

1. **Fail loud, not silent** — a hardcode violation is a build error, not a warning
2. **Zero runtime cost for valid code** — enforcement happens at parse/build time
3. **Progressive strictness** — `cronus build` warns, `cronus build --strict` errors
4. **Developer velocity preserved** — no extra syntax, no boilerplate, smart defaults
5. **Self-documenting** — violations explain what's wrong AND how to fix it

---

## 3. Architecture Overview

```
                    PARSE TIME              BUILD TIME              RUNTIME
                    ──────────              ──────────              ───────
.cronus file ──→ [ AST Parser ] ──→ [ Lint Pipeline ] ──→ [ Live Watchdog ]
                       │                     │                      │
                       ▼                     ▼                      ▼
                  Structural          Static Analysis          Behavioral
                  Validation          (7 Lint Rules)           Monitoring
                       │                     │                      │
                       ▼                     ▼                      ▼
                  ParseError            LintWarning              Console
                  (blocks run)          or LintError             Warning
                                        (blocks build             + /api/
                                         in strict)              _health
```

---

## 4. The 7 Lint Rules

### Rule 1: `no-dead-text` — Detect Hardcoded Metrics in Templates

**What it catches:**
- Numbers with units in template HTML: "99.99%", "14ms", "$142,804", "1.2M"
- Patterns like `Noun: Value` where Value looks numeric: "Uptime: 99.99%"
- Version strings: "v2.4.0", "v1.0.0-stable"

**How it works:**
```
For each section with template:
  1. Parse HTML text nodes (strip tags)
  2. Regex scan for metric patterns:
     - /\d+(\.\d+)?%/          → percentage
     - /\d+(\.\d+)?(ms|s|m|h)/ → duration
     - /\$[\d,.]+/             → currency
     - /v\d+\.\d+/            → version
     - /\d+(\.\d+)?(K|M|B|T)/ → magnitude
  3. Check if the text node is inside a <script> tag → SKIP (JS is OK)
  4. Check if the text node is inside an attribute like placeholder="" → SKIP
  5. Otherwise → VIOLATION
```

**Severity:** WARNING (default), ERROR (--strict)

**Fix suggestion:**
```
⚠ no-dead-text: "Uptime: 99.99%" in footer template looks like hardcoded metric
  → Use a <span id="..."> and populate via JS fetch from /api/server/stats
  → Or bind to an entity field: bind SystemMetric { where key eq "uptime" }
```

---

### Rule 2: `no-dead-links` — Every Clickable Element Must Navigate

**What it catches:**
- `href="#"` anywhere
- `<a>` or `<button>` without `href`, `onclick`, `data-scroll`, `data-cronus-action`, or `data-modal-open`
- `href` pointing to a route that doesn't exist in the parsed pages

**How it works:**
```
For each section with template:
  1. Parse HTML, find all <a> and <button> elements
  2. For <a>: must have href (not "#") OR onclick OR data-* action attribute
  3. For <button>: must have onclick OR type="submit" (inside form) OR data-* attribute
  4. If href exists and is internal (starts with /):
     - Check if route exists in parsed pages OR is a kernel route (/docs, /graphql)
     - If not → VIOLATION (dead link)
```

**Severity:** ERROR always (dead links are never acceptable)

**Fix suggestion:**
```
✗ no-dead-links: <a href="#"> in topbar has no destination
  → Remove the element or point it to a real page route
```

---

### Rule 3: `no-dead-ui` — Every Interactive Element Must Have Behavior

**What it catches:**
- Visual elements that look interactive but aren't: chevron_right icons without onclick, settings icons without navigation
- Sections with `section modal` but no `action` items
- Items in sidebar/nav that don't have `->` routing

**How it works:**
```
For each section with template:
  1. Find elements with cursor:pointer or hover effects
  2. Check they have an event handler (onclick, href, data-*)
  3. If not → VIOLATION
```

**Severity:** WARNING

---

### Rule 4: `no-fake-state` — Detect Static Text That Implies Live State

**What it catches:**
- Text containing: "Loading", "Live", "Online", "Active", "Connected"
  that is rendered in static HTML (not populated by JS)
- Dot/badge with `animate-pulse` next to static text (implies live but isn't)

**How it works:**
```
For each section with template:
  1. Find text nodes containing state keywords
  2. Check if the parent element has an id="" (will be populated by JS) → OK
  3. Check if it's inside a <script> tag → OK
  4. Otherwise → VIOLATION (static text pretending to be live)
```

**Severity:** WARNING

**Exception:** Skeleton/loading states shown before JS hydrates are OK if they have a corresponding JS that replaces them.

---

### Rule 5: `no-orphan-reload` — Ban location.reload() in Generated Code

**What it catches:**
- Any `location.reload()` or `window.location.reload()` in:
  - Template scripts
  - Runtime JS (render.rs, runtime_js.rs)
  - UI renderers (ui.rs)
  
**Exceptions:**
- HMR (hmr.rs) — development only, acceptable
- Explicit `data-hard-reload="true"` attribute — opt-in escape hatch

**How it works:**
```
At build time:
  1. Scan all generated JS (templates + kernel runtime)
  2. Regex: /(?:window\.)?location\.reload\(\)/
  3. If found outside hmr.rs → VIOLATION

At parse time:
  1. Scan template strings in .cronus for location.reload
  2. If found → VIOLATION
```

**Severity:** ERROR always

---

### Rule 6: `no-hardcode-user` — User Data Must Come From Auth

**What it catches:**
- Hardcoded names in templates: "System Admin", "Admin", "Guest"
  that aren't inside `<script>` tags or `id=""` elements
- Hardcoded emails: anything matching `*@*.*` in HTML text
- Hardcoded roles: "admin", "operator", "viewer" in HTML text (not in enum definitions)

**How it works:**
```
For each section with template:
  1. Parse HTML text nodes
  2. Pattern match for user-like data
  3. Check if inside <script> (dynamic) → OK
  4. Check if element has id="" (will be replaced) → OK
  5. Otherwise → VIOLATION
```

**Severity:** WARNING

---

### Rule 7: `bind-or-empty` — Sections With Data Must Bind or Show Empty State

**What it catches:**
- Section type `kpi`, `table`, `chart`, `kanban`, `timeline`, `stats`
  that has neither:
  - `bind Entity { ... }` (data from DB)
  - Static items (explicitly defined data)
  
  AND doesn't render an empty state

**How it works:**
```
For each data section (kpi, table, chart, etc.):
  1. Has binding? → OK (kernel renders empty state automatically)
  2. Has items? → OK (static data is intentional)
  3. Neither → VIOLATION (section will render empty with no explanation)
```

**Severity:** WARNING

---

## 5. Implementation Plan

### Phase 1: Lint Engine (Parser Integration)

**File:** `src/lint.rs` (NEW — ~400 lines)

```rust
pub struct LintResult {
    pub rule: &'static str,
    pub severity: Severity,
    pub message: String,
    pub fix: String,
    pub line: usize,
    pub section: String,
}

pub enum Severity { Warning, Error }

pub fn lint_ast(nodes: &[AstNode], strict: bool) -> Vec<LintResult> {
    let mut results = Vec::new();
    
    // Run all 7 rules
    results.extend(rule_no_dead_text(nodes));
    results.extend(rule_no_dead_links(nodes));
    results.extend(rule_no_dead_ui(nodes));
    results.extend(rule_no_fake_state(nodes));
    results.extend(rule_no_orphan_reload(nodes));
    results.extend(rule_no_hardcode_user(nodes));
    results.extend(rule_bind_or_empty(nodes));
    
    // In strict mode, promote all warnings to errors
    if strict {
        for r in &mut results {
            if matches!(r.severity, Severity::Warning) {
                r.severity = Severity::Error;
            }
        }
    }
    
    results
}
```

**Integration point:** Called in `cmd_build()` and `cmd_run()` after parsing.

### Phase 2: CLI Output

```
$ cronus build --strict

  CRONUS v0.1.0 — Lint Results

  ✗ no-dead-text (footer): "Uptime: 99.99%" is hardcoded metric
    → Populate via JS from /api/server/stats

  ✗ no-orphan-reload (modal): location.reload() found in template script
    → Use CRONUS.reload() for soft refresh

  ✗ no-dead-links (topbar): <a href="#"> has no destination
    → Point to a real page route or remove

  2 errors, 1 warning — build blocked (strict mode)
```

```
$ cronus build

  CRONUS v0.1.0 — Lint Results

  ⚠ no-dead-text (footer): "Uptime: 99.99%" is hardcoded metric
  ⚠ no-dead-links (topbar): <a href="#"> has no destination

  0 errors, 2 warnings — build OK (use --strict to block)
```

### Phase 3: Runtime Watchdog

**File:** Extend `src/brain.rs`

The brain already tracks requests. Add behavioral detection:

```rust
impl CronusBrain {
    /// Detect runtime violations after N requests
    pub fn behavioral_audit(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        // Check: are there pages that NEVER receive data mutations?
        // (indicates they might have hardcoded data)
        
        // Check: are there entities with 0 rows that have bound sections?
        // (indicates empty states or missing seed)
        
        // Check: are there endpoints with 100% identical responses?
        // (indicates static/cached responses that should be dynamic)
        
        issues
    }
}
```

**Exposed via:** `GET /api/_health` endpoint (auto-generated, always available)

```json
{
  "status": "healthy",
  "lint": {
    "warnings": 0,
    "errors": 0
  },
  "behavioral": [
    "Entity 'Deployment' has 0 rows — /deployments page shows empty state"
  ]
}
```

### Phase 4: Continuous Integration

**`cronus doctor` enhancement:**

```
$ cronus doctor

  ✓ Syntax valid (9 pages, 6 entities)
  ✓ Port 5175 available
  ✓ Database ./data.db accessible (6 tables)
  ✓ Zero hardcode violations (7 rules passed)     ← NEW
  ✓ Zero dead links (all hrefs resolve)            ← NEW  
  ✓ Zero orphan reloads                            ← NEW
  ✓ All data sections have bindings                ← NEW
  
  Health: CLEAN
```

---

## 6. Template Lint API (For AI Code Generation)

When AI (Luminus, Claude, etc.) generates .cronus code, it should validate before outputting:

```rust
/// Validate a template string before emitting it
pub fn lint_template(html: &str) -> Vec<LintResult> {
    let mut issues = Vec::new();
    
    // Quick regex checks on raw HTML
    // No need to parse full DOM — pattern matching is fast enough
    
    issues.extend(check_dead_metrics(html));
    issues.extend(check_dead_links(html));
    issues.extend(check_orphan_reload(html));
    issues.extend(check_hardcode_user(html));
    
    issues
}
```

This can be called by the `cronus generate` command and by any AI integration to prevent hardcoded output **before it's written**.

---

## 7. Performance Impact

| Phase | When | Cost |
|-------|------|------|
| Parse-time lint | `cronus build` / `cronus run` | +2-5ms (regex on templates) |
| Runtime watchdog | Every 60s background | +1ms (DB count queries) |
| Template lint API | On AI generation | +<1ms per template |

**Total impact on startup:** <5ms additional. **Zero impact on request handling.**

---

## 8. Migration Path

1. **Week 1:** Implement `lint.rs` with rules 1, 2, 5 (dead-text, dead-links, orphan-reload)
2. **Week 2:** Add rules 3, 4, 6, 7 (dead-ui, fake-state, hardcode-user, bind-or-empty)
3. **Week 3:** Integrate with `cronus build --strict`, `cronus doctor`, `/api/_health`
4. **Week 4:** Runtime watchdog in brain.rs, template lint API

---

## 9. Non-Goals

- **NOT a type checker** — we don't validate JS logic inside templates
- **NOT a security scanner** — that's a separate concern (already handled)
- **NOT a style enforcer** — design decisions are the user's choice
- **NOT blocking by default** — warnings don't stop `cronus run`, only `--strict` does

---

## 10. Success Criteria

The system is successful when:

1. `cronus build --strict` catches 100% of the 21 hardcode issues found in the audit
2. Zero `location.reload()` can exist in kernel code without explicit opt-in
3. AI-generated .cronus code passes all 7 lint rules before being written
4. `/api/_health` reports real-time behavioral issues
5. `cronus doctor` gives a clean bill of health for properly written apps
6. No performance regression on startup (stays under 50ms)

---

## Appendix A: Regex Patterns for Rule 1

```
METRIC_PATTERNS = [
    r"\d+(\.\d+)?%"                    # 99.99%, 12%
    r"\d+(\.\d+)?(ms|s|m|h|d)"        # 14ms, 2.5s, 30m
    r"\$[\d,.]+"                        # $142,804
    r"\d+(\.\d+)?(K|M|B|T|GB|MB|TB)"  # 1.2M, 4.5GB
    r"v\d+\.\d+(\.\d+)?(-\w+)?"       # v2.4.0-stable
]

SKIP_CONTEXTS = [
    inside <script> tags
    inside placeholder="" attributes  
    inside data-* attributes
    inside id="" elements (will be populated by JS)
    inside .cronus config values (periods, y_axis, x_axis)
]
```

## Appendix B: Known Acceptable Hardcodes

These are NOT violations:

- **App name** in `app "NOVA CORE"` — this IS the config, not hardcode
- **Entity field names** — structural, not data
- **Placeholder text** in inputs — UX guidance, not fake data
- **Chart axis labels** in .cronus config — DSL configuration, not live data
- **Webhook URLs** — user-configured endpoints, not fake
- **CSS values** — design tokens, not data
