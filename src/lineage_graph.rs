//! Cross-model lineage graph — track derivation relationships between
//! different models (fine-tunes, merges, distillations, quantisations).
//!
//! While `VersionControl` tracks versions *within* a single model, this
//! module tracks relationships *across* models in the vault.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;

// ── Types ────────────────────────────────────────────────────────────────────

/// Kind of derivation relationship.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DerivationKind {
    FineTune,
    Merge,
    Distillation,
    Quantization,
    Conversion,
    Prune,
    Custom(String),
}

impl std::fmt::Display for DerivationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DerivationKind::FineTune => write!(f, "fine-tune"),
            DerivationKind::Merge => write!(f, "merge"),
            DerivationKind::Distillation => write!(f, "distillation"),
            DerivationKind::Quantization => write!(f, "quantization"),
            DerivationKind::Conversion => write!(f, "conversion"),
            DerivationKind::Prune => write!(f, "prune"),
            DerivationKind::Custom(s) => write!(f, "{s}"),
        }
    }
}

/// An edge in the lineage graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEdge {
    /// Source model(s). Multiple for merge.
    pub parents: Vec<String>,
    /// Derived model.
    pub child: String,
    /// Relationship type.
    pub kind: DerivationKind,
    /// Free-form notes (training params, merge recipe, …).
    pub notes: BTreeMap<String, String>,
    /// When this edge was recorded.
    pub created_at: String,
}

/// Persisted graph data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LineageGraphData {
    pub edges: Vec<LineageEdge>,
}

// ── Graph ────────────────────────────────────────────────────────────────────

/// Cross-model lineage graph stored alongside the vault.
pub struct LineageGraph {
    path: PathBuf,
    data: LineageGraphData,
}

impl LineageGraph {
    const FILE_NAME: &'static str = "lineage_graph.json";

    pub fn new(vault_path: &Path) -> Result<Self> {
        let path = vault_path.join(Self::FILE_NAME);
        let data = if path.exists() {
            let contents = fs::read_to_string(&path)?;
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            LineageGraphData::default()
        };
        Ok(Self { path, data })
    }

    fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.path, json)?;
        crate::permissions::restrict_file(&self.path)?;
        Ok(())
    }

    /// Record a derivation edge.
    pub fn add_edge(&mut self, edge: LineageEdge) -> Result<()> {
        self.data.edges.push(edge);
        self.save()
    }

    /// All edges.
    pub fn edges(&self) -> &[LineageEdge] {
        &self.data.edges
    }

    /// All unique model names (nodes).
    pub fn nodes(&self) -> BTreeSet<String> {
        let mut nodes = BTreeSet::new();
        for edge in &self.data.edges {
            for p in &edge.parents {
                nodes.insert(p.clone());
            }
            nodes.insert(edge.child.clone());
        }
        nodes
    }

    /// Find direct parents of a model.
    pub fn parents(&self, model: &str) -> Vec<&LineageEdge> {
        self.data
            .edges
            .iter()
            .filter(|e| e.child == model)
            .collect()
    }

    /// Find direct children of a model.
    pub fn children(&self, model: &str) -> Vec<&LineageEdge> {
        self.data
            .edges
            .iter()
            .filter(|e| e.parents.contains(&model.to_string()))
            .collect()
    }

    /// Walk ancestors (BFS) — returns all models from which `model` derives.
    pub fn ancestors(&self, model: &str) -> Vec<String> {
        let mut visited = BTreeSet::new();
        let mut queue = vec![model.to_string()];
        let mut result = Vec::new();

        while let Some(current) = queue.pop() {
            for edge in self.parents(&current) {
                for parent in &edge.parents {
                    if visited.insert(parent.clone()) {
                        result.push(parent.clone());
                        queue.push(parent.clone());
                    }
                }
            }
        }
        result
    }

    /// Walk descendants (BFS) — returns all models derived from `model`.
    pub fn descendants(&self, model: &str) -> Vec<String> {
        let mut visited = BTreeSet::new();
        let mut queue = vec![model.to_string()];
        let mut result = Vec::new();

        while let Some(current) = queue.pop() {
            for edge in self.children(&current) {
                if visited.insert(edge.child.clone()) {
                    result.push(edge.child.clone());
                    queue.push(edge.child.clone());
                }
            }
        }
        result
    }

    /// Render a simple text representation of the graph.
    pub fn display(&self) -> String {
        let mut out = String::new();
        if self.data.edges.is_empty() {
            out.push_str("No lineage edges recorded.\n");
            return out;
        }

        out.push_str("Cross-model lineage graph:\n");
        out.push_str(&format!("{}\n", "─".repeat(50)));

        for edge in &self.data.edges {
            let parents = edge.parents.join(" + ");
            out.push_str(&format!(
                "  {} --[{}]--> {}\n",
                parents, edge.kind, edge.child
            ));
        }
        out
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_edge(parents: &[&str], child: &str, kind: DerivationKind) -> LineageEdge {
        LineageEdge {
            parents: parents.iter().map(|s| s.to_string()).collect(),
            child: child.to_string(),
            kind,
            notes: BTreeMap::new(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_add_and_query() {
        let dir = tempfile::tempdir().unwrap();
        let mut graph = LineageGraph::new(dir.path()).unwrap();

        graph
            .add_edge(make_edge(
                &["llama-base"],
                "llama-ft",
                DerivationKind::FineTune,
            ))
            .unwrap();
        graph
            .add_edge(make_edge(
                &["llama-ft"],
                "llama-q4",
                DerivationKind::Quantization,
            ))
            .unwrap();

        assert_eq!(graph.nodes().len(), 3);
        assert_eq!(graph.parents("llama-ft").len(), 1);
        assert_eq!(graph.children("llama-ft").len(), 1);
    }

    #[test]
    fn test_ancestors_descendants() {
        let dir = tempfile::tempdir().unwrap();
        let mut graph = LineageGraph::new(dir.path()).unwrap();

        graph
            .add_edge(make_edge(&["base"], "ft1", DerivationKind::FineTune))
            .unwrap();
        graph
            .add_edge(make_edge(&["ft1"], "ft2", DerivationKind::FineTune))
            .unwrap();
        graph
            .add_edge(make_edge(&["ft2"], "q4", DerivationKind::Quantization))
            .unwrap();

        let ancestors = graph.ancestors("q4");
        assert!(ancestors.contains(&"ft2".to_string()));
        assert!(ancestors.contains(&"ft1".to_string()));
        assert!(ancestors.contains(&"base".to_string()));

        let descendants = graph.descendants("base");
        assert!(descendants.contains(&"ft1".to_string()));
        assert!(descendants.contains(&"ft2".to_string()));
        assert!(descendants.contains(&"q4".to_string()));
    }

    #[test]
    fn test_merge_parents() {
        let dir = tempfile::tempdir().unwrap();
        let mut graph = LineageGraph::new(dir.path()).unwrap();

        graph
            .add_edge(make_edge(
                &["model-a", "model-b"],
                "merged",
                DerivationKind::Merge,
            ))
            .unwrap();

        assert_eq!(graph.parents("merged").len(), 1);
        assert_eq!(graph.parents("merged")[0].parents.len(), 2);
    }

    #[test]
    fn test_persistence() {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut g = LineageGraph::new(dir.path()).unwrap();
            g.add_edge(make_edge(&["a"], "b", DerivationKind::Distillation))
                .unwrap();
        }
        let g2 = LineageGraph::new(dir.path()).unwrap();
        assert_eq!(g2.edges().len(), 1);
    }

    #[test]
    fn test_display() {
        let dir = tempfile::tempdir().unwrap();
        let mut graph = LineageGraph::new(dir.path()).unwrap();
        graph
            .add_edge(make_edge(&["base"], "child", DerivationKind::FineTune))
            .unwrap();
        let text = graph.display();
        assert!(text.contains("base"));
        assert!(text.contains("child"));
    }
}
