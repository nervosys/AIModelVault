//! Model Card implementation following industry standards
//!
//! Based on:
//! - Google's Model Cards for Model Reporting (Mitchell et al., 2019)
//! - HuggingFace Model Card specifications
//! - Partnership on AI Model Card standards

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Result;

/// Complete Model Card following industry standards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCard {
    /// Model details section
    pub model_details: ModelDetails,

    /// Intended use section
    pub intended_use: IntendedUse,

    /// Training data information
    pub training_data: Option<TrainingData>,

    /// Evaluation metrics and results
    pub evaluation: Option<Evaluation>,

    /// Ethical considerations
    pub ethical_considerations: Option<EthicalConsiderations>,

    /// Caveats and recommendations
    pub caveats_and_recommendations: Option<CaveatsAndRecommendations>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// Card creation timestamp
    pub created_at: DateTime<Utc>,

    /// Card last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Model details section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDetails {
    /// Model name
    pub name: String,

    /// Model version
    pub version: String,

    /// Model description
    pub description: String,

    /// Model type (e.g., "Large Language Model", "Image Classifier")
    pub model_type: String,

    /// Architecture (e.g., "Transformer", "CNN", "LSTM")
    pub architecture: String,

    /// Model size/parameters (e.g., "7B parameters")
    pub size: String,

    /// Framework used (e.g., "PyTorch", "TensorFlow")
    pub framework: String,

    /// Model format (e.g., "safetensors", "gguf")
    pub format: String,

    /// License information
    pub license: Option<String>,

    /// Citation information
    pub citation: Option<String>,

    /// Model authors/developers
    pub developers: Vec<String>,

    /// Contact information
    pub contact: Option<String>,

    /// Model repository URL
    pub repository: Option<String>,

    /// Paper URL
    pub paper: Option<String>,
}

/// Intended use section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntendedUse {
    /// Primary intended uses
    pub primary_uses: Vec<String>,

    /// Primary intended users
    pub primary_users: Vec<String>,

    /// Out-of-scope uses
    pub out_of_scope_uses: Vec<String>,

    /// Use case examples
    pub use_case_examples: Option<Vec<String>>,
}

/// Training data information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingData {
    /// Dataset name(s)
    pub datasets: Vec<String>,

    /// Data sources
    pub sources: Option<Vec<String>>,

    /// Data collection methods
    pub collection_methods: Option<String>,

    /// Data preprocessing steps
    pub preprocessing: Option<Vec<String>>,

    /// Training data size
    pub size: Option<String>,

    /// Data splits (train/val/test)
    pub splits: Option<HashMap<String, String>>,

    /// Languages (for text models)
    pub languages: Option<Vec<String>>,

    /// Demographic information
    pub demographics: Option<String>,
}

/// Evaluation metrics and results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evaluation {
    /// Evaluation datasets
    pub datasets: Vec<String>,

    /// Metrics used
    pub metrics: Vec<Metric>,

    /// Benchmark results
    pub benchmarks: Option<HashMap<String, f64>>,

    /// Performance by group (fairness metrics)
    pub performance_by_group: Option<HashMap<String, HashMap<String, f64>>>,

    /// Evaluation methodology
    pub methodology: Option<String>,
}

/// Individual metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name (e.g., "Accuracy", "F1 Score", "Perplexity")
    pub name: String,

    /// Metric value
    pub value: f64,

    /// Metric description
    pub description: Option<String>,

    /// Threshold or target value
    pub threshold: Option<f64>,
}

/// Ethical considerations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalConsiderations {
    /// Sensitive data handling
    pub sensitive_data: Option<String>,

    /// Bias considerations
    pub bias: Option<Vec<String>>,

    /// Fairness considerations
    pub fairness: Option<Vec<String>>,

    /// Privacy considerations
    pub privacy: Option<String>,

    /// Environmental impact
    pub environmental_impact: Option<EnvironmentalImpact>,

    /// Human oversight requirements
    pub human_oversight: Option<String>,

    /// Risk assessment
    pub risks: Option<Vec<String>>,

    /// Mitigations
    pub mitigations: Option<Vec<String>>,
}

