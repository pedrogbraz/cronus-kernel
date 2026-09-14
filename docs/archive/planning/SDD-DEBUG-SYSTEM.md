# SDD — CRONUS Debug System

> Debug integrado na linguagem: backend + frontend + browser + audit.
> Author: Zedd + Claude | Date: 2026-04-02 | Status: PROPOSED

---

## 1. Problem Statement

CRONUS gera apps fullstack a partir de um arquivo .cronus. Quando algo quebra, o desenvolvedor não tem ferramentas integradas para debugar — precisa abrir DevTools manualmente, caçar em logs do terminal, ou adivinhar onde o erro está.

Uma linguagem que gera o app inteiro DEVE debugar o app inteiro. O debug deve ser tão automático quanto o render.

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    CRONUS DEBUG SYSTEM                    │
├─────────────┬──────────────┬──────────────┬─────────────┤
│  Layer 1    │   Layer 2    │   Layer 3    │   Layer 4   │
│  BACKEND    │   FRONTEND   │   BROWSER    │   AUDIT     │
│  (Rust)     │   (Generated │   (Overlay)  │   (Chain)   │
│             │    HTML/JS)  │              │             │
├─────────────┼──────────────┼──────────────┼─────────────┤
│ Request log │ Error catch  │ Debug panel  │ Hash verify │
│ SQL trace   │ Render trace │ Network tab  │ Diff viewer │
│ Panic guard │ State viewer │ Component    │ Timeline    │
│ Perf timing │ Binding log  │ Console      │ Rollback    │
│ SSE debug   │ SPA trace    │ Responsive   │ Export      │
└─────────────┴──────────────┴──────────────┴─────────────┘
```

---

## 3. Layer 1: Backend Debug (Rust/Server)

### 3.1 Request Tracing

**Already exists (brain.rs):** Every HTTP request is tracked with method, path, status, duration.

**Enhancement needed:**

```rust
// In handle_request, wrap with debug tracing
struct RequestTrace {
    id: String,           // unique request ID (passed via X-Request-Id header)
    method: String,
    path: String,
    started_at: Instant,
    query_count: u32,     // number of SQL queries executed
    queries: Vec<SqlTrace>,
    binding_resolution: Option<BindingTrace>,
    render_ms: u64,
    response_status: u16,
    response_size: usize,
}

struct SqlTrace {
    query: String,        // sanitized SQL (no values, just structure)
    table: String,
    rows_returned: usize,
    duration_ms: u64,
}

struct BindingTrace {
    entity: String,
    query_type: String,   // "all", "one", "count"
    filters: Vec<String>,
    rows_resolved: usize,
}
```

**Endpoint:** `GET /api/debug/traces?limit=50`

Returns the last N request traces with full detail — which SQL queries ran, how long each took, which bindings were resolved.

### 3.2 SQL Query Log

Every database query gets logged with:
- The sanitized SQL (parameters replaced with `?`)
- Execution time
- Rows returned/affected
- The entity it came from
- Which page/section triggered it

**Endpoint:** `GET /api/debug/queries?limit=50`

### 3.3 Panic Guard

Wrap `handle_request` in a catch_unwind:

```rust
match std::panic::catch_unwind(AssertUnwindSafe(|| {
    handle_request(req, state.clone())
})) {
    Ok(result) => result,
    Err(panic) => {
        let msg = format!("Internal error on {} {}", method, path);
        eprintln!("  \x1b[31m✗ PANIC\x1b[0m {}: {:?}", path, panic);
        // Log to debug traces
        // Return 500 with debug info (only in dev mode)
        json_response(500, json!({
            "error": "internal_error",
            "path": path,
            "debug": format!("{:?}", panic)
        }))
    }
}
```

### 3.4 Performance Timing

Every response gets timing headers:

```
X-Request-Id: req_abc123
X-Response-Time: 12ms
X-Query-Count: 3
X-Render-Time: 4ms
X-Binding-Time: 2ms
```

### 3.5 SSE Debug Channel

A dedicated SSE channel for debug events:

**Endpoint:** `GET /api/debug/stream`

Streams real-time events:
```json
{"type": "request", "method": "GET", "path": "/api/deployments", "status": 200, "ms": 12}
{"type": "query", "sql": "SELECT * FROM Deployment WHERE _owner_id = ?", "rows": 5, "ms": 2}
{"type": "error", "message": "...", "stack": "..."}
{"type": "sse_broadcast", "entity": "Deployment", "action": "created"}
{"type": "audit", "action": "INSERT", "entity": "Deployment", "hash": "abc..."}
```

---

## 4. Layer 2: Frontend Debug (Generated HTML/JS)

### 4.1 Error Boundary

Every rendered page gets a global error handler:

```javascript
// Injected in CRONUS_RUNTIME_JS
window.addEventListener('error', function(e) {
    cronusDebug.logError({
        type: 'js_error',
        message: e.message,
        filename: e.filename,
        line: e.lineno,
        col: e.colno,
        stack: e.error?.stack,
        page: location.pathname,
        timestamp: new Date().toISOString()
    });
});

