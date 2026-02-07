//! Main Vault implementation
//!
//! Provides high-level API for secure model storage and retrieval.

use std::collections::HashMap;
use std::fs;

use crate::audit::{AuditEntry, AuditEventType, AuditLogger};
use crate::config::VaultConfig;
use crate::crypto::{FipsCrypto, KeyManager, SecureKey};
use crate::error::{Result, VaultError};
use crate::formats::ModelMetadata;
use crate::storage::Storage;
use crate::version::{ModelVersion, VersionControl};

/// Main vault for secure model storage
pub struct Vault {
    config: VaultConfig,
    storage: Storage,
    version_control: VersionControl,
    audit_logger: Option<AuditLogger>,
    crypto: FipsCrypto,
    #[allow(dead_code)]
    key_manager: KeyManager,
    active_key: Option<SecureKey>,
}

impl Vault {
    /// Create or open a vault
    pub fn new(config: Option<VaultConfig>) -> Result<Self> {
        let config = match config {
            Some(c) => c,
            None => VaultConfig::new()?,
        };

        let vault_path = config.get_vault_path(None);

        // Ensure vault directory exists
        if !vault_path.exists() {
            fs::create_dir_all(&vault_path)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(0o700);
                fs::set_permissions(&vault_path, perms)?;
            }
        }

        let storage = Storage::new(&vault_path)?;
        let version_control = VersionControl::new(&vault_path)?;

        let audit_logger = if config.security.audit_log {
            Some(AuditLogger::new(&config.get_audit_log_path())?)
        } else {
            None
        };

        let crypto = FipsCrypto::new()?;
        let key_manager = KeyManager::new()?;

        if let Some(logger) = &audit_logger {
            logger.log(AuditEntry {
                timestamp: chrono::Utc::now(),
                event_type: AuditEventType::VaultOpened,
                description: "Vault opened".to_string(),
                model_name: None,
                version: None,
                success: true,
                metadata: None,
            })?;
        }

