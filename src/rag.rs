//! RAG (Retrieval-Augmented Generation) and Rule-Based System Support
//!
//! Provides infrastructure for:
//! - Document storage and retrieval for RAG systems
//! - Vector embeddings management
//! - Knowledge base operations
//! - Rule-based system integration
//! - Cache management for retrieval
//! - Database connectivity abstractions
//! - Persistent storage with SQLite
//! - High-performance KV store with Sled
//! - Vector database integration

use crate::error::{Result, VaultError};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

#[cfg(feature = "sqlite")]
use rusqlite;

#[cfg(feature = "kv-store")]
use sled;

#[cfg(feature = "vector-db")]
use qdrant_client;

/// Document for RAG systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique document identifier
    pub id: String,

    /// Document content/text
    pub content: String,

    /// Document metadata
    pub metadata: HashMap<String, String>,

    /// Optional embedding vector
    pub embedding: Option<Vec<f32>>,

    /// Chunk information (if document is split)
    pub chunk_info: Option<ChunkInfo>,
}

/// Information about document chunking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    /// Parent document ID (if this is a chunk)
    pub parent_id: Option<String>,

    /// Chunk index
    pub chunk_index: usize,

    /// Total number of chunks
    pub total_chunks: usize,

    /// Overlap with adjacent chunks (in characters)
    pub overlap: usize,
}

/// Document store for RAG systems
pub struct DocumentStore {
    documents: HashMap<String, Document>,
    index: Vec<(String, Vec<f32>)>, // (doc_id, embedding)
}

impl DocumentStore {
    /// Create a new document store
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            index: Vec::new(),
        }
    }

    /// Add a document to the store
    pub fn add_document(&mut self, doc: Document) -> Result<()> {
        let id = doc.id.clone();

        // If document has embedding, add to index
        if let Some(embedding) = &doc.embedding {
            self.index.push((id.clone(), embedding.clone()));
        }

        self.documents.insert(id, doc);
        Ok(())
    }

    /// Get a document by ID
    pub fn get_document(&self, id: &str) -> Option<&Document> {
        self.documents.get(id)
    }

    /// Search documents by similarity (cosine similarity)
    pub fn search_similar(&self, query_embedding: &[f32], top_k: usize) -> Vec<(String, f32)> {
        let mut similarities: Vec<(String, f32)> = self
            .index
            .iter()
            .map(|(id, embedding)| {
                let similarity = cosine_similarity(query_embedding, embedding);
                (id.clone(), similarity)
            })
            .collect();

        // Sort by similarity (descending)
        // Handle NaN values by treating them as equal (should never occur with cosine similarity)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top_k results
        similarities.into_iter().take(top_k).collect()
    }

    /// Get all documents
    pub fn get_all_documents(&self) -> Vec<&Document> {
        self.documents.values().collect()
    }

    /// Delete a document
    pub fn delete_document(&mut self, id: &str) -> Result<()> {
        self.documents.remove(id);
        self.index.retain(|(doc_id, _)| doc_id != id);
        Ok(())
    }

    /// Get document count
    pub fn count(&self) -> usize {
        self.documents.len()
    }

    /// Clear all documents
    pub fn clear(&mut self) {
        self.documents.clear();
        self.index.clear();
    }
}

impl Default for DocumentStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

/// Knowledge base for RAG systems
pub struct KnowledgeBase {
    /// Document store
    pub store: DocumentStore,

    /// Knowledge base name
    pub name: String,

    /// Configuration
    pub config: KnowledgeBaseConfig,
}

/// Knowledge base configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBaseConfig {
    /// Embedding dimension
    pub embedding_dim: usize,

    /// Chunk size (for document splitting)
    pub chunk_size: usize,

    /// Chunk overlap
    pub chunk_overlap: usize,

    /// Maximum results to return
    pub max_results: usize,

    /// Minimum similarity threshold
    pub similarity_threshold: f32,
}

impl Default for KnowledgeBaseConfig {
    fn default() -> Self {
        Self {
            embedding_dim: 384, // Common for all-MiniLM-L6-v2
            chunk_size: 512,
            chunk_overlap: 50,
            max_results: 5,
            similarity_threshold: 0.5,
        }
    }
}

impl KnowledgeBase {
    /// Create a new knowledge base
    pub fn new(name: String, config: KnowledgeBaseConfig) -> Self {
        Self {
            store: DocumentStore::new(),
            name,
            config,
        }
    }

