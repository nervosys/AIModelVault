//! Storage backend for encrypted model data
//!
//! Supports multiple storage backends:
//! - Local filesystem (default)
//! - AWS S3
//! - Azure Blob Storage
//! - Google Cloud Storage

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::crypto::compression::{compress, decompress, CompressionAlgorithm, CompressionLevel};
use crate::crypto::{FipsCrypto, SecureKey};
use crate::error::{Result, VaultError};
use async_trait::async_trait;

// Cloud backend modules
pub mod local;

#[cfg(feature = "s3")]
pub mod s3;

#[cfg(feature = "azure")]
pub mod azure;

// GCS support disabled due to security vulnerabilities in cloud-storage dependency
// See SECURITY_AUDIT.md for details on RUSTSEC-2025-0009 and RUSTSEC-2025-0010
// #[cfg(feature = "gcs")]
// pub mod gcs;

/// Storage backend trait for different storage providers
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Upload data to storage
    async fn upload(&self, key: &str, data: &[u8]) -> Result<()>;

    /// Download data from storage
    async fn download(&self, key: &str) -> Result<Vec<u8>>;

    /// Delete data from storage
    async fn delete(&self, key: &str) -> Result<bool>;

    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool>;

    /// List all keys (files)
    async fn list(&self) -> Result<Vec<String>>;

    /// Get size of stored data
    async fn size(&self, key: &str) -> Result<u64>;
}

/// Storage backend configuration
#[derive(Debug, Clone)]
pub enum StorageConfig {
    Local {
        path: PathBuf,
    },
    S3 {
        bucket: String,
        region: String,
        prefix: Option<String>,
    },
    Azure {
        account: String,
        container: String,
        prefix: Option<String>,
    },
    Gcs {
        bucket: String,
        project: String,
        prefix: Option<String>,
    },
}

impl StorageConfig {
    /// Create a storage backend from configuration
    pub async fn create_backend(&self) -> Result<Box<dyn StorageBackend>> {
        match self {
            StorageConfig::Local { path } => {
                let backend = local::LocalBackend::new(path.clone())?;
                Ok(Box::new(backend))
            }
            #[cfg(feature = "s3")]
            StorageConfig::S3 {
                bucket,
                region,
                prefix,
            } => {
                let backend =
                    s3::S3Backend::new(bucket.clone(), region.clone(), prefix.clone()).await?;
                Ok(Box::new(backend))
            }
            #[cfg(not(feature = "s3"))]
            StorageConfig::S3 { .. } => Err(VaultError::ConfigError(
                "S3 support not enabled. Rebuild with --features s3".to_string(),
            )),
            #[cfg(feature = "azure")]
            StorageConfig::Azure {
                account,
                container,
                prefix,
            } => {
                let backend =
                    azure::AzureBackend::new(account.clone(), container.clone(), prefix.clone())
                        .await?;
                Ok(Box::new(backend))
            }
            #[cfg(not(feature = "azure"))]
            StorageConfig::Azure { .. } => Err(VaultError::ConfigError(
                "Azure support not enabled. Rebuild with --features azure".to_string(),
            )),
            // GCS support disabled due to critical security vulnerabilities
            // in cloud-storage dependency (RUSTSEC-2025-0009, RUSTSEC-2025-0010)
            StorageConfig::Gcs { .. } => Err(VaultError::ConfigError(
                "GCS support temporarily disabled due to security vulnerabilities. Use S3 or Azure instead.".to_string(),
            )),
        }
    }
}

/// Storage backend for encrypted and compressed model data
pub struct Storage {
    vault_path: PathBuf,
    crypto: FipsCrypto,
}

impl Storage {
    /// Create new storage instance
    pub fn new(vault_path: &Path) -> Result<Self> {
        if !vault_path.exists() {
            fs::create_dir_all(vault_path)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(0o700);
                fs::set_permissions(vault_path, perms)?;
            }
        }

        Ok(Self {
            vault_path: vault_path.to_path_buf(),
            crypto: FipsCrypto::new()?,
        })
    }

    /// Store data (compress then encrypt)
    pub fn store(
        &self,
        filename: &str,
        data: &[u8],
        key: &SecureKey,
        compression: CompressionAlgorithm,
        compression_level: CompressionLevel,
    ) -> Result<(u64, u64)> {
        // Compress data
        let compressed = compress(data, compression, compression_level)?;
        let compressed_size = compressed.len() as u64;

        // Encrypt compressed data
        let encrypted = self.crypto.encrypt(&compressed, key)?;

        // Write to file
        let file_path = self.vault_path.join(filename);
        let mut file = File::create(&file_path)?;
        file.write_all(&encrypted)?;

        // Set restrictive permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            fs::set_permissions(&file_path, perms)?;
        }

        Ok((data.len() as u64, compressed_size))
    }

    /// Retrieve data (decrypt then decompress)
    pub fn retrieve(
        &self,
        filename: &str,
        key: &SecureKey,
        compression: CompressionAlgorithm,
    ) -> Result<Vec<u8>> {
        let file_path = self.vault_path.join(filename);

        if !file_path.exists() {
            return Err(VaultError::ModelNotFound(filename.to_string()));
        }

        // Read encrypted data
        let mut file = File::open(&file_path)?;
        let mut encrypted = Vec::new();
        file.read_to_end(&mut encrypted)?;

        // Decrypt
        let compressed = self.crypto.decrypt(&encrypted, key)?;

        // Decompress
        let data = decompress(&compressed, compression)?;

        Ok(data)
    }

    /// Delete stored file
    pub fn delete(&self, filename: &str) -> Result<bool> {
        let file_path = self.vault_path.join(filename);

        if file_path.exists() {
            fs::remove_file(&file_path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if file exists
    pub fn exists(&self, filename: &str) -> bool {
        self.vault_path.join(filename).exists()
    }

    /// Get file size
    pub fn file_size(&self, filename: &str) -> Result<u64> {
        let file_path = self.vault_path.join(filename);
        let metadata = fs::metadata(&file_path)?;
        Ok(metadata.len())
    }

    /// List all stored files
    pub fn list_files(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();

        for entry in fs::read_dir(&self.vault_path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    files.push(name.to_string());
                }
            }
        }

        Ok(files)
    }

    /// Get storage statistics
    pub fn get_stats(&self) -> Result<StorageStats> {
        let mut total_size = 0u64;
        let mut file_count = 0usize;

        for entry in fs::read_dir(&self.vault_path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                total_size += entry.metadata()?.len();
                file_count += 1;
            }
        }

        Ok(StorageStats {
            total_size_bytes: total_size,
            file_count,
        })
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_size_bytes: u64,
    pub file_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_storage() {
        let temp_dir = tempdir().unwrap();
        let storage = Storage::new(temp_dir.path()).unwrap();

        let crypto = FipsCrypto::new().unwrap();
        let passphrase = b"test_passphrase_with_sufficient_entropy".to_vec();
        let (key, _) = crypto.derive_key(passphrase, None).unwrap();

        let data = b"Test model data";
        // Get stats
        let (orig_size, _comp_size) = storage
            .store(
                "test.enc",
                data,
                &key,
                CompressionAlgorithm::Gzip,
                CompressionLevel::Balanced,
            )
            .unwrap();

        assert_eq!(orig_size, data.len() as u64);

        let retrieved = storage
            .retrieve("test.enc", &key, CompressionAlgorithm::Gzip)
            .unwrap();

        assert_eq!(data, &retrieved[..]);
    }
}
