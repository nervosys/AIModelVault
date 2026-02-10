//! Vault core command handlers (init, store, get, list, versions, lineage, delete, stats, compliance, change-passphrase, cache).

use ai_model_vault::compliance::ComplianceChecker;
use ai_model_vault::formats::{ModelFormat, ModelMetadata};
use ai_model_vault::{Result, Vault, VaultConfig, VaultError};
use std::io::{self, Write};

use crate::cli::helpers::prompt_passphrase;

pub fn handle_init(name: String, config: VaultConfig) -> Result<()> {
    println!("Initializing vault: {}", name);
    let vault = Vault::new(Some(config))?;
    println!("✓ Vault '{}' initialized successfully", name);
    println!("Location: {:?}", vault.get_config().dirs.vault_dir);
    Ok(())
}

pub fn handle_store(
    name: String,
    path: std::path::PathBuf,
    format: Option<String>,
    description: Option<String>,
    framework: Option<String>,
    task: Option<String>,
    config: VaultConfig,
) -> Result<()> {
    // Read model file
    let data = std::fs::read(&path)?;
    println!("Read {} bytes from {:?}", data.len(), path);

    // Detect format
    let model_format = if let Some(fmt) = format {
        match fmt.to_lowercase().as_str() {
            // LLM formats
            "safetensors" => ModelFormat::Safetensors,
            "gguf" => ModelFormat::GGUF,
            "pytorch" | "pt" | "torch" => ModelFormat::PyTorch,
            "tensorrt" | "trt" => ModelFormat::TensorRT,
            "onnx" => ModelFormat::ONNX,
            "mlx" => ModelFormat::MLX,
            "coreml" | "mlmodel" => ModelFormat::CoreML,
            "torchscript" => ModelFormat::TorchScript,
            "tflite" | "tensorflow-lite" => ModelFormat::TFLite,
            // General DL formats
            "tensorflow" | "tf" | "savedmodel" => ModelFormat::TensorFlow,
            "keras" | "h5" => ModelFormat::Keras,
            "openvino" => ModelFormat::OpenVINO,
            "tvm" => ModelFormat::TVM,
            "ncnn" => ModelFormat::NCNN,
            "mnn" => ModelFormat::MNN,
            "rknn" => ModelFormat::RKNN,
            // Legacy formats
            "caffe" => ModelFormat::Caffe,
            "mxnet" => ModelFormat::MXNet,
            "darknet" => ModelFormat::Darknet,
            // Data formats
            "hdf5" => ModelFormat::HDF5,
            "pickle" | "pkl" => ModelFormat::Pickle,
            "numpy" | "npy" => ModelFormat::NumPy,
            _ => ModelFormat::Custom(fmt),
        }
    } else {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("bin");
        ModelFormat::from_extension(ext)
    };

    // Create metadata
    let mut metadata = ModelMetadata::new(name.clone(), model_format);
    if let Some(desc) = description {
        metadata = metadata.with_description(desc);
    }
    if let Some(fw) = framework {
        metadata = metadata.with_framework(fw);
    }
    if let Some(t) = task {
        metadata = metadata.with_task(t);
    }

    // Get passphrase
    let passphrase = prompt_passphrase("Enter vault passphrase: ")?;

    // Store model
    let mut vault = Vault::new(Some(config))?;
    vault.unlock(passphrase)?;
    let version = vault.store_model(&name, data, metadata, None)?;

    println!("✓ Model '{}' stored successfully", name);
    println!("  Version: {}", version.version);
    println!("  Checkpoint ID: {}", version.checkpoint_id);
    println!("  Original size: {} bytes", version.size_bytes);
    println!("  Compressed size: {} bytes", version.compressed_size_bytes);
    println!(
        "  Compression ratio: {:.1}%",
        (1.0 - version.compressed_size_bytes as f64 / version.size_bytes as f64) * 100.0
    );
    Ok(())
}

pub fn handle_get(
    name: String,
    output: std::path::PathBuf,
    version: Option<u32>,
    config: VaultConfig,
) -> Result<()> {
    let passphrase = prompt_passphrase("Enter vault passphrase: ")?;

    let mut vault = Vault::new(Some(config))?;
    vault.unlock(passphrase)?;

    let data = vault.get_model(&name, version)?;
    std::fs::write(&output, &data)?;

    println!("✓ Model '{}' retrieved successfully", name);
    println!("  Written to: {:?}", output);
    println!("  Size: {} bytes", data.len());
    Ok(())
}

pub fn handle_list(config: VaultConfig) -> Result<()> {
    let passphrase = prompt_passphrase("Enter vault passphrase: ")?;

    let mut vault = Vault::new(Some(config))?;
    vault.unlock(passphrase)?;

    let models = vault.list_models();

    if models.is_empty() {
        println!("No models in vault");
    } else {
        println!("Models in vault:");
        for model in models {
            let versions = vault.list_versions(&model);
            println!("  {} ({} versions)", model, versions.len());
        }
    }
    Ok(())
}

