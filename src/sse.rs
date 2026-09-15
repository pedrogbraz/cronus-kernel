//! CRONUS SSE — Server-Sent Events for real-time updates
//!
//! Uses tokio::sync::broadcast to fan-out data change events
//! to all connected SSE clients.

use std::convert::Infallible;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_stream::stream;
use bytes::Bytes;
use http_body_util::StreamBody;
use hyper::body::Frame;
use hyper::{Response, StatusCode};
use serde_json::json;
use tokio::sync::broadcast;

/// An event representing a data change (entity mutation).
#[derive(Debug, Clone)]
pub struct DataChangeEvent {
    pub entity: String,
    pub action: String, // "created", "deleted", "updated"
    pub id: String,
}

/// A debug event broadcast alongside data_change events when DEBUG_MODE is active.
#[derive(Debug, Clone)]
pub struct DebugEvent {
    pub event_type: String, // "request", "error", etc.
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: u64,
    pub query_count: u64,
}

/// Shared SSE hub — holds the broadcast sender.
/// Uses tokio::sync::broadcast which automatically cleans up receivers when dropped.
/// The active_connections counter tracks live SSE streams for monitoring.
pub struct SseHub {
    tx: broadcast::Sender<DataChangeEvent>,
    debug_tx: broadcast::Sender<DebugEvent>,
    active_connections: Arc<AtomicUsize>,
}

/// RAII guard that decrements the active connection counter when dropped.
/// This ensures cleanup even if the SSE stream is abruptly terminated.
struct SseConnectionGuard(Arc<AtomicUsize>);

impl Drop for SseConnectionGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

