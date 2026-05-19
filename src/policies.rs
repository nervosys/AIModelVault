//! Snapshot / retention policies — automated version cleanup rules.
//!
//! Each model can have an optional policy that limits how many versions are
//! kept, or removes versions older than a threshold.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::version::{ModelVersion, VersionControl};

// ── Types ────────────────────────────────────────────────────────────────────

/// A retention policy for a model's versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Maximum number of versions to keep (0 = unlimited).
    pub max_versions: usize,
    /// Maximum age in days (0 = unlimited).
    pub max_age_days: u32,
    /// Always keep at least the latest N versions even if they exceed max_age.
    pub keep_minimum: usize,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_versions: 0,
            max_age_days: 0,
            keep_minimum: 1,
        }
    }
}

/// Persisted policy data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PolicyData {
    /// model_name → policy
    pub policies: HashMap<String, RetentionPolicy>,
    /// Default policy applied when a model has no explicit override.
    pub default_policy: Option<RetentionPolicy>,
}

/// Result of applying retention policies.
#[derive(Debug, Clone, Serialize)]
pub struct PolicyReport {
    pub model: String,
    pub versions_before: usize,
    pub versions_removed: Vec<u32>,
    pub versions_after: usize,
}

// ── Store ────────────────────────────────────────────────────────────────────

/// Manages retention policies for a vault.
pub struct PolicyStore {
    path: PathBuf,
    data: PolicyData,
}

impl PolicyStore {
    const FILE_NAME: &'static str = "policies.json";

    pub fn new(vault_path: &Path) -> Result<Self> {
        let path = vault_path.join(Self::FILE_NAME);
        let data = if path.exists() {
            let contents = fs::read_to_string(&path)?;
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            PolicyData::default()
        };
        Ok(Self { path, data })
    }

    fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.path, json)?;
        crate::permissions::restrict_file(&self.path)?;
        Ok(())
    }

    /// Set a retention policy for a specific model.
    pub fn set(&mut self, model: &str, policy: RetentionPolicy) -> Result<()> {
        self.data.policies.insert(model.to_string(), policy);
        self.save()
    }

    /// Remove the policy for a model (falls back to default).
    pub fn remove(&mut self, model: &str) -> Result<()> {
        self.data.policies.remove(model);
        self.save()
    }

    /// Set the default policy.
    pub fn set_default(&mut self, policy: RetentionPolicy) -> Result<()> {
        self.data.default_policy = Some(policy);
        self.save()
    }

    /// Get the effective policy for a model.
    pub fn get(&self, model: &str) -> Option<&RetentionPolicy> {
        self.data
            .policies
            .get(model)
            .or(self.data.default_policy.as_ref())
    }

    /// List all explicit model policies.
    pub fn list(&self) -> &HashMap<String, RetentionPolicy> {
        &self.data.policies
    }

    /// Apply the policy for a single model, returning which versions would be
    /// (or were) removed.
    pub fn apply(
        &self,
        model: &str,
        vc: &mut VersionControl,
        dry_run: bool,
    ) -> Result<PolicyReport> {
        let Some(policy) = self.get(model).cloned() else {
            let versions = vc.list_versions(model);
            return Ok(PolicyReport {
                model: model.to_string(),
                versions_before: versions.len(),
                versions_removed: vec![],
                versions_after: versions.len(),
            });
        };

        let mut versions: Vec<ModelVersion> =
            vc.list_versions(model).into_iter().cloned().collect();
        versions.sort_by(|a, b| b.version.cmp(&a.version)); // newest first

        let versions_before = versions.len();
        let mut to_remove: Vec<u32> = Vec::new();
        let now = Utc::now();

        for (idx, ver) in versions.iter().enumerate() {
            // Always keep the minimum
            if idx < policy.keep_minimum {
                continue;
            }

            let mut should_remove = false;

            // Max versions check
            if policy.max_versions > 0 && idx >= policy.max_versions {
                should_remove = true;
            }

            // Max age check
            if policy.max_age_days > 0 {
                let age = now.signed_duration_since(ver.timestamp);
                if age > Duration::days(policy.max_age_days as i64) {
                    should_remove = true;
                }
            }

            if should_remove {
                to_remove.push(ver.version);
            }
        }

        if !dry_run {
            for v in &to_remove {
                vc.delete_version(model, *v)?;
            }
        }

        Ok(PolicyReport {
            model: model.to_string(),
            versions_before,
            versions_removed: to_remove.clone(),
            versions_after: versions_before - to_remove.len(),
        })
    }

    /// Apply policies to all models.
    pub fn apply_all(&self, vc: &mut VersionControl, dry_run: bool) -> Result<Vec<PolicyReport>> {
        let models = vc.list_models_owned();
        let mut reports = Vec::new();
        for model in &models {
            reports.push(self.apply(model, vc, dry_run)?);
        }
        Ok(reports)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_crud() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = PolicyStore::new(dir.path()).unwrap();

        let policy = RetentionPolicy {
            max_versions: 5,
            max_age_days: 90,
            keep_minimum: 1,
        };

        store.set("llama", policy.clone()).unwrap();
        assert!(store.get("llama").is_some());
        assert_eq!(store.get("llama").unwrap().max_versions, 5);

        store.remove("llama").unwrap();
        assert!(store.get("llama").is_none());
    }

    #[test]
    fn test_default_policy() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = PolicyStore::new(dir.path()).unwrap();

        assert!(store.get("any-model").is_none());

        store
            .set_default(RetentionPolicy {
                max_versions: 3,
                max_age_days: 0,
                keep_minimum: 1,
            })
            .unwrap();

        assert!(store.get("any-model").is_some());
        assert_eq!(store.get("any-model").unwrap().max_versions, 3);
    }

    #[test]
    fn test_persistence() {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut store = PolicyStore::new(dir.path()).unwrap();
            store
                .set(
                    "llama",
                    RetentionPolicy {
                        max_versions: 10,
                        max_age_days: 30,
                        keep_minimum: 2,
                    },
                )
                .unwrap();
        }
        let store2 = PolicyStore::new(dir.path()).unwrap();
        assert_eq!(store2.get("llama").unwrap().max_versions, 10);
    }
}
