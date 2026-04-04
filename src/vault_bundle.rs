//! Vault export/import — portable encrypted vault bundles.
//!
//! Exports selected models (with all versions and metadata) to a self-contained
//! `.aimvault` archive. Imports merge models back into a vault.

use std::collections::HashMap;
use std::fs;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Result, VaultError};

// ── Manifest ─────────────────────────────────────────────────────────────────

/// Bundle manifest stored inside the archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleManifest {
    /// Bundle format version
    pub format_version: u32,
    /// Vault name that produced this bundle
    pub source_vault: String,
    /// When the bundle was created
    pub created_at: String,
    /// Models included (name → list of versions)
    pub models: HashMap<String, Vec<u32>>,
    /// SHA-256 of the data section for integrity
    pub data_checksum: String,
}

// ── Export ────────────────────────────────────────────────────────────────────

/// Export selected models from a vault into a portable archive.
///
/// The output is a tar file containing:
///   - `manifest.json` — the bundle manifest
///   - `versions.json` — version metadata for included models
///   - `data/<uuid>.vault` — encrypted blobs
///   - `tags.json` — tag data for included models (if any)
pub fn export_vault(
    vault_path: &Path,
    output: &Path,
    model_filter: Option<&[String]>,
) -> Result<ExportReport> {
    use crate::version::VersionControl;

    let vc = VersionControl::new(vault_path)?;
    let all_models = vc.list_models_owned();

    let models_to_export: Vec<String> = if let Some(filter) = model_filter {
        // Support glob-like patterns (* → match any)
        all_models
            .into_iter()
            .filter(|m| {
                filter.iter().any(|pat| {
                    if pat.contains('*') {
                        let pat_lower = pat.to_lowercase().replace('*', "");
                        m.to_lowercase().contains(&pat_lower)
                    } else {
                        m == pat
                    }
                })
            })
            .collect()
    } else {
        all_models
    };

    if models_to_export.is_empty() {
        return Err(VaultError::InvalidInput(
            "No models matched the export filter".to_string(),
        ));
    }

    // Collect version data for selected models
    let mut version_data: HashMap<String, Vec<crate::version::ModelVersion>> = HashMap::new();
    let mut blob_files: Vec<String> = Vec::new();

    for model in &models_to_export {
        let versions: Vec<crate::version::ModelVersion> =
            vc.list_versions(model).into_iter().cloned().collect();
        for v in &versions {
            blob_files.push(v.file_path.clone());
        }
        version_data.insert(model.clone(), versions);
    }

    // Build manifest
    let models_summary: HashMap<String, Vec<u32>> = version_data
        .iter()
        .map(|(name, vers)| (name.clone(), vers.iter().map(|v| v.version).collect()))
        .collect();

    // Write tar archive
    let out_file = fs::File::create(output)?;
    let mut tar_builder = tar::Builder::new(out_file);

    // Write versions.json
    let versions_json = serde_json::to_string_pretty(&version_data)?;
    let versions_bytes = versions_json.as_bytes();
    let mut header = tar::Header::new_gnu();
    header.set_size(versions_bytes.len() as u64);
    header.set_mode(0o600);
    header.set_cksum();
    tar_builder.append_data(&mut header, "versions.json", versions_bytes)?;

    // Copy blob files
    let data_dir = vault_path.join("data");
    let mut data_hash = sha2::Sha256::new();
    use sha2::Digest;

    for blob in &blob_files {
        let blob_path = data_dir.join(blob);
        if blob_path.exists() {
            let blob_data = fs::read(&blob_path)?;
            data_hash.update(&blob_data);

            let mut header = tar::Header::new_gnu();
            header.set_size(blob_data.len() as u64);
            header.set_mode(0o600);
            header.set_cksum();
            let archive_path = format!("data/{}", blob);
            tar_builder.append_data(&mut header, &archive_path, blob_data.as_slice())?;
        }
    }

    // Write tags (if present)
    let tags_path = vault_path.join("tags.json");
    if tags_path.exists() {
        let tags_data = fs::read(&tags_path)?;
        let mut header = tar::Header::new_gnu();
        header.set_size(tags_data.len() as u64);
        header.set_mode(0o600);
        header.set_cksum();
        tar_builder.append_data(&mut header, "tags.json", tags_data.as_slice())?;
    }

    let checksum = hex::encode(data_hash.finalize());
    let manifest = BundleManifest {
        format_version: 1,
        source_vault: vault_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        models: models_summary,
        data_checksum: checksum,
    };

    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    let manifest_bytes = manifest_json.as_bytes();
    let mut header = tar::Header::new_gnu();
    header.set_size(manifest_bytes.len() as u64);
    header.set_mode(0o600);
    header.set_cksum();
    tar_builder.append_data(&mut header, "manifest.json", manifest_bytes)?;

    tar_builder.finish()?;

    Ok(ExportReport {
        output_path: output.to_path_buf(),
        models_exported: manifest.models.keys().cloned().collect(),
        total_versions: manifest.models.values().map(|v| v.len()).sum(),
        total_blobs: blob_files.len(),
    })
}

