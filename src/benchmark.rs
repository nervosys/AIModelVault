//! Benchmark metadata — attach evaluation results to model versions.
//!
//! Records structured benchmark scores (perplexity, MMLU, HumanEval, latency,
//! throughput, etc.) alongside model versions and serialises them to JSON.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::{Result, VaultError};

// ── Types ────────────────────────────────────────────────────────────────────

/// A single benchmark result for a model version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Name of the benchmark (e.g. "MMLU", "HumanEval", "perplexity")
    pub benchmark: String,
    /// Numeric score or value
    pub score: f64,
    /// Unit of measurement (e.g. "accuracy", "ppl", "tokens/s", "ms")
    pub unit: String,
    /// Higher is better? (true for accuracy, false for perplexity/latency)
    pub higher_is_better: bool,
    /// Dataset or split used (e.g. "test", "validation", "5-shot")
    pub dataset: Option<String>,
    /// Additional key-value metadata
    pub metadata: BTreeMap<String, String>,
    /// When the benchmark was recorded
    pub recorded_at: String,
}

/// A collection of benchmark results for a specific model version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRecord {
    /// Model name
    pub model_name: String,
    /// Model version
    pub version: u64,
    /// Hardware description (e.g. "NVIDIA A100 80GB", "Apple M2 Max")
    pub hardware: Option<String>,
    /// Software environment (e.g. "PyTorch 2.1, CUDA 12.1")
    pub environment: Option<String>,
    /// Individual benchmark results
    pub results: Vec<BenchmarkResult>,
    /// When this record was created
    pub created_at: String,
    /// When this record was last updated
    pub updated_at: String,
}

impl BenchmarkRecord {
    /// Create a new empty benchmark record for a model version.
    pub fn new(model_name: &str, version: u64) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            model_name: model_name.to_string(),
            version,
            hardware: None,
            environment: None,
            results: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Add a benchmark result.
    pub fn add_result(&mut self, benchmark: &str, score: f64, unit: &str, higher_is_better: bool) {
        self.results.push(BenchmarkResult {
            benchmark: benchmark.to_string(),
            score,
            unit: unit.to_string(),
            higher_is_better,
            dataset: None,
            metadata: BTreeMap::new(),
            recorded_at: Utc::now().to_rfc3339(),
        });
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Add a benchmark result with full details.
    pub fn add_detailed_result(
        &mut self,
        benchmark: &str,
        score: f64,
        unit: &str,
        higher_is_better: bool,
        dataset: Option<&str>,
        metadata: BTreeMap<String, String>,
    ) {
        self.results.push(BenchmarkResult {
            benchmark: benchmark.to_string(),
            score,
            unit: unit.to_string(),
            higher_is_better,
            dataset: dataset.map(|s| s.to_string()),
            metadata,
            recorded_at: Utc::now().to_rfc3339(),
        });
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Get a result by benchmark name.
    pub fn get_result(&self, benchmark: &str) -> Option<&BenchmarkResult> {
        self.results
            .iter()
            .find(|r| r.benchmark.eq_ignore_ascii_case(benchmark))
    }

    /// Save benchmark record to a JSON file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| VaultError::SerializationError(e.to_string()))?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load benchmark record from a JSON file.
    pub fn load(path: &Path) -> Result<Self> {
        let data = fs::read_to_string(path)?;
        let record: Self = serde_json::from_str(&data)
            .map_err(|e| VaultError::SerializationError(e.to_string()))?;
        Ok(record)
    }

    /// Format as a human-readable table.
    pub fn display(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!(
            "Benchmarks for {} v{}\n",
            self.model_name, self.version
        ));
        out.push_str("──────────────────────────────────\n");

        if let Some(hw) = &self.hardware {
            out.push_str(&format!("Hardware: {}\n", hw));
        }
        if let Some(env) = &self.environment {
            out.push_str(&format!("Environment: {}\n", env));
        }

        if self.results.is_empty() {
            out.push_str("No benchmark results recorded.\n");
            return out;
        }

        out.push_str(&format!(
            "\n{:<20} {:>12} {:<12} {:<8}\n",
            "Benchmark", "Score", "Unit", "Direction"
        ));
        out.push_str(&format!("{}\n", "─".repeat(56)));

        for r in &self.results {
            let direction = if r.higher_is_better { "↑" } else { "↓" };
            out.push_str(&format!(
                "{:<20} {:>12.4} {:<12} {:<8}\n",
                r.benchmark, r.score, r.unit, direction
            ));
        }

        out
    }
}

// ── Store helper ─────────────────────────────────────────────────────────────

/// Benchmark store — manages benchmark records alongside a vault.
pub struct BenchmarkStore {
    /// Directory where benchmark JSON files are stored
    base_dir: std::path::PathBuf,
}

impl BenchmarkStore {
    /// Create a new store rooted at the given directory.
    pub fn new(base_dir: &Path) -> Result<Self> {
        fs::create_dir_all(base_dir)?;
        Ok(Self {
            base_dir: base_dir.to_path_buf(),
        })
    }

