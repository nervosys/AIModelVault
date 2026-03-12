//! Archive and extract command handlers.

use ai_model_vault::utils::ModelArchive;
use ai_model_vault::{Result, VaultConfig};

use crate::cli::helpers::{build_vault, prompt_passphrase};

pub fn handle_archive(
    models: Vec<String>,
    output: std::path::PathBuf,
    format: String,
    versions: Option<Vec<u32>>,
    config: VaultConfig,
    use_sqlite: bool,
) -> Result<()> {
    let passphrase = prompt_passphrase("Enter vault passphrase: ")?;

    let mut vault = build_vault(config, use_sqlite)?;
    vault.unlock(passphrase)?;

    println!("Archiving {} models...", models.len());

    let mut archive_data = Vec::new();
    for (i, model_name) in models.iter().enumerate() {
        let version = versions.as_ref().and_then(|v| v.get(i).copied());
        let data = vault.get_model(model_name, version)?;
        archive_data.push((model_name.clone(), data));
        println!("  ✓ Loaded '{}'", model_name);
    }

    let total = match format.to_lowercase().as_str() {
        "tar" => ModelArchive::create_tar(archive_data, &output)?,
        "zip" => ModelArchive::create_zip(archive_data, &output)?,
        _ => {
            eprintln!("Unknown format: {}. Use 'tar' or 'zip'", format);
            std::process::exit(1);
        }
    };

    println!("✓ Archive created: {:?} ({} bytes)", output, total);
    Ok(())
}

pub fn handle_extract(archive: std::path::PathBuf, output: std::path::PathBuf) -> Result<()> {
    println!("Extracting archive...");

    let ext = archive
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("tar");

    let models = match ext {
        "tar" => ModelArchive::extract_tar(&archive)?,
        "zip" => ModelArchive::extract_zip(&archive)?,
        _ => {
            eprintln!("Unknown archive format. Expected .tar or .zip");
            std::process::exit(1);
        }
    };

    std::fs::create_dir_all(&output)?;

    let count = models.len();
    for (name, data) in models {
        let file_path = output.join(&name);
        std::fs::write(&file_path, &data)?;
        println!("  ✓ Extracted '{}' ({} bytes)", name, data.len());
    }

    println!("✓ Extracted {} models to {:?}", count, output);
    Ok(())
}
