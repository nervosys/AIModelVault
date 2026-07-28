//! Pickle safety scanner for PyTorch model files.
//!
//! PyTorch `.pt`, `.pth`, and `.bin` files use Python's pickle protocol,
//! which can execute arbitrary code during deserialization.  This module
//! scans for dangerous opcodes (REDUCE, GLOBAL, BUILD, INST, OBJ, STACK_GLOBAL,
//! etc.) and reports findings with severity levels.
//!
//! This is a static analysis tool — it does NOT execute any pickle code.

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

// ── Dangerous pickle opcodes ─────────────────────────────────────────────────

/// Pickle opcodes that can trigger arbitrary code execution.
const DANGEROUS_OPCODES: &[(u8, &str, &str)] = &[
    (
        0x52,
        "REDUCE",
        "Calls a callable with args — arbitrary code execution",
    ),
    (
        0x63,
        "GLOBAL",
        "Imports a module attribute — can import os, subprocess, etc.",
    ),
    (
        0x62,
        "BUILD",
        "Calls __setstate__ — can trigger arbitrary code in __setstate__",
    ),
    (
        0x69,
        "INST",
        "Instantiates a class — can run __init__ with arbitrary args",
    ),
    (
        0x81,
        "NEWOBJ",
        "Creates a new object — tp.__new__(cls, *args)",
    ),
    (0x92, "NEWOBJ_EX", "Extended NEWOBJ with kwargs"),
    (
        0x93,
        "STACK_GLOBAL",
        "Push a global from stack values — similar to GLOBAL",
    ),
];

/// Patterns found in malicious pickle payloads (byte sequences).
const DANGEROUS_PATTERNS: &[(&[u8], &str, &str)] = &[
    (
        b"os\n",
        "os module import",
        "Accesses the operating system module",
    ),
    (
        b"subprocess",
        "subprocess module",
        "Can execute shell commands",
    ),
    (
        b"__builtin__",
        "__builtin__ access",
        "Access to Python builtins (exec, eval)",
    ),
    (b"builtins", "builtins module", "Access to Python builtins"),
    (
        b"commands",
        "commands module",
        "Legacy command execution module",
    ),
    (b"nt\nsystem", "nt.system call", "Windows system() call"),
    (b"posix\nsystem", "posix.system call", "Unix system() call"),
    (
        b"exec\n",
        "exec function",
        "Arbitrary Python code execution",
    ),
    (
        b"eval\n",
        "eval function",
        "Arbitrary Python expression evaluation",
    ),
    (
        b"__import__",
        "__import__ function",
        "Dynamic module importing",
    ),
    (b"runpy", "runpy module", "Runs Python modules/scripts"),
    (
        b"webbrowser",
        "webbrowser module",
        "Can open arbitrary URLs",
    ),
];

// ── Severity ─────────────────────────────────────────────────────────────────

/// Severity level of a scanning finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational — common in legitimate files
    Info,
    /// Warning — potentially dangerous but may be legitimate
    Warning,
    /// Critical — strong indicator of malicious payload
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Critical => write!(f, "CRIT"),
        }
    }
}

// ── Scan finding ─────────────────────────────────────────────────────────────

/// Individual finding from a pickle scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanFinding {
    /// Severity level
    pub severity: Severity,
    /// Short identifier
    pub code: String,
    /// Human-readable description
    pub description: String,
    /// Byte offset(s) where the pattern was found
    pub offsets: Vec<usize>,
    /// Number of occurrences
    pub count: usize,
}

// ── Scan report ──────────────────────────────────────────────────────────────

/// Complete scan report for a model file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    /// File that was scanned
    pub file_path: String,
    /// File size in bytes
    pub file_size: u64,
    /// Whether this appears to be a pickle-based format
    pub is_pickle_format: bool,
    /// Whether this appears to be a ZIP-based PyTorch checkpoint
    pub is_zip_archive: bool,
    /// Overall safety assessment
    pub safe: bool,
    /// Individual findings
    pub findings: Vec<ScanFinding>,
    /// Summary counts by severity
    pub summary: HashMap<String, usize>,
    /// Recommendation
    pub recommendation: String,
}

// ── Scanner ──────────────────────────────────────────────────────────────────

/// Scans model files for dangerous pickle opcodes and patterns.
pub struct PickleScanner;

impl PickleScanner {
    /// Scan a file and return a [`ScanReport`].
    pub fn scan(path: &Path) -> Result<ScanReport> {
        let data = fs::read(path)?;
        let file_size = data.len() as u64;
        let file_path = path.display().to_string();

        let is_zip = data.len() >= 4
            && (data[0..2] == [0x50, 0x4B]   // PK zip magic
                || data[0..4] == [0x80, 0x02, 0x7D, 0x71]); // pickle proto 2

        // Check for pickle magic bytes (protocol markers \x80\x02 through \x80\x05)
        let is_pickle = data.len() >= 2 && data[0] == 0x80 && (2..=5).contains(&data[1]);

        let is_pickle_format = is_pickle || is_zip || Self::has_pickle_extension(path);

        let mut findings = Vec::new();

        // Scan for dangerous opcodes
        if is_pickle_format {
            Self::scan_opcodes(&data, &mut findings);
        }

        // Always scan for dangerous string patterns
        Self::scan_patterns(&data, &mut findings);

        // Sort findings by severity (critical first)
        findings.sort_by_key(|f| std::cmp::Reverse(f.severity));

        // Build summary
        let mut summary = HashMap::new();
        for f in &findings {
            *summary.entry(format!("{}", f.severity)).or_insert(0) += 1;
        }

        let critical_count = summary.get("CRIT").copied().unwrap_or(0);
        let warning_count = summary.get("WARN").copied().unwrap_or(0);

        let safe = critical_count == 0 && warning_count == 0;

        let recommendation = if critical_count > 0 {
            "DANGEROUS: This file contains opcodes/patterns strongly associated with \
             malicious payloads. Do NOT load this file with pickle.loads() or torch.load(). \
             Consider using safetensors format instead."
                .to_string()
        } else if warning_count > 0 {
            "CAUTION: This file contains potentially dangerous opcodes. These are common \
             in legitimate PyTorch files but can be exploited. Consider converting to \
             safetensors format for maximum safety."
                .to_string()
        } else if is_pickle_format {
            "LOW RISK: No dangerous opcodes or patterns detected. The file uses pickle \
             format which inherently carries risk. Consider converting to safetensors \
             for guaranteed safety."
                .to_string()
        } else {
            "SAFE: This file does not appear to use pickle serialization.".to_string()
        };

        Ok(ScanReport {
            file_path,
            file_size,
            is_pickle_format,
            is_zip_archive: is_zip,
            safe,
            findings,
            summary,
            recommendation,
        })
    }

