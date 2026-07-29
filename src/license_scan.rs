//! License scanning — detect model licenses from metadata and model cards.
//!
//! Scans HuggingFace model card YAML frontmatter, GGUF metadata fields,
//! and common LICENSE/README files for license information.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

// ── Types ────────────────────────────────────────────────────────────────────

/// A detected license.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedLicense {
    /// SPDX identifier if recognized (e.g. "Apache-2.0", "MIT")
    pub spdx_id: Option<String>,
    /// Raw license string as found in the source
    pub raw: String,
    /// Where the license was found
    pub source: LicenseSource,
    /// Permissiveness classification
    pub classification: LicenseClass,
}

/// Where a license was detected from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseSource {
    /// YAML frontmatter in a model card / README
    ModelCard,
    /// GGUF metadata header
    GgufMetadata,
    /// A LICENSE or LICENSE.md file
    LicenseFile,
    /// config.json or similar
    ConfigFile,
    /// User-specified
    Manual,
}

/// Classification of a license by permissiveness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseClass {
    /// Highly permissive (MIT, Apache-2.0, BSD)
    Permissive,
    /// Copyleft / share-alike (GPL, AGPL, CC-BY-SA)
    Copyleft,
    /// Non-commercial or restricted use
    Restricted,
    /// Custom / proprietary license
    Proprietary,
    /// Could not classify
    Unknown,
}

