//! Model tagging and search — attach tags and query models by metadata.
//!
//! Tags are stored in a JSON file alongside the vault's `versions.json`.
//! Each model can have arbitrary string tags and key-value annotations.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;

// ── Types ────────────────────────────────────────────────────────────────────

/// Tag store — persists per-model tags and annotations to JSON.
#[derive(Debug)]
pub struct TagStore {
    path: PathBuf,
    data: TagData,
}

/// Serialized tag data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TagData {
    /// model_name → set of tags
    pub tags: HashMap<String, BTreeSet<String>>,
    /// model_name → key-value annotations
    pub annotations: HashMap<String, BTreeMap<String, String>>,
}

/// A search query against the tag store.
#[derive(Debug, Default)]
pub struct SearchQuery {
    /// All of these tags must be present
    pub tags: Vec<String>,
    /// Annotation key-value filters (all must match)
    pub annotations: Vec<(String, String)>,
    /// Substring match on model name
    pub name_pattern: Option<String>,
}

/// A search result entry.
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub model: String,
    pub tags: BTreeSet<String>,
    pub annotations: BTreeMap<String, String>,
}

// ── Implementation ───────────────────────────────────────────────────────────

impl TagStore {
    const FILE_NAME: &'static str = "tags.json";

    /// Open or create a tag store in `vault_path`.
    pub fn new(vault_path: &Path) -> Result<Self> {
        let path = vault_path.join(Self::FILE_NAME);
        let data = if path.exists() {
            let contents = fs::read_to_string(&path)?;
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            TagData::default()
        };
        Ok(Self { path, data })
    }

