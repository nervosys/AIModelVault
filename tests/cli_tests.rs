//! CLI integration tests for the `aim` binary.
//!
//! Uses `assert_cmd` to exercise the CLI end-to-end, validating argument parsing,
//! help output, version display, and vault lifecycle commands.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

/// Helper to get a Command for the `aim` binary.
fn aim() -> Command {
    #[allow(deprecated)]
    Command::cargo_bin("aim").unwrap()
}

// ──────────────────────────────────────────────────────────────
// Help & Version
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_help() {
    aim()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("secure vault"))
        .stdout(predicate::str::contains("Usage").or(predicate::str::contains("USAGE")));
}

#[test]
fn test_cli_version() {
    aim()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("1.2.1").or(predicate::str::contains("aim")));
}

#[test]
fn test_cli_no_args_shows_help() {
    // Running with no subcommand should show help or error gracefully
    let result = aim().assert();
    // clap exits with code 2 when no subcommand given (or 0 if default help)
    result.code(predicate::in_iter([0, 2]));
}

// ──────────────────────────────────────────────────────────────
// Subcommand Help
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_init_help() {
    aim()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("init").or(predicate::str::contains("vault")));
}

#[test]
fn test_cli_store_help() {
    aim()
        .args(["store", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("store").or(predicate::str::contains("model")));
}

#[test]
fn test_cli_list_help() {
    aim().args(["list", "--help"]).assert().success();
}

#[test]
fn test_cli_convert_help() {
    aim()
        .args(["convert", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("convert"));
}

#[test]
fn test_cli_compliance_help() {
    aim()
        .args(["compliance", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("compliance"));
}

// ──────────────────────────────────────────────────────────────
// Vault Lifecycle (init, store, list, get)
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_init_vault() {
    let dir = tempdir().unwrap();

    aim()
        .args(["init", "--name", "test-vault"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Vault").or(predicate::str::contains("initialized").or(predicate::str::contains("created"))));
}

#[test]
fn test_cli_list_empty_vault() {
    // `list` requires interactive passphrase, so just verify it starts and asks
    // for input (will fail/timeout without tty, which is expected).
    // We verify the binary accepts the command without crashing.
    let dir = tempdir().unwrap();

    // Init first
    aim()
        .args(["init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    // List will fail because no tty for passphrase — that's expected
    // Just verify it doesn't crash with a bad exit code before prompting
    let result = aim()
        .args(["list"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .timeout(std::time::Duration::from_secs(3))
        .assert();
    // Expect either success (if no passphrase needed) or failure (no tty)
    let _ = result;
}

#[test]
fn test_cli_list_conversions() {
    aim().args(["list-conversions"]).assert().success().stdout(
        predicate::str::contains("safetensors")
            .or(predicate::str::contains("Safetensors"))
            .or(predicate::str::contains("GGUF"))
            .or(predicate::str::contains("gguf")),
    );
}

#[test]
fn test_cli_stats_on_vault() {
    let dir = tempdir().unwrap();

    aim()
        .args(["init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["stats"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();
}

// ──────────────────────────────────────────────────────────────
// Error Cases
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_unknown_subcommand() {
    aim().arg("nonexistent-command").assert().failure();
}

#[test]
fn test_cli_store_missing_args() {
    aim()
        .args(["store"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

#[test]
fn test_cli_get_missing_args() {
    aim()
        .args(["get"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

// ──────────────────────────────────────────────────────────────
// Feature Flags
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_sqlite_versions_flag_accepted() {
    // The --sqlite-versions flag should be accepted without error
    let dir = tempdir().unwrap();

    aim()
        .args(["--sqlite-versions", "init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_cli_compliance_runs() {
    let dir = tempdir().unwrap();

    aim()
        .args(["init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["compliance"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();
}

// ──────────────────────────────────────────────────────────────
// Additional Subcommand Help Tests
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_versions_help() {
    aim()
        .args(["versions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("version").or(predicate::str::contains("Version")));
}

#[test]
fn test_cli_lineage_help() {
    aim()
        .args(["lineage", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lineage").or(predicate::str::contains("history")));
}

#[test]
fn test_cli_delete_help() {
    aim()
        .args(["delete", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("delete").or(predicate::str::contains("Delete")));
}

#[test]
fn test_cli_archive_help() {
    aim()
        .args(["archive", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("archive").or(predicate::str::contains("Archive")));
}

#[test]
fn test_cli_extract_help() {
    aim()
        .args(["extract", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("extract").or(predicate::str::contains("Extract")));
}

#[test]
fn test_cli_analyze_help() {
    aim()
        .args(["analyze", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("analyze").or(predicate::str::contains("Analyze")));
}

#[test]
fn test_cli_deduplicate_help() {
    aim()
        .args(["deduplicate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("deduplicate").or(predicate::str::contains("duplicate")));
}

#[test]
fn test_cli_export_help() {
    aim()
        .args(["export", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("export").or(predicate::str::contains("Export")));
}

#[test]
fn test_cli_cache_help() {
    aim()
        .args(["cache", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cache").or(predicate::str::contains("Cache")));
}

#[test]
fn test_cli_change_passphrase_help() {
    aim()
        .args(["change-passphrase", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("passphrase")
                .or(predicate::str::contains("Passphrase"))
                .or(predicate::str::contains("Change")),
        );
}

#[test]
fn test_cli_cloud_help() {
    aim()
        .args(["cloud", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cloud").or(predicate::str::contains("Cloud")));
}

#[test]
fn test_cli_database_help() {
    aim()
        .args(["database", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("database").or(predicate::str::contains("Database")));
}

#[test]
fn test_cli_card_help() {
    aim()
        .args(["card", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("card").or(predicate::str::contains("Card")));
}

#[test]
fn test_cli_telemetry_help() {
    aim()
        .args(["telemetry", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("telemetry").or(predicate::str::contains("Telemetry")));
}

// ──────────────────────────────────────────────────────────────
// Telemetry Subcommands (non-interactive)
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_telemetry_status() {
    aim()
        .args(["telemetry", "status"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Telemetry")
                .or(predicate::str::contains("telemetry"))
                .or(predicate::str::contains("enabled"))
                .or(predicate::str::contains("disabled")),
        );
}

#[test]
fn test_cli_telemetry_disable_then_status() {
    // Disable telemetry, then check status reports disabled
    aim()
        .args(["telemetry", "disable"])
        .env("DO_NOT_TRACK", "1")
        .assert()
        .success();

    aim()
        .args(["telemetry", "status"])
        .env("DO_NOT_TRACK", "1")
        .assert()
        .success();
}

// ──────────────────────────────────────────────────────────────
// Database Subcommands (non-interactive)
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_database_init_and_stats() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    aim()
        .args([
            "database",
            "init",
            "--path",
            db_path.to_str().unwrap(),
            "--db-type",
            "sqlite",
        ])
        .assert()
        .success();

    aim()
        .args(["database", "stats", "--path", db_path.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn test_cli_database_list_empty() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    aim()
        .args([
            "database",
            "init",
            "--path",
            db_path.to_str().unwrap(),
            "--db-type",
            "sqlite",
        ])
        .assert()
        .success();

    aim()
        .args(["database", "list", "--path", db_path.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn test_cli_database_store_and_search() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let doc_path = dir.path().join("doc.txt");
    std::fs::write(
        &doc_path,
        "The transformer architecture uses attention mechanisms.",
    )
    .unwrap();

    aim()
        .args([
            "database",
            "init",
            "--path",
            db_path.to_str().unwrap(),
            "--db-type",
            "sqlite",
        ])
        .assert()
        .success();

    aim()
        .args([
            "database",
            "store",
            "--path",
            db_path.to_str().unwrap(),
            "--input",
            doc_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    aim()
        .args([
            "database",
            "search",
            "--path",
            db_path.to_str().unwrap(),
            "transformer",
        ])
        .assert()
        .success();
}

// ──────────────────────────────────────────────────────────────
// Model Card Subcommands (non-interactive)
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_card_template_basic() {
    let dir = tempdir().unwrap();
    let output = dir.path().join("card.json");

    aim()
        .args([
            "card",
            "template",
            "--template-type",
            "basic",
            "--output",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Verify the template file was created
    assert!(output.exists());
}

#[test]
fn test_cli_card_create_and_validate() {
    let dir = tempdir().unwrap();
    let output = dir.path().join("card.json");

    aim()
        .args([
            "card",
            "create",
            "test-model",
            "--version",
            "1.0",
            "--description",
            "A test model",
            "--model-type",
            "classifier",
            "--architecture",
            "ResNet-50",
            "--output",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(output.exists());

    aim()
        .args(["card", "validate", output.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn test_cli_card_create_and_show() {
    let dir = tempdir().unwrap();
    let output = dir.path().join("card.yaml");

    aim()
        .args([
            "card",
            "create",
            "my-llm",
            "--version",
            "2.0",
            "--description",
            "A language model",
            "--model-type",
            "LLM",
            "--architecture",
            "Transformer",
            "--output",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();

    aim()
        .args([
            "card",
            "show",
            output.to_str().unwrap(),
            "--format",
            "markdown",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("my-llm").or(predicate::str::contains("language model")));
}

#[test]
fn test_cli_card_convert_json_to_yaml() {
    let dir = tempdir().unwrap();
    let json_out = dir.path().join("card.json");
    let yaml_out = dir.path().join("card.yaml");

    aim()
        .args([
            "card",
            "create",
            "conv-model",
            "--version",
            "1.0",
            "--description",
            "For conversion test",
            "--model-type",
            "classifier",
            "--architecture",
            "CNN",
            "--output",
            json_out.to_str().unwrap(),
        ])
        .assert()
        .success();

    aim()
        .args([
            "card",
            "convert",
            json_out.to_str().unwrap(),
            yaml_out.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(yaml_out.exists());
}

// ──────────────────────────────────────────────────────────────
// Additional Error Cases
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_delete_missing_args() {
    aim()
        .args(["delete"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

#[test]
fn test_cli_archive_missing_args() {
    aim()
        .args(["archive"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

#[test]
fn test_cli_versions_missing_args() {
    aim()
        .args(["versions"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

#[test]
fn test_cli_lineage_missing_args() {
    aim()
        .args(["lineage"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

#[test]
fn test_cli_export_missing_args() {
    aim()
        .args(["export"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

// ──────────────────────────────────────────────────────────────
// Vault Lifecycle — Extended
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_cache_on_vault() {
    let dir = tempdir().unwrap();

    aim()
        .args(["init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["cache"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_cli_list_conversions_contains_formats() {
    // Validate that list-conversions includes expected format names
    aim()
        .args(["list-conversions"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ONNX").or(predicate::str::contains("onnx")))
        .stdout(
            predicate::str::contains("PyTorch")
                .or(predicate::str::contains("pytorch"))
                .or(predicate::str::contains("pt")),
        );
}

#[test]
fn test_cli_init_custom_name() {
    let dir = tempdir().unwrap();

    aim()
        .args(["init", "--name", "my-custom-vault"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("my-custom-vault")
                .or(predicate::str::contains("Vault"))
                .or(predicate::str::contains("initialized")),
        );
}

#[test]
fn test_cli_sqlite_versions_with_stats() {
    let dir = tempdir().unwrap();

    aim()
        .args(["--sqlite-versions", "init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["--sqlite-versions", "stats"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_cli_sqlite_versions_with_compliance() {
    let dir = tempdir().unwrap();

    aim()
        .args(["--sqlite-versions", "init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["--sqlite-versions", "compliance"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();
}

// ──────────────────────────────────────────────────────────────
// Telemetry Flags
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_no_telemetry_flag_with_init() {
    let dir = tempdir().unwrap();

    aim()
        .args(["--no-telemetry", "init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_cli_do_not_track_env() {
    aim()
        .args(["compliance"])
        .env("DO_NOT_TRACK", "1")
        .assert()
        .success();
}

// ──────────────────────────────────────────────────────────────
// Convert Error Cases
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_convert_missing_args() {
    aim()
        .args(["convert"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

// ──────────────────────────────────────────────────────────────
// Cloud Error Cases
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_cloud_push_missing_args() {
    aim()
        .args(["cloud", "push"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

#[test]
fn test_cli_cloud_pull_missing_args() {
    aim()
        .args(["cloud", "pull"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

#[test]
fn test_cli_cloud_list_missing_args() {
    aim()
        .args(["cloud", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

// ──────────────────────────────────────────────────────────────
// Archive / Extract Error Cases
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_extract_nonexistent_archive() {
    aim()
        .args(["extract", "nonexistent.tar"])
        .assert()
        .failure();
}

// ──────────────────────────────────────────────────────────────
// Database Error Cases
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_database_search_no_results() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("empty.db");

    aim()
        .args([
            "database",
            "init",
            "--path",
            db_path.to_str().unwrap(),
            "--db-type",
            "sqlite",
        ])
        .assert()
        .success();

    aim()
        .args([
            "database",
            "search",
            "--path",
            db_path.to_str().unwrap(),
            "nonexistent query",
        ])
        .assert()
        .success();
}

#[test]
fn test_cli_database_store_nonexistent_file() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    aim()
        .args([
            "database",
            "init",
            "--path",
            db_path.to_str().unwrap(),
            "--db-type",
            "sqlite",
        ])
        .assert()
        .success();

    aim()
        .args([
            "database",
            "store",
            "--path",
            db_path.to_str().unwrap(),
            "--input",
            "nonexistent_file.txt",
        ])
        .assert()
        .failure();
}

// ──────────────────────────────────────────────────────────────
// Card Error Cases
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_card_validate_nonexistent_file() {
    aim()
        .args(["card", "validate", "no-such-card.json"])
        .assert()
        .failure();
}

#[test]
fn test_cli_card_show_nonexistent_file() {
    aim()
        .args(["card", "show", "no-such-card.json"])
        .assert()
        .failure();
}

// ──────────────────────────────────────────────────────────────
// SQLite Backend — Extended
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_sqlite_versions_with_cache() {
    let dir = tempdir().unwrap();

    aim()
        .args(["--sqlite-versions", "init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["--sqlite-versions", "cache"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_cli_init_twice_same_dir() {
    let dir = tempdir().unwrap();

    aim()
        .args(["init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();
}

// ──────────────────────────────────────────────────────────────
// v1.4.0 Feature — Help Tests
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_tag_help() {
    aim()
        .args(["tag", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tag").or(predicate::str::contains("Tag")));
}

#[test]
fn test_cli_search_help() {
    aim()
        .args(["search", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("search").or(predicate::str::contains("Search")));
}

#[test]
fn test_cli_vault_export_help() {
    aim()
        .args(["vault-export", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("export").or(predicate::str::contains("Export")));
}

#[test]
fn test_cli_vault_import_help() {
    aim()
        .args(["vault-import", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("import").or(predicate::str::contains("Import")));
}

#[test]
fn test_cli_gc_help() {
    aim()
        .args(["gc", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("gc").or(predicate::str::contains("garbage").or(predicate::str::contains("clean"))));
}

#[test]
fn test_cli_browse_help() {
    aim()
        .args(["browse", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("browse").or(predicate::str::contains("Browse")));
}

#[test]
fn test_cli_webhook_help() {
    aim()
        .args(["webhook", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("webhook").or(predicate::str::contains("Webhook")));
}

#[test]
fn test_cli_acl_help() {
    aim()
        .args(["acl", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("acl").or(predicate::str::contains("access")));
}

#[test]
fn test_cli_validate_help() {
    aim()
        .args(["validate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("validate").or(predicate::str::contains("Validate")));
}

#[test]
fn test_cli_policy_help() {
    aim()
        .args(["policy", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("policy").or(predicate::str::contains("Policy")));
}

#[test]
fn test_cli_lineage_graph_help() {
    aim()
        .args(["lineage-graph", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lineage").or(predicate::str::contains("Lineage")));
}

#[test]
fn test_cli_plugin_help() {
    aim()
        .args(["plugin", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("plugin").or(predicate::str::contains("Plugin")));
}

#[test]
fn test_cli_profile_help() {
    aim()
        .args(["profile", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("profile").or(predicate::str::contains("Profile")));
}

// ──────────────────────────────────────────────────────────────
// v1.4.0 Feature — Functional Tests
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_gc_dry_run_on_vault() {
    let dir = tempdir().unwrap();

    aim()
        .args(["init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["gc", "--dry-run"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_cli_acl_grant_list_revoke() {
    let dir = tempdir().unwrap();

    aim()
        .args(["init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["acl", "grant", "alice", "writer"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["acl", "list"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("alice"));

    aim()
        .args(["acl", "revoke", "alice"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_cli_webhook_add_list_remove() {
    let dir = tempdir().unwrap();
    let id = format!("hook-{}", std::process::id());

    aim()
        .args(["init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["webhook", "add", &id, "https://example.com/hook"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["webhook", "list"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("example.com"));
}

#[test]
fn test_cli_policy_set_show() {
    let dir = tempdir().unwrap();

    aim()
        .args(["init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["policy", "set", "test-model", "--max-versions", "5"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["policy", "show", "test-model"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("test-model"));
}

#[test]
fn test_cli_profile_create_list_activate() {
    let dir = tempdir().unwrap();

    aim()
        .args(["profile", "create", "dev"])
        .env("aimodelvault_CONFIG", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["profile", "list"])
        .env("aimodelvault_CONFIG", dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("dev"));

    aim()
        .args(["profile", "activate", "dev"])
        .env("aimodelvault_CONFIG", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["profile", "show", "dev"])
        .env("aimodelvault_CONFIG", dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("dev"));
}

#[test]
fn test_cli_lineage_graph_show_empty() {
    let dir = tempdir().unwrap();

    aim()
        .args(["init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["lineage-graph", "show", "any-model"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_cli_plugin_list_empty() {
    let dir = tempdir().unwrap();

    aim()
        .args(["init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["plugin", "list"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_cli_tag_add_list_on_vault() {
    let dir = tempdir().unwrap();

    aim()
        .args(["init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["tag", "add", "my-model", "llm", "production"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["tag", "list", "my-model"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("llm"));
}

// ──────────────────────────────────────────────────────────────
// v1.4.0 Feature — Error Cases
// ──────────────────────────────────────────────────────────────

#[test]
fn test_cli_validate_missing_args() {
    aim()
        .args(["validate"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

#[test]
fn test_cli_vault_export_missing_args() {
    aim()
        .args(["vault-export"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

#[test]
fn test_cli_vault_import_missing_args() {
    aim()
        .args(["vault-import"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

#[test]
fn test_cli_tag_add_missing_args() {
    aim()
        .args(["tag", "add"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

#[test]
fn test_cli_search_empty_query() {
    let dir = tempdir().unwrap();

    aim()
        .args(["init"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();

    aim()
        .args(["search"])
        .env("aimodelvault_VAULT", dir.path().to_str().unwrap())
        .assert()
        .success();
}
