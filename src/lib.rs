//! AI Model Vault - Universal secure vault for AI model formats
//!
//! A cross-platform, XDG-compliant secure storage system for Neural and Neurosymbolic AI models with:
//! - FIPS 140-3 compliant encryption
//! - CVE scanning and compliance
//! - MITRE ATT&CK framework alignment
//! - CMMC 2.0 compliance
//! - Version control with complete checkpoint history
//! - Format conversion capabilities

pub mod audit;
pub mod compliance;
pub mod config;
pub mod crypto;
pub mod error;
pub mod formats;
pub mod model_card;
pub mod rag;
pub mod storage;
pub mod utils;
pub mod vault;
pub mod version;

pub use config::VaultConfig;
pub use error::{Result, VaultError};
pub use model_card::{
    CaveatsAndRecommendations, EnvironmentalImpact, EthicalConsiderations, Evaluation, IntendedUse,
    Metric, ModelCard, ModelDetails, TrainingData,
};
pub use rag::{
    Database, Document, DocumentStore, InMemoryDatabase, KnowledgeBase, KnowledgeBaseConfig,
    MCPServer, MCPTool, RetrievalCache, Rule, RuleAction, RuleCondition, RuleEngine, ToolContext,
    ToolExecutor, ToolResult,
};
pub use utils::{
    CompressionAnalyzer, CompressionReport, ModelAnalysis, ModelAnalyzer, ModelArchive,
    ModelDeduplicator, ModelExporter, PruningInfo, PruningMethod, QuantizationInfo,
    QuantizationSavings, RetrievalOptimizer,
};
pub use vault::Vault;
pub use version::{ModelVersion, VersionControl};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
