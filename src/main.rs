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
use cli::handlers::{
    analyze, archive, card, cloud, convert, database, telemetry as telemetry_handler, vault,
};

use ai_model_vault::{telemetry, Result, VaultConfig};

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Load or create config
    let config = if let Some(config_path) = &cli.config {
        let contents = std::fs::read_to_string(config_path)?;
        serde_yaml_ng::from_str(&contents)?
    } else {
        VaultConfig::new()?
    };

    // Initialize telemetry (enabled by default, can be disabled via --no-telemetry or config)
    if cli.no_telemetry || !config.telemetry.enabled {
        telemetry::disable();
    } else {
        telemetry::init_default(Some(&config.dirs.config_dir))?;
        telemetry::track_app_start();
    }

    // Extract sqlite-versions flag (feature-gated)
    #[cfg(feature = "sqlite")]
    let use_sqlite = cli.sqlite_versions;
    #[cfg(not(feature = "sqlite"))]
    let use_sqlite = false;

    let result = match cli.command {
        Commands::Init { name } => vault::handle_init(name, config, use_sqlite),
        Commands::Store {
            name,
            path,
            format,
            description,
            framework,
            task,
        } => vault::handle_store(
            name,
            path,
            format,
            description,
            framework,
            task,
            config,
            use_sqlite,
        ),
        Commands::Get {
            name,
            output,
            version,
        } => vault::handle_get(name, output, version, config, use_sqlite),
        Commands::List => vault::handle_list(config, use_sqlite),
        Commands::Versions { name } => vault::handle_versions(name, config, use_sqlite),
        Commands::Lineage { name, version } => {
            vault::handle_lineage(name, version, config, use_sqlite)
        }
        Commands::Delete {
            name,
            version,
            force,
        } => vault::handle_delete(name, version, force, config, use_sqlite),
        Commands::Stats => vault::handle_stats(config, use_sqlite),
        Commands::Compliance => vault::handle_compliance(),
        Commands::ChangePassphrase => vault::handle_change_passphrase(config, use_sqlite),
        Commands::Archive {
            models,
            output,
            format,
            versions,
        } => archive::handle_archive(models, output, format, versions, config, use_sqlite),
        Commands::Extract { archive, output } => archive::handle_extract(archive, output),
        Commands::Analyze { name, version } => {
            analyze::handle_analyze(name, version, config, use_sqlite)
        }
        Commands::Deduplicate { detailed } => {
            analyze::handle_deduplicate(detailed, config, use_sqlite)
        }
        Commands::Export {
            name,
            output,
            version,
        } => analyze::handle_export(name, output, version, config, use_sqlite),
        Commands::Convert {
            name,
            to_format,
            output,
            version,
            quantization,
            opset,
            validate,
            plan_only,
        } => convert::handle_convert(
            name,
            to_format,
            output,
            version,
            quantization,
            opset,
            validate,
            plan_only,
            config,
            use_sqlite,
        ),
        Commands::ListConversions => convert::handle_list_conversions(),
        #[cfg(feature = "api")]
        Commands::Serve {
            host,
            port,
            jwt_secret,
            token_expiry,
            cors_permissive,
            no_dashboard,
        } => {
            let api_config = ai_model_vault::api::ApiConfig {
                host,
                port,
                jwt_secret,
                token_expiry_secs: token_expiry,
                cors_permissive,
                enable_dashboard: !no_dashboard,
                ..Default::default()
            };
            let rt = tokio::runtime::Runtime::new().map_err(ai_model_vault::VaultError::IoError)?;
            rt.block_on(ai_model_vault::api::server::serve(config, api_config))
        }
        Commands::Cache => vault::handle_cache(),
        Commands::Cloud { command } => cloud::handle_cloud(command, config, use_sqlite),
        Commands::Card { command } => card::handle_card(command, config, use_sqlite),
        Commands::Database { command } => database::handle_database(command),
        Commands::Telemetry { command } => telemetry_handler::handle_telemetry(command, config),
    };

    // Flush telemetry before exit
    telemetry::flush();

    result
}