        Ok(Self {
            config,
            storage,
            version_control,
            audit_logger,
            crypto,
            key_manager,
            active_key: None,
        })
    }

    /// Unlock vault with passphrase
    ///
    /// The salt used for key derivation is persisted in the vault directory.
    /// This ensures the same passphrase always derives the same key across sessions.
    pub fn unlock(&mut self, passphrase: Vec<u8>) -> Result<()> {
        let vault_path = self.config.get_vault_path(None);
        let salt_file = vault_path.join("vault.salt");

        // Load existing salt or generate a new one
        let existing_salt = if salt_file.exists() {
            Some(fs::read(&salt_file)?)
        } else {
            None
        };

        let (key, salt) = self.crypto.derive_key(passphrase, existing_salt)?;

        // Persist the salt if it's new
        if !salt_file.exists() {
            fs::write(&salt_file, &salt)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(0o600);
                fs::set_permissions(&salt_file, perms)?;
            }
        }

        self.active_key = Some(key);

        if let Some(logger) = &self.audit_logger {
            logger.log_auth(true, None)?;
        }

        Ok(())
    }

    /// Lock vault (clear active key)
    pub fn lock(&mut self) {
        self.active_key = None;
    }

    /// Check if vault is unlocked
    #[must_use]
    pub fn is_unlocked(&self) -> bool {
        self.active_key.is_some()
    }

    /// Store a model
    pub fn store_model(
        &mut self,
        name: &str,
        data: Vec<u8>,
        metadata: ModelMetadata,
        parent_version: Option<u32>,
    ) -> Result<ModelVersion> {
        let key = self
            .active_key
            .as_ref()
            .ok_or_else(|| VaultError::SecurityViolation("Vault is locked".to_string()))?;

        // Compute checksum before compression/encryption
        let checksum = hex::encode(FipsCrypto::hash_sha256(&data));

        // Generate filename
        let filename = format!("{}.vault", uuid::Uuid::new_v4());

        // Store data (compress + encrypt)
        let (original_size, compressed_size) = self.storage.store(
            &filename,
            &data,
            key,
            self.config.get_compression_algorithm(),
            self.config.get_compression_level(),
        )?;

        // Convert metadata to version control format
        let mut version_metadata = HashMap::new();
        if let Some(desc) = metadata.description {
            version_metadata.insert("description".to_string(), desc);
        }
        if let Some(framework) = metadata.framework {
            version_metadata.insert("framework".to_string(), framework);
        }
        if let Some(task) = metadata.task {
            version_metadata.insert("task".to_string(), task);
        }
        version_metadata.extend(metadata.custom_fields);

        // Add version control entry
        let version = self.version_control.add_version(
            name,
            &filename,
            &metadata.format.name(),
            original_size,
            compressed_size,
            &checksum,
            Some(version_metadata),
            parent_version,
        )?;

        // Audit log
        if let Some(logger) = &self.audit_logger {
            logger.log_model_stored(name, version.version, true)?;
        }

        // Auto-cleanup old versions if enabled
        if self.config.storage.auto_cleanup {
            let deleted = self
                .version_control
                .cleanup_old_versions(name, self.config.storage.max_versions as usize)?;

            // Delete associated files
            for ver in deleted {
                if let Some(old_version) = self.version_control.get_version(name, Some(ver)) {
                    let _ = self.storage.delete(&old_version.file_path);
                }
            }
        }

        Ok(version)
    }

    /// Retrieve a model
    pub fn get_model(&self, name: &str, version: Option<u32>) -> Result<Vec<u8>> {
        let key = self
            .active_key
            .as_ref()
            .ok_or_else(|| VaultError::SecurityViolation("Vault is locked".to_string()))?;

        let model_version = self
            .version_control
            .get_version(name, version)
            .ok_or_else(|| {
                if let Some(v) = version {
                    VaultError::VersionNotFound(v, name.to_string())
                } else {
                    VaultError::ModelNotFound(name.to_string())
                }
            })?;

        // Retrieve data (decrypt + decompress)
        let data = self.storage.retrieve(
            &model_version.file_path,
            key,
            self.config.get_compression_algorithm(),
        )?;

        // Verify integrity
        if !self
            .version_control
            .verify_checksum(name, model_version.version, &data)
        {
            if let Some(logger) = &self.audit_logger {
                let _ = logger.log(AuditEntry {
                    timestamp: chrono::Utc::now(),
                    event_type: AuditEventType::IntegrityFailure,
                    description: format!(
                        "Integrity check failed for model '{}' version {}",
                        name, model_version.version
                    ),
                    model_name: Some(name.to_string()),
                    version: Some(model_version.version),
                    success: false,
                    metadata: None,
                });
            }
            return Err(VaultError::IntegrityError(format!(
                "Checksum mismatch for model '{}' version {}",
                name, model_version.version
            )));
        }

        // Audit log
        if let Some(logger) = &self.audit_logger {
            logger.log_model_retrieved(name, model_version.version, true)?;
        }

        Ok(data)
    }

    /// List all models in vault
    #[must_use]
    pub fn list_models(&self) -> Vec<String> {
        self.version_control.versions.keys().cloned().collect()
    }

    /// List versions of a model
    pub fn list_versions(&self, name: &str) -> Vec<&ModelVersion> {
        self.version_control.list_versions(name)
    }

    /// Get model lineage/history
    pub fn get_lineage(&self, name: &str, version: u32) -> Vec<&ModelVersion> {
        self.version_control.get_lineage(name, version)
    }

    /// Delete a specific version
    pub fn delete_version(&mut self, name: &str, version: u32) -> Result<bool> {
        if let Some(model_version) = self.version_control.get_version(name, Some(version)) {
            let file_path = model_version.file_path.clone();

            // Delete from version control
            let deleted = self.version_control.delete_version(name, version)?;

            if deleted {
                // Delete file
                self.storage.delete(&file_path)?;

                // Audit log
                if let Some(logger) = &self.audit_logger {
                    logger.log(AuditEntry {
                        timestamp: chrono::Utc::now(),
                        event_type: AuditEventType::VersionDeleted,
                        description: format!("Deleted model '{}' version {}", name, version),
                        model_name: Some(name.to_string()),
                        version: Some(version),
                        success: true,
                        metadata: None,
                    })?;
                }
            }

            Ok(deleted)
        } else {
            Ok(false)
        }
    }

    /// Get vault statistics
    pub fn get_stats(&self) -> Result<VaultStats> {
        let storage_stats = self.storage.get_stats()?;
        let model_count = self.version_control.versions.len();
        let total_versions: usize = self
            .version_control
            .versions
            .values()
            .map(|v| v.len())
            .sum();

        Ok(VaultStats {
            model_count,
            total_versions,
            total_size_bytes: storage_stats.total_size_bytes,
            file_count: storage_stats.file_count,
        })
    }

    /// Get vault configuration
    pub fn get_config(&self) -> &VaultConfig {
        &self.config
    }

    /// Change vault passphrase
    ///
    /// Re-derives and persists a new salt, then re-encrypts all stored model files.
    pub fn change_passphrase(&mut self, new_passphrase: Vec<u8>) -> Result<usize> {
        let old_key = self
            .active_key
            .as_ref()
            .ok_or_else(|| VaultError::SecurityViolation("Vault is locked".to_string()))?
            .clone();

        // Derive new key (fresh salt)
        let (new_key, new_salt) = self.crypto.derive_key(new_passphrase, None)?;

        let compression_algo = self.config.get_compression_algorithm();

        // Re-encrypt every stored file
        let mut re_encrypted = 0usize;
        let model_names: Vec<String> = self.version_control.versions.keys().cloned().collect();

        for model_name in &model_names {
            let versions: Vec<ModelVersion> = self
                .version_control
                .versions
                .get(model_name)
                .cloned()
                .unwrap_or_default();

            for ver in &versions {
                // Decrypt with old key
                let data = self
                    .storage
                    .retrieve(&ver.file_path, &old_key, compression_algo)?;

                // Delete old file & re-store with new key
                self.storage.delete(&ver.file_path)?;
                self.storage.store(
                    &ver.file_path,
                    &data,
                    &new_key,
                    compression_algo,
                    self.config.get_compression_level(),
                )?;

                re_encrypted += 1;
            }
        }

        // Persist new salt
        let vault_path = self.config.get_vault_path(None);
        let salt_file = vault_path.join("vault.salt");
        fs::write(&salt_file, &new_salt)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            fs::set_permissions(&salt_file, perms)?;
        }

        self.active_key = Some(new_key);

        if let Some(logger) = &self.audit_logger {
            logger.log(AuditEntry {
                timestamp: chrono::Utc::now(),
                event_type: AuditEventType::VaultOpened,
                description: format!("Passphrase changed, {} files re-encrypted", re_encrypted),
                model_name: None,
                version: None,
                success: true,
                metadata: None,
            })?;
        }

        Ok(re_encrypted)
    }

    /// Update metadata for a specific model version
    pub fn update_version_metadata(
        &mut self,
        model_name: &str,
        version: u32,
        key: &str,
        value: String,
    ) -> Result<()> {
        self.version_control
            .update_metadata(model_name, version, key, value)
    }

    /// Get metadata for a specific model version
    pub fn get_version_metadata(
        &self,
        model_name: &str,
        version: u32,
        key: &str,
    ) -> Option<String> {
        self.version_control.get_metadata(model_name, version, key)
    }
}

