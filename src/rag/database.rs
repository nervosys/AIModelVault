//! Database connectivity abstractions for RAG systems.

use crate::error::{Result, VaultError};
use std::collections::HashMap;

use super::documents::{ChunkInfo, Document};

/// Database abstraction for rule-based systems
pub trait Database {
    /// Query the database
    fn query(&self, query: &str) -> Result<Vec<HashMap<String, String>>>;

    /// Insert data
    fn insert(&mut self, table: &str, data: HashMap<String, String>) -> Result<()>;

    /// Update data
    fn update(&mut self, table: &str, id: &str, data: HashMap<String, String>) -> Result<()>;

    /// Delete data
    fn delete(&mut self, table: &str, id: &str) -> Result<()>;
}

/// In-memory database implementation
pub struct InMemoryDatabase {
    tables: HashMap<String, Vec<HashMap<String, String>>>,
}

impl InMemoryDatabase {
    /// Create a new in-memory database
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    /// Create a table
    pub fn create_table(&mut self, name: String) {
        self.tables.insert(name, Vec::new());
    }
}

impl Default for InMemoryDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl Database for InMemoryDatabase {
    fn query(&self, query: &str) -> Result<Vec<HashMap<String, String>>> {
        // Simple query: "table_name WHERE key=value"
        let parts: Vec<&str> = query.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(Vec::new());
        }

        let table_name = parts[0];

        if let Some(table) = self.tables.get(table_name) {
            if parts.len() > 2 && parts[1] == "WHERE" {
                // Simple WHERE clause
                let condition = parts[2];
                let cond_parts: Vec<&str> = condition.split('=').collect();
                if cond_parts.len() == 2 {
                    let key = cond_parts[0];
                    let value = cond_parts[1];

                    let results: Vec<HashMap<String, String>> = table
                        .iter()
                        .filter(|row| row.get(key).map(|v| v == value).unwrap_or(false))
                        .cloned()
                        .collect();

                    return Ok(results);
                }
            }

            // Return all rows
            Ok(table.clone())
        } else {
            Ok(Vec::new())
        }
    }

    fn insert(&mut self, table: &str, data: HashMap<String, String>) -> Result<()> {
        if let Some(table_data) = self.tables.get_mut(table) {
            table_data.push(data);
            Ok(())
        } else {
            Err(VaultError::InvalidInput(format!(
                "Table {} not found",
                table
            )))
        }
    }

    fn update(&mut self, table: &str, id: &str, data: HashMap<String, String>) -> Result<()> {
        if let Some(table_data) = self.tables.get_mut(table) {
            if let Some(row) = table_data
                .iter_mut()
                .find(|r| r.get("id").map(|v| v == id).unwrap_or(false))
            {
                for (k, v) in data {
                    row.insert(k, v);
                }
                Ok(())
            } else {
                Err(VaultError::InvalidInput(format!(
                    "Row with id {} not found",
                    id
                )))
            }
        } else {
            Err(VaultError::InvalidInput(format!(
                "Table {} not found",
                table
            )))
        }
    }

    fn delete(&mut self, table: &str, id: &str) -> Result<()> {
        if let Some(table_data) = self.tables.get_mut(table) {
            table_data.retain(|r| !r.get("id").map(|v| v == id).unwrap_or(false));
            Ok(())
        } else {
            Err(VaultError::InvalidInput(format!(
                "Table {} not found",
                table
            )))
        }
    }
}

/// SQLite database implementation for persistent storage
#[cfg(feature = "sqlite")]
pub struct SQLiteDatabase {
    conn: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
}

