//! Document storage and retrieval for RAG systems.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::vector::cosine_similarity;

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
