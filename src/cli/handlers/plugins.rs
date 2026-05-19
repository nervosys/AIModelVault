//! CLI handler for plugin management (aim plugin).

use ai_model_vault::{PluginRegistry, Result, VaultConfig};

use crate::cli::args::PluginCommands;

pub fn handle_plugin(command: PluginCommands, config: VaultConfig) -> Result<()> {
    let mut registry = PluginRegistry::new(&config.dirs.data_dir)?;

    match command {
        PluginCommands::List => {
            let plugins = registry.list();
            if plugins.is_empty() {
                println!("No plugins installed.");
            } else {
                for p in plugins {
                    println!(
                        "  {} v{} -- {}",
                        p.manifest.name, p.manifest.version, p.manifest.description
                    );
                }
            }
        }
        PluginCommands::Install { path } => {
            // Load manifest from the plugin directory
            let manifest_path = path.join("plugin.json");
            let data = std::fs::read_to_string(&manifest_path)?;
            let manifest: ai_model_vault::plugins::PluginManifest = serde_json::from_str(&data)
                .map_err(|e| ai_model_vault::VaultError::SerializationError(e.to_string()))?;
            registry.install(manifest)?;
            println!("Plugin installed from {:?}", path);
        }
        PluginCommands::Uninstall { id } => {
            registry.uninstall(&id)?;
            println!("Plugin '{}' uninstalled", id);
        }
        PluginCommands::Discover => {
            let found = registry.discover()?;
            println!("Discovered {} plugin(s)", found);
        }
    }

    Ok(())
}