/// Validate that a SQL identifier (table or column name) contains only safe characters.
/// Prevents SQL injection through dynamic table/column names.
#[cfg(feature = "sqlite")]
fn validate_sql_identifier(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(VaultError::InvalidInput(
            "SQL identifier cannot be empty".to_string(),
        ));
    }
    if name.len() > 128 {
        return Err(VaultError::InvalidInput(
            "SQL identifier too long (max 128 chars)".to_string(),
        ));
    }
    // Only allow alphanumeric characters and underscores; must start with letter or underscore
    // Safety: empty case is handled above, so .expect() is unreachable
    let first = name
        .chars()
        .next()
        .expect("BUG: empty check above should have returned");
    if !first.is_ascii_alphabetic() && first != '_' {
        return Err(VaultError::InvalidInput(format!(
            "SQL identifier '{}' must start with a letter or underscore",
            name
        )));
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(VaultError::InvalidInput(format!(
            "SQL identifier '{}' contains invalid characters (only alphanumeric and underscore allowed)",
            name
        )));
    }
    Ok(())
}

#[cfg(feature = "sqlite")]
impl SQLiteDatabase {
    /// Create a new SQLite database
    pub fn new(path: &std::path::Path) -> Result<Self> {
        let conn = rusqlite::Connection::open(path).map_err(|e| {
            VaultError::StorageError(format!("Failed to open SQLite database: {}", e))
        })?;

        Ok(Self {
            conn: std::sync::Arc::new(std::sync::Mutex::new(conn)),
        })
    }

    /// Create in-memory SQLite database
    pub fn in_memory() -> Result<Self> {
        let conn = rusqlite::Connection::open_in_memory().map_err(|e| {
            VaultError::StorageError(format!("Failed to create in-memory database: {}", e))
        })?;

        Ok(Self {
            conn: std::sync::Arc::new(std::sync::Mutex::new(conn)),
        })
    }

    /// Create a table with schema
    pub fn create_table(&self, name: &str, columns: &[(&str, &str)]) -> Result<()> {
        validate_sql_identifier(name)?;
        for (col_name, col_type) in columns {
            validate_sql_identifier(col_name)?;
            validate_sql_identifier(col_type)?;
        }

        let conn = self
            .conn
            .lock()
            .map_err(|e| VaultError::StorageError(format!("Lock poisoned: {}", e)))?;

        let mut sql = format!("CREATE TABLE IF NOT EXISTS {} (", name);
        sql.push_str("id TEXT PRIMARY KEY, ");
        for (i, (col_name, col_type)) in columns.iter().enumerate() {
            sql.push_str(&format!("{} {}", col_name, col_type));
            if i < columns.len() - 1 {
                sql.push_str(", ");
            }
        }
        sql.push(')');

        conn.execute(&sql, [])
            .map_err(|e| VaultError::StorageError(format!("Failed to create table: {}", e)))?;

        Ok(())
    }

