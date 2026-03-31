// CRONUS Database Engine — SQLite via rusqlite
//
// Manages schema migration from EntityNode definitions
// and provides CRUD operations with JSON I/O.

use rusqlite::{Connection, params, types::ValueRef};
use serde_json::{Value, json, Map};
use std::sync::Mutex;

use crate::parser::{EntityNode, FieldType};

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

/// Generate a simple random hex ID (16 chars).
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:016x}", nanos)
}

impl CronusDB {
    /// Open (or create) a SQLite database at the given path.
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA journal_mode=WAL;").ok();
        conn.execute_batch("PRAGMA foreign_keys=ON;").ok();
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Open an in-memory database (useful for tests).
    pub fn open_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA journal_mode=WAL;").ok();
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Execute a raw SQL statement (for schema changes like ALTER TABLE).
    /// Silently ignores errors (e.g. column already exists).
    pub fn execute_raw(&self, sql: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(sql).map_err(|e| e.to_string())
    }

    /// Execute a raw SELECT query and return rows as Vec<Value>.
    /// Used by aggregation bindings (GROUP BY queries).
    pub fn query_raw(&self, sql: &str) -> Result<Vec<Value>, String> {
        let conn = self.conn.lock().unwrap();
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

    // ──────────────────────────────────────────────
    // Migration
    // ──────────────────────────────────────────────

    /// Create tables from parsed entity definitions.
    /// Existing tables are left untouched (`CREATE TABLE IF NOT EXISTS`).
    pub fn migrate(&self, entities: &[EntityNode]) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        for entity in entities {
            let mut cols = vec!["id TEXT PRIMARY KEY".to_string()];

            let mut has_created_at = false;
            let mut has_updated_at = false;
            let mut seen_cols: std::collections::HashSet<String> = std::collections::HashSet::new();
            seen_cols.insert("id".to_string());

            for field in &entity.fields {
                let lower = field.name.to_lowercase();
                // Skip timestamps (auto-added below)
                if lower == "createdat" || lower == "created_at" { has_created_at = true; continue; }
                if lower == "updatedat" || lower == "updated_at" { has_updated_at = true; continue; }
                // Skip id (already PK)
                if lower == "id" { continue; }
                // Skip duplicate column names
                if seen_cols.contains(&lower) { continue; }
                // Skip fields whose name is a SQL/CRONUS keyword that would confuse things
                if matches!(lower.as_str(), "string" | "integer" | "text" | "real" | "blob" | "null" | "primary" | "table" | "index" | "select" | "from" | "where") {
                    continue;
                }
                seen_cols.insert(lower);

                let st = sql_type_for(&field.field_type);
                let not_null = if field.required { " NOT NULL" } else { "" };
                let unique = if field.unique { " UNIQUE" } else { "" };
                cols.push(format!("\"{}\" {}{}{}", field.name, st, not_null, unique));
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
        let conn = self.conn.lock().unwrap();
        let obj = data.as_object().ok_or("insert data must be a JSON object")?;

        let id = generate_id();
        let mut col_names: Vec<String> = vec!["id".into()];
        let mut placeholders: Vec<String> = vec!["?".into()];
        let mut values: Vec<String> = vec![id.clone()];

        for (key, val) in obj {
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

        // Return the inserted row by reading it back
        drop(conn);
        self.find_by_id(table, &id).map(|opt| opt.unwrap_or(json!({"id": id})))
    }

    /// Update a row by id with partial data. Returns the updated row.
    pub fn update(&self, table: &str, id: &str, data: &Value) -> Result<Value, String> {
        let obj = data.as_object().ok_or("update data must be a JSON object")?;

        let mut sets = Vec::new();
        let mut values: Vec<String> = Vec::new();

        for (key, val) in obj {
            if key == "id" || key == "created_at" { continue; }
            if matches!(val, Value::Null) { continue; }
            sets.push(format!("\"{}\" = ?", key));
            let s = match val {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            values.push(s);
        }

        if sets.is_empty() {
            return self.find_by_id(table, id).map(|opt| opt.unwrap_or(json!({"id": id})));
        }

        // Add updated_at
        sets.push("\"updated_at\" = datetime('now')".to_string());

        values.push(id.to_string()); // WHERE id = ?

        let sql = format!(
            "UPDATE \"{}\" SET {} WHERE id = ?",
            table,
            sets.join(", ")
        );

        let conn = self.conn.lock().unwrap();
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = values
            .iter()
            .map(|v| v as &dyn rusqlite::types::ToSql)
            .collect();

        conn.execute(&sql, param_refs.as_slice())
            .map_err(|e| e.to_string())?;

        drop(conn);
        self.find_by_id(table, id).map(|opt| opt.unwrap_or(json!({"id": id})))
    }

    /// Retrieve all rows with LIMIT / OFFSET. Returns a JSON array.
    pub fn find_all(&self, table: &str, limit: usize, offset: usize) -> Result<Value, String> {
        let conn = self.conn.lock().unwrap();
        let sql = format!(
            "SELECT * FROM \"{}\" ORDER BY rowid DESC LIMIT ? OFFSET ?",
            table
        );

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let col_names: Vec<String> = stmt
            .column_names()
            .iter()
            .map(|c| c.to_string())
            .collect();

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
        let conn = self.conn.lock().unwrap();
        let sql = format!("SELECT * FROM \"{}\" WHERE id = ?", table);

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let col_names: Vec<String> = stmt
            .column_names()
            .iter()
            .map(|c| c.to_string())
            .collect();

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
        let conn = self.conn.lock().unwrap();
        let sql = format!("DELETE FROM \"{}\" WHERE id = ?", table);
        let affected = conn.execute(&sql, params![id]).map_err(|e| e.to_string())?;
        Ok(affected > 0)
    }

    /// Count total rows in a table.
    pub fn count(&self, table: &str) -> Result<usize, String> {
        let conn = self.conn.lock().unwrap();
        let sql = format!("SELECT COUNT(*) FROM \"{}\"", table);
        let count: i64 = conn
            .query_row(&sql, [], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        Ok(count as usize)
    }

    /// Find a single row by a specific field value.
    /// e.g. find_by_field("User", "email", "zedd@cooud.com")
    pub fn find_by_field(&self, table: &str, field: &str, value: &str) -> Result<Option<Value>, String> {
        let conn = self.conn.lock().unwrap();
        let sql = format!("SELECT * FROM \"{}\" WHERE \"{}\" = ? LIMIT 1", table, field);

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
        let conn = self.conn.lock().unwrap();
        let like_pattern = format!("%{}%", query);

        // Get column names
        let probe_sql = format!("SELECT * FROM \"{}\" LIMIT 0", table);
        let probe_stmt = conn.prepare(&probe_sql).map_err(|e| e.to_string())?;
        let col_names: Vec<String> = probe_stmt.column_names().iter().map(|c| c.to_string()).collect();
        drop(probe_stmt);

        // Build OR conditions for all text-like columns
        let text_cols: Vec<String> = col_names.iter()
            .filter(|c| *c != "id" && *c != "created_at" && *c != "updated_at")
            .map(|c| format!("\"{}\" LIKE ?", c))
            .collect();

        if text_cols.is_empty() {
            return self.find_all(table, limit, 0);
        }

        let where_clause = text_cols.join(" OR ");
        let sql = format!("SELECT * FROM \"{}\" WHERE {} ORDER BY rowid DESC LIMIT ?", table, where_clause);

        let mut params_vec: Vec<String> = text_cols.iter().map(|_| like_pattern.clone()).collect();
        params_vec.push(limit.to_string());

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            let mut map = Map::new();
            for (i, name) in col_names.iter().enumerate() {
                map.insert(name.clone(), column_to_json(row, i)?);
            }
            Ok(Value::Object(map))
        }).map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for r in rows { results.push(r.map_err(|e| e.to_string())?); }
        Ok(Value::Array(results))
    }

    /// Find all with query string filters (e.g. ?owner=abc&status=active)
    pub fn find_filtered(&self, table: &str, filters: &[(String, String)], limit: usize, offset: usize) -> Result<Value, String> {
        let conn = self.conn.lock().unwrap();
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

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            let mut map = Map::new();
            for (i, name) in col_names.iter().enumerate() {
                map.insert(name.clone(), column_to_json(row, i)?);
            }
            Ok(Value::Object(map))
        }).map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for r in rows { results.push(r.map_err(|e| e.to_string())?); }
        Ok(Value::Array(results))
    }

    /// Validate data against entity field definitions before insert
    pub fn validate(&self, entity: &EntityNode, data: &Value) -> Result<(), String> {
        let obj = data.as_object().ok_or("data must be a JSON object")?;

        for field in &entity.fields {
            let val = obj.get(&field.name);
            let is_empty = val.map_or(true, |v| v.is_null() || (v.is_string() && v.as_str().unwrap_or("").is_empty()));

            // Required check
            if field.required && is_empty {
                return Err(format!("field '{}' is required", field.name));
            }

            // Skip further validation if empty and not required
            if is_empty { continue; }

            let val_str = val.unwrap().as_str().unwrap_or(&val.unwrap().to_string()).to_string();

            // Enum validation
            if field.field_type == FieldType::Enum {
                if let Some(ref allowed) = field.enum_values {
                    if !allowed.contains(&val_str) {
                        return Err(format!("'{}' must be one of: {}", field.name, allowed.join(", ")));
                    }
                }
            }

            // Email validation
            if field.field_type == FieldType::Email {
                if !val_str.contains('@') || !val_str.contains('.') {
                    return Err(format!("'{}' must be a valid email", field.name));
                }
            }

            // Money/Number validation
            if matches!(field.field_type, FieldType::Money | FieldType::Number) {
                if val_str.parse::<f64>().is_err() {
                    return Err(format!("'{}' must be a number", field.name));
                }
            }
        }
        Ok(())
    }

    /// Check unique constraint before insert
    pub fn check_unique(&self, entity: &EntityNode, data: &Value) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let obj = data.as_object().ok_or("data must be a JSON object")?;

        for field in &entity.fields {
            if !field.unique { continue; }
            if let Some(val) = obj.get(&field.name) {
                if val.is_null() { continue; }
                let val_str = val.as_str().unwrap_or(&val.to_string()).to_string();
                let sql = format!("SELECT COUNT(*) FROM \"{}\" WHERE \"{}\" = ?", entity.name, field.name);
                let count: i64 = conn.query_row(&sql, params![val_str], |row| row.get(0)).unwrap_or(0);
                if count > 0 {
                    return Err(format!("'{}' already exists", field.name));
                }
            }
        }
        Ok(())
    }