/// Import models from a vault bundle archive.
pub fn import_vault(
    vault_path: &Path,
    archive_path: &Path,
    overwrite: bool,
) -> Result<ImportReport> {
    use crate::version::VersionControl;

    let file = fs::File::open(archive_path)?;
    let mut archive = tar::Archive::new(file);

    let temp_dir = tempfile::tempdir().map_err(VaultError::IoError)?;

    // Extract everything to temp dir
    archive.unpack(temp_dir.path())?;

    // Read manifest
    let manifest_path = temp_dir.path().join("manifest.json");
    if !manifest_path.exists() {
        return Err(VaultError::InvalidInput(
            "Invalid vault bundle: missing manifest.json".to_string(),
        ));
    }
    let manifest: BundleManifest = serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;

    // Read exported versions
    let versions_path = temp_dir.path().join("versions.json");
    let imported_versions: HashMap<String, Vec<crate::version::ModelVersion>> =
        serde_json::from_str(&fs::read_to_string(&versions_path)?)?;

    // Merge into target vault
    let mut vc = VersionControl::new(vault_path)?;
    let existing_models: Vec<String> = vc.list_models_owned();
    let data_dir = vault_path.join("data");
    fs::create_dir_all(&data_dir)?;

    let mut models_imported = 0usize;
    let mut versions_imported = 0usize;
    let mut skipped = 0usize;

    for (model_name, versions) in &imported_versions {
        if existing_models.contains(model_name) && !overwrite {
            skipped += versions.len();
            continue;
        }
        models_imported += 1;

        for version in versions {
            // Copy blob
            let src_blob = temp_dir.path().join("data").join(&version.file_path);
            let dst_blob = data_dir.join(&version.file_path);
            if src_blob.exists() {
                fs::copy(&src_blob, &dst_blob)?;
                crate::permissions::restrict_file(&dst_blob)?;
            }

            // Add version to VC
            vc.import_version(model_name, version.clone())?;
            versions_imported += 1;
        }
    }

    // Merge tags if present
    let imported_tags = temp_dir.path().join("tags.json");
    if imported_tags.exists() {
        let mut target_tags = crate::tags::TagStore::new(vault_path)?;
        let src_data: crate::tags::TagData =
            serde_json::from_str(&fs::read_to_string(&imported_tags)?)?;

        for (model, tags) in &src_data.tags {
            if imported_versions.contains_key(model) {
                let tag_vec: Vec<String> = tags.iter().cloned().collect();
                target_tags.add_tags(model, &tag_vec)?;
            }
        }
        for (model, annots) in &src_data.annotations {
            if imported_versions.contains_key(model) {
                for (k, v) in annots {
                    target_tags.set_annotation(model, k, v)?;
                }
            }
        }
    }

    Ok(ImportReport {
        source_vault: manifest.source_vault,
        models_imported,
        versions_imported,
        versions_skipped: skipped,
    })
}

// ── Reports ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ExportReport {
    pub output_path: PathBuf,
    pub models_exported: Vec<String>,
    pub total_versions: usize,
    pub total_blobs: usize,
}

#[derive(Debug, Serialize)]
pub struct ImportReport {
    pub source_vault: String,
    pub models_imported: usize,
    pub versions_imported: usize,
    pub versions_skipped: usize,
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_manifest_roundtrip() {
        let manifest = BundleManifest {
            format_version: 1,
            source_vault: "test".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            models: {
                let mut m = HashMap::new();
                m.insert("llama".to_string(), vec![1, 2, 3]);
                m
            },
            data_checksum: "abc123".to_string(),
        };
        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: BundleManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.format_version, 1);
        assert_eq!(parsed.models.get("llama").unwrap().len(), 3);
    }

    #[test]
    fn test_export_report_fields() {
        let report = ExportReport {
            output_path: PathBuf::from("/tmp/test.aimvault"),
            models_exported: vec!["m1".into(), "m2".into()],
            total_versions: 5,
            total_blobs: 5,
        };
        assert_eq!(report.models_exported.len(), 2);
        assert_eq!(report.total_versions, 5);
    }
}
