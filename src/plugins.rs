//! Plugin system — dynamic extension loading for custom commands and hooks.
//!
//! Plugins are shared libraries (`.so` / `.dll` / `.dylib`) placed in
//! `$XDG_DATA_HOME/ai/models/plugins/`.  Each plugin exports a set of
//! well-known symbols that register custom commands, event subscribers, or
//! storage backends.
//!
//! This module provides the plugin manifest, discovery, and registry — actual
//! dynamic loading requires the `libloading` crate (not included to avoid
//! adding a mandatory dependency; users can integrate via the registry API).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Result, VaultError};

// ── Types ────────────────────────────────────────────────────────────────────

/// Plugin manifest (discovered from `plugin.json` inside the plugin dir).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique plugin identifier (e.g. "my-company.custom-converter").
    pub id: String,
    /// Display name.
    pub name: String,
    /// Semantic version.
    pub version: String,
    /// Short description.
    pub description: String,
    /// Author / organisation.
    pub author: Option<String>,
    /// Minimum AIM version required.
    pub min_aim_version: Option<String>,
    /// Capabilities provided.
    pub capabilities: Vec<String>,
    /// Entry-point library filename (e.g. "libplugin.so").
    pub entry_point: Option<String>,
}

/// Runtime state for a loaded plugin.
#[derive(Debug, Clone, Serialize)]
pub struct PluginInfo {
    pub manifest: PluginManifest,
    pub path: PathBuf,
    pub loaded: bool,
}

// ── Registry ─────────────────────────────────────────────────────────────────

/// Plugin registry — discovers, loads, and tracks plugins.
pub struct PluginRegistry {
    plugins_dir: PathBuf,
    plugins: BTreeMap<String, PluginInfo>,
}

impl PluginRegistry {
    /// Create a new registry scanning the given directory.
    pub fn new(plugins_dir: &Path) -> Result<Self> {
        let mut registry = Self {
            plugins_dir: plugins_dir.to_path_buf(),
            plugins: BTreeMap::new(),
        };
        registry.discover()?;
        Ok(registry)
    }

    /// Scan the plugins directory for manifests.
    pub fn discover(&mut self) -> Result<usize> {
        self.plugins.clear();

        if !self.plugins_dir.is_dir() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Each plugin lives in a subdirectory with a plugin.json
            let manifest_path = if path.is_dir() {
                path.join("plugin.json")
            } else if path.extension().and_then(|e| e.to_str()) == Some("json")
                && path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.ends_with(".plugin"))
                    .unwrap_or(false)
            {
                path.clone()
            } else {
                continue;
            };

