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
    acl, analyze, archive, benchmark as benchmark_handler, browse, card, cloud, convert, database,
    diff as diff_handler, evaluation as evaluation_handler, gc, introspect,
    license_scan as license_scan_handler, lineage_graph, multi_vault as multi_vault_handler,
    plugins, policies, profiles, pull, quantization as quantization_handler, register, scan,
    scheduler as scheduler_handler, sign, tags, telemetry as telemetry_handler, validation, vault,
    vault_bundle as vault_bundle_handler, webhooks as webhooks_handler,
};

use ai_model_vault::{telemetry, Result, VaultConfig, VaultError};

fn main() -> std::process::ExitCode {
    // Increase stack size for large clap enum on Windows
    let result = std::thread::Builder::new()
        .stack_size(4 * 1024 * 1024) // 4 MB
        .spawn(run)
        .expect("Failed to spawn main thread")
        .join()
        .expect("Main thread panicked");

    // Map the error category to its documented exit code. Returning `Result`
    // from `main` collapsed every failure to 1 and printed the `Debug` form,
    // so the exit codes published in README.md, AGENTS.md, docs/CLI.md and
    // `.well-known/` described behaviour the binary never had.
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::ExitCode::from(err.exit_code())
        }
    }
}

fn run() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parsed by hand rather than with `Cli::parse()`, which exits 2 on a usage
    // error. Code 2 is `EXIT_AUTH` in the published table, so a mistyped
    // subcommand would be indistinguishable from a wrong passphrase to any
    // agent branching on the exit code. A usage error is invalid input.
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            // `--help` and `--version` arrive here too, and both are successes:
            // clap has already written the text to stdout. A *missing*
            // subcommand is not in that set — clap prints help for it as a
            // convenience, but the command line was still incomplete, and
            // reporting success for it is the bug class this mapping exists
            // to remove.
            let requested_output = matches!(
                err.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            );
            let _ = err.print();
            if requested_output {
                return Ok(());
            }
            return Err(VaultError::InvalidInput(
                "invalid command line — see the usage message above".to_string(),
            ));
        }
    };

    // Load or create config
    let config = if let Some(config_path) = &cli.config {
        let contents = std::fs::read_to_string(config_path)
            .map_err(|e| VaultError::ConfigError(format!("{}: {e}", config_path.display())))?;
        serde_yaml_ng::from_str(&contents)
            .map_err(|e| VaultError::ConfigError(format!("{}: {e}", config_path.display())))?
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
            revocation_store,
        } => {
            // Assigned field-by-field rather than with `..Default::default()`:
            // `ApiConfig` implements `Drop` (it zeroizes `jwt_secret`), and
            // functional update syntax cannot move a non-`Copy` field out of a
            // `Drop` type.
            let mut api_config = ai_model_vault::api::ApiConfig::default();
            api_config.host = host;
            api_config.port = port;
            api_config.jwt_secret = jwt_secret;
            api_config.token_expiry_secs = token_expiry;
            api_config.cors_permissive = cors_permissive;
            api_config.enable_dashboard = !no_dashboard;
            api_config.revocation_store = revocation_store;
            let rt = tokio::runtime::Runtime::new().map_err(ai_model_vault::VaultError::IoError)?;
            rt.block_on(ai_model_vault::api::server::serve(config, api_config))
        }
        Commands::Cache => vault::handle_cache(),
        Commands::Cloud { command } => cloud::handle_cloud(command, config, use_sqlite),
        Commands::Card { command } => card::handle_card(command, config, use_sqlite),
        Commands::Database { command } => database::handle_database(command),
        Commands::Telemetry { command } => telemetry_handler::handle_telemetry(command, config),
        Commands::Introspect { format, compact } => introspect::handle_introspect(format, compact),
        Commands::Pull {
            source,
            output,
            sha256,
            token,
            store,
            name,
        } => pull::handle_pull(
            source, output, sha256, token, store, name, config, use_sqlite,
        ),
        Commands::Sign {
            name,
            version,
            key,
            identity,
            file,
        } => sign::handle_sign(name, version, key, identity, file, config, use_sqlite),
        Commands::Verify {
            name,
            version,
            signature,
            key,
            file,
        } => sign::handle_verify(name, version, signature, key, file, config, use_sqlite),
        Commands::Scan {
            name,
            file,
            version,
            format,
        } => scan::handle_scan(name, file, version, format, config, use_sqlite),
        Commands::Diff {
            left,
            right,
            format,
        } => diff_handler::handle_diff(left, right, format, config, use_sqlite),
        Commands::Register {
            name,
            engine,
            version,
            alias,
            system_prompt,
        } => register::handle_register(
            name,
            engine,
            version,
            alias,
            system_prompt,
            config,
            use_sqlite,
        ),
        Commands::Benchmark { command } => benchmark_handler::handle_benchmark(command, config),
        Commands::LicenseScan { path, format } => {
            license_scan_handler::handle_license_scan(path, format)
        }
        Commands::Tag { command } => tags::handle_tag(command, config),
        Commands::Search { query, tag, format } => tags::handle_search(query, tag, format, config),
        Commands::VaultExport { output } => {
            vault_bundle_handler::handle_vault_export(output, config)
        }
        Commands::VaultImport { archive, target } => {
            vault_bundle_handler::handle_vault_import(archive, target, config)
        }
        Commands::Gc { dry_run } => gc::handle_gc(dry_run, config),
        Commands::Browse => browse::handle_browse(config),
        Commands::Webhook { command } => webhooks_handler::handle_webhook(command, config),
        Commands::Acl { command } => acl::handle_acl(command, config),
        Commands::Validate { name, version } => {
            validation::handle_validate(name, version, config, use_sqlite)
        }
        Commands::Policy { command } => policies::handle_policy(command, config, use_sqlite),
        Commands::LineageGraph { command } => lineage_graph::handle_lineage_graph(command, config),
        Commands::Plugin { command } => plugins::handle_plugin(command, config),
        Commands::Profile { command } => profiles::handle_profile(command, config),
        Commands::Quantize { command } => quantization_handler::handle_quantize(command, config),
        Commands::Eval { command } => evaluation_handler::handle_eval(command, config),
        Commands::Backup { command } => scheduler_handler::handle_backup(command, config),
        Commands::Vaults { command } => multi_vault_handler::handle_vaults(command, config),
    };

    // Flush telemetry before exit
    telemetry::flush();

    result
}
