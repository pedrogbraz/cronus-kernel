// CRONUS Database Engine — SQLite via rusqlite
//
// Manages schema migration from EntityNode definitions
// and provides CRUD operations with JSON I/O.

use rusqlite::{params, types::ValueRef, Connection};
use serde_json::{json, Map, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::parser::{EntityNode, FieldType};

// ── SQL query counter for debug tracing ──
static QUERY_COUNT: AtomicU64 = AtomicU64::new(0);

/// Get the current query count (since last reset).
pub fn query_count() -> u64 {
    QUERY_COUNT.load(Ordering::Relaxed)
}

/// Reset the query counter to zero. Call at the start of each request.
pub fn reset_query_count() {
    QUERY_COUNT.store(0, Ordering::Relaxed);
}

/// Increment the query counter by 1.
fn tick_query() {
    QUERY_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Read a rusqlite column as a serde_json Value, handling any type.
fn column_to_json(row: &rusqlite::Row, idx: usize) -> rusqlite::Result<Value> {
    let raw = row.get_ref(idx)?;
    Ok(match raw {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => json!(i),
        ValueRef::Real(f) => json!(f),
        ValueRef::Text(t) => Value::String(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(b) => Value::String(format!("<blob {} bytes>", b.len())),
    })
}

pub struct CronusDB {
    conn: Mutex<Connection>,
}

/// Map CRONUS field types to SQLite column types.
fn sql_type_for(ft: &FieldType) -> &'static str {
    match ft {
        FieldType::Number | FieldType::Money | FieldType::Percentage => "INTEGER",
        FieldType::Boolean => "INTEGER", // 0/1
        FieldType::Json => "TEXT",       // stored as JSON string
        // Everything else is text-like
        FieldType::String
        | FieldType::Text
        | FieldType::Email
        | FieldType::Url
        | FieldType::Slug
        | FieldType::Phone
        | FieldType::Date
        | FieldType::Ulid
        | FieldType::Enum
        | FieldType::Ip
        | FieldType::Relation => "TEXT",
    }
}

/// Generate an opaque, time-ordered UUIDv7 (RFC 9562), e.g.
/// `01920f3a-7b2c-7d41-8e5f-0a1b2c3d4e5f`.
///
/// Layout: 48-bit unix ms | ver 7 | 12-bit `rand_a` | variant 10 | 62-bit `rand_b`.
/// Randomness comes from `rand::thread_rng()` (ChaCha CSPRNG seeded from the OS
/// via `getrandom`), which is already a direct dependency — no new crate.
/// `rand_a` is used as a per-process counter (RFC 9562 §6.2 method 1) so ids
/// generated in the same millisecond still sort in creation order; the counter
/// is re-seeded with 11 random bits on each new millisecond, and a counter
/// overflow borrows the next millisecond.
fn generate_id() -> String {
    use rand::RngCore;
    use std::time::{SystemTime, UNIX_EPOCH};

    static LAST: Mutex<(u64, u16)> = Mutex::new((0, 0));

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let mut rng = rand::thread_rng();

    let (ms, counter) = {
        let mut last = LAST.lock().unwrap_or_else(|p| p.into_inner());
        if now_ms > last.0 {
            *last = (now_ms, (rng.next_u32() & 0x07FF) as u16);
        } else if last.1 >= 0x0FFF {
            *last = (last.0 + 1, (rng.next_u32() & 0x07FF) as u16);
        } else {
            last.1 += 1;
        }
        *last
    };

    uuid_v7_string(ms, counter, rng.next_u64())
}

fn uuid_v7_string(unix_ms: u64, rand_a: u16, rand_b: u64) -> String {
    let mut b = [0u8; 16];
    b[..6].copy_from_slice(&unix_ms.to_be_bytes()[2..]);
    b[6] = 0x70 | ((rand_a >> 8) as u8 & 0x0F);
    b[7] = rand_a as u8;
    b[8..].copy_from_slice(&rand_b.to_be_bytes());
    b[8] = (b[8] & 0x3F) | 0x80;
    let h = hex::encode(b);
    format!(
        "{}-{}-{}-{}-{}",
        &h[0..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..]
    )
}

impl CronusDB {
    /// Open (or create) a SQLite database at the given path.
    /// Uses WAL mode for concurrent reads + busy timeout to avoid lock errors.
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        // WAL mode: readers don't block writers, writers don't block readers
        conn.execute_batch("PRAGMA journal_mode=WAL;").ok();
        conn.execute_batch("PRAGMA foreign_keys=ON;").ok();
        // Busy timeout: wait up to 5s if DB is locked instead of failing immediately
        conn.busy_timeout(std::time::Duration::from_secs(5)).ok();
        // Synchronous NORMAL: good balance of safety vs speed (WAL makes this safe)
        conn.execute_batch("PRAGMA synchronous=NORMAL;").ok();
        // Cache size: 10MB (default is 2MB) — faster reads
        conn.execute_batch("PRAGMA cache_size=-10000;").ok();
        // Temp store in memory — faster temp table operations
        conn.execute_batch("PRAGMA temp_store=MEMORY;").ok();
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Open an in-memory database (useful for tests).
    pub fn open_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA journal_mode=WAL;").ok();
        // Same as `open`: many-to-many join tables cascade through FKs.
        conn.execute_batch("PRAGMA foreign_keys=ON;").ok();
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Execute multiple operations atomically in a SQLite transaction.
    /// If the closure returns Err, the transaction is rolled back.
    /// If it returns Ok, the transaction is committed.
    pub fn transaction<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&Connection) -> Result<T, String>,
    {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| e.to_string())?;
        match f(&conn) {
            Ok(result) => {
                conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
                Ok(result)
            }
            Err(e) => {
                conn.execute_batch("ROLLBACK").ok(); // best effort rollback
                Err(e)
            }
        }
    }

    /// Execute a raw SQL statement (for schema changes like ALTER TABLE).
    /// Silently ignores errors (e.g. column already exists).
    pub fn execute_raw(&self, sql: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.execute_batch(sql).map_err(|e| e.to_string())
    }

    /// Execute a raw SELECT query and return rows as Vec<Value>.
    /// Used by aggregation bindings (GROUP BY queries).
    pub fn query_raw(&self, sql: &str) -> Result<Vec<Value>, String> {
        tick_query();
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let col_names: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();

        let rows = stmt
            .query_map([], |row| {
                let mut map = Map::new();
                for (i, name) in col_names.iter().enumerate() {
                    map.insert(name.clone(), column_to_json(row, i)?);
                }
                Ok(Value::Object(map))
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| e.to_string())?);
        }
        Ok(results)
    }

    /// Execute a parameterized raw SQL query. Values are bound as ?1, ?2, etc.
    /// SECURITY: Use this instead of query_raw when filter values come from user input.
    pub fn query_raw_params(&self, sql: &str, params: &[String]) -> Result<Vec<Value>, String> {
        tick_query();
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let col_names: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params
            .iter()
            .map(|s| s as &dyn rusqlite::types::ToSql)
            .collect();

        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                let mut map = Map::new();
                for (i, name) in col_names.iter().enumerate() {
                    map.insert(name.clone(), column_to_json(row, i)?);
                }
                Ok(Value::Object(map))
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| e.to_string())?);
        }
        Ok(results)
    }

    // ──────────────────────────────────────────────
    // Migration
    // ──────────────────────────────────────────────

    /// Create tables from parsed entity definitions, plus one join table per
    /// many-to-many field (`tags -> Tag[]` → `Post_tags`, see `relations.rs`).
    /// Existing tables are left untouched (`CREATE TABLE IF NOT EXISTS`).
    pub fn migrate(&self, entities: &[EntityNode]) -> Result<(), String> {
        self.migrate_tables(entities)?;
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        for entity in entities {
            for field in entity.fields.iter().filter(|f| f.is_many()) {
                let Some(target) = field
                    .reference
                    .as_deref()
                    .filter(|t| entities.iter().any(|e| e.name == *t))
                else {
                    continue; // unresolved target: `build` reports RESOLVE_001
                };
                if let Some(ddl) =
                    crate::relations::join_table_ddl(&entity.name, &field.name, target)
                {
                    conn.execute_batch(&ddl).map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }

    fn migrate_tables(&self, entities: &[EntityNode]) -> Result<(), String> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        for entity in entities {
            let mut cols = vec!["id TEXT PRIMARY KEY".to_string()];

            let mut has_created_at = false;
            let mut has_updated_at = false;
            let mut seen_cols: std::collections::HashSet<String> = std::collections::HashSet::new();
            seen_cols.insert("id".to_string());

            for field in &entity.fields {
                let lower = field.name.to_lowercase();
                // Skip timestamps (auto-added below)
                if lower == "createdat" || lower == "created_at" {
                    has_created_at = true;
                    continue;
                }
                if lower == "updatedat" || lower == "updated_at" {
                    has_updated_at = true;
                    continue;
                }
                // Skip id (already PK) and many-to-many fields (join table)
                if lower == "id" || field.is_many() {
                    continue;
                }
                // Skip duplicate column names
                if seen_cols.contains(&lower) {
                    continue;
                }
                // Skip fields whose name is a SQL/CRONUS keyword that would confuse things
                if matches!(
                    lower.as_str(),
                    "string"
                        | "integer"
                        | "text"
                        | "real"
                        | "blob"
                        | "null"
                        | "primary"
                        | "table"
                        | "index"
                        | "select"
                        | "from"
                        | "where"
                ) {
                    continue;
                }
                seen_cols.insert(lower);

                let st = sql_type_for(&field.field_type);
                let not_null = if field.required { " NOT NULL" } else { "" };
                let unique = if field.unique { " UNIQUE" } else { "" };
                cols.push(format!("\"{}\" {}{}{}", field.name, st, not_null, unique));
            }

            // SECURITY: Auto-add _owner_id for data isolation per user
            if !seen_cols.contains("_owner_id") {
                cols.push("_owner_id TEXT".into());
            }

            if !has_created_at {
                cols.push("created_at TEXT DEFAULT (datetime('now'))".into());
            }
            if !has_updated_at {
                cols.push("updated_at TEXT DEFAULT (datetime('now'))".into());
            }

            let sql = format!(
                "CREATE TABLE IF NOT EXISTS \"{}\" ({})",
                entity.name,
                cols.join(", ")
            );
            conn.execute(&sql, []).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    // ──────────────────────────────────────────────
    // CRUD
    // ──────────────────────────────────────────────

    /// Insert a row from a JSON object. Returns the full row including generated id.
    pub fn insert(&self, table: &str, data: &Value) -> Result<Value, String> {
        tick_query();
        let obj = data
            .as_object()
            .ok_or("insert data must be a JSON object")?;
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let id = Self::insert_row(&conn, table, obj)?;
        drop(conn);
        self.find_by_id(table, &id)
            .map(|opt| opt.unwrap_or(json!({"id": id})))
    }

    /// Insert a row and run `then(conn, new_id)` in the same transaction
    /// (e.g. many-to-many join rows). Either both commit or neither does.
    pub fn insert_with<F>(&self, table: &str, data: &Value, then: F) -> Result<Value, String>
    where
        F: FnOnce(&Connection, &str) -> Result<(), String>,
    {
        tick_query();
        let obj = data
            .as_object()
            .ok_or("insert data must be a JSON object")?;
        let id = self.transaction(|conn| {
            let id = Self::insert_row(conn, table, obj)?;
            then(conn, &id)?;
            Ok(id)
        })?;
        self.find_by_id(table, &id)
            .map(|opt| opt.unwrap_or(json!({"id": id})))
    }

    fn insert_row(
        conn: &Connection,
        table: &str,
        obj: &Map<String, Value>,
    ) -> Result<String, String> {
        let id = generate_id();
        let mut col_names: Vec<String> = vec!["id".into()];
        let mut placeholders: Vec<String> = vec!["?".into()];
        let mut values: Vec<String> = vec![id.clone()];

        for (key, val) in obj {
            if !crate::security::is_safe_identifier(key) {
                continue; // skip invalid column names — SQL injection prevention
            }
            if matches!(val, Value::Null) {
                continue;
            }
            col_names.push(key.clone());
            placeholders.push("?".into());
            let s = match val {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            values.push(s);
        }

        let sql = format!(
            "INSERT INTO \"{}\" ({}) VALUES ({})",
            table,
            col_names.join(", "),
            placeholders.join(", ")
        );

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = values
            .iter()
            .map(|v| v as &dyn rusqlite::types::ToSql)
            .collect();

        conn.execute(&sql, param_refs.as_slice())
            .map_err(|e| e.to_string())?;
        Ok(id)
    }

    /// Update a row by id with partial data. Returns the updated row.
    pub fn update(&self, table: &str, id: &str, data: &Value) -> Result<Value, String> {
        tick_query();
        let obj = data
            .as_object()
            .ok_or("update data must be a JSON object")?;

        let mut sets = Vec::new();
        let mut values: Vec<String> = Vec::new();

        for (key, val) in obj {
            if !crate::security::is_safe_identifier(key) {
                continue; // skip invalid column names — SQL injection prevention
            }
            if key == "id" || key == "created_at" {
                continue;
            }
            if matches!(val, Value::Null) {
                continue;
            }
            sets.push(format!("\"{}\" = ?", key));
            let s = match val {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            values.push(s);
        }

        if sets.is_empty() {
            return self
                .find_by_id(table, id)
                .map(|opt| opt.unwrap_or(json!({"id": id})));
        }

        // Add updated_at
        sets.push("\"updated_at\" = datetime('now')".to_string());

        values.push(id.to_string()); // WHERE id = ?

        let sql = format!("UPDATE \"{}\" SET {} WHERE id = ?", table, sets.join(", "));

        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = values
            .iter()
            .map(|v| v as &dyn rusqlite::types::ToSql)
            .collect();

        conn.execute(&sql, param_refs.as_slice())
            .map_err(|e| e.to_string())?;

        drop(conn);
        self.find_by_id(table, id)
            .map(|opt| opt.unwrap_or(json!({"id": id})))
    }

    /// Retrieve all rows with LIMIT / OFFSET. Returns a JSON array.
    pub fn find_all(&self, table: &str, limit: usize, offset: usize) -> Result<Value, String> {
        tick_query();
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let sql = format!(
            "SELECT * FROM \"{}\" ORDER BY rowid DESC LIMIT ? OFFSET ?",
            table
        );

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let col_names: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();

        let rows = stmt
            .query_map(params![limit as i64, offset as i64], |row| {
                let mut map = Map::new();
                for (i, name) in col_names.iter().enumerate() {
                    map.insert(name.clone(), column_to_json(row, i)?);
                }
                Ok(Value::Object(map))
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| e.to_string())?);
        }

        Ok(Value::Array(results))
    }

    /// Find a single row by id. Returns `None` if not found.
    pub fn find_by_id(&self, table: &str, id: &str) -> Result<Option<Value>, String> {
        tick_query();
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let sql = format!("SELECT * FROM \"{}\" WHERE id = ?", table);

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let col_names: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();

        let mut rows = stmt
            .query_map(params![id], |row| {
                let mut map = Map::new();
                for (i, name) in col_names.iter().enumerate() {
                    map.insert(name.clone(), column_to_json(row, i)?);
                }
                Ok(Value::Object(map))
            })
            .map_err(|e| e.to_string())?;

        match rows.next() {
            Some(r) => Ok(Some(r.map_err(|e| e.to_string())?)),
            None => Ok(None),
        }
    }

    /// Delete a row by id. Returns `true` if a row was actually deleted.
    pub fn delete(&self, table: &str, id: &str) -> Result<bool, String> {
        tick_query();
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let sql = format!("DELETE FROM \"{}\" WHERE id = ?", table);
        let affected = conn.execute(&sql, params![id]).map_err(|e| e.to_string())?;
        Ok(affected > 0)
    }

    /// Count total rows in a table.
    pub fn count(&self, table: &str) -> Result<usize, String> {
        tick_query();
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let sql = format!("SELECT COUNT(*) FROM \"{}\"", table);
        let count: i64 = conn
            .query_row(&sql, [], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        Ok(count as usize)
    }

    /// Find a single row by a specific field value.
    /// e.g. find_by_field("User", "email", "zedd@cooud.com")
    pub fn find_by_field(
        &self,
        table: &str,
        field: &str,
        value: &str,
    ) -> Result<Option<Value>, String> {
        tick_query();
        if !crate::security::is_safe_identifier(field) {
            return Err("invalid field name".to_string()); // SQL injection prevention
        }
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let sql = format!(
            "SELECT * FROM \"{}\" WHERE \"{}\" = ? LIMIT 1",
            table, field
        );

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let col_names: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();

        let mut rows = stmt
            .query_map(params![value], |row| {
                let mut map = Map::new();
                for (i, name) in col_names.iter().enumerate() {
                    map.insert(name.clone(), column_to_json(row, i)?);
                }
                Ok(Value::Object(map))
            })
            .map_err(|e| e.to_string())?;

        match rows.next() {
            Some(r) => Ok(Some(r.map_err(|e| e.to_string())?)),
            None => Ok(None),
        }
    }

    /// Search across text fields with LIKE query
    pub fn search(&self, table: &str, query: &str, limit: usize) -> Result<Value, String> {
        tick_query();
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let like_pattern = format!("%{}%", query);

        // Get column names
        let probe_sql = format!("SELECT * FROM \"{}\" LIMIT 0", table);
        let probe_stmt = conn.prepare(&probe_sql).map_err(|e| e.to_string())?;
        let col_names: Vec<String> = probe_stmt
            .column_names()
            .iter()
            .map(|c| c.to_string())
            .collect();
        drop(probe_stmt);

        // Build OR conditions for all text-like columns
        let text_cols: Vec<String> = col_names
            .iter()
            .filter(|c| *c != "id" && *c != "created_at" && *c != "updated_at")
            .map(|c| format!("\"{}\" LIKE ?", c))
            .collect();

        if text_cols.is_empty() {
            return self.find_all(table, limit, 0);
        }

        let where_clause = text_cols.join(" OR ");
        let sql = format!(
            "SELECT * FROM \"{}\" WHERE {} ORDER BY rowid DESC LIMIT ?",
            table, where_clause
        );

        let mut params_vec: Vec<String> = text_cols.iter().map(|_| like_pattern.clone()).collect();
        params_vec.push(limit.to_string());

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec
            .iter()
            .map(|v| v as &dyn rusqlite::types::ToSql)
            .collect();

        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                let mut map = Map::new();
                for (i, name) in col_names.iter().enumerate() {
                    map.insert(name.clone(), column_to_json(row, i)?);
                }
                Ok(Value::Object(map))
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| e.to_string())?);
        }
        Ok(Value::Array(results))
    }

    /// Find all with query string filters (e.g. ?owner=abc&status=active)
    pub fn find_filtered(
        &self,
        table: &str,
        filters: &[(String, String)],
        limit: usize,
        offset: usize,
    ) -> Result<Value, String> {
        tick_query();
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut conditions = Vec::new();
        let mut params: Vec<String> = Vec::new();

        for (key, val) in filters {
            conditions.push(format!("\"{}\" = ?", key));
            params.push(val.clone());
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            "SELECT * FROM \"{}\"{} ORDER BY rowid DESC LIMIT ? OFFSET ?",
            table, where_clause
        );
        params.push(limit.to_string());
        params.push(offset.to_string());

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let col_names: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params
            .iter()
            .map(|v| v as &dyn rusqlite::types::ToSql)
            .collect();
        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                let mut map = Map::new();
                for (i, name) in col_names.iter().enumerate() {
                    map.insert(name.clone(), column_to_json(row, i)?);
                }
                Ok(Value::Object(map))
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| e.to_string())?);
        }
        Ok(Value::Array(results))
    }

    /// Validate a create body against the entity rules (`validation.rs`).
    /// Returns the first failing field as `'field' message`. HTTP surfaces
    /// use `validation::field_errors` directly to report every field.
    pub fn validate(&self, entity: &EntityNode, data: &Value) -> Result<(), String> {
        let obj = data.as_object().ok_or("data must be a JSON object")?;
        let errors = crate::validation::field_errors(entity, obj, crate::validation::Mode::Create);
        match errors.into_iter().next() {
            None => Ok(()),
            Some((field, messages)) => Err(format!("'{}' {}", field, messages.join("; "))),
        }
    }

    /// Check unique constraint before insert
    pub fn check_unique(&self, entity: &EntityNode, data: &Value) -> Result<(), String> {
        tick_query();
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let obj = data.as_object().ok_or("data must be a JSON object")?;

        for field in &entity.fields {
            if !field.unique {
                continue;
            }
            if let Some(val) = obj.get(&field.name) {
                if val.is_null() {
                    continue;
                }
                let val_str = val.as_str().unwrap_or(&val.to_string()).to_string();
                let sql = format!(
                    "SELECT COUNT(*) FROM \"{}\" WHERE \"{}\" = ?",
                    entity.name, field.name
                );
                let count: i64 = conn
                    .query_row(&sql, params![val_str], |row| row.get(0))
                    .unwrap_or(0);
                if count > 0 {
                    return Err(format!("'{}' already exists", field.name));
                }
            }
        }
        Ok(())
    }

    /// Insert with validation
    pub fn validated_insert(&self, entity: &EntityNode, data: &Value) -> Result<Value, String> {
        // Apply default values for missing fields
        let mut data = data.clone();
        if let Some(obj) = data.as_object_mut() {
            for field in &entity.fields {
                if let Some(ref default_val) = field.default_value {
                    let is_missing = obj.get(&field.name).map_or(true, |v| v.is_null());
                    if is_missing {
                        obj.insert(field.name.clone(), Value::String(default_val.clone()));
                    }
                }
            }
        }
        self.validate(entity, &data)?;
        self.check_unique(entity, &data)?;
        self.insert(&entity.name, &data)
    }

    /// Find a row with a related entity joined
    /// e.g. find_with_relation("Order", "abc123", "User") →
    /// SELECT o.*, u.* FROM Order o LEFT JOIN User u ON o.user = u.id WHERE o.id = ?
    pub fn find_with_relation(
        &self,
        table: &str,
        id: &str,
        relation_table: &str,
    ) -> Result<Value, String> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let relation_field = relation_table.to_lowercase();

        // Try to find the FK column (lowercase of relation table name)
        let sql = format!("SELECT * FROM \"{}\" WHERE id = ?", table);
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let col_names: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();

        let main_row = stmt
            .query_row(params![id], |row| {
                let mut map = Map::new();
                for (i, name) in col_names.iter().enumerate() {
                    map.insert(name.clone(), column_to_json(row, i)?);
                }
                Ok(map)
            })
            .map_err(|e| e.to_string())?;

        // Now fetch the related row if FK exists
        let mut result = main_row.clone();
        if let Some(fk_value) = main_row.get(&relation_field).and_then(|v| v.as_str()) {
            if !fk_value.is_empty() {
                let rel_sql = format!("SELECT * FROM \"{}\" WHERE id = ?", relation_table);
                if let Ok(mut rel_stmt) = conn.prepare(&rel_sql) {
                    let rel_cols: Vec<String> = rel_stmt
                        .column_names()
                        .iter()
                        .map(|c| c.to_string())
                        .collect();
                    if let Ok(rel_row) = rel_stmt.query_row(params![fk_value], |row| {
                        let mut map = Map::new();
                        for (i, name) in rel_cols.iter().enumerate() {
                            map.insert(name.clone(), column_to_json(row, i)?);
                        }
                        Ok(Value::Object(map))
                    }) {
                        result.insert(format!("_{}", relation_field), rel_row);
                    }
                }
            }
        }

        Ok(Value::Object(result))
    }

    /// Find all rows where a field matches a value
    /// e.g. find_all_where("Product", "store_id", "store123", 100) →
    /// SELECT * FROM Product WHERE store_id = ? LIMIT ?
    pub fn find_all_where(
        &self,
        table: &str,
        field: &str,
        value: &str,
        limit: usize,
    ) -> Result<Value, String> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let sql = format!(
            "SELECT * FROM \"{}\" WHERE \"{}\" = ? ORDER BY rowid DESC LIMIT ?",
            table, field
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let col_names: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();

        let rows = stmt
            .query_map(params![value, limit as i64], |row| {
                let mut map = Map::new();
                for (i, name) in col_names.iter().enumerate() {
                    map.insert(name.clone(), column_to_json(row, i)?);
                }
                Ok(Value::Object(map))
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| e.to_string())?);
        }
        Ok(Value::Array(results))
    }

    // ──────────────────────────────────────────────
    // Binding Model queries
    // ──────────────────────────────────────────────

    /// Query with filters, ordering, and pagination.
    /// This is the binding model's primary query function.
    pub fn find_many(
        &self,
        table: &str,
        filters: &[SqlFilter],
        order_field: Option<&str>,
        order_dir: Option<&str>, // "ASC" or "DESC"
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Value, String> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let (where_clause, mut params_vec) = sql_where(filters)?;

        // Build ORDER BY
        let order_clause = match (order_field, order_dir) {
            (Some(field), Some(dir)) => {
                if !field.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Err(format!("Invalid order field: {}", field));
                }
                let safe_dir = if dir.eq_ignore_ascii_case("DESC") {
                    "DESC"
                } else {
                    "ASC"
                };
                format!(" ORDER BY \"{}\" {}", field, safe_dir)
            }
            (Some(field), None) => {
                if !field.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Err(format!("Invalid order field: {}", field));
                }
                format!(" ORDER BY \"{}\" ASC", field)
            }
            _ => " ORDER BY rowid DESC".to_string(),
        };

        // Build LIMIT/OFFSET — use parameterized values to prevent SQL injection
        let limit_val = limit.unwrap_or(100).min(1000) as i64; // cap at 1000
        let offset_val = offset.unwrap_or(0) as i64;

        let sql = format!(
            "SELECT * FROM \"{}\"{}{} LIMIT ? OFFSET ?",
            table, where_clause, order_clause
        );

        // Execute with params (filters + limit + offset all parameterized)
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let col_names: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();

        // Append LIMIT and OFFSET as bound parameters
        params_vec.push(limit_val.to_string());
        params_vec.push(offset_val.to_string());

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec
            .iter()
            .map(|s| s as &dyn rusqlite::types::ToSql)
            .collect();

        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                let mut map = Map::new();
                for (i, name) in col_names.iter().enumerate() {
                    map.insert(name.clone(), column_to_json(row, i)?);
                }
                Ok(Value::Object(map))
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| e.to_string())?);
        }
        Ok(Value::Array(results))
    }

    /// Query for a single record (first match).
    pub fn find_one(
        &self,
        table: &str,
        filters: &[SqlFilter],
        order_field: Option<&str>,
        order_dir: Option<&str>,
    ) -> Result<Option<Value>, String> {
        let result = self.find_many(table, filters, order_field, order_dir, Some(1), Some(0))?;
        if let Value::Array(arr) = result {
            Ok(arr.into_iter().next())
        } else {
            Ok(None)
        }
    }

    /// Count records matching filters.
    pub fn count_where(&self, table: &str, filters: &[SqlFilter]) -> Result<u64, String> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let (where_clause, params_vec) = sql_where(filters)?;
        let sql = format!("SELECT COUNT(*) FROM \"{}\"{}", table, where_clause);
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec
            .iter()
            .map(|s| s as &dyn rusqlite::types::ToSql)
            .collect();
        let count: i64 = conn
            .query_row(&sql, params_refs.as_slice(), |row| row.get(0))
            .map_err(|e| e.to_string())?;
        Ok(count as u64)
    }

    /// Seed an entity with fake data (10 rows)
    pub fn seed_entity(&self, entity: &EntityNode) -> Result<usize, String> {
        let names = [
            "Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Hank", "Ivy", "Jack",
        ];
        let mut count = 0;

        for (i, name) in names.iter().enumerate() {
            let mut obj = serde_json::Map::new();

            for field in &entity.fields {
                let val = match field.field_type {
                    FieldType::String | FieldType::Text => {
                        if field.name == "name" || field.name == "title" {
                            Value::String(name.to_string())
                        } else if field.name == "description" {
                            Value::String(format!("Description for {}", name))
                        } else if field.name == "slug" {
                            Value::String(name.to_lowercase())
                        } else if field.name == "password" {
                            Value::String(crate::auth::hash_password("password123"))
                        } else {
                            Value::String(format!("{}-{}", field.name, i))
                        }
                    }
                    FieldType::Email => {
                        Value::String(format!("{}@example.com", name.to_lowercase()))
                    }
                    FieldType::Number => Value::String(format!("{}", (i + 1) * 10)),
                    FieldType::Money => Value::String(format!("{}", 1000 + i * 500)),
                    FieldType::Boolean => Value::String(if i % 2 == 0 { "1" } else { "0" }.into()),
                    FieldType::Date => Value::String("2026-03-29".into()),
                    FieldType::Url => {
                        Value::String(format!("https://example.com/{}", name.to_lowercase()))
                    }
                    FieldType::Enum => {
                        if let Some(ref values) = field.enum_values {
                            Value::String(values[i % values.len()].clone())
                        } else {
                            Value::String("default".into())
                        }
                    }
                    FieldType::Relation => {
                        // Try to find an existing row in the related table to use as FK
                        if let Some(ref target) = field.reference {
                            match self.find_all(target, 1, 0) {
                                Ok(rows) => {
                                    if let Some(arr) = rows.as_array() {
                                        if let Some(first) = arr.first() {
                                            if let Some(id) =
                                                first.get("id").and_then(|v| v.as_str())
                                            {
                                                Value::String(id.to_string())
                                            } else {
                                                Value::Null
                                            }
                                        } else {
                                            Value::Null
                                        }
                                    } else {
                                        Value::Null
                                    }
                                }
                                Err(_) => Value::Null,
                            }
                        } else {
                            Value::Null
                        }
                    }
                    _ => Value::String(format!("{}-{}", field.name, i)),
                };

                if !val.is_null() {
                    obj.insert(field.name.clone(), val);
                }
            }

            if self.insert(&entity.name, &Value::Object(obj)).is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }
}

