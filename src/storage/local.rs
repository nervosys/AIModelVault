//! Local filesystem storage backend

use async_trait::async_trait;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::error::{Result, VaultError};
use crate::storage::StorageBackend;

/// Local filesystem storage backend
pub struct LocalBackend {
    base_path: PathBuf,
}

impl LocalBackend {
    /// Create new local storage backend
    pub fn new(base_path: PathBuf) -> Result<Self> {
        if !base_path.exists() {
            fs::create_dir_all(&base_path)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(0o700);
                fs::set_permissions(&base_path, perms)?;
            }
        }

        Ok(Self { base_path })
    }

    fn get_path(&self, key: &str) -> PathBuf {
        self.base_path.join(key)
    }
}

#[async_trait]
impl StorageBackend for LocalBackend {
    async fn upload(&self, key: &str, data: &[u8]) -> Result<()> {
        let path = self.get_path(key);

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&path)?;
        file.write_all(data)?;

        // Set restrictive permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    async fn download(&self, key: &str) -> Result<Vec<u8>> {
        let path = self.get_path(key);

        if !path.exists() {
            return Err(VaultError::ModelNotFound(key.to_string()));
        }

        let mut file = File::open(&path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        Ok(data)
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let path = self.get_path(key);

        if path.exists() {
            fs::remove_file(&path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.get_path(key).exists())
    }

    async fn list(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();

        fn scan_dir(base: &PathBuf, current: &PathBuf, files: &mut Vec<String>) -> Result<()> {
            for entry in fs::read_dir(current)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    if let Ok(relative) = path.strip_prefix(base) {
                        if let Some(name) = relative.to_str() {
                            files.push(name.to_string());
                        }
                    }
                } else if path.is_dir() {
                    scan_dir(base, &path, files)?;
                }
            }
            Ok(())
        }

        scan_dir(&self.base_path, &self.base_path, &mut files)?;
        Ok(files)
    }

    async fn size(&self, key: &str) -> Result<u64> {
        let path = self.get_path(key);
        let metadata = fs::metadata(&path)?;
        Ok(metadata.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_local_backend() {
        let temp_dir = tempdir().unwrap();
        let backend = LocalBackend::new(temp_dir.path().to_path_buf()).unwrap();

        let data = b"test data";
        backend.upload("test.txt", data).await.unwrap();

        assert!(backend.exists("test.txt").await.unwrap());

        let retrieved = backend.download("test.txt").await.unwrap();
        assert_eq!(data, &retrieved[..]);

        let size = backend.size("test.txt").await.unwrap();
        assert_eq!(size, data.len() as u64);

        let files = backend.list().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], "test.txt");

        let deleted = backend.delete("test.txt").await.unwrap();
        assert!(deleted);
        assert!(!backend.exists("test.txt").await.unwrap());
    }
}
