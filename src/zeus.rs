//! Zeus Mode — Deep observability for CRONUS
//!
//! Every request generates a trace with granular spans showing exactly
//! what happened: auth → script → db queries → render → response.
//!
//! Endpoints:
//!   GET /zeus     → HTML dashboard with recent traces
//!   GET /zeus/api → JSON API for traces

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Mutex;
use std::time::Instant;

// ── Span ──

#[derive(Debug, Clone, Serialize)]
pub struct Span {
    pub name: String,
    pub kind: SpanKind,
    pub start_us: u64,      // microseconds from request start
    pub duration_us: u64,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum SpanKind {
    Auth,
    Db,
    Script,
    Render,
    Http,
    Other,
}

// ── Trace ──

#[derive(Debug, Clone, Serialize)]
pub struct ZeusTrace {
    pub id: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: f64,
    pub timestamp: String,
    pub spans: Vec<Span>,
    pub query_count: u64,
    pub script_block: Option<String>,
}

// ── Trace Builder (per-request) ──

pub struct TraceBuilder {
    pub id: String,
    pub method: String,
    pub path: String,
    start: Instant,
    spans: Vec<Span>,
}

impl TraceBuilder {
    pub fn new(id: &str, method: &str, path: &str) -> Self {
        Self {
            id: id.to_string(),
            method: method.to_string(),
            path: path.to_string(),
            start: Instant::now(),
            spans: Vec::new(),
        }
    }

    /// Start a span, returns a SpanGuard that auto-closes on drop
    pub fn span(&mut self, name: &str, kind: SpanKind) -> SpanHandle {
        let start_us = self.start.elapsed().as_micros() as u64;
        SpanHandle {
            name: name.to_string(),
            kind,
            start_us,
            start_instant: Instant::now(),
            metadata: Value::Null,
        }
    }

    /// Record a completed span
    pub fn record(&mut self, handle: SpanHandle) {
        self.spans.push(Span {
            name: handle.name,
            kind: handle.kind,
            start_us: handle.start_us,
            duration_us: handle.start_instant.elapsed().as_micros() as u64,
            metadata: handle.metadata,
        });
    }

    /// Quick span (already completed)
    pub fn add_span(&mut self, name: &str, kind: SpanKind, duration_us: u64, metadata: Value) {
        let start_us = self.start.elapsed().as_micros() as u64 - duration_us;
        self.spans.push(Span {
            name: name.to_string(),
            kind,
            start_us,
            duration_us,
            metadata,
        });
    }

