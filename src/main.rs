//! AI Model Vault CLI application
//!
//! Supported formats include:
//! - LLM formats: Safetensors (.safetensors), GGUF (.gguf), PyTorch (.pt/.pth/.bin)
//! - Production formats: TensorRT (.plan), ONNX (.onnx), TFLite (.tflite)
//! - Platform-specific: MLX (.npz), Core ML (.mlmodel), OpenVINO (.xml)
//! - Mobile/Edge: NCNN (.param), MNN (.mnn), RKNN (.rknn)
//! - Legacy: Caffe (.caffemodel), MXNet (.params), Darknet (.weights)
//! - Data formats: HDF5 (.h5/.hdf5), Pickle (.pkl), NumPy (.npy/.npz)

mod cli;

use clap::Parser;
use cli::args::{Cli, Commands};
use cli::handlers::{analyze, archive, card, cloud, convert, database, vault};

use ai_model_vault::{Result, VaultConfig};

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
        Commands::Init { name } => vault::handle_init(name, config),
        Commands::Store {
            name,
            path,
            format,
            description,
            framework,
            task,
        } => vault::handle_store(name, path, format, description, framework, task, config),
        Commands::Get {
            name,
            output,
            version,
        } => vault::handle_get(name, output, version, config),
        Commands::List => vault::handle_list(config),
        Commands::Versions { name } => vault::handle_versions(name, config),
        Commands::Lineage { name, version } => vault::handle_lineage(name, version, config),
        Commands::Delete {
            name,
            version,
            force,
        } => vault::handle_delete(name, version, force, config),
        Commands::Stats => vault::handle_stats(config),
        Commands::Compliance => vault::handle_compliance(),
        Commands::ChangePassphrase => vault::handle_change_passphrase(config),
        Commands::Archive {
            models,
            output,
            format,
            versions,
        } => archive::handle_archive(models, output, format, versions, config),
        Commands::Extract { archive, output } => archive::handle_extract(archive, output),
        Commands::Analyze { name, version } => analyze::handle_analyze(name, version, config),
        Commands::Deduplicate { detailed } => analyze::handle_deduplicate(detailed, config),
        Commands::Export {
            name,
            output,
            version,
        } => analyze::handle_export(name, output, version, config),
        Commands::Convert {
            name,
            to_format,
            output,
            version,
            quantization,
            opset,
            validate,
            plan_only,
        } => convert::handle_convert(name, to_format, output, version, quantization, opset, validate, plan_only, config),
        Commands::ListConversions => convert::handle_list_conversions(),
        Commands::Cache => vault::handle_cache(),
        Commands::Cloud { command } => cloud::handle_cloud(command, config),
        Commands::Card { command } => card::handle_card(command, config),
        Commands::Database { command } => database::handle_database(command),
    }
}

