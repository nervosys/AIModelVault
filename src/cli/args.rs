//! CLI argument definitions and command structures.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "aim")]
#[command(version = "0.1.0")]
#[command(about = "Universal secure vault for AI model formats (Neural & Neurosymbolic)", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Vault name (uses default if not specified)
    #[arg(short, long)]
    pub vault: Option<String>,

    /// Config file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new vault
    Init {
        /// Vault name
        #[arg(short, long, default_value = "default")]
        name: String,
    },

    /// Store a model in the vault
    Store {
        /// Model name
        name: String,

        /// Path to model file
        path: PathBuf,

        /// Model format (safetensors, gguf, pytorch, onnx, tflite, coreml, tensorrt, mlx, etc.)
        /// Auto-detected from extension if not specified
        #[arg(short, long)]
        format: Option<String>,

        /// Description
        #[arg(short, long)]
        description: Option<String>,

        /// Framework (e.g., pytorch, tensorflow)
        #[arg(long)]
        framework: Option<String>,

        /// Task (e.g., text-generation, image-classification)
        #[arg(long)]
        task: Option<String>,
    },

    /// Retrieve a model from the vault
    Get {
        /// Model name
        name: String,

        /// Output path
        output: PathBuf,

        /// Version number (latest if not specified)
        #[arg(short, long)]
        version: Option<u32>,
    },

    /// List all models in the vault
    List,

    /// Show versions of a model
    Versions {
        /// Model name
        name: String,
    },

    /// Show lineage/history of a model version
    Lineage {
        /// Model name
        name: String,

        /// Version number
        version: u32,
    },

    /// Delete a model version
    Delete {
        /// Model name
        name: String,

        /// Version number
        version: u32,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Show vault statistics
    Stats,

    /// Run compliance checks
    Compliance,

    /// Change vault passphrase
    ChangePassphrase,

    /// Archive models to TAR or ZIP
    Archive {
        /// Model names to archive
        models: Vec<String>,

        /// Output archive path
        output: PathBuf,

        /// Archive format (tar or zip)
        #[arg(short, long, default_value = "tar")]
        format: String,

        /// Version numbers (if not specified, uses latest for each model)
        #[arg(short, long)]
        versions: Option<Vec<u32>>,
    },

    /// Extract models from archive
    Extract {
        /// Archive path
        archive: PathBuf,

        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },

    /// Analyze compression efficiency
    Analyze {
        /// Model name
        name: String,

        /// Version number (latest if not specified)
        #[arg(short, long)]
        version: Option<u32>,
    },

    /// Find duplicate models in vault
    Deduplicate {
        /// Show detailed similarity scores
        #[arg(short, long)]
        detailed: bool,
    },

    /// Export model with metadata
    Export {
        /// Model name
        name: String,

        /// Output directory
        output: PathBuf,

        /// Version number (latest if not specified)
        #[arg(short, long)]
        version: Option<u32>,
    },

    /// Show cache statistics (if caching is enabled)
    Cache,

    /// Convert model between formats
    Convert {
        /// Model name in vault
        name: String,

        /// Target format (safetensors, onnx, gguf, tflite, coreml, etc.)
        #[arg(short = 't', long)]
        to_format: String,

        /// Output file path (optional, defaults to model_name.{extension})
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Version number (latest if not specified)
        #[arg(short, long)]
        version: Option<u32>,

        /// Quantization level for GGUF conversion (q4_0, q4_k_m, q8_0, etc.)
        #[arg(short, long)]
        quantization: Option<String>,
    },

    /// Cloud storage operations
    Cloud {
        #[command(subcommand)]
        command: CloudCommands,
    },

    /// Model card operations
    Card {
        #[command(subcommand)]
        command: CardCommands,
    },

    /// Database operations for RAG knowledge base
    Database {
        #[command(subcommand)]
        command: DatabaseCommands,
    },
}

#[derive(Subcommand)]
pub enum DatabaseCommands {
    /// Initialize a new database
    Init {
        /// Database path
        #[arg(short, long)]
        path: PathBuf,

        /// Database type (sqlite, sled)
        #[arg(short = 't', long, default_value = "sqlite")]
        db_type: String,
    },

    /// Store a document in the database
    Store {
        /// Database path
        #[arg(short, long)]
        path: PathBuf,

        /// Document content (from file or stdin)
        #[arg(short, long)]
        input: PathBuf,

        /// Document ID (optional, generated if not provided)
        #[arg(short, long)]
        id: Option<String>,

        /// Metadata key=value pairs
        #[arg(short, long)]
        metadata: Vec<String>,
    },

    /// Retrieve a document by ID
    Get {
        /// Database path
        #[arg(short, long)]
        path: PathBuf,

        /// Document ID
        id: String,
    },

    /// Search documents
    Search {
        /// Database path
        #[arg(short, long)]
        path: PathBuf,

        /// Search query
        query: String,

        /// Maximum number of results
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,
    },

