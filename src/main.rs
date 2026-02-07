//! AI Model Vault CLI application
//!
//! Supported formats include:
//! - LLM formats: Safetensors (.safetensors), GGUF (.gguf), PyTorch (.pt/.pth/.bin)
//! - Production formats: TensorRT (.plan), ONNX (.onnx), TFLite (.tflite)
//! - Platform-specific: MLX (.npz), Core ML (.mlmodel), OpenVINO (.xml)
//! - Mobile/Edge: NCNN (.param), MNN (.mnn), RKNN (.rknn)
//! - Legacy: Caffe (.caffemodel), MXNet (.params), Darknet (.weights)
//! - Data formats: HDF5 (.h5/.hdf5), Pickle (.pkl), NumPy (.npy/.npz)

use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::path::PathBuf;

use ai_model_vault::compliance::ComplianceChecker;
use ai_model_vault::formats::{ModelFormat, ModelMetadata};
use ai_model_vault::utils::{
    CompressionAnalyzer, ModelAnalyzer, ModelArchive, ModelDeduplicator, ModelExporter,
};
use ai_model_vault::{Result, Vault, VaultConfig, VaultError};

#[derive(Parser)]
#[command(name = "aim")]
#[command(version = "0.1.0")]
#[command(about = "Universal secure vault for AI model formats (Neural & Neurosymbolic)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Vault name (uses default if not specified)
    #[arg(short, long)]
    vault: Option<String>,

    /// Config file path
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
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
enum DatabaseCommands {
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
enum CardCommands {
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
enum CloudCommands {
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

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Load or create config
    let config = if let Some(config_path) = cli.config {
        let contents = std::fs::read_to_string(config_path)?;
        serde_yaml::from_str(&contents)?
    } else {
        VaultConfig::new()?
    };

    match cli.command {
        Commands::Init { name } => {
            println!("Initializing vault: {}", name);
            let vault = Vault::new(Some(config))?;
            println!("✓ Vault '{}' initialized successfully", name);
            println!("Location: {:?}", vault.get_config().dirs.vault_dir);
        }

        Commands::Store {
            name,
            path,
            format,
            description,
            framework,
            task,
        } => {
            // Read model file
            let data = std::fs::read(&path)?;
            println!("Read {} bytes from {:?}", data.len(), path);

            // Detect format
            let model_format = if let Some(fmt) = format {
                match fmt.to_lowercase().as_str() {
                    // LLM formats
                    "safetensors" => ModelFormat::Safetensors,
                    "gguf" => ModelFormat::GGUF,
                    "pytorch" | "pt" | "torch" => ModelFormat::PyTorch,
                    "tensorrt" | "trt" => ModelFormat::TensorRT,
                    "onnx" => ModelFormat::ONNX,
                    "mlx" => ModelFormat::MLX,
                    "coreml" | "mlmodel" => ModelFormat::CoreML,
                    "torchscript" => ModelFormat::TorchScript,
                    "tflite" | "tensorflow-lite" => ModelFormat::TFLite,
                    // General DL formats
                    "tensorflow" | "tf" | "savedmodel" => ModelFormat::TensorFlow,
                    "keras" | "h5" => ModelFormat::Keras,
                    "openvino" => ModelFormat::OpenVINO,
                    "tvm" => ModelFormat::TVM,
                    "ncnn" => ModelFormat::NCNN,
                    "mnn" => ModelFormat::MNN,
                    "rknn" => ModelFormat::RKNN,
                    // Legacy formats
                    "caffe" => ModelFormat::Caffe,
                    "mxnet" => ModelFormat::MXNet,
                    "darknet" => ModelFormat::Darknet,
                    // Data formats
                    "hdf5" => ModelFormat::HDF5,
                    "pickle" | "pkl" => ModelFormat::Pickle,
                    "numpy" | "npy" => ModelFormat::NumPy,
                    _ => ModelFormat::Custom(fmt),
                }
            } else {
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("bin");
                ModelFormat::from_extension(ext)
            };

            // Create metadata
            let mut metadata = ModelMetadata::new(name.clone(), model_format);
            if let Some(desc) = description {
                metadata = metadata.with_description(desc);
            }
            if let Some(fw) = framework {
                metadata = metadata.with_framework(fw);
            }
            if let Some(t) = task {
                metadata = metadata.with_task(t);
            }

            // Get passphrase
            let passphrase = prompt_passphrase("Enter vault passphrase: ")?;

            // Store model
            let mut vault = Vault::new(Some(config))?;
            vault.unlock(passphrase)?;
            let version = vault.store_model(&name, data, metadata, None)?;

            println!("✓ Model '{}' stored successfully", name);
            println!("  Version: {}", version.version);
            println!("  Checkpoint ID: {}", version.checkpoint_id);
            println!("  Original size: {} bytes", version.size_bytes);
            println!("  Compressed size: {} bytes", version.compressed_size_bytes);
            println!(
                "  Compression ratio: {:.1}%",
                (1.0 - version.compressed_size_bytes as f64 / version.size_bytes as f64) * 100.0
            );
        }

        Commands::Get {
            name,
            output,
            version,
        } => {
            let passphrase = prompt_passphrase("Enter vault passphrase: ")?;

            let mut vault = Vault::new(Some(config))?;
            vault.unlock(passphrase)?;

            let data = vault.get_model(&name, version)?;
            std::fs::write(&output, &data)?;

            println!("✓ Model '{}' retrieved successfully", name);
            println!("  Written to: {:?}", output);
            println!("  Size: {} bytes", data.len());
        }

        Commands::List => {
            let passphrase = prompt_passphrase("Enter vault passphrase: ")?;

            let mut vault = Vault::new(Some(config))?;
            vault.unlock(passphrase)?;

            let models = vault.list_models();

            if models.is_empty() {
                println!("No models in vault");
            } else {
                println!("Models in vault:");
                for model in models {
                    let versions = vault.list_versions(&model);
                    println!("  {} ({} versions)", model, versions.len());
                }
            }
        }

        Commands::Versions { name } => {
            let vault = Vault::new(Some(config))?;
            let versions = vault.list_versions(&name);

            if versions.is_empty() {
                println!("No versions found for model '{}'", name);
            } else {
                println!("Versions of '{}':", name);
                for v in versions {
                    println!(
                        "  v{} - {} - {} bytes ({})",
                        v.version,
                        v.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                        v.size_bytes,
                        v.format
                    );
                }
            }
        }

        Commands::Lineage { name, version } => {
            let vault = Vault::new(Some(config))?;
            let lineage = vault.get_lineage(&name, version);

            if lineage.is_empty() {
                println!("Version {} not found for model '{}'", version, name);
            } else {
                println!("Lineage for '{}' v{}:", name, version);
                for (i, v) in lineage.iter().enumerate() {
                    println!(
                        "  {}v{} - {} - {}",
                        "  ".repeat(i),
                        v.version,
                        v.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                        v.checkpoint_id
                    );
                }
            }
        }

        Commands::Delete {
            name,
            version,
            force,
        } => {
            if !force {
                print!(
                    "Are you sure you want to delete '{}' v{}? [y/N]: ",
                    name, version
                );
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled");
                    return Ok(());
                }
            }

            let passphrase = prompt_passphrase("Enter vault passphrase: ")?;

            let mut vault = Vault::new(Some(config))?;
            vault.unlock(passphrase)?;

            if vault.delete_version(&name, version)? {
                println!("✓ Deleted '{}' v{}", name, version);
            } else {
                println!("Version not found");
            }
        }

        Commands::Stats => {
            let vault = Vault::new(Some(config))?;
            let stats = vault.get_stats()?;

            println!("Vault Statistics:");
            println!("  Models: {}", stats.model_count);
            println!("  Total versions: {}", stats.total_versions);
            println!(
                "  Total size: {} bytes ({:.2} MB)",
                stats.total_size_bytes,
                stats.total_size_bytes as f64 / 1_048_576.0
            );
            println!("  Files: {}", stats.file_count);
        }

        Commands::Compliance => {
            println!("Running compliance checks...\n");

            let checker = ComplianceChecker::new();
            let status = checker.run_all_checks()?;

            println!("Compliance Status:");
            println!(
                "  FIPS 140-3: {}",
                if status.fips_140_3 {
                    "✓ PASS"
                } else {
                    "✗ FAIL"
                }
            );
            println!(
                "  CVE Scan: {}",
                if status.cve_scan_passed {
                    "✓ PASS"
                } else {
                    "✗ FAIL"
                }
            );
            println!(
                "  MITRE ATT&CK: {}",
                if status.mitre_attack_aligned {
                    "✓ PASS"
                } else {
                    "✗ FAIL"
                }
            );
            println!("  CMMC Level: {}", status.cmmc_level);

            if !status.violations.is_empty() {
                println!("\nViolations:");
                for violation in status.violations {
                    println!(
                        "  [{:?}] {} - {}: {}",
                        violation.severity,
                        violation.standard,
                        violation.control,
                        violation.description
                    );
                }
            } else {
                println!("\n✓ No violations detected");
            }
        }