/// Environmental impact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalImpact {
    /// Hardware used for training
    pub hardware: String,

    /// Hours of training
    pub hours: f64,

    /// Cloud provider (if applicable)
    pub cloud_provider: Option<String>,

    /// Carbon emitted (in kg CO2 equivalent)
    pub carbon_emitted: Option<f64>,

    /// Energy consumed (in kWh)
    pub energy_consumed: Option<f64>,
}

/// Caveats and recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaveatsAndRecommendations {
    /// Model limitations
    pub limitations: Vec<String>,

    /// Known issues
    pub known_issues: Option<Vec<String>>,

    /// Recommendations for use
    pub recommendations: Vec<String>,

    /// Recommendations for further testing
    pub testing_recommendations: Option<Vec<String>>,

    /// Trade-offs
    pub tradeoffs: Option<Vec<String>>,
}

impl ModelCard {
    /// Create a new model card
    pub fn new(model_details: ModelDetails, intended_use: IntendedUse) -> Self {
        let now = Utc::now();
        Self {
            model_details,
            intended_use,
            training_data: None,
            evaluation: None,
            ethical_considerations: None,
            caveats_and_recommendations: None,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Builder pattern for optional sections
    pub fn with_training_data(mut self, training_data: TrainingData) -> Self {
        self.training_data = Some(training_data);
        self
    }

    /// Set the evaluation metrics and results section.
    pub fn with_evaluation(mut self, evaluation: Evaluation) -> Self {
        self.evaluation = Some(evaluation);
        self
    }

    /// Set the ethical considerations section.
    pub fn with_ethical_considerations(mut self, ethical: EthicalConsiderations) -> Self {
        self.ethical_considerations = Some(ethical);
        self
    }

    /// Set the caveats and recommendations section.
    pub fn with_caveats_and_recommendations(mut self, caveats: CaveatsAndRecommendations) -> Self {
        self.caveats_and_recommendations = Some(caveats);
        self
    }

    /// Add a custom metadata key-value pair to the model card.
    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Update the updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| crate::error::VaultError::SerializationError(e.to_string()))
    }