    fn record_path(&self, model_name: &str, version: u64) -> std::path::PathBuf {
        self.base_dir
            .join(format!("{}__v{}.bench.json", model_name, version))
    }

    /// Get or create a benchmark record.
    pub fn get_or_create(&self, model_name: &str, version: u64) -> Result<BenchmarkRecord> {
        let path = self.record_path(model_name, version);
        if path.exists() {
            BenchmarkRecord::load(&path)
        } else {
            Ok(BenchmarkRecord::new(model_name, version))
        }
    }

    /// Save a benchmark record.
    pub fn save(&self, record: &BenchmarkRecord) -> Result<()> {
        let path = self.record_path(&record.model_name, record.version);
        record.save(&path)
    }

    /// List all benchmark records for a model.
    pub fn list_for_model(&self, model_name: &str) -> Result<Vec<BenchmarkRecord>> {
        let prefix = format!("{}__v", model_name);
        let mut records = Vec::new();

        if !self.base_dir.exists() {
            return Ok(records);
        }

        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&prefix) && name.ends_with(".bench.json") {
                if let Ok(record) = BenchmarkRecord::load(&entry.path()) {
                    records.push(record);
                }
            }
        }

        records.sort_by_key(|r| r.version);
        Ok(records)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
// Exact float comparison is intentional here: these assert on literal
// constants that round-trip bit-for-bit, not on computed results.
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_record_basic() {
        let mut record = BenchmarkRecord::new("llama-7b", 1);
        record.add_result("MMLU", 0.654, "accuracy", true);
        record.add_result("perplexity", 5.12, "ppl", false);

        assert_eq!(record.results.len(), 2);
        assert_eq!(record.get_result("mmlu").unwrap().score, 0.654);
        assert!(record.get_result("nonexistent").is_none());
    }

    #[test]
    fn test_benchmark_record_save_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bench.json");

        let mut record = BenchmarkRecord::new("gpt-neo", 3);
        record.hardware = Some("NVIDIA A100".to_string());
        record.add_result("HumanEval", 0.482, "pass@1", true);
        record.save(&path).unwrap();

        let loaded = BenchmarkRecord::load(&path).unwrap();
        assert_eq!(loaded.model_name, "gpt-neo");
        assert_eq!(loaded.version, 3);
        assert_eq!(loaded.results.len(), 1);
        assert_eq!(loaded.hardware.as_deref(), Some("NVIDIA A100"));
    }

    #[test]
    fn test_benchmark_store() {
        let dir = tempfile::tempdir().unwrap();
        let store = BenchmarkStore::new(dir.path()).unwrap();

        let mut r1 = store.get_or_create("model-a", 1).unwrap();
        r1.add_result("MMLU", 0.6, "accuracy", true);
        store.save(&r1).unwrap();

        let mut r2 = store.get_or_create("model-a", 2).unwrap();
        r2.add_result("MMLU", 0.7, "accuracy", true);
        store.save(&r2).unwrap();

        let records = store.list_for_model("model-a").unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].version, 1);
        assert_eq!(records[1].version, 2);
    }

    #[test]
    fn test_benchmark_display() {
        let mut record = BenchmarkRecord::new("test", 1);
        record.add_result("MMLU", 0.75, "accuracy", true);
        record.add_result("latency", 42.5, "ms", false);

        let display = record.display();
        assert!(display.contains("MMLU"));
        assert!(display.contains("latency"));
        assert!(display.contains("↑"));
        assert!(display.contains("↓"));
    }

    #[test]
    fn test_detailed_result() {
        let mut record = BenchmarkRecord::new("test", 1);
        let mut meta = BTreeMap::new();
        meta.insert("batch_size".to_string(), "32".to_string());
        record.add_detailed_result(
            "throughput",
            1500.0,
            "tokens/s",
            true,
            Some("wikitext"),
            meta,
        );

        let r = &record.results[0];
        assert_eq!(r.benchmark, "throughput");
        assert_eq!(r.dataset.as_deref(), Some("wikitext"));
        assert_eq!(r.metadata.get("batch_size").unwrap(), "32");
    }
}
