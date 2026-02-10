//! Tests covering previously untested critical functions:
//! - Vault::change_passphrase
//! - AuditLogger read_entries, log_auth, log_security_violation
//! - FormatConverter register, can_convert, convert
//! - VersionControl cleanup_old_versions, verify_checksum
//! - ComplianceChecker set_check_enabled / is_check_enabled

use ai_model_vault::audit::{AuditEventType, AuditLogger};
use ai_model_vault::compliance::ComplianceChecker;
use ai_model_vault::crypto::FipsCrypto;
use ai_model_vault::formats::{FormatConverter, ModelFormat, ModelMetadata};
use ai_model_vault::version::VersionControl;
use ai_model_vault::{Vault, VaultConfig};
use tempfile::tempdir;

// ========================= AuditLogger Tests =========================

#[test]
fn test_audit_log_and_read_entries() {
    let tmp = tempdir().unwrap();
    let log_path = tmp.path().join("audit.jsonl");
    let logger = AuditLogger::new(&log_path).unwrap();

    // Log some events
    logger
        .log_model_stored("gpt2", 1, true)
        .unwrap();
    logger
        .log_model_retrieved("gpt2", 1, true)
        .unwrap();
    logger
        .log_model_stored("bert", 1, false)
        .unwrap();

    // Read all entries
    let entries = logger.read_entries(None).unwrap();
    assert_eq!(entries.len(), 3);
    assert!(matches!(
        entries[0].event_type,
        AuditEventType::ModelStored
    ));
    assert_eq!(entries[0].model_name.as_deref(), Some("gpt2"));
    assert!(entries[0].success);
    assert!(matches!(
        entries[1].event_type,
        AuditEventType::ModelRetrieved
    ));
    assert!(!entries[2].success);

    // Read with limit
    let limited = logger.read_entries(Some(2)).unwrap();
    assert_eq!(limited.len(), 2);
}

#[test]
fn test_audit_read_entries_missing_file() {
    let tmp = tempdir().unwrap();
    let log_path = tmp.path().join("nonexistent.jsonl");
    let logger = AuditLogger::new(&log_path).unwrap();
    let entries = logger.read_entries(None).unwrap();
    assert!(entries.is_empty());
}

#[test]
fn test_audit_log_auth_success() {
    let tmp = tempdir().unwrap();
    let log_path = tmp.path().join("auth.jsonl");
    let logger = AuditLogger::new(&log_path).unwrap();

    logger.log_auth(true, None).unwrap();
    let entries = logger.read_entries(None).unwrap();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        entries[0].event_type,
        AuditEventType::AuthSuccess
    ));
    assert!(entries[0].success);
}

#[test]
fn test_audit_log_auth_failure() {
    let tmp = tempdir().unwrap();
    let log_path = tmp.path().join("auth_fail.jsonl");
    let logger = AuditLogger::new(&log_path).unwrap();

    logger.log_auth(false, Some("bad password")).unwrap();
    let entries = logger.read_entries(None).unwrap();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        entries[0].event_type,
        AuditEventType::AuthFailure
    ));
    assert!(!entries[0].success);
    assert!(entries[0].description.contains("bad password"));
}

#[test]
fn test_audit_log_security_violation() {
    let tmp = tempdir().unwrap();
    let log_path = tmp.path().join("sec.jsonl");
    let logger = AuditLogger::new(&log_path).unwrap();

    logger
        .log_security_violation("tampering detected")
        .unwrap();
    let entries = logger.read_entries(None).unwrap();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        entries[0].event_type,
        AuditEventType::SecurityViolation
    ));
    assert!(entries[0].description.contains("tampering"));
    assert!(!entries[0].success);
}

// ========================= FormatConverter Tests =========================

#[test]
fn test_format_converter_same_format_passthrough() {
    let converter = FormatConverter::new();
    let data = b"model data";
    let result = converter
        .convert(data, ModelFormat::PyTorch, ModelFormat::PyTorch)
        .unwrap();
    assert_eq!(result, data);
}

