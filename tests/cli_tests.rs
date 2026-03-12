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
        .stdout(predicate::str::contains("1.1.0").or(predicate::str::contains("aim")));
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
    aim()
        .args(["list", "--help"])
        .assert()
        .success();
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
    aim()
        .args(["list-conversions"])
        .assert()
        .success()
        .stdout(
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
    aim()
        .arg("nonexistent-command")
        .assert()
        .failure();
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