            if manifest_path.exists() {
                match self.load_manifest(&manifest_path) {
                    Ok(manifest) => {
                        let info = PluginInfo {
                            manifest: manifest.clone(),
                            path: manifest_path
                                .parent()
                                .unwrap_or(&self.plugins_dir)
                                .to_path_buf(),
                            loaded: false,
                        };
                        self.plugins.insert(manifest.id.clone(), info);
                        count += 1;
                    }
                    Err(e) => {
                        tracing::warn!("Skipping plugin at {}: {}", manifest_path.display(), e);
                    }
                }
            }
        }

        Ok(count)
    }

    fn load_manifest(&self, path: &Path) -> Result<PluginManifest> {
        let data = fs::read_to_string(path)?;
        let manifest: PluginManifest = serde_json::from_str(&data)?;

        if manifest.id.is_empty() {
            return Err(VaultError::InvalidInput(
                "Plugin manifest missing 'id' field".into(),
            ));
        }

        Ok(manifest)
    }

    /// List all discovered plugins.
    pub fn list(&self) -> Vec<&PluginInfo> {
        self.plugins.values().collect()
    }

    /// Get a plugin by ID.
    pub fn get(&self, id: &str) -> Option<&PluginInfo> {
        self.plugins.get(id)
    }

    /// Install a plugin from a manifest struct and optional files.
    pub fn install(&mut self, manifest: PluginManifest) -> Result<()> {
        let plugin_dir = self.plugins_dir.join(&manifest.id);
        fs::create_dir_all(&plugin_dir)?;

        let manifest_path = plugin_dir.join("plugin.json");
        let json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, json)?;

        let info = PluginInfo {
            manifest: manifest.clone(),
            path: plugin_dir,
            loaded: false,
        };
        self.plugins.insert(manifest.id, info);
        Ok(())
    }

    /// Uninstall a plugin by ID (removes its directory).
    pub fn uninstall(&mut self, id: &str) -> Result<bool> {
        if let Some(info) = self.plugins.remove(id) {
            if info.path.is_dir() {
                fs::remove_dir_all(&info.path)?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Display a summary.
    pub fn display(&self) -> String {
        let mut out = String::new();
        if self.plugins.is_empty() {
            out.push_str("No plugins installed.\n");
            return out;
        }

        out.push_str(&format!(
            "{:<30} {:<10} {}\n",
            "ID", "Version", "Description"
        ));
        out.push_str(&format!("{}\n", "─".repeat(60)));

        for info in self.plugins.values() {
            out.push_str(&format!(
                "{:<30} {:<10} {}\n",
                info.manifest.id, info.manifest.version, info.manifest.description
            ));
        }
        out
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_registry() {
        let dir = tempfile::tempdir().unwrap();
        let plugins_dir = dir.path().join("plugins");
        fs::create_dir_all(&plugins_dir).unwrap();
        let registry = PluginRegistry::new(&plugins_dir).unwrap();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_discover_plugin() {
        let dir = tempfile::tempdir().unwrap();
        let plugins_dir = dir.path().join("plugins");
        let plugin_dir = plugins_dir.join("test-plugin");
        fs::create_dir_all(&plugin_dir).unwrap();

        let manifest = PluginManifest {
            id: "test-plugin".into(),
            name: "Test Plugin".into(),
            version: "1.0.0".into(),
            description: "A test plugin".into(),
            author: Some("Test".into()),
            min_aim_version: None,
            capabilities: vec!["custom-command".into()],
            entry_point: None,
        };

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(plugin_dir.join("plugin.json"), json).unwrap();

        let registry = PluginRegistry::new(&plugins_dir).unwrap();
        assert_eq!(registry.list().len(), 1);
        assert_eq!(
            registry.get("test-plugin").unwrap().manifest.name,
            "Test Plugin"
        );
    }

    #[test]
    fn test_install_and_uninstall() {
        let dir = tempfile::tempdir().unwrap();
        let plugins_dir = dir.path().join("plugins");
        fs::create_dir_all(&plugins_dir).unwrap();

        let mut registry = PluginRegistry::new(&plugins_dir).unwrap();

        let manifest = PluginManifest {
            id: "my-plugin".into(),
            name: "My Plugin".into(),
            version: "0.1.0".into(),
            description: "Custom converter".into(),
            author: None,
            min_aim_version: Some("1.3.0".into()),
            capabilities: vec![],
            entry_point: None,
        };

        registry.install(manifest).unwrap();
        assert_eq!(registry.list().len(), 1);

        assert!(registry.uninstall("my-plugin").unwrap());
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_display() {
        let dir = tempfile::tempdir().unwrap();
        let plugins_dir = dir.path().join("plugins");
        fs::create_dir_all(&plugins_dir).unwrap();

        let mut registry = PluginRegistry::new(&plugins_dir).unwrap();
        registry
            .install(PluginManifest {
                id: "demo".into(),
                name: "Demo".into(),
                version: "1.0.0".into(),
                description: "Demo plugin".into(),
                author: None,
                min_aim_version: None,
                capabilities: vec![],
                entry_point: None,
            })
            .unwrap();

        let text = registry.display();
        assert!(text.contains("demo"));
    }
}