    /// Store a document in the database
    pub fn store_document(&self, doc: &Document) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| VaultError::StorageError(format!("Lock poisoned: {}", e)))?;

        // Ensure documents table exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                metadata TEXT,
                embedding BLOB,
                chunk_parent_id TEXT,
                chunk_index INTEGER,
                chunk_total INTEGER,
                chunk_overlap INTEGER,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| {
            VaultError::StorageError(format!("Failed to create documents table: {}", e))
        })?;

        // Serialize metadata and embedding
        let metadata_json = serde_json::to_string(&doc.metadata)
            .map_err(|e| VaultError::SerializationError(e.to_string()))?;

        let embedding_blob = doc.embedding.as_ref().map(|emb| {
            emb.iter()
                .flat_map(|f| f.to_le_bytes())
                .collect::<Vec<u8>>()
        });

        let (chunk_parent, chunk_idx, chunk_total, chunk_overlap) =
            if let Some(chunk_info) = &doc.chunk_info {
                (
                    chunk_info.parent_id.clone(),
                    Some(chunk_info.chunk_index as i64),
                    Some(chunk_info.total_chunks as i64),
                    Some(chunk_info.overlap as i64),
                )
            } else {
                (None, None, None, None)
            };

        conn.execute(
            "INSERT OR REPLACE INTO documents 
             (id, content, metadata, embedding, chunk_parent_id, chunk_index, chunk_total, chunk_overlap)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                &doc.id,
                &doc.content,
                &metadata_json,
                &embedding_blob,
                &chunk_parent,
                &chunk_idx,
                &chunk_total,
                &chunk_overlap,
            ],
        ).map_err(|e| VaultError::StorageError(format!("Failed to insert document: {}", e)))?;

        Ok(())
    }

    /// Retrieve a document by ID
    pub fn get_document(&self, id: &str) -> Result<Option<Document>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| VaultError::StorageError(format!("Lock poisoned: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, content, metadata, embedding, chunk_parent_id, chunk_index, chunk_total, chunk_overlap 
             FROM documents WHERE id = ?1"
        ).map_err(|e| VaultError::StorageError(format!("Failed to prepare statement: {}", e)))?;

        let result = stmt.query_row([id], |row| {
            let id: String = row.get(0)?;
            let content: String = row.get(1)?;
            let metadata_json: String = row.get(2)?;
            let embedding_blob: Option<Vec<u8>> = row.get(3)?;
            let chunk_parent: Option<String> = row.get(4)?;
            let chunk_idx: Option<i64> = row.get(5)?;
            let chunk_total: Option<i64> = row.get(6)?;
            let chunk_overlap: Option<i64> = row.get(7)?;

            let metadata: HashMap<String, String> =
                serde_json::from_str(&metadata_json).unwrap_or_default();

            let embedding = embedding_blob.map(|blob| {
                blob.chunks_exact(4)
                    .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
                    .collect()
            });

            let chunk_info = if let (Some(parent), Some(idx), Some(total), Some(overlap)) =
                (chunk_parent, chunk_idx, chunk_total, chunk_overlap)
            {
                Some(ChunkInfo {
                    parent_id: Some(parent),
                    chunk_index: idx as usize,
                    total_chunks: total as usize,
                    overlap: overlap as usize,
                })
            } else {
                None
            };

            Ok(Document {
                id,
                content,
                metadata,
                embedding,
                chunk_info,
            })
        });

        match result {
            Ok(doc) => Ok(Some(doc)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(VaultError::StorageError(format!(
                "Failed to query document: {}",
                e
            ))),
        }
    }

    /// Search for documents containing text
    pub fn search_documents(&self, query: &str, limit: usize) -> Result<Vec<Document>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| VaultError::StorageError(format!("Lock poisoned: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, content, metadata, embedding, chunk_parent_id, chunk_index, chunk_total, chunk_overlap 
             FROM documents 
             WHERE content LIKE ?1
             LIMIT ?2"
        ).map_err(|e| VaultError::StorageError(format!("Failed to prepare statement: {}", e)))?;

        let search_pattern = format!("%{}%", query);
        let mut rows = stmt
            .query(rusqlite::params![&search_pattern, limit as i64])
            .map_err(|e| VaultError::StorageError(format!("Failed to query: {}", e)))?;

        let mut documents = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| VaultError::StorageError(format!("Failed to iterate rows: {}", e)))?
        {
            let id: String = row
                .get(0)
                .map_err(|e| VaultError::StorageError(format!("Failed to get id: {}", e)))?;
            let content: String = row
                .get(1)
                .map_err(|e| VaultError::StorageError(format!("Failed to get content: {}", e)))?;
            let metadata_json: String = row
                .get(2)
                .map_err(|e| VaultError::StorageError(format!("Failed to get metadata: {}", e)))?;
            let embedding_blob: Option<Vec<u8>> = row
                .get(3)
                .map_err(|e| VaultError::StorageError(format!("Failed to get embedding: {}", e)))?;

            let metadata: HashMap<String, String> =
                serde_json::from_str(&metadata_json).unwrap_or_default();

            let embedding = embedding_blob.map(|blob| {
                blob.chunks_exact(4)
                    .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
                    .collect()
            });

            documents.push(Document {
                id,
                content,
                metadata,
                embedding,
                chunk_info: None, // Simplified for search results
            });
        }

        Ok(documents)
    }
}