        Commands::ChangePassphrase => {
            let old_passphrase = prompt_passphrase("Enter current vault passphrase: ")?;
            let new_passphrase = prompt_passphrase("Enter new vault passphrase: ")?;
            let confirm_passphrase = prompt_passphrase("Confirm new vault passphrase: ")?;

            if new_passphrase != confirm_passphrase {
                return Err(VaultError::InvalidInput("Passphrases do not match".to_string()));
            }

            let mut vault = Vault::new(Some(config))?;
            vault.unlock(old_passphrase)?;
            let count = vault.change_passphrase(new_passphrase)?;

            println!("✓ Passphrase changed successfully");
            println!("  Re-encrypted {} model file(s)", count);
        }

        Commands::Archive {
            models,
            output,
            format,
            versions,
        } => {
            let passphrase = prompt_passphrase("Enter vault passphrase: ")?;

            let mut vault = Vault::new(Some(config))?;
            vault.unlock(passphrase)?;

            println!("Archiving {} models...", models.len());

            let mut archive_data = Vec::new();
            for (i, model_name) in models.iter().enumerate() {
                let version = versions.as_ref().and_then(|v| v.get(i).copied());
                let data = vault.get_model(model_name, version)?;
                archive_data.push((model_name.clone(), data));
                println!("  ✓ Loaded '{}'", model_name);
            }

            let total = match format.to_lowercase().as_str() {
                "tar" => ModelArchive::create_tar(archive_data, &output)?,
                "zip" => ModelArchive::create_zip(archive_data, &output)?,
                _ => {
                    eprintln!("Unknown format: {}. Use 'tar' or 'zip'", format);
                    std::process::exit(1);
                }
            };

            println!("✓ Archive created: {:?} ({} bytes)", output, total);
        }

        Commands::Extract { archive, output } => {
            println!("Extracting archive...");

            let ext = archive
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("tar");

            let models = match ext {
                "tar" => ModelArchive::extract_tar(&archive)?,
                "zip" => ModelArchive::extract_zip(&archive)?,
                _ => {
                    eprintln!("Unknown archive format. Expected .tar or .zip");
                    std::process::exit(1);
                }
            };

            std::fs::create_dir_all(&output)?;

            let count = models.len();
            for (name, data) in models {
                let file_path = output.join(&name);
                std::fs::write(&file_path, &data)?;
                println!("  ✓ Extracted '{}' ({} bytes)", name, data.len());
            }

            println!("✓ Extracted {} models to {:?}", count, output);
        }

        Commands::Analyze { name, version } => {
            let passphrase = prompt_passphrase("Enter vault passphrase: ")?;

            let mut vault = Vault::new(Some(config))?;
            vault.unlock(passphrase)?;

            // Get model data
            let data = vault.get_model(&name, version)?;

            // Get version info for compression stats
            let versions = vault.list_versions(&name);
            let version_info = if let Some(v) = version {
                versions.iter().find(|vi| vi.version == v)
            } else {
                versions.last()
            };

            if let Some(vi) = version_info {
                println!("Compression Analysis for '{}' v{}:", name, vi.version);
                println!("  Original size: {} bytes", vi.size_bytes);
                println!("  Compressed size: {} bytes", vi.compressed_size_bytes);

                let ratio =
                    CompressionAnalyzer::compression_ratio(vi.size_bytes, vi.compressed_size_bytes);
                println!("  Compression ratio: {:.2}x", ratio);

                // Try to parse format
                let model_format = ModelFormat::from_extension(&vi.format);
                let report = CompressionAnalyzer::analyze_compression(
                    vi.size_bytes,
                    vi.compressed_size_bytes,
                    &model_format,
                );
                println!("  Space saved: {:.2}%", report.space_saved_percent);
                println!("  Efficiency: {:.2}x expected", report.efficiency);

                // Model analysis
                let metadata = ModelMetadata::new(name.clone(), model_format);
                let analysis = ModelAnalyzer::analyze(&data, &metadata);

                println!("\nModel Analysis:");
                println!(
                    "  Size: {}",
                    ModelAnalyzer::format_size(analysis.size_bytes)
                );
                println!("  Format: {}", analysis.format);
                if let Some(params) = analysis.estimated_parameters {
                    println!(
                        "  Parameters: ~{}",
                        ModelAnalyzer::format_parameters(params)
                    );
                }
                if let Some(fw) = analysis.framework {
                    println!("  Framework: {}", fw);
                }
                if let Some(task) = analysis.task {
                    println!("  Task: {}", task);
                }
            } else {
                println!("Version not found");
            }
        }

        Commands::Deduplicate { detailed } => {
            let passphrase = prompt_passphrase("Enter vault passphrase: ")?;

            let mut vault = Vault::new(Some(config))?;
            vault.unlock(passphrase)?;

            println!("Scanning for duplicate models...");

            let model_names = vault.list_models();
            let mut all_models = Vec::new();

            for name in &model_names {
                let data = vault.get_model(name, None)?;
                all_models.push((name.clone(), data));
            }

            // Create a copy for hash calculation
            let models_for_dedup = all_models
                .iter()
                .map(|(n, d)| (n.clone(), d.clone()))
                .collect();
            let duplicates = ModelDeduplicator::find_duplicates(models_for_dedup);

            if duplicates.is_empty() {
                println!("✓ No duplicate models found");
            } else {
                println!("\nFound {} duplicate groups:", duplicates.len());
                for (i, (_hash, names)) in duplicates.iter().enumerate() {
                    println!("\nGroup {} ({} models):", i + 1, names.len());
                    for n in names {
                        println!("  - {}", n);
                    }

                    if detailed && names.len() == 2 {
                        let data1 = all_models
                            .iter()
                            .find(|(n, _)| n == &names[0])
                            .map(|(_, d)| d.as_slice());
                        let data2 = all_models
                            .iter()
                            .find(|(n, _)| n == &names[1])
                            .map(|(_, d)| d.as_slice());

                        if let (Some(d1), Some(d2)) = (data1, data2) {
                            let similarity = ModelDeduplicator::similarity_score(d1, d2);
                            println!("    Similarity: {:.2}%", similarity * 100.0);
                        }
                    }
                }

                println!("\nYou can save space by removing duplicates.");
            }
        }

        Commands::Export {
            name,
            output,
            version,
        } => {
            let passphrase = prompt_passphrase("Enter vault passphrase: ")?;

            let mut vault = Vault::new(Some(config))?;
            vault.unlock(passphrase)?;

            let data = vault.get_model(&name, version)?;

            // Get metadata
            let versions = vault.list_versions(&name);
            let version_info = if let Some(v) = version {
                versions.iter().find(|vi| vi.version == v)
            } else {
                versions.last()
            };

            if let Some(vi) = version_info {
                let model_format = ModelFormat::from_extension(&vi.format);
                let metadata = ModelMetadata::new(name.clone(), model_format.clone());

                std::fs::create_dir_all(&output)?;

                let _path = ModelExporter::export_with_metadata(data, &metadata, &output)?;

                println!("✓ Exported '{}' v{} to {:?}", name, vi.version, output);
                println!("  Model file: {}.{}", name, model_format.extension());
                println!("  Metadata: {}.meta.json", name);
            } else {
                println!("Version not found");
            }
        }

        Commands::Convert {
            name,
            to_format,
            output,
            version,
            quantization,
        } => {
            handle_convert_command(name, to_format, output, version, quantization, config)?;
        }

        Commands::Cache => {
            use ai_model_vault::utils::RetrievalOptimizer;
            let cache = RetrievalOptimizer::new(1024 * 1024 * 1024); // 1 GB
            let stats = cache.cache_stats();
            println!("Cache Statistics:");
            println!("  Capacity: {:.2} MB", stats.max_size as f64 / 1_048_576.0);
            println!("  Used: {:.2} MB", stats.total_size as f64 / 1_048_576.0);
            println!("  Entries: {}", stats.total_entries);
            println!("  Utilization: {:.1}%", stats.utilization);
            println!("\n💡 The cache is per-process. Use RetrievalOptimizer in your");
            println!("   application code for persistent caching.");
        }

        Commands::Cloud { command } => {
            handle_cloud_command(command, config)?;
        }

        Commands::Card { command } => {
            handle_card_command(command, config)?;
        }

        Commands::Database { command } => {
            handle_database_command(command)?;
        }
    }