pub fn handle_versions(name: String, config: VaultConfig) -> Result<()> {
    let vault = Vault::new(Some(config))?;
    let versions = vault.list_versions(&name);

    if versions.is_empty() {
        println!("No versions found for model '{}'", name);
    } else {
        println!("Versions of '{}':", name);
        for v in versions {
            println!(
                "  v{} - {} - {} bytes ({})",
                v.version,
                v.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                v.size_bytes,
                v.format
            );
        }
    }
    Ok(())
}

pub fn handle_lineage(name: String, version: u32, config: VaultConfig) -> Result<()> {
    let vault = Vault::new(Some(config))?;
    let lineage = vault.get_lineage(&name, version);

    if lineage.is_empty() {
        println!("Version {} not found for model '{}'", version, name);
    } else {
        println!("Lineage for '{}' v{}:", name, version);
        for (i, v) in lineage.iter().enumerate() {
            println!(
                "  {}v{} - {} - {}",
                "  ".repeat(i),
                v.version,
                v.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                v.checkpoint_id
            );
        }
    }
    Ok(())
}

pub fn handle_delete(name: String, version: u32, force: bool, config: VaultConfig) -> Result<()> {
    if !force {
        print!(
            "Are you sure you want to delete '{}' v{}? [y/N]: ",
            name, version
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled");
            return Ok(());
        }
    }

    let passphrase = prompt_passphrase("Enter vault passphrase: ")?;

    let mut vault = Vault::new(Some(config))?;
    vault.unlock(passphrase)?;

    if vault.delete_version(&name, version)? {
        println!("✓ Deleted '{}' v{}", name, version);
    } else {
        println!("Version not found");
    }
    Ok(())
}

pub fn handle_stats(config: VaultConfig) -> Result<()> {
    let vault = Vault::new(Some(config))?;
    let stats = vault.get_stats()?;

    println!("Vault Statistics:");
    println!("  Models: {}", stats.model_count);
    println!("  Total versions: {}", stats.total_versions);
    println!(
        "  Total size: {} bytes ({:.2} MB)",
        stats.total_size_bytes,
        stats.total_size_bytes as f64 / 1_048_576.0
    );
    println!("  Files: {}", stats.file_count);
    Ok(())
}

pub fn handle_compliance() -> Result<()> {
    println!("Running compliance checks...\n");

    let checker = ComplianceChecker::new();
    let status = checker.run_all_checks()?;

    println!("Compliance Status:");
    println!(
        "  FIPS 140-3: {}",
        if status.fips_140_3 {
            "✓ PASS"
        } else {
            "✗ FAIL"
        }
    );
    println!(
        "  CVE Scan: {}",
        if status.cve_scan_passed {
            "✓ PASS"
        } else {
            "✗ FAIL"
        }
    );
    println!(
        "  MITRE ATT&CK: {}",
        if status.mitre_attack_aligned {
            "✓ PASS"
        } else {
            "✗ FAIL"
        }
    );
    println!("  CMMC Level: {}", status.cmmc_level);

    if !status.violations.is_empty() {
        println!("\nViolations:");
        for violation in status.violations {
            println!(
                "  [{:?}] {} - {}: {}",
                violation.severity,
                violation.standard,
                violation.control,
                violation.description
            );
        }
    } else {
        println!("\n✓ No violations detected");
    }
    Ok(())
}

pub fn handle_change_passphrase(config: VaultConfig) -> Result<()> {
    let old_passphrase = prompt_passphrase("Enter current vault passphrase: ")?;
    let new_passphrase = prompt_passphrase("Enter new vault passphrase: ")?;
    let confirm_passphrase = prompt_passphrase("Confirm new vault passphrase: ")?;

    if new_passphrase != confirm_passphrase {
        return Err(VaultError::InvalidInput(
            "Passphrases do not match".to_string(),
        ));
    }

    let mut vault = Vault::new(Some(config))?;
    vault.unlock(old_passphrase)?;
    let count = vault.change_passphrase(new_passphrase)?;

    println!("✓ Passphrase changed successfully");
    println!("  Re-encrypted {} model file(s)", count);
    Ok(())
}

pub fn handle_cache() -> Result<()> {
    use ai_model_vault::utils::RetrievalOptimizer;
    let cache = RetrievalOptimizer::new(1024 * 1024 * 1024); // 1 GB
    let stats = cache.cache_stats();
    println!("Cache Statistics:");
    println!("  Capacity: {:.2} MB", stats.max_size as f64 / 1_048_576.0);
    println!("  Used: {:.2} MB", stats.total_size as f64 / 1_048_576.0);
    println!("  Entries: {}", stats.total_entries);
    println!("  Utilization: {:.1}%", stats.utilization);
    println!("\n💡 The cache is per-process. Use RetrievalOptimizer in your");
    println!("   application code for persistent caching.");
    Ok(())
}
