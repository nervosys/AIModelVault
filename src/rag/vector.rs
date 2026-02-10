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
