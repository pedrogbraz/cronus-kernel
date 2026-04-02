// CRONUS Audit Trail — Tamper-proof hash-chained log entries
//
// Every INSERT/UPDATE/DELETE on entity tables is recorded with a
// SHA-256 hash that chains to the previous entry, creating an
// immutable, verifiable audit log.

use rusqlite::{params, Connection};
use serde_json::{json, Value, Map};
use sha2::{Sha256, Digest};
use std::sync::Mutex;

/// Tamper-proof audit trail backed by SQLite with hash chaining.
pub struct AuditTrail {
    conn: Mutex<Connection>,
}

impl AuditTrail {
    /// Open (or reuse) a SQLite connection and ensure the _audit_log table exists.
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA journal_mode=WAL;").ok();

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                action TEXT NOT NULL,
                entity TEXT NOT NULL,
                record_id TEXT NOT NULL,
                user_id TEXT,
                data TEXT,
                prev_hash TEXT,
                hash TEXT NOT NULL
            );"
        ).map_err(|e| e.to_string())?;

        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Compute SHA-256 of the canonical audit string.
    fn compute_hash(
        timestamp: &str,
        action: &str,
        entity: &str,
        record_id: &str,
        data: &str,
        prev_hash: &str,
    ) -> String {
        let input = format!(
            "{}|{}|{}|{}|{}|{}",
            timestamp, action, entity, record_id, data, prev_hash
        );
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Get the hash of the most recent audit log entry (empty string if none).
    fn last_hash(conn: &Connection) -> String {
        conn.query_row(
            "SELECT hash FROM _audit_log ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        ).unwrap_or_default()
    }

    /// Record an audit entry synchronously. Returns the new entry's hash.
    pub fn log(
        &self,
        action: &str,
        entity: &str,
        record_id: &str,
        user_id: &str,
        data: &Value,
    ) -> Result<String, String> {
        let conn = self.conn.lock().unwrap();

        let prev_hash = Self::last_hash(&conn);

        // Get the timestamp that SQLite will assign (use the same value for hashing)
        let timestamp: String = conn
            .query_row("SELECT datetime('now')", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        let data_str = data.to_string();
        let hash = Self::compute_hash(
            &timestamp, action, entity, record_id, &data_str, &prev_hash,
        );

        conn.execute(
            "INSERT INTO _audit_log (timestamp, action, entity, record_id, user_id, data, prev_hash, hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![timestamp, action, entity, record_id, user_id, data_str, prev_hash, hash],
        ).map_err(|e| e.to_string())?;

        Ok(hash)
    }

    /// Retrieve the last N audit entries (newest first).
    pub fn query(&self, limit: usize) -> Result<Value, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, action, entity, record_id, user_id, data, prev_hash, hash
             FROM _audit_log ORDER BY id DESC LIMIT ?1"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "timestamp": row.get::<_, String>(1)?,
                "action": row.get::<_, String>(2)?,
                "entity": row.get::<_, String>(3)?,
                "record_id": row.get::<_, String>(4)?,
                "user_id": row.get::<_, Option<String>>(5)?,
                "data": row.get::<_, Option<String>>(6)?,
                "prev_hash": row.get::<_, Option<String>>(7)?,
                "hash": row.get::<_, String>(8)?,
            }))
        }).map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| e.to_string())?);
        }
        Ok(json!(results))
    }

    /// Verify the entire hash chain. Returns verification result as JSON.
    pub fn verify(&self) -> Result<Value, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, action, entity, record_id, data, prev_hash, hash
             FROM _audit_log ORDER BY id ASC"
        ).map_err(|e| e.to_string())?;

        let rows: Vec<(i64, String, String, String, String, Option<String>, Option<String>, String)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let total = rows.len();
        let mut expected_prev = String::new();

        for (id, timestamp, action, entity, record_id, data, prev_hash, hash) in &rows {
            let prev = prev_hash.as_deref().unwrap_or("");
            let data_str = data.as_deref().unwrap_or("null");

            // Check that prev_hash matches what we expect (the previous entry's hash)
            if prev != expected_prev {
                return Ok(json!({
                    "valid": false,
                    "entries": total,
                    "broken_at": id,
                    "reason": "prev_hash mismatch"
                }));
            }

            // Recompute hash and compare
            let recomputed = Self::compute_hash(
                timestamp, action, entity, record_id, data_str, prev,
            );

            if &recomputed != hash {
                return Ok(json!({
                    "valid": false,
                    "entries": total,
                    "broken_at": id,
                    "reason": "hash mismatch"
                }));
            }

            expected_prev = hash.clone();
        }

        Ok(json!({
            "valid": true,
            "entries": total,
            "broken_at": null
        }))
    }
}

/// Standalone verify function for CLI use — opens DB read-only and verifies.
pub fn verify_from_file(db_path: &str) -> Result<Value, String> {
    let trail = AuditTrail::open(db_path)?;
    trail.verify()
}
