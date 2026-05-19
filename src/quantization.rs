//! Model quantization pipeline.
//!
//! Provides structured quantization profiles, batch quantization, and
//! integration with the conversion system.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::{Result, VaultError};

// ── Types ────────────────────────────────────────────────────────────────────

/// Supported quantization methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuantMethod {
    /// 4-bit quantization (fastest inference, lowest quality).
    Q4_0,
    /// 4-bit K-quant medium (good balance).
    Q4KM,
    /// 5-bit K-quant medium.
    Q5KM,
    /// 8-bit quantization (highest quality quantized).
    Q8_0,
    /// 16-bit float (half precision).
    F16,
    /// 32-bit float (full precision, no quantization).
    F32,
}

impl QuantMethod {
    /// Approximate bits-per-weight for this method.
    pub fn bits_per_weight(self) -> f64 {
        match self {
            Self::Q4_0 => 4.0,
            Self::Q4KM => 4.5,
            Self::Q5KM => 5.5,
            Self::Q8_0 => 8.0,
            Self::F16 => 16.0,
            Self::F32 => 32.0,
        }
    }

    /// Returns the string used in conversion options.
    pub fn as_conversion_str(&self) -> &'static str {
        match self {
            Self::Q4_0 => "q4_0",
            Self::Q4KM => "q4_k_m",
            Self::Q5KM => "q5_k_m",
            Self::Q8_0 => "q8_0",
            Self::F16 => "f16",
            Self::F32 => "f32",
        }
    }
}

impl std::fmt::Display for QuantMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_conversion_str())
    }
}

impl std::str::FromStr for QuantMethod {
    type Err = VaultError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "q4_0" | "q4" => Ok(Self::Q4_0),
            "q4_k_m" | "q4km" | "q4_k" => Ok(Self::Q4KM),
            "q5_k_m" | "q5km" | "q5_k" => Ok(Self::Q5KM),
            "q8_0" | "q8" => Ok(Self::Q8_0),
            "f16" | "fp16" | "half" => Ok(Self::F16),
            "f32" | "fp32" | "float" | "full" => Ok(Self::F32),
            _ => Err(VaultError::InvalidInput(format!(
                "Unknown quantization method: {s}"
            ))),
        }
    }
}

/// A named quantization profile with target method and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantProfile {
    /// Profile name (e.g. "edge-deploy", "server-quality").
    pub name: String,
    /// Target quantization method.
    pub method: QuantMethod,
    /// Optional description.
    pub description: Option<String>,
    /// Extra key-value metadata.
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

/// Result of a quantization operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantResult {
    /// Source file path.
    pub source: PathBuf,
    /// Output file path.
    pub output: PathBuf,
    /// Method used.
    pub method: QuantMethod,
    /// Original size in bytes.
    pub original_bytes: u64,
    /// Quantized size in bytes.
    pub quantized_bytes: u64,
    /// Compression ratio (original / quantized).
    pub compression_ratio: f64,
}

/// Batch quantization report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatchQuantReport {
    pub results: Vec<QuantResult>,
    pub failed: Vec<(PathBuf, String)>,
}

// ── Profile Store ────────────────────────────────────────────────────────────

const PROFILES_FILE: &str = "quant_profiles.json";

/// Manages named quantization profiles on disk.
#[derive(Debug)]
pub struct QuantProfileStore {
    path: PathBuf,
    profiles: BTreeMap<String, QuantProfile>,
}

impl QuantProfileStore {
    /// Load or create profile store.
    pub fn new(vault_path: &Path) -> Result<Self> {
        let path = vault_path.join(PROFILES_FILE);
        let profiles = if path.exists() {
            let data = std::fs::read_to_string(&path)
                .map_err(|e| VaultError::StorageError(format!("read quant profiles: {e}")))?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            BTreeMap::new()
        };
        Ok(Self { path, profiles })
    }

    fn save(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(&self.profiles)
            .map_err(|e| VaultError::StorageError(format!("serialize quant profiles: {e}")))?;
        std::fs::write(&self.path, data)
            .map_err(|e| VaultError::StorageError(format!("write quant profiles: {e}")))
    }