#[cfg(feature = "sqlite")]
impl Database for SQLiteDatabase {
    fn query(&self, query: &str) -> Result<Vec<HashMap<String, String>>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| VaultError::StorageError(format!("Lock poisoned: {}", e)))?;

        let mut stmt = conn
            .prepare(query)
            .map_err(|e| VaultError::StorageError(format!("Failed to prepare query: {}", e)))?;

        let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

        let mut rows = stmt
            .query([])
            .map_err(|e| VaultError::StorageError(format!("Failed to execute query: {}", e)))?;

        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| VaultError::StorageError(format!("Failed to iterate rows: {}", e)))?
        {
            let mut record = HashMap::new();
            for (i, col_name) in column_names.iter().enumerate() {
                let value: String = row.get(i).unwrap_or_else(|_| String::new());
                record.insert(col_name.clone(), value);
            }
            results.push(record);
        }

        Ok(results)
    }

    fn insert(&mut self, table: &str, data: HashMap<String, String>) -> Result<()> {
        validate_sql_identifier(table)?;
        let conn = self
            .conn
            .lock()
            .map_err(|e| VaultError::StorageError(format!("Lock poisoned: {}", e)))?;

        let columns: Vec<String> = data.keys().cloned().collect();
        for col in &columns {
            validate_sql_identifier(col)?;
        }
        let placeholders: Vec<String> = (1..=columns.len()).map(|i| format!("?{}", i)).collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            columns.join(", "),
            placeholders.join(", ")
        );

        let values: Vec<&str> = columns
            .iter()
            .map(|k| data.get(k).map(|s| s.as_str()).unwrap_or(""))
            .collect();

        conn.execute(&sql, rusqlite::params_from_iter(values))
            .map_err(|e| VaultError::StorageError(format!("Failed to insert: {}", e)))?;

        Ok(())
    }

    fn update(&mut self, table: &str, id: &str, data: HashMap<String, String>) -> Result<()> {
        validate_sql_identifier(table)?;
        for col in data.keys() {
            validate_sql_identifier(col)?;
        }

        let conn = self
            .conn
            .lock()
            .map_err(|e| VaultError::StorageError(format!("Lock poisoned: {}", e)))?;

        let set_clause: Vec<String> = data.keys().map(|k| format!("{} = ?", k)).collect();
        let sql = format!(
            "UPDATE {} SET {} WHERE id = ?",
            table,
            set_clause.join(", ")
        );

        let mut values: Vec<&str> = data.values().map(|s| s.as_str()).collect();
        values.push(id);

        conn.execute(&sql, rusqlite::params_from_iter(values))
            .map_err(|e| VaultError::StorageError(format!("Failed to update: {}", e)))?;

        Ok(())
    }

    fn delete(&mut self, table: &str, id: &str) -> Result<()> {
        validate_sql_identifier(table)?;

        let conn = self
            .conn
            .lock()
            .map_err(|e| VaultError::StorageError(format!("Lock poisoned: {}", e)))?;

        let sql = format!("DELETE FROM {} WHERE id = ?", table);
        conn.execute(&sql, [id])
            .map_err(|e| VaultError::StorageError(format!("Failed to delete: {}", e)))?;

        Ok(())
    }
}

/// Sled key-value database implementation
#[cfg(feature = "kv-store")]
pub struct SledDatabase {
    db: sled::Db,
}

#[cfg(feature = "kv-store")]
impl SledDatabase {
    /// Create a new Sled database
    pub fn new(path: &std::path::Path) -> Result<Self> {
        let db = sled::open(path).map_err(|e| {
            VaultError::StorageError(format!("Failed to open Sled database: {}", e))
        })?;

        Ok(Self { db })
    }

    /// Create temporary in-memory database
    pub fn temporary() -> Result<Self> {
        let config = sled::Config::new().temporary(true);
        let db = config.open().map_err(|e| {
            VaultError::StorageError(format!("Failed to create temporary database: {}", e))
        })?;

        Ok(Self { db })
    }

    /// Store a document
    pub fn store_document(&self, doc: &Document) -> Result<()> {
        let doc_json =
            serde_json::to_vec(doc).map_err(|e| VaultError::SerializationError(e.to_string()))?;

        self.db
            .insert(doc.id.as_bytes(), doc_json)
            .map_err(|e| VaultError::StorageError(format!("Failed to insert document: {}", e)))?;

        self.db
            .flush()
            .map_err(|e| VaultError::StorageError(format!("Failed to flush: {}", e)))?;

        Ok(())
    }

