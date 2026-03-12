//! Cloud storage command handlers (push, pull, list, config).

use ai_model_vault::{Result, VaultConfig, VaultError};

use crate::cli::args::CloudCommands;
use crate::cli::helpers::{build_vault, prompt_passphrase};

pub fn handle_cloud(command: CloudCommands, config: VaultConfig, use_sqlite: bool) -> Result<()> {
    match command {
        CloudCommands::Push {
            model,
            version,
            provider,
            bucket,
        } => {
            println!("☁️  Pushing model to cloud storage");
            println!("   Model: {}", model);
            println!("   Provider: {}", provider);
            println!("   Bucket: {}", bucket);

            // Open vault and get model
            let mut vault = build_vault(config.clone(), use_sqlite)?;
            let passphrase = prompt_passphrase("Enter vault passphrase: ")?;
            vault.unlock(passphrase)?;

            // Get version to push
            let version_num = if let Some(v) = version {
                v
            } else {
                vault
                    .list_versions(&model)
                    .last()
                    .map(|mv| mv.version)
                    .ok_or_else(|| {
                        VaultError::ModelNotFound(format!(
                            "Model '{}' not found or has no versions",
                            model
                        ))
                    })?
            };

            // Get model data
            let _data = vault.get_model(&model, Some(version_num))?;
            let versions = vault.list_versions(&model);
            let model_version = versions
                .iter()
                .find(|v| v.version == version_num)
                .ok_or_else(|| VaultError::VersionNotFound(version_num, model.clone()))?;

            // Construct remote path
            let _remote_path = format!("{}/{}/v{}.vault", model, model_version.format, version_num);

            // Push to cloud based on provider
            match provider.to_lowercase().as_str() {
                "s3" => {
                    #[cfg(feature = "s3")]
                    {
                        use ai_model_vault::storage::StorageConfig;
                        println!("📤 Uploading to S3...");
                        let region =
                            std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
                        println!("   Region: {}", region);
                        println!("   Path: {}", _remote_path);
                        println!("   Size: {} bytes", _data.len());

                        let storage_config = StorageConfig::S3 {
                            bucket: bucket.clone(),
                            region,
                            prefix: None,
                        };
                        let rt = tokio::runtime::Runtime::new().map_err(|e| {
                            VaultError::StorageError(format!(
                                "Failed to create async runtime: {}",
                                e
                            ))
                        })?;
                        rt.block_on(async {
                            let backend = storage_config.create_backend().await?;
                            backend.upload(&_remote_path, &_data).await
                        })?;

                        println!("\n✅ Model pushed to S3 successfully!");
                        println!("   Bucket: {}", bucket);
                        println!("   Key: {}", _remote_path);
                    }
                    #[cfg(not(feature = "s3"))]
                    {
                        println!("⚠️  S3 support not enabled in this build");
                        println!("   To enable: cargo build --release --features s3");
                    }
                }
                "azure" => {
                    #[cfg(feature = "azure")]
                    {
                        use ai_model_vault::storage::StorageConfig;
                        let account = std::env::var("AZURE_STORAGE_ACCOUNT").map_err(|_| {
                            VaultError::ConfigError(
                                "AZURE_STORAGE_ACCOUNT env var not set".to_string(),
                            )
                        })?;
                        println!("📤 Uploading to Azure Blob Storage...");
                        println!("   Container: {}", bucket);
                        println!("   Path: {}", _remote_path);
                        println!("   Size: {} bytes", _data.len());

                        let storage_config = StorageConfig::Azure {
                            account,
                            container: bucket.clone(),
                            prefix: None,
                        };
                        let rt = tokio::runtime::Runtime::new().map_err(|e| {
                            VaultError::StorageError(format!(
                                "Failed to create async runtime: {}",
                                e
                            ))
                        })?;
                        rt.block_on(async {
                            let backend = storage_config.create_backend().await?;
                            backend.upload(&_remote_path, &_data).await
                        })?;

                        println!("\n✅ Model pushed to Azure successfully!");
                    }
                    #[cfg(not(feature = "azure"))]
                    {
                        println!("⚠️  Azure support not enabled in this build");
                        println!("   To enable: cargo build --release --features azure");
                    }
                }
                "gcs" => {
                    println!(
                        "⚠️  GCS support temporarily disabled due to security vulnerabilities"
                    );
                    println!("   Use S3 or Azure instead. See SECURITY_AUDIT.md for details.");
                }
                _ => {
                    return Err(VaultError::InvalidInput(format!(
                        "Unsupported provider: {}. Use 's3', 'azure', or 'gcs'",
                        provider
                    )));
                }
            }
        }

        CloudCommands::Pull {
            model,
            provider,
            bucket,
            remote_path,
        } => {
            println!("☁️  Pulling model from cloud storage");
            println!("   Model: {}", model);
            println!("   Provider: {}", provider);
            println!("   Bucket: {}", bucket);
            println!("   Remote path: {}", remote_path);

            match provider.to_lowercase().as_str() {
                "s3" => {
                    #[cfg(feature = "s3")]
                    {
                        use ai_model_vault::storage::StorageConfig;
                        let region =
                            std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
                        println!("📥 Downloading from S3...");

                        let storage_config = StorageConfig::S3 {
                            bucket: bucket.clone(),
                            region,
                            prefix: None,
                        };
                        let rt = tokio::runtime::Runtime::new().map_err(|e| {
                            VaultError::StorageError(format!(
                                "Failed to create async runtime: {}",
                                e
                            ))
                        })?;
                        let data = rt.block_on(async {
                            let backend = storage_config.create_backend().await?;
                            backend.download(&remote_path).await
                        })?;

                        // Store into vault
                        let passphrase = prompt_passphrase("Enter vault passphrase: ")?;
                        let mut vault = build_vault(config.clone(), use_sqlite)?;
                        vault.unlock(passphrase)?;

                        let model_format = ModelFormat::from_extension(
                            std::path::Path::new(&remote_path)
                                .extension()
                                .and_then(|s| s.to_str())
                                .unwrap_or("bin"),
                        );
                        let metadata = ModelMetadata::new(model.clone(), model_format);
                        let version = vault.store_model(&model, data, metadata, None)?;

                        println!("\n✅ Model pulled and stored successfully!");
                        println!("   Model: {} v{}", model, version.version);
                    }
                    #[cfg(not(feature = "s3"))]
                    {
                        println!("⚠️  S3 support not enabled in this build");
                        println!("   To enable: cargo build --release --features s3");
                    }
                }
                "azure" => {
                    #[cfg(feature = "azure")]
                    {
                        use ai_model_vault::storage::StorageConfig;
                        let account = std::env::var("AZURE_STORAGE_ACCOUNT").map_err(|_| {
                            VaultError::ConfigError(
                                "AZURE_STORAGE_ACCOUNT env var not set".to_string(),
                            )
                        })?;
                        println!("📥 Downloading from Azure Blob Storage...");

                        let storage_config = StorageConfig::Azure {
                            account,
                            container: bucket.clone(),
                            prefix: None,
                        };
                        let rt = tokio::runtime::Runtime::new().map_err(|e| {
                            VaultError::StorageError(format!(
                                "Failed to create async runtime: {}",
                                e
                            ))
                        })?;
                        let data = rt.block_on(async {
                            let backend = storage_config.create_backend().await?;
                            backend.download(&remote_path).await
                        })?;

                        // Store into vault
                        let passphrase = prompt_passphrase("Enter vault passphrase: ")?;
                        let mut vault = build_vault(config.clone(), use_sqlite)?;
                        vault.unlock(passphrase)?;

                        let model_format = ModelFormat::from_extension(
                            std::path::Path::new(&remote_path)
                                .extension()
                                .and_then(|s| s.to_str())
                                .unwrap_or("bin"),
                        );
                        let metadata = ModelMetadata::new(model.clone(), model_format);
                        let version = vault.store_model(&model, data, metadata, None)?;

                        println!("\n✅ Model pulled and stored successfully!");
                        println!("   Model: {} v{}", model, version.version);
                    }
                    #[cfg(not(feature = "azure"))]
                    {
                        println!("⚠️  Azure support not enabled in this build");
                        println!("   To enable: cargo build --release --features azure");
                    }
                }
                "gcs" => {
                    println!(
                        "⚠️  GCS support temporarily disabled due to security vulnerabilities"
                    );
                    println!("   Use S3 or Azure instead.");
                }
                _ => {
                    return Err(VaultError::InvalidInput(format!(
                        "Unsupported provider: {}. Use 's3', 'azure', or 'gcs'",
                        provider
                    )));
                }
            }
        }

        CloudCommands::List {
            provider,
            bucket,
            prefix,
        } => {
            println!("☁️  Listing cloud storage contents");
            println!("   Provider: {}", provider);
            println!("   Bucket: {}", bucket);
            if let Some(ref p) = prefix {
                println!("   Prefix: {}", p);
            }

            match provider.to_lowercase().as_str() {
                "s3" => {
                    #[cfg(feature = "s3")]
                    {
                        use ai_model_vault::storage::StorageConfig;
                        let region =
                            std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());

                        let storage_config = StorageConfig::S3 {
                            bucket: bucket.clone(),
                            region,
                            prefix: prefix.clone(),
                        };
                        let rt = tokio::runtime::Runtime::new().map_err(|e| {
                            VaultError::StorageError(format!(
                                "Failed to create async runtime: {}",
                                e
                            ))
                        })?;
                        let keys = rt.block_on(async {
                            let backend = storage_config.create_backend().await?;
                            backend.list().await
                        })?;

                        println!(
                            "\n📋 S3 Bucket '{}' Contents ({} items):",
                            bucket,
                            keys.len()
                        );
                        for key in &keys {
                            println!("   {}", key);
                        }
                        if keys.is_empty() {
                            println!("   (empty)");
                        }
                    }
                    #[cfg(not(feature = "s3"))]
                    {
                        println!("⚠️  S3 support not enabled in this build");
                        println!("   To enable: cargo build --release --features s3");
                    }
                }
                "azure" => {
                    #[cfg(feature = "azure")]
                    {
                        use ai_model_vault::storage::StorageConfig;
                        let account = std::env::var("AZURE_STORAGE_ACCOUNT").map_err(|_| {
                            VaultError::ConfigError(
                                "AZURE_STORAGE_ACCOUNT env var not set".to_string(),
                            )
                        })?;

                        let storage_config = StorageConfig::Azure {
                            account,
                            container: bucket.clone(),
                            prefix: prefix.clone(),
                        };
                        let rt = tokio::runtime::Runtime::new().map_err(|e| {
                            VaultError::StorageError(format!(
                                "Failed to create async runtime: {}",
                                e
                            ))
                        })?;
                        let keys = rt.block_on(async {
                            let backend = storage_config.create_backend().await?;
                            backend.list().await
                        })?;

                        println!(
                            "\n📋 Azure Container '{}' Contents ({} items):",
                            bucket,
                            keys.len()
                        );
                        for key in &keys {
                            println!("   {}", key);
                        }
                        if keys.is_empty() {
                            println!("   (empty)");
                        }
                    }
                    #[cfg(not(feature = "azure"))]
                    {
                        println!("⚠️  Azure support not enabled in this build");
                        println!("   To enable: cargo build --release --features azure");
                    }
                }
                "gcs" => {
                    println!(
                        "⚠️  GCS support temporarily disabled due to security vulnerabilities"
                    );
                    println!("   Use S3 or Azure instead.");
                }
                _ => {
                    return Err(VaultError::InvalidInput(format!(
                        "Unsupported provider: {}. Use 's3', 'azure', or 'gcs'",
                        provider
                    )));
                }
            }
        }

        CloudCommands::Config { provider, show } => {
            println!("☁️  Cloud Storage Configuration");
            println!("   Provider: {}", provider);

            if show {
                match provider.to_lowercase().as_str() {
                    "s3" => {
                        println!("\n📝 AWS S3 Configuration:");
                        println!("   Required environment variables:");
                        println!(
                            "   - AWS_ACCESS_KEY_ID: {}",
                            if std::env::var("AWS_ACCESS_KEY_ID").is_ok() {
                                "✅ Set"
                            } else {
                                "❌ Not set"
                            }
                        );
                        println!(
                            "   - AWS_SECRET_ACCESS_KEY: {}",
                            if std::env::var("AWS_SECRET_ACCESS_KEY").is_ok() {
                                "✅ Set"
                            } else {
                                "❌ Not set"
                            }
                        );
                        println!(
                            "   - AWS_REGION (optional): {}",
                            std::env::var("AWS_REGION")
                                .unwrap_or_else(|_| "Not set (defaults to us-east-1)".to_string())
                        );

                        println!("\n💡 To configure:");
                        println!("   export AWS_ACCESS_KEY_ID=your_access_key");
                        println!("   export AWS_SECRET_ACCESS_KEY=your_secret_key");
                        println!("   export AWS_REGION=us-east-1  # optional");
                    }
                    "azure" => {
                        println!("\n📝 Azure Blob Storage Configuration:");
                        println!("   Required environment variables:");
                        println!(
                            "   - AZURE_STORAGE_ACCOUNT: {}",
                            if std::env::var("AZURE_STORAGE_ACCOUNT").is_ok() {
                                "✅ Set"
                            } else {
                                "❌ Not set"
                            }
                        );
                        println!(
                            "   - AZURE_STORAGE_KEY: {}",
                            if std::env::var("AZURE_STORAGE_KEY").is_ok() {
                                "✅ Set"
                            } else {
                                "❌ Not set"
                            }
                        );

                        println!("\n💡 To configure:");
                        println!("   export AZURE_STORAGE_ACCOUNT=your_account_name");
                        println!("   export AZURE_STORAGE_KEY=your_account_key");
                    }
                    "gcs" => {
                        println!("\n📝 Google Cloud Storage Configuration:");
                        println!("   ⚠️  GCS support temporarily disabled due to security vulnerabilities");
                        println!("   Use S3 or Azure instead");
                        println!("\n   For details, see SECURITY_AUDIT.md");
                    }
                    _ => {
                        return Err(VaultError::InvalidInput(format!(
                            "Unsupported provider: {}. Use 's3', 'azure', or 'gcs'",
                            provider
                        )));
                    }
                }
            } else {
                println!("\n💡 Use --show flag to display current configuration");
                println!("   Example: aim cloud config --provider s3 --show");
            }
        }
    }

    Ok(())
}
