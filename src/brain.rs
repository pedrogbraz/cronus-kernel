//! CRONUS Hydra Brain — Pattern Learning Engine
//!
//! Tracks usage patterns from HTTP requests and provides
//! suggestions for optimization based on observed behavior.

use crate::database::CronusDB;
use serde_json::{json, Value};
use std::sync::Arc;

pub struct CronusBrain {
    db: Arc<CronusDB>,
}

impl CronusBrain {
    /// Initialize the brain: create _brain_patterns table if not exists
    pub fn init(db: Arc<CronusDB>) -> Self {
        // Create tracking table directly via the internal connection
        let brain = CronusBrain { db };
        brain.ensure_tables();
        brain
    }

    fn ensure_tables(&self) {
        // Use the db's insert mechanism won't work for DDL, so we track in a simpler way
        // We'll use the CronusDB's migrate with a virtual entity, or just execute raw SQL
        // Since CronusDB wraps Connection in Mutex, we need to add a raw_execute or use a workaround
        // For now, we'll create a separate connection to the same db file
        // This is safe because SQLite handles concurrent access with WAL mode
    }

    /// Track an event with metadata
    pub fn track(&self, event: &str, metadata: &Value) {
        // Store in-memory for now (patterns vec)
        // In production this would go to _brain_patterns table
        let _ = self.db.insert(
            "_brain_events",
            &json!({
                "event": event,
                "metadata": metadata.to_string(),
                "timestamp": chrono_now(),
            }),
        );
    }

    /// Track an HTTP request
    pub fn track_request(&self, method: &str, path: &str, status: u16, duration_ms: u64) {
        let _ = self.db.insert(
            "_brain_events",
            &json!({
                "event": format!("{} {}", method, path),
                "metadata": json!({
                    "method": method,
                    "path": path,
                    "status": status,
                    "duration_ms": duration_ms,
                }).to_string(),
                "timestamp": chrono_now(),
            }),
        );
    }

    /// Get suggestions based on tracked patterns
    pub fn suggest(&self, _context: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Analyze event frequency
        if let Ok(events) = self.db.find_all("_brain_events", 1000, 0) {
            if let Some(arr) = events.as_array() {
                let mut freq: std::collections::HashMap<String, u32> =
                    std::collections::HashMap::new();
                let mut errors: u32 = 0;
                let mut slow: u32 = 0;

                for event in arr {
                    if let Some(e) = event.get("event").and_then(|v| v.as_str()) {
                        *freq.entry(e.to_string()).or_insert(0) += 1;
                    }
                    if let Some(meta_str) = event.get("metadata").and_then(|v| v.as_str()) {
                        if let Ok(meta) = serde_json::from_str::<Value>(meta_str) {
                            if let Some(status) = meta.get("status").and_then(|v| v.as_u64()) {
                                if status >= 500 {
                                    errors += 1;
                                }
                            }
                            if let Some(dur) = meta.get("duration_ms").and_then(|v| v.as_u64()) {
                                if dur > 100 {
                                    slow += 1;
                                }
                            }
                        }
                    }
                }

                // Find most/least used endpoints
                let mut sorted: Vec<_> = freq.iter().collect();
                sorted.sort_by(|a, b| b.1.cmp(a.1));

                if let Some((top, count)) = sorted.first() {
                    if **count > 10 {
                        suggestions.push(format!(
                            "Hot endpoint: {} ({} calls) — consider caching",
                            top, count
                        ));
                    }
                }

                if let Some((cold, count)) = sorted.last() {
                    if sorted.len() > 2 && **count <= 1 {
                        suggestions.push(format!(
                            "Cold endpoint: {} ({} calls) — consider removing",
                            cold, count
                        ));
                    }
                }

                if errors > 0 {
                    let pct = (errors as f64 / arr.len() as f64 * 100.0) as u32;
                    suggestions.push(format!(
                        "Error rate: {}% ({} errors / {} requests)",
                        pct,
                        errors,
                        arr.len()
                    ));
                }

                if slow > 0 {
                    suggestions.push(format!(
                        "Slow requests: {} (>100ms) — check database queries",
                        slow
                    ));
                }
            }
        }

        if suggestions.is_empty() {
            suggestions.push("No patterns detected yet. Keep using the app!".into());
        }

        suggestions
    }

    /// Return brain stats as JSON
    pub fn stats(&self) -> Value {
        let total_events = self.db.count("_brain_events").unwrap_or(0);
        let suggestions = self.suggest("");

        let mut freq: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        if let Ok(events) = self.db.find_all("_brain_events", 1000, 0) {
            if let Some(arr) = events.as_array() {
                for event in arr {
                    if let Some(e) = event.get("event").and_then(|v| v.as_str()) {
                        *freq.entry(e.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut top_endpoints: Vec<Value> = freq
            .iter()
            .map(|(k, v)| json!({"endpoint": k, "count": v}))
            .collect();
        top_endpoints.sort_by(|a, b| {
            b.get("count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .cmp(&a.get("count").and_then(|v| v.as_u64()).unwrap_or(0))
        });
        top_endpoints.truncate(10);

        json!({
            "total_events": total_events,
            "top_endpoints": top_endpoints,
            "suggestions": suggestions,
            "status": if total_events > 0 { "learning" } else { "idle" },
        })
    }
}

fn chrono_now() -> String {
    // Simple ISO timestamp without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Approximate ISO format
    let days = secs / 86400;
    let years = 1970 + days / 365;
    format!("{}-01-01T00:00:00Z", years) // Simplified; real impl would compute properly
}
