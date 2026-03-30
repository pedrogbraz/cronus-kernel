#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS SSE — Server-Sent Events for real-time updates
//!
//! Uses tokio::sync::broadcast to fan-out data change events
//! to all connected SSE clients.

use std::convert::Infallible;
use std::sync::Arc;

use async_stream::stream;
use bytes::Bytes;
use http_body_util::{BodyExt, StreamBody};
use hyper::body::Frame;
use hyper::{Response, StatusCode};
use serde_json::{json, Value};
use tokio::sync::broadcast;

/// An event representing a data change (entity mutation).
#[derive(Debug, Clone)]
pub struct DataChangeEvent {
    pub entity: String,
    pub action: String, // "created", "deleted", "updated"
    pub id: String,
}

/// Shared SSE hub — holds the broadcast sender.
pub struct SseHub {
    tx: broadcast::Sender<DataChangeEvent>,
}

impl SseHub {
    /// Create a new hub with a broadcast channel (capacity 256 events).
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        SseHub { tx }
    }

    /// Broadcast a data change event to all connected SSE clients.
    pub fn broadcast(&self, event: DataChangeEvent) {
        // Ignore send errors (no subscribers connected)
        let _ = self.tx.send(event);
    }

    /// Create an SSE response that streams events to the client.
    /// Returns a Response with `text/event-stream` content type.
    pub fn subscribe(&self) -> Response<StreamBody<impl futures_core::Stream<Item = Result<Frame<Bytes>, Infallible>>>> {
        let mut rx = self.tx.subscribe();

        let body_stream = stream! {
            // Initial connection message
            yield Ok::<Frame<Bytes>, Infallible>(Frame::data(Bytes::from(
                ": connected to CRONUS SSE\n\n"
            )));

            loop {
                match rx.recv().await {
                    Ok(event) => {
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
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        // Client fell behind, send a warning
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

                // Heartbeat every 30 seconds is handled by the client reconnect
                // but we can send a comment to keep connection alive
            }
        };

        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/event-stream")
            .header("cache-control", "no-cache")
            .header("connection", "keep-alive")
            .header("x-accel-buffering", "no")
            .header("access-control-allow-origin", "*")
            .body(StreamBody::new(body_stream))
            .unwrap()
    }
}

/// Client-side JS snippet for connecting to SSE and auto-refreshing data lists.
pub const SSE_CLIENT_JS: &str = r##"
// CRONUS SSE — Real-time data updates
(function() {
  var es = new EventSource('/api/events');

  es.addEventListener('data_change', function(e) {
    try {
      var data = JSON.parse(e.data);
      var entity = data.entity;
      var action = data.action;

      // Re-fetch any table/list showing this entity
      document.querySelectorAll('[data-entity="' + entity + '"]').forEach(function(el) {
        if (el._cronusRefresh) el._cronusRefresh();
      });

      // Re-fetch any element with data-list matching entity
      document.querySelectorAll('[data-list="' + entity + '"]').forEach(function(el) {
        if (el._cronusRefresh) el._cronusRefresh();
      });

      // Update count badges
      document.querySelectorAll('[data-count="' + entity + '"]').forEach(function(el) {
        if (el._cronusRefresh) el._cronusRefresh();
        else {
          // Simple: fetch count from API
          fetch('/api/' + entity + 's').then(function(r) { return r.json(); }).then(function(d) {
            el.textContent = Array.isArray(d) ? d.length : '?';
          }).catch(function() {});
        }
      });

      // Flash notification
      console.log('[CRONUS SSE] ' + action + ': ' + entity + '/' + data.id);
    } catch(err) {}
  });

  es.onerror = function() {
    console.log('[CRONUS SSE] reconnecting...');
  };
})();
"##;