    Ok(())
}

fn handle_database_command(command: DatabaseCommands) -> Result<()> {
    #[cfg(not(any(feature = "sqlite", feature = "kv-store")))]
    {
        return Err(VaultError::InvalidInput(
            "Database features not enabled. Rebuild with --features sqlite or --features kv-store"
                .to_string(),
        ));
    }

    #[cfg(any(feature = "sqlite", feature = "kv-store"))]
    {
        use ai_model_vault::rag::Document;
        use std::collections::HashMap;

        match command {
            DatabaseCommands::Init { path, db_type } => {
                println!("🗄️  Initializing database");
                println!("   Path: {}", path.display());
                println!("   Type: {}", db_type);

                match db_type.to_lowercase().as_str() {
                    #[cfg(feature = "sqlite")]
                    "sqlite" => {
                        use ai_model_vault::rag::SQLiteDatabase;
                        let _db = SQLiteDatabase::new(&path)?;
                        // Create documents table with full schema
                        // We need to manually create the table since create_table doesn't support all columns
                        let conn = rusqlite::Connection::open(&path).map_err(|e| {
                            VaultError::StorageError(format!("Failed to open database: {}", e))
                        })?;
                        conn.execute(
                            "CREATE TABLE IF NOT EXISTS documents (
                                id TEXT PRIMARY KEY,
                                content TEXT NOT NULL,
                                metadata TEXT,
                                embedding BLOB,
                                chunk_parent_id TEXT,
                                chunk_index INTEGER,
                                chunk_total INTEGER,
                                chunk_overlap INTEGER,
                                created_at TEXT DEFAULT CURRENT_TIMESTAMP
                            )",
                            [],
                        )
                        .map_err(|e| {
                            VaultError::StorageError(format!("Failed to create table: {}", e))
                        })?;
                        println!("✅ SQLite database initialized successfully!");
                    }
                    #[cfg(feature = "kv-store")]
                    "sled" => {
                        use ai_model_vault::rag::SledDatabase;
                        let _db = SledDatabase::new(&path)?;
                        println!("✅ Sled database initialized successfully!");
                    }
                    _ => {
                        return Err(VaultError::InvalidInput(format!(
                            "Unknown database type: {}. Use 'sqlite' or 'sled'",
                            db_type
                        )));
                    }
                }
            }

            DatabaseCommands::Store {
                path,
                input,
                id,
                metadata,
            } => {
                println!("📝 Storing document");
                println!("   Database: {}", path.display());
                println!("   Input: {}", input.display());

                // Read document content
                let content = std::fs::read_to_string(&input)?;

                // Generate ID if not provided
                let doc_id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

                // Parse metadata
                let mut meta_map = HashMap::new();
                for meta_str in metadata {
                    if let Some((key, value)) = meta_str.split_once('=') {
                        meta_map.insert(key.to_string(), value.to_string());
                    }
                }

                let doc = Document {
                    id: doc_id.clone(),
                    content,
                    metadata: meta_map,
                    embedding: None,
                    chunk_info: None,
                };

                // Detect database type from path
                if path.extension().and_then(|s| s.to_str()) == Some("db")
                    || path.to_str().unwrap_or("").contains("sqlite")
                {
                    #[cfg(feature = "sqlite")]
                    {
                        use ai_model_vault::rag::SQLiteDatabase;
                        let db = SQLiteDatabase::new(&path)?;
                        db.store_document(&doc)?;
                    }
                    #[cfg(not(feature = "sqlite"))]
                    {
                        return Err(VaultError::InvalidInput(
                            "SQLite support not enabled".to_string(),
                        ));
                    }
                } else {
                    #[cfg(feature = "kv-store")]
                    {
                        use ai_model_vault::rag::SledDatabase;
                        let db = SledDatabase::new(&path)?;
                        db.store_document(&doc)?;
                    }
                    #[cfg(not(feature = "kv-store"))]
                    {
                        return Err(VaultError::InvalidInput(
                            "Sled support not enabled".to_string(),
                        ));
                    }
                }

                println!("✅ Document stored successfully!");
                println!("   ID: {}", doc_id);
            }

            DatabaseCommands::Get { path, id } => {
                println!("🔍 Retrieving document");
                println!("   Database: {}", path.display());
                println!("   ID: {}", id);

                // Detect database type
                if path.extension().and_then(|s| s.to_str()) == Some("db") {
                    #[cfg(feature = "sqlite")]
                    {
                        use ai_model_vault::rag::SQLiteDatabase;
                        let db = SQLiteDatabase::new(&path)?;
                        if let Some(doc) = db.get_document(&id)? {
                            println!("\n📄 Document Found:");
                            println!("   ID: {}", doc.id);
                            println!("   Content ({} chars):", doc.content.len());
                            println!("   {}", doc.content);
                            if !doc.metadata.is_empty() {
                                println!("\n   Metadata:");
                                for (key, value) in &doc.metadata {
                                    println!("     {}: {}", key, value);
                                }
                            }
                        } else {
                            println!("❌ Document not found");
                        }
                    }
                    #[cfg(not(feature = "sqlite"))]
                    {
                        return Err(VaultError::InvalidInput(
                            "SQLite support not enabled".to_string(),
                        ));
                    }
                } else {
                    #[cfg(feature = "kv-store")]
                    {
                        use ai_model_vault::rag::SledDatabase;
                        let db = SledDatabase::new(&path)?;
                        if let Some(doc) = db.get_document(&id)? {
                            println!("\n📄 Document Found:");
                            println!("   ID: {}", doc.id);
                            println!("   Content: {}", doc.content);
                        } else {
                            println!("❌ Document not found");
                        }
                    }
                    #[cfg(not(feature = "kv-store"))]
                    {
                        return Err(VaultError::InvalidInput(
                            "Sled support not enabled".to_string(),
                        ));
                    }
                }
            }

            DatabaseCommands::Search { path, query, limit } => {
                println!("🔍 Searching documents");
                println!("   Database: {}", path.display());
                println!("   Query: {}", query);

                if path.extension().and_then(|s| s.to_str()) == Some("db") {
                    #[cfg(feature = "sqlite")]
                    {
                        use ai_model_vault::rag::SQLiteDatabase;
                        let db = SQLiteDatabase::new(&path)?;
                        let results = db.search_documents(&query, limit)?;

                        println!("\n📊 Found {} document(s):", results.len());
                        for (i, doc) in results.iter().enumerate() {
                            println!("\n{}. {} ({})", i + 1, doc.id, doc.content.len());
                            let preview = if doc.content.len() > 100 {
                                format!("{}...", &doc.content[..100])
                            } else {
                                doc.content.clone()
                            };
                            println!("   {}", preview);
                        }
                    }
                    #[cfg(not(feature = "sqlite"))]
                    {
                        return Err(VaultError::InvalidInput(
                            "SQLite support not enabled".to_string(),
                        ));
                    }
                }
            }

            DatabaseCommands::List { path } => {
                println!("📋 Listing documents");
                println!("   Database: {}", path.display());

                if path.extension().and_then(|s| s.to_str()) == Some("db") {
                    #[cfg(feature = "sqlite")]
                    {
                        use ai_model_vault::rag::SQLiteDatabase;
                        let db = SQLiteDatabase::new(&path)?;
                        let results = db.search_documents("", 1000)?; // Get all

                        println!("\n📊 Total documents: {}", results.len());
                        for (i, doc) in results.iter().enumerate() {
                            println!("{}. {} ({} chars)", i + 1, doc.id, doc.content.len());
                        }
                    }
                    #[cfg(not(feature = "sqlite"))]
                    {
                        return Err(VaultError::InvalidInput(
                            "SQLite support not enabled".to_string(),
                        ));
                    }
                } else {
                    #[cfg(feature = "kv-store")]
                    {
                        use ai_model_vault::rag::SledDatabase;
                        let db = SledDatabase::new(&path)?;
                        let ids = db.list_documents()?;

                        println!("\n📊 Total documents: {}", ids.len());
                        for (i, id) in ids.iter().enumerate() {
                            println!("{}. {}", i + 1, id);
                        }
                    }
                    #[cfg(not(feature = "kv-store"))]
                    {
                        return Err(VaultError::InvalidInput(
                            "Sled support not enabled".to_string(),
                        ));
                    }
                }
            }

            DatabaseCommands::Delete { path, id } => {
                println!("🗑️  Deleting document");
                println!("   Database: {}", path.display());
                println!("   ID: {}", id);

                if path.extension().and_then(|s| s.to_str()) == Some("db") {
                    #[cfg(feature = "sqlite")]
                    {
                        use ai_model_vault::rag::{Database, SQLiteDatabase};
                        let mut db = SQLiteDatabase::new(&path)?;
                        db.delete("documents", &id)?;
                        println!("✅ Document deleted successfully!");
                    }
                    #[cfg(not(feature = "sqlite"))]
                    {
                        return Err(VaultError::InvalidInput(
                            "SQLite support not enabled".to_string(),
                        ));
                    }
                } else {
                    #[cfg(feature = "kv-store")]
                    {
                        use ai_model_vault::rag::{Database, SledDatabase};
                        let mut db = SledDatabase::new(&path)?;
                        db.delete("", &id)?; // No table prefix for direct delete
                        println!("✅ Document deleted successfully!");
                    }
                    #[cfg(not(feature = "kv-store"))]
                    {
                        return Err(VaultError::InvalidInput(
                            "Sled support not enabled".to_string(),
                        ));
                    }
                }
            }

            DatabaseCommands::Export { path, output } => {
                println!("📤 Exporting database");
                println!("   Database: {}", path.display());
                println!("   Output: {}", output.display());

                let mut documents = Vec::new();

                if path.extension().and_then(|s| s.to_str()) == Some("db") {
                    #[cfg(feature = "sqlite")]
                    {
                        use ai_model_vault::rag::SQLiteDatabase;
                        let db = SQLiteDatabase::new(&path)?;
                        documents = db.search_documents("", 100000)?; // Export all
                    }
                    #[cfg(not(feature = "sqlite"))]
                    {
                        return Err(VaultError::InvalidInput(
                            "SQLite support not enabled".to_string(),
                        ));
                    }
                }

                let json = serde_json::to_string_pretty(&documents)?;
                std::fs::write(&output, json)?;

                println!("✅ Exported {} documents successfully!", documents.len());
            }

            DatabaseCommands::Import { path, input } => {
                println!("📥 Importing documents");
                println!("   Database: {}", path.display());
                println!("   Input: {}", input.display());

                let json_content = std::fs::read_to_string(&input)?;
                let documents: Vec<Document> = serde_json::from_str(&json_content)?;

                if path.extension().and_then(|s| s.to_str()) == Some("db") {
                    #[cfg(feature = "sqlite")]
                    {
                        use ai_model_vault::rag::SQLiteDatabase;
                        let db = SQLiteDatabase::new(&path)?;
                        for doc in &documents {
                            db.store_document(doc)?;
                        }
                    }
                    #[cfg(not(feature = "sqlite"))]
                    {
                        return Err(VaultError::InvalidInput(
                            "SQLite support not enabled".to_string(),
                        ));
                    }
                } else {
                    #[cfg(feature = "kv-store")]
                    {
                        use ai_model_vault::rag::SledDatabase;
                        let db = SledDatabase::new(&path)?;
                        for doc in &documents {
                            db.store_document(doc)?;
                        }
                    }
                    #[cfg(not(feature = "kv-store"))]
                    {
                        return Err(VaultError::InvalidInput(
                            "Sled support not enabled".to_string(),
                        ));
                    }
                }

                println!("✅ Imported {} documents successfully!", documents.len());
            }

            DatabaseCommands::Stats { path } => {
                println!("📊 Database statistics");
                println!("   Database: {}", path.display());

                if path.extension().and_then(|s| s.to_str()) == Some("db") {
                    #[cfg(feature = "sqlite")]
                    {
                        use ai_model_vault::rag::SQLiteDatabase;
                        let db = SQLiteDatabase::new(&path)?;
                        let all_docs = db.search_documents("", 100000)?;

                        let total_docs = all_docs.len();
                        let total_chars: usize = all_docs.iter().map(|d| d.content.len()).sum();
                        let with_embeddings =
                            all_docs.iter().filter(|d| d.embedding.is_some()).count();

                        println!("\n   Documents: {}", total_docs);
                        println!("   Total characters: {}", total_chars);
                        println!("   With embeddings: {}", with_embeddings);
                        println!(
                            "   Average document size: {} chars",
                            if total_docs > 0 {
                                total_chars / total_docs
                            } else {
                                0
                            }
                        );
                    }
                    #[cfg(not(feature = "sqlite"))]
                    {
                        return Err(VaultError::InvalidInput(
                            "SQLite support not enabled".to_string(),
                        ));
                    }
                } else {
                    #[cfg(feature = "kv-store")]
                    {
                        use ai_model_vault::rag::SledDatabase;
                        let db = SledDatabase::new(&path)?;
                        let ids = db.list_documents()?;
                        println!("\n   Documents: {}", ids.len());
                    }
                    #[cfg(not(feature = "kv-store"))]
                    {
                        return Err(VaultError::InvalidInput(
                            "Sled support not enabled".to_string(),
                        ));
                    }
                }
            }

            DatabaseCommands::BuildIndex { path, output } => {
                println!("🔨 Building vector index");
                println!("   Database: {}", path.display());
                println!("   Output: {}", output.display());

                use ai_model_vault::rag::{SimpleVectorStore, VectorStore};

                // Load documents from database
                let all_docs = if path.extension().and_then(|s| s.to_str()) == Some("db") {
                    #[cfg(feature = "sqlite")]
                    {
                        use ai_model_vault::rag::SQLiteDatabase;
                        let db = SQLiteDatabase::new(&path)?;
                        db.search_documents("", 100000)?
                    }
                    #[cfg(not(feature = "sqlite"))]
                    {
                        return Err(VaultError::InvalidInput(
                            "SQLite support not enabled".to_string(),
                        ));
                    }
                } else {
                    #[cfg(feature = "kv-store")]
                    {
                        use ai_model_vault::rag::SledDatabase;
                        let db = SledDatabase::new(&path)?;
                        let ids = db.list_documents()?;
                        let mut docs = Vec::new();
                        for id in ids {
                            if let Some(doc) = db.get_document(&id)? {
                                docs.push(doc);
                            }
                        }
                        docs
                    }
                    #[cfg(not(feature = "kv-store"))]
                    {
                        return Err(VaultError::InvalidInput(
                            "Sled support not enabled".to_string(),
                        ));
                    }
                };

                // Filter documents with embeddings
                let docs_with_embeddings: Vec<_> = all_docs
                    .into_iter()
                    .filter(|d| d.embedding.is_some())
                    .collect();

                if docs_with_embeddings.is_empty() {
                    println!("⚠️  No documents with embeddings found");
                    println!("   Add embeddings to documents before building index");
                    return Ok(());
                }

                // Build vector store
                let mut store = SimpleVectorStore::new();
                for doc in &docs_with_embeddings {
                    store.store_with_embedding(doc)?;
                }

                // Serialize and save index
                let index_data = serde_json::to_string_pretty(&docs_with_embeddings)?;
                std::fs::write(&output, index_data)?;

                println!("✅ Vector index built successfully!");
                println!("   Documents indexed: {}", docs_with_embeddings.len());
                println!("   Index size: {} bytes", std::fs::metadata(&output)?.len());
            }

            DatabaseCommands::VectorSearch {
                index,
                query,
                limit,
            } => {
                println!("🔍 Vector similarity search");
                println!("   Index: {}", index.display());
                println!("   Query: {}", query.display());

                use ai_model_vault::rag::{SimpleVectorStore, VectorStore};

                // Load index
                let index_data = std::fs::read_to_string(&index)?;
                let documents: Vec<ai_model_vault::rag::Document> =
                    serde_json::from_str(&index_data)?;

                let mut store = SimpleVectorStore::new();
                for doc in &documents {
                    store.store_with_embedding(doc)?;
                }

                // Load query embedding
                // For now, assume the query file contains a JSON array of f32
                let query_data = std::fs::read_to_string(&query)?;
                let query_embedding: Vec<f32> = serde_json::from_str(&query_data)?;

                // Search
                let results = store.search_similar(&query_embedding, limit)?;

                println!("\n📊 Found {} similar document(s):", results.len());
                for (i, (doc_id, similarity)) in results.iter().enumerate() {
                    if let Some(doc) = documents.iter().find(|d| d.id == *doc_id) {
                        println!("\n{}. {} (similarity: {:.4})", i + 1, doc_id, similarity);
                        let preview = if doc.content.len() > 200 {
                            format!("{}...", &doc.content[..200])
                        } else {
                            doc.content.clone()
                        };
                        println!("   {}", preview);
                    }
                }
            }
        }

        Ok(())
    }
}