    /// Convert to YAML string
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml_ng::to_string(self)
            .map_err(|e| crate::error::VaultError::SerializationError(e.to_string()))
    }

    /// Convert to Markdown format (HuggingFace style)
    pub fn to_markdown(&self) -> String {
        use std::fmt::Write;
        let mut md = String::with_capacity(2048);

        // Header
        let _ = writeln!(md, "# Model Card: {}\n", self.model_details.name);

        // Model Details
        md.push_str("## Model Details\n\n");
        let _ = writeln!(md, "- **Name**: {}", self.model_details.name);
        let _ = writeln!(md, "- **Version**: {}", self.model_details.version);
        let _ = writeln!(md, "- **Type**: {}", self.model_details.model_type);
        let _ = writeln!(
            md,
            "- **Architecture**: {}",
            self.model_details.architecture
        );
        let _ = writeln!(md, "- **Size**: {}", self.model_details.size);
        let _ = writeln!(md, "- **Framework**: {}", self.model_details.framework);
        let _ = writeln!(md, "- **Format**: {}", self.model_details.format);

        if let Some(license) = &self.model_details.license {
            let _ = writeln!(md, "- **License**: {}", license);
        }

        if !self.model_details.developers.is_empty() {
            let _ = writeln!(
                md,
                "- **Developers**: {}",
                self.model_details.developers.join(", ")
            );
        }

        if let Some(repo) = &self.model_details.repository {
            let _ = writeln!(md, "- **Repository**: {}", repo);
        }

        let _ = write!(md, "\n{}\n\n", self.model_details.description);

        // Intended Use
        md.push_str("## Intended Use\n\n");
        md.push_str("### Primary Uses\n");
        for use_case in &self.intended_use.primary_uses {
            let _ = writeln!(md, "- {}", use_case);
        }
        md.push_str("\n### Primary Users\n");
        for user in &self.intended_use.primary_users {
            let _ = writeln!(md, "- {}", user);
        }
        md.push_str("\n### Out-of-Scope Uses\n");
        for use_case in &self.intended_use.out_of_scope_uses {
            let _ = writeln!(md, "- {}", use_case);
        }
        md.push('\n');

        // Training Data
        if let Some(training) = &self.training_data {
            md.push_str("## Training Data\n\n");
            md.push_str("### Datasets\n");
            for dataset in &training.datasets {
                let _ = writeln!(md, "- {}", dataset);
            }
            if let Some(size) = &training.size {
                let _ = writeln!(md, "\n**Dataset Size**: {}", size);
            }
            if let Some(languages) = &training.languages {
                let _ = writeln!(md, "\n**Languages**: {}", languages.join(", "));
            }
            md.push('\n');
        }

        // Evaluation
        if let Some(eval) = &self.evaluation {
            md.push_str("## Evaluation\n\n");
            md.push_str("### Metrics\n");
            for metric in &eval.metrics {
                let _ = write!(md, "- **{}**: {:.4}", metric.name, metric.value);
                if let Some(desc) = &metric.description {
                    let _ = write!(md, " - {}", desc);
                }
                md.push('\n');
            }

            if let Some(benchmarks) = &eval.benchmarks {
                md.push_str("\n### Benchmark Results\n");
                for (name, score) in benchmarks {
                    let _ = writeln!(md, "- **{}**: {:.4}", name, score);
                }
            }
            md.push('\n');
        }

        // Ethical Considerations
        if let Some(ethical) = &self.ethical_considerations {
            md.push_str("## Ethical Considerations\n\n");

            if let Some(bias) = &ethical.bias {
                md.push_str("### Bias Considerations\n");
                for item in bias {
                    let _ = writeln!(md, "- {}", item);
                }
                md.push('\n');
            }

            if let Some(risks) = &ethical.risks {
                md.push_str("### Risk Assessment\n");
                for risk in risks {
                    let _ = writeln!(md, "- {}", risk);
                }
                md.push('\n');
            }

            if let Some(impact) = &ethical.environmental_impact {
                md.push_str("### Environmental Impact\n");
                let _ = writeln!(md, "- **Hardware**: {}", impact.hardware);
                let _ = writeln!(md, "- **Training Hours**: {:.1}", impact.hours);
                if let Some(carbon) = impact.carbon_emitted {
                    let _ = writeln!(md, "- **Carbon Emitted**: {:.2} kg CO2e", carbon);
                }
                md.push('\n');
            }
        }

        // Caveats and Recommendations
        if let Some(caveats) = &self.caveats_and_recommendations {
            md.push_str("## Limitations and Recommendations\n\n");
            md.push_str("### Limitations\n");
            for limitation in &caveats.limitations {
                let _ = writeln!(md, "- {}", limitation);
            }
            md.push_str("\n### Recommendations\n");
            for rec in &caveats.recommendations {
                let _ = writeln!(md, "- {}", rec);
            }
            md.push('\n');
        }

        // Citation
        if let Some(citation) = &self.model_details.citation {
            md.push_str("## Citation\n\n");
            md.push_str("```bibtex\n");
            md.push_str(citation);
            md.push_str("\n```\n\n");
        }

        // Footer
        let _ = writeln!(
            md,
            "\n---\n*Model card created: {}*",
            self.created_at.format("%Y-%m-%d")
        );
        let _ = writeln!(md, "*Last updated: {}*", self.updated_at.format("%Y-%m-%d"));

        md
    }

    /// Parse from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| crate::error::VaultError::SerializationError(e.to_string()))
    }

    /// Parse from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml_ng::from_str(yaml)
            .map_err(|e| crate::error::VaultError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_card_creation() {
        let details = ModelDetails {
            name: "test-model".to_string(),
            version: "1.0.0".to_string(),
            description: "A test model".to_string(),
            model_type: "Classifier".to_string(),
            architecture: "ResNet".to_string(),
            size: "25M parameters".to_string(),
            framework: "PyTorch".to_string(),
            format: "safetensors".to_string(),
            license: Some("MIT".to_string()),
            citation: None,
            developers: vec!["Test Team".to_string()],
            contact: None,
            repository: None,
            paper: None,
        };

        let intended_use = IntendedUse {
            primary_uses: vec!["Image classification".to_string()],
            primary_users: vec!["Researchers".to_string()],
            out_of_scope_uses: vec!["Medical diagnosis".to_string()],
            use_case_examples: None,
        };

        let card = ModelCard::new(details, intended_use);

        assert_eq!(card.model_details.name, "test-model");
        assert_eq!(card.model_details.version, "1.0.0");
    }

    #[test]
    fn test_model_card_serialization() {
        let details = ModelDetails {
            name: "test-model".to_string(),
            version: "1.0.0".to_string(),
            description: "A test model".to_string(),
            model_type: "Classifier".to_string(),
            architecture: "ResNet".to_string(),
            size: "25M parameters".to_string(),
            framework: "PyTorch".to_string(),
            format: "safetensors".to_string(),
            license: Some("MIT".to_string()),
            citation: None,
            developers: vec!["Test Team".to_string()],
            contact: None,
            repository: None,
            paper: None,
        };

        let intended_use = IntendedUse {
            primary_uses: vec!["Image classification".to_string()],
            primary_users: vec!["Researchers".to_string()],
            out_of_scope_uses: vec!["Medical diagnosis".to_string()],
            use_case_examples: None,
        };

        let card = ModelCard::new(details, intended_use);

        // Test JSON serialization
        let json = card.to_json().unwrap();
        let parsed = ModelCard::from_json(&json).unwrap();
        assert_eq!(parsed.model_details.name, card.model_details.name);

        // Test YAML serialization
        let yaml = card.to_yaml().unwrap();
        let parsed_yaml = ModelCard::from_yaml(&yaml).unwrap();
        assert_eq!(parsed_yaml.model_details.name, card.model_details.name);
    }

    #[test]
    fn test_model_card_markdown() {
        let details = ModelDetails {
            name: "test-model".to_string(),
            version: "1.0.0".to_string(),
            description: "A test model".to_string(),
            model_type: "Classifier".to_string(),
            architecture: "ResNet".to_string(),
            size: "25M parameters".to_string(),
            framework: "PyTorch".to_string(),
            format: "safetensors".to_string(),
            license: Some("MIT".to_string()),
            citation: None,
            developers: vec!["Test Team".to_string()],
            contact: None,
            repository: None,
            paper: None,
        };

        let intended_use = IntendedUse {
            primary_uses: vec!["Image classification".to_string()],
            primary_users: vec!["Researchers".to_string()],
            out_of_scope_uses: vec!["Medical diagnosis".to_string()],
            use_case_examples: None,
        };

        let card = ModelCard::new(details, intended_use);
        let markdown = card.to_markdown();

        assert!(markdown.contains("# Model Card: test-model"));
        assert!(markdown.contains("## Model Details"));
        assert!(markdown.contains("## Intended Use"));
    }

    #[test]
    fn test_model_card_to_json() {
        // Covers line 291
        let details = ModelDetails {
            name: "json-model".to_string(),
            version: "1.0".to_string(),
            description: "JSON test".to_string(),
            model_type: "LLM".to_string(),
            architecture: "Transformer".to_string(),
            size: "7B".to_string(),
            framework: "PyTorch".to_string(),
            format: "safetensors".to_string(),
            license: Some("MIT".to_string()),
            citation: None,
            developers: vec!["author".to_string()],
            contact: None,
            repository: None,
            paper: None,
        };
        let intended_use = IntendedUse {
            primary_uses: vec!["Testing".to_string()],
            primary_users: vec!["Devs".to_string()],
            out_of_scope_uses: vec![],
            use_case_examples: None,
        };
        let card = ModelCard::new(details, intended_use);
        let json = card.to_json().unwrap();
        assert!(json.contains("json-model"));
        assert!(json.contains("Transformer"));
        // Verify it's valid JSON
        let _: serde_json::Value = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_model_card_to_yaml() {
        // Covers line 297
        let details = ModelDetails {
            name: "yaml-model".to_string(),
            version: "2.0".to_string(),
            description: "YAML test".to_string(),
            model_type: "CNN".to_string(),
            architecture: "CNN".to_string(),
            size: "50M".to_string(),
            framework: "TensorFlow".to_string(),
            format: "savedmodel".to_string(),
            license: Some("Apache-2.0".to_string()),
            citation: None,
            developers: vec![],
            contact: None,
            repository: None,
            paper: None,
        };
        let intended_use = IntendedUse {
            primary_uses: vec![],
            primary_users: vec![],
            out_of_scope_uses: vec![],
            use_case_examples: None,
        };
        let card = ModelCard::new(details, intended_use);
        let yaml = card.to_yaml().unwrap();
        assert!(yaml.contains("yaml-model"));
    }

    fn full_model_details() -> ModelDetails {
        ModelDetails {
            name: "full-model".to_string(),
            version: "2.0.0".to_string(),
            description: "A fully loaded model".to_string(),
            model_type: "LLM".to_string(),
            architecture: "Transformer".to_string(),
            size: "7B parameters".to_string(),
            framework: "PyTorch".to_string(),
            format: "safetensors".to_string(),
            license: Some("Apache-2.0".to_string()),
            citation: Some("@article{test2024}".to_string()),
            developers: vec!["Alice".to_string(), "Bob".to_string()],
            contact: Some("alice@example.com".to_string()),
            repository: Some("https://github.com/test/model".to_string()),
            paper: Some("https://arxiv.org/abs/1234".to_string()),
        }
    }

    fn full_intended_use() -> IntendedUse {
        IntendedUse {
            primary_uses: vec!["Text generation".to_string()],
            primary_users: vec!["Researchers".to_string()],
            out_of_scope_uses: vec!["Medical advice".to_string()],
            use_case_examples: Some(vec!["Summarization".to_string()]),
        }
    }

    #[test]
    fn test_builder_with_training_data() {
        let card = ModelCard::new(full_model_details(), full_intended_use()).with_training_data(
            TrainingData {
                datasets: vec!["dataset1".to_string()],
                sources: Some(vec!["web".to_string()]),
                collection_methods: Some("scraping".to_string()),
                preprocessing: Some(vec!["tokenize".to_string()]),
                size: Some("100GB".to_string()),
                splits: None,
                languages: Some(vec!["en".to_string(), "fr".to_string()]),
                demographics: None,
            },
        );
        assert!(card.training_data.is_some());
    }

    #[test]
    fn test_builder_with_evaluation() {
        let card =
            ModelCard::new(full_model_details(), full_intended_use()).with_evaluation(Evaluation {
                datasets: vec!["MMLU".to_string()],
                metrics: vec![Metric {
                    name: "Accuracy".to_string(),
                    value: 0.95,
                    description: Some("Top-1 accuracy".to_string()),
                    threshold: Some(0.90),
                }],
                benchmarks: Some({
                    let mut m = HashMap::new();
                    m.insert("MMLU".to_string(), 0.85);
                    m
                }),
                performance_by_group: None,
                methodology: Some("Standard eval".to_string()),
            });
        assert!(card.evaluation.is_some());
    }

    #[test]
    fn test_builder_with_ethical_considerations() {
        let card = ModelCard::new(full_model_details(), full_intended_use())
            .with_ethical_considerations(EthicalConsiderations {
                sensitive_data: Some("None".to_string()),
                bias: Some(vec!["English-centric".to_string()]),
                fairness: None,
                privacy: None,
                environmental_impact: Some(EnvironmentalImpact {
                    hardware: "8xA100".to_string(),
                    hours: 100.0,
                    cloud_provider: Some("AWS".to_string()),
                    carbon_emitted: Some(42.5),
                    energy_consumed: Some(800.0),
                }),
                human_oversight: None,
                risks: Some(vec!["Hallucination".to_string()]),
                mitigations: None,
            });
        assert!(card.ethical_considerations.is_some());
    }

    #[test]
    fn test_builder_with_caveats() {
        let card = ModelCard::new(full_model_details(), full_intended_use())
            .with_caveats_and_recommendations(CaveatsAndRecommendations {
                limitations: vec!["English only".to_string()],
                known_issues: None,
                recommendations: vec!["Fine-tune for domain".to_string()],
                testing_recommendations: None,
                tradeoffs: None,
            });
        assert!(card.caveats_and_recommendations.is_some());
    }

    #[test]
    fn test_builder_add_metadata() {
        let card = ModelCard::new(full_model_details(), full_intended_use())
            .add_metadata("key1", "value1")
            .add_metadata("key2", "value2");
        assert_eq!(card.metadata.len(), 2);
        assert_eq!(card.metadata.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_touch_updates_timestamp() {
        let mut card = ModelCard::new(full_model_details(), full_intended_use());
        let before = card.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        card.touch();
        assert!(card.updated_at >= before);
    }

    #[test]
    fn test_full_markdown_with_all_sections() {
        let card = ModelCard::new(full_model_details(), full_intended_use())
            .with_training_data(TrainingData {
                datasets: vec!["CommonCrawl".to_string()],
                sources: None,
                collection_methods: None,
                preprocessing: None,
                size: Some("500GB".to_string()),
                splits: None,
                languages: Some(vec!["en".to_string()]),
                demographics: None,
            })
            .with_evaluation(Evaluation {
                datasets: vec!["MMLU".to_string()],
                metrics: vec![
                    Metric {
                        name: "Accuracy".to_string(),
                        value: 0.95,
                        description: Some("Top-1".to_string()),
                        threshold: None,
                    },
                    Metric {
                        name: "F1".to_string(),
                        value: 0.92,
                        description: None,
                        threshold: None,
                    },
                ],
                benchmarks: Some({
                    let mut m = HashMap::new();
                    m.insert("HellaSwag".to_string(), 0.88);
                    m
                }),
                performance_by_group: None,
                methodology: None,
            })
            .with_ethical_considerations(EthicalConsiderations {
                sensitive_data: None,
                bias: Some(vec!["Western bias".to_string()]),
                fairness: None,
                privacy: None,
                environmental_impact: Some(EnvironmentalImpact {
                    hardware: "8xA100".to_string(),
                    hours: 200.0,
                    cloud_provider: None,
                    carbon_emitted: Some(55.0),
                    energy_consumed: None,
                }),
                human_oversight: None,
                risks: Some(vec!["Misinformation".to_string()]),
                mitigations: None,
            })
            .with_caveats_and_recommendations(CaveatsAndRecommendations {
                limitations: vec!["English only".to_string()],
                known_issues: None,
                recommendations: vec!["Fine-tune".to_string()],
                testing_recommendations: None,
                tradeoffs: None,
            });

        let md = card.to_markdown();

        // Header and model details
        assert!(md.contains("# Model Card: full-model"));
        assert!(md.contains("**License**: Apache-2.0"));
        assert!(md.contains("**Developers**: Alice, Bob"));
        assert!(md.contains("**Repository**: https://github.com/test/model"));

        // Training data
        assert!(md.contains("## Training Data"));
        assert!(md.contains("CommonCrawl"));
        assert!(md.contains("**Dataset Size**: 500GB"));
        assert!(md.contains("**Languages**: en"));

        // Evaluation
        assert!(md.contains("## Evaluation"));
        assert!(md.contains("**Accuracy**: 0.9500 - Top-1"));
        assert!(md.contains("**F1**: 0.9200"));
        assert!(md.contains("### Benchmark Results"));
        assert!(md.contains("HellaSwag"));

        // Ethical considerations
        assert!(md.contains("## Ethical Considerations"));
        assert!(md.contains("Western bias"));
        assert!(md.contains("Misinformation"));
        assert!(md.contains("**Hardware**: 8xA100"));
        assert!(md.contains("**Training Hours**: 200.0"));
        assert!(md.contains("**Carbon Emitted**: 55.00 kg CO2e"));

        // Caveats
        assert!(md.contains("## Limitations and Recommendations"));
        assert!(md.contains("English only"));
        assert!(md.contains("Fine-tune"));

        // Citation
        assert!(md.contains("## Citation"));
        assert!(md.contains("@article{test2024}"));

        // Footer
        assert!(md.contains("Model card created:"));
        assert!(md.contains("Last updated:"));
    }
}
