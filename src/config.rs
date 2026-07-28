//! XDG Base Directory Specification compliant configuration
//!
//! Cross-platform support for Linux, macOS, and Windows following XDG standards.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::crypto::compression::{CompressionAlgorithm, CompressionLevel};
use crate::error::{Result, VaultError};

/// AI Model Vault (AIMV) Configuration
///
/// Directory structure:
/// - Config: ~/.config/ai/models/ (or platform equivalent)
/// - Data: ~/.local/share/ai/models/ (or platform equivalent)
/// - Cache: ~/.cache/ai/models/ (or platform equivalent)
/// - Backends: ~/.config/ai/backends/ (cloud storage configs)
/// - Utilities: ~/.config/ai/utilities/ (utility configs)
/// - Databases: ~/.config/ai/databases/ (knowledge bases, training data)
///
/// Compliance:
/// - XDG Base Directory Specification
/// - CMMC AC.3.014: Separate duties of individuals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    /// Version of configuration format
    pub version: String,

    /// Vault settings
    pub vault: VaultSettings,

    /// Cryptographic settings
    pub crypto: CryptoSettings,

    /// Compression settings
    pub compression: CompressionSettings,

    /// Storage settings
    pub storage: StorageSettings,

    /// Security settings
    pub security: SecuritySettings,

    /// Compliance settings
    pub compliance: ComplianceSettings,

    /// Telemetry settings
    #[serde(default)]
    pub telemetry: TelemetrySettings,

    /// Directory paths (not serialized, computed at runtime)
    #[serde(skip)]
    pub dirs: DirectoryPaths,
}

/// Default vault selection settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSettings {
    pub default_vault: String,
}

/// Cryptographic algorithm and key derivation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoSettings {
    pub algorithm: String,
    pub kdf: String,
}

/// Compression algorithm and level settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionSettings {
    pub algorithm: String,
    pub level: u8,
}

/// Storage backend behavior settings (versioning, cleanup).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSettings {
    pub max_versions: u32,
    pub auto_cleanup: bool,
    pub checkpoint_format: String,
    /// Models larger than this threshold (in bytes) use chunked streaming
    /// encryption instead of monolithic encryption. Default: 16 MiB.
    /// Set to 0 to always use streaming, or `u64::MAX` to disable.
    #[serde(default = "default_streaming_threshold")]
    pub streaming_threshold: u64,
}

/// Default streaming threshold: 16 MiB.
fn default_streaming_threshold() -> u64 {
    16 * 1024 * 1024
}

/// Security policy settings (passphrase, session timeout, audit).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub require_passphrase: bool,
    pub session_timeout_seconds: u64,
    pub audit_log: bool,
}

/// Compliance and regulatory settings (FIPS mode, CVE scanning).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSettings {
    pub fips_mode: bool,
    pub cve_scanning: bool,
    pub audit_retention_days: u32,
}

/// Telemetry and analytics settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySettings {
    /// Whether telemetry is enabled (default: true)
    pub enabled: bool,
    /// Anonymous device ID (auto-generated)
    #[serde(default = "default_device_id")]
    pub device_id: String,
}

fn default_device_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

impl Default for TelemetrySettings {
    fn default() -> Self {
        Self {
            enabled: true,
            device_id: default_device_id(),
        }
    }
}

/// Relocates every config/data/cache directory under one root.
pub const ENV_HOME: &str = "aimodelvault_HOME";
/// Overrides the config directory (holds `config.yaml`, profiles, plugins).
pub const ENV_CONFIG: &str = "aimodelvault_CONFIG";
/// Overrides the default vault name.
pub const ENV_VAULT: &str = "aimodelvault_VAULT";

/// Serialises directory creation and permission tightening within the process.
static INIT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Create a directory, tolerating a concurrent creator.
///
/// Two processes (or two threads) initialising the vault at once will race:
/// one may be rewriting a parent's ACL while the other creates a child. A
/// single retry covers that window; `AlreadyExists` is always success.
fn create_dir_resilient(dir: &std::path::Path) -> Result<()> {
    match fs::create_dir_all(dir) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(_) if dir.is_dir() => Ok(()),
        Err(first) => match fs::create_dir_all(dir) {
            Ok(()) => Ok(()),
            Err(_) if dir.is_dir() => Ok(()),
            Err(_) => Err(VaultError::IoError(first)),
        },
    }
}