impl SseHub {
    /// Create a new hub with a broadcast channel (capacity 256 events).
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        let (debug_tx, _) = broadcast::channel(256);
        SseHub {
            tx,
            debug_tx,
            active_connections: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Broadcast a data change event to all connected SSE clients.
    pub fn broadcast(&self, event: DataChangeEvent) {
        // Ignore send errors (no subscribers connected)
        let _ = self.tx.send(event);
    }

    /// Broadcast a debug event (only meaningful when DEBUG_MODE is active).
    pub fn broadcast_debug(&self, event: DebugEvent) {
        let _ = self.debug_tx.send(event);
    }

    /// Create an SSE response that streams events to the client.
    /// Returns a Response with `text/event-stream` content type.
    /// Includes a 30-second heartbeat to keep the connection alive.
    ///
    /// SECURITY: `filter` decides per event whether this subscriber may see
    /// it (see `access::can_see_event`); request-trace `debug` events are only
    /// streamed when `include_debug` is set (admins).
    pub fn subscribe_filtered<F>(
        &self,
        filter: F,
        include_debug: bool,
    ) -> Response<StreamBody<impl futures_core::Stream<Item = Result<Frame<Bytes>, Infallible>>>>
    where
        F: Fn(&DataChangeEvent) -> bool + Send + Sync + 'static,
    {
        let mut rx = self.tx.subscribe();
        let mut debug_rx = self.debug_tx.subscribe();
        let conn_counter = Arc::clone(&self.active_connections);
        conn_counter.fetch_add(1, Ordering::Relaxed);

        let body_stream = stream! {
            // Guard: decrement counter when stream ends (client disconnects)
            let _guard = SseConnectionGuard(Arc::clone(&conn_counter));

            // Initial connection message
            yield Ok::<Frame<Bytes>, Infallible>(Frame::data(Bytes::from(
                ": connected to CRONUS SSE\n\n"
            )));

            loop {
                tokio::select! {
                    result = rx.recv() => {
                        match result {
                            Ok(event) => {
                                if filter(&event) {
                                    let data = json!({
                                        "entity": event.entity,
                                        "action": event.action,
                                        "id": event.id,
                                    });
                                    let payload = format!(
                                        "event: data_change\ndata: {}\n\n",
                                        data.to_string()
                                    );
                                    yield Ok(Frame::data(Bytes::from(payload)));
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(n)) => {
                                let payload = format!(
                                    "event: warning\ndata: {{\"message\":\"missed {} events\"}}\n\n",
                                    n
                                );
                                yield Ok(Frame::data(Bytes::from(payload)));
                            }
                            Err(broadcast::error::RecvError::Closed) => {
                                break;
                            }
                        }
                    }
                    debug_result = debug_rx.recv() => {
                        match debug_result {
                            Ok(evt) if include_debug => {
                                let data = json!({
                                    "type": evt.event_type,
                                    "method": evt.method,
                                    "path": evt.path,
                                    "status": evt.status,
                                    "ms": evt.duration_ms,
                                    "queries": evt.query_count,
                                });
                                let payload = format!(
                                    "event: debug\ndata: {}\n\n",
                                    data.to_string()
                                );
                                yield Ok(Frame::data(Bytes::from(payload)));
                            }
                            Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => {}
                            Err(broadcast::error::RecvError::Closed) => {
                                break;
                            }
                        }
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
                        // Heartbeat comment to keep connection alive
                        yield Ok(Frame::data(Bytes::from(": heartbeat\n\n")));
                    }
                }
            }
        };

        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/event-stream")
            .header("cache-control", "no-cache")
            .header("connection", "keep-alive")
            .header("x-accel-buffering", "no");
        for (k, v) in cors_headers(&crate::server::response::cors_origin()) {
            builder = builder.header(k, v);
        }
        builder.body(StreamBody::new(body_stream)).unwrap()
    }
}

/// CORS headers for the SSE stream. The stream is authenticated by the session
/// cookie, so it is same-origin unless `CRONUS_CORS_ORIGIN` names an origin;
/// that origin is echoed with credentials. `*` is refused (a wildcard cannot
/// carry credentials and would expose the stream to any site).
fn cors_headers(origin: &str) -> Vec<(&'static str, String)> {
    let origin = origin.trim();
    if origin.is_empty() || origin == "same-origin" || origin == "*" {
        return Vec::new();
    }
    vec![
        ("access-control-allow-origin", origin.to_string()),
        ("access-control-allow-credentials", "true".to_string()),
        ("vary", "Origin".to_string()),
    ]
}

/// Client-side JS snippet for connecting to SSE and auto-refreshing.
/// Connects to /api/sse, listens for data_change events, and calls
/// cronusLiveReload() (from the CRONUS runtime) to refresh the page.
pub const SSE_CLIENT_JS: &str = r##"
// CRONUS SSE — Real-time data updates via Server-Sent Events
(function() {
  var connected = false;
  var retryMs = 3000;

  function connect() {
    var es = new EventSource('/api/sse');

    es.addEventListener('open', function() {
      connected = true;
      var dot = document.getElementById('cronus-sse-dot');
      if (dot) dot.style.background = '#22c55e';
      console.log('[CRONUS SSE] connected');
    });

    es.addEventListener('data_change', function(e) {
      try {
        var data = JSON.parse(e.data);
        console.log('[CRONUS SSE] ' + data.action + ': ' + data.entity + '/' + data.id);
        // Use the CRONUS runtime's live reload to refresh page content
        if (window.CRONUS && window.CRONUS.reload) {
          window.CRONUS.reload();
        }
      } catch(err) {}
    });

    es.addEventListener('open', function() { retryMs = 3000; });

    es.onerror = function() {
      connected = false;
      var dot = document.getElementById('cronus-sse-dot');
      if (dot) dot.style.background = '#ef4444';
      es.close();
      // /api/sse requires a session: without one, stop instead of retrying
      // forever; otherwise back off exponentially (3s → 60s).
      fetch('/api/auth/me', { credentials: 'same-origin' }).then(function(r) {
        if (r.status === 401) { console.log('[CRONUS SSE] no session, not reconnecting'); return; }
        console.log('[CRONUS SSE] disconnected, reconnecting in ' + (retryMs / 1000) + 's...');
        setTimeout(connect, retryMs);
        retryMs = Math.min(retryMs * 2, 60000);
      }).catch(function() {
        setTimeout(connect, retryMs);
        retryMs = Math.min(retryMs * 2, 60000);
      });
    };
  }

  // Create connection indicator
  var indicator = document.createElement('div');
  indicator.id = 'cronus-sse-indicator';
  indicator.innerHTML = '<span id="cronus-sse-dot" style="width:8px;height:8px;border-radius:50%;background:#71717a;display:inline-block;transition:background 0.3s"></span>';
  indicator.style.cssText = 'position:fixed;bottom:16px;right:16px;z-index:100;font-size:11px;color:#71717a;display:flex;align-items:center;gap:6px;font-family:Inter,system-ui,sans-serif;opacity:0.7';
  indicator.innerHTML += ' <span>SSE</span>';
  if (document.body) document.body.appendChild(indicator);
  else document.addEventListener('DOMContentLoaded', function() { document.body.appendChild(indicator); });

  connect();
})();
"##;

#[cfg(test)]
mod tests {
    use super::{cors_headers, SseHub, SSE_CLIENT_JS};

    /// The stream needs the session cookie, so a wildcard origin let any site
    /// probe it. Same-origin by default; a configured origin is echoed with
    /// credentials, never `*`.
    #[test]
    fn sse_has_no_wildcard_cors() {
        assert!(cors_headers("same-origin").is_empty());
        assert!(cors_headers("").is_empty());
        assert!(cors_headers("*").is_empty());
        let h = cors_headers("https://app.example.com");
        assert!(h.contains(&(
            "access-control-allow-origin",
            "https://app.example.com".to_string()
        )));
        assert!(h.contains(&("access-control-allow-credentials", "true".to_string())));
        assert!(h.contains(&("vary", "Origin".to_string())));

        if std::env::var_os("CRONUS_CORS_ORIGIN").is_none() {
            let resp = SseHub::new().subscribe_filtered(|_| true, false);
            assert!(resp.headers().get("access-control-allow-origin").is_none());
            assert_eq!(resp.headers()["content-type"], "text/event-stream");
        }
    }

    /// /api/sse returns 401 without a session; the client used to retry every
    /// 3s forever (hundreds of 401s per anonymous page view).
    #[test]
    fn client_stops_without_session_and_backs_off() {
        assert!(!SSE_CLIENT_JS.contains("setTimeout(connect, 3000)"));
        assert!(SSE_CLIENT_JS.contains("fetch('/api/auth/me'"));
        assert!(SSE_CLIENT_JS.contains("r.status === 401"));
        assert!(SSE_CLIENT_JS.contains("retryMs = Math.min(retryMs * 2, 60000)"));
    }
}
