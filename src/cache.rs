#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Cache — LRU in-memory cache with TTL
//!
//! Caches API responses by method+path key.
//! Auto-invalidates on POST/PUT/PATCH/DELETE.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Instant, Duration};

struct CacheEntry {
    value: String,
    created: Instant,
    ttl: Duration,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.created.elapsed() > self.ttl
    }
}

pub struct Cache {
    entries: Mutex<HashMap<String, CacheEntry>>,
    default_ttl: Duration,
    max_entries: usize,
}

impl Cache {
    pub fn new(ttl_secs: u64, max_entries: usize) -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            default_ttl: Duration::from_secs(ttl_secs),
            max_entries,
        }
    }

    /// Get a cached value. Returns None if missing or expired.
    pub fn get(&self, key: &str) -> Option<String> {
        let mut map = self.entries.lock().unwrap();
        if let Some(entry) = map.get(key) {
            if entry.is_expired() {
                map.remove(key);
                return None;
            }
            return Some(entry.value.clone());
        }
        None
    }

    /// Set a cached value with default TTL.
    pub fn set(&self, key: &str, value: &str) {
        self.set_with_ttl(key, value, self.default_ttl);
    }

    /// Set a cached value with custom TTL.
    pub fn set_with_ttl(&self, key: &str, value: &str, ttl: Duration) {
        let mut map = self.entries.lock().unwrap();
        // Evict expired entries if at capacity
        if map.len() >= self.max_entries {
            let expired: Vec<String> = map.iter()
                .filter(|(_, v)| v.is_expired())
                .map(|(k, _)| k.clone())
                .collect();
            for k in expired { map.remove(&k); }

            // If still at capacity, remove oldest
            if map.len() >= self.max_entries {
                if let Some(oldest) = map.iter()
                    .min_by_key(|(_, v)| v.created)
                    .map(|(k, _)| k.clone())
                {
                    map.remove(&oldest);
                }
            }
        }
        map.insert(key.to_string(), CacheEntry {
            value: value.to_string(),
            created: Instant::now(),
            ttl,
        });
    }

    /// Invalidate cache entries matching a prefix (e.g. "GET:/api/users")
    pub fn invalidate(&self, prefix: &str) {
        let mut map = self.entries.lock().unwrap();
        let keys: Vec<String> = map.keys()
            .filter(|k| k.contains(prefix))
            .cloned()
            .collect();
        for k in keys { map.remove(&k); }
    }

    /// Invalidate all entries for an entity (called on write operations)
    pub fn invalidate_entity(&self, entity: &str) {
        self.invalidate(&format!("/api/{}",  entity.to_lowercase()));
    }

    /// Clear all cache
    pub fn clear(&self) {
        let mut map = self.entries.lock().unwrap();
        map.clear();
    }

    /// Get cache stats
    pub fn stats(&self) -> (usize, usize) {
        let map = self.entries.lock().unwrap();
        let total = map.len();
        let expired = map.values().filter(|v| v.is_expired()).count();
        (total, total - expired)
    }

    /// Build cache key from method + path
    pub fn key(method: &str, path: &str) -> String {
        format!("{}:{}", method, path)
    }
}

/// Create default cache (60s TTL, 1000 max entries)
pub fn default_cache() -> Cache {
    Cache::new(60, 1000)
}