#[test]
fn test_format_converter_register_and_convert() {
    let mut converter = FormatConverter::new();

    fn mock_convert(data: &[u8]) -> ai_model_vault::Result<Vec<u8>> {
        let mut out = b"converted:".to_vec();
        out.extend_from_slice(data);
        Ok(out)
    }

    converter.register(ModelFormat::PyTorch, ModelFormat::ONNX, mock_convert);

    assert!(converter.can_convert(ModelFormat::PyTorch, ModelFormat::ONNX));
    assert!(!converter.can_convert(ModelFormat::ONNX, ModelFormat::PyTorch));

    let result = converter
        .convert(b"hello", ModelFormat::PyTorch, ModelFormat::ONNX)
        .unwrap();
    assert_eq!(result, b"converted:hello");
}

#[test]
fn test_format_converter_unsupported_returns_error() {
    let converter = FormatConverter::new();
    let result = converter.convert(b"data", ModelFormat::PyTorch, ModelFormat::ONNX);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("No converter available"));
}

// ========================= VersionControl Tests =========================

fn setup_version_control_with_models(count: u32) -> (tempfile::TempDir, VersionControl) {
    let tmp = tempdir().unwrap();
    let mut vc = VersionControl::new(tmp.path()).unwrap();
    for i in 1..=count {
        vc.add_version(
            "test_model",
            &format!("file_v{}.enc", i),
            "pytorch",
            1000 + u64::from(i),
            500 + u64::from(i),
            &format!("checksum_{}", i),
            None,
            if i > 1 { Some(i - 1) } else { None },
        )
        .unwrap();
    }
    (tmp, vc)
}

#[test]
fn test_cleanup_old_versions_trims_correctly() {
    let (_tmp, mut vc) = setup_version_control_with_models(5);

    let deleted = vc.cleanup_old_versions("test_model", 2).unwrap();
    assert_eq!(deleted.len(), 3);

    // Kept versions should be the 2 most recent (v4, v5)
    let remaining = vc.list_versions("test_model");
    assert_eq!(remaining.len(), 2);
    let version_nums: Vec<u32> = remaining.iter().map(|v| v.version).collect();
    assert!(version_nums.contains(&5));
    assert!(version_nums.contains(&4));
}

#[test]
fn test_cleanup_old_versions_no_op_when_fewer() {
    let (_tmp, mut vc) = setup_version_control_with_models(2);

    let deleted = vc.cleanup_old_versions("test_model", 5).unwrap();
    assert!(deleted.is_empty());
    assert_eq!(vc.list_versions("test_model").len(), 2);
}

#[test]
fn test_cleanup_old_versions_nonexistent_model() {
    let tmp = tempdir().unwrap();
    let mut vc = VersionControl::new(tmp.path()).unwrap();

    let deleted = vc.cleanup_old_versions("not_here", 3).unwrap();
    assert!(deleted.is_empty());
}

#[test]
fn test_verify_checksum_correct_data() {
    let tmp = tempdir().unwrap();
    let mut vc = VersionControl::new(tmp.path()).unwrap();

    let data = b"test model data for checksum";
    let checksum = hex::encode(FipsCrypto::hash_sha256(data));

    vc.add_version("cksum_model", "file.enc", "onnx", 100, 50, &checksum, None, None)
        .unwrap();

    assert!(vc.verify_checksum("cksum_model", 1, data));
}

#[test]
fn test_verify_checksum_wrong_data() {
    let tmp = tempdir().unwrap();
    let mut vc = VersionControl::new(tmp.path()).unwrap();

    let data = b"original data";
    let checksum = hex::encode(FipsCrypto::hash_sha256(data));

    vc.add_version("cksum_model", "file.enc", "onnx", 100, 50, &checksum, None, None)
        .unwrap();

    assert!(!vc.verify_checksum("cksum_model", 1, b"tampered data"));
}