fn handle_convert_command(
    name: String,
    to_format_str: String,
    output: Option<PathBuf>,
    version: Option<u32>,
    quantization: Option<String>,
    config: VaultConfig,
) -> Result<()> {
    use ai_model_vault::formats::ModelFormat;

    println!("🔄 Converting model format");
    println!("   Model: {}", name);
    println!("   Target format: {}", to_format_str);

    // Parse target format
    let to_format = match to_format_str.to_lowercase().as_str() {
        "safetensors" => ModelFormat::Safetensors,
        "gguf" => ModelFormat::GGUF,
        "pytorch" | "pt" | "torch" => ModelFormat::PyTorch,
        "onnx" => ModelFormat::ONNX,
        "tensorrt" | "trt" => ModelFormat::TensorRT,
        "tflite" | "tensorflow-lite" => ModelFormat::TFLite,
        "coreml" | "mlmodel" => ModelFormat::CoreML,
        "mlx" => ModelFormat::MLX,
        "torchscript" => ModelFormat::TorchScript,
        "openvino" => ModelFormat::OpenVINO,
        "ncnn" => ModelFormat::NCNN,
        "mnn" => ModelFormat::MNN,
        _ => {
            return Err(VaultError::InvalidInput(format!(
                "Unsupported target format: {}. Supported formats: safetensors, gguf, pytorch, onnx, tensorrt, tflite, coreml, mlx, torchscript, openvino, ncnn, mnn",
                to_format_str
            )));
        }
    };

    // Open vault and get model
    let mut vault = Vault::new(Some(config.clone()))?;
    let passphrase = prompt_passphrase("Enter vault passphrase: ")?;
    vault.unlock(passphrase)?;

    // Get version to convert
    let version_num = if let Some(v) = version {
        v
    } else {
        vault
            .list_versions(&name)
            .last()
            .map(|mv| mv.version)
            .ok_or_else(|| {
                VaultError::ModelNotFound(format!("Model '{}' not found or has no versions", name))
            })?
    };

    // Get model data and metadata
    let data = vault.get_model(&name, Some(version_num))?;
    let versions = vault.list_versions(&name);
    let model_version = versions
        .iter()
        .find(|v| v.version == version_num)
        .ok_or_else(|| VaultError::VersionNotFound(version_num, name.clone()))?;

    // Parse source format from string
    let from_format = ModelFormat::from_extension(&model_version.format);
    println!("   Source format: {}", from_format.name());
    println!("   Source size: {} bytes", data.len());

    // Check if conversion is needed
    if from_format == to_format {
        println!("\n⚠️  Model is already in {} format", to_format.name());
        println!("   No conversion needed!");
        return Ok(());
    }

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let extension = to_format.extension();
        PathBuf::from(format!("{}_converted.{}", name, extension))
    });

    println!("   Output: {}", output_path.display());
    if let Some(quant) = &quantization {
        println!("   Quantization: {}", quant);
    }

    // Show conversion path
    println!(
        "\n🔄 Conversion: {} → {}",
        from_format.name(),
        to_format.name()
    );

    // Model format conversion requires external tools (PyTorch, ONNX Runtime,
    // llama.cpp, etc.) because binary format internals are framework-specific.
    // We export the model from the vault, provide the exact conversion commands,
    // and the user can re-import the converted file.
    //
    // This is the standard approach used by all major ML platforms.

    // Export the raw model for the user to convert
    let export_ext = from_format.extension();
    let export_file = format!("{}_v{}.{}", name, version_num, export_ext);

    // Write the model data out for conversion
    std::fs::write(&export_file, &data)?;
    println!("   Exported: {} ({} bytes)", export_file, data.len());

    println!("\n💡 Conversion steps:");

    // Suggest conversion paths
    match (from_format.name().as_str(), to_format.name().as_str()) {
        ("PyTorch", "Safetensors") => {
            println!("   1. Convert with Python:");
            println!("      from safetensors.torch import save_file");
            println!("      import torch");
            println!("      state_dict = torch.load('{}')", export_file);
            println!("      save_file(state_dict, '{}')", output_path.display());
            println!(
                "   2. Store back: aim store {} {} --format safetensors",
                name,
                output_path.display()
            );
        }
        ("PyTorch", "ONNX") => {
            println!("   1. Convert with torch.onnx.export():");
            println!("      import torch");
            println!("      model = torch.load('{}')", export_file);
            println!(
                "      torch.onnx.export(model, dummy_input, '{}')",
                output_path.display()
            );
            println!(
                "   2. Store back: aim store {} {} --format onnx",
                name,
                output_path.display()
            );
        }
        ("Safetensors", "GGUF") => {
            println!("   1. Use llama.cpp convert.py:");
            if let Some(quant) = &quantization {
                println!(
                    "      python convert.py {} --outtype {} --outfile {}",
                    export_file,
                    quant,
                    output_path.display()
                );
            } else {
                println!(
                    "      python convert.py {} --outfile {}",
                    export_file,
                    output_path.display()
                );
            }
            println!(
                "   2. Store back: aim store {}-gguf {} --format gguf",
                name,
                output_path.display()
            );
        }
        ("ONNX", "TensorRT") => {
            println!("   1. Use TensorRT:");
            println!(
                "      trtexec --onnx={} --saveEngine={} --fp16",
                export_file,
                output_path.display()
            );
            println!(
                "   2. Store back: aim store {} {} --format tensorrt",
                name,
                output_path.display()
            );
        }
        ("PyTorch", "TFLite") => {
            println!("   1. Use ai_edge_torch or ONNX→TFLite path");
            println!(
                "   2. Store back: aim store {} {} --format tflite",
                name,
                output_path.display()
            );
        }
        ("PyTorch", "Core ML") => {
            println!("   1. Use coremltools.convert()");
            println!(
                "   2. Store back: aim store {} {} --format coreml",
                name,
                output_path.display()
            );
        }
        (from, to) => {
            println!("   Conversion path for {} → {} not predefined", from, to);
            println!("   Consider using intermediate formats like ONNX");
            println!("   Common path: {} → ONNX → {}", from, to);
        }
    }

    Ok(())
}

