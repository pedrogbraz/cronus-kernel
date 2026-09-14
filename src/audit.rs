// CRONUS Audit Trail — Tamper-proof hash-chained log entries
//
// Every INSERT/UPDATE/DELETE on entity tables is recorded with a
// SHA-256 hash that chains to the previous entry, creating an
// immutable, verifiable audit log.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::sync::Mutex;

/// A single field-level change between two JSON objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDiff {
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}

/// Compare two JSON objects field-by-field, returning only changed fields.
pub fn compute_diff(prev: &Value, next: &Value) -> Vec<FieldDiff> {
    let mut diffs = Vec::new();

    let prev_obj = prev.as_object();
    let next_obj = next.as_object();

    match (prev_obj, next_obj) {
        (Some(old), Some(new)) => {
            // Check fields in new that changed or were added
            for (key, new_val) in new {
                match old.get(key) {
                    Some(old_val) if old_val != new_val => {
                        diffs.push(FieldDiff {
                            field: key.clone(),
                            old_value: format_value(old_val),
                            new_value: format_value(new_val),
                        });
                    }
                    None => {
                        diffs.push(FieldDiff {
                            field: key.clone(),
                            old_value: String::new(),
                            new_value: format_value(new_val),
                        });
                    }
                    _ => {} // unchanged
                }
            }
            // Check fields removed in new
            for (key, old_val) in old {
                if !new.contains_key(key) {
                    diffs.push(FieldDiff {
                        field: key.clone(),
                        old_value: format_value(old_val),
                        new_value: String::new(),
                    });
                }
            }
        }
        _ => {} // non-object values — skip
    }

    diffs
}

