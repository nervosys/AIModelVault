//! Version control system for model checkpoints
//!
//! Maintains complete history of model versions with metadata and generations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::crypto::FipsCrypto;
use crate::error::Result;

/// Represents a single model version/checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    /// Version number (sequential)
    pub version: u32,

    /// Unique checkpoint identifier
    pub checkpoint_id: String,

    /// Timestamp when version was created
    pub timestamp: DateTime<Utc>,

    /// Parent version for branching/lineage
    pub parent_version: Option<u32>,

    /// Model format
    pub format: String,

    /// Original size in bytes
    pub size_bytes: u64,

    /// Compressed size in bytes
    pub compressed_size_bytes: u64,

    /// SHA-256 checksum of original data
    pub checksum_sha256: String,

    /// User-provided metadata
    pub metadata: HashMap<String, String>,

    /// Relative path to encrypted file
    pub file_path: String,
}

/// Version control system for model checkpoints
///
/// Features:
/// - Complete version history
/// - Parent-child relationships (branching)
/// - Metadata tracking
/// - Checksum verification
/// - Generation/lineage tracking
///
/// Compliance:
/// - CMMC AU.3.046: Alert in the event of an audit logging process failure
/// - CMMC AU.3.049: Protect audit information from unauthorized access
pub struct VersionControl {
    vault_path: PathBuf,
    version_file: PathBuf,
    pub(crate) versions: HashMap<String, Vec<ModelVersion>>,
}

impl VersionControl {
    const VERSION_FILE: &'static str = "versions.json";

    /// Create new version control instance
    pub fn new(vault_path: &Path) -> Result<Self> {
        let version_file = vault_path.join(Self::VERSION_FILE);
        let mut vc = Self {
            vault_path: vault_path.to_path_buf(),
            version_file,
            versions: HashMap::new(),
        };
        vc.load_versions()?;
        Ok(vc)
    }

    /// Return the vault directory path
    pub fn vault_path(&self) -> &std::path::Path {
        &self.vault_path
    }

    /// Load version history from file
    fn load_versions(&mut self) -> Result<()> {
        if self.version_file.exists() {
            let contents = fs::read_to_string(&self.version_file)?;
            self.versions = serde_json::from_str(&contents)?;
        }
        Ok(())
    }

    /// Save version history to file
    fn save_versions(&self) -> Result<()> {
        let contents = serde_json::to_string_pretty(&self.versions)?;
        fs::write(&self.version_file, contents)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            fs::set_permissions(&self.version_file, perms)?;
        }