/// Read an environment variable, treating empty/whitespace values as unset.
fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// XDG-compliant directory paths for config, data, cache, and logs.
#[derive(Debug, Clone, Default)]
pub struct DirectoryPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub vault_dir: PathBuf,
    pub log_dir: PathBuf,
    pub backends_dir: PathBuf,
    pub utilities_dir: PathBuf,
    pub databases_dir: PathBuf,
}

impl VaultConfig {
    /// Create new configuration with defaults
    ///
    /// Honors three environment overrides:
    /// - `aimodelvault_HOME` — relocate all config/data/cache directories
    /// - `aimodelvault_CONFIG` — path to the config file to load
    /// - `aimodelvault_VAULT` — default vault name
    pub fn new() -> Result<Self> {
        let dirs = Self::get_project_dirs()?;
        Self::ensure_directories(&dirs)?;

        let config_file = dirs.config_dir.join("config.yaml");

        let mut config = if config_file.exists() {
            Self::load_from_file(&config_file, dirs)?
        } else {
            let config = Self::default_with_dirs(dirs);
            config.save()?;
            config
        };

        if let Some(name) = non_empty_env(ENV_VAULT) {
            config.vault.default_vault = name;
        }

        Ok(config)
    }

    /// Create configuration with custom directory paths
    pub fn with_dirs(dirs: DirectoryPaths) -> Result<Self> {
        Self::ensure_directories(&dirs)?;
        Ok(Self::default_with_dirs(dirs))
    }

    /// Get XDG project directories for AI Model Vault (AIMV)
    ///
    /// Uses shorter, organized paths:
    /// - ~/.config/ai/models/
    /// - ~/.local/share/ai/models/
    /// - ~/.cache/ai/models/
    /// - ~/.config/ai/backends/
    /// - ~/.config/ai/utilities/
    /// - ~/.config/ai/databases/
    fn get_project_dirs() -> Result<DirectoryPaths> {
        let mut dirs = Self::platform_dirs()?;

        // `aimodelvault_CONFIG` relocates just the config tree.
        if let Some(config_root) = non_empty_env(ENV_CONFIG) {
            let config_dir = PathBuf::from(config_root);
            dirs.backends_dir = config_dir.join("backends");
            dirs.utilities_dir = config_dir.join("utilities");
            dirs.databases_dir = config_dir.join("databases");
            dirs.config_dir = config_dir;
        }

        Ok(dirs)
    }

    /// Directory layout before environment overrides are applied.
    fn platform_dirs() -> Result<DirectoryPaths> {
        use directories::BaseDirs;

        // `aimodelvault_HOME` relocates every directory under one root. Used for
        // test isolation, containers, and per-project vaults.
        if let Some(root) = non_empty_env(ENV_HOME) {
            let root = PathBuf::from(root);
            let config_dir = root.join("config");
            let data_dir = root.join("data");
            return Ok(DirectoryPaths {
                cache_dir: root.join("cache"),
                vault_dir: data_dir.join("vaults"),
                log_dir: data_dir.join("logs"),
                backends_dir: config_dir.join("backends"),
                utilities_dir: config_dir.join("utilities"),
                databases_dir: config_dir.join("databases"),
                config_dir,
                data_dir,
            });
        }

        let base_dirs = BaseDirs::new().ok_or_else(|| {
            VaultError::ConfigError("Failed to determine base directories".to_string())
        })?;

        // Use shorter paths under ~/.config/ai/, ~/.local/share/ai/, etc.
        let config_base = base_dirs.config_dir().join("ai");
        let data_base = base_dirs.data_dir().join("ai");
        let cache_base = base_dirs.cache_dir().join("ai");

        let config_dir = config_base.join("models");
        let data_dir = data_base.join("models");
        let cache_dir = cache_base.join("models");
        let vault_dir = data_dir.join("vaults");
        let log_dir = data_dir.join("logs");
        let backends_dir = config_base.join("backends");
        let utilities_dir = config_base.join("utilities");
        let databases_dir = config_base.join("databases");

        Ok(DirectoryPaths {
            config_dir,
            data_dir,
            cache_dir,
            vault_dir,
            log_dir,
            backends_dir,
            utilities_dir,
            databases_dir,
        })
    }

