//! Knowledge base for RAG systems.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::documents::{ChunkInfo, Document, DocumentStore};

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
    pub fn add(&mut self, doc: Document) -> crate::error::Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_base_config_default() {
        let cfg = KnowledgeBaseConfig::default();
        assert_eq!(cfg.embedding_dim, 384);
        assert_eq!(cfg.chunk_size, 512);
        assert_eq!(cfg.chunk_overlap, 50);
        assert_eq!(cfg.max_results, 5);
        assert!((cfg.similarity_threshold - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_knowledge_base_add_and_retrieve() {
        let kb = KnowledgeBase::new("test".to_string(), KnowledgeBaseConfig::default());
        // Empty retrieve
        let results = kb.retrieve(&[1.0, 0.0], None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_knowledge_base_add_doc() {
        let mut kb = KnowledgeBase::new("test".to_string(), KnowledgeBaseConfig::default());
        let doc = Document {
            id: "d1".to_string(),
            content: "hello".to_string(),
            metadata: HashMap::new(),
            embedding: Some(vec![1.0, 0.0]),
            chunk_info: None,
        };
        kb.add(doc).unwrap();
        assert_eq!(kb.store.count(), 1);
    }

    #[test]
    fn test_knowledge_base_retrieve_with_threshold() {
        let cfg = KnowledgeBaseConfig {
            similarity_threshold: 0.9,
            ..KnowledgeBaseConfig::default()
        };
        let mut kb = KnowledgeBase::new("test".to_string(), cfg);

        let doc = Document {
            id: "d1".to_string(),
            content: "hello".to_string(),
            metadata: HashMap::new(),
            embedding: Some(vec![1.0, 0.0]),
            chunk_info: None,
        };
        kb.add(doc).unwrap();

        // Query orthogonal — should be filtered by threshold
        let results = kb.retrieve(&[0.0, 1.0], Some(10));
        assert!(results.is_empty());

        // Query same direction — should pass threshold
        let results = kb.retrieve(&[1.0, 0.0], Some(10));
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_chunk_text() {
        let cfg = KnowledgeBaseConfig {
            chunk_size: 10,
            chunk_overlap: 3,
            ..KnowledgeBaseConfig::default()
        };
        let kb = KnowledgeBase::new("kb".to_string(), cfg);

        let text = "abcdefghijklmnopqrstuvwxyz"; // 26 chars
        let chunks = kb.chunk_text(text, "doc1");
        assert!(chunks.len() >= 3);

        // First chunk
        assert_eq!(chunks[0].content.len(), 10);
        assert_eq!(chunks[0].id, "doc1_chunk_0");

        // Verify chunk_info
        for chunk in &chunks {
            let info = chunk.chunk_info.as_ref().unwrap();
            assert_eq!(info.total_chunks, chunks.len());
            assert_eq!(info.parent_id, Some("doc1".to_string()));
        }
    }
}