        Ok(())
    }

    /// Add new model version
    #[allow(clippy::too_many_arguments)]
    pub fn add_version(
        &mut self,
        model_name: &str,
        file_path: &str,
        format: &str,
        size_bytes: u64,
        compressed_size_bytes: u64,
        checksum: &str,
        metadata: Option<HashMap<String, String>>,
        parent_version: Option<u32>,
    ) -> Result<ModelVersion> {
        let versions = self.versions.entry(model_name.to_string()).or_default();

        // Determine next version number
        let version = if versions.is_empty() {
            1
        } else {
            versions.iter().map(|v| v.version).max().unwrap_or(0) + 1
        };

        let timestamp = Utc::now();
        let checkpoint_id = Self::generate_checkpoint_id(model_name, version, &timestamp);

        let model_version = ModelVersion {
            version,
            checkpoint_id,
            timestamp,
            parent_version,
            format: format.to_string(),
            size_bytes,
            compressed_size_bytes,
            checksum_sha256: checksum.to_string(),
            metadata: metadata.unwrap_or_default(),
            file_path: file_path.to_string(),
        };

        versions.push(model_version.clone());
        self.save_versions()?;

        Ok(model_version)
    }

    /// Get specific model version
    pub fn get_version(&self, model_name: &str, version: Option<u32>) -> Option<&ModelVersion> {
        let versions = self.versions.get(model_name)?;

        if versions.is_empty() {
            return None;
        }

        if let Some(v) = version {
            versions.iter().find(|mv| mv.version == v)
        } else {
            // Return latest version
            versions.iter().max_by_key(|mv| mv.version)
        }
    }

    /// List all versions of a model
    pub fn list_versions(&self, model_name: &str) -> Vec<&ModelVersion> {
        self.versions
            .get(model_name)
            .map(|v| {
                let mut sorted: Vec<&ModelVersion> = v.iter().collect();
                sorted.sort_by_key(|mv| mv.version);
                sorted
            })
            .unwrap_or_default()
    }

    /// Get complete lineage/generation history for a version
    pub fn get_lineage(&self, model_name: &str, version: u32) -> Vec<&ModelVersion> {
        let mut lineage = Vec::new();

        if let Some(mut current) = self.get_version(model_name, Some(version)) {
            lineage.push(current);

            while let Some(parent_ver) = current.parent_version {
                if let Some(parent) = self.get_version(model_name, Some(parent_ver)) {
                    lineage.insert(0, parent);
                    current = parent;
                } else {
                    break;
                }
            }
        }

        lineage
    }

    /// Delete a specific version
    pub fn delete_version(&mut self, model_name: &str, version: u32) -> Result<bool> {
        if let Some(versions) = self.versions.get_mut(model_name) {
            let original_len = versions.len();
            versions.retain(|v| v.version != version);

            if versions.len() < original_len {
                self.save_versions()?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Clean up old versions, keeping only the most recent
    pub fn cleanup_old_versions(
        &mut self,
        model_name: &str,
        keep_count: usize,
    ) -> Result<Vec<u32>> {
        let Some(versions) = self.versions.get_mut(model_name) else {
            return Ok(Vec::new());
        };

        if versions.len() <= keep_count {
            return Ok(Vec::new());
        }

        // Sort by version number descending
        versions.sort_by(|a, b| b.version.cmp(&a.version));

        // Keep the most recent
        let to_delete: Vec<u32> = versions
            .iter()
            .skip(keep_count)
            .map(|v| v.version)
            .collect();

        versions.truncate(keep_count);
        self.save_versions()?;

        Ok(to_delete)
    }

    /// Verify data integrity using stored checksum
    pub fn verify_checksum(&self, model_name: &str, version: u32, data: &[u8]) -> bool {
        if let Some(model_version) = self.get_version(model_name, Some(version)) {
            let checksum = hex::encode(FipsCrypto::hash_sha256(data));
            return checksum == model_version.checksum_sha256;
        }
        false
    }

    /// Update metadata for a specific model version
    pub fn update_metadata(
        &mut self,
        model_name: &str,
        version: u32,
        key: &str,
        value: String,
    ) -> Result<()> {
        if let Some(versions) = self.versions.get_mut(model_name) {
            if let Some(model_version) = versions.iter_mut().find(|v| v.version == version) {
                model_version.metadata.insert(key.to_string(), value);
                self.save_versions()?;
                return Ok(());
            }
        }
        Err(crate::error::VaultError::VersionNotFound(
            version,
            model_name.to_string(),
        ))
    }

    /// Get metadata for a specific model version
    pub fn get_metadata(&self, model_name: &str, version: u32, key: &str) -> Option<String> {
        self.get_version(model_name, Some(version))
            .and_then(|v| v.metadata.get(key).cloned())
    }

    /// Generate unique checkpoint identifier
    fn generate_checkpoint_id(
        model_name: &str,
        version: u32,
        _timestamp: &DateTime<Utc>,
    ) -> String {
        let uuid = Uuid::new_v4();
        format!("{}-v{}-{}", model_name, version, uuid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_version_control() {
        let temp_dir = tempdir().unwrap();
        let mut vc = VersionControl::new(temp_dir.path()).unwrap();

        let v1 = vc
            .add_version(
                "test_model",
                "test_file.enc",
                "pytorch",
                1000,
                500,
                "abc123",
                None,
                None,
            )
            .unwrap();

        assert_eq!(v1.version, 1);
        assert_eq!(v1.format, "pytorch");

        let v2 = vc
            .add_version(
                "test_model",
                "test_file2.enc",
                "pytorch",
                2000,
                1000,
                "def456",
                None,
                Some(1),
            )
            .unwrap();

        assert_eq!(v2.version, 2);
        assert_eq!(v2.parent_version, Some(1));

        let lineage = vc.get_lineage("test_model", 2);
        assert_eq!(lineage.len(), 2);
    }
}