    /// List all documents
    List {
        /// Database path
        #[arg(short, long)]
        path: PathBuf,
    },

    /// Delete a document
    Delete {
        /// Database path
        #[arg(short, long)]
        path: PathBuf,

        /// Document ID
        id: String,
    },

    /// Export database to JSON
    Export {
        /// Database path
        #[arg(short, long)]
        path: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Import documents from JSON
    Import {
        /// Database path
        #[arg(short, long)]
        path: PathBuf,

        /// Input JSON file
        #[arg(short, long)]
        input: PathBuf,
    },

    /// Show database statistics
    Stats {
        /// Database path
        #[arg(short, long)]
        path: PathBuf,
    },

    /// Build vector index for similarity search
    BuildIndex {
        /// Database path
        #[arg(short, long)]
        path: PathBuf,

        /// Output index path
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Search by vector similarity
    VectorSearch {
        /// Index path
        #[arg(short, long)]
        index: PathBuf,

        /// Query text or embedding file
        #[arg(short, long)]
        query: PathBuf,

        /// Number of results
        #[arg(short = 'n', long, default_value = "5")]
        limit: usize,
    },
}

#[derive(Subcommand)]
pub enum CardCommands {
    /// Create a new model card
    Create {
        /// Model name
        name: String,

        /// Model version
        #[arg(short, long)]
        version: String,

        /// Description
        #[arg(short, long)]
        description: String,

        /// Model type (e.g., "Large Language Model", "Image Classifier")
        #[arg(short = 't', long)]
        model_type: String,

        /// Architecture (e.g., "Transformer", "ResNet-50")
        #[arg(short, long)]
        architecture: String,

        /// Output file (JSON, YAML, or Markdown based on extension)
        #[arg(short, long)]
        output: PathBuf,

        /// Open interactive wizard for additional fields
        #[arg(short, long)]
        interactive: bool,
    },

    /// Show a model card
    Show {
        /// Path to model card file (JSON or YAML)
        path: PathBuf,

        /// Output format (json, yaml, markdown)
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },

    /// Validate a model card
    Validate {
        /// Path to model card file
        path: PathBuf,

        /// Check for required fields
        #[arg(short, long)]
        strict: bool,
    },

    /// Convert model card between formats
    Convert {
        /// Input model card file
        input: PathBuf,

        /// Output file (format determined by extension)
        output: PathBuf,
    },

    /// Generate a template model card
    Template {
        /// Template type (llm, classifier, medical, hiring)
        #[arg(short, long, default_value = "basic")]
        template_type: String,

        /// Output file
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Attach a model card to a vault model
    Attach {
        /// Model name in vault
        model: String,

        /// Version number (latest if not specified)
        #[arg(short, long)]
        version: Option<u32>,

        /// Path to model card file
        card: PathBuf,
    },

    /// Extract model card from a vault model
    Extract {
        /// Model name in vault
        model: String,

        /// Version number (latest if not specified)
        #[arg(short, long)]
        version: Option<u32>,

        /// Output file
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Generate model card from model metadata
    Generate {
        /// Model name in vault
        model: String,

        /// Version number (latest if not specified)
        #[arg(short, long)]
        version: Option<u32>,

        /// Output file
        #[arg(short, long)]
        output: PathBuf,

        /// Include training data section
        #[arg(long)]
        include_training: bool,

        /// Include evaluation section
        #[arg(long)]
        include_evaluation: bool,
    },
}

#[derive(Subcommand)]
pub enum CloudCommands {
    /// Push model to cloud storage
    Push {
        /// Model name
        model: String,

        /// Version number (latest if not specified)
        #[arg(short, long)]
        version: Option<u32>,

        /// Cloud provider (s3, azure, gcs)
        #[arg(short, long)]
        provider: String,

        /// Cloud bucket/container name
        #[arg(short, long)]
        bucket: String,
    },

    /// Pull model from cloud storage
    Pull {
        /// Model name
        model: String,

        /// Cloud provider (s3, azure, gcs)
        #[arg(short, long)]
        provider: String,

        /// Cloud bucket/container name
        #[arg(short, long)]
        bucket: String,

        /// Remote path/key
        #[arg(short = 'k', long)]
        remote_path: String,
    },

    /// List models in cloud storage
    List {
        /// Cloud provider (s3, azure, gcs)
        #[arg(short, long)]
        provider: String,

        /// Cloud bucket/container name
        #[arg(short, long)]
        bucket: String,

        /// Prefix/folder path (optional)
        #[arg(short = 'p', long)]
        prefix: Option<String>,
    },

    /// Configure cloud credentials
    Config {
        /// Cloud provider (s3, azure, gcs)
        #[arg(short, long)]
        provider: String,

        /// Show current configuration
        #[arg(short, long)]
        show: bool,
    },
}
