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

/// Ceiling on bytes decompressed out of a ZIP while scanning, so a zip bomb
/// cannot turn a scan into an out-of-memory abort.
const MAX_SCAN_INFLATE_BYTES: u64 = 512 * 1024 * 1024;

/// Ceiling on ZIP members inspected per file.
const MAX_SCAN_ZIP_MEMBERS: usize = 4096;

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

        let is_zip = data.len() >= 2 && data[0..2] == [0x50, 0x4B]; // PK zip magic

        // Check for pickle magic bytes (protocol markers \x80\x02 through \x80\x05)
        let is_pickle = data.len() >= 2 && data[0] == 0x80 && (2..=5).contains(&data[1]);

        let is_pickle_format = is_pickle || is_zip || Self::has_pickle_extension(path);

        let mut findings = Vec::new();
        Self::scan_container(&data, is_zip, is_pickle_format, &mut findings);

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

        Self::scan_container(data, is_zip, is_pickle_format, &mut findings);

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

    /// Scan a file's contents, descending into a ZIP container when there is
    /// one so that compressed members are examined as the loader will see them.
    fn scan_container(
        data: &[u8],
        is_zip: bool,
        is_pickle_format: bool,
        findings: &mut Vec<ScanFinding>,
    ) {
        if is_zip {
            let members = Self::zip_members(data);
            if members.is_empty() {
                // Unparseable as a ZIP despite the magic — fall back to the
                // raw bytes rather than silently scanning nothing.
                Self::scan_payload(data, None, is_pickle_format, findings);
                return;
            }
            for (name, bytes) in members {
                let treat_as_pickle = Self::looks_like_pickle(&bytes, &name);
                Self::scan_payload(&bytes, Some(&name), treat_as_pickle, findings);
            }
            Self::merge_findings(findings);
        } else {
            Self::scan_payload(data, None, is_pickle_format, findings);
        }
    }

    /// Collapse findings that share a code, summing counts.
    ///
    /// A container yields one finding per member; the report is about the file.
    fn merge_findings(findings: &mut Vec<ScanFinding>) {
        let mut merged: Vec<ScanFinding> = Vec::new();
        for f in findings.drain(..) {
            if let Some(existing) = merged.iter_mut().find(|e| e.code == f.code) {
                existing.count += f.count;
                existing.severity = existing.severity.max(f.severity);
                if existing.offsets.len() < 10 {
                    let room = 10 - existing.offsets.len();
                    existing.offsets.extend(f.offsets.into_iter().take(room));
                }
            } else {
                merged.push(f);
            }
        }
        *findings = merged;
    }

    /// Run the opcode and pattern scans over one logical payload.
    ///
    /// `label` is `None` for the file itself and `Some(member)` for a ZIP
    /// member, so a finding can say where inside the container it came from.
    fn scan_payload(
        data: &[u8],
        label: Option<&str>,
        treat_as_pickle: bool,
        findings: &mut Vec<ScanFinding>,
    ) {
        let before = findings.len();
        if treat_as_pickle {
            Self::scan_opcodes(data, findings);
        }
        Self::scan_patterns(data, findings);

        if let Some(member) = label {
            for f in &mut findings[before..] {
                f.description = format!("{} (in ZIP member `{member}`)", f.description);
            }
        }
    }

    /// Decompress the members of a ZIP container so their contents can be
    /// scanned.
    ///
    /// `torch.save` writes *stored* (uncompressed) members, so a raw-byte scan
    /// happens to see the payload. But `torch.load` accepts a DEFLATE-compressed
    /// archive just as happily, and then nothing in the file literally contains
    /// the pickle opcodes or the `os\nsystem` string — a scanner that only reads
    /// raw bytes declares such a file clean. Anything that decides whether a
    /// model is safe to load has to look at what will actually be unpickled.
    fn zip_members(data: &[u8]) -> Vec<(String, Vec<u8>)> {
        Self::zip_members_bounded(data, MAX_SCAN_INFLATE_BYTES, MAX_SCAN_ZIP_MEMBERS)
    }

    /// [`Self::zip_members`] with explicit limits, so the bounds can be tested
    /// without inflating half a gigabyte.
    fn zip_members_bounded(
        data: &[u8],
        mut budget: u64,
        max_members: usize,
    ) -> Vec<(String, Vec<u8>)> {
        use std::io::Read;

        let mut out = Vec::new();
        let Ok(mut archive) = zip::ZipArchive::new(std::io::Cursor::new(data)) else {
            return out;
        };

        for i in 0..archive.len().min(max_members) {
            let Ok(entry) = archive.by_index(i) else {
                continue;
            };
            if entry.is_dir() {
                continue;
            }
            let name = entry.name().to_string();

            // Read at most the remaining budget, so a zip bomb cannot make the
            // scanner allocate without bound.
            let mut buf = Vec::new();
            if entry.take(budget).read_to_end(&mut buf).is_err() {
                continue;
            }
            budget = budget.saturating_sub(buf.len() as u64);
            out.push((name, buf));

            if budget == 0 {
                break;
            }
        }

        out
    }

    /// Does this payload look like a pickle stream worth opcode-scanning?
    fn looks_like_pickle(data: &[u8], name: &str) -> bool {
        let proto = data.len() >= 2 && data[0] == 0x80 && (2..=5).contains(&data[1]);
        proto || name.ends_with(".pkl") || name.ends_with(".pickle") || name.ends_with("data.pkl")
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

    /// Build a PyTorch-shaped ZIP (`archive/data.pkl` plus a tensor blob) with
    /// a chosen compression method.
    fn torch_zip(pickle: &[u8], method: zip::CompressionMethod) -> Vec<u8> {
        use std::io::Write;
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zw = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::SimpleFileOptions::default().compression_method(method);
            zw.start_file("archive/data.pkl", opts).unwrap();
            zw.write_all(pickle).unwrap();
            zw.start_file("archive/data/0", opts).unwrap();
            zw.write_all(&[0u8; 64]).unwrap();
            zw.finish().unwrap();
        }
        buf.into_inner()
    }

    /// A malicious pickle: protocol 2, a GLOBAL naming `os.system`, and REDUCE.
    fn malicious_pickle() -> Vec<u8> {
        let mut p = vec![0x80, 0x02];
        p.push(0x63); // GLOBAL
        p.extend_from_slice(b"os\nsystem\n");
        p.push(0x52); // REDUCE
        p.push(0x2e); // STOP
        p
    }

    /// `torch.save` writes uncompressed ZIP members, so the payload is visible
    /// in the raw bytes. But `torch.load` accepts a DEFLATE-compressed archive
    /// just the same, and then the payload is not literally present anywhere in
    /// the file — a scanner reading raw bytes sees nothing and reports the file
    /// as clean.
    #[test]
    fn test_scan_sees_payload_inside_a_compressed_zip_member() {
        let stored = torch_zip(&malicious_pickle(), zip::CompressionMethod::Stored);
        let deflated = torch_zip(&malicious_pickle(), zip::CompressionMethod::Deflated);

        let stored_report = PickleScanner::scan_bytes(&stored, "stored.pt");
        assert!(
            stored_report
                .findings
                .iter()
                .any(|f| f.code == "os module import"),
            "uncompressed payload must be caught"
        );

        let deflated_report = PickleScanner::scan_bytes(&deflated, "deflated.pt");
        assert!(
            deflated_report
                .findings
                .iter()
                .any(|f| f.code == "os module import"),
            "a DEFLATE-compressed member hides the payload from a raw-byte scan; \
             the scanner must decompress before deciding a file is clean"
        );
        assert!(!deflated_report.safe);
    }

    /// Decompressing to scan must stay bounded: a small archive that inflates
    /// far beyond the budget has to stop at it rather than exhaust memory.
    /// Exercised with small limits so the test costs milliseconds.
    #[test]
    fn test_zip_bomb_is_bounded() {
        use std::io::Write;

        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zw = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            // 1 MiB of zeroes per member compresses to ~1 KiB.
            for i in 0..8 {
                zw.start_file(format!("m{i}.pkl"), opts).unwrap();
                zw.write_all(&vec![0u8; 1024 * 1024]).unwrap();
            }
            zw.finish().unwrap();
        }
        let bomb = buf.into_inner();
        assert!(
            bomb.len() < 64 * 1024,
            "8 MiB of zeroes should compress small; got {} bytes",
            bomb.len()
        );

        // Budget of 100 KiB against 8 MiB of content.
        let budget = 100 * 1024;
        let members = PickleScanner::zip_members_bounded(&bomb, budget, MAX_SCAN_ZIP_MEMBERS);
        let total: u64 = members.iter().map(|(_, b)| b.len() as u64).sum();
        assert!(
            total <= budget,
            "inflated {total} bytes, over the {budget}-byte budget"
        );

        // The member cap is enforced independently of the byte budget.
        let capped = PickleScanner::zip_members_bounded(&bomb, u64::MAX, 3);
        assert_eq!(capped.len(), 3);

        // And a bomb still yields a report rather than hanging or aborting.
        let report = PickleScanner::scan_bytes(&bomb, "bomb.pt");
        assert!(report.is_zip_archive);
    }

    /// Findings from several members collapse into one entry per code, so the
    /// report describes the file rather than listing the same issue N times.
    #[test]
    fn test_findings_merge_across_zip_members() {
        use std::io::Write;

        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zw = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            for i in 0..3 {
                zw.start_file(format!("archive/{i}/data.pkl"), opts)
                    .unwrap();
                zw.write_all(&malicious_pickle()).unwrap();
            }
            zw.finish().unwrap();
        }

        let report = PickleScanner::scan_bytes(&buf.into_inner(), "multi.pt");
        let os_findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.code == "os module import")
            .collect();
        assert_eq!(os_findings.len(), 1, "one finding per code, not per member");
        assert_eq!(
            os_findings[0].count, 3,
            "but the count reflects all members"
        );
        assert!(!report.safe);
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
