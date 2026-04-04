//! Configuration profiles — named sets of config overrides.
//!
//! Profiles let users switch between different vault configurations
//! (e.g. "dev", "staging", "production") without editing the main config
//! file.  Profiles are stored in the XDG config directory.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Result, VaultError};

// ── Types ────────────────────────────────────────────────────────────────────

/// A configuration profile — key-value overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Profile name.
    pub name: String,
    /// Human-readable description.
    pub description: Option<String>,
    /// Flat key-value overrides (e.g. "vault.default_vault" → "staging").
    pub overrides: BTreeMap<String, String>,
    /// When this profile was created.
    pub created_at: String,
}

/// Persisted profiles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileData {
    /// Active profile name (None = no profile active).
    pub active: Option<String>,
    /// Named profiles.
    pub profiles: BTreeMap<String, Profile>,
}

// ── Store ────────────────────────────────────────────────────────────────────

/// Manages configuration profiles.
pub struct ProfileStore {
    path: PathBuf,
    data: ProfileData,
}

impl ProfileStore {
    const FILE_NAME: &'static str = "profiles.json";

    /// Open or create a profile store in the given config directory.
    pub fn new(config_dir: &Path) -> Result<Self> {
        let path = config_dir.join(Self::FILE_NAME);
        let data = if path.exists() {
            let contents = fs::read_to_string(&path)?;
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            ProfileData::default()
        };
        Ok(Self { path, data })
    }

    fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.path, json)?;
        crate::permissions::restrict_file(&self.path)?;
        Ok(())
    }

    /// Create or update a profile.
    pub fn set(&mut self, profile: Profile) -> Result<()> {
        self.data.profiles.insert(profile.name.clone(), profile);
        self.save()
    }

    /// Remove a profile by name.
    pub fn remove(&mut self, name: &str) -> Result<bool> {
        if self.data.profiles.remove(name).is_some() {
            // If the removed profile was active, clear it
            if self.data.active.as_deref() == Some(name) {
                self.data.active = None;
            }
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get a profile by name.
    pub fn get(&self, name: &str) -> Option<&Profile> {
        self.data.profiles.get(name)
    }

    /// List all profiles.
    pub fn list(&self) -> Vec<&Profile> {
        self.data.profiles.values().collect()
    }

    /// Activate a profile (it will be used for subsequent operations).
    pub fn activate(&mut self, name: &str) -> Result<()> {
        if !self.data.profiles.contains_key(name) {
            return Err(VaultError::InvalidInput(format!(
                "Profile '{}' not found",
                name
            )));
        }
        self.data.active = Some(name.to_string());
        self.save()
    }

    /// Deactivate the current profile.
    pub fn deactivate(&mut self) -> Result<()> {
        self.data.active = None;
        self.save()
    }

    /// Get the active profile (if any).
    pub fn active(&self) -> Option<&Profile> {
        self.data
            .active
            .as_deref()
            .and_then(|name| self.data.profiles.get(name))
    }

    /// Get the active profile name.
    pub fn active_name(&self) -> Option<&str> {
        self.data.active.as_deref()
    }

    /// Display profiles summary.
    pub fn display(&self) -> String {
        let mut out = String::new();
        if self.data.profiles.is_empty() {
            out.push_str("No profiles configured.\n");
            return out;
        }

        out.push_str(&format!(
            "{:<20} {:<8} {}\n",
            "Name", "Active", "Description"
        ));
        out.push_str(&format!("{}\n", "─".repeat(50)));

        for profile in self.data.profiles.values() {
            let active = if self.data.active.as_deref() == Some(&profile.name) {
                "  *"
            } else {
                ""
            };
            out.push_str(&format!(
                "{:<20} {:<8} {}\n",
                profile.name,
                active,
                profile.description.as_deref().unwrap_or("")
            ));
        }
        out
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_profile(name: &str) -> Profile {
        Profile {
            name: name.to_string(),
            description: Some(format!("{} profile", name)),
            overrides: {
                let mut m = BTreeMap::new();
                m.insert("vault.default_vault".into(), name.into());
                m
            },
            created_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn test_create_and_get() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = ProfileStore::new(dir.path()).unwrap();

        store.set(make_profile("dev")).unwrap();
        assert!(store.get("dev").is_some());
        assert_eq!(store.get("dev").unwrap().name, "dev");
    }

    #[test]
    fn test_remove() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = ProfileStore::new(dir.path()).unwrap();

        store.set(make_profile("dev")).unwrap();
        assert!(store.remove("dev").unwrap());
        assert!(!store.remove("dev").unwrap());
    }

    #[test]
    fn test_activate_deactivate() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = ProfileStore::new(dir.path()).unwrap();

        store.set(make_profile("prod")).unwrap();
        store.activate("prod").unwrap();
        assert_eq!(store.active_name(), Some("prod"));

        store.deactivate().unwrap();
        assert!(store.active_name().is_none());
    }

    #[test]
    fn test_activate_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = ProfileStore::new(dir.path()).unwrap();
        assert!(store.activate("nope").is_err());
    }

    #[test]
    fn test_persistence() {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut store = ProfileStore::new(dir.path()).unwrap();
            store.set(make_profile("staging")).unwrap();
            store.activate("staging").unwrap();
        }
        let store2 = ProfileStore::new(dir.path()).unwrap();
        assert_eq!(store2.active_name(), Some("staging"));
        assert!(store2.get("staging").is_some());
    }

    #[test]
    fn test_list() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = ProfileStore::new(dir.path()).unwrap();

        store.set(make_profile("dev")).unwrap();
        store.set(make_profile("prod")).unwrap();
        assert_eq!(store.list().len(), 2);
    }

    #[test]
    fn test_remove_active_clears() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = ProfileStore::new(dir.path()).unwrap();

        store.set(make_profile("dev")).unwrap();
        store.activate("dev").unwrap();
        store.remove("dev").unwrap();
        assert!(store.active_name().is_none());
    }
}