/// Vault statistics
#[derive(Debug, Clone)]
pub struct VaultStats {
    pub model_count: usize,
    pub total_versions: usize,
    pub total_size_bytes: u64,
    pub file_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::ModelFormat;
    use tempfile::tempdir;

    #[test]
    fn test_vault_operations() {
        let temp_dir = tempdir().unwrap();
        let dirs = crate::config::DirectoryPaths {
            config_dir: temp_dir.path().join("config"),
            data_dir: temp_dir.path().join("data"),
            cache_dir: temp_dir.path().join("cache"),
            vault_dir: temp_dir.path().join("data/vaults/default"),
            log_dir: temp_dir.path().join("data/logs"),
            backends_dir: temp_dir.path().join("config/backends"),
            utilities_dir: temp_dir.path().join("config/utilities"),
            databases_dir: temp_dir.path().join("config/databases"),
        };

        let config = VaultConfig::with_dirs(dirs).unwrap();
        let mut vault = Vault::new(Some(config)).unwrap();

        // Unlock vault
        let passphrase = b"test_passphrase_with_sufficient_entropy".to_vec();
        vault.unlock(passphrase).unwrap();

        // Store model
        let data = b"Test model data".to_vec();
        let metadata = ModelMetadata::new("test_model".to_string(), ModelFormat::PyTorch)
            .with_description("Test model".to_string());

        let version = vault
            .store_model("test_model", data.clone(), metadata, None)
            .unwrap();
        assert_eq!(version.version, 1);

        // Retrieve model
        let retrieved = vault.get_model("test_model", None).unwrap();
        assert_eq!(data, retrieved);

        // List models
        let models = vault.list_models();
        assert_eq!(models.len(), 1);
        assert!(models.contains(&"test_model".to_string()));
    }
}