    /// Add or update a profile.
    pub fn set(&mut self, profile: QuantProfile) -> Result<()> {
        self.profiles.insert(profile.name.clone(), profile);
        self.save()
    }

    /// Remove a profile by name.
    pub fn remove(&mut self, name: &str) -> Result<bool> {
        let removed = self.profiles.remove(name).is_some();
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    /// Get a profile by name.
    pub fn get(&self, name: &str) -> Option<&QuantProfile> {
        self.profiles.get(name)
    }

    /// List all profiles.
    pub fn list(&self) -> Vec<&QuantProfile> {
        self.profiles.values().collect()
    }
}

// ── Estimation ───────────────────────────────────────────────────────────────

/// Estimate the output size after quantization.
pub fn estimate_quantized_size(original_bytes: u64, from: QuantMethod, to: QuantMethod) -> u64 {
    let ratio = to.bits_per_weight() / from.bits_per_weight();
    (original_bytes as f64 * ratio).ceil() as u64
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_quant_method_roundtrip() {
        for method in [
            QuantMethod::Q4_0,
            QuantMethod::Q4KM,
            QuantMethod::Q5KM,
            QuantMethod::Q8_0,
            QuantMethod::F16,
            QuantMethod::F32,
        ] {
            let s = method.to_string();
            let parsed: QuantMethod = s.parse().unwrap();
            assert_eq!(parsed, method);
        }
    }

    #[test]
    fn test_quant_method_parse_aliases() {
        assert_eq!("q4".parse::<QuantMethod>().unwrap(), QuantMethod::Q4_0);
        assert_eq!("fp16".parse::<QuantMethod>().unwrap(), QuantMethod::F16);
        assert_eq!("half".parse::<QuantMethod>().unwrap(), QuantMethod::F16);
        assert_eq!("q4_k".parse::<QuantMethod>().unwrap(), QuantMethod::Q4KM);
        assert_eq!("q8".parse::<QuantMethod>().unwrap(), QuantMethod::Q8_0);
    }

    #[test]
    fn test_quant_method_parse_invalid() {
        assert!("q3".parse::<QuantMethod>().is_err());
    }

    #[test]
    fn test_bits_per_weight() {
        assert!((QuantMethod::Q4_0.bits_per_weight() - 4.0).abs() < f64::EPSILON);
        assert!((QuantMethod::F32.bits_per_weight() - 32.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_estimate_quantized_size() {
        // F32 → Q4_0 should be ~8× smaller
        let est = estimate_quantized_size(1_000_000, QuantMethod::F32, QuantMethod::Q4_0);
        assert_eq!(est, 125000);
    }

    #[test]
    fn test_profile_store_crud() {
        let dir = tempdir().unwrap();
        let mut store = QuantProfileStore::new(dir.path()).unwrap();

        let profile = QuantProfile {
            name: "edge".into(),
            method: QuantMethod::Q4KM,
            description: Some("Edge deployment".into()),
            metadata: BTreeMap::new(),
        };
        store.set(profile).unwrap();
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.get("edge").unwrap().method, QuantMethod::Q4KM);

        store.remove("edge").unwrap();
        assert!(store.list().is_empty());
    }

    #[test]
    fn test_profile_store_persistence() {
        let dir = tempdir().unwrap();

        {
            let mut store = QuantProfileStore::new(dir.path()).unwrap();
            store
                .set(QuantProfile {
                    name: "prod".into(),
                    method: QuantMethod::Q8_0,
                    description: None,
                    metadata: BTreeMap::new(),
                })
                .unwrap();
        }

        // Re-open
        let store = QuantProfileStore::new(dir.path()).unwrap();
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.get("prod").unwrap().method, QuantMethod::Q8_0);
    }

    #[test]
    fn test_batch_report_default() {
        let report = BatchQuantReport::default();
        assert!(report.results.is_empty());
        assert!(report.failed.is_empty());
    }
}