window.addEventListener('unhandledrejection', function(e) {
    cronusDebug.logError({
        type: 'promise_rejection',
        message: e.reason?.message || String(e.reason),
        stack: e.reason?.stack,
        page: location.pathname
    });
});
```

Errors are:
1. Shown in the debug overlay (if open)
2. Sent to `/api/debug/client-error` endpoint
3. Stored in the brain for pattern analysis

### 4.2 SPA Navigation Trace

Every SPA navigation is traced:

```javascript
// In cronusNavigate()
cronusDebug.trace({
    type: 'navigation',
    from: location.pathname,
    to: url,
    duration_ms: elapsed,
    content_size: newContent.length,
    scripts_executed: scriptCount
});
```

### 4.3 Binding Debug

When a section resolves its binding:

```javascript
// Auto-injected when debug mode is on
cronusDebug.trace({
    type: 'binding',
    section: sectionType,
    entity: entityName,
    rows: data.length,
    page: location.pathname
});
```

### 4.4 Fetch Interceptor Debug

The existing fetch interceptor gets debug logging:

```javascript
// Already intercepts fetch — add debug logging
var _originalFetch = window.fetch;
window.fetch = function() {
    var url = arguments[0];
    var start = performance.now();
    return _originalFetch.apply(this, arguments).then(function(response) {
        cronusDebug.trace({
            type: 'fetch',
            url: url,
            method: (arguments[1]?.method || 'GET'),
            status: response.status,
            duration_ms: Math.round(performance.now() - start),
            size: response.headers.get('content-length')
        });
        return response;
    });
};
```

### 4.5 Render Performance

Track how long each SPA content swap takes:

```javascript
cronusDebug.trace({
    type: 'render',
    page: url,
    fetch_ms: fetchDuration,
    parse_ms: parseDuration,
    swap_ms: swapDuration,
    total_ms: totalDuration,
    dom_nodes: document.getElementById('cronus-content')?.childNodes.length
});
```

---

## 5. Layer 3: Browser Debug Overlay

### 5.1 Debug Panel (Cmd+Shift+D)

A sliding panel that shows real-time debug info. Activated by keyboard shortcut or `?debug=1` URL param.

```
┌──────────────────────────────────────┐
│ CRONUS DEBUG                    [×]  │
├──────────────────────────────────────┤
│ [Network] [Errors] [Bindings] [Perf]│
├──────────────────────────────────────┤
│                                      │
│ GET /api/deployments    200   12ms   │
│ GET /api/kpisnapshots   200    3ms   │
│ SSE connected           ●            │
│ SPA nav → /analytics    fade  180ms  │
│                                      │
│ Errors: 0                            │
│ Queries this page: 4                 │
│ Bindings resolved: 3                 │
│ Last audit: INSERT Deployment 2s ago │
│                                      │
├──────────────────────────────────────┤
│ Page: /deployments                   │
│ Sections: 3 (kpi, kpi, table)       │
│ Entities bound: Deployment, KPI      │
│ Auth: jwt (user: zedd)              │
│ CSP nonce: abc123...                 │
│ SSE: connected (42 events received)  │
└──────────────────────────────────────┘
```

### 5.2 Component Inspector

Click any section to see its debug info:

```
┌─────────────────────────┐
│ section kpi              │
│ bind: KpiSnapshot        │
│ where: page eq "overview"│
│ rows: 4                  │
│ render time: 2ms         │
│ last update: 3s ago      │
│ doc: "Metricas de alto…" │
└─────────────────────────┘
```

Implemented by adding `data-cronus-debug` attributes to each rendered section:

```html
<section data-cronus-debug='{
    "type": "kpi",
    "entity": "KpiSnapshot",
    "rows": 4,
    "render_ms": 2
}'>
```

### 5.3 Responsive Debug

Show breakpoint indicator and layout metrics:

```
┌────────────────────────┐
│ 🖥 Desktop (1440px)     │
│ Sidebar: 256px          │
│ Content: 1184px         │
│ Grid: 4 cols            │
│ Viewport: 1440×900      │
└────────────────────────┘
```

### 5.4 Network Waterfall

Visual waterfall of all network requests:

```
GET /                    ████████░░░░  200  45ms
GET /api/kpisnapshots    ░░████░░░░░░  200  12ms
GET /api/deployments     ░░░████░░░░░  200  15ms
SSE /api/sse             ░░░░░████████ conn  ∞
```

---

## 6. Layer 4: Audit Debug

### 6.1 Audit Diff Viewer

When viewing the audit trail in /server, show the actual diff of what changed:

```
AUDIT #42 — UPDATE Deployment dep_001
┌─────────────┬───────────────┐
│ Before      │ After         │
├─────────────┼───────────────┤
│ status:     │ status:       │
│ "Rolling"   │ "Live"        │
│ duration:   │ duration:     │
│ "0s"        │ "28s"         │
└─────────────┴───────────────┘
Hash: abc123... → def456...
Chain: ✓ valid
```

### 6.2 Audit Timeline

Visual timeline of mutations:

```
12:00  ──●── INSERT Deployment DPL-5001
12:01  ──●── INSERT KpiSnapshot (overview)
12:02  ──●── UPDATE Deployment DPL-5001 (status: Rolling → Live)
12:03  ──●── DELETE Notification notif_001
12:05  ──●── INSERT SecurityEvent SEC-004
         ↓
       [now]
