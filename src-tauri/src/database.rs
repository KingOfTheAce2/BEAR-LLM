use anyhow::{Result, anyhow};
use rusqlite::Connection;
use serde_json::{Value as JsonValue, json};
use std::path::PathBuf;
use dirs;
use tracing;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;

pub struct DatabaseManager {
    db_path: PathBuf,
    pool: Pool<SqliteConnectionManager>,
}

impl DatabaseManager {
    pub fn new() -> Result<Self> {
        let mut db_path = dirs::data_local_dir()
            .ok_or_else(|| anyhow!("Could not find local data directory"))?;
        db_path.push("bear-ai");
        std::fs::create_dir_all(&db_path)?;
        db_path.push("bear_ai.db");

        // Create connection pool with optimal settings
        let manager = SqliteConnectionManager::file(&db_path);
        let pool = Pool::builder()
            .max_size(5) // Maximum 5 connections in pool
            .min_idle(Some(1)) // Keep at least 1 connection ready
            .connection_timeout(std::time::Duration::from_secs(30))
            .build(manager)
            .map_err(|e| anyhow!("Failed to create connection pool: {}", e))?;

        let db_manager = Self {
            db_path,
            pool,
        };

        db_manager.initialize_database()?;
        tracing::info!("Database connection pool initialized with max_size=5");

        Ok(db_manager)
    }

    pub fn new_in_memory() -> Self {
        let db_path = PathBuf::from(":memory:");

        // Create in-memory connection pool
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder()
            .max_size(5)
            .build(manager)
            .expect("Failed to create in-memory pool");

        let db_manager = Self { db_path, pool };
        let _ = db_manager.initialize_database();
        db_manager
    }

    /// Get a pooled connection from the pool
    fn get_connection(&self) -> Result<PooledConnection<SqliteConnectionManager>> {
        self.pool.get()
            .map_err(|e| anyhow!("Failed to get database connection from pool: {}", e))
    }

    /// Get pool health status
    #[allow(dead_code)]
    pub fn get_pool_status(&self) -> JsonValue {
        let state = self.pool.state();
        json!({
            "connections": state.connections,
            "idle_connections": state.idle_connections,
            "max_size": self.pool.max_size(),
        })
    }

    fn initialize_database(&self) -> Result<()> {
        let conn = self.get_connection()?;

        // Run migrations in order
        self.run_migrations(&conn)?;

        Ok(())
    }

