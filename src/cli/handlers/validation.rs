//! CLI handler for model validation (aim validate).

use ai_model_vault::{Result, ValidationStore, VaultConfig};

pub fn handle_validate(
    name: String,
    version: Option<u32>,
    config: VaultConfig,
    _use_sqlite: bool,
) -> Result<()> {
    let store = ValidationStore::new(&config.dirs.vault_dir)?;

    // Resolve file path from version control
    let vc = ai_model_vault::version::VersionControl::new(&config.dirs.vault_dir)?;
    let versions = vc.list_versions(&name);
    let ver = version.unwrap_or(0);
    let target = if ver == 0 {
        versions.last()
    } else {
        versions.iter().find(|v| v.version == ver)
    };

    match target {
        Some(v) => {
            let file_path = std::path::PathBuf::from(&v.file_path);
            let report = store.validate(&name, &file_path)?;
            println!("Validation for '{}' (v{}):", name, v.version);
            for r in &report.results {
                let icon = if r.passed { "✓" } else { "✗" };
                println!("  {} {}: {}", icon, r.probe_label, r.message);
            }
            if report.overall_pass {
                println!("All checks passed.");
            } else {
                println!("Some checks failed.");
            }
        }
        None => {
            println!("No version found for '{}'", name);
        }
    }

    Ok(())
}
