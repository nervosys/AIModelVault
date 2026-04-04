//! AI Model Vault - Universal secure vault for AI model formats
//!
//! A cross-platform, XDG-compliant secure storage system for Neural and Neurosymbolic AI models with:
//! - FIPS 140-3 compliant encryption
//! - CVE scanning and compliance
//! - MITRE ATT&CK framework alignment
//! - CMMC 2.0 compliance
//! - Version control with complete checkpoint history
//! - Format conversion capabilities

#[cfg(feature = "api")]
pub mod api;
pub mod audit;
pub mod benchmark;
pub mod blockchain;
pub mod compliance;
pub mod config;
pub mod conversion;
pub mod crypto;
pub mod diff;
pub mod download;
pub mod error;
pub mod federation;
pub mod formats;
pub mod interop;
pub mod license_scan;
pub mod model_card;
pub mod permissions;
pub mod rag;
pub mod scanning;
pub mod signing;
pub mod storage;
pub mod telemetry;
pub mod traits;
pub mod utils;
pub mod vault;
pub mod version;
#[cfg(feature = "sqlite")]
pub mod version_sqlite;

#[cfg(feature = "python")]
mod python;

pub use config::VaultConfig;
pub use conversion::{
    ConversionOptions, ConversionPipeline, ConversionProgress, ConversionResult, Converter,
    ValidationCheck, ValidationReport,
};
pub use crypto::streaming::{
    decrypt_chunked, encrypt_chunked, is_chunked_format, StreamHeader, DEFAULT_CHUNK_SIZE,
    HEADER_SIZE, STREAM_MAGIC, STREAM_VERSION,
};
pub use error::{ConversionError, CryptoError, Result, StorageError, VaultError};
pub use model_card::{
    CaveatsAndRecommendations, EnvironmentalImpact, EthicalConsiderations, Evaluation, IntendedUse,
    Metric, ModelCard, ModelDetails, TrainingData,
};
pub use rag::{
    Database, Document, DocumentStore, InMemoryDatabase, KnowledgeBase, KnowledgeBaseConfig,
    MCPServer, MCPTool, RetrievalCache, Rule, RuleAction, RuleCondition, RuleEngine, ToolContext,
    ToolExecutor, ToolResult,
};
pub use traits::{
    AimvUri, AsyncBlobStore, AsyncBlobStoreAdapter, AuditLogSubscriber, AuditSink, BlobInfo,
    BlobReceipt, BlobStore, BlobStoreStats, CryptoProvider, EventBus, EventSubscriber,
    MetricsSnapshot, MetricsSubscriber, NullAuditSink, VaultEvent, VaultMetrics, VaultState,
    VersionRepo,
};
pub use utils::{
    CompressionAnalyzer, CompressionReport, ModelAnalysis, ModelAnalyzer, ModelArchive,
    ModelDeduplicator, ModelExporter, PruningInfo, PruningMethod, QuantizationInfo,
    QuantizationSavings, RetrievalOptimizer,
};
pub use vault::{Vault, VaultBuilder, VersionBackend};
pub use version::{ModelVersion, VersionControl};
#[cfg(feature = "sqlite")]
pub use version_sqlite::SqliteVersionRepo;

// Benchmark exports
pub use benchmark::{BenchmarkRecord, BenchmarkResult, BenchmarkStore};

// Diff exports
pub use diff::{DiffSummary, ModelDiff, ModelDiffer, TensorChange, TensorInfo};

// Download exports
pub use download::{ModelDownloader, ModelSource};

// Interop exports
pub use interop::{
    InferenceEngine, LmStudioOptions, OllamaOptions, RegistrationResult,
    register_lm_studio, register_ollama,
};

// License scanning exports
pub use license_scan::{
    DetectedLicense, LicenseClass, LicenseScanReport, LicenseScanner, LicenseSource,
};

// Pickle scanning exports
pub use scanning::{PickleScanner, ScanFinding, ScanReport, Severity};

// Signing exports
pub use signing::{ModelSignature, ModelSigner, SignatureVerification, SigningKeyPair};

// Blockchain audit exports
pub use blockchain::{
    AuditBlock, AuditProof, BlockchainAudit, ChainVerification, MerkleProof, MerkleTree,
    ProofVerification,
};

// Federation exports
pub use federation::{
    ConflictResolution, FederationConfig, FederationManager, FederationStatus, PeerConfig,
    SyncConflict, SyncManifest, SyncResult, VectorClock,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
