//! Cache management for retrieval optimization.

use crate::error::Result;
use std::collections::HashMap;

use super::documents::Document;

/// Cache for retrieval optimization
pub struct RetrievalCache {
    cache: HashMap<String, CachedResult>,
    max_size: usize,
    current_size: usize,
}

#[derive(Debug, Clone)]
struct CachedResult {
    results: Vec<Document>,
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
                query_hash,
                CachedResult {
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

    /// Evict least recently used entry (lowest access_count; oldest timestamp breaks ties)
    fn evict_lru(&mut self) {
        if let Some((key_to_remove, size)) = self
            .cache
            .iter()
            .min_by(|(_, a), (_, b)| {
                a.access_count
                    .cmp(&b.access_count)
                    .then_with(|| a.timestamp.cmp(&b.timestamp))
            })
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