    /// Finalize into a ZeusTrace
    pub fn finish(self, status: u16, query_count: u64) -> ZeusTrace {
        let duration_ms = self.start.elapsed().as_secs_f64() * 1000.0;
        let timestamp = crate::server::state::iso_timestamp();
        ZeusTrace {
            id: self.id,
            method: self.method,
            path: self.path,
            status,
            duration_ms,
            timestamp,
            spans: self.spans,
            query_count,
            script_block: None,
        }
    }
}

/// Handle for an in-progress span
pub struct SpanHandle {
    pub name: String,
    pub kind: SpanKind,
    pub start_us: u64,
    start_instant: Instant,
    pub metadata: Value,
}

impl SpanHandle {
    pub fn set_metadata(&mut self, meta: Value) {
        self.metadata = meta;
    }
}

// ── Zeus Buffer (stores recent traces) ──

pub struct ZeusBuffer {
    traces: Mutex<Vec<ZeusTrace>>,
    max_size: usize,
}

impl ZeusBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            traces: Mutex::new(Vec::with_capacity(max_size)),
            max_size,
        }
    }

    pub fn push(&self, trace: ZeusTrace) {
        if let Ok(mut buf) = self.traces.lock() {
            if buf.len() >= self.max_size {
                buf.remove(0);
            }
            buf.push(trace);
        }
    }

    pub fn last_n(&self, n: usize) -> Vec<ZeusTrace> {
        self.traces.lock().ok()
            .map(|buf| {
                let start = buf.len().saturating_sub(n);
                buf[start..].to_vec()
            })
            .unwrap_or_default()
    }

    pub fn slow_traces(&self, threshold_ms: f64) -> Vec<ZeusTrace> {
        self.traces.lock().ok()
            .map(|buf| {
                buf.iter()
                    .filter(|t| t.duration_ms > threshold_ms)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn error_traces(&self) -> Vec<ZeusTrace> {
        self.traces.lock().ok()
            .map(|buf| {
                buf.iter()
                    .filter(|t| t.status >= 400)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn stats(&self) -> Value {
        let traces = self.last_n(self.max_size);
        if traces.is_empty() {
            return json!({"total": 0});
        }
        let total = traces.len();
        let errors = traces.iter().filter(|t| t.status >= 400).count();
        let avg_ms: f64 = traces.iter().map(|t| t.duration_ms).sum::<f64>() / total as f64;
        let max_ms = traces.iter().map(|t| t.duration_ms).fold(0.0_f64, f64::max);
        let p95_idx = (total as f64 * 0.95) as usize;
        let mut durations: Vec<f64> = traces.iter().map(|t| t.duration_ms).collect();
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95 = durations.get(p95_idx).copied().unwrap_or(0.0);

        json!({
            "total": total,
            "errors": errors,
            "error_rate": format!("{:.2}%", errors as f64 / total as f64 * 100.0),
            "avg_ms": format!("{:.2}", avg_ms),
            "p95_ms": format!("{:.2}", p95),
            "max_ms": format!("{:.2}", max_ms),
        })
    }
}

// ── Zeus Dashboard HTML ──

pub fn render_dashboard(buffer: &ZeusBuffer) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let traces = buffer.last_n(50);
    let stats = buffer.stats();

    let mut rows = String::new();
    for t in traces.iter().rev() {
        let status_color = if t.status < 300 { "#10b981" } else if t.status < 400 { "#eab308" } else { "#ef4444" };
        let method_color = match t.method.as_str() {
            "GET" => "#10b981",
            "POST" => "#adc6ff",
            "PATCH" | "PUT" => "#eab308",
            "DELETE" => "#ef4444",
            _ => "#757575",
        };
        let span_tags: Vec<String> = t.spans.iter().map(|s| {
            let (color, label) = match s.kind {
                SpanKind::Auth => ("#c2c1ff", "auth"),
                SpanKind::Db => ("#eab308", "db"),
                SpanKind::Script => ("#e9b3ff", "script"),
                SpanKind::Render => ("#adc6ff", "render"),
                SpanKind::Http => ("#10b981", "http"),
                SpanKind::Other => ("#757575", "other"),
            };
            format!(
                "<span class=\"span-tag\" style=\"--tag-color:{}\">{} {:.0}us</span>",
                color, label, s.duration_us
            )
        }).collect();

        let duration_class = if t.duration_ms > 100.0 { "dur-slow" } else if t.duration_ms > 50.0 { "dur-warn" } else { "dur-ok" };

        rows.push_str(&format!(
            r#"<tr>
                <td class="cell cell-time">{}</td>
                <td class="cell"><span class="method-badge" style="color:{}">{}</span></td>
                <td class="cell cell-path">{}</td>
                <td class="cell cell-center"><span class="status" style="color:{}">{}</span></td>
                <td class="cell cell-right {}"><span>{:.2}ms</span></td>
                <td class="cell cell-right cell-dim">{}</td>
                <td class="cell cell-spans">{}</td>
            </tr>"#,
            t.timestamp, method_color, t.method, t.path,
            status_color, t.status,
            duration_class, t.duration_ms, t.query_count,
            span_tags.join(" ")
        ));
    }

    format!(r##"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Zeus — CRONUS Observability</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;900&family=Space+Grotesk:wght@500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet">
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200" rel="stylesheet">
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
:root{{
  --bg:#0a0a0a;--surface:rgba(25,25,25,0.8);--card:rgba(20,20,20,0.9);
  --border:rgba(255,255,255,0.04);--border-accent:rgba(135,173,255,0.15);
  --text:#e2e2e2;--text-muted:#757575;--text-dim:#484848;
  --primary:#adc6ff;--secondary:#c2c1ff;--tertiary:#e9b3ff;
  --success:#10b981;--danger:#ef4444;--warning:#eab308;
  --radius:12px;
}}
body{{background:var(--bg);color:var(--text);font-family:'Inter',system-ui,-apple-system,sans-serif;-webkit-font-smoothing:antialiased;min-height:100vh}}
::selection{{background:rgba(135,173,255,0.2)}}

.shell{{max-width:1440px;margin:0 auto;padding:40px 32px}}
.topbar{{display:flex;justify-content:space-between;align-items:flex-end;margin-bottom:40px;padding-bottom:20px;border-bottom:1px solid var(--border)}}
.topbar h1{{font-family:'Space Grotesk',sans-serif;font-size:22px;font-weight:700;letter-spacing:-0.03em;color:#fff}}
.topbar h1 span{{color:var(--primary)}}
.topbar p{{color:var(--text-muted);font-size:13px;margin-top:4px}}

.nav-links{{display:flex;gap:16px}}
.nav-links a{{color:var(--text-muted);font-size:12px;text-decoration:none;padding:6px 12px;border-radius:8px;background:rgba(255,255,255,0.03);border:1px solid var(--border);transition:all 0.15s}}
.nav-links a:hover{{color:#fff;border-color:rgba(255,255,255,0.1)}}

/* ── KPIs ── */
.kpis{{display:grid;grid-template-columns:repeat(6,1fr);gap:12px;margin-bottom:32px}}
.kpi{{background:var(--surface);backdrop-filter:blur(40px);border:1px solid var(--border);border-top:0.5px solid var(--border-accent);border-radius:var(--radius);padding:20px 24px}}
.kpi-val{{font-family:'Space Grotesk',sans-serif;font-size:26px;font-weight:700;letter-spacing:-0.03em}}
.kpi-label{{font-size:11px;font-weight:500;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;margin-top:4px}}

/* ── Table ── */
.table-wrap{{background:var(--surface);backdrop-filter:blur(40px);border:1px solid var(--border);border-top:0.5px solid var(--border-accent);border-radius:var(--radius);overflow:hidden}}
.table-header{{padding:16px 24px;border-bottom:1px solid var(--border);display:flex;justify-content:space-between;align-items:center}}
.table-header h2{{font-family:'Space Grotesk',sans-serif;font-size:15px;font-weight:600;color:#fff}}
.table-header span{{color:var(--text-dim);font-size:11px}}
.live-dot{{display:inline-block;width:6px;height:6px;border-radius:50%;background:var(--success);margin-right:6px;animation:pulse 2s infinite}}
@keyframes pulse{{0%,100%{{opacity:1}}50%{{opacity:0.4}}}}

table{{width:100%;border-collapse:collapse}}
thead th{{padding:10px 16px;text-align:left;font-size:10px;font-weight:600;color:var(--text-dim);text-transform:uppercase;letter-spacing:0.06em;border-bottom:1px solid var(--border)}}
thead th.r{{text-align:right}}
thead th.c{{text-align:center}}

.cell{{padding:10px 16px;border-bottom:1px solid rgba(255,255,255,0.02);font-size:12px;vertical-align:middle}}
.cell-time{{font-family:'JetBrains Mono',monospace;font-size:11px;color:var(--text-dim)}}
.cell-path{{font-family:'JetBrains Mono',monospace;font-size:12px;color:var(--text);max-width:320px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}}
.cell-center{{text-align:center}}
.cell-right{{text-align:right;font-family:'JetBrains Mono',monospace;font-size:11px}}
.cell-dim{{color:var(--text-dim)}}
.cell-spans{{display:flex;gap:4px;flex-wrap:wrap}}

tr:hover .cell{{background:rgba(255,255,255,0.02)}}

.method-badge{{font-family:'JetBrains Mono',monospace;font-size:11px;font-weight:600;padding:2px 8px;background:rgba(255,255,255,0.03);border-radius:6px}}
.status{{font-weight:700;font-size:12px}}

.dur-ok span{{color:var(--text-muted)}}
.dur-warn span{{color:var(--warning)}}
.dur-slow span{{color:var(--danger)}}

.span-tag{{font-family:'JetBrains Mono',monospace;font-size:10px;font-weight:500;padding:2px 8px;border-radius:6px;background:color-mix(in srgb,var(--tag-color) 12%,transparent);color:var(--tag-color)}}

.footer{{margin-top:20px;padding-top:16px;border-top:1px solid var(--border);display:flex;justify-content:center;gap:16px}}
.footer a{{color:var(--text-dim);font-size:11px;text-decoration:none;transition:color 0.15s}}
.footer a:hover{{color:var(--primary)}}
</style>
</head><body>
<div class="shell">
  <div class="topbar">
    <div>
      <h1><span>Zeus</span> Observability</h1>
      <p>Real-time request tracing and performance monitoring</p>
    </div>
    <div class="nav-links">
      <a href="/blocks">Blocks</a>
      <a href="/trust">Trust</a>
      <a href="/docs">Docs</a>
    </div>
  </div>

  <div class="kpis">
    <div class="kpi"><div class="kpi-val" style="color:#fff">{total}</div><div class="kpi-label">Total</div></div>
    <div class="kpi"><div class="kpi-val" style="color:var(--danger)">{errors}</div><div class="kpi-label">Errors</div></div>
    <div class="kpi"><div class="kpi-val">{error_rate}</div><div class="kpi-label">Error Rate</div></div>
    <div class="kpi"><div class="kpi-val">{avg_ms}<span style="font-size:14px;color:var(--text-muted)">ms</span></div><div class="kpi-label">Avg Latency</div></div>
    <div class="kpi"><div class="kpi-val">{p95_ms}<span style="font-size:14px;color:var(--text-muted)">ms</span></div><div class="kpi-label">p95</div></div>
    <div class="kpi"><div class="kpi-val" style="color:var(--warning)">{max_ms}<span style="font-size:14px;color:var(--text-muted)">ms</span></div><div class="kpi-label">Max</div></div>
  </div>

  <div class="table-wrap">
    <div class="table-header">
      <h2><span class="material-symbols-outlined" style="font-size:16px;vertical-align:-3px;margin-right:6px;color:var(--primary)">monitoring</span>Recent Requests</h2>
      <span><span class="live-dot"></span>Live &middot; Last 50 traces</span>
    </div>
    <table>
      <thead>
        <tr>
          <th>Time</th>
          <th>Method</th>
          <th>Path</th>
          <th class="c">Status</th>
          <th class="r">Duration</th>
          <th class="r">Queries</th>
          <th>Spans</th>
        </tr>
      </thead>
      <tbody>{rows}</tbody>
    </table>
  </div>

  <div class="footer">
    <a href="/zeus/api">JSON API</a>
    <a href="/zeus/slow">Slow Traces</a>
    <a href="/zeus/errors">Error Traces</a>
    <a href="/trust">Trust Engine</a>
  </div>
</div>
<script{script_nonce}>setTimeout(function(){{location.reload()}},5000)</script>
</body></html>"##,
        total = stats["total"],
        errors = stats["errors"],
        error_rate = stats.get("error_rate").and_then(|v| v.as_str()).unwrap_or("0%"),
        avg_ms = stats.get("avg_ms").and_then(|v| v.as_str()).unwrap_or("0"),
        p95_ms = stats.get("p95_ms").and_then(|v| v.as_str()).unwrap_or("0"),
        max_ms = stats.get("max_ms").and_then(|v| v.as_str()).unwrap_or("0"),
        rows = rows,
    )
}
