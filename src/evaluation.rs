//! Model evaluation harness.
//!
//! Store, query, and compare evaluation results per model version.
//! Builds on the benchmark module with structured eval suites.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::{Result, VaultError};

// ── Types ────────────────────────────────────────────────────────────────────

/// A single metric result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricResult {
    /// Metric name (e.g. "accuracy", "perplexity", "f1").
    pub name: String,
    /// Numeric value.
    pub value: f64,
    /// Unit of measurement.
    pub unit: String,
    /// Higher is better?
    #[serde(default = "default_higher_is_better")]
    pub higher_is_better: bool,
}

fn default_higher_is_better() -> bool {
    true
}

/// An evaluation run — one suite executed on one model version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRun {
    /// Evaluation suite name (e.g. "mmlu", "hellaswag", "custom-qa").
    pub suite: String,
    /// Model name.
    pub model: String,
    /// Model version.
    pub version: u64,
    /// Metric results.
    pub metrics: Vec<MetricResult>,
    /// When the evaluation was run.
    pub timestamp: String,
    /// Extra context (dataset size, hardware, etc.).
    #[serde(default)]
    pub context: BTreeMap<String, String>,
}

/// Comparison between two eval runs.
#[derive(Debug, Clone, Serialize)]
pub struct EvalComparison {
    pub suite: String,
    pub model_a: String,
    pub version_a: u64,
    pub model_b: String,
    pub version_b: u64,
    pub deltas: Vec<MetricDelta>,
}

/// Delta between the same metric across two runs.
#[derive(Debug, Clone, Serialize)]
pub struct MetricDelta {
    pub metric: String,
    pub value_a: f64,
    pub value_b: f64,
    pub delta: f64,
    pub improved: bool,
}

// ── Store ────────────────────────────────────────────────────────────────────

const EVALS_FILE: &str = "evaluations.json";

/// Persisted evaluation store.
#[derive(Debug)]
pub struct EvalStore {
    path: PathBuf,
    runs: Vec<EvalRun>,
}

impl EvalStore {
    /// Open or create an evaluation store.
    pub fn new(vault_path: &Path) -> Result<Self> {
        let path = vault_path.join(EVALS_FILE);
        let runs = if path.exists() {
            let data = std::fs::read_to_string(&path)
                .map_err(|e| VaultError::StorageError(format!("read evals: {e}")))?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };
        Ok(Self { path, runs })
    }

    fn save(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(&self.runs)
            .map_err(|e| VaultError::StorageError(format!("serialize evals: {e}")))?;
        std::fs::write(&self.path, data)
            .map_err(|e| VaultError::StorageError(format!("write evals: {e}")))
    }

    /// Record an evaluation run.
    pub fn record(&mut self, run: EvalRun) -> Result<()> {
        self.runs.push(run);
        self.save()
    }

    /// Get all runs for a model (optionally filtered by version).
    pub fn get_runs(&self, model: &str, version: Option<u64>) -> Vec<&EvalRun> {
        self.runs
            .iter()
            .filter(|r| {
                r.model == model && version.map_or(true, |v| r.version == v)
            })
            .collect()
    }

    /// Get runs for a model in a specific suite.
    pub fn get_suite_runs(&self, model: &str, suite: &str) -> Vec<&EvalRun> {
        self.runs
            .iter()
            .filter(|r| r.model == model && r.suite == suite)
            .collect()
    }

    /// Compare two model versions on a given suite.
    pub fn compare(
        &self,
        model_a: &str,
        version_a: u64,
        model_b: &str,
        version_b: u64,
        suite: &str,
    ) -> Option<EvalComparison> {
        let run_a = self.runs.iter().find(|r| {
            r.model == model_a && r.version == version_a && r.suite == suite
        })?;
        let run_b = self.runs.iter().find(|r| {
            r.model == model_b && r.version == version_b && r.suite == suite
        })?;

        let deltas: Vec<MetricDelta> = run_a
            .metrics
            .iter()
            .filter_map(|ma| {
                let mb = run_b.metrics.iter().find(|m| m.name == ma.name)?;
                let delta = mb.value - ma.value;
                Some(MetricDelta {
                    metric: ma.name.clone(),
                    value_a: ma.value,
                    value_b: mb.value,
                    delta,
                    improved: if ma.higher_is_better {
                        delta > 0.0
                    } else {
                        delta < 0.0
                    },
                })
            })
            .collect();

        Some(EvalComparison {
            suite: suite.to_string(),
            model_a: model_a.to_string(),
            version_a,
            model_b: model_b.to_string(),
            version_b,
            deltas,
        })
    }