    /// Scan a model stored in the vault by name/version.
    pub fn scan_bytes(data: &[u8], name: &str) -> ScanReport {
        let mut findings = Vec::new();

        let is_pickle = data.len() >= 2 && data[0] == 0x80 && (2..=5).contains(&data[1]);
        let is_zip = data.len() >= 2 && data[0..2] == [0x50, 0x4B];
        let is_pickle_format = is_pickle || is_zip;

        if is_pickle_format {
            Self::scan_opcodes(data, &mut findings);
        }
        Self::scan_patterns(data, &mut findings);

        findings.sort_by_key(|f| std::cmp::Reverse(f.severity));

        let mut summary = HashMap::new();
        for f in &findings {
            *summary.entry(format!("{}", f.severity)).or_insert(0) += 1;
        }

        let critical_count = summary.get("CRIT").copied().unwrap_or(0);
        let warning_count = summary.get("WARN").copied().unwrap_or(0);
        let safe = critical_count == 0 && warning_count == 0;

        let recommendation = if critical_count > 0 {
            "DANGEROUS: Contains malicious opcodes/patterns.".to_string()
        } else if warning_count > 0 {
            "CAUTION: Contains potentially dangerous opcodes.".to_string()
        } else {
            "No dangerous patterns detected.".to_string()
        };

        ScanReport {
            file_path: name.to_string(),
            file_size: data.len() as u64,
            is_pickle_format,
            is_zip_archive: is_zip,
            safe,
            findings,
            summary,
            recommendation,
        }
    }

    fn has_pickle_extension(path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| {
                matches!(
                    ext.to_lowercase().as_str(),
                    "pt" | "pth" | "bin" | "pkl" | "pickle"
                )
            })
    }

    fn scan_opcodes(data: &[u8], findings: &mut Vec<ScanFinding>) {
        for &(opcode, name, desc) in DANGEROUS_OPCODES {
            let offsets: Vec<usize> = data
                .iter()
                .enumerate()
                .filter(|(_, &b)| b == opcode)
                .map(|(i, _)| i)
                .collect();

            if !offsets.is_empty() {
                let severity = match name {
                    "REDUCE" | "STACK_GLOBAL" => Severity::Warning,
                    "GLOBAL" | "INST" => Severity::Warning,
                    _ => Severity::Info,
                };

                findings.push(ScanFinding {
                    severity,
                    code: name.to_string(),
                    description: desc.to_string(),
                    count: offsets.len(),
                    offsets: offsets.into_iter().take(10).collect(), // limit stored offsets
                });
            }
        }
    }

    fn scan_patterns(data: &[u8], findings: &mut Vec<ScanFinding>) {
        for &(pattern, name, desc) in DANGEROUS_PATTERNS {
            let offsets: Vec<usize> = data
                .windows(pattern.len())
                .enumerate()
                .filter(|(_, w)| *w == pattern)
                .map(|(i, _)| i)
                .collect();

            if !offsets.is_empty() {
                findings.push(ScanFinding {
                    severity: Severity::Critical,
                    code: name.to_string(),
                    description: desc.to_string(),
                    count: offsets.len(),
                    offsets: offsets.into_iter().take(10).collect(),
                });
            }
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_safe_file() {
        let data = b"This is just some safe binary data with no pickle opcodes";
        let report = PickleScanner::scan_bytes(data, "safe.safetensors");
        assert!(report.safe);
        assert!(!report.is_pickle_format);
    }

    #[test]
    fn test_scan_dangerous_patterns() {
        let mut data = Vec::new();
        data.extend_from_slice(b"some header data");
        data.extend_from_slice(b"os\nsystem");
        data.extend_from_slice(b"more data");

        let report = PickleScanner::scan_bytes(&data, "suspicious.pt");
        assert!(!report.safe);
        assert!(report.findings.iter().any(|f| f.code == "os module import"));
    }

    #[test]
    fn test_scan_pickle_with_reduce() {
        // Simulate a pickle stream with REDUCE opcode
        let mut data = vec![0x80, 0x02]; // pickle protocol 2
        data.push(0x52); // REDUCE opcode
        data.extend_from_slice(b"some more data");

        let report = PickleScanner::scan_bytes(&data, "model.pt");
        assert!(report.is_pickle_format);
        assert!(report.findings.iter().any(|f| f.code == "REDUCE"));
    }

    #[test]
    fn test_scan_zip_format() {
        let mut data = vec![0x50, 0x4B, 0x03, 0x04]; // PK zip magic
        data.extend_from_slice(&[0; 100]);

        let report = PickleScanner::scan_bytes(&data, "checkpoint.pt");
        assert!(report.is_zip_archive);
        assert!(report.is_pickle_format);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
    }
}