fn handle_cloud_command(command: CloudCommands, config: VaultConfig) -> Result<()> {
    match command {
        CloudCommands::Push {
            model,
            version,
            provider,
            bucket,
        } => {
            println!("☁️  Pushing model to cloud storage");
            println!("   Model: {}", model);
            println!("   Provider: {}", provider);
            println!("   Bucket: {}", bucket);

            // Open vault and get model
            let mut vault = Vault::new(Some(config.clone()))?;
            let passphrase = prompt_passphrase("Enter vault passphrase: ")?;
            vault.unlock(passphrase)?;

            // Get version to push
            let version_num = if let Some(v) = version {
                v
            } else {
                vault
                    .list_versions(&model)
                    .last()
                    .map(|mv| mv.version)
                    .ok_or_else(|| {
                        VaultError::ModelNotFound(format!(
                            "Model '{}' not found or has no versions",
                            model
                        ))
                    })?
            };

            // Get model data
            let _data = vault.get_model(&model, Some(version_num))?;
            let versions = vault.list_versions(&model);
            let model_version = versions
                .iter()
                .find(|v| v.version == version_num)
                .ok_or_else(|| VaultError::VersionNotFound(version_num, model.clone()))?;

            // Construct remote path
            let _remote_path = format!("{}/{}/v{}.vault", model, model_version.format, version_num);

            // Push to cloud based on provider
            match provider.to_lowercase().as_str() {
                "s3" => {
                    #[cfg(feature = "s3")]
                    {
                        use ai_model_vault::storage::StorageConfig;
                        println!("📤 Uploading to S3...");
                        let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
                        println!("   Region: {}", region);
                        println!("   Path: {}", _remote_path);
                        println!("   Size: {} bytes", _data.len());

                        let storage_config = StorageConfig::S3 {
                            bucket: bucket.clone(),
                            region,
                            prefix: None,
                        };
                        let rt = tokio::runtime::Runtime::new().map_err(|e| {
                            VaultError::StorageError(format!("Failed to create async runtime: {}", e))
                        })?;
                        rt.block_on(async {
                            let backend = storage_config.create_backend().await?;
                            backend.upload(&_remote_path, &_data).await
                        })?;

                        println!("\n✅ Model pushed to S3 successfully!");
                        println!("   Bucket: {}", bucket);
                        println!("   Key: {}", _remote_path);
                    }
                    #[cfg(not(feature = "s3"))]
                    {
                        println!("⚠️  S3 support not enabled in this build");
                        println!("   To enable: cargo build --release --features s3");
                    }
                }
                "azure" => {
                    #[cfg(feature = "azure")]
                    {
                        use ai_model_vault::storage::StorageConfig;
                        let account = std::env::var("AZURE_STORAGE_ACCOUNT").map_err(|_| {
                            VaultError::ConfigError("AZURE_STORAGE_ACCOUNT env var not set".to_string())
                        })?;
                        println!("📤 Uploading to Azure Blob Storage...");
                        println!("   Container: {}", bucket);
                        println!("   Path: {}", _remote_path);
                        println!("   Size: {} bytes", _data.len());

                        let storage_config = StorageConfig::Azure {
                            account,
                            container: bucket.clone(),
                            prefix: None,
                        };
                        let rt = tokio::runtime::Runtime::new().map_err(|e| {
                            VaultError::StorageError(format!("Failed to create async runtime: {}", e))
                        })?;
                        rt.block_on(async {
                            let backend = storage_config.create_backend().await?;
                            backend.upload(&_remote_path, &_data).await
                        })?;

                        println!("\n✅ Model pushed to Azure successfully!");
                    }
                    #[cfg(not(feature = "azure"))]
                    {
                        println!("⚠️  Azure support not enabled in this build");
                        println!("   To enable: cargo build --release --features azure");
                    }
                }
                "gcs" => {
                    println!("⚠️  GCS support temporarily disabled due to security vulnerabilities");
                    println!("   Use S3 or Azure instead. See SECURITY_AUDIT.md for details.");
                }
                _ => {
                    return Err(VaultError::InvalidInput(format!(
                        "Unsupported provider: {}. Use 's3', 'azure', or 'gcs'",
                        provider
                    )));
                }
            }
        }

        CloudCommands::Pull {
            model,
            provider,
            bucket,
            remote_path,
        } => {
            println!("☁️  Pulling model from cloud storage");
            println!("   Model: {}", model);
            println!("   Provider: {}", provider);
            println!("   Bucket: {}", bucket);
            println!("   Remote path: {}", remote_path);

            match provider.to_lowercase().as_str() {
                "s3" => {
                    #[cfg(feature = "s3")]
                    {
                        use ai_model_vault::storage::StorageConfig;
                        let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
                        println!("📥 Downloading from S3...");

                        let storage_config = StorageConfig::S3 {
                            bucket: bucket.clone(),
                            region,
                            prefix: None,
                        };
                        let rt = tokio::runtime::Runtime::new().map_err(|e| {
                            VaultError::StorageError(format!("Failed to create async runtime: {}", e))
                        })?;
                        let data = rt.block_on(async {
                            let backend = storage_config.create_backend().await?;
                            backend.download(&remote_path).await
                        })?;

                        // Store into vault
                        let passphrase = prompt_passphrase("Enter vault passphrase: ")?;
                        let mut vault = Vault::new(Some(config.clone()))?;
                        vault.unlock(passphrase)?;

                        let model_format = ModelFormat::from_extension(
                            std::path::Path::new(&remote_path)
                                .extension()
                                .and_then(|s| s.to_str())
                                .unwrap_or("bin"),
                        );
                        let metadata = ModelMetadata::new(model.clone(), model_format);
                        let version = vault.store_model(&model, data, metadata, None)?;

                        println!("\n✅ Model pulled and stored successfully!");
                        println!("   Model: {} v{}", model, version.version);
                    }
                    #[cfg(not(feature = "s3"))]
                    {
                        println!("⚠️  S3 support not enabled in this build");
                        println!("   To enable: cargo build --release --features s3");
                    }
                }
                "azure" => {
                    #[cfg(feature = "azure")]
                    {
                        use ai_model_vault::storage::StorageConfig;
                        let account = std::env::var("AZURE_STORAGE_ACCOUNT").map_err(|_| {
                            VaultError::ConfigError("AZURE_STORAGE_ACCOUNT env var not set".to_string())
                        })?;
                        println!("📥 Downloading from Azure Blob Storage...");

                        let storage_config = StorageConfig::Azure {
                            account,
                            container: bucket.clone(),
                            prefix: None,
                        };
                        let rt = tokio::runtime::Runtime::new().map_err(|e| {
                            VaultError::StorageError(format!("Failed to create async runtime: {}", e))
                        })?;
                        let data = rt.block_on(async {
                            let backend = storage_config.create_backend().await?;
                            backend.download(&remote_path).await
                        })?;

                        // Store into vault
                        let passphrase = prompt_passphrase("Enter vault passphrase: ")?;
                        let mut vault = Vault::new(Some(config.clone()))?;
                        vault.unlock(passphrase)?;

                        let model_format = ModelFormat::from_extension(
                            std::path::Path::new(&remote_path)
                                .extension()
                                .and_then(|s| s.to_str())
                                .unwrap_or("bin"),
                        );
                        let metadata = ModelMetadata::new(model.clone(), model_format);
                        let version = vault.store_model(&model, data, metadata, None)?;

                        println!("\n✅ Model pulled and stored successfully!");
                        println!("   Model: {} v{}", model, version.version);
                    }
                    #[cfg(not(feature = "azure"))]
                    {
                        println!("⚠️  Azure support not enabled in this build");
                        println!("   To enable: cargo build --release --features azure");
                    }
                }
                "gcs" => {
                    println!("⚠️  GCS support temporarily disabled due to security vulnerabilities");
                    println!("   Use S3 or Azure instead.");
                }
                _ => {
                    return Err(VaultError::InvalidInput(format!(
                        "Unsupported provider: {}. Use 's3', 'azure', or 'gcs'",
                        provider
                    )));
                }
            }
        }

        CloudCommands::List {
            provider,
            bucket,
            prefix,
        } => {
            println!("☁️  Listing cloud storage contents");
            println!("   Provider: {}", provider);
            println!("   Bucket: {}", bucket);
            if let Some(ref p) = prefix {
                println!("   Prefix: {}", p);
            }

            match provider.to_lowercase().as_str() {
                "s3" => {
                    #[cfg(feature = "s3")]
                    {
                        use ai_model_vault::storage::StorageConfig;
                        let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());

                        let storage_config = StorageConfig::S3 {
                            bucket: bucket.clone(),
                            region,
                            prefix: prefix.clone(),
                        };
                        let rt = tokio::runtime::Runtime::new().map_err(|e| {
                            VaultError::StorageError(format!("Failed to create async runtime: {}", e))
                        })?;
                        let keys = rt.block_on(async {
                            let backend = storage_config.create_backend().await?;
                            backend.list().await
                        })?;

                        println!("\n📋 S3 Bucket '{}' Contents ({} items):", bucket, keys.len());
                        for key in &keys {
                            println!("   {}", key);
                        }
                        if keys.is_empty() {
                            println!("   (empty)");
                        }
                    }
                    #[cfg(not(feature = "s3"))]
                    {
                        println!("⚠️  S3 support not enabled in this build");
                        println!("   To enable: cargo build --release --features s3");
                    }
                }
                "azure" => {
                    #[cfg(feature = "azure")]
                    {
                        use ai_model_vault::storage::StorageConfig;
                        let account = std::env::var("AZURE_STORAGE_ACCOUNT").map_err(|_| {
                            VaultError::ConfigError("AZURE_STORAGE_ACCOUNT env var not set".to_string())
                        })?;

                        let storage_config = StorageConfig::Azure {
                            account,
                            container: bucket.clone(),
                            prefix: prefix.clone(),
                        };
                        let rt = tokio::runtime::Runtime::new().map_err(|e| {
                            VaultError::StorageError(format!("Failed to create async runtime: {}", e))
                        })?;
                        let keys = rt.block_on(async {
                            let backend = storage_config.create_backend().await?;
                            backend.list().await
                        })?;

                        println!("\n📋 Azure Container '{}' Contents ({} items):", bucket, keys.len());
                        for key in &keys {
                            println!("   {}", key);
                        }
                        if keys.is_empty() {
                            println!("   (empty)");
                        }
                    }
                    #[cfg(not(feature = "azure"))]
                    {
                        println!("⚠️  Azure support not enabled in this build");
                        println!("   To enable: cargo build --release --features azure");
                    }
                }
                "gcs" => {
                    println!("⚠️  GCS support temporarily disabled due to security vulnerabilities");
                    println!("   Use S3 or Azure instead.");
                }
                _ => {
                    return Err(VaultError::InvalidInput(format!(
                        "Unsupported provider: {}. Use 's3', 'azure', or 'gcs'",
                        provider
                    )));
                }
            }
        }

        CloudCommands::Config { provider, show } => {
            println!("☁️  Cloud Storage Configuration");
            println!("   Provider: {}", provider);

            if show {
                match provider.to_lowercase().as_str() {
                    "s3" => {
                        println!("\n📝 AWS S3 Configuration:");
                        println!("   Required environment variables:");
                        println!(
                            "   - AWS_ACCESS_KEY_ID: {}",
                            if std::env::var("AWS_ACCESS_KEY_ID").is_ok() {
                                "✅ Set"
                            } else {
                                "❌ Not set"
                            }
                        );
                        println!(
                            "   - AWS_SECRET_ACCESS_KEY: {}",
                            if std::env::var("AWS_SECRET_ACCESS_KEY").is_ok() {
                                "✅ Set"
                            } else {
                                "❌ Not set"
                            }
                        );
                        println!(
                            "   - AWS_REGION (optional): {}",
                            std::env::var("AWS_REGION")
                                .unwrap_or_else(|_| "Not set (defaults to us-east-1)".to_string())
                        );

                        println!("\n💡 To configure:");
                        println!("   export AWS_ACCESS_KEY_ID=your_access_key");
                        println!("   export AWS_SECRET_ACCESS_KEY=your_secret_key");
                        println!("   export AWS_REGION=us-east-1  # optional");
                    }
                    "azure" => {
                        println!("\n📝 Azure Blob Storage Configuration:");
                        println!("   Required environment variables:");
                        println!(
                            "   - AZURE_STORAGE_ACCOUNT: {}",
                            if std::env::var("AZURE_STORAGE_ACCOUNT").is_ok() {
                                "✅ Set"
                            } else {
                                "❌ Not set"
                            }
                        );
                        println!(
                            "   - AZURE_STORAGE_KEY: {}",
                            if std::env::var("AZURE_STORAGE_KEY").is_ok() {
                                "✅ Set"
                            } else {
                                "❌ Not set"
                            }
                        );

                        println!("\n💡 To configure:");
                        println!("   export AZURE_STORAGE_ACCOUNT=your_account_name");
                        println!("   export AZURE_STORAGE_KEY=your_account_key");
                    }
                    "gcs" => {
                        println!("\n📝 Google Cloud Storage Configuration:");
                        println!("   ⚠️  GCS support temporarily disabled due to security vulnerabilities");
                        println!("   Use S3 or Azure instead");
                        println!("\n   For details, see SECURITY_AUDIT.md");
                    }
                    _ => {
                        return Err(VaultError::InvalidInput(format!(
                            "Unsupported provider: {}. Use 's3', 'azure', or 'gcs'",
                            provider
                        )));
                    }
                }
            } else {
                println!("\n💡 Use --show flag to display current configuration");
                println!("   Example: aim cloud config --provider s3 --show");
            }
        }
    }

    Ok(())
}

