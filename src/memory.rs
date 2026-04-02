// CRONUS Semantic Memory — Persistent SQLite storage for AI sessions,
// decisions, anti-patterns, and changelog entries across sessions.

use rusqlite::{Connection, params};
use serde_json::{json, Value};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SemanticMemory {
    conn: Mutex<Connection>,
}

/// Generate a session ID based on current timestamp.
fn generate_session_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("sess_{:016x}", nanos)
}

/// Current UTC timestamp as ISO 8601 string.
fn now_iso() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    // Simple UTC formatting without chrono dependency
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Days since epoch to Y-M-D (simplified)
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let year_days = if is_leap(y) { 366 } else { 365 };
        if remaining < year_days {
            break;
        }
        remaining -= year_days;
        y += 1;
    }
    let leap = is_leap(y);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut m = 1u32;
    for &md in &month_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        m += 1;
    }
    let d = remaining + 1;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hours, minutes, seconds
    )
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

impl SemanticMemory {
    /// Open or create memory.db at the given path, creating all tables.
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA journal_mode=WAL;").ok();
        conn.execute_batch("PRAGMA foreign_keys=ON;").ok();

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                agent TEXT,
                summary TEXT,
                changes_count INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS decisions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT,
                date TEXT NOT NULL,
                decision TEXT NOT NULL,
                reason TEXT,
                category TEXT
            );

            CREATE TABLE IF NOT EXISTS anti_patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT,
                date TEXT NOT NULL,
                description TEXT NOT NULL,
                resolution TEXT
            );

            CREATE TABLE IF NOT EXISTS changelog (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT,
                timestamp TEXT NOT NULL,
                change_type TEXT NOT NULL,
                description TEXT NOT NULL
            );"
        ).map_err(|e| format!("Failed to init memory tables: {}", e))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Create a new session, returning its ID.
    pub fn create_session(&self, agent: Option<&str>) -> Result<String, String> {
        let id = generate_session_id();
        let now = now_iso();
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO sessions (id, started_at, agent) VALUES (?1, ?2, ?3)",
            params![id, now, agent],
        )
        .map_err(|e| e.to_string())?;
        Ok(id)
    }

    /// End a session (set ended_at and summary).
    pub fn end_session(&self, session_id: &str, summary: Option<&str>) -> Result<(), String> {
        let now = now_iso();
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE sessions SET ended_at = ?1, summary = ?2 WHERE id = ?3",
            params![now, summary, session_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Record a decision.
    pub fn add_decision(
        &self,
        session_id: Option<&str>,
        decision: &str,
        reason: Option<&str>,
        category: Option<&str>,
    ) -> Result<i64, String> {
        let now = now_iso();
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO decisions (session_id, date, decision, reason, category) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![session_id, now, decision, reason, category],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    /// Record an anti-pattern.
    pub fn add_anti_pattern(
        &self,
        session_id: Option<&str>,
        description: &str,
        resolution: Option<&str>,
    ) -> Result<i64, String> {
        let now = now_iso();
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO anti_patterns (session_id, date, description, resolution) VALUES (?1, ?2, ?3, ?4)",
            params![session_id, now, description, resolution],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    /// Add a changelog entry.
    pub fn add_changelog(
        &self,
        session_id: Option<&str>,
        change_type: &str,
        description: &str,
    ) -> Result<i64, String> {
        let now = now_iso();
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO changelog (session_id, timestamp, change_type, description) VALUES (?1, ?2, ?3, ?4)",
            params![session_id, now, change_type, description],
        )
        .map_err(|e| e.to_string())?;

        // Increment changes_count on the session if we have one
        if let Some(sid) = session_id {
            conn.execute(
                "UPDATE sessions SET changes_count = changes_count + 1 WHERE id = ?1",
                params![sid],
            )
            .ok();
        }

        Ok(conn.last_insert_rowid())
    }

    /// Get recent sessions (newest first).
    pub fn get_recent_sessions(&self, limit: usize) -> Result<Vec<Value>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, started_at, ended_at, agent, summary, changes_count FROM sessions ORDER BY started_at DESC LIMIT ?1")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "started_at": row.get::<_, String>(1)?,
                    "ended_at": row.get::<_, Option<String>>(2)?,
                    "agent": row.get::<_, Option<String>>(3)?,
                    "summary": row.get::<_, Option<String>>(4)?,
                    "changes_count": row.get::<_, i64>(5)?,
                }))
            })
            .map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| e.to_string())?);
        }
        Ok(result)
    }

    /// Get recent decisions (newest first).
    pub fn get_decisions(&self, limit: usize) -> Result<Vec<Value>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, session_id, date, decision, reason, category FROM decisions ORDER BY date DESC LIMIT ?1")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "session_id": row.get::<_, Option<String>>(1)?,
                    "date": row.get::<_, String>(2)?,
                    "decision": row.get::<_, String>(3)?,
                    "reason": row.get::<_, Option<String>>(4)?,
                    "category": row.get::<_, Option<String>>(5)?,
                }))
            })
            .map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| e.to_string())?);
        }
        Ok(result)
    }

    /// Get recent changelog entries (newest first).
    pub fn get_changelog(&self, limit: usize) -> Result<Vec<Value>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, session_id, timestamp, change_type, description FROM changelog ORDER BY timestamp DESC LIMIT ?1")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "session_id": row.get::<_, Option<String>>(1)?,
                    "timestamp": row.get::<_, String>(2)?,
                    "change_type": row.get::<_, String>(3)?,
                    "description": row.get::<_, String>(4)?,
                }))
            })
            .map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| e.to_string())?);
        }
        Ok(result)
    }

    /// Get context data for /api/_context (decisions + changelog).
    pub fn get_context_data(&self) -> Result<Value, String> {
        let decisions = self.get_decisions(10)?;
        let changelog = self.get_changelog(20)?;
        Ok(json!({
            "decisions": decisions,
            "changelog": changelog,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem_db() -> SemanticMemory {
        SemanticMemory::open(":memory:").expect("open in-memory db")
    }

    #[test]
    fn test_create_session() {
        let mem = mem_db();
        let id = mem.create_session(Some("claude")).unwrap();
        assert!(id.starts_with("sess_"));

        let sessions = mem.get_recent_sessions(10).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0]["agent"], "claude");
    }

    #[test]
    fn test_end_session() {
        let mem = mem_db();
        let id = mem.create_session(None).unwrap();
        mem.end_session(&id, Some("did stuff")).unwrap();

        let sessions = mem.get_recent_sessions(10).unwrap();
        assert_eq!(sessions[0]["summary"], "did stuff");
        assert!(sessions[0]["ended_at"].as_str().unwrap().len() > 0);
    }

    #[test]
    fn test_add_decision() {
        let mem = mem_db();
        let sid = mem.create_session(None).unwrap();
        mem.add_decision(Some(&sid), "use SQLite", Some("simple"), Some("arch"))
            .unwrap();

        let decisions = mem.get_decisions(10).unwrap();
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0]["decision"], "use SQLite");
        assert_eq!(decisions[0]["category"], "arch");
    }

    #[test]
    fn test_add_changelog() {
        let mem = mem_db();
        let sid = mem.create_session(None).unwrap();
        mem.add_changelog(Some(&sid), "entity_added", "Added User entity")
            .unwrap();
        mem.add_changelog(Some(&sid), "field_added", "Added email to User")
            .unwrap();

        let changelog = mem.get_changelog(10).unwrap();
        assert_eq!(changelog.len(), 2);

        // changes_count should be 2
        let sessions = mem.get_recent_sessions(10).unwrap();
        assert_eq!(sessions[0]["changes_count"], 2);
    }

    #[test]
    fn test_add_anti_pattern() {
        let mem = mem_db();
        mem.add_anti_pattern(None, "hardcoded credentials", Some("use env vars"))
            .unwrap();
    }

    #[test]
    fn test_get_context_data() {
        let mem = mem_db();
        mem.add_decision(None, "decision 1", None, None).unwrap();
        mem.add_changelog(None, "change", "something changed").unwrap();

        let ctx = mem.get_context_data().unwrap();
        assert_eq!(ctx["decisions"].as_array().unwrap().len(), 1);
        assert_eq!(ctx["changelog"].as_array().unwrap().len(), 1);
    }
}