    fn run_migrations(&self, conn: &Connection) -> Result<()> {
        // Run all migrations
        let migrations = vec![
            include_str!("../migrations/005_create_processing_records.sql"),
            include_str!("../migrations/006_create_consent_log.sql"),
        ];

        for migration in migrations {
            for statement in migration.split(';') {
                let trimmed = statement.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("--") {
                    let _ = conn.execute(trimmed, []); // Ignore errors for IF NOT EXISTS
                }
            }
        }

        // Create documents table for RAG
        conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL,
                content TEXT NOT NULL,
                file_type TEXT NOT NULL,
                upload_date DATETIME DEFAULT CURRENT_TIMESTAMP,
                vector_embedding BLOB,
                chunk_count INTEGER DEFAULT 0
            )",
            [],
        )?;

        // Create document_chunks table for RAG chunking
        conn.execute(
            "CREATE TABLE IF NOT EXISTS document_chunks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                document_id INTEGER NOT NULL,
                chunk_text TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                vector_embedding BLOB,
                FOREIGN KEY (document_id) REFERENCES documents (id)
            )",
            [],
        )?;

        // Create pii_detections table for PII tracking
        // SECURITY: We do NOT store original_text to prevent PII exposure
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pii_detections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                document_id INTEGER NOT NULL,
                pii_type TEXT NOT NULL,
                replacement_text TEXT NOT NULL,
                confidence REAL NOT NULL,
                position_start INTEGER NOT NULL,
                position_end INTEGER NOT NULL,
                detection_date DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (document_id) REFERENCES documents (id)
            )",
            [],
        )?;

        // Create legal_cases table for case management
        conn.execute(
            "CREATE TABLE IF NOT EXISTS legal_cases (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                case_number TEXT UNIQUE NOT NULL,
                case_title TEXT NOT NULL,
                case_type TEXT,
                status TEXT DEFAULT 'active',
                created_date DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_updated DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create case_documents relationship table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS case_documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                case_id INTEGER NOT NULL,
                document_id INTEGER NOT NULL,
                document_role TEXT,
                FOREIGN KEY (case_id) REFERENCES legal_cases (id),
                FOREIGN KEY (document_id) REFERENCES documents (id),
                UNIQUE(case_id, document_id)
            )",
            [],
        )?;

        // Create query_history table for tracking searches
        conn.execute(
            "CREATE TABLE IF NOT EXISTS query_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                query_text TEXT NOT NULL,
                query_type TEXT NOT NULL, -- 'rag', 'sql', 'agentic'
                results_count INTEGER DEFAULT 0,
                execution_time_ms INTEGER DEFAULT 0,
                success BOOLEAN DEFAULT 1,
                error_message TEXT,
                query_date DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create chat_sessions table for chat history export
        conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                model_used TEXT NOT NULL,
                tags TEXT DEFAULT '[]'
            )",
            [],
        )?;

        // Create chat_messages table for message storage
        conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                metadata TEXT,
                FOREIGN KEY (chat_id) REFERENCES chat_sessions (id)
            )",
            [],
        )?;

        // Create user_settings table for compliance preferences
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_settings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                setting_key TEXT UNIQUE NOT NULL,
                setting_value TEXT NOT NULL,
                last_updated DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        Ok(())
    }

    pub fn validate_query_security(query: &str) -> Result<()> {
        // SECURITY FIX: Check raw query BEFORE normalization to prevent bypass
        // This prevents attacks like: SELECT * FROM users; DROP TABLE users;--

        // 1. First, check raw query for dangerous patterns BEFORE any transformation
        let raw_query = query.trim();

        // Block semicolons (statement terminators) - critical for preventing query chaining
        if raw_query.contains(';') {
            return Err(anyhow!(
                "Security violation: Multiple statements not allowed (semicolon detected)"
            ));
        }

        // Block comment syntax that could be used to bypass validation
        if raw_query.contains("/*") || raw_query.contains("*/") {
            return Err(anyhow!(
                "Security violation: Block comments not allowed"
            ));
        }

        if raw_query.contains("--") {
            return Err(anyhow!(
                "Security violation: Line comments not allowed"
            ));
        }

        // 2. Check that query starts with SELECT (case-insensitive but strict position)
        // Use regex to ensure SELECT is at the very beginning (only whitespace before it)
        let select_regex = regex::Regex::new(r"(?i)^\s*SELECT\s+").unwrap();
        if !select_regex.is_match(raw_query) {
            return Err(anyhow!(
                "Only SELECT queries are allowed. Query must start with SELECT"
            ));
        }

        // 3. Now normalize for additional checks (safe since we've blocked injection vectors)
        let query_normalized = raw_query.to_uppercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        // 4. Block dangerous keywords - check as separate tokens
        let dangerous = [
            "DROP", "DELETE", "INSERT", "UPDATE", "ALTER",
            "EXEC", "EXECUTE", "CREATE", "PRAGMA", "ATTACH",
            "DETACH", "REPLACE", "TRUNCATE", "MERGE"
        ];

        let tokens: Vec<&str> = query_normalized.split_whitespace().collect();
        for token in &tokens {
            if dangerous.contains(token) {
                return Err(anyhow!("Query contains forbidden keyword: {}", token));
            }
        }

        // 5. Additional security checks
        if query_normalized.contains("UNION") {
            return Err(anyhow!("UNION queries are not allowed"));
        }

        if query_normalized.contains("INTO OUTFILE") || query_normalized.contains("INTO DUMPFILE") {
            return Err(anyhow!("File write operations are not allowed"));
        }

        // 6. Limit query length to prevent resource exhaustion
        if query.len() > 10000 {
            return Err(anyhow!("Query exceeds maximum length of 10000 characters"));
        }

        Ok(())
    }

    pub fn execute_sql_query(&self, query: &str) -> Result<JsonValue> {
        // Validate query security before execution
        Self::validate_query_security(query)?;

        let conn = self.get_connection()?;
        let start_time = std::time::Instant::now();

        let mut stmt = conn.prepare(query)?;
        let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

        let rows: Result<Vec<Vec<JsonValue>>, _> = stmt.query_map([], |row| {
            let mut row_data = Vec::new();
            for i in 0..column_names.len() {
                let value: JsonValue = match row.get::<usize, rusqlite::types::Value>(i)? {
                    rusqlite::types::Value::Null => JsonValue::Null,
                    rusqlite::types::Value::Integer(i) => json!(i),
                    rusqlite::types::Value::Real(r) => json!(r),
                    rusqlite::types::Value::Text(s) => json!(s),
                    rusqlite::types::Value::Blob(b) => json!(format!("BLOB({} bytes)", b.len())),
                };
                row_data.push(value);
            }
            Ok(row_data)
        })?.collect();

        let execution_time = start_time.elapsed().as_millis() as i64;
        let row_data = rows?;
        let row_count = row_data.len();

        // Log query to history
        let _ = self.log_query_history(query, "sql", row_count, execution_time, true, None);

        Ok(json!({
            "columns": column_names,
            "rows": row_data,
            "rowCount": row_count,
            "executionTime": execution_time
        }))
    }

    pub fn store_document(&self, filename: &str, content: &str, file_type: &str) -> Result<i64> {
        let conn = self.get_connection()?;

        conn.execute(
            "INSERT INTO documents (filename, content, file_type) VALUES (?1, ?2, ?3)",
            [filename, content, file_type],
        )?;

        let doc_id = conn.last_insert_rowid();

        // Set default retention period for documents (2 years)
        let retention_until = chrono::Utc::now() + chrono::Duration::days(365 * 2);
        let _ = conn.execute(
            "UPDATE documents SET retention_until = ?1 WHERE id = ?2",
            [retention_until.to_rfc3339(), doc_id.to_string()],
        );

        tracing::debug!(
            "Stored document id={} with retention_until={}",
            doc_id,
            retention_until.to_rfc3339()
        );

        Ok(doc_id)
    }

    #[allow(dead_code)]
    pub fn store_pii_detection(
        &self,
        document_id: i64,
        pii_type: &str,
        replacement: &str,
        confidence: f64,
        position_start: usize,
        position_end: usize
    ) -> Result<()> {
        let conn = self.get_connection()?;

        // SECURITY: We do NOT store the original PII text
        // Only store: type, replacement, confidence, and position info
        conn.execute(
            "INSERT INTO pii_detections (document_id, pii_type, replacement_text, confidence, position_start, position_end)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [
                &document_id.to_string(),
                pii_type,
                replacement,
                &confidence.to_string(),
                &position_start.to_string(),
                &position_end.to_string()
            ],
        )?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn search_documents(&self, query: &str, limit: usize) -> Result<JsonValue> {
        let conn = self.get_connection()?;
        let start_time = std::time::Instant::now();

        // Text-based search (vector similarity handled by RAG engine)
        let mut stmt = conn.prepare(
            "SELECT id, filename, content, file_type, upload_date
             FROM documents
             WHERE content LIKE ?1 OR filename LIKE ?1
             LIMIT ?2"
        )?;

        let search_term = format!("%{}%", query);
        let rows: Result<Vec<JsonValue>, _> = stmt.query_map([&search_term, &limit.to_string()], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "filename": row.get::<_, String>(1)?,
                "content_snippet": row.get::<_, String>(2)?.chars().take(200).collect::<String>(),
                "file_type": row.get::<_, String>(3)?,
                "upload_date": row.get::<_, String>(4)?,
                "relevance": 0.75 // Text search baseline score
            }))
        })?.collect();

        let execution_time = start_time.elapsed().as_millis() as i64;
        let results = rows?;
        let result_count = results.len();

        // Log search to history
        let _ = self.log_query_history(query, "rag", result_count, execution_time, true, None);

        Ok(json!({
            "results": results,
            "total": result_count,
            "query": query,
            "execution_time": execution_time
        }))
    }

    pub fn get_document_statistics(&self) -> Result<JsonValue> {
        let conn = self.get_connection()?;

        let doc_count: i64 = conn.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
        let pii_count: i64 = conn.query_row("SELECT COUNT(*) FROM pii_detections", [], |row| row.get(0))?;
        let case_count: i64 = conn.query_row("SELECT COUNT(*) FROM legal_cases", [], |row| row.get(0))?;

        // Get file type distribution
        let mut stmt = conn.prepare("SELECT file_type, COUNT(*) as count FROM documents GROUP BY file_type")?;
        let file_types: Result<Vec<JsonValue>, _> = stmt.query_map([], |row| {
            Ok(json!({
                "type": row.get::<_, String>(0)?,
                "count": row.get::<_, i64>(1)?
            }))
        })?.collect();

        Ok(json!({
            "total_documents": doc_count,
            "total_pii_detections": pii_count,
            "total_cases": case_count,
            "file_type_distribution": file_types?,
            "database_path": self.db_path.to_string_lossy()
        }))
    }

    fn log_query_history(&self, query: &str, query_type: &str, results_count: usize, execution_time: i64, success: bool, error_message: Option<&str>) -> Result<()> {
        let conn = self.get_connection()?;

        conn.execute(
            "INSERT INTO query_history (query_text, query_type, results_count, execution_time_ms, success, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [
                query,
                query_type,
                &results_count.to_string(),
                &execution_time.to_string(),
                if success { "1" } else { "0" },
                error_message.unwrap_or("")
            ],
        )?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_query_history(&self, limit: usize) -> Result<JsonValue> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT query_text, query_type, results_count, execution_time_ms, success, error_message, query_date
             FROM query_history
             ORDER BY query_date DESC
             LIMIT ?1"
        )?;

        let history: Result<Vec<JsonValue>, _> = stmt.query_map([&limit.to_string()], |row| {
            Ok(json!({
                "query": row.get::<_, String>(0)?,
                "type": row.get::<_, String>(1)?,
                "results_count": row.get::<_, i64>(2)?,
                "execution_time": row.get::<_, i64>(3)?,
                "success": row.get::<_, bool>(4)?,
                "error": row.get::<_, String>(5).unwrap_or_default(),
                "timestamp": row.get::<_, String>(6)?
            }))
        })?.collect();

        Ok(json!({
            "history": history?,
            "total": limit
        }))
    }

    /// GDPR Article 30 - Log processing activity
    /// Records all data processing operations for compliance
    pub fn log_processing_activity(
        &self,
        user_id: &str,
        processing_purpose: &str,
        data_categories: &[&str],
        legal_basis: &str,
        retention_days: i64,
        recipients: &[&str],
        entity_type: &str,
        entity_id: Option<&str>,
    ) -> Result<i64> {
        let conn = self.get_connection()?;

        let data_categories_json = json!(data_categories).to_string();
        let recipients_json = json!(recipients).to_string();
        let controller_info = json!({
            "name": "BEAR AI - Legal Document Assistant",
            "contact": "privacy@bear-ai.local",
            "role": "Data Controller"
        }).to_string();

        let security_measures = json!([
            "End-to-end encryption",
            "PII detection and redaction",
            "Access control and authentication",
            "Audit logging",
            "Regular security updates"
        ]).to_string();

        conn.execute(
            "INSERT INTO processing_records (
                user_id, processing_purpose, data_categories, legal_basis,
                retention_period, recipients, controller_info, security_measures,
                entity_type, entity_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            [
                user_id,
                processing_purpose,
                &data_categories_json,
                legal_basis,
                &retention_days.to_string(),
                &recipients_json,
                &controller_info,
                &security_measures,
                entity_type,
                entity_id.unwrap_or(""),
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Health check for database connectivity
    /// Returns true if database connection pool is healthy and responsive
    pub fn health_check(&self) -> Result<bool> {
        // Check if we can get a connection from the pool
        let conn = self.get_connection()?;

        // Try a simple query to verify database is responsive
        let result: Result<i64, _> = conn.query_row("SELECT 1", [], |row| row.get(0));

        match result {
            Ok(val) if val == 1 => {
                // Log pool health status
                let state = self.pool.state();
                if state.idle_connections == 0 && state.connections >= self.pool.max_size() as u32 {
                    tracing::warn!(
                        connections = state.connections,
                        idle = state.idle_connections,
                        max_size = self.pool.max_size(),
                        "Database pool exhausted - all connections in use"
                    );
                }
                Ok(true)
            }
            _ => {
                tracing::error!("Database health check failed - query returned unexpected result");
                Ok(false)
            }
        }
    }

    /// Get processing records for GDPR Article 30 compliance report
    #[allow(dead_code)]
    pub fn get_processing_records(&self, user_id: Option<&str>, limit: usize) -> Result<JsonValue> {
        let conn = self.get_connection()?;

        let mut sql = String::from(
            "SELECT id, timestamp, processing_purpose, data_categories, legal_basis,
                    retention_period, recipients, controller_info, entity_type, entity_id
             FROM processing_records"
        );

        let records = if let Some(uid) = user_id {
            sql.push_str(" WHERE user_id = ?1 ORDER BY timestamp DESC LIMIT ?2");
            let mut stmt = conn.prepare(&sql)?;
            stmt.query_map([uid, &limit.to_string()], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "timestamp": row.get::<_, String>(1)?,
                    "processing_purpose": row.get::<_, String>(2)?,
                    "data_categories": serde_json::from_str::<JsonValue>(&row.get::<_, String>(3)?).unwrap_or(json!([])),
                    "legal_basis": row.get::<_, String>(4)?,
                    "retention_period_days": row.get::<_, i64>(5)?,
                    "recipients": serde_json::from_str::<JsonValue>(&row.get::<_, String>(6)?).unwrap_or(json!([])),
                    "controller_info": serde_json::from_str::<JsonValue>(&row.get::<_, String>(7)?).unwrap_or(json!({})),
                    "entity_type": row.get::<_, String>(8)?,
                    "entity_id": row.get::<_, String>(9).unwrap_or_default()
                }))
            })?.collect::<Result<Vec<_>, _>>()?
        } else {
            sql.push_str(" ORDER BY timestamp DESC LIMIT ?1");
            let mut stmt = conn.prepare(&sql)?;
            stmt.query_map([&limit.to_string()], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "timestamp": row.get::<_, String>(1)?,
                    "processing_purpose": row.get::<_, String>(2)?,
                    "data_categories": serde_json::from_str::<JsonValue>(&row.get::<_, String>(3)?).unwrap_or(json!([])),
                    "legal_basis": row.get::<_, String>(4)?,
                    "retention_period_days": row.get::<_, i64>(5)?,
                    "recipients": serde_json::from_str::<JsonValue>(&row.get::<_, String>(6)?).unwrap_or(json!([])),
                    "controller_info": serde_json::from_str::<JsonValue>(&row.get::<_, String>(7)?).unwrap_or(json!({})),
                    "entity_type": row.get::<_, String>(8)?,
                    "entity_id": row.get::<_, String>(9).unwrap_or_default()
                }))
            })?.collect::<Result<Vec<_>, _>>()?
        };

        Ok(json!({
            "processing_records": records,
            "total": records.len(),
            "generated_at": chrono::Utc::now().to_rfc3339()
        }))
    }
}