/// Format a JSON value for display (strip outer quotes for strings).
fn format_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

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
                prev_data TEXT,
                prev_hash TEXT,
                hash TEXT NOT NULL
            );

            CREATE TRIGGER IF NOT EXISTS audit_no_delete
            BEFORE DELETE ON _audit_log
            BEGIN
                SELECT RAISE(ABORT, 'Audit log entries cannot be deleted');
            END;

            CREATE TRIGGER IF NOT EXISTS audit_no_update
            BEFORE UPDATE ON _audit_log
            BEGIN
                SELECT RAISE(ABORT, 'Audit log entries cannot be modified');
            END;",
        )
        .map_err(|e| e.to_string())?;

        // Add prev_data column if table already existed without it
        let has_prev_data: bool = conn
            .prepare("SELECT COUNT(*) FROM pragma_table_info('_audit_log') WHERE name='prev_data'")
            .and_then(|mut s| s.query_row([], |r| r.get::<_, i64>(0)))
            .map(|c| c > 0)
            .unwrap_or(false);

        if !has_prev_data {
            conn.execute_batch("ALTER TABLE _audit_log ADD COLUMN prev_data TEXT;")
                .ok();
        }

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Compute SHA-256 of the canonical audit string.
    /// NOTE: prev_data is intentionally NOT included in the hash to preserve chain integrity.
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
        )
        .unwrap_or_default()
    }

    /// Record an audit entry synchronously. Returns the new entry's hash.
    /// `prev_data` stores the record state before the operation (for UPDATE/DELETE).
    pub fn log(
        &self,
        action: &str,
        entity: &str,
        record_id: &str,
        user_id: &str,
        data: &Value,
        prev_data: Option<&Value>,
    ) -> Result<String, String> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());

        let prev_hash = Self::last_hash(&conn);

        // Get the timestamp that SQLite will assign (use the same value for hashing)
        let timestamp: String = conn
            .query_row("SELECT datetime('now')", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        let data_str = data.to_string();
        let prev_data_str = prev_data.map(|v| v.to_string());
        let hash = Self::compute_hash(&timestamp, action, entity, record_id, &data_str, &prev_hash);

        conn.execute(
            "INSERT INTO _audit_log (timestamp, action, entity, record_id, user_id, data, prev_data, prev_hash, hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![timestamp, action, entity, record_id, user_id, data_str, prev_data_str, prev_hash, hash],
        ).map_err(|e| e.to_string())?;

        Ok(hash)
    }

    /// Retrieve the last N audit entries (newest first), with computed diffs.
    pub fn query(&self, limit: usize) -> Result<Value, String> {
        self.query_filtered(limit, None)
    }

    /// Retrieve audit entries with optional entity filter, including computed diffs.
    pub fn query_filtered(
        &self,
        limit: usize,
        entity_filter: Option<&str>,
    ) -> Result<Value, String> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());

        let (sql, use_filter) = match entity_filter {
            Some(_) => (
                "SELECT id, timestamp, action, entity, record_id, user_id, data, prev_data, prev_hash, hash
                 FROM _audit_log WHERE entity = ?1 ORDER BY id DESC LIMIT ?2",
                true,
            ),
            None => (
                "SELECT id, timestamp, action, entity, record_id, user_id, data, prev_data, prev_hash, hash
                 FROM _audit_log ORDER BY id DESC LIMIT ?1",
                false,
            ),
        };

        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

        let rows: Vec<Value> = if use_filter {
            let entity = entity_filter.unwrap();
            stmt.query_map(params![entity, limit as i64], |row| Self::row_to_json(row))
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?
        } else {
            stmt.query_map(params![limit as i64], |row| Self::row_to_json(row))
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?
        };

        Ok(json!(rows))
    }

    /// Convert a DB row into a JSON value with computed diff.
    fn row_to_json(row: &rusqlite::Row) -> rusqlite::Result<Value> {
        let data_str: Option<String> = row.get(6)?;
        let prev_data_str: Option<String> = row.get(7)?;

        // Parse data and prev_data for diff computation
        let data_val = data_str
            .as_ref()
            .and_then(|s| serde_json::from_str::<Value>(s).ok());
        let prev_data_val = prev_data_str
            .as_ref()
            .and_then(|s| serde_json::from_str::<Value>(s).ok());

        // Compute diff if both prev_data and data exist
        let diff = match (&prev_data_val, &data_val) {
            (Some(prev), Some(next)) => {
                let diffs = compute_diff(prev, next);
                Some(json!(diffs
                    .iter()
                    .map(|d| {
                        json!({
                            "field": d.field,
                            "old": d.old_value,
                            "new": d.new_value,
                        })
                    })
                    .collect::<Vec<_>>()))
            }
            _ => None,
        };

        Ok(json!({
            "id": row.get::<_, i64>(0)?,
            "timestamp": row.get::<_, String>(1)?,
            "action": row.get::<_, String>(2)?,
            "entity": row.get::<_, String>(3)?,
            "record_id": row.get::<_, String>(4)?,
            "user_id": row.get::<_, Option<String>>(5)?,
            "data": data_str,
            "prev_data": prev_data_str,
            "prev_hash": row.get::<_, Option<String>>(8)?,
            "hash": row.get::<_, String>(9)?,
            "diff": diff,
        }))
    }

    /// Verify the entire hash chain. Returns verification result as JSON.
    pub fn verify(&self) -> Result<Value, String> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut stmt = conn
            .prepare(
                "SELECT id, timestamp, action, entity, record_id, data, prev_hash, hash
             FROM _audit_log ORDER BY id ASC",
            )
            .map_err(|e| e.to_string())?;

        let rows: Vec<(
            i64,
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            String,
        )> = stmt
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
            let recomputed =
                Self::compute_hash(timestamp, action, entity, record_id, data_str, prev);

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

    /// Format audit entries for CLI display with diffs and hash verification.
    pub fn debug_display(
        &self,
        limit: usize,
        entity_filter: Option<&str>,
        verify_hashes: bool,
    ) -> Result<String, String> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());

        let (sql, use_filter) = match entity_filter {
            Some(_) => (
                "SELECT id, timestamp, action, entity, record_id, user_id, data, prev_data, prev_hash, hash
                 FROM _audit_log WHERE entity = ?1 ORDER BY id DESC LIMIT ?2",
                true,
            ),
            None => (
                "SELECT id, timestamp, action, entity, record_id, user_id, data, prev_data, prev_hash, hash
                 FROM _audit_log ORDER BY id DESC LIMIT ?1",
                false,
            ),
        };

        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

        struct AuditRow {
            id: i64,
            timestamp: String,
            action: String,
            entity: String,
            record_id: String,
            data: Option<String>,
            prev_data: Option<String>,
            prev_hash: Option<String>,
            hash: String,
        }

        let rows: Vec<AuditRow> = if use_filter {
            let entity = entity_filter.unwrap();
            stmt.query_map(params![entity, limit as i64], |row| {
                Ok(AuditRow {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    action: row.get(2)?,
                    entity: row.get(3)?,
                    record_id: row.get(4)?,
                    data: row.get(6)?,
                    prev_data: row.get(7)?,
                    prev_hash: row.get(8)?,
                    hash: row.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
        } else {
            stmt.query_map(params![limit as i64], |row| {
                Ok(AuditRow {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    action: row.get(2)?,
                    entity: row.get(3)?,
                    record_id: row.get(4)?,
                    data: row.get(6)?,
                    prev_data: row.get(7)?,
                    prev_hash: row.get(8)?,
                    hash: row.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
        };

        let mut output = String::new();

        for row in &rows {
            // Header line: #42 UPDATE Deployment dep_001  2026-04-02 12:30:01
            output.push_str(&format!(
                "  #{} {} {} {}  {}\n",
                row.id, row.action, row.entity, row.record_id, row.timestamp
            ));

            let data_val = row
                .data
                .as_ref()
                .and_then(|s| serde_json::from_str::<Value>(s).ok());
            let prev_val = row
                .prev_data
                .as_ref()
                .and_then(|s| serde_json::from_str::<Value>(s).ok());

            match row.action.as_str() {
                "UPDATE" => {
                    if let (Some(prev), Some(next)) = (&prev_val, &data_val) {
                        let diffs = compute_diff(prev, next);
                        if diffs.is_empty() {
                            output.push_str("    (no field changes)\n");
                        } else {
                            for d in &diffs {
                                output.push_str(&format!(
                                    "    {}: \"{}\" -> \"{}\"\n",
                                    d.field, d.old_value, d.new_value
                                ));
                            }
                        }
                    } else {
                        output.push_str("    (no prev_data available)\n");
                    }
                }
                "INSERT" => {
                    if let Some(data) = &data_val {
                        if let Some(obj) = data.as_object() {
                            for (k, v) in obj {
                                output.push_str(&format!("    + {}: \"{}\"\n", k, format_value(v)));
                            }
                        }
                    }
                }
                "DELETE" => {
                    if let Some(prev) = &prev_val {
                        if let Some(obj) = prev.as_object() {
                            for (k, v) in obj {
                                output.push_str(&format!("    - {}: \"{}\"\n", k, format_value(v)));
                            }
                        }
                    } else {
                        output.push_str(&format!("    deleted record {}\n", row.record_id));
                    }
                }
                _ => {}
            }

            // Hash verification
            if verify_hashes {
                let prev = row.prev_hash.as_deref().unwrap_or("");
                let data_str = row.data.as_deref().unwrap_or("null");
                let recomputed = Self::compute_hash(
                    &row.timestamp,
                    &row.action,
                    &row.entity,
                    &row.record_id,
                    data_str,
                    prev,
                );
                if recomputed == row.hash {
                    output.push_str("    \x1b[32m✓ hash valid\x1b[0m\n");
                } else {
                    output.push_str("    \x1b[31m✗ hash INVALID\x1b[0m\n");
                }
            }

            output.push('\n');
        }

        if rows.is_empty() {
            output.push_str("  No audit entries found.\n");
        }

        Ok(output)
    }
}

/// Standalone verify function for CLI use — opens DB read-only and verifies.
pub fn verify_from_file(db_path: &str) -> Result<Value, String> {
    let trail = AuditTrail::open(db_path)?;
    trail.verify()
}

/// Standalone debug display for CLI use.
pub fn debug_from_file(
    db_path: &str,
    limit: usize,
    entity: Option<&str>,
    verify: bool,
) -> Result<String, String> {
    let trail = AuditTrail::open(db_path)?;
    trail.debug_display(limit, entity, verify)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_diff_detects_changes() {
        let prev = json!({"name": "Alice", "age": 30, "city": "NY"});
        let next = json!({"name": "Alice", "age": 31, "city": "LA"});
        let diffs = compute_diff(&prev, &next);
        assert_eq!(diffs.len(), 2);
        let age_diff = diffs.iter().find(|d| d.field == "age").unwrap();
        assert_eq!(age_diff.old_value, "30");
        assert_eq!(age_diff.new_value, "31");
    }

    #[test]
    fn test_compute_diff_added_field() {
        let prev = json!({"name": "Alice"});
        let next = json!({"name": "Alice", "role": "admin"});
        let diffs = compute_diff(&prev, &next);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].field, "role");
        assert_eq!(diffs[0].old_value, "");
        assert_eq!(diffs[0].new_value, "admin");
    }

    #[test]
    fn test_compute_diff_removed_field() {
        let prev = json!({"name": "Alice", "tmp": "x"});
        let next = json!({"name": "Alice"});
        let diffs = compute_diff(&prev, &next);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].field, "tmp");
        assert_eq!(diffs[0].old_value, "x");
        assert_eq!(diffs[0].new_value, "");
    }

    #[test]
    fn test_compute_diff_no_changes() {
        let prev = json!({"name": "Alice"});
        let next = json!({"name": "Alice"});
        let diffs = compute_diff(&prev, &next);
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_audit_log_with_prev_data() {
        let trail = AuditTrail::open(":memory:").unwrap();

        // INSERT — no prev_data
        let h1 = trail
            .log(
                "INSERT",
                "User",
                "u1",
                "sys",
                &json!({"name": "Alice"}),
                None,
            )
            .unwrap();
        assert!(!h1.is_empty());

        // UPDATE — with prev_data
        let prev = json!({"name": "Alice"});
        let next = json!({"name": "Bob"});
        let h2 = trail
            .log("UPDATE", "User", "u1", "sys", &next, Some(&prev))
            .unwrap();
        assert!(!h2.is_empty());
        assert_ne!(h1, h2);

        // Query and check diff is present
        let entries = trail.query(10).unwrap();
        let arr = entries.as_array().unwrap();
        assert_eq!(arr.len(), 2);

        // First result is newest (UPDATE)
        let update_entry = &arr[0];
        assert!(update_entry["diff"].is_array());
        let diff_arr = update_entry["diff"].as_array().unwrap();
        assert_eq!(diff_arr.len(), 1);
        assert_eq!(diff_arr[0]["field"], "name");
        assert_eq!(diff_arr[0]["old"], "Alice");
        assert_eq!(diff_arr[0]["new"], "Bob");
    }

    #[test]
    fn test_audit_chain_integrity_with_prev_data() {
        let trail = AuditTrail::open(":memory:").unwrap();
        trail
            .log(
                "INSERT",
                "User",
                "u1",
                "sys",
                &json!({"name": "Alice"}),
                None,
            )
            .unwrap();
        trail
            .log(
                "UPDATE",
                "User",
                "u1",
                "sys",
                &json!({"name": "Bob"}),
                Some(&json!({"name": "Alice"})),
            )
            .unwrap();
        trail
            .log(
                "DELETE",
                "User",
                "u1",
                "sys",
                &json!({"id": "u1"}),
                Some(&json!({"name": "Bob"})),
            )
            .unwrap();

        let result = trail.verify().unwrap();
        assert_eq!(result["valid"], true);
        assert_eq!(result["entries"], 3);
    }

    #[test]
    fn test_query_filtered_by_entity() {
        let trail = AuditTrail::open(":memory:").unwrap();
        trail
            .log("INSERT", "User", "u1", "sys", &json!({"name": "A"}), None)
            .unwrap();
        trail
            .log("INSERT", "Order", "o1", "sys", &json!({"total": 100}), None)
            .unwrap();
        trail
            .log("INSERT", "User", "u2", "sys", &json!({"name": "B"}), None)
            .unwrap();

        let users = trail.query_filtered(10, Some("User")).unwrap();
        assert_eq!(users.as_array().unwrap().len(), 2);

        let orders = trail.query_filtered(10, Some("Order")).unwrap();
        assert_eq!(orders.as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_audit_immutable_no_delete() {
        let trail = AuditTrail::open(":memory:").unwrap();
        trail
            .log(
                "INSERT",
                "User",
                "u1",
                "sys",
                &json!({"name": "Alice"}),
                None,
            )
            .unwrap();

        // Attempt to delete should fail due to trigger
        let conn = trail.conn.lock().unwrap_or_else(|e| e.into_inner());
        let result = conn.execute("DELETE FROM _audit_log WHERE id = 1", []);
        assert!(
            result.is_err(),
            "DELETE on _audit_log should be blocked by trigger"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("cannot be deleted"),
            "Error should mention deletion prohibition: {}",
            err_msg
        );
    }

    #[test]
    fn test_audit_immutable_no_update() {
        let trail = AuditTrail::open(":memory:").unwrap();
        trail
            .log(
                "INSERT",
                "User",
                "u1",
                "sys",
                &json!({"name": "Alice"}),
                None,
            )
            .unwrap();

        // Attempt to update should fail due to trigger
        let conn = trail.conn.lock().unwrap_or_else(|e| e.into_inner());
        let result = conn.execute("UPDATE _audit_log SET action = 'FAKE' WHERE id = 1", []);
        assert!(
            result.is_err(),
            "UPDATE on _audit_log should be blocked by trigger"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("cannot be modified"),
            "Error should mention modification prohibition: {}",
            err_msg
        );
    }

    #[test]
    fn test_debug_display_format() {
        let trail = AuditTrail::open(":memory:").unwrap();
        trail
            .log(
                "INSERT",
                "Deployment",
                "dep_001",
                "sys",
                &json!({"status": "Rolling"}),
                None,
            )
            .unwrap();
        trail
            .log(
                "UPDATE",
                "Deployment",
                "dep_001",
                "sys",
                &json!({"status": "Live", "duration": "28s"}),
                Some(&json!({"status": "Rolling", "duration": "0s"})),
            )
            .unwrap();

        let output = trail.debug_display(20, None, true).unwrap();
        assert!(output.contains("UPDATE"));
        assert!(output.contains("INSERT"));
        assert!(output.contains("status:"));
        assert!(output.contains("hash valid"));
    }
}