    /// Insert with validation
    pub fn validated_insert(&self, entity: &EntityNode, data: &Value) -> Result<Value, String> {
        self.validate(entity, data)?;
        self.check_unique(entity, data)?;
        self.insert(&entity.name, data)
    }

    /// Find a row with a related entity joined
    /// e.g. find_with_relation("Order", "abc123", "User") →
    /// SELECT o.*, u.* FROM Order o LEFT JOIN User u ON o.user = u.id WHERE o.id = ?
    pub fn find_with_relation(&self, table: &str, id: &str, relation_table: &str) -> Result<Value, String> {
        let conn = self.conn.lock().unwrap();
        let relation_field = relation_table.to_lowercase();

        // Try to find the FK column (lowercase of relation table name)
        let sql = format!(
            "SELECT * FROM \"{}\" WHERE id = ?",
            table
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let col_names: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();

        let main_row = stmt.query_row(params![id], |row| {
            let mut map = Map::new();
            for (i, name) in col_names.iter().enumerate() {
                map.insert(name.clone(), column_to_json(row, i)?);
            }
            Ok(map)
        }).map_err(|e| e.to_string())?;

        // Now fetch the related row if FK exists
        let mut result = main_row.clone();
        if let Some(fk_value) = main_row.get(&relation_field).and_then(|v| v.as_str()) {
            if !fk_value.is_empty() {
                let rel_sql = format!("SELECT * FROM \"{}\" WHERE id = ?", relation_table);
                if let Ok(mut rel_stmt) = conn.prepare(&rel_sql) {
                    let rel_cols: Vec<String> = rel_stmt.column_names().iter().map(|c| c.to_string()).collect();
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
    pub fn find_all_where(&self, table: &str, field: &str, value: &str, limit: usize) -> Result<Value, String> {
        let conn = self.conn.lock().unwrap();
        let sql = format!(
            "SELECT * FROM \"{}\" WHERE \"{}\" = ? ORDER BY rowid DESC LIMIT ?",
            table, field
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let col_names: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();

        let rows = stmt.query_map(params![value, limit as i64], |row| {
            let mut map = Map::new();
            for (i, name) in col_names.iter().enumerate() {
                map.insert(name.clone(), column_to_json(row, i)?);
            }
            Ok(Value::Object(map))
        }).map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for r in rows { results.push(r.map_err(|e| e.to_string())?); }
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
        filters: &[(String, String, String)], // (field, sql_op, value)
        order_field: Option<&str>,
        order_dir: Option<&str>, // "ASC" or "DESC"
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Value, String> {
        let conn = self.conn.lock().unwrap();

        // Build WHERE clause
        let mut where_parts = Vec::new();
        let mut params_vec: Vec<String> = Vec::new();

        let valid_ops = ["=", "!=", ">", ">=", "<", "<=", "LIKE"];

        for (field, op, value) in filters {
            // Validate: field name must be alphanumeric + underscore (prevent SQL injection)
            if !field.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(format!("Invalid field name: {}", field));
            }
            // Validate operator against whitelist
            if !valid_ops.contains(&op.as_str()) {
                return Err(format!("Invalid operator: {}", op));
            }
            where_parts.push(format!("\"{}\" {} ?", field, op));
            params_vec.push(value.clone());
        }

        let where_clause = if where_parts.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_parts.join(" AND "))
        };

        // Build ORDER BY
        let order_clause = match (order_field, order_dir) {
            (Some(field), Some(dir)) => {
                if !field.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Err(format!("Invalid order field: {}", field));
                }
                let safe_dir = if dir.eq_ignore_ascii_case("DESC") { "DESC" } else { "ASC" };
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

        // Build LIMIT/OFFSET
        let limit_val = limit.unwrap_or(100);
        let offset_val = offset.unwrap_or(0);

        let sql = format!(
            "SELECT * FROM \"{}\"{}{} LIMIT {} OFFSET {}",
            table, where_clause, order_clause, limit_val, offset_val
        );

        // Execute with params
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let col_names: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();

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
        filters: &[(String, String, String)],
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
    pub fn count_where(
        &self,
        table: &str,
        filters: &[(String, String, String)],
    ) -> Result<u64, String> {
        let conn = self.conn.lock().unwrap();

        let valid_ops = ["=", "!=", ">", ">=", "<", "<=", "LIKE"];

        let mut where_parts = Vec::new();
        let mut params_vec: Vec<String> = Vec::new();

        for (field, op, value) in filters {
            if !field.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(format!("Invalid field name: {}", field));
            }
            if !valid_ops.contains(&op.as_str()) {
                return Err(format!("Invalid operator: {}", op));
            }
            where_parts.push(format!("\"{}\" {} ?", field, op));
            params_vec.push(value.clone());
        }

        let where_clause = if where_parts.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_parts.join(" AND "))
        };

        let sql = format!("SELECT COUNT(*) FROM \"{}\"{}",  table, where_clause);

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
        let names = ["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Hank", "Ivy", "Jack"];
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
                    FieldType::Email => Value::String(format!("{}@example.com", name.to_lowercase())),
                    FieldType::Number => Value::String(format!("{}", (i + 1) * 10)),
                    FieldType::Money => Value::String(format!("{}", 1000 + i * 500)),
                    FieldType::Boolean => Value::String(if i % 2 == 0 { "1" } else { "0" }.into()),
                    FieldType::Date => Value::String("2026-03-29".into()),
                    FieldType::Url => Value::String(format!("https://example.com/{}", name.to_lowercase())),
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
                                            if let Some(id) = first.get("id").and_then(|v| v.as_str()) {
                                                Value::String(id.to_string())
                                            } else { Value::Null }
                                        } else { Value::Null }
                                    } else { Value::Null }
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
        "contains" => "LIKE",     // value needs %value% wrapping
        "starts_with" => "LIKE",  // value needs value% wrapping
        _ => "=",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{EntityNode, FieldNode, FieldType};

    fn test_entity() -> EntityNode {
        EntityNode {
            name: "users".into(),
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
                },
            ],
        }
    }

