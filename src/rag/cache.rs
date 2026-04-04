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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::documents::Document;

    fn make_doc(id: &str, content: &str) -> Document {
        Document {
            id: id.to_string(),
            content: content.to_string(),
            metadata: HashMap::new(),
            embedding: None,
            chunk_info: None,
        }
    }

    #[test]
    fn test_cache_store_and_retrieve() {
        let mut cache = RetrievalCache::new(10_000);
        let docs = vec![make_doc("d1", "hello world")];
        cache.cache_results("query1", docs.clone()).unwrap();

        let cached = cache.get_cached("query1");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);

        // Miss
        assert!(cache.get_cached("nonexistent").is_none());
    }

    #[test]
    fn test_cache_eviction() {
        // Max size = 20 bytes
        let mut cache = RetrievalCache::new(20);
        let doc1 = make_doc("d1", "twelve chars"); // 12 bytes
        cache.cache_results("q1", vec![doc1]).unwrap();

        // This should trigger eviction of q1
        let doc2 = make_doc("d2", "another twelve"); // 14 bytes
        cache.cache_results("q2", vec![doc2]).unwrap();

        // q1 should have been evicted
        assert!(cache.get_cached("q1").is_none());
        assert!(cache.get_cached("q2").is_some());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = RetrievalCache::new(10_000);
        cache
            .cache_results("q1", vec![make_doc("d1", "x")])
            .unwrap();
        cache.clear();
        assert!(cache.get_cached("q1").is_none());
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = RetrievalCache::new(1000);
        cache
            .cache_results("q1", vec![make_doc("d1", "data")])
            .unwrap();
        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
        assert_eq!(stats.max_size_bytes, 1000);
        assert!(stats.size_bytes > 0);
    }

    #[test]
    fn test_cache_oversized_item_rejected() {
        // Max size = 5, doc with 10-byte content won't fit
        let mut cache = RetrievalCache::new(5);
        cache
            .cache_results("q", vec![make_doc("d", "0123456789")])
            .unwrap();
        // Item too large, should not be cached
        assert!(cache.get_cached("q").is_none());
    }

    #[test]
    fn test_evict_lru_tiebreaker_by_timestamp() {
        // Covers L84-86 — LRU eviction tiebreaker using timestamp
        use crate::rag::documents::Document;
        use std::collections::HashMap;

        // Create cache with capacity for only 2 entries
        let mut cache = RetrievalCache::new(8);

        // Insert two entries so they have the same access_count (1)
        let doc1 = Document {
            id: "doc1".to_string(),
            content: "aaaa".to_string(), // 4 bytes
            metadata: HashMap::new(),
            embedding: None,
            chunk_info: None,
        };
        let doc2 = Document {
            id: "doc2".to_string(),
            content: "bbbb".to_string(), // 4 bytes
            metadata: HashMap::new(),
            embedding: None,
            chunk_info: None,
        };
        let doc3 = Document {
            id: "doc3".to_string(),
            content: "cccc".to_string(), // 4 bytes
            metadata: HashMap::new(),
            embedding: None,
            chunk_info: None,
        };

        // Set results — first entry gets older timestamp
        let _ = cache.cache_results("query1", vec![doc1]);
        // Tiny sleep to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(10));
        let _ = cache.cache_results("query2", vec![doc2]);
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Both have access_count = 0 after set (same). Adding a third should evict the oldest.
        let _ = cache.cache_results("query3", vec![doc3]);

        // query1 (oldest timestamp) should have been evicted
        assert!(
            cache.get_cached("query1").is_none(),
            "Oldest entry should be evicted"
        );
        // query2 and query3 should still be present
        assert!(cache.get_cached("query3").is_some());
    }
}