```

### 6.3 Hash Chain Verifier (Enhanced)

The existing `/api/audit/trail/verify` gets a visual output in the debug panel:

```
Chain Verification
══════════════════
Entry 1: ✓ genesis (no prev)
Entry 2: ✓ SHA-256 matches
Entry 3: ✓ SHA-256 matches
Entry 4: ✗ BROKEN — hash mismatch
          Expected: abc123...
          Computed: def456...
Entry 5: ✓ (chain restarted)
══════════════════
Result: BROKEN at entry #4
```

---

## 7. CLI Debug Commands

### 7.1 `cronus debug`

Starts the server with full debug mode enabled:

```bash
cronus debug          # Same as cronus run but with debug overlay + verbose logging
cronus debug --port 5175
cronus debug --trace-sql    # Log every SQL query to terminal
cronus debug --trace-sse    # Log every SSE event to terminal
cronus debug --trace-all    # Everything
```

### 7.2 `cronus debug audit`

Detailed audit trail analysis:

```bash
cronus debug audit                    # Show last 20 audit entries with diffs
cronus debug audit --entity Deployment # Filter by entity
cronus debug audit --verify           # Full chain verification with details
cronus debug audit --export json      # Export audit trail as JSON
```

### 7.3 `cronus debug requests`

Replay and analyze recent requests:

```bash
cronus debug requests                 # Show last 50 requests
cronus debug requests --slow          # Only requests > 100ms
cronus debug requests --errors        # Only 4xx/5xx
cronus debug requests --entity Task   # Only requests touching Task entity
```

---

## 8. Implementation Plan

### Phase 1: Debug Overlay (frontend) — HIGH IMPACT

1. Create debug panel JS in `render.rs` or new `debug.rs`
2. Keyboard shortcut Cmd+Shift+D to toggle
3. `?debug=1` URL param to auto-open
4. Network tab (fetch interceptor logging)
5. Error tab (global error/rejection handlers)
6. Info tab (page, sections, entities, auth, CSP)
7. `data-cronus-debug` attributes on rendered sections
8. Only injected when `CRONUS_DEBUG=1` env var or `cronus debug` command

### Phase 2: Backend Traces

1. `RequestTrace` struct in main.rs
2. SQL query logging in database.rs
3. Timing headers on all responses
4. `GET /api/debug/traces` endpoint
5. `GET /api/debug/queries` endpoint
6. `GET /api/debug/stream` SSE endpoint

### Phase 3: Audit Enhancement

1. Store before/after data in audit entries (diff)
2. Audit timeline visualization in /server
3. `cronus debug audit` CLI
4. Hash chain visual verifier in debug panel

### Phase 4: CLI Debug Mode

1. `cronus debug` command (run + debug flags)
2. `--trace-sql` terminal output
3. `--trace-sse` terminal output
4. Performance profiling summary on exit

---

## 9. Activation

Debug features are ONLY active when:
- `cronus debug` command is used (not `cronus run`)
- `CRONUS_DEBUG=1` environment variable is set
- `?debug=1` URL parameter is present

In production (`cronus run` without debug flag):
- No debug overlay JS is injected (zero overhead)
- No debug endpoints exist (security)
- No SQL tracing (performance)
- Audit trail still works (it's not debug, it's compliance)

---

## 10. Performance Budget

| Feature | Cost (dev mode) | Cost (production) |
|---------|----------------|-------------------|
| Debug overlay JS | ~3KB | 0 (not injected) |
| data-cronus-debug attrs | ~200 bytes/page | 0 |
| SQL tracing | +1ms/query | 0 |
| Request traces | +2ms/request | 0 |
| Timing headers | +0.1ms | 0 |
| Error handlers | ~500 bytes | ~500 bytes (always on) |
| Debug SSE stream | ~1KB/s | 0 |

**Total production overhead: ~500 bytes (error handlers only)**

---

## 11. Success Criteria

1. Developer finds a bug → opens debug panel → sees the error, which request caused it, which SQL query failed, and the audit trail of what changed
2. `cronus debug --trace-sql` shows every query in real-time in the terminal
3. Debug panel shows component tree with entity bindings and row counts
4. Frontend errors are caught and displayed with stack trace + page context
5. Audit diff shows exactly what changed in each mutation
6. Zero performance impact in production mode
