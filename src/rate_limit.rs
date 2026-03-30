#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Rate Limiter — Sliding window counter per IP
//!
//! Token bucket algorithm with configurable limits.
//! Returns 429 Too Many Requests when exceeded.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

struct WindowEntry {
    timestamps: Vec<Instant>,
}

pub struct RateLimiter {
    windows: Mutex<HashMap<String, WindowEntry>>,
    max_requests: usize,
    window_secs: u64,
}

impl RateLimiter {
    /// Create a rate limiter with max requests per window.
    /// Default: 100 requests per 60 seconds.
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            windows: Mutex::new(HashMap::new()),
            max_requests,
            window_secs,
        }
    }

    /// Check if a request from this IP is allowed.
    /// Returns Ok(remaining) if allowed, Err(retry_after_secs) if rate limited.
    pub fn check(&self, ip: &str) -> Result<usize, u64> {
        let mut map = self.windows.lock().unwrap();
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);

        let entry = map.entry(ip.to_string()).or_insert_with(|| WindowEntry {
            timestamps: Vec::new(),
        });

        // Remove timestamps outside the window
        entry.timestamps.retain(|t| now.duration_since(*t) < window);

        if entry.timestamps.len() >= self.max_requests {
            // Calculate retry-after
            if let Some(oldest) = entry.timestamps.first() {
                let elapsed = now.duration_since(*oldest).as_secs();
                let retry_after = self.window_secs.saturating_sub(elapsed);
                return Err(retry_after.max(1));
            }
            return Err(self.window_secs);
        }

        // Allow the request
        entry.timestamps.push(now);
        let remaining = self.max_requests - entry.timestamps.len();
        Ok(remaining)
    }

    /// Get current request count for an IP
    pub fn count(&self, ip: &str) -> usize {
        let mut map = self.windows.lock().unwrap();
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);

        if let Some(entry) = map.get_mut(ip) {
            entry.timestamps.retain(|t| now.duration_since(*t) < window);
            entry.timestamps.len()
        } else {
            0
        }
    }

    /// Clean up expired entries (call periodically)
    pub fn cleanup(&self) {
        let mut map = self.windows.lock().unwrap();
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);

        map.retain(|_, entry| {
            entry.timestamps.retain(|t| now.duration_since(*t) < window);
            !entry.timestamps.is_empty()
        });
    }

    /// Get stats
    pub fn stats(&self) -> (usize, usize) {
        let map = self.windows.lock().unwrap();
        let ips = map.len();
        let total: usize = map.values().map(|e| e.timestamps.len()).sum();
        (ips, total)
    }
}

/// Create default rate limiter (100 req/min)
pub fn default_rate_limiter() -> RateLimiter {
    RateLimiter::new(100, 60)
}