    /// Persist to disk.
    fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.path, json)?;
        crate::permissions::restrict_file(&self.path)?;
        Ok(())
    }

    // ── Tag operations ───────────────────────────────────────────────────

    /// Add one or more tags to a model.
    pub fn add_tags(&mut self, model: &str, tags: &[String]) -> Result<()> {
        let set = self.data.tags.entry(model.to_string()).or_default();
        for tag in tags {
            let tag = Self::normalize_tag(tag);
            if tag.is_empty() {
                continue;
            }
            set.insert(tag);
        }
        self.save()
    }

    /// Remove tags from a model.
    pub fn remove_tags(&mut self, model: &str, tags: &[String]) -> Result<()> {
        if let Some(set) = self.data.tags.get_mut(model) {
            for tag in tags {
                set.remove(&Self::normalize_tag(tag));
            }
            if set.is_empty() {
                self.data.tags.remove(model);
            }
        }
        self.save()
    }

    /// List tags for a model.
    pub fn get_tags(&self, model: &str) -> BTreeSet<String> {
        self.data.tags.get(model).cloned().unwrap_or_default()
    }

    /// List all known tags across all models.
    pub fn all_tags(&self) -> BTreeSet<String> {
        self.data
            .tags
            .values()
            .flat_map(|s| s.iter().cloned())
            .collect()
    }

    // ── Annotation operations ────────────────────────────────────────────

    /// Set an annotation (key-value) on a model.
    pub fn set_annotation(&mut self, model: &str, key: &str, value: &str) -> Result<()> {
        self.data
            .annotations
            .entry(model.to_string())
            .or_default()
            .insert(key.to_string(), value.to_string());
        self.save()
    }

    /// Remove an annotation.
    pub fn remove_annotation(&mut self, model: &str, key: &str) -> Result<()> {
        if let Some(map) = self.data.annotations.get_mut(model) {
            map.remove(key);
            if map.is_empty() {
                self.data.annotations.remove(model);
            }
        }
        self.save()
    }

    /// Get annotations for a model.
    pub fn get_annotations(&self, model: &str) -> BTreeMap<String, String> {
        self.data
            .annotations
            .get(model)
            .cloned()
            .unwrap_or_default()
    }

    // ── Search ───────────────────────────────────────────────────────────

    /// Search models by tags, annotations, and name pattern.
    pub fn search(&self, query: &SearchQuery, known_models: &[String]) -> Vec<SearchResult> {
        let candidates: Vec<&str> = if query.tags.is_empty()
            && query.annotations.is_empty()
            && query.name_pattern.is_none()
        {
            known_models.iter().map(|s| s.as_str()).collect()
        } else {
            known_models.iter().map(|s| s.as_str()).collect()
        };

        candidates
            .into_iter()
            .filter(|model| {
                // Name filter
                if let Some(ref pat) = query.name_pattern {
                    let pat_lower = pat.to_lowercase();
                    if !model.to_lowercase().contains(&pat_lower) {
                        return false;
                    }
                }
                // Tag filter — all required tags must be present
                if !query.tags.is_empty() {
                    let model_tags = self.get_tags(model);
                    for required in &query.tags {
                        if !model_tags.contains(&Self::normalize_tag(required)) {
                            return false;
                        }
                    }
                }
                // Annotation filter
                if !query.annotations.is_empty() {
                    let model_annots = self.get_annotations(model);
                    for (k, v) in &query.annotations {
                        match model_annots.get(k) {
                            Some(val) if val == v => {}
                            _ => return false,
                        }
                    }
                }
                true
            })
            .map(|model| SearchResult {
                model: model.to_string(),
                tags: self.get_tags(model),
                annotations: self.get_annotations(model),
            })
            .collect()
    }

    /// Remove all tags and annotations for a model.
    pub fn remove_model(&mut self, model: &str) -> Result<()> {
        self.data.tags.remove(model);
        self.data.annotations.remove(model);
        self.save()
    }

    fn normalize_tag(tag: &str) -> String {
        tag.trim().to_lowercase().replace(' ', "-")
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> (tempfile::TempDir, TagStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = TagStore::new(dir.path()).unwrap();
        (dir, store)
    }

    #[test]
    fn test_add_and_get_tags() {
        let (_dir, mut store) = temp_store();
        store
            .add_tags("llama", &["production".into(), "text-gen".into()])
            .unwrap();
        let tags = store.get_tags("llama");
        assert!(tags.contains("production"));
        assert!(tags.contains("text-gen"));
    }

    #[test]
    fn test_remove_tags() {
        let (_dir, mut store) = temp_store();
        store
            .add_tags("llama", &["a".into(), "b".into(), "c".into()])
            .unwrap();
        store.remove_tags("llama", &["b".into()]).unwrap();
        let tags = store.get_tags("llama");
        assert!(tags.contains("a"));
        assert!(!tags.contains("b"));
        assert!(tags.contains("c"));
    }

    #[test]
    fn test_annotations() {
        let (_dir, mut store) = temp_store();
        store.set_annotation("gpt2", "task", "text-gen").unwrap();
        store.set_annotation("gpt2", "params", "124M").unwrap();
        let annots = store.get_annotations("gpt2");
        assert_eq!(annots.get("task").unwrap(), "text-gen");
        assert_eq!(annots.get("params").unwrap(), "124M");
    }

    #[test]
    fn test_search_by_tag() {
        let (_dir, mut store) = temp_store();
        store.add_tags("llama", &["production".into()]).unwrap();
        store.add_tags("gpt2", &["dev".into()]).unwrap();
        let query = SearchQuery {
            tags: vec!["production".into()],
            ..Default::default()
        };
        let models = vec!["llama".into(), "gpt2".into()];
        let results = store.search(&query, &models);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].model, "llama");
    }

    #[test]
    fn test_search_by_name() {
        let (_dir, store) = temp_store();
        let query = SearchQuery {
            name_pattern: Some("lla".into()),
            ..Default::default()
        };
        let models = vec!["llama".into(), "gpt2".into(), "llama-7b".into()];
        let results = store.search(&query, &models);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_by_annotation() {
        let (_dir, mut store) = temp_store();
        store.set_annotation("llama", "task", "text-gen").unwrap();
        store
            .set_annotation("resnet", "task", "image-class")
            .unwrap();
        let query = SearchQuery {
            annotations: vec![("task".into(), "text-gen".into())],
            ..Default::default()
        };
        let models = vec!["llama".into(), "resnet".into()];
        let results = store.search(&query, &models);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].model, "llama");
    }

    #[test]
    fn test_persistence() {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut store = TagStore::new(dir.path()).unwrap();
            store.add_tags("m1", &["t1".into()]).unwrap();
            store.set_annotation("m1", "k", "v").unwrap();
        }
        {
            let store = TagStore::new(dir.path()).unwrap();
            assert!(store.get_tags("m1").contains("t1"));
            assert_eq!(store.get_annotations("m1").get("k").unwrap(), "v");
        }
    }

    #[test]
    fn test_all_tags() {
        let (_dir, mut store) = temp_store();
        store.add_tags("a", &["x".into(), "y".into()]).unwrap();
        store.add_tags("b", &["y".into(), "z".into()]).unwrap();
        let all = store.all_tags();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_remove_model() {
        let (_dir, mut store) = temp_store();
        store.add_tags("m", &["t".into()]).unwrap();
        store.set_annotation("m", "k", "v").unwrap();
        store.remove_model("m").unwrap();
        assert!(store.get_tags("m").is_empty());
        assert!(store.get_annotations("m").is_empty());
    }

    #[test]
    fn test_normalize_tag() {
        assert_eq!(TagStore::normalize_tag("  Text Gen "), "text-gen");
        assert_eq!(TagStore::normalize_tag("PROD"), "prod");
    }
}