fn handle_card_command(command: CardCommands, _config: VaultConfig) -> Result<()> {
    use ai_model_vault::model_card::*;

    match command {
        CardCommands::Create {
            name,
            version,
            description,
            model_type,
            architecture,
            output,
            interactive,
        } => {
            println!("📝 Creating model card: {}", name);

            let mut details = ModelDetails {
                name: name.clone(),
                version,
                description: description.clone(),
                model_type,
                architecture,
                size: String::new(),
                framework: String::new(),
                format: String::new(),
                license: None,
                citation: None,
                developers: vec![],
                contact: None,
                repository: None,
                paper: None,
            };

            let mut intended_use = IntendedUse {
                primary_uses: vec![],
                primary_users: vec![],
                out_of_scope_uses: vec![],
                use_case_examples: None,
            };

            if interactive {
                println!("\n🔧 Interactive mode - Fill in additional details");
                println!("(Press Enter to skip optional fields)\n");

                // Size
                print!("Model size (e.g., '7B parameters', '125M'): ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                details.size = input.trim().to_string();

                // Framework
                print!("Framework (e.g., 'PyTorch', 'TensorFlow'): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                details.framework = input.trim().to_string();

                // Format
                print!("Model format (e.g., 'safetensors', 'onnx'): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                details.format = input.trim().to_string();

                // License
                print!("License (e.g., 'MIT', 'Apache-2.0'): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let license_str = input.trim().to_string();
                if !license_str.is_empty() {
                    details.license = Some(license_str);
                }

                // Primary uses
                println!("\nPrimary uses (one per line, empty line to finish):");
                loop {
                    print!("  > ");
                    io::stdout().flush()?;
                    input.clear();
                    io::stdin().read_line(&mut input)?;
                    let use_case = input.trim().to_string();
                    if use_case.is_empty() {
                        break;
                    }
                    intended_use.primary_uses.push(use_case);
                }

                // Out-of-scope uses
                println!("\nOut-of-scope uses (one per line, empty line to finish):");
                loop {
                    print!("  > ");
                    io::stdout().flush()?;
                    input.clear();
                    io::stdin().read_line(&mut input)?;
                    let use_case = input.trim().to_string();
                    if use_case.is_empty() {
                        break;
                    }
                    intended_use.out_of_scope_uses.push(use_case);
                }
            }

            let card = ModelCard::new(details, intended_use);

            // Determine output format from extension
            let ext = output
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("json");
            let content = match ext {
                "yaml" | "yml" => card.to_yaml()?,
                "md" | "markdown" => card.to_markdown(),
                _ => card.to_json()?,
            };

            std::fs::write(&output, content)?;
            println!("✅ Model card created: {}", output.display());
            println!("   Format: {}", ext);
        }

        CardCommands::Show { path, format } => {
            println!("📖 Loading model card: {}", path.display());

            let content = std::fs::read_to_string(&path)?;
            let card = if path.extension().and_then(|e| e.to_str()) == Some("yaml")
                || path.extension().and_then(|e| e.to_str()) == Some("yml")
            {
                ModelCard::from_yaml(&content)?
            } else {
                ModelCard::from_json(&content)?
            };

            let output = match format.as_str() {
                "yaml" | "yml" => card.to_yaml()?,
                "markdown" | "md" => card.to_markdown(),
                _ => card.to_json()?,
            };

            println!("\n{}", output);
        }

        CardCommands::Validate { path, strict } => {
            println!("🔍 Validating model card: {}", path.display());

            let content = std::fs::read_to_string(&path)?;
            let card = if path.extension().and_then(|e| e.to_str()) == Some("yaml")
                || path.extension().and_then(|e| e.to_str()) == Some("yml")
            {
                ModelCard::from_yaml(&content)?
            } else {
                ModelCard::from_json(&content)?
            };

            let mut issues = Vec::new();

            // Check required fields
            if card.model_details.name.is_empty() {
                issues.push("❌ Model name is empty");
            }
            if card.model_details.version.is_empty() {
                issues.push("❌ Model version is empty");
            }
            if card.intended_use.primary_uses.is_empty() {
                issues.push("⚠️  No primary uses specified");
            }

            if strict {
                // Strict mode checks
                if card.model_details.size.is_empty() {
                    issues.push("⚠️  Model size not specified");
                }
                if card.model_details.framework.is_empty() {
                    issues.push("⚠️  Framework not specified");
                }
                if card.training_data.is_none() {
                    issues.push("⚠️  Training data section missing");
                }
                if card.evaluation.is_none() {
                    issues.push("⚠️  Evaluation section missing");
                }
                if card.ethical_considerations.is_none() {
                    issues.push("⚠️  Ethical considerations section missing");
                }
            }

            if issues.is_empty() {
                println!("✅ Model card is valid!");
                println!(
                    "   Model: {} v{}",
                    card.model_details.name, card.model_details.version
                );
                if card.training_data.is_some() {
                    println!("   ✓ Has training data");
                }
                if card.evaluation.is_some() {
                    println!("   ✓ Has evaluation");
                }
                if card.ethical_considerations.is_some() {
                    println!("   ✓ Has ethical considerations");
                }
            } else {
                println!("⚠️  Validation issues found:");
                for issue in issues {
                    println!("   {}", issue);
                }
            }
        }

        CardCommands::Convert { input, output } => {
            println!("🔄 Converting model card");
            println!("   From: {}", input.display());
            println!("   To: {}", output.display());

            let content = std::fs::read_to_string(&input)?;
            let card = if input.extension().and_then(|e| e.to_str()) == Some("yaml")
                || input.extension().and_then(|e| e.to_str()) == Some("yml")
            {
                ModelCard::from_yaml(&content)?
            } else {
                ModelCard::from_json(&content)?
            };

            let ext = output
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("json");
            let output_content = match ext {
                "yaml" | "yml" => card.to_yaml()?,
                "md" | "markdown" => card.to_markdown(),
                _ => card.to_json()?,
            };

            std::fs::write(&output, output_content)?;
            println!("✅ Conversion complete!");
        }

        CardCommands::Template {
            template_type,
            output,
        } => {
            println!("📋 Generating {} template", template_type);

            let (details, intended_use) = match template_type.as_str() {
                "llm" => {
                    let details = ModelDetails {
                        name: "my-llm-model".to_string(),
                        version: "1.0.0".to_string(),
                        description: "Large Language Model for text generation".to_string(),
                        model_type: "Large Language Model".to_string(),
                        architecture: "Transformer".to_string(),
                        size: "7B parameters".to_string(),
                        framework: "PyTorch".to_string(),
                        format: "safetensors".to_string(),
                        license: Some("Apache-2.0".to_string()),
                        citation: None,
                        developers: vec!["Your Team".to_string()],
                        contact: Some("team@example.com".to_string()),
                        repository: None,
                        paper: None,
                    };
                    let intended_use = IntendedUse {
                        primary_uses: vec![
                            "Text generation".to_string(),
                            "Question answering".to_string(),
                        ],
                        primary_users: vec!["Researchers".to_string(), "Developers".to_string()],
                        out_of_scope_uses: vec![
                            "Medical diagnosis".to_string(),
                            "Legal advice".to_string(),
                        ],
                        use_case_examples: None,
                    };
                    (details, intended_use)
                }
                "classifier" => {
                    let details = ModelDetails {
                        name: "my-classifier".to_string(),
                        version: "1.0.0".to_string(),
                        description: "Image classification model".to_string(),
                        model_type: "Image Classifier".to_string(),
                        architecture: "ResNet-50".to_string(),
                        size: "25M parameters".to_string(),
                        framework: "PyTorch".to_string(),
                        format: "onnx".to_string(),
                        license: Some("MIT".to_string()),
                        citation: None,
                        developers: vec!["Your Team".to_string()],
                        contact: None,
                        repository: None,
                        paper: None,
                    };
                    let intended_use = IntendedUse {
                        primary_uses: vec!["Image classification".to_string()],
                        primary_users: vec!["Developers".to_string()],
                        out_of_scope_uses: vec!["Medical diagnosis".to_string()],
                        use_case_examples: None,
                    };
                    (details, intended_use)
                }
                "medical" => {
                    let details = ModelDetails {
                        name: "medical-model".to_string(),
                        version: "1.0.0".to_string(),
                        description: "Medical imaging analysis model".to_string(),
                        model_type: "Medical Image Classifier".to_string(),
                        architecture: "ResNet-50".to_string(),
                        size: "25M parameters".to_string(),
                        framework: "PyTorch".to_string(),
                        format: "onnx".to_string(),
                        license: Some("⚠️ RESEARCH USE ONLY - NOT FOR CLINICAL USE".to_string()),
                        citation: None,
                        developers: vec!["Medical AI Team".to_string()],
                        contact: Some("medical-ai@example.com".to_string()),
                        repository: None,
                        paper: None,
                    };
                    let intended_use = IntendedUse {
                        primary_uses: vec!["Research".to_string()],
                        primary_users: vec!["Researchers".to_string()],
                        out_of_scope_uses: vec![
                            "❌ Clinical diagnosis".to_string(),
                            "❌ Patient treatment decisions".to_string(),
                            "❌ ANY clinical use without FDA approval".to_string(),
                        ],
                        use_case_examples: None,
                    };
                    (details, intended_use)
                }
                "hiring" => {
                    let details = ModelDetails {
                        name: "hiring-model".to_string(),
                        version: "1.0.0".to_string(),
                        description: "Resume screening model".to_string(),
                        model_type: "Text Classifier".to_string(),
                        architecture: "BERT".to_string(),
                        size: "110M parameters".to_string(),
                        framework: "PyTorch".to_string(),
                        format: "safetensors".to_string(),
                        license: Some("Proprietary".to_string()),
                        citation: None,
                        developers: vec!["HR AI Team".to_string()],
                        contact: Some("hr-ai@example.com".to_string()),
                        repository: None,
                        paper: None,
                    };
                    let intended_use = IntendedUse {
                        primary_uses: vec!["Resume screening".to_string()],
                        primary_users: vec!["HR teams".to_string()],
                        out_of_scope_uses: vec![
                            "Final hiring decisions without human review".to_string()
                        ],
                        use_case_examples: Some(vec!["Initial candidate screening".to_string()]),
                    };
                    (details, intended_use)
                }
                _ => {
                    // Basic template
                    let details = ModelDetails {
                        name: "my-model".to_string(),
                        version: "1.0.0".to_string(),
                        description: "Model description".to_string(),
                        model_type: "Model Type".to_string(),
                        architecture: "Architecture".to_string(),
                        size: "Size".to_string(),
                        framework: "Framework".to_string(),
                        format: "Format".to_string(),
                        license: Some("License".to_string()),
                        citation: None,
                        developers: vec!["Team".to_string()],
                        contact: None,
                        repository: None,
                        paper: None,
                    };
                    let intended_use = IntendedUse {
                        primary_uses: vec!["Primary use".to_string()],
                        primary_users: vec!["Primary users".to_string()],
                        out_of_scope_uses: vec!["Out of scope use".to_string()],
                        use_case_examples: None,
                    };
                    (details, intended_use)
                }
            };

            let card = ModelCard::new(details, intended_use);

            let ext = output
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("json");
            let content = match ext {
                "yaml" | "yml" => card.to_yaml()?,
                "md" | "markdown" => card.to_markdown(),
                _ => card.to_json()?,
            };

            std::fs::write(&output, content)?;
            println!("✅ Template created: {}", output.display());
            println!("   Type: {}", template_type);
            println!("\n💡 Edit the file to customize your model card");
        }

        CardCommands::Attach {
            model,
            version,
            card,
        } => {
            println!("📎 Attaching model card to vault model");
            println!("   Model: {}", model);
            println!("   Card: {}", card.display());

            // Read card
            let card_content = std::fs::read_to_string(&card)?;
            let model_card = if card.extension().and_then(|e| e.to_str()) == Some("yaml")
                || card.extension().and_then(|e| e.to_str()) == Some("yml")
            {
                ModelCard::from_yaml(&card_content)?
            } else {
                ModelCard::from_json(&card_content)?
            };

            // Convert to JSON for storage
            let card_json = model_card.to_json()?;

            // Open vault
            let mut vault = Vault::new(Some(_config.clone()))?;

            // Get the specified version or latest
            let version_num = if let Some(v) = version {
                v
            } else {
                vault
                    .list_versions(&model)
                    .last()
                    .map(|mv| mv.version)
                    .ok_or_else(|| {
                        VaultError::ModelNotFound(format!(
                            "Model '{}' not found or has no versions",
                            model
                        ))
                    })?
            };

            // Update metadata with model card
            vault.update_version_metadata(&model, version_num, "model_card", card_json)?;

            println!("✅ Model card attached successfully!");
            println!("   Model: {} v{}", model, version_num);
        }

        CardCommands::Extract {
            model,
            version,
            output,
        } => {
            println!("📤 Extracting model card from vault model");
            println!("   Model: {}", model);
            println!("   Output: {}", output.display());

            // Open vault
            let vault = Vault::new(Some(_config.clone()))?;

            // Get the specified version or latest
            let version_num = if let Some(v) = version {
                v
            } else {
                vault
                    .list_versions(&model)
                    .last()
                    .map(|mv| mv.version)
                    .ok_or_else(|| {
                        VaultError::ModelNotFound(format!(
                            "Model '{}' not found or has no versions",
                            model
                        ))
                    })?
            };

            // Get model card from metadata
            let card_json = vault
                .get_version_metadata(&model, version_num, "model_card")
                .ok_or_else(|| {
                    VaultError::ModelNotFound(format!(
                        "Model '{}' v{} does not have an attached model card",
                        model, version_num
                    ))
                })?;

            // Parse and convert to desired format
            let model_card = ModelCard::from_json(&card_json)?;

            let ext = output
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("json");
            let output_content = match ext {
                "yaml" | "yml" => model_card.to_yaml()?,
                "md" | "markdown" => model_card.to_markdown(),
                _ => model_card.to_json()?,
            };

            std::fs::write(&output, output_content)?;
            println!("✅ Model card extracted successfully!");
            println!("   Model: {} v{}", model, version_num);
            println!("   Output: {}", output.display());
        }

        CardCommands::Generate {
            model,
            version,
            output,
            include_training,
            include_evaluation,
        } => {
            println!("🤖 Generating model card from metadata");
            println!("   Model: {}", model);
            println!("   Output: {}", output.display());

            // Open vault
            let vault = Vault::new(Some(_config.clone()))?;

            // Get the specified version or latest
            let version_num = if let Some(v) = version {
                v
            } else {
                vault
                    .list_versions(&model)
                    .last()
                    .map(|mv| mv.version)
                    .ok_or_else(|| {
                        VaultError::ModelNotFound(format!(
                            "Model '{}' not found or has no versions",
                            model
                        ))
                    })?
            };

            // Get model version info
            let versions = vault.list_versions(&model);
            let model_version = versions
                .iter()
                .find(|v| v.version == version_num)
                .ok_or_else(|| VaultError::VersionNotFound(version_num, model.to_string()))?;

            // Extract metadata
            let description = model_version
                .metadata
                .get("description")
                .cloned()
                .unwrap_or_else(|| format!("Model {}", model));

            let framework = model_version
                .metadata
                .get("framework")
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());

            let task = model_version
                .metadata
                .get("task")
                .cloned()
                .unwrap_or_else(|| "General".to_string());

            // Format size
            let size_str = if model_version.size_bytes > 1_000_000_000 {
                format!(
                    "{:.2} GB",
                    model_version.size_bytes as f64 / 1_000_000_000.0
                )
            } else if model_version.size_bytes > 1_000_000 {
                format!("{:.2} MB", model_version.size_bytes as f64 / 1_000_000.0)
            } else {
                format!("{:.2} KB", model_version.size_bytes as f64 / 1_000.0)
            };

            // Create model details
            let details = ModelDetails {
                name: model.clone(),
                version: version_num.to_string(),
                description,
                model_type: task.clone(),
                architecture: model_version
                    .metadata
                    .get("architecture")
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string()),
                size: size_str,
                framework,
                format: model_version.format.clone(),
                license: model_version.metadata.get("license").cloned(),
                citation: model_version.metadata.get("citation").cloned(),
                developers: vec!["Vault User".to_string()],
                contact: model_version.metadata.get("contact").cloned(),
                repository: model_version.metadata.get("repository").cloned(),
                paper: model_version.metadata.get("paper").cloned(),
            };

            // Create intended use
            let intended_use = IntendedUse {
                primary_uses: vec![task],
                primary_users: vec!["Researchers".to_string(), "Developers".to_string()],
                out_of_scope_uses: vec!["Production use without validation".to_string()],
                use_case_examples: None,
            };

            // Create basic model card
            let mut card = ModelCard::new(details, intended_use);

            // Add training data if requested
            if include_training {
                let training = TrainingData {
                    datasets: vec!["Unknown - please update".to_string()],
                    sources: None,
                    collection_methods: None,
                    preprocessing: None,
                    size: None,
                    splits: None,
                    languages: None,
                    demographics: None,
                };
                card = card.with_training_data(training);
            }

            // Add evaluation if requested
            if include_evaluation {
                let evaluation = Evaluation {
                    datasets: vec!["Unknown - please update".to_string()],
                    metrics: vec![],
                    benchmarks: None,
                    performance_by_group: None,
                    methodology: None,
                };
                card = card.with_evaluation(evaluation);
            }

            // Add vault-specific metadata as custom field
            card.metadata.insert(
                "vault_info".to_string(),
                format!(
                    "Generated from vault model '{}' v{} on {}",
                    model,
                    version_num,
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
                ),
            );
            card.metadata.insert(
                "original_size".to_string(),
                model_version.size_bytes.to_string(),
            );
            card.metadata.insert(
                "compressed_size".to_string(),
                model_version.compressed_size_bytes.to_string(),
            );
            card.metadata.insert(
                "checksum_sha256".to_string(),
                model_version.checksum_sha256.clone(),
            );

            // Convert to desired format
            let ext = output
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("json");
            let output_content = match ext {
                "yaml" | "yml" => card.to_yaml()?,
                "md" | "markdown" => card.to_markdown(),
                _ => card.to_json()?,
            };

            std::fs::write(&output, output_content)?;
            println!("✅ Model card generated successfully!");
            println!("   Model: {} v{}", model, version_num);
            println!("   Output: {}", output.display());
            println!("\n💡 Edit the file to add more details:");
            println!("   - Training data and evaluation metrics");
            println!("   - Ethical considerations");
            println!("   - Environmental impact");
        }
    }

    Ok(())
}

fn prompt_passphrase(prompt: &str) -> Result<Vec<u8>> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let passphrase = rpassword::read_password()?;
    Ok(passphrase.into_bytes())
}