    /// Add a document
    pub fn add(&mut self, doc: Document) -> Result<()> {
        self.store.add_document(doc)
    }

    /// Retrieve relevant documents
    pub fn retrieve(&self, query_embedding: &[f32], top_k: Option<usize>) -> Vec<Document> {
        let k = top_k.unwrap_or(self.config.max_results);
        let results = self.store.search_similar(query_embedding, k);

        results
            .into_iter()
            .filter(|(_, score)| *score >= self.config.similarity_threshold)
            .filter_map(|(id, _)| self.store.get_document(&id).cloned())
            .collect()
    }

    /// Split text into chunks
    pub fn chunk_text(&self, text: &str, doc_id: &str) -> Vec<Document> {
        let chars: Vec<char> = text.chars().collect();
        let mut chunks = Vec::new();
        let mut start = 0;
        let chunk_size = self.config.chunk_size;
        let overlap = self.config.chunk_overlap;

        let mut chunk_index = 0;
        while start < chars.len() {
            let end = (start + chunk_size).min(chars.len());
            let chunk_text: String = chars[start..end].iter().collect();

            let chunk = Document {
                id: format!("{}_chunk_{}", doc_id, chunk_index),
                content: chunk_text,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("parent_id".to_string(), doc_id.to_string());
                    meta.insert("chunk_index".to_string(), chunk_index.to_string());
                    meta
                },
                embedding: None,
                chunk_info: Some(ChunkInfo {
                    parent_id: Some(doc_id.to_string()),
                    chunk_index,
                    total_chunks: 0, // Will be updated later
                    overlap,
                }),
            };

            chunks.push(chunk);
            chunk_index += 1;

            if end >= chars.len() {
                break;
            }

            start = end - overlap.min(end);
        }

        // Update total_chunks
        let total = chunks.len();
        for chunk in &mut chunks {
            if let Some(info) = &mut chunk.chunk_info {
                info.total_chunks = total;
            }
        }

        chunks
    }
}

/// Rule for rule-based systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Rule ID
    pub id: String,

    /// Rule name
    pub name: String,

    /// Rule conditions (key-value pairs)
    pub conditions: HashMap<String, RuleCondition>,

    /// Rule actions
    pub actions: Vec<RuleAction>,

    /// Rule priority (higher = more priority)
    pub priority: i32,

    /// Is rule enabled
    pub enabled: bool,
}

/// Rule condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Exact match
    Equals(String),

    /// Contains substring
    Contains(String),

    /// Matches regex pattern
    Matches(String),

    /// Numeric comparison
    GreaterThan(f64),
    LessThan(f64),

    /// In list
    In(Vec<String>),

    /// Custom condition (serialized as string)
    Custom(String),
}

/// Rule action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    /// Set a value
    SetValue { key: String, value: String },

    /// Add to list
    AddToList { key: String, value: String },

    /// Log message
    Log { level: String, message: String },

    /// Call function (by name)
    CallFunction { function: String, args: Vec<String> },

    /// Stop rule processing
    Stop,
}

/// Rule engine for rule-based systems
pub struct RuleEngine {
    rules: Vec<Rule>,
    context: HashMap<String, String>,
}