/// Convert a binding FilterOp string to SQL operator.
/// Called by the runtime binding resolver, not by the DB directly.
pub fn filter_op_to_sql(op: &str) -> &'static str {
    match op {
        "eq" => "=",
        "ne" => "!=",
        "gt" => ">",
        "gte" => ">=",
        "lt" => "<",
        "lte" => "<=",
        "contains" => "LIKE",
        "starts_with" => "LIKE",
        "ends_with" => "LIKE",
        "in" => "IN",
        _ => "INVALID",
    }
}

/// One parameterized predicate. `IN` carries many values; others carry one.
#[derive(Debug, Clone)]
pub struct SqlFilter {
    pub field: String,
    pub op: String,
    pub values: Vec<String>,
}

impl SqlFilter {
    pub fn one(field: impl Into<String>, op: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            op: op.into(),
            values: vec![value.into()],
        }
    }

    pub fn from_triple(t: (String, String, String)) -> Self {
        Self::one(t.0, t.1, t.2)
    }
}

pub(crate) fn sql_where(filters: &[SqlFilter]) -> Result<(String, Vec<String>), String> {
    let valid_ops = ["=", "!=", ">", ">=", "<", "<=", "LIKE", "IN"];
    let mut parts = Vec::new();
    let mut params = Vec::new();
    for f in filters {
        if !f.field.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(format!("Invalid field name: {}", f.field));
        }
        if !valid_ops.contains(&f.op.as_str()) {
            return Err(format!("Invalid operator: {}", f.op));
        }
        match f.op.as_str() {
            "IN" => {
                if f.values.is_empty() {
                    parts.push("1=0".into());
                } else {
                    let ph = vec!["?"; f.values.len()].join(", ");
                    parts.push(format!("\"{}\" IN ({})", f.field, ph));
                    params.extend(f.values.iter().cloned());
                }
            }
            _ => {
                parts.push(format!("\"{}\" {} ?", f.field, f.op));
                params.push(f.values.first().cloned().unwrap_or_default());
            }
        }
    }
    let clause = if parts.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", parts.join(" AND "))
    };
    Ok((clause, params))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{EntityNode, FieldNode, FieldType};

    #[test]
    fn poisoned_connection_lock_does_not_panic_callers() {
        // Regression: `lock().unwrap()` turned one panicking request into a
        // panic in every later DB call.
        let db = CronusDB::open_memory().expect("db");
        let _ = std::thread::scope(|s| {
            s.spawn(|| {
                let _g = db.conn.lock().unwrap();
                panic!("poison");
            })
            .join()
        });
        assert!(db.conn.is_poisoned());
        let _ = db.count("missing_table");
    }

    fn test_entity() -> EntityNode {
        EntityNode {
            name: "users".into(),
            shared: false,
            fields: vec![
                FieldNode {
                    name: "name".into(),
                    field_type: FieldType::String,
                    required: true,
                    unique: false,
                    sensitive: false,
                    optional: false,
                    searchable: false,
                    index: false,
                    featured: false,
                    formatted: false,
                    array: false,
                    enum_values: None,
                    reference: None,
                    doc: None,
                    default_value: None,
                    min: None,
                    max: None,
                    min_length: None,
                    max_length: None,
                    pattern: None,
                },
                FieldNode {
                    name: "email".into(),
                    field_type: FieldType::Email,
                    required: true,
                    unique: true,
                    sensitive: false,
                    optional: false,
                    searchable: false,
                    index: false,
                    featured: false,
                    formatted: false,
                    array: false,
                    enum_values: None,
                    reference: None,
                    doc: None,
                    default_value: None,
                    min: None,
                    max: None,
                    min_length: None,
                    max_length: None,
                    pattern: None,
                },
                FieldNode {
                    name: "age".into(),
                    field_type: FieldType::Number,
                    required: false,
                    unique: false,
                    sensitive: false,
                    optional: true,
                    searchable: false,
                    index: false,
                    featured: false,
                    formatted: false,
                    array: false,
                    enum_values: None,
                    reference: None,
                    doc: None,
                    default_value: None,
                    min: None,
                    max: None,
                    min_length: None,
                    max_length: None,
                    pattern: None,
                },
            ],
            transitions: vec![],
            effects: vec![],
            remote_url: None,
            doc: None,
        }
    }

    #[test]
    fn test_migrate_and_insert() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();

        let row = db
            .insert(
                "users",
                &json!({"name": "Zedd", "email": "z@cooud.com", "age": "25"}),
            )
            .unwrap();

        assert!(row.get("id").is_some());
        assert_eq!(row.get("name").unwrap(), "Zedd");
    }

    #[test]
    fn test_find_all_and_count() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();

        db.insert("users", &json!({"name": "A", "email": "a@x.com"}))
            .unwrap();
        db.insert("users", &json!({"name": "B", "email": "b@x.com"}))
            .unwrap();

        let all = db.find_all("users", 100, 0).unwrap();
        assert_eq!(all.as_array().unwrap().len(), 2);
        assert_eq!(db.count("users").unwrap(), 2);
    }

    #[test]
    fn test_find_by_id_and_delete() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();

        let row = db
            .insert("users", &json!({"name": "C", "email": "c@x.com"}))
            .unwrap();
        let id = row.get("id").unwrap().as_str().unwrap().to_string();

        assert!(db.find_by_id("users", &id).unwrap().is_some());
        assert!(db.delete("users", &id).unwrap());
        assert!(db.find_by_id("users", &id).unwrap().is_none());
        assert_eq!(db.count("users").unwrap(), 0);
    }

    #[test]
    fn test_update() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();
        let row = db
            .insert("users", &json!({"name": "Old", "email": "u@x.com"}))
            .unwrap();
        let id = row.get("id").unwrap().as_str().unwrap();
        let updated = db.update("users", id, &json!({"name": "New"})).unwrap();
        assert_eq!(updated.get("name").unwrap(), "New");
    }

    #[test]
    fn test_validate_required() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();
        let result = db.validated_insert(&test_entity(), &json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("required"));
    }

    #[test]
    fn test_validate_email() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();
        let result =
            db.validated_insert(&test_entity(), &json!({"name": "T", "email": "notanemail"}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("email"));
    }

    #[test]
    fn test_unique_constraint() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();
        db.validated_insert(&test_entity(), &json!({"name": "A", "email": "a@x.com"}))
            .unwrap();
        let result = db.validated_insert(&test_entity(), &json!({"name": "B", "email": "a@x.com"}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_find_all_pagination() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();
        for i in 0..5 {
            db.insert(
                "users",
                &json!({"name": format!("User{}", i), "email": format!("u{}@x.com", i)}),
            )
            .unwrap();
        }
        let page1 = db.find_all("users", 2, 0).unwrap();
        assert_eq!(page1.as_array().unwrap().len(), 2);
        let page2 = db.find_all("users", 2, 2).unwrap();
        assert_eq!(page2.as_array().unwrap().len(), 2);
        let all = db.find_all("users", 100, 0).unwrap();
        assert_eq!(all.as_array().unwrap().len(), 5);
    }

    #[test]
    fn test_seed_entity() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();
        let count = db.seed_entity(&test_entity()).unwrap();
        assert_eq!(count, 10);
        assert_eq!(db.count("users").unwrap(), 10);
    }

    #[test]
    fn test_find_filtered() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();
        db.insert("users", &json!({"name": "Alice", "email": "a@x.com"}))
            .unwrap();
        db.insert("users", &json!({"name": "Bob", "email": "b@x.com"}))
            .unwrap();
        let results = db
            .find_filtered(
                "users",
                &[("name".to_string(), "Alice".to_string())],
                100,
                0,
            )
            .unwrap();
        assert_eq!(results.as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_find_by_field() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();
        db.insert("users", &json!({"name": "FindMe", "email": "find@x.com"}))
            .unwrap();
        let found = db.find_by_field("users", "email", "find@x.com").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().get("name").unwrap(), "FindMe");
        let not_found = db.find_by_field("users", "email", "nope@x.com").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_search() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();
        db.insert(
            "users",
            &json!({"name": "Alice Wonder", "email": "alice@x.com"}),
        )
        .unwrap();
        db.insert(
            "users",
            &json!({"name": "Bob Builder", "email": "bob@x.com"}),
        )
        .unwrap();
        let results = db.search("users", "alice", 100).unwrap();
        assert_eq!(results.as_array().unwrap().len(), 1);
    }

    fn entity_with_constraints() -> EntityNode {
        EntityNode {
            name: "products".into(),
            shared: false,
            fields: vec![
                FieldNode {
                    name: "title".into(),
                    field_type: FieldType::String,
                    required: true,
                    unique: false,
                    sensitive: false,
                    optional: false,
                    searchable: false,
                    index: false,
                    featured: false,
                    formatted: false,
                    array: false,
                    enum_values: None,
                    reference: None,
                    doc: None,
                    default_value: None,
                    min: None,
                    max: None,
                    min_length: Some(3),
                    max_length: Some(100),
                    pattern: Some("^[A-Za-z0-9 ]+$".to_string()),
                },
                FieldNode {
                    name: "price".into(),
                    field_type: FieldType::Number,
                    required: true,
                    unique: false,
                    sensitive: false,
                    optional: false,
                    searchable: false,
                    index: false,
                    featured: false,
                    formatted: false,
                    array: false,
                    enum_values: None,
                    reference: None,
                    doc: None,
                    default_value: None,
                    min: Some(0.0),
                    max: Some(999999.0),
                    min_length: None,
                    max_length: None,
                    pattern: None,
                },
                FieldNode {
                    name: "sku".into(),
                    field_type: FieldType::String,
                    required: false,
                    unique: false,
                    sensitive: false,
                    optional: true,
                    searchable: false,
                    index: false,
                    featured: false,
                    formatted: false,
                    array: false,
                    enum_values: None,
                    reference: None,
                    doc: None,
                    default_value: None,
                    min: None,
                    max: None,
                    min_length: None,
                    max_length: Some(20),
                    pattern: Some("^[A-Z0-9-]+$".to_string()),
                },
            ],
            transitions: vec![],
            effects: vec![],
            remote_url: None,
            doc: None,
        }
    }

    #[test]
    fn test_validate_min_number() {
        let db = CronusDB::open_memory().unwrap();
        let entity = entity_with_constraints();
        let result = db.validate(&entity, &json!({"title": "Valid", "price": -5}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be at least 0"));
    }

    #[test]
    fn test_validate_max_number() {
        let db = CronusDB::open_memory().unwrap();
        let entity = entity_with_constraints();
        let result = db.validate(&entity, &json!({"title": "Valid", "price": 1000000}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be at most 999999"));
    }

    #[test]
    fn test_validate_min_length() {
        let db = CronusDB::open_memory().unwrap();
        let entity = entity_with_constraints();
        let result = db.validate(&entity, &json!({"title": "AB", "price": 10}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least 3 characters"));
    }

    #[test]
    fn test_validate_max_length() {
        let db = CronusDB::open_memory().unwrap();
        let entity = entity_with_constraints();
        let long_title = "A".repeat(101);
        let result = db.validate(&entity, &json!({"title": long_title, "price": 10}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at most 100 characters"));
    }

    #[test]
    fn test_validate_pattern_match() {
        let db = CronusDB::open_memory().unwrap();
        let entity = entity_with_constraints();
        let result = db.validate(&entity, &json!({"title": "Invalid@Title!", "price": 10}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must match the pattern"));
    }

    #[test]
    fn test_validate_pattern_optional_field() {
        let db = CronusDB::open_memory().unwrap();
        let entity = entity_with_constraints();
        // Valid: optional sku not provided
        let result = db.validate(&entity, &json!({"title": "Good Product", "price": 50}));
        assert!(result.is_ok());
        // Invalid: sku provided but doesn't match
        let result = db.validate(
            &entity,
            &json!({"title": "Good Product", "price": 50, "sku": "bad sku!"}),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must match the pattern"));
    }

    #[test]
    fn test_validate_constraints_pass() {
        let db = CronusDB::open_memory().unwrap();
        let entity = entity_with_constraints();
        let result = db.validate(
            &entity,
            &json!({"title": "Good Product", "price": 29.99, "sku": "SKU-001"}),
        );
        assert!(result.is_ok());
    }

    // ── UUIDv7 ids ──

    #[test]
    fn generated_id_is_uuid_v7_format() {
        let re = regex::Regex::new(
            r"^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$",
        )
        .unwrap();
        for _ in 0..100 {
            let id = generate_id();
            assert!(re.is_match(&id), "not a UUIDv7: {id}");
            assert_eq!(&id[14..15], "7", "version nibble must be 7: {id}");
        }
    }

    #[test]
    fn generated_id_embeds_current_ms_timestamp() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let id = generate_id();
        let ts = u64::from_str_radix(&id.replace('-', "")[..12], 16).unwrap();
        // counter overflow may borrow a few ms ahead; never behind `before`
        assert!(
            ts >= before && ts <= before + 1000,
            "ts {ts} vs now {before}"
        );
    }

    #[test]
    fn generated_ids_are_unique_and_ordered() {
        let ids: Vec<String> = (0..10_000).map(|_| generate_id()).collect();
        let unique: std::collections::HashSet<&String> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len(), "duplicate ids generated");
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(sorted, ids, "ids must sort in creation order");
    }

    #[test]
    fn uuid_v7_bit_layout() {
        let id = uuid_v7_string(0x0123_4567_89AB, 0x0FFF, u64::MAX);
        assert_eq!(id, "01234567-89ab-7fff-bfff-ffffffffffff");
    }

    #[test]
    fn test_validated_insert_with_constraints() {
        let db = CronusDB::open_memory().unwrap();
        let entity = entity_with_constraints();
        db.migrate(&[entity.clone()]).unwrap();
        // Should succeed
        let ok = db.validated_insert(&entity, &json!({"title": "Widget", "price": 10}));
        assert!(ok.is_ok());
        // Should fail (price below min)
        let err = db.validated_insert(&entity, &json!({"title": "Widget", "price": -1}));
        assert!(err.is_err());
    }
}