    /// Ensure all required directories exist with secure permissions
    fn ensure_directories(dirs: &DirectoryPaths) -> Result<()> {
        let all = [
            &dirs.config_dir,
            &dirs.data_dir,
            &dirs.cache_dir,
            &dirs.vault_dir,
            &dirs.log_dir,
            &dirs.backends_dir,
            &dirs.utilities_dir,
            &dirs.databases_dir,
        ];

        // Serialise first-run setup. Several callers in one process (CLI
        // handlers, the API server's workers, the test harness) can initialise
        // the same directories at once, and on Windows `icacls /inheritance:r`
        // briefly leaves a directory without a usable DACL — a concurrent
        // create or write in that window fails with "Access is denied".
        let _guard = INIT_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        // Restrict each directory immediately after creating it, parents before
        // children: `vault_dir` and `log_dir` live under `data_dir`, and
        // tightening a parent's ACL once a child already exists makes `icacls`
        // fail on the child.
        for dir in all {
            if !dir.is_dir() {
                // A separate process may still be mid-setup.
                create_dir_resilient(dir)?;
                crate::permissions::restrict_dir(dir)?;
            }
        }

        Ok(())
    }

    /// Load configuration from file
    fn load_from_file(path: &PathBuf, dirs: DirectoryPaths) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let mut config: VaultConfig = serde_yaml_ng::from_str(&contents)?;
        config.dirs = dirs;
        Ok(config)
    }

    /// Create default configuration with directories
    fn default_with_dirs(dirs: DirectoryPaths) -> Self {
        Self {
            version: "1.0".to_string(),
            vault: VaultSettings {
                default_vault: "default".to_string(),
            },
            crypto: CryptoSettings {
                algorithm: "aes-256-gcm".to_string(),
                kdf: "argon2id".to_string(),
            },
            compression: CompressionSettings {
                algorithm: "gzip".to_string(),
                level: 6,
            },
            storage: StorageSettings {
                max_versions: 10,
                auto_cleanup: true,
                checkpoint_format: "v{version}_{timestamp}".to_string(),
                streaming_threshold: default_streaming_threshold(),
            },
            security: SecuritySettings {
                require_passphrase: true,
                session_timeout_seconds: 3600,
                audit_log: true,
            },
            compliance: ComplianceSettings {
                fips_mode: true,
                cve_scanning: true,
                audit_retention_days: 90,
            },
            telemetry: TelemetrySettings::default(),
            dirs,
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_file = self.dirs.config_dir.join("config.yaml");
        let contents = serde_yaml_ng::to_string(self)?;
        fs::write(&config_file, contents)?;
        crate::permissions::restrict_file(&config_file)?;

        Ok(())
    }

    /// Get path to specific vault
    pub fn get_vault_path(&self, vault_name: Option<&str>) -> PathBuf {
        let name = vault_name.unwrap_or(&self.vault.default_vault);
        self.dirs.vault_dir.join(name)
    }

    /// Get audit log path
    pub fn get_audit_log_path(&self) -> PathBuf {
        self.dirs.log_dir.join("audit.log")
    }

    /// Get compression algorithm
    pub fn get_compression_algorithm(&self) -> CompressionAlgorithm {
        match self.compression.algorithm.as_str() {
            "gzip" => CompressionAlgorithm::Gzip,
            "lzma" => CompressionAlgorithm::Lzma,
            "none" => CompressionAlgorithm::None,
            _ => CompressionAlgorithm::Gzip,
        }
    }

    /// Get compression level
    pub fn get_compression_level(&self) -> CompressionLevel {
        match self.compression.level {
            0 => CompressionLevel::None,
            1 => CompressionLevel::Fast,
            9 => CompressionLevel::Maximum,
            _ => CompressionLevel::Balanced,
        }
    }
}