impl RuleEngine {
    /// Create a new rule engine
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            context: HashMap::new(),
        }
    }

    /// Add a rule
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
        // Sort by priority (descending)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Set context value
    pub fn set_context(&mut self, key: String, value: String) {
        self.context.insert(key, value);
    }

    /// Get context value
    pub fn get_context(&self, key: &str) -> Option<&String> {
        self.context.get(key)
    }

    /// Evaluate a rule condition
    fn evaluate_condition(&self, key: &str, condition: &RuleCondition) -> bool {
        let value = match self.context.get(key) {
            Some(v) => v,
            None => return false,
        };

        match condition {
            RuleCondition::Equals(expected) => value == expected,
            RuleCondition::Contains(substring) => value.contains(substring),
            RuleCondition::Matches(pattern) => {
                // Simple pattern matching (could use regex crate for more complex patterns)
                value.contains(pattern)
            }
            RuleCondition::GreaterThan(threshold) => value
                .parse::<f64>()
                .map(|v| v > *threshold)
                .unwrap_or(false),
            RuleCondition::LessThan(threshold) => value
                .parse::<f64>()
                .map(|v| v < *threshold)
                .unwrap_or(false),
            RuleCondition::In(list) => list.contains(value),
            RuleCondition::Custom(_) => {
                // Custom conditions would need specialized handling
                false
            }
        }
    }

    /// Execute rule actions
    fn execute_actions(&mut self, actions: &[RuleAction]) -> Result<bool> {
        for action in actions {
            match action {
                RuleAction::SetValue { key, value } => {
                    self.context.insert(key.clone(), value.clone());
                }
                RuleAction::AddToList { key, value } => {
                    let current = self.context.get(key).cloned().unwrap_or_default();
                    let new_value = if current.is_empty() {
                        value.clone()
                    } else {
                        format!("{},{}", current, value)
                    };
                    self.context.insert(key.clone(), new_value);
                }
                RuleAction::Log {
                    level: _,
                    message: _,
                } => {
                    // Logging would be handled by tracing/logging framework
                }
                RuleAction::CallFunction {
                    function: _,
                    args: _,
                } => {
                    // Function calls would need to be handled by caller
                }
                RuleAction::Stop => {
                    return Ok(true); // Stop processing
                }
            }
        }
        Ok(false) // Continue processing
    }

    /// Execute all matching rules
    pub fn execute(&mut self) -> Result<Vec<String>> {
        let mut executed_rules = Vec::new();

        for rule in &self.rules.clone() {
            if !rule.enabled {
                continue;
            }

            // Check all conditions
            let all_conditions_met = rule
                .conditions
                .iter()
                .all(|(key, condition)| self.evaluate_condition(key, condition));

            if all_conditions_met {
                executed_rules.push(rule.id.clone());
                let should_stop = self.execute_actions(&rule.actions)?;
                if should_stop {
                    break;
                }
            }
        }

        Ok(executed_rules)
    }

    /// Clear all rules
    pub fn clear_rules(&mut self) {
        self.rules.clear();
    }

    /// Get all rules
    pub fn get_rules(&self) -> &[Rule] {
        &self.rules
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache for retrieval optimization
pub struct RetrievalCache {
    cache: HashMap<String, CachedResult>,
    max_size: usize,
    current_size: usize,
}

#[derive(Debug, Clone)]
struct CachedResult {
    #[allow(dead_code)]
    query_hash: String,
    results: Vec<Document>,
    #[allow(dead_code)]
    timestamp: std::time::SystemTime,
    access_count: usize,
}

impl RetrievalCache {
    /// Create a new retrieval cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            current_size: 0,
        }
    }

    /// Generate hash for query
    fn hash_query(&self, query: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(query.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Cache retrieval results
    pub fn cache_results(&mut self, query: &str, results: Vec<Document>) -> Result<()> {
        let query_hash = self.hash_query(query);

        let result_size = results.iter().map(|d| d.content.len()).sum::<usize>();

        // Evict if necessary
        while self.current_size + result_size > self.max_size && !self.cache.is_empty() {
            self.evict_lru();
        }

        if result_size <= self.max_size {
            self.cache.insert(
                query_hash.clone(),
                CachedResult {
                    query_hash,
                    results,
                    timestamp: std::time::SystemTime::now(),
                    access_count: 0,
                },
            );
            self.current_size += result_size;
        }

        Ok(())
    }

    /// Get cached results
    pub fn get_cached(&mut self, query: &str) -> Option<Vec<Document>> {
        let query_hash = self.hash_query(query);

        if let Some(cached) = self.cache.get_mut(&query_hash) {
            cached.access_count += 1;
            Some(cached.results.clone())
        } else {
            None
        }
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) {
        if let Some((key_to_remove, size)) = self
            .cache
            .iter()
            .min_by_key(|(_, v)| v.access_count)
            .map(|(k, v)| {
                let size = v.results.iter().map(|d| d.content.len()).sum::<usize>();
                (k.clone(), size)
            })
        {
            self.cache.remove(&key_to_remove);
            self.current_size = self.current_size.saturating_sub(size);
        }
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_size = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.len(),
            size_bytes: self.current_size,
            max_size_bytes: self.max_size,
            hit_rate: 0.0, // Would need to track hits/misses
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub size_bytes: usize,
    pub max_size_bytes: usize,
    pub hit_rate: f64,
}

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
    let first = name.chars().next().expect("BUG: empty check above should have returned");
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

        let conn = self.conn.lock().map_err(|e| {
            VaultError::StorageError(format!("Lock poisoned: {}", e))
        })?;

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
        let conn = self.conn.lock().map_err(|e| {
            VaultError::StorageError(format!("Lock poisoned: {}", e))
        })?;

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
        let conn = self.conn.lock().map_err(|e| {
            VaultError::StorageError(format!("Lock poisoned: {}", e))
        })?;

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
        let conn = self.conn.lock().map_err(|e| {
            VaultError::StorageError(format!("Lock poisoned: {}", e))
        })?;

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
        let conn = self.conn.lock().map_err(|e| {
            VaultError::StorageError(format!("Lock poisoned: {}", e))
        })?;

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
        let conn = self.conn.lock().map_err(|e| {
            VaultError::StorageError(format!("Lock poisoned: {}", e))
        })?;

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

        let conn = self.conn.lock().map_err(|e| {
            VaultError::StorageError(format!("Lock poisoned: {}", e))
        })?;

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

        let conn = self.conn.lock().map_err(|e| {
            VaultError::StorageError(format!("Lock poisoned: {}", e))
        })?;

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

/// Vector database trait for similarity search
pub trait VectorStore {
    /// Store a document with its embedding
    fn store_with_embedding(&mut self, doc: &Document) -> Result<()>;

    /// Search for similar documents using vector similarity
    fn search_similar(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<(String, f32)>>;

    /// Delete a document by ID
    fn delete_document(&mut self, id: &str) -> Result<()>;

    /// Get document count
    fn count(&self) -> Result<usize>;
}

/// Simple in-memory vector store using cosine similarity
pub struct SimpleVectorStore {
    documents: HashMap<String, Document>,
    index: Vec<(String, Vec<f32>)>,
}

impl SimpleVectorStore {
    /// Create a new simple vector store
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            index: Vec::new(),
        }
    }

    /// Load documents from a database
    pub fn from_documents(docs: Vec<Document>) -> Self {
        let mut store = Self::new();
        for doc in docs {
            if doc.embedding.is_some() {
                let _ = store.store_with_embedding(&doc);
            }
        }
        store
    }
}

impl Default for SimpleVectorStore {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorStore for SimpleVectorStore {
    fn store_with_embedding(&mut self, doc: &Document) -> Result<()> {
        if let Some(embedding) = &doc.embedding {
            self.index.push((doc.id.clone(), embedding.clone()));
            self.documents.insert(doc.id.clone(), doc.clone());
            Ok(())
        } else {
            Err(VaultError::InvalidInput(
                "Document must have an embedding".to_string(),
            ))
        }
    }

    fn search_similar(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<(String, f32)>> {
        let mut similarities: Vec<(String, f32)> = self
            .index
            .iter()
            .map(|(id, embedding)| {
                let similarity = cosine_similarity(query_embedding, embedding);
                (id.clone(), similarity)
            })
            .collect();

        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top results
        Ok(similarities.into_iter().take(limit).collect())
    }

    fn delete_document(&mut self, id: &str) -> Result<()> {
        self.documents.remove(id);
        self.index.retain(|(doc_id, _)| doc_id != id);
        Ok(())
    }

    fn count(&self) -> Result<usize> {
        Ok(self.documents.len())
    }
}

/// Qdrant vector database client (optional feature)
#[cfg(feature = "vector-db")]
pub struct QdrantVectorStore {
    client: qdrant_client::client::QdrantClient,
    collection_name: String,
}

#[cfg(feature = "vector-db")]
impl QdrantVectorStore {
    /// Create a new Qdrant vector store
    pub async fn new(url: &str, collection_name: String) -> Result<Self> {
        let client = qdrant_client::client::QdrantClient::from_url(url)
            .build()
            .map_err(|e| VaultError::StorageError(format!("Failed to connect to Qdrant: {}", e)))?;

        Ok(Self {
            client,
            collection_name,
        })
    }

    /// Create collection if it doesn't exist
    pub async fn create_collection(&self, vector_size: u64) -> Result<()> {
        use qdrant_client::qdrant::{CreateCollection, Distance, VectorParams, VectorsConfig};

        let vectors_config = VectorsConfig {
            config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                VectorParams {
                    size: vector_size,
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                },
            )),
        };

        self.client
            .create_collection(&CreateCollection {
                collection_name: self.collection_name.clone(),
                vectors_config: Some(vectors_config),
                ..Default::default()
            })
            .await
            .map_err(|e| VaultError::StorageError(format!("Failed to create collection: {}", e)))?;

        Ok(())
    }

    /// Store document with embedding
    pub async fn store_document_async(&self, doc: &Document) -> Result<()> {
        use qdrant_client::qdrant::{PointStruct, UpsertPoints};

        if let Some(embedding) = &doc.embedding {
            let point = PointStruct::new(
                doc.id.clone(),
                embedding.clone(),
                serde_json::to_value(&doc.metadata).unwrap_or_default(),
            );

            self.client
                .upsert_points(&UpsertPoints {
                    collection_name: self.collection_name.clone(),
                    points: vec![point],
                    ..Default::default()
                })
                .await
                .map_err(|e| VaultError::StorageError(format!("Failed to upsert point: {}", e)))?;

            Ok(())
        } else {
            Err(VaultError::InvalidInput(
                "Document must have an embedding".to_string(),
            ))
        }
    }

    /// Search for similar documents
    pub async fn search_similar_async(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, f32)>> {
        use qdrant_client::qdrant::SearchPoints;

        let results = self
            .client
            .search_points(&SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: query_embedding.to_vec(),
                limit: limit as u64,
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await
            .map_err(|e| VaultError::StorageError(format!("Failed to search: {}", e)))?;

        let scored_points: Vec<(String, f32)> = results
            .result
            .into_iter()
            .map(|point| {
                let id = point.id.map(|id| id.to_string()).unwrap_or_default();
                let score = point.score;
                (id, score)
            })
            .collect();

        Ok(scored_points)
    }

    /// Delete document by ID
    pub async fn delete_document_async(&self, id: &str) -> Result<()> {
        use qdrant_client::qdrant::{
            points_selector::PointsSelectorOneOf, DeletePoints, PointsIdsList, PointsSelector,
        };

        self.client
            .delete_points(&DeletePoints {
                collection_name: self.collection_name.clone(),
                points: Some(PointsSelector {
                    points_selector_one_of: Some(PointsSelectorOneOf::Points(PointsIdsList {
                        ids: vec![id.into()],
                    })),
                }),
                ..Default::default()
            })
            .await
            .map_err(|e| VaultError::StorageError(format!("Failed to delete: {}", e)))?;

        Ok(())
    }
}

/// MCP (Model Context Protocol) Tool Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Input schema (JSON Schema)
    pub input_schema: JsonValue,

    /// Tool metadata
    pub metadata: HashMap<String, String>,
}

impl MCPTool {
    /// Create a new MCP tool
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            metadata: HashMap::new(),
        }
    }

    /// Add a parameter to the tool schema
    pub fn add_parameter(
        mut self,
        name: &str,
        param_type: &str,
        description: &str,
        required: bool,
    ) -> Self {
        if let Some(properties) = self.input_schema.get_mut("properties") {
            if let Some(props) = properties.as_object_mut() {
                props.insert(
                    name.to_string(),
                    serde_json::json!({
                        "type": param_type,
                        "description": description
                    }),
                );
            }
        }

        if required {
            if let Some(required_array) = self.input_schema.get_mut("required") {
                if let Some(arr) = required_array.as_array_mut() {
                    arr.push(serde_json::json!(name));
                }
            }
        }

        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Tool execution context
#[derive(Debug, Clone)]
pub struct ToolContext {
    /// Document store reference
    pub document_store: Option<String>,

    /// Knowledge base reference
    pub knowledge_base: Option<String>,

    /// Additional context data
    pub data: HashMap<String, String>,
}

impl ToolContext {
    /// Create a new tool context
    pub fn new() -> Self {
        Self {
            document_store: None,
            knowledge_base: None,
            data: HashMap::new(),
        }
    }

    /// Set document store reference
    pub fn with_document_store(mut self, store_id: String) -> Self {
        self.document_store = Some(store_id);
        self
    }

    /// Set knowledge base reference
    pub fn with_knowledge_base(mut self, kb_id: String) -> Self {
        self.knowledge_base = Some(kb_id);
        self
    }

    /// Add context data
    pub fn with_data(mut self, key: String, value: String) -> Self {
        self.data.insert(key, value);
        self
    }
}

impl Default for ToolContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Success status
    pub success: bool,

    /// Result data (JSON)
    pub data: JsonValue,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Execution metadata
    pub metadata: HashMap<String, String>,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(data: JsonValue) -> Self {
        Self {
            success: true,
            data,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a failed result
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            data: serde_json::json!(null),
            error: Some(error),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Tool executor trait
pub trait ToolExecutor {
    /// Execute a tool with given parameters
    fn execute(
        &self,
        tool_name: &str,
        params: JsonValue,
        context: &ToolContext,
    ) -> Result<ToolResult>;

    /// List available tools
    fn list_tools(&self) -> Vec<MCPTool>;

    /// Get tool by name
    fn get_tool(&self, name: &str) -> Option<&MCPTool>;
}

/// Type alias for tool executor functions
type ToolExecutorFn = Box<dyn Fn(JsonValue, &ToolContext) -> Result<ToolResult>>;

/// MCP Server for tool management
pub struct MCPServer {
    tools: HashMap<String, MCPTool>,
    executors: HashMap<String, ToolExecutorFn>,
}

impl MCPServer {
    /// Create a new MCP server
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            executors: HashMap::new(),
        }
    }

    /// Register a tool with executor function
    pub fn register_tool<F>(&mut self, tool: MCPTool, executor: F) -> Result<()>
    where
        F: Fn(JsonValue, &ToolContext) -> Result<ToolResult> + 'static,
    {
        let name = tool.name.clone();
        self.tools.insert(name.clone(), tool);
        self.executors.insert(name, Box::new(executor));
        Ok(())
    }

    /// Execute a tool
    pub fn execute_tool(
        &self,
        tool_name: &str,
        params: JsonValue,
        context: &ToolContext,
    ) -> Result<ToolResult> {
        let executor = self
            .executors
            .get(tool_name)
            .ok_or_else(|| VaultError::InvalidInput(format!("Tool '{}' not found", tool_name)))?;

        executor(params, context)
    }

    /// List all registered tools
    pub fn list_tools(&self) -> Vec<&MCPTool> {
        self.tools.values().collect()
    }

    /// Get tool by name
    pub fn get_tool(&self, name: &str) -> Option<&MCPTool> {
        self.tools.get(name)
    }

    /// Register built-in RAG tools
    pub fn register_builtin_tools(&mut self) -> Result<()> {
        // Tool: Search documents
        let search_tool = MCPTool::new(
            "search_documents".to_string(),
            "Search for similar documents using vector embeddings".to_string(),
        )
        .add_parameter("query", "string", "Search query text", true)
        .add_parameter("top_k", "number", "Number of results to return", false)
        .add_parameter("threshold", "number", "Minimum similarity threshold", false);

        self.register_tool(search_tool, |params, _ctx| {
            let query = params
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| VaultError::InvalidInput("Missing 'query' parameter".to_string()))?;

            let top_k = params.get("top_k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

            let _threshold = params
                .get("threshold")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5);

            // NOTE: MCP tool closures are stateless — actual search requires a DocumentStore
            // or database connection injected via ToolContext. Without it, we return an honest
            // response indicating the tool was invoked but needs integration with a backing store.
            Ok(ToolResult::success(serde_json::json!({
                "query": query,
                "top_k": top_k,
                "results": [],
                "note": "Connect a DocumentStore or SQLiteDatabase via ToolContext for live search results"
            })))
        })?;

        // Tool: Add document
        let add_doc_tool = MCPTool::new(
            "add_document".to_string(),
            "Add a document to the knowledge base".to_string(),
        )
        .add_parameter("id", "string", "Document ID", true)
        .add_parameter("content", "string", "Document content", true)
        .add_parameter("metadata", "object", "Document metadata", false);

        self.register_tool(add_doc_tool, |params, _ctx| {
            let id = params
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| VaultError::InvalidInput("Missing 'id' parameter".to_string()))?;

            let content = params
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    VaultError::InvalidInput("Missing 'content' parameter".to_string())
                })?;

            let metadata = params
                .get("metadata")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));

            Ok(ToolResult::success(serde_json::json!({
                "id": id,
                "content_length": content.len(),
                "metadata": metadata,
                "status": "accepted",
                "note": "Connect a DocumentStore or SQLiteDatabase via ToolContext for persistent storage"
            })))
        })?;

        // Tool: Chunk text
        let chunk_tool = MCPTool::new(
            "chunk_text".to_string(),
            "Split text into chunks for processing".to_string(),
        )
        .add_parameter("text", "string", "Text to chunk", true)
        .add_parameter("chunk_size", "number", "Size of each chunk", false)
        .add_parameter("overlap", "number", "Overlap between chunks", false);

        self.register_tool(chunk_tool, |params, _ctx| {
            let text = params
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| VaultError::InvalidInput("Missing 'text' parameter".to_string()))?;

            let chunk_size = params
                .get("chunk_size")
                .and_then(|v| v.as_u64())
                .unwrap_or(512) as usize;

            let overlap = params.get("overlap").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

            // Actually split text into chunks
            let chars: Vec<char> = text.chars().collect();
            let mut chunks = Vec::new();
            let mut start = 0;

            while start < chars.len() {
                let end = (start + chunk_size).min(chars.len());
                let chunk_text: String = chars[start..end].iter().collect();
                chunks.push(serde_json::json!({
                    "index": chunks.len(),
                    "start": start,
                    "end": end,
                    "text": chunk_text
                }));

                if end >= chars.len() {
                    break;
                }
                start = end - overlap.min(end);
            }

            Ok(ToolResult::success(serde_json::json!({
                "text_length": text.len(),
                "chunk_size": chunk_size,
                "overlap": overlap,
                "num_chunks": chunks.len(),
                "chunks": chunks
            })))
        })?;

        // Tool: Execute rule
        let rule_tool = MCPTool::new(
            "execute_rule".to_string(),
            "Execute a business rule with given context".to_string(),
        )
        .add_parameter("rule_id", "string", "Rule identifier", true)
        .add_parameter("context", "object", "Rule execution context", true);

        self.register_tool(rule_tool, |params, _ctx| {
            let rule_id = params
                .get("rule_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    VaultError::InvalidInput("Missing 'rule_id' parameter".to_string())
                })?;

            let context = params
                .get("context")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));

            // NOTE: Rule execution requires a RuleEngine instance with registered rules.
            // The MCP tool accepts the request but needs integration via ToolContext.
            Ok(ToolResult::success(serde_json::json!({
                "rule_id": rule_id,
                "context_received": context,
                "status": "accepted",
                "note": "Connect a RuleEngine via ToolContext for live rule execution"
            })))
        })?;

        Ok(())
    }
}

