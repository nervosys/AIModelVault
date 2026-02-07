//! Azure Blob Storage backend

use async_trait::async_trait;
use azure_storage::StorageCredentials;
use azure_storage_blobs::prelude::*;

use crate::error::{Result, VaultError};
use crate::storage::StorageBackend;

/// Azure Blob Storage backend
pub struct AzureBackend {
    client: ContainerClient,
    prefix: String,
}

impl AzureBackend {
    /// Create new Azure Blob Storage backend
    ///
    /// # Arguments
    /// * `account` - Storage account name
    /// * `container` - Container name
    /// * `prefix` - Optional blob prefix (folder path)
    ///
    /// # Authentication
    /// Uses AZURE_STORAGE_KEY or AZURE_STORAGE_SAS_TOKEN environment variable
    pub async fn new(account: String, container: String, prefix: Option<String>) -> Result<Self> {
        // Try to get credentials from environment
        let credentials = if let Ok(key) = std::env::var("AZURE_STORAGE_KEY") {
            StorageCredentials::access_key(account.clone(), key)
        } else if let Ok(sas) = std::env::var("AZURE_STORAGE_SAS_TOKEN") {
            StorageCredentials::sas_token(sas)
                .map_err(|e| VaultError::ConfigError(format!("Invalid SAS token: {}", e)))?
        } else {
            return Err(VaultError::ConfigError(
                "Azure credentials not found. Set AZURE_STORAGE_KEY or AZURE_STORAGE_SAS_TOKEN"
                    .to_string(),
            ));
        };

        let client = ClientBuilder::new(account, credentials).container_client(container);

        Ok(Self {
            client,
            prefix: prefix.unwrap_or_default(),
        })
    }

    fn get_blob_name(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}/{}", self.prefix.trim_end_matches('/'), key)
        }
    }
}

#[async_trait]
impl StorageBackend for AzureBackend {
    async fn upload(&self, key: &str, data: &[u8]) -> Result<()> {
        let blob_name = self.get_blob_name(key);
        let blob_client = self.client.blob_client(blob_name);

        blob_client
            .put_block_blob(data)
            .await
            .map_err(|e| VaultError::StorageError(format!("Azure upload failed: {}", e)))?;

        Ok(())
    }

    async fn download(&self, key: &str) -> Result<Vec<u8>> {
        let blob_name = self.get_blob_name(key);
        let blob_client = self.client.blob_client(blob_name);

        let response = blob_client.get_content().await.map_err(|e| {
            if e.to_string().contains("BlobNotFound") {
                VaultError::ModelNotFound(key.to_string())
            } else {
                VaultError::StorageError(format!("Azure download failed: {}", e))
            }
        })?;

        Ok(response)
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let blob_name = self.get_blob_name(key);
        let blob_client = self.client.blob_client(blob_name);

        // Check if exists first
        let exists = self.exists(key).await?;
        if !exists {
            return Ok(false);
        }

        blob_client
            .delete()
            .await
            .map_err(|e| VaultError::StorageError(format!("Azure delete failed: {}", e)))?;

        Ok(true)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let blob_name = self.get_blob_name(key);
        let blob_client = self.client.blob_client(blob_name);

        match blob_client.get_properties().await {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.to_string().contains("BlobNotFound") {
                    Ok(false)
                } else {
                    Err(VaultError::StorageError(format!(
                        "Azure head failed: {}",
                        e
                    )))
                }
            }
        }
    }

    async fn list(&self) -> Result<Vec<String>> {
        let mut keys = Vec::new();

        let mut stream = self
            .client
            .list_blobs()
            .prefix(if !self.prefix.is_empty() {
                Some(self.prefix.clone())
            } else {
                None
            })
            .into_stream();

        while let Some(response) = stream.next().await {
            let response = response
                .map_err(|e| VaultError::StorageError(format!("Azure list failed: {}", e)))?;

            for blob in response.blobs.blobs() {
                let name = &blob.name;
                // Strip prefix if present
                let clean_name = if !self.prefix.is_empty() {
                    name.strip_prefix(&format!("{}/", self.prefix.trim_end_matches('/')))
                        .unwrap_or(name)
                        .to_string()
                } else {
                    name.clone()
                };
                keys.push(clean_name);
            }
        }

        Ok(keys)
    }

    async fn size(&self, key: &str) -> Result<u64> {
        let blob_name = self.get_blob_name(key);
        let blob_client = self.client.blob_client(blob_name);

        let properties = blob_client.get_properties().await.map_err(|e| {
            if e.to_string().contains("BlobNotFound") {
                VaultError::ModelNotFound(key.to_string())
            } else {
                VaultError::StorageError(format!("Azure properties failed: {}", e))
            }
        })?;

        Ok(properties.blob.properties.content_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require Azure credentials and a test container
    // They are disabled by default. Enable with: cargo test --features azure-integration-tests

    #[tokio::test]
    #[ignore]
    async fn test_azure_backend() {
        let account = std::env::var("TEST_AZURE_ACCOUNT").unwrap();
        let container = std::env::var("TEST_AZURE_CONTAINER").unwrap();

        let backend = AzureBackend::new(account, container, Some("test-neuronvault".to_string()))
            .await
            .unwrap();

        let data = b"test data";
        backend.upload("test.txt", data).await.unwrap();

        assert!(backend.exists("test.txt").await.unwrap());

        let retrieved = backend.download("test.txt").await.unwrap();
        assert_eq!(data, &retrieved[..]);

        let size = backend.size("test.txt").await.unwrap();
        assert_eq!(size, data.len() as u64);

        let deleted = backend.delete("test.txt").await.unwrap();
        assert!(deleted);
    }
}
