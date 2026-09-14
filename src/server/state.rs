//! AppState and related types extracted from main.rs.

use std::sync::Arc;

use crate::audit;
use crate::brain;
use crate::database;
use crate::parser::{self, ApiNode, AppNode, EntityNode, PageNode, StyleNode};
use crate::rate_limit;
use crate::sse;

// ──────────────────────────────────────────────
// Request tracing
// ──────────────────────────────────────────────

#[derive(Clone, serde::Serialize)]
pub(crate) struct RequestTrace {
    pub(crate) id: String,
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) status: u16,
    pub(crate) duration_ms: u64,
    pub(crate) query_count: u64,
    pub(crate) timestamp: String,
}

pub(crate) struct TraceBuffer {
    pub(crate) traces: std::sync::Mutex<Vec<RequestTrace>>,
}

impl TraceBuffer {
    pub(crate) fn new() -> Self {
        Self {
            traces: std::sync::Mutex::new(Vec::with_capacity(200)),
        }
    }
    pub(crate) fn push(&self, trace: RequestTrace) {
        let mut buf = self.traces.lock().unwrap_or_else(|e| e.into_inner());
        if buf.len() >= 200 {
            buf.remove(0);
        }
        buf.push(trace);
    }
    pub(crate) fn last_n(&self, n: usize) -> Vec<RequestTrace> {
        let buf = self.traces.lock().unwrap_or_else(|e| e.into_inner());
        let start = buf.len().saturating_sub(n);
        buf[start..].to_vec()
    }
}

pub(crate) fn generate_request_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("req_{:012x}", nanos & 0xFFFF_FFFF_FFFF)
}

pub(crate) fn current_time_hms() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

pub(crate) fn iso_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("T{:02}:{:02}:{:02}Z", h, m, s)
}

// ──────────────────────────────────────────────
// AppState
// ──────────────────────────────────────────────

pub(crate) struct AppState {
    pub(crate) app: AppNode,
    pub(crate) entities: Vec<EntityNode>,
    pub(crate) pages: Vec<PageNode>,
    pub(crate) components: Vec<parser::ComponentNode>,
    pub(crate) style: Option<StyleNode>,
    pub(crate) apis: Vec<ApiNode>,
    pub(crate) db_path: String,
    pub(crate) db: database::CronusDB,
    pub(crate) brain: Option<brain::CronusBrain>,
    pub(crate) auth_entity: Option<String>,
    pub(crate) auth_roles: Vec<String>,
    pub(crate) auth_required_pages: Vec<(String, String)>,
    /// `auth { redirect "/" }` — post-login href. Empty = pick `/` then `/dashboard`.
    pub(crate) auth_redirect: Option<String>,
    /// Token + cookie lifetime from `auth { session jwt expires:<dur> }`.
    pub(crate) session_policy: crate::auth::SessionPolicy,
    pub(crate) layout: Option<parser::LayoutNode>,
    pub(crate) webhooks: Vec<parser::WebhookNode>,
    pub(crate) rate_limiter: rate_limit::RateLimiter,
    pub(crate) auth_rate_limiter: rate_limit::RateLimiter,
    pub(crate) sse_hub: Arc<sse::SseHub>,
    pub(crate) audit_trail: audit::AuditTrail,
    pub(crate) trace_buffer: Arc<TraceBuffer>,
    pub(crate) script_registry: crate::scripting::ScriptRegistry,
    pub(crate) zeus: Arc<crate::zeus::ZeusBuffer>,
}
