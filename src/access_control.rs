//! Access control — role-based permissions for multi-user vaults.
//!
//! Policies are persisted in `acl.json` alongside the vault.  Each entry maps
//! a *principal* (username or group) to a [`Role`].  The [`AclGuard`] can be
//! queried before any vault operation to enforce the policy.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Result, VaultError};

// ── Types ────────────────────────────────────────────────────────────────────

/// Role-based permission levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Can list models and read data.
    Reader,
    /// Reader + store, tag, annotate.
    Writer,
    /// Writer + delete, change passphrase, manage ACLs.
    Admin,
}

impl Role {
    pub fn can_read(self) -> bool {
        true
    }
    pub fn can_write(self) -> bool {
        matches!(self, Role::Writer | Role::Admin)
    }
    pub fn can_admin(self) -> bool {
        matches!(self, Role::Admin)
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Reader => write!(f, "reader"),
            Role::Writer => write!(f, "writer"),
            Role::Admin => write!(f, "admin"),
        }
    }
}

impl std::str::FromStr for Role {
    type Err = VaultError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "reader" | "read" | "ro" => Ok(Role::Reader),
            "writer" | "write" | "rw" => Ok(Role::Writer),
            "admin" | "administrator" => Ok(Role::Admin),
            _ => Err(VaultError::InvalidInput(format!("Unknown role: {s}"))),
        }
    }
}

/// Access-control list entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclEntry {
    pub principal: String,
    pub role: Role,
}

/// Persisted ACL file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AclData {
    pub entries: Vec<AclEntry>,
}

// ── Guard ────────────────────────────────────────────────────────────────────

/// Enforces access-control policies for a vault.
pub struct AclGuard {
    path: PathBuf,
    data: AclData,
}

impl AclGuard {
    const FILE_NAME: &'static str = "acl.json";

    /// Load or create ACL data for a vault.
    pub fn new(vault_path: &Path) -> Result<Self> {
        let path = vault_path.join(Self::FILE_NAME);
        let data = if path.exists() {
            let contents = fs::read_to_string(&path)?;
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            AclData::default()
        };
        Ok(Self { path, data })
    }

    fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.path, json)?;
        crate::permissions::restrict_file(&self.path)?;
        Ok(())
    }

    /// Grant a role to a principal (replaces existing role if any).
    pub fn grant(&mut self, principal: &str, role: Role) -> Result<()> {
        self.data.entries.retain(|e| e.principal != principal);
        self.data.entries.push(AclEntry {
            principal: principal.to_string(),
            role,
        });
        self.save()
    }

    /// Revoke all roles from a principal.
    pub fn revoke(&mut self, principal: &str) -> Result<bool> {
        let before = self.data.entries.len();
        self.data.entries.retain(|e| e.principal != principal);
        if self.data.entries.len() < before {
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Resolve the effective role for a principal.
    ///
    /// Returns `None` if no policy covers this principal — callers should
    /// decide the default (deny or allow).
    pub fn resolve(&self, principal: &str) -> Option<Role> {
        self.data
            .entries
            .iter()
            .find(|e| e.principal == principal)
            .map(|e| e.role)
    }

    /// List all entries.
    pub fn list(&self) -> &[AclEntry] {
        &self.data.entries
    }

    /// Check if a principal has the given minimum role, returning a nice error
    /// when denied.
    pub fn require(&self, principal: &str, min_role: Role) -> Result<()> {
        match self.resolve(principal) {
            Some(role) if role >= min_role => Ok(()),
            Some(role) => Err(VaultError::SecurityViolation(format!(
                "Principal '{}' has role '{}', required '{}'",
                principal, role, min_role
            ))),
            None => Err(VaultError::SecurityViolation(format!(
                "No ACL entry for principal '{}'",
                principal
            ))),
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_ordering() {
        assert!(Role::Reader < Role::Writer);
        assert!(Role::Writer < Role::Admin);
    }

    #[test]
    fn test_role_parse() {
        assert_eq!("reader".parse::<Role>().unwrap(), Role::Reader);
        assert_eq!("write".parse::<Role>().unwrap(), Role::Writer);
        assert_eq!("admin".parse::<Role>().unwrap(), Role::Admin);
        assert!("unknown".parse::<Role>().is_err());
    }

    #[test]
    fn test_grant_and_resolve() {
        let dir = tempfile::tempdir().unwrap();
        let mut acl = AclGuard::new(dir.path()).unwrap();

        acl.grant("alice", Role::Admin).unwrap();
        acl.grant("bob", Role::Reader).unwrap();

        assert_eq!(acl.resolve("alice"), Some(Role::Admin));
        assert_eq!(acl.resolve("bob"), Some(Role::Reader));
        assert_eq!(acl.resolve("charlie"), None);
    }

    #[test]
    fn test_revoke() {
        let dir = tempfile::tempdir().unwrap();
        let mut acl = AclGuard::new(dir.path()).unwrap();

        acl.grant("alice", Role::Writer).unwrap();
        assert!(acl.revoke("alice").unwrap());
        assert!(!acl.revoke("alice").unwrap()); // already gone
        assert_eq!(acl.resolve("alice"), None);
    }

    #[test]
    fn test_require() {
        let dir = tempfile::tempdir().unwrap();
        let mut acl = AclGuard::new(dir.path()).unwrap();

        acl.grant("alice", Role::Writer).unwrap();

        assert!(acl.require("alice", Role::Reader).is_ok());
        assert!(acl.require("alice", Role::Writer).is_ok());
        assert!(acl.require("alice", Role::Admin).is_err());
        assert!(acl.require("nobody", Role::Reader).is_err());
    }

    #[test]
    fn test_persistence() {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut acl = AclGuard::new(dir.path()).unwrap();
            acl.grant("alice", Role::Admin).unwrap();
        }
        let acl2 = AclGuard::new(dir.path()).unwrap();
        assert_eq!(acl2.resolve("alice"), Some(Role::Admin));
    }

    #[test]
    fn test_grant_replaces() {
        let dir = tempfile::tempdir().unwrap();
        let mut acl = AclGuard::new(dir.path()).unwrap();

        acl.grant("bob", Role::Reader).unwrap();
        acl.grant("bob", Role::Admin).unwrap();
        assert_eq!(acl.resolve("bob"), Some(Role::Admin));
        assert_eq!(acl.list().len(), 1);
    }
}