    #[test]
    fn test_migrate_and_insert() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();

        let row = db
            .insert("users", &json!({"name": "Zedd", "email": "z@cooud.com", "age": "25"}))
            .unwrap();

        assert!(row.get("id").is_some());
        assert_eq!(row.get("name").unwrap(), "Zedd");
    }

    #[test]
    fn test_find_all_and_count() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();

        db.insert("users", &json!({"name": "A", "email": "a@x.com"})).unwrap();
        db.insert("users", &json!({"name": "B", "email": "b@x.com"})).unwrap();

        let all = db.find_all("users", 100, 0).unwrap();
        assert_eq!(all.as_array().unwrap().len(), 2);
        assert_eq!(db.count("users").unwrap(), 2);
    }

    #[test]
    fn test_find_by_id_and_delete() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();

        let row = db.insert("users", &json!({"name": "C", "email": "c@x.com"})).unwrap();
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
        let row = db.insert("users", &json!({"name": "Old", "email": "u@x.com"})).unwrap();
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
        let result = db.validated_insert(&test_entity(), &json!({"name": "T", "email": "notanemail"}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("email"));
    }

    #[test]
    fn test_unique_constraint() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();
        db.validated_insert(&test_entity(), &json!({"name": "A", "email": "a@x.com"})).unwrap();
        let result = db.validated_insert(&test_entity(), &json!({"name": "B", "email": "a@x.com"}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_find_all_pagination() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();
        for i in 0..5 {
            db.insert("users", &json!({"name": format!("User{}", i), "email": format!("u{}@x.com", i)})).unwrap();
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
        db.insert("users", &json!({"name": "Alice", "email": "a@x.com"})).unwrap();
        db.insert("users", &json!({"name": "Bob", "email": "b@x.com"})).unwrap();
        let results = db.find_filtered("users", &[("name".to_string(), "Alice".to_string())], 100, 0).unwrap();
        assert_eq!(results.as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_find_by_field() {
        let db = CronusDB::open_memory().unwrap();
        db.migrate(&[test_entity()]).unwrap();
        db.insert("users", &json!({"name": "FindMe", "email": "find@x.com"})).unwrap();
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
        db.insert("users", &json!({"name": "Alice Wonder", "email": "alice@x.com"})).unwrap();
        db.insert("users", &json!({"name": "Bob Builder", "email": "bob@x.com"})).unwrap();
        let results = db.search("users", "alice", 100).unwrap();
        assert_eq!(results.as_array().unwrap().len(), 1);
    }
}