    /// List all recorded suites.
    pub fn suites(&self) -> Vec<String> {
        let mut s: Vec<String> = self.runs.iter().map(|r| r.suite.clone()).collect();
        s.sort();
        s.dedup();
        s
    }

    /// Total number of recorded runs.
    pub fn count(&self) -> usize {
        self.runs.len()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_run(model: &str, version: u64, suite: &str, accuracy: f64) -> EvalRun {
        EvalRun {
            suite: suite.into(),
            model: model.into(),
            version,
            metrics: vec![MetricResult {
                name: "accuracy".into(),
                value: accuracy,
                unit: "%".into(),
                higher_is_better: true,
            }],
            timestamp: "2025-01-01T00:00:00Z".into(),
            context: BTreeMap::new(),
        }
    }

    #[test]
    fn test_record_and_query() {
        let dir = tempdir().unwrap();
        let mut store = EvalStore::new(dir.path()).unwrap();

        store.record(make_run("llama", 1, "mmlu", 65.0)).unwrap();
        store.record(make_run("llama", 2, "mmlu", 72.0)).unwrap();

        assert_eq!(store.count(), 2);
        assert_eq!(store.get_runs("llama", None).len(), 2);
        assert_eq!(store.get_runs("llama", Some(1)).len(), 1);
    }

    #[test]
    fn test_compare() {
        let dir = tempdir().unwrap();
        let mut store = EvalStore::new(dir.path()).unwrap();

        store.record(make_run("llama", 1, "mmlu", 65.0)).unwrap();
        store.record(make_run("llama", 2, "mmlu", 72.0)).unwrap();

        let cmp = store.compare("llama", 1, "llama", 2, "mmlu").unwrap();
        assert_eq!(cmp.deltas.len(), 1);
        assert!((cmp.deltas[0].delta - 7.0).abs() < f64::EPSILON);
        assert!(cmp.deltas[0].improved);
    }

    #[test]
    fn test_compare_lower_is_better() {
        let dir = tempdir().unwrap();
        let mut store = EvalStore::new(dir.path()).unwrap();

        let mut run1 = make_run("gpt", 1, "perpl", 12.0);
        run1.metrics[0].higher_is_better = false;
        run1.metrics[0].name = "perplexity".into();

        let mut run2 = make_run("gpt", 2, "perpl", 8.0);
        run2.metrics[0].higher_is_better = false;
        run2.metrics[0].name = "perplexity".into();

        store.record(run1).unwrap();
        store.record(run2).unwrap();

        let cmp = store.compare("gpt", 1, "gpt", 2, "perpl").unwrap();
        assert!(cmp.deltas[0].improved); // lower perplexity = improved
    }

    #[test]
    fn test_suites() {
        let dir = tempdir().unwrap();
        let mut store = EvalStore::new(dir.path()).unwrap();

        store.record(make_run("m", 1, "mmlu", 50.0)).unwrap();
        store
            .record(make_run("m", 1, "hellaswag", 60.0))
            .unwrap();
        store.record(make_run("m", 2, "mmlu", 55.0)).unwrap();

        let suites = store.suites();
        assert_eq!(suites, vec!["hellaswag", "mmlu"]);
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();

        {
            let mut store = EvalStore::new(dir.path()).unwrap();
            store.record(make_run("llama", 1, "mmlu", 65.0)).unwrap();
        }

        let store = EvalStore::new(dir.path()).unwrap();
        assert_eq!(store.count(), 1);
    }

    #[test]
    fn test_get_suite_runs() {
        let dir = tempdir().unwrap();
        let mut store = EvalStore::new(dir.path()).unwrap();

        store.record(make_run("m", 1, "mmlu", 50.0)).unwrap();
        store.record(make_run("m", 1, "other", 60.0)).unwrap();

        assert_eq!(store.get_suite_runs("m", "mmlu").len(), 1);
        assert_eq!(store.get_suite_runs("m", "other").len(), 1);
        assert_eq!(store.get_suite_runs("m", "nonexistent").len(), 0);
    }
}