    /// Retrieve a document by ID
    pub fn get_document(&self, id: &str) -> Result<Option<Document>> {
        if let Some(doc_bytes) = self
            .db
            .get(id.as_bytes())
            .map_err(|e| VaultError::StorageError(format!("Failed to get document: {}", e)))?
        {
            let doc: Document = serde_json::from_slice(&doc_bytes).map_err(|e| {
                VaultError::SerializationError(format!("Failed to deserialize: {}", e))
            })?;

            Ok(Some(doc))
        } else {
            Ok(None)
        }
    }

    /// List all document IDs
    pub fn list_documents(&self) -> Result<Vec<String>> {
        let mut ids = Vec::new();

        for item in self.db.iter() {
            if let Ok((key, _)) = item {
                if let Ok(id) = String::from_utf8(key.to_vec()) {
                    ids.push(id);
                }
            }
        }

        Ok(ids)
    }

    /// Search documents by prefix
    pub fn search_prefix(&self, prefix: &str) -> Result<Vec<Document>> {
        let mut documents = Vec::new();

        for item in self.db.scan_prefix(prefix.as_bytes()) {
            if let Ok((_, value)) = item {
                if let Ok(doc) = serde_json::from_slice::<Document>(&value) {
                    documents.push(doc);
                }
            }
        }

        Ok(documents)
    }
}

#[cfg(feature = "kv-store")]
impl Database for SledDatabase {
    fn query(&self, query: &str) -> Result<Vec<HashMap<String, String>>> {
        // Simple prefix-based query for Sled
        let parts: Vec<&str> = query.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(Vec::new());
        }

        let prefix = parts[0];
        let mut results = Vec::new();

        for item in self.db.scan_prefix(prefix.as_bytes()) {
            if let Ok((key, value)) = item {
                let mut record = HashMap::new();
                record.insert("key".to_string(), String::from_utf8_lossy(&key).to_string());
                record.insert(
                    "value".to_string(),
                    String::from_utf8_lossy(&value).to_string(),
                );
                results.push(record);
            }
        }