/// Note: `VaultConfig::default()` panics if the home directory cannot be determined.
/// Prefer `VaultConfig::new()` which returns `Result` for fallible creation.
impl Default for VaultConfig {
    fn default() -> Self {
        Self::new().expect("Failed to create default configuration: home directory unavailable")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_settings_default() {
        // Covers line 126 — TelemetrySettings::default()
        let ts = TelemetrySettings::default();
        assert!(ts.enabled);
        assert!(!ts.device_id.is_empty());
    }

    #[test]
    fn test_vault_config_with_dirs() {
        // Covers line 165 — VaultConfig::with_dirs()
        let temp = tempfile::tempdir().unwrap();
        let dirs = DirectoryPaths {
            config_dir: temp.path().join("config"),
            data_dir: temp.path().join("data"),
            cache_dir: temp.path().join("cache"),
            vault_dir: temp.path().join("vaults"),
            log_dir: temp.path().join("logs"),
            backends_dir: temp.path().join("backends"),
            utilities_dir: temp.path().join("utils"),
            databases_dir: temp.path().join("dbs"),
        };
        let config = VaultConfig::with_dirs(dirs).unwrap();
        assert!(config.dirs.config_dir.starts_with(temp.path()));
    }

    #[test]
    fn test_vault_config_new() {
        // Covers lines 155, 158, 159, 160 — VaultConfig::new() both branches
        let config = VaultConfig::new().unwrap();
        assert!(!config.dirs.config_dir.as_os_str().is_empty());
    }

    #[test]
    fn test_compression_level_settings() {
        let mut config = VaultConfig::new().unwrap();
        config.compression.level = 0;
        assert!(matches!(
            config.get_compression_level(),
            CompressionLevel::None
        ));
        config.compression.level = 1;
        assert!(matches!(
            config.get_compression_level(),
            CompressionLevel::Fast
        ));
        config.compression.level = 9;
        assert!(matches!(
            config.get_compression_level(),
            CompressionLevel::Maximum
        ));
        config.compression.level = 5;
        assert!(matches!(
            config.get_compression_level(),
            CompressionLevel::Balanced
        ));
    }

    #[test]
    fn test_compression_algorithm_variants() {
        let mut config = VaultConfig::new().unwrap();
        config.compression.algorithm = "gzip".to_string();
        assert!(matches!(
            config.get_compression_algorithm(),
            CompressionAlgorithm::Gzip
        ));
        config.compression.algorithm = "lzma".to_string();
        assert!(matches!(
            config.get_compression_algorithm(),
            CompressionAlgorithm::Lzma
        ));
        config.compression.algorithm = "none".to_string();
        assert!(matches!(
            config.get_compression_algorithm(),
            CompressionAlgorithm::None
        ));
        config.compression.algorithm = "unknown_algo".to_string();
        assert!(matches!(
            config.get_compression_algorithm(),
            CompressionAlgorithm::Gzip
        ));
    }

    #[test]
    fn test_vault_config_save_and_reload() {
        let temp = tempfile::tempdir().unwrap();
        let dirs = DirectoryPaths {
            config_dir: temp.path().join("config"),
            data_dir: temp.path().join("data"),
            cache_dir: temp.path().join("cache"),
            vault_dir: temp.path().join("vaults"),
            log_dir: temp.path().join("logs"),
            backends_dir: temp.path().join("backends"),
            utilities_dir: temp.path().join("utils"),
            databases_dir: temp.path().join("dbs"),
        };
        let config = VaultConfig::with_dirs(dirs.clone()).unwrap();
        config.save().unwrap();

        // Now load_from_file is exercised
        let config_file = dirs.config_dir.join("config.yaml");
        assert!(config_file.exists());
        let reloaded = VaultConfig::load_from_file(&config_file, dirs).unwrap();
        assert_eq!(reloaded.vault.default_vault, "default");
        assert_eq!(reloaded.crypto.algorithm, "aes-256-gcm");
    }

    #[test]
    fn test_vault_path_and_audit_log_path() {
        let config = VaultConfig::new().unwrap();
        let vault_path = config.get_vault_path(None);
        assert!(vault_path.ends_with("default"));

        let custom_path = config.get_vault_path(Some("my-vault"));
        assert!(custom_path.ends_with("my-vault"));

        let audit_path = config.get_audit_log_path();
        assert!(audit_path.ends_with("audit.log"));
    }

    #[test]
    fn test_vault_config_default_impl() {
        // Covers L335-336 — Default for VaultConfig
        let config = VaultConfig::default();
        assert_eq!(config.vault.default_vault, "default");
        assert_eq!(config.crypto.algorithm, "aes-256-gcm");
    }
}
