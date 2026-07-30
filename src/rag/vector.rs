//! Vector store abstractions and implementations.

use crate::error::{Result, VaultError};
use std::collections::HashMap;

use super::documents::Document;

/// Calculate cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
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
    client: qdrant_client::Qdrant,
    collection_name: String,
}

#[cfg(feature = "vector-db")]
impl QdrantVectorStore {
    /// Create a new Qdrant vector store
    ///
    /// `async` is kept deliberately: it is public API, every other constructor
    /// in this module is async, and dropping it would break callers for a
    /// cosmetic lint.
    #[allow(clippy::unused_async)]
    pub async fn new(url: &str, collection_name: String) -> Result<Self> {
        let client = qdrant_client::Qdrant::from_url(url)
            .build()
            .map_err(|e| VaultError::StorageError(format!("Failed to connect to Qdrant: {}", e)))?;

        Ok(Self {
            client,
            collection_name,
        })
    }

    /// Create collection if it doesn't exist
    pub async fn create_collection(&self, vector_size: u64) -> Result<()> {
        use qdrant_client::qdrant::{CreateCollectionBuilder, Distance, VectorParamsBuilder};

        self.client
            .create_collection(
                CreateCollectionBuilder::new(&self.collection_name)
                    .vectors_config(VectorParamsBuilder::new(vector_size, Distance::Cosine)),
            )
            .await
            .map_err(|e| VaultError::StorageError(format!("Failed to create collection: {}", e)))?;

        Ok(())
    }

    /// Store document with embedding
    pub async fn store_document_async(&self, doc: &Document) -> Result<()> {
        use qdrant_client::qdrant::{PointStruct, UpsertPointsBuilder};

        if let Some(embedding) = &doc.embedding {
            let payload: qdrant_client::qdrant::value::Kind =
                qdrant_client::qdrant::value::Kind::StringValue(
                    serde_json::to_string(&doc.metadata).unwrap_or_default(),
                );
            let point = PointStruct::new(
                doc.id.clone(),
                embedding.clone(),
                [(
                    "metadata".to_string(),
                    qdrant_client::qdrant::Value {
                        kind: Some(payload),
                    },
                )],
            );

            self.client
                .upsert_points(UpsertPointsBuilder::new(&self.collection_name, vec![point]))
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
        use qdrant_client::qdrant::SearchPointsBuilder;

        let results = self
            .client
            .search_points(
                SearchPointsBuilder::new(
                    &self.collection_name,
                    query_embedding.to_vec(),
                    limit as u64,
                )
                .with_payload(true),
            )
            .await
            .map_err(|e| VaultError::StorageError(format!("Failed to search: {}", e)))?;

        let scored_points: Vec<(String, f32)> = results
            .result
            .into_iter()
            .map(|point| {
                let id = point
                    .id
                    .map(|id| match id.point_id_options {
                        Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(u)) => u,
                        Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(n)) => {
                            n.to_string()
                        }
                        None => String::new(),
                    })
                    .unwrap_or_default();
                let score = point.score;
                (id, score)
            })
            .collect();

        Ok(scored_points)
    }

    /// Delete document by ID
    pub async fn delete_document_async(&self, id: &str) -> Result<()> {
        use qdrant_client::qdrant::{DeletePointsBuilder, PointId};

        let point_id: PointId = id.into();
        self.client
            .delete_points(DeletePointsBuilder::new(&self.collection_name).points(vec![point_id]))
            .await
            .map_err(|e| VaultError::StorageError(format!("Failed to delete: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
// Exact float comparison is intentional here: these assert on literal
// constants that round-trip bit-for-bit, not on computed results.
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::rag::documents::Document;
    use std::collections::HashMap;

    fn make_doc(id: &str, embedding: Vec<f32>) -> Document {
        Document {
            id: id.to_string(),
            content: format!("content for {}", id),
            metadata: HashMap::new(),
            embedding: Some(embedding),
            chunk_info: None,
        }
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-5);
    }

    #[test]
    fn test_cosine_similarity_different_lengths() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_cosine_similarity_zero_vector() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_simple_vector_store_default() {
        let store = SimpleVectorStore::default();
        assert_eq!(store.count().unwrap(), 0);
    }

    #[test]
    fn test_simple_vector_store_crud() {
        let mut store = SimpleVectorStore::new();

        let doc = make_doc("d1", vec![1.0, 0.0, 0.0]);
        store.store_with_embedding(&doc).unwrap();
        assert_eq!(store.count().unwrap(), 1);

        let doc2 = make_doc("d2", vec![0.0, 1.0, 0.0]);
        store.store_with_embedding(&doc2).unwrap();
        assert_eq!(store.count().unwrap(), 2);

        // Search
        let results = store.search_similar(&[1.0, 0.0, 0.0], 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "d1"); // most similar

        // Delete
        store.delete_document("d1").unwrap();
        assert_eq!(store.count().unwrap(), 1);
    }

    #[test]
    fn test_simple_vector_store_no_embedding_error() {
        let mut store = SimpleVectorStore::new();
        let doc = Document {
            id: "no-emb".to_string(),
            content: "test".to_string(),
            metadata: HashMap::new(),
            embedding: None,
            chunk_info: None,
        };
        assert!(store.store_with_embedding(&doc).is_err());
    }

    #[test]
    fn test_simple_vector_store_from_documents() {
        let docs = vec![
            make_doc("a", vec![1.0, 0.0]),
            make_doc("b", vec![0.0, 1.0]),
            Document {
                id: "c".to_string(),
                content: "no embedding".to_string(),
                metadata: HashMap::new(),
                embedding: None,
                chunk_info: None,
            },
        ];
        let store = SimpleVectorStore::from_documents(docs);
        // Only 2 docs have embeddings
        assert_eq!(store.count().unwrap(), 2);
    }
}