        Ok(results)
    }

    fn insert(&mut self, table: &str, data: HashMap<String, String>) -> Result<()> {
        // For Sled, use table as prefix and id as key
        if let Some(id) = data.get("id") {
            let key = format!("{}:{}", table, id);
            let value = serde_json::to_vec(&data)
                .map_err(|e| VaultError::SerializationError(e.to_string()))?;

            self.db
                .insert(key.as_bytes(), value)
                .map_err(|e| VaultError::StorageError(format!("Failed to insert: {}", e)))?;

            self.db
                .flush()
                .map_err(|e| VaultError::StorageError(format!("Failed to flush: {}", e)))?;

            Ok(())
        } else {
            Err(VaultError::InvalidInput(
                "Data must contain 'id' field".to_string(),
            ))
        }
    }

    fn update(&mut self, table: &str, id: &str, data: HashMap<String, String>) -> Result<()> {
        let key = format!("{}:{}", table, id);

        // Get existing data, merge with new data
        if let Some(existing) = self
            .db
            .get(key.as_bytes())
            .map_err(|e| VaultError::StorageError(format!("Failed to get: {}", e)))?
        {
            let mut existing_data: HashMap<String, String> =
                serde_json::from_slice(&existing).unwrap_or_default();

            existing_data.extend(data);

            let value = serde_json::to_vec(&existing_data)
                .map_err(|e| VaultError::SerializationError(e.to_string()))?;

            self.db
                .insert(key.as_bytes(), value)
                .map_err(|e| VaultError::StorageError(format!("Failed to update: {}", e)))?;

            self.db
                .flush()
                .map_err(|e| VaultError::StorageError(format!("Failed to flush: {}", e)))?;

            Ok(())
        } else {
            Err(VaultError::InvalidInput(format!(
                "Record {}:{} not found",
                table, id
            )))
        }
    }

    fn delete(&mut self, table: &str, id: &str) -> Result<()> {
        let key = format!("{}:{}", table, id);
        self.db
            .remove(key.as_bytes())
            .map_err(|e| VaultError::StorageError(format!("Failed to delete: {}", e)))?;

        self.db
            .flush()
            .map_err(|e| VaultError::StorageError(format!("Failed to flush: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── InMemoryDatabase ──

    #[test]
    fn test_in_memory_create_and_insert() {
        let mut db = InMemoryDatabase::new();
        db.create_table("models".to_string());
        let mut data = HashMap::new();
        data.insert("id".to_string(), "m1".to_string());
        data.insert("name".to_string(), "gpt".to_string());
        db.insert("models", data).unwrap();

        let results = db.query("models").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("name").unwrap(), "gpt");
    }

    #[test]
    fn test_in_memory_update() {
        let mut db = InMemoryDatabase::new();
        db.create_table("t".to_string());
        let mut data = HashMap::new();
        data.insert("id".to_string(), "r1".to_string());
        data.insert("val".to_string(), "old".to_string());
        db.insert("t", data).unwrap();

        let mut upd = HashMap::new();
        upd.insert("val".to_string(), "new".to_string());
        db.update("t", "r1", upd).unwrap();

        let results = db.query("t").unwrap();
        assert_eq!(results[0].get("val").unwrap(), "new");
    }

    #[test]
    fn test_in_memory_delete() {
        let mut db = InMemoryDatabase::new();
        db.create_table("t".to_string());
        let mut data = HashMap::new();
        data.insert("id".to_string(), "r1".to_string());
        db.insert("t", data).unwrap();

        db.delete("t", "r1").unwrap();
        let results = db.query("t").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_in_memory_insert_missing_table() {
        let mut db = InMemoryDatabase::new();
        let data = HashMap::new();
        let err = db.insert("nonexistent", data).unwrap_err();
        assert!(format!("{err}").contains("not found"));
    }

    #[test]
    fn test_in_memory_update_missing_table() {
        let mut db = InMemoryDatabase::new();
        let err = db.update("nonexistent", "x", HashMap::new()).unwrap_err();
        assert!(format!("{err}").contains("not found"));
    }

    #[test]
    fn test_in_memory_update_missing_row() {
        let mut db = InMemoryDatabase::new();
        db.create_table("t".to_string());
        let err = db.update("t", "missing", HashMap::new()).unwrap_err();
        assert!(format!("{err}").contains("not found"));
    }

    #[test]
    fn test_in_memory_delete_missing_table() {
        let mut db = InMemoryDatabase::new();
        let err = db.delete("nonexistent", "x").unwrap_err();
        assert!(format!("{err}").contains("not found"));
    }

    #[test]
    fn test_in_memory_query_missing_table() {
        let db = InMemoryDatabase::new();
        let results = db.query("nonexistent").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_in_memory_query_where() {
        let mut db = InMemoryDatabase::new();
        db.create_table("t".to_string());
        let mut d1 = HashMap::new();
        d1.insert("id".to_string(), "1".to_string());
        d1.insert("color".to_string(), "red".to_string());
        db.insert("t", d1).unwrap();

        let mut d2 = HashMap::new();
        d2.insert("id".to_string(), "2".to_string());
        d2.insert("color".to_string(), "blue".to_string());
        db.insert("t", d2).unwrap();

        // WHERE color=red
        let results = db.query("t WHERE color=red").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("color").unwrap(), "red");
    }

    // ── validate_sql_identifier ──

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_validate_sql_identifier_empty() {
        let err = validate_sql_identifier("").unwrap_err();
        assert!(format!("{err}").contains("cannot be empty"));
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_validate_sql_identifier_too_long() {
        let long = "a".repeat(129);
        let err = validate_sql_identifier(&long).unwrap_err();
        assert!(format!("{err}").contains("too long"));
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_validate_sql_identifier_bad_start() {
        let err = validate_sql_identifier("1abc").unwrap_err();
        assert!(format!("{err}").contains("must start with"));
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_validate_sql_identifier_invalid_chars() {
        let err = validate_sql_identifier("abc def").unwrap_err();
        assert!(format!("{err}").contains("invalid characters"));
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_validate_sql_identifier_valid() {
        assert!(validate_sql_identifier("_my_table_1").is_ok());
        assert!(validate_sql_identifier("users").is_ok());
    }

    // ── SQLiteDatabase ──

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_sqlite_new_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let db = SQLiteDatabase::new(&path).unwrap();
        assert!(path.exists());

        // Verify we can create a table and insert
        db.create_table("t", &[("name", "TEXT")]).unwrap();
        let mut data = HashMap::new();
        data.insert("id".to_string(), "1".to_string());
        data.insert("name".to_string(), "alice".to_string());
        let mut db = db;
        db.insert("t", data).unwrap();
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_sqlite_in_memory() {
        let db = SQLiteDatabase::in_memory().unwrap();
        db.create_table("items", &[("val", "TEXT")]).unwrap();
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_sqlite_create_table_and_crud() {
        let mut db = SQLiteDatabase::in_memory().unwrap();
        db.create_table("users", &[("name", "TEXT"), ("age", "TEXT")])
            .unwrap();

        // Insert
        let mut data = HashMap::new();
        data.insert("id".to_string(), "u1".to_string());
        data.insert("name".to_string(), "Alice".to_string());
        data.insert("age".to_string(), "30".to_string());
        db.insert("users", data).unwrap();

        // Query
        let rows = db.query("SELECT * FROM users").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("name").unwrap(), "Alice");

        // Update
        let mut upd = HashMap::new();
        upd.insert("name".to_string(), "Bob".to_string());
        db.update("users", "u1", upd).unwrap();
        let rows = db.query("SELECT name FROM users WHERE id='u1'").unwrap();
        assert_eq!(rows[0].get("name").unwrap(), "Bob");

        // Delete
        db.delete("users", "u1").unwrap();
        let rows = db.query("SELECT * FROM users").unwrap();
        assert!(rows.is_empty());
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_sqlite_store_and_get_document() {
        let db = SQLiteDatabase::in_memory().unwrap();
        let doc = Document {
            id: "d1".to_string(),
            content: "Test document content".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("author".to_string(), "test".to_string());
                m
            },
            embedding: Some(vec![1.0, 2.0, 3.0]),
            chunk_info: Some(ChunkInfo {
                parent_id: Some("parent1".to_string()),
                chunk_index: 0,
                total_chunks: 3,
                overlap: 50,
            }),
        };
        db.store_document(&doc).unwrap();

        let retrieved = db.get_document("d1").unwrap().unwrap();
        assert_eq!(retrieved.id, "d1");
        assert_eq!(retrieved.content, "Test document content");
        assert_eq!(retrieved.metadata.get("author").unwrap(), "test");
        assert!(retrieved.embedding.is_some());
        assert!(retrieved.chunk_info.is_some());
        let ci = retrieved.chunk_info.unwrap();
        assert_eq!(ci.parent_id.as_deref(), Some("parent1"));
        assert_eq!(ci.chunk_index, 0);
        assert_eq!(ci.total_chunks, 3);
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_sqlite_get_document_not_found() {
        let db = SQLiteDatabase::in_memory().unwrap();
        // Create documents table first
        let dummy = Document {
            id: "setup".to_string(),
            content: "x".to_string(),
            metadata: HashMap::new(),
            embedding: None,
            chunk_info: None,
        };
        db.store_document(&dummy).unwrap();

        let result = db.get_document("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_sqlite_search_documents() {
        let db = SQLiteDatabase::in_memory().unwrap();
        let doc1 = Document {
            id: "s1".to_string(),
            content: "Machine learning with neural networks".to_string(),
            metadata: HashMap::new(),
            embedding: None,
            chunk_info: None,
        };
        let doc2 = Document {
            id: "s2".to_string(),
            content: "Database management systems".to_string(),
            metadata: HashMap::new(),
            embedding: None,
            chunk_info: None,
        };
        db.store_document(&doc1).unwrap();
        db.store_document(&doc2).unwrap();

        let results = db.search_documents("neural", 10).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|d| d.id == "s1"));
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_sqlite_store_document_no_embedding_no_chunk() {
        let db = SQLiteDatabase::in_memory().unwrap();
        let doc = Document {
            id: "plain".to_string(),
            content: "Plain text".to_string(),
            metadata: HashMap::new(),
            embedding: None,
            chunk_info: None,
        };
        db.store_document(&doc).unwrap();

        let retrieved = db.get_document("plain").unwrap().unwrap();
        assert_eq!(retrieved.content, "Plain text");
        assert!(retrieved.embedding.is_none());
        assert!(retrieved.chunk_info.is_none());
    }

    #[test]
    fn test_sqlite_get_document_nonexistent() {
        let db = SQLiteDatabase::in_memory().unwrap();
        let doc = Document {
            id: "existing".to_string(),
            content: "some content".to_string(),
            metadata: HashMap::new(),
            embedding: None,
            chunk_info: None,
        };
        db.store_document(&doc).unwrap();
        let result = db.get_document("does_not_exist").unwrap();
        assert!(result.is_none(), "Nonexistent document should return None");
    }

    #[test]
    fn test_sqlite_database_crud() {
        let mut db = SQLiteDatabase::in_memory().unwrap();
        db.create_table("items", &[("name", "TEXT")]).unwrap();
        let mut data = std::collections::HashMap::new();
        data.insert("id".to_string(), "item1".to_string());
        data.insert("name".to_string(), "widget".to_string());
        db.insert("items", data).unwrap();
        let rows = db.query("SELECT * FROM items").unwrap();
        assert!(!rows.is_empty());
        let mut upd = std::collections::HashMap::new();
        upd.insert("name".to_string(), "gizmo".to_string());
        db.update("items", "item1", upd).unwrap();
        db.delete("items", "item1").unwrap();
        let rows2 = db.query("SELECT * FROM items").unwrap();
        assert!(rows2.is_empty());
    }

    #[test]
    fn test_sqlite_search_documents_no_results() {
        let db = SQLiteDatabase::in_memory().unwrap();
        // Store a document first to create documents table
        let doc = Document {
            id: "seed".to_string(),
            content: "seed content".to_string(),
            metadata: HashMap::new(),
            embedding: None,
            chunk_info: None,
        };
        db.store_document(&doc).unwrap();
        let results = db.search_documents("nonexistent query", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_in_memory_default() {
        let mut db = InMemoryDatabase::default();
        db.create_table("t".to_string());
        let mut data = std::collections::HashMap::new();
        data.insert("id".to_string(), "1".to_string());
        data.insert("col".to_string(), "v".to_string());
        db.insert("t", data).unwrap();
        let rows = db.query("t").unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn test_in_memory_empty_query() {
        let db = InMemoryDatabase::new();
        let rows = db.query("").unwrap();
        assert!(rows.is_empty());
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_sqlite_search_documents_with_embedding() {
        let db = SQLiteDatabase::in_memory().unwrap();
        db.create_table("items", &[("name", "TEXT")]).unwrap();
        // Store document with embedding
        let doc = crate::rag::documents::Document {
            id: "doc_emb".to_string(),
            content: "embedding search test".to_string(),
            metadata: std::collections::HashMap::new(),
            embedding: Some(vec![0.1, 0.2, 0.3]),
            chunk_info: None,
        };
        db.store_document(&doc).unwrap();
        // Search should hit the document and parse embedding
        let results = db.search_documents("embedding", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc_emb");
        // Embedding roundtrip through BLOB
        assert!(results[0].embedding.is_some());
        let emb = results[0].embedding.as_ref().unwrap();
        assert_eq!(emb.len(), 3);
        assert!((emb[0] - 0.1).abs() < 1e-6);
    }
}
