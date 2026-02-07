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
    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
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
        serde_yaml::to_string(self)
            .map_err(|e| crate::error::VaultError::SerializationError(e.to_string()))
    }

    /// Convert to Markdown format (HuggingFace style)
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        // Header
        md.push_str(&format!("# Model Card: {}\n\n", self.model_details.name));

        // Model Details
        md.push_str("## Model Details\n\n");
        md.push_str(&format!("- **Name**: {}\n", self.model_details.name));
        md.push_str(&format!("- **Version**: {}\n", self.model_details.version));
        md.push_str(&format!("- **Type**: {}\n", self.model_details.model_type));
        md.push_str(&format!(
            "- **Architecture**: {}\n",
            self.model_details.architecture
        ));
        md.push_str(&format!("- **Size**: {}\n", self.model_details.size));
        md.push_str(&format!(
            "- **Framework**: {}\n",
            self.model_details.framework
        ));
        md.push_str(&format!("- **Format**: {}\n", self.model_details.format));

        if let Some(license) = &self.model_details.license {
            md.push_str(&format!("- **License**: {}\n", license));
        }

        if !self.model_details.developers.is_empty() {
            md.push_str(&format!(
                "- **Developers**: {}\n",
                self.model_details.developers.join(", ")
            ));
        }

        if let Some(repo) = &self.model_details.repository {
            md.push_str(&format!("- **Repository**: {}\n", repo));
        }

        md.push_str(&format!("\n{}\n\n", self.model_details.description));

        // Intended Use
        md.push_str("## Intended Use\n\n");
        md.push_str("### Primary Uses\n");
        for use_case in &self.intended_use.primary_uses {
            md.push_str(&format!("- {}\n", use_case));
        }
        md.push_str("\n### Primary Users\n");
        for user in &self.intended_use.primary_users {
            md.push_str(&format!("- {}\n", user));
        }
        md.push_str("\n### Out-of-Scope Uses\n");
        for use_case in &self.intended_use.out_of_scope_uses {
            md.push_str(&format!("- {}\n", use_case));
        }
        md.push('\n');

        // Training Data
        if let Some(training) = &self.training_data {
            md.push_str("## Training Data\n\n");
            md.push_str("### Datasets\n");
            for dataset in &training.datasets {
                md.push_str(&format!("- {}\n", dataset));
            }
            if let Some(size) = &training.size {
                md.push_str(&format!("\n**Dataset Size**: {}\n", size));
            }
            if let Some(languages) = &training.languages {
                md.push_str(&format!("\n**Languages**: {}\n", languages.join(", ")));
            }
            md.push('\n');
        }

        // Evaluation
        if let Some(eval) = &self.evaluation {
            md.push_str("## Evaluation\n\n");
            md.push_str("### Metrics\n");
            for metric in &eval.metrics {
                md.push_str(&format!("- **{}**: {:.4}", metric.name, metric.value));
                if let Some(desc) = &metric.description {
                    md.push_str(&format!(" - {}", desc));
                }
                md.push('\n');
            }

            if let Some(benchmarks) = &eval.benchmarks {
                md.push_str("\n### Benchmark Results\n");
                for (name, score) in benchmarks {
                    md.push_str(&format!("- **{}**: {:.4}\n", name, score));
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
                    md.push_str(&format!("- {}\n", item));
                }
                md.push('\n');
            }

            if let Some(risks) = &ethical.risks {
                md.push_str("### Risk Assessment\n");
                for risk in risks {
                    md.push_str(&format!("- {}\n", risk));
                }
                md.push('\n');
            }

            if let Some(impact) = &ethical.environmental_impact {
                md.push_str("### Environmental Impact\n");
                md.push_str(&format!("- **Hardware**: {}\n", impact.hardware));
                md.push_str(&format!("- **Training Hours**: {:.1}\n", impact.hours));
                if let Some(carbon) = impact.carbon_emitted {
                    md.push_str(&format!("- **Carbon Emitted**: {:.2} kg CO2e\n", carbon));
                }
                md.push('\n');
            }
        }

        // Caveats and Recommendations
        if let Some(caveats) = &self.caveats_and_recommendations {
            md.push_str("## Limitations and Recommendations\n\n");
            md.push_str("### Limitations\n");
            for limitation in &caveats.limitations {
                md.push_str(&format!("- {}\n", limitation));
            }
            md.push_str("\n### Recommendations\n");
            for rec in &caveats.recommendations {
                md.push_str(&format!("- {}\n", rec));
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
        md.push_str(&format!(
            "\n---\n*Model card created: {}*\n",
            self.created_at.format("%Y-%m-%d")
        ));
        md.push_str(&format!(
            "*Last updated: {}*\n",
            self.updated_at.format("%Y-%m-%d")
        ));

        md
    }

    /// Parse from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| crate::error::VaultError::SerializationError(e.to_string()))
    }

    /// Parse from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml)
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
}