impl Default for MCPServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_store() {
        let mut store = DocumentStore::new();

        let doc = Document {
            id: "doc1".to_string(),
            content: "Test content".to_string(),
            metadata: HashMap::new(),
            embedding: Some(vec![0.1, 0.2, 0.3]),
            chunk_info: None,
        };

        store.add_document(doc.clone()).unwrap();
        assert_eq!(store.count(), 1);

        let retrieved = store.get_document("doc1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "Test content");
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_knowledge_base_chunking() {
        let config = KnowledgeBaseConfig {
            chunk_size: 10,
            chunk_overlap: 2,
            ..Default::default()
        };

        let kb = KnowledgeBase::new("test".to_string(), config);
        let text = "This is a test document with multiple chunks";
        let chunks = kb.chunk_text(text, "doc1");

        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].chunk_info.as_ref().unwrap().chunk_index, 0);
    }

    #[test]
    fn test_rule_engine() {
        let mut engine = RuleEngine::new();

        let rule = Rule {
            id: "rule1".to_string(),
            name: "Test Rule".to_string(),
            conditions: {
                let mut cond = HashMap::new();
                cond.insert(
                    "status".to_string(),
                    RuleCondition::Equals("active".to_string()),
                );
                cond
            },
            actions: vec![RuleAction::SetValue {
                key: "processed".to_string(),
                value: "true".to_string(),
            }],
            priority: 10,
            enabled: true,
        };

        engine.add_rule(rule);
        engine.set_context("status".to_string(), "active".to_string());

        let executed = engine.execute().unwrap();
        assert_eq!(executed.len(), 1);
        assert_eq!(engine.get_context("processed"), Some(&"true".to_string()));
    }

    #[test]
    fn test_retrieval_cache() {
        let mut cache = RetrievalCache::new(1024);

        let doc = Document {
            id: "doc1".to_string(),
            content: "Test".to_string(),
            metadata: HashMap::new(),
            embedding: None,
            chunk_info: None,
        };

        cache.cache_results("query1", vec![doc.clone()]).unwrap();

        let cached = cache.get_cached("query1");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap()[0].id, "doc1");
    }

    #[test]
    fn test_in_memory_database() {
        let mut db = InMemoryDatabase::new();
        db.create_table("users".to_string());

        let mut data = HashMap::new();
        data.insert("id".to_string(), "1".to_string());
        data.insert("name".to_string(), "Alice".to_string());

        db.insert("users", data).unwrap();

        let results = db.query("users WHERE id=1").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("name"), Some(&"Alice".to_string()));
    }

    #[test]
    fn test_mcp_tool_creation() {
        let tool = MCPTool::new("test_tool".to_string(), "A test tool".to_string())
            .add_parameter("param1", "string", "First parameter", true)
            .add_parameter("param2", "number", "Second parameter", false);

        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool");

        let props = tool.input_schema.get("properties").unwrap();
        assert!(props.get("param1").is_some());
        assert!(props.get("param2").is_some());
    }

    #[test]
    fn test_tool_context() {
        let ctx = ToolContext::new()
            .with_document_store("store1".to_string())
            .with_knowledge_base("kb1".to_string())
            .with_data("key1".to_string(), "value1".to_string());

        assert_eq!(ctx.document_store, Some("store1".to_string()));
        assert_eq!(ctx.knowledge_base, Some("kb1".to_string()));
        assert_eq!(ctx.data.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_tool_result() {
        let success = ToolResult::success(serde_json::json!({"result": "ok"}));
        assert!(success.success);
        assert!(success.error.is_none());

        let failure = ToolResult::failure("Error occurred".to_string());
        assert!(!failure.success);
        assert_eq!(failure.error, Some("Error occurred".to_string()));
    }

    #[test]
    fn test_mcp_server_registration() {
        let mut server = MCPServer::new();

        let tool = MCPTool::new("simple_tool".to_string(), "A simple tool".to_string());

        server
            .register_tool(tool, |_params, _ctx| {
                Ok(ToolResult::success(serde_json::json!({"status": "ok"})))
            })
            .unwrap();

        let tools = server.list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "simple_tool");
    }

    #[test]
    fn test_mcp_server_execution() {
        let mut server = MCPServer::new();

        let tool = MCPTool::new("echo_tool".to_string(), "Echoes the input".to_string());

        server
            .register_tool(tool, |params, _ctx| Ok(ToolResult::success(params)))
            .unwrap();

        let ctx = ToolContext::new();
        let params = serde_json::json!({"message": "hello"});
        let result = server
            .execute_tool("echo_tool", params.clone(), &ctx)
            .unwrap();

        assert!(result.success);
        assert_eq!(result.data, params);
    }

    #[test]
    fn test_mcp_builtin_tools() {
        let mut server = MCPServer::new();
        server.register_builtin_tools().unwrap();

        let tools = server.list_tools();
        assert!(tools.len() >= 4); // At least 4 built-in tools

        // Check for specific tools
        assert!(server.get_tool("search_documents").is_some());
        assert!(server.get_tool("add_document").is_some());
        assert!(server.get_tool("chunk_text").is_some());
        assert!(server.get_tool("execute_rule").is_some());
    }

    #[test]
    fn test_builtin_search_tool() {
        let mut server = MCPServer::new();
        server.register_builtin_tools().unwrap();

        let ctx = ToolContext::new();
        let params = serde_json::json!({
            "query": "test query",
            "top_k": 3
        });

        let result = server
            .execute_tool("search_documents", params, &ctx)
            .unwrap();
        assert!(result.success);
        assert_eq!(
            result.data.get("query").unwrap().as_str().unwrap(),
            "test query"
        );
        assert_eq!(result.data.get("top_k").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_builtin_chunk_tool() {
        let mut server = MCPServer::new();
        server.register_builtin_tools().unwrap();

        let ctx = ToolContext::new();
        let params = serde_json::json!({
            "text": "This is a test text that will be chunked",
            "chunk_size": 10,
            "overlap": 2
        });

        let result = server.execute_tool("chunk_text", params, &ctx).unwrap();
        assert!(result.success);
        assert_eq!(result.data.get("chunk_size").unwrap().as_u64().unwrap(), 10);
        assert_eq!(result.data.get("overlap").unwrap().as_u64().unwrap(), 2);
        // Verify actual chunks were produced
        let chunks = result.data.get("chunks").unwrap().as_array().unwrap();
        assert!(!chunks.is_empty());
        assert_eq!(
            result.data.get("num_chunks").unwrap().as_u64().unwrap(),
            chunks.len() as u64
        );
    }
}