impl std::fmt::Display for LicenseClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Permissive => write!(f, "Permissive"),
            Self::Copyleft => write!(f, "Copyleft"),
            Self::Restricted => write!(f, "Restricted"),
            Self::Proprietary => write!(f, "Proprietary"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Full license scan report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseScanReport {
    /// All detected licenses
    pub licenses: Vec<DetectedLicense>,
    /// Warnings (e.g. incompatible licenses, missing license info)
    pub warnings: Vec<String>,
    /// Whether any license was found at all
    pub has_license: bool,
}

// ── Scanner ──────────────────────────────────────────────────────────────────

/// Scans model files and directories for license information.
pub struct LicenseScanner;

impl LicenseScanner {
    /// Scan a directory for license information.
    ///
    /// Looks at README.md (YAML frontmatter), LICENSE files, config.json.
    pub fn scan_directory(dir: &Path) -> Result<LicenseScanReport> {
        let mut licenses = Vec::new();
        let mut warnings = Vec::new();

        // Check README.md / README for YAML frontmatter
        for readme_name in &["README.md", "readme.md", "README"] {
            let path = dir.join(readme_name);
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Some(lic) = Self::extract_yaml_license(&content) {
                        licenses.push(lic);
                        break; // avoid duplicates on case-insensitive filesystems
                    }
                }
            }
        }

        // Check LICENSE files
        for license_name in &[
            "LICENSE",
            "LICENSE.md",
            "LICENSE.txt",
            "license",
            "license.md",
            "license.txt",
        ] {
            let path = dir.join(license_name);
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Some(lic) = Self::detect_license_from_text(&content) {
                        licenses.push(lic);
                        break; // avoid duplicates on case-insensitive filesystems
                    }
                }
            }
        }

        // Check config.json for "license" field
        let config_path = dir.join("config.json");
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Some(lic) = Self::extract_config_license(&content) {
                    licenses.push(lic);
                }
            }
        }

        if licenses.is_empty() {
            warnings.push("No license information found".to_string());
        }

        // Check for potential incompatibilities
        Self::check_compatibility(&licenses, &mut warnings);

        let has_license = !licenses.is_empty();
        Ok(LicenseScanReport {
            licenses,
            warnings,
            has_license,
        })
    }

    /// Scan a single file for license info (e.g. GGUF metadata).
    pub fn scan_file(path: &Path) -> Result<LicenseScanReport> {
        let mut licenses = Vec::new();
        let mut warnings = Vec::new();

        let data = fs::read(path)?;

        // Check GGUF metadata
        if data.len() > 4 && &data[0..4] == b"GGUF" {
            if let Some(lic) = Self::extract_gguf_license(&data) {
                licenses.push(lic);
            }
        }

        // Check if it's a text file (README, LICENSE)
        if let Ok(text) = std::str::from_utf8(&data) {
            if let Some(lic) = Self::extract_yaml_license(text) {
                licenses.push(lic);
            }
            if licenses.is_empty() {
                if let Some(lic) = Self::detect_license_from_text(text) {
                    licenses.push(lic);
                }
            }
        }

        if licenses.is_empty() {
            warnings.push(format!(
                "No license information found in {}",
                path.display()
            ));
        }

        let has_license = !licenses.is_empty();
        Ok(LicenseScanReport {
            licenses,
            warnings,
            has_license,
        })
    }

    /// Scan model bytes from a vault (in-memory).
    pub fn scan_bytes(data: &[u8], name: &str) -> LicenseScanReport {
        let mut licenses = Vec::new();
        let mut warnings = Vec::new();

        // Check GGUF
        if data.len() > 4 && &data[0..4] == b"GGUF" {
            if let Some(lic) = Self::extract_gguf_license(data) {
                licenses.push(lic);
            }
        }

        if licenses.is_empty() {
            warnings.push(format!("No license information found in {}", name));
        }

        let has_license = !licenses.is_empty();
        LicenseScanReport {
            licenses,
            warnings,
            has_license,
        }
    }

    // ── Extractors ───────────────────────────────────────────────────────

    /// Extract license from YAML frontmatter (HuggingFace model cards).
    fn extract_yaml_license(text: &str) -> Option<DetectedLicense> {
        // Frontmatter is between --- markers
        if !text.starts_with("---") {
            return None;
        }
        let end = text[3..].find("---")?;
        let frontmatter = &text[3..3 + end];

        // Look for "license: <value>" line
        for line in frontmatter.lines() {
            let trimmed = line.trim();
            if let Some(value) = trimmed
                .strip_prefix("license:")
                .or_else(|| trimmed.strip_prefix("license :"))
            {
                let raw = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                if !raw.is_empty() {
                    let spdx = Self::normalize_spdx(&raw);
                    let classification = Self::classify_license(&raw);
                    return Some(DetectedLicense {
                        spdx_id: spdx,
                        raw,
                        source: LicenseSource::ModelCard,
                        classification,
                    });
                }
            }
        }
        None
    }

    /// Extract license from config.json.
    fn extract_config_license(json_text: &str) -> Option<DetectedLicense> {
        let parsed: BTreeMap<String, serde_json::Value> = serde_json::from_str(json_text).ok()?;

        let raw = parsed.get("license")?.as_str()?.to_string();
        let spdx = Self::normalize_spdx(&raw);
        let classification = Self::classify_license(&raw);

        Some(DetectedLicense {
            spdx_id: spdx,
            raw,
            source: LicenseSource::ConfigFile,
            classification,
        })
    }

    /// Extract license from GGUF metadata.
    ///
    /// Reads the value stored under the `general.license` key by walking the
    /// metadata block properly (see [`crate::gguf`]).
    ///
    /// This used to search the raw bytes for the literal text
    /// `"general.license"` and then scan the following 512 bytes for any entry
    /// in [`KNOWN_LICENSES`], returning the first that matched anywhere in that
    /// window. Two things made that unsound for a compliance signal. The window
    /// ran well past the value and into unrelated metadata, so a license name
    /// could be picked up from a description or a tokenizer entry; and the table is
    /// scanned in order with `"mit"` first, matched as a bare substring — so
    /// any window containing "li**mit**ations" or "per**mit**ted", which is
    /// ordinary license boilerplate, reported **MIT / permissive** no matter
    /// what the model was actually licensed under.
    fn extract_gguf_license(data: &[u8]) -> Option<DetectedLicense> {
        let raw = crate::gguf::metadata_string(data, "general.license")?;
        let raw = raw.trim().to_string();
        if raw.is_empty() {
            return None;
        }

        Some(DetectedLicense {
            spdx_id: Self::normalize_spdx(&raw),
            classification: Self::classify_license(&raw),
            raw,
            source: LicenseSource::GgufMetadata,
        })
    }

    /// Detect a license from the full text of a LICENSE file.
    fn detect_license_from_text(text: &str) -> Option<DetectedLicense> {
        let lower = text.to_lowercase();

        // Check for well-known license texts
        let detected = if lower.contains("apache license") && lower.contains("version 2.0") {
            ("Apache-2.0", LicenseClass::Permissive)
        } else if lower.contains("mit license")
            || lower.contains("permission is hereby granted, free of charge")
        {
            ("MIT", LicenseClass::Permissive)
        } else if lower.contains("gnu affero general public license") {
            ("AGPL-3.0", LicenseClass::Copyleft)
        } else if lower.contains("gnu general public license") && lower.contains("version 3") {
            ("GPL-3.0", LicenseClass::Copyleft)
        } else if lower.contains("gnu general public license") {
            ("GPL-2.0", LicenseClass::Copyleft)
        } else if lower.contains("bsd 3-clause")
            || lower.contains("redistribution and use in source and binary")
        {
            ("BSD-3-Clause", LicenseClass::Permissive)
        } else if lower.contains("bsd 2-clause") {
            ("BSD-2-Clause", LicenseClass::Permissive)
        } else if lower.contains("creative commons")
            && lower.contains("attribution")
            && lower.contains("noncommercial")
        {
            ("CC-BY-NC-4.0", LicenseClass::Restricted)
        } else if lower.contains("creative commons") && lower.contains("share alike") {
            ("CC-BY-SA-4.0", LicenseClass::Copyleft)
        } else if lower.contains("creative commons") && lower.contains("attribution") {
            ("CC-BY-4.0", LicenseClass::Permissive)
        } else if lower.contains("llama") && lower.contains("community license") {
            ("Llama-Community", LicenseClass::Restricted)
        } else {
            return None;
        };

        Some(DetectedLicense {
            spdx_id: Some(detected.0.to_string()),
            raw: detected.0.to_string(),
            source: LicenseSource::LicenseFile,
            classification: detected.1,
        })
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    /// Normalize a raw license string to an SPDX identifier.
    fn normalize_spdx(raw: &str) -> Option<String> {
        let lower = raw.to_lowercase().replace(' ', "-");
        for known in KNOWN_LICENSES {
            if lower == known.0.to_lowercase() || lower == known.1.to_lowercase() {
                return Some(known.1.to_string());
            }
        }
        // Return as-is if it looks like an SPDX ID
        if raw.contains('-') && raw.len() < 40 {
            Some(raw.to_string())
        } else {
            None
        }
    }

    /// Classify a license string.
    fn classify_license(raw: &str) -> LicenseClass {
        let lower = raw.to_lowercase();

        if lower.contains("mit")
            || lower.contains("apache")
            || lower.contains("bsd")
            || lower.contains("isc")
            || lower.contains("unlicense")
            || lower.contains("cc0")
            || lower.contains("cc-by-4")
            || lower.contains("wtfpl")
        {
            LicenseClass::Permissive
        } else if lower.contains("agpl")
            || lower.contains("gpl")
            || lower.contains("lgpl")
            || lower.contains("mpl")
            || lower.contains("cc-by-sa")
        {
            LicenseClass::Copyleft
        } else if lower.contains("nc")
            || lower.contains("noncommercial")
            || lower.contains("llama")
            || lower.contains("gemma")
            || lower.contains("restricted")
        {
            LicenseClass::Restricted
        } else if lower.contains("proprietary")
            || lower.contains("commercial")
            || lower.contains("all rights reserved")
        {
            LicenseClass::Proprietary
        } else {
            LicenseClass::Unknown
        }
    }

    /// Check for license compatibility issues.
    fn check_compatibility(licenses: &[DetectedLicense], warnings: &mut Vec<String>) {
        let classes: Vec<&LicenseClass> = licenses.iter().map(|l| &l.classification).collect();

        // Warn on mix of copyleft and permissive
        if classes.contains(&&LicenseClass::Copyleft)
            && classes.contains(&&LicenseClass::Permissive)
        {
            warnings.push(
                "Mixed copyleft and permissive licenses detected — check compatibility".to_string(),
            );
        }

        // Warn on restricted licenses
        if classes.contains(&&LicenseClass::Restricted) {
            warnings.push(
                "Non-commercial or restricted license detected — review usage terms".to_string(),
            );
        }

        // Warn on proprietary
        if classes.contains(&&LicenseClass::Proprietary) {
            warnings.push(
                "Proprietary license detected — redistribution may be prohibited".to_string(),
            );
        }
    }
}