#[test]
fn test_verify_checksum_nonexistent_version() {
    let tmp = tempdir().unwrap();
    let vc = VersionControl::new(tmp.path()).unwrap();
    assert!(!vc.verify_checksum("no_model", 1, b"doesn't matter"));
}

// ========================= ComplianceChecker Tests =========================

#[test]
fn test_compliance_enable_disable_checks() {
    let mut checker = ComplianceChecker::new();

    // All checks enabled by default
    assert!(checker.is_check_enabled("fips_140_3"));
    assert!(checker.is_check_enabled("cve"));
    assert!(checker.is_check_enabled("mitre_attack"));
    assert!(checker.is_check_enabled("cmmc"));

    // Disable fips check
    checker.set_check_enabled("fips_140_3", false);
    assert!(!checker.is_check_enabled("fips_140_3"));

    // fips check should return true (early-exit when disabled)
    assert!(checker.check_fips_140_3());

    // Disable cmmc check
    checker.set_check_enabled("cmmc", false);
    assert_eq!(checker.check_cmmc(), 0); // disabled returns 0
}

#[test]
fn test_compliance_unknown_check_defaults_disabled() {
    let checker = ComplianceChecker::new();
    assert!(!checker.is_check_enabled("nonexistent_check"));
}

// ========================= Vault::change_passphrase Tests =========================

fn create_test_vault() -> (tempfile::TempDir, Vault) {
    let tmp = tempdir().unwrap();
    let dirs = ai_model_vault::config::DirectoryPaths {
        config_dir: tmp.path().join("config"),
        data_dir: tmp.path().join("data"),
        cache_dir: tmp.path().join("cache"),
        vault_dir: tmp.path().join("data/vaults/default"),
        log_dir: tmp.path().join("data/logs"),
        backends_dir: tmp.path().join("config/backends"),
        utilities_dir: tmp.path().join("config/utilities"),
        databases_dir: tmp.path().join("config/databases"),
    };
    let config = VaultConfig::with_dirs(dirs).unwrap();
    let vault = Vault::new(Some(config)).unwrap();
    (tmp, vault)
}

#[test]
fn test_change_passphrase_reencrypts_models() {
    let (_tmp, mut vault) = create_test_vault();

    let passphrase = b"original_passphrase_with_entropy".to_vec();
    vault.unlock(passphrase).unwrap();

    // Store two models
    let data1 = b"model one data".to_vec();
    let meta1 =
        ModelMetadata::new("model_a".to_string(), ModelFormat::PyTorch);
    vault
        .store_model("model_a", data1.clone(), meta1, None)
        .unwrap();

    let data2 = b"model two data".to_vec();
    let meta2 =
        ModelMetadata::new("model_b".to_string(), ModelFormat::ONNX);
    vault
        .store_model("model_b", data2.clone(), meta2, None)
        .unwrap();

    // Change passphrase
    let new_passphrase = b"new_passphrase_with_sufficient_entropy".to_vec();
    let count = vault.change_passphrase(new_passphrase).unwrap();
    assert_eq!(count, 2);

    // Verify data still retrievable with new key
    let retrieved1 = vault.get_model("model_a", None).unwrap();
    assert_eq!(retrieved1, data1);
    let retrieved2 = vault.get_model("model_b", None).unwrap();
    assert_eq!(retrieved2, data2);
}

#[test]
fn test_change_passphrase_fails_when_locked() {
    let (_tmp, mut vault) = create_test_vault();
    // Don't unlock
    let result = vault.change_passphrase(b"any".to_vec());
    assert!(result.is_err());
}

// ========================= VersionControl::vault_path getter =========================

#[test]
fn test_version_control_vault_path_getter() {
    let tmp = tempdir().unwrap();
    let vc = VersionControl::new(tmp.path()).unwrap();
    assert_eq!(vc.vault_path(), tmp.path());
}
