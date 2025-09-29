use anyhow::{Result, anyhow};
use rusqlite::Connection;
use serde_json::{Value as JsonValue, json};
use std::path::PathBuf;
use dirs;

pub struct DatabaseManager {
    db_path: PathBuf,
}

impl DatabaseManager {
    pub fn new() -> Result<Self> {
        let mut db_path = dirs::data_local_dir()
            .ok_or_else(|| anyhow!("Could not find local data directory"))?;
        db_path.push("bear-ai");
        std::fs::create_dir_all(&db_path)?;
        db_path.push("bear_ai.db");

        let manager = Self { db_path };
        manager.initialize_database()?;
        Ok(manager)
    }

    fn initialize_database(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;

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
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pii_detections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                document_id INTEGER NOT NULL,
                pii_type TEXT NOT NULL,
                original_text TEXT NOT NULL,
                replacement_text TEXT NOT NULL,
                confidence REAL NOT NULL,
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

        Ok(())
    }

    pub fn execute_sql_query(&self, query: &str) -> Result<JsonValue> {
        let conn = Connection::open(&self.db_path)?;

        // Security check: only allow SELECT, WITH statements for read-only access
        let trimmed_query = query.trim().to_uppercase();
        if !trimmed_query.starts_with("SELECT") &&
           !trimmed_query.starts_with("WITH") &&
           !trimmed_query.starts_with("PRAGMA") {
            return Err(anyhow!("Only SELECT, WITH, and PRAGMA statements are allowed for security"));
        }

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
        let conn = Connection::open(&self.db_path)?;

        conn.execute(
            "INSERT INTO documents (filename, content, file_type) VALUES (?1, ?2, ?3)",
            [filename, content, file_type],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn store_pii_detection(&self, document_id: i64, pii_type: &str, original: &str, replacement: &str, confidence: f64) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;

        conn.execute(
            "INSERT INTO pii_detections (document_id, pii_type, original_text, replacement_text, confidence)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                &document_id.to_string(),
                pii_type,
                original,
                replacement,
                &confidence.to_string()
            ],
        )?;

        Ok(())
    }

    pub fn search_documents(&self, query: &str, limit: usize) -> Result<JsonValue> {
        let conn = Connection::open(&self.db_path)?;
        let start_time = std::time::Instant::now();

        // Simple text search for now (would be enhanced with vector similarity)
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
                "relevance": 0.75 // Mock relevance score
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
        let conn = Connection::open(&self.db_path)?;

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
        let conn = Connection::open(&self.db_path)?;

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

    pub fn get_query_history(&self, limit: usize) -> Result<JsonValue> {
        let conn = Connection::open(&self.db_path)?;

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
}