/// Display helper for license reports.
impl LicenseScanReport {
    pub fn display(&self) -> String {
        let mut out = String::new();

        out.push_str("License Scan Report\n");
        out.push_str("──────────────────────────────────\n");

        if self.licenses.is_empty() {
            out.push_str("⚠ No licenses detected.\n");
        } else {
            for lic in &self.licenses {
                let spdx = lic.spdx_id.as_deref().unwrap_or("(unknown)");
                out.push_str(&format!(
                    "  {} [{}] (from {:?})\n",
                    spdx, lic.classification, lic.source
                ));
            }
        }

        if !self.warnings.is_empty() {
            out.push_str("\nWarnings:\n");
            for w in &self.warnings {
                out.push_str(&format!("  ⚠ {}\n", w));
            }
        }

        out
    }
}

// ── Known licenses ───────────────────────────────────────────────────────────

/// (raw_variant, spdx_id) pairs for normalization.
static KNOWN_LICENSES: &[(&str, &str)] = &[
    ("mit", "MIT"),
    ("apache-2.0", "Apache-2.0"),
    ("apache 2.0", "Apache-2.0"),
    ("gpl-3.0", "GPL-3.0-only"),
    ("gpl-2.0", "GPL-2.0-only"),
    ("agpl-3.0", "AGPL-3.0-only"),
    ("agpl-3.0-or-later", "AGPL-3.0-or-later"),
    ("lgpl-3.0", "LGPL-3.0-only"),
    ("bsd-3-clause", "BSD-3-Clause"),
    ("bsd-2-clause", "BSD-2-Clause"),
    ("cc-by-4.0", "CC-BY-4.0"),
    ("cc-by-sa-4.0", "CC-BY-SA-4.0"),
    ("cc-by-nc-4.0", "CC-BY-NC-4.0"),
    ("cc0-1.0", "CC0-1.0"),
    ("unlicense", "Unlicense"),
    ("isc", "ISC"),
    ("mpl-2.0", "MPL-2.0"),
    ("llama2", "Llama-2-Community"),
    ("llama3", "Llama-3-Community"),
    ("llama3.1", "Llama-3.1-Community"),
    ("gemma", "Gemma-Terms"),
    ("openrail", "OpenRAIL-M"),
    ("bigscience-openrail-m", "BigScience-OpenRAIL-M"),
    ("creativeml-openrail-m", "CreativeML-OpenRAIL-M"),
];

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gguf::test_support::{build, Meta};

    #[test]
    fn test_gguf_license_read_from_the_real_key() {
        let data = build(
            &[
                ("general.architecture", Meta::Str("llama")),
                ("general.license", Meta::Str("apache-2.0")),
            ],
            &[("blk.0.attn_q.weight", &[64, 64], 0)],
        );

        let lic = LicenseScanner::extract_gguf_license(&data).unwrap();
        assert_eq!(lic.raw, "apache-2.0");
        assert_eq!(lic.spdx_id.as_deref(), Some("Apache-2.0"));
        assert_eq!(lic.classification, LicenseClass::Permissive);
        assert_eq!(lic.source, LicenseSource::GgufMetadata);
    }

    /// The scanner used to search a 512-byte window after the literal text
    /// `general.license` for any entry in `KNOWN_LICENSES`, taking the first
    /// table hit. `("mit", "MIT")` is first, matched as a bare substring — so
    /// ordinary license boilerplate containing "limitations" or "permitted"
    /// anywhere nearby classified a restricted model as MIT / permissive.
    #[test]
    fn test_gguf_license_not_confused_by_neighbouring_metadata() {
        let data = build(
            &[
                ("general.license", Meta::Str("llama3.1")),
                (
                    "general.description",
                    Meta::Str(
                        "Licensed as-is; see the LICENSE file for permitted uses and \
                         limitations under the applicable terms.",
                    ),
                ),
            ],
            &[],
        );

        let lic = LicenseScanner::extract_gguf_license(&data).unwrap();
        assert_eq!(lic.raw, "llama3.1");
        assert_eq!(lic.spdx_id.as_deref(), Some("Llama-3.1-Community"));
        assert_eq!(
            lic.classification,
            LicenseClass::Restricted,
            "a Llama community licence must not be reported as permissive"
        );
    }

    #[test]
    fn test_gguf_license_absent_or_malformed() {
        // No `general.license` key at all.
        let no_key = build(&[("general.architecture", Meta::Str("llama"))], &[]);
        assert!(LicenseScanner::extract_gguf_license(&no_key).is_none());

        // Present but empty — not a licence claim.
        let empty = build(&[("general.license", Meta::Str("   "))], &[]);
        assert!(LicenseScanner::extract_gguf_license(&empty).is_none());

        // Truncated at every length: never panics, never invents a licence.
        let full = build(&[("general.license", Meta::Str("mit"))], &[]);
        for cut in 0..full.len() {
            let _ = LicenseScanner::extract_gguf_license(&full[..cut]);
        }
    }

    #[test]
    fn test_scan_bytes_picks_up_gguf_license() {
        let data = build(&[("general.license", Meta::Str("cc-by-nc-4.0"))], &[]);
        let report = LicenseScanner::scan_bytes(&data, "model.gguf");

        let lic = report
            .licenses
            .iter()
            .find(|l| l.source == LicenseSource::GgufMetadata)
            .expect("GGUF licence should be detected end-to-end");
        assert_eq!(lic.raw, "cc-by-nc-4.0");
        assert_eq!(lic.classification, LicenseClass::Restricted);
    }

    #[test]
    fn test_yaml_frontmatter() {
        let text = "---\nlicense: apache-2.0\ntags:\n  - llm\n---\n# My Model\n";
        let lic = LicenseScanner::extract_yaml_license(text).unwrap();
        assert_eq!(lic.raw, "apache-2.0");
        assert_eq!(lic.classification, LicenseClass::Permissive);
    }

    #[test]
    fn test_yaml_no_frontmatter() {
        let text = "# Just a readme\nNo frontmatter here.";
        assert!(LicenseScanner::extract_yaml_license(text).is_none());
    }

    #[test]
    fn test_config_json_license() {
        let json = r#"{"model_type": "llama", "license": "mit"}"#;
        let lic = LicenseScanner::extract_config_license(json).unwrap();
        assert_eq!(lic.raw, "mit");
        assert_eq!(lic.classification, LicenseClass::Permissive);
    }

    #[test]
    fn test_license_text_detection() {
        let apache = "Apache License\nVersion 2.0, January 2004\nTerms and conditions...";
        let lic = LicenseScanner::detect_license_from_text(apache).unwrap();
        assert_eq!(lic.spdx_id.as_deref(), Some("Apache-2.0"));

        let mit = "MIT License\n\nPermission is hereby granted, free of charge...";
        let lic = LicenseScanner::detect_license_from_text(mit).unwrap();
        assert_eq!(lic.spdx_id.as_deref(), Some("MIT"));
    }

    #[test]
    fn test_classify_license() {
        assert_eq!(
            LicenseScanner::classify_license("MIT"),
            LicenseClass::Permissive
        );
        assert_eq!(
            LicenseScanner::classify_license("AGPL-3.0"),
            LicenseClass::Copyleft
        );
        assert_eq!(
            LicenseScanner::classify_license("cc-by-nc-4.0"),
            LicenseClass::Restricted
        );
        assert_eq!(
            LicenseScanner::classify_license("proprietary"),
            LicenseClass::Proprietary
        );
        assert_eq!(
            LicenseScanner::classify_license("custom-xyz"),
            LicenseClass::Unknown
        );
    }

    #[test]
    fn test_compatibility_warnings() {
        let licenses = vec![
            DetectedLicense {
                spdx_id: Some("MIT".to_string()),
                raw: "mit".to_string(),
                source: LicenseSource::LicenseFile,
                classification: LicenseClass::Permissive,
            },
            DetectedLicense {
                spdx_id: Some("GPL-3.0".to_string()),
                raw: "gpl-3.0".to_string(),
                source: LicenseSource::ModelCard,
                classification: LicenseClass::Copyleft,
            },
        ];
        let mut warnings = Vec::new();
        LicenseScanner::check_compatibility(&licenses, &mut warnings);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("copyleft"));
    }

    #[test]
    fn test_scan_directory() {
        let dir = tempfile::tempdir().unwrap();

        // Create a README with frontmatter
        fs::write(
            dir.path().join("README.md"),
            "---\nlicense: apache-2.0\n---\n# Model",
        )
        .unwrap();

        let report = LicenseScanner::scan_directory(dir.path()).unwrap();
        assert!(report.has_license);
        assert_eq!(report.licenses.len(), 1);
        assert_eq!(report.licenses[0].raw, "apache-2.0");
    }

    #[test]
    fn test_normalize_spdx() {
        assert_eq!(
            LicenseScanner::normalize_spdx("apache-2.0"),
            Some("Apache-2.0".to_string())
        );
        assert_eq!(
            LicenseScanner::normalize_spdx("MIT"),
            Some("MIT".to_string())
        );
        assert_eq!(
            LicenseScanner::normalize_spdx("llama3.1"),
            Some("Llama-3.1-Community".to_string())
        );
    }

    #[test]
    fn test_display() {
        let report = LicenseScanReport {
            licenses: vec![DetectedLicense {
                spdx_id: Some("MIT".to_string()),
                raw: "mit".to_string(),
                source: LicenseSource::ModelCard,
                classification: LicenseClass::Permissive,
            }],
            warnings: vec![],
            has_license: true,
        };
        let display = report.display();
        assert!(display.contains("MIT"));
        assert!(display.contains("Permissive"));
    }
}
