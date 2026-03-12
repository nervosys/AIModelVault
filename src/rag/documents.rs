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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc(id: &str, content: &str, embedding: Option<Vec<f32>>) -> Document {
        Document {
            id: id.to_string(),
            content: content.to_string(),
            metadata: HashMap::new(),
            embedding,
            chunk_info: None,
        }
    }

    #[test]
    fn test_document_store_default() {
        let store = DocumentStore::default();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_document_store_add_and_get() {
        let mut store = DocumentStore::new();
        let doc = make_doc("d1", "hello", Some(vec![1.0, 0.0]));
        store.add_document(doc).unwrap();

        assert_eq!(store.count(), 1);
        let got = store.get_document("d1").unwrap();
        assert_eq!(got.content, "hello");

        // Nonexistent
        assert!(store.get_document("nonexistent").is_none());
    }

    #[test]
    fn test_document_store_search_similar() {
        let mut store = DocumentStore::new();
        store
            .add_document(make_doc("a", "alpha", Some(vec![1.0, 0.0])))
            .unwrap();
        store
            .add_document(make_doc("b", "beta", Some(vec![0.0, 1.0])))
            .unwrap();

        let results = store.search_similar(&[1.0, 0.0], 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "a"); // most similar
    }

    #[test]
    fn test_document_store_delete() {
        let mut store = DocumentStore::new();
        store
            .add_document(make_doc("d1", "x", Some(vec![1.0])))
            .unwrap();
        store
            .add_document(make_doc("d2", "y", Some(vec![0.0])))
            .unwrap();

        store.delete_document("d1").unwrap();
        assert_eq!(store.count(), 1);
        assert!(store.get_document("d1").is_none());
    }

    #[test]
    fn test_document_store_get_all() {
        let mut store = DocumentStore::new();
        store.add_document(make_doc("a", "x", None)).unwrap();
        store.add_document(make_doc("b", "y", None)).unwrap();
        assert_eq!(store.get_all_documents().len(), 2);
    }

    #[test]
    fn test_document_store_clear() {
        let mut store = DocumentStore::new();
        store.add_document(make_doc("a", "x", None)).unwrap();
        store.clear();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_document_without_embedding_not_indexed() {
        let mut store = DocumentStore::new();
        store.add_document(make_doc("noem", "x", None)).unwrap();
        // Search should return nothing since doc has no embedding
        let results = store.search_similar(&[1.0], 10);
        assert!(results.is_empty());
    }
}
