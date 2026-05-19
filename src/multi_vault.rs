//! Multi-vault management.
//!
//! Track multiple vault locations, set an active vault, and switch
//! between vaults from a single CLI session.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::{Result, VaultError};

// ── Types ────────────────────────────────────────────────────────────────────

/// A registered vault entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    /// Display name / alias.
    pub name: String,
    /// Absolute path to the vault directory.
    pub path: PathBuf,
    /// Optional description.
    pub description: Option<String>,
    /// When this entry was registered.
    pub registered_at: String,
}

/// Summary of a registered vault.
#[derive(Debug, Clone, Serialize)]
pub struct VaultSummary {
    pub name: String,
    pub path: PathBuf,
    pub is_active: bool,
    pub exists: bool,
}

// ── Registry ─────────────────────────────────────────────────────────────────

const REGISTRY_FILE: &str = "vault_registry.json";

#[derive(Debug, Default, Serialize, Deserialize)]
struct RegistryData {
    active: Option<String>,
    vaults: BTreeMap<String, VaultEntry>,
}

/// Manages a registry of vaults with activate/switch support.
#[derive(Debug)]
pub struct VaultRegistry {
    path: PathBuf,
    data: RegistryData,
}

impl VaultRegistry {
    /// Open or create a vault registry in the given config directory.
    pub fn new(config_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(config_dir)
            .map_err(|e| VaultError::StorageError(format!("create config dir: {e}")))?;

        let path = config_dir.join(REGISTRY_FILE);
        let data = if path.exists() {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| VaultError::StorageError(format!("read registry: {e}")))?;
            serde_json::from_str(&text).unwrap_or_default()
        } else {
            RegistryData::default()
        };

        Ok(Self { path, data })
    }

    fn save(&self) -> Result<()> {
        let text = serde_json::to_string_pretty(&self.data)
            .map_err(|e| VaultError::StorageError(format!("serialize registry: {e}")))?;
        std::fs::write(&self.path, text)
            .map_err(|e| VaultError::StorageError(format!("write registry: {e}")))
    }

    /// Register a vault.
    pub fn register(&mut self, entry: VaultEntry) -> Result<()> {
        self.data.vaults.insert(entry.name.clone(), entry);
        self.save()
    }

    /// Unregister a vault by name.
    pub fn unregister(&mut self, name: &str) -> Result<bool> {
        let removed = self.data.vaults.remove(name).is_some();
        if removed {
            if self.data.active.as_deref() == Some(name) {
                self.data.active = None;
            }
            self.save()?;
        }
        Ok(removed)
    }

    /// Set the active vault.
    pub fn activate(&mut self, name: &str) -> Result<()> {
        if !self.data.vaults.contains_key(name) {
            return Err(VaultError::InvalidInput(format!(
                "Vault '{name}' is not registered"
            )));
        }
        self.data.active = Some(name.to_string());
        self.save()
    }

    /// Clear the active vault.
    pub fn deactivate(&mut self) -> Result<()> {
        self.data.active = None;
        self.save()
    }

    /// Get the active vault's path.
    pub fn active_path(&self) -> Option<&Path> {
        self.data
            .active
            .as_deref()
            .and_then(|name| self.data.vaults.get(name))
            .map(|e| e.path.as_path())
    }

    /// Get the active vault name.
    pub fn active_name(&self) -> Option<&str> {
        self.data.active.as_deref()
    }

    /// List all registered vaults.
    pub fn list(&self) -> Vec<VaultSummary> {
        self.data
            .vaults
            .values()
            .map(|e| VaultSummary {
                name: e.name.clone(),
                path: e.path.clone(),
                is_active: self.data.active.as_deref() == Some(e.name.as_str()),
                exists: e.path.exists(),
            })
            .collect()
    }

    /// Get a vault entry by name.
    pub fn get(&self, name: &str) -> Option<&VaultEntry> {
        self.data.vaults.get(name)
    }

    /// Total number of registered vaults.
    pub fn count(&self) -> usize {
        self.data.vaults.len()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_entry(name: &str) -> VaultEntry {
        VaultEntry {
            name: name.into(),
            path: PathBuf::from(format!("/vaults/{name}")),
            description: Some(format!("Vault {name}")),
            registered_at: "2025-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn test_register_and_list() {
        let dir = tempdir().unwrap();
        let mut reg = VaultRegistry::new(dir.path()).unwrap();

        reg.register(make_entry("prod")).unwrap();
        reg.register(make_entry("staging")).unwrap();

        assert_eq!(reg.count(), 2);
        let list = reg.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_activate_deactivate() {
        let dir = tempdir().unwrap();
        let mut reg = VaultRegistry::new(dir.path()).unwrap();

        reg.register(make_entry("prod")).unwrap();
        assert!(reg.active_name().is_none());

        reg.activate("prod").unwrap();
        assert_eq!(reg.active_name(), Some("prod"));
        assert_eq!(reg.active_path().unwrap(), Path::new("/vaults/prod"));

        reg.deactivate().unwrap();
        assert!(reg.active_name().is_none());
    }

    #[test]
    fn test_activate_nonexistent() {
        let dir = tempdir().unwrap();
        let mut reg = VaultRegistry::new(dir.path()).unwrap();

        assert!(reg.activate("nonexistent").is_err());
    }

    #[test]
    fn test_unregister() {
        let dir = tempdir().unwrap();
        let mut reg = VaultRegistry::new(dir.path()).unwrap();

        reg.register(make_entry("prod")).unwrap();
        reg.activate("prod").unwrap();

        reg.unregister("prod").unwrap();
        assert_eq!(reg.count(), 0);
        assert!(reg.active_name().is_none()); // auto-cleared
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();

        {
            let mut reg = VaultRegistry::new(dir.path()).unwrap();
            reg.register(make_entry("prod")).unwrap();
            reg.activate("prod").unwrap();
        }

        let reg = VaultRegistry::new(dir.path()).unwrap();
        assert_eq!(reg.count(), 1);
        assert_eq!(reg.active_name(), Some("prod"));
    }

    #[test]
    fn test_list_active_flag() {
        let dir = tempdir().unwrap();
        let mut reg = VaultRegistry::new(dir.path()).unwrap();

        reg.register(make_entry("a")).unwrap();
        reg.register(make_entry("b")).unwrap();
        reg.activate("b").unwrap();

        let list = reg.list();
        let active: Vec<_> = list.iter().filter(|v| v.is_active).collect();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "b");
    }
}
