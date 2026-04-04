//! CLI handler for TUI browse (aim browse).

use ai_model_vault::{Result, VaultConfig};

pub fn handle_browse(config: VaultConfig) -> Result<()> {
    let output = ai_model_vault::tui::browse(&config.dirs.vault_dir)?;
    println!("{}", output);
    Ok(())
}
