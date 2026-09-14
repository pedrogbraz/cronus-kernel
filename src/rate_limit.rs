//! CRONUS Rate Limiter — Sliding window counter per key (client IP).
//!
//! Returns 429 Too Many Requests when exceeded. Keys come from
//! `http_guard::client_ip` (socket peer unless behind a trusted proxy).
//! The key map is bounded and cleaned periodically by `cmd_run`.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

/// Upper bound on tracked keys; beyond it expired keys are pruned and then
/// the least recently used key is evicted.
pub const MAX_KEYS: usize = 50_000;

struct WindowEntry {
    timestamps: Vec<Instant>,
}

pub struct RateLimiter {
    windows: Mutex<HashMap<String, WindowEntry>>,
    max_requests: usize,
    window_secs: u64,
    max_keys: usize,
}

impl RateLimiter {
    /// Create a rate limiter with max requests per window.
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self::with_max_keys(max_requests, window_secs, MAX_KEYS)
    }

    pub fn with_max_keys(max_requests: usize, window_secs: u64, max_keys: usize) -> Self {
        Self {
            windows: Mutex::new(HashMap::new()),
            max_requests,
            window_secs,
            max_keys: max_keys.max(1),
        }
    }

    fn map(&self) -> MutexGuard<'_, HashMap<String, WindowEntry>> {
        self.windows.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn window(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.window_secs)
    }

    /// Check if a request from this key is allowed.
    /// Returns Ok(remaining) if allowed, Err(retry_after_secs) if rate limited.
    pub fn check(&self, ip: &str) -> Result<usize, u64> {
        let mut map = self.map();
        let now = Instant::now();
        let window = self.window();

        if !map.contains_key(ip) && map.len() >= self.max_keys {
            map.retain(|_, e| {
                e.timestamps.retain(|t| now.duration_since(*t) < window);
                !e.timestamps.is_empty()
            });
            if map.len() >= self.max_keys {
                let victim = map
                    .iter()
                    .min_by_key(|(_, e)| e.timestamps.last().copied())
                    .map(|(k, _)| k.clone());
                if let Some(v) = victim {
                    map.remove(&v);
                }
            }
        }

        let entry = map.entry(ip.to_string()).or_insert_with(|| WindowEntry {
            timestamps: Vec::new(),
        });

        entry.timestamps.retain(|t| now.duration_since(*t) < window);

        if entry.timestamps.len() >= self.max_requests {
            if let Some(oldest) = entry.timestamps.first() {
                let elapsed = now.duration_since(*oldest).as_secs();
                let retry_after = self.window_secs.saturating_sub(elapsed);
                return Err(retry_after.max(1));
            }
            return Err(self.window_secs);
        }

        entry.timestamps.push(now);
        Ok(self.max_requests - entry.timestamps.len())
    }

    /// Clean up expired entries (called periodically by the server).
    pub fn cleanup(&self) {
        let mut map = self.map();
        let now = Instant::now();
        let window = self.window();
        map.retain(|_, entry| {
            entry.timestamps.retain(|t| now.duration_since(*t) < window);
            !entry.timestamps.is_empty()
        });
    }

    /// (tracked keys, total timestamps)
    #[cfg(test)]
    pub fn stats(&self) -> (usize, usize) {
        let map = self.map();
        (map.len(), map.values().map(|e| e.timestamps.len()).sum())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_after_max_requests() {
        let l = RateLimiter::new(3, 60);
        assert_eq!(l.check("a"), Ok(2));
        assert_eq!(l.check("a"), Ok(1));
        assert_eq!(l.check("a"), Ok(0));
        assert!(l.check("a").is_err());
        assert_eq!(l.check("b"), Ok(2), "separate keys");
    }

    #[test]
    fn key_map_is_bounded() {
        let l = RateLimiter::with_max_keys(5, 60, 4);
        for i in 0..100 {
            let _ = l.check(&format!("10.0.0.{}", i));
        }
        assert!(l.stats().0 <= 4);
    }

    #[test]
    fn poisoned_lock_does_not_panic() {
        let l = RateLimiter::new(5, 60);
        let _ = std::thread::scope(|s| {
            s.spawn(|| {
                let _g = l.windows.lock().unwrap();
                panic!("poison");
            })
            .join()
        });
        assert!(l.windows.is_poisoned());
        assert!(l.check("a").is_ok());
        l.cleanup();
    }
}
