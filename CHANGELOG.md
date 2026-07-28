# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.7.0] - 2026-07-27

### Security

- **All RUSTSEC advisories are now genuinely resolved — both ignore lists are empty.** Migrating Azure to the SDK for Rust v1 (`azure_storage_blob`) cleared the last two: `azure_core` 0.21 pinned quick-xml 0.31 (RUSTSEC-2026-0194/0195) and pulled `http-types` (RUSTSEC-2026-0174); the v1 stack uses quick-xml 0.41 and drops `http-types` entirely. Six further ignores (`fxhash`, `instant`, `paste`, `rustls-pemfile`, `lru`, `rand`) no longer matched the dependency graph and were removed. `cargo audit` and `cargo deny check` now pass with **no suppressions at all**.
- **An empty passphrase can no longer unlock a vault.** On a closed or non-interactive stdin, `rpassword` returns `""`, which was accepted and used to derive a key — so `aim list` on a fresh vault succeeded with no secret at all. The prompt now rejects an empty passphrase and points at the three supported sources.
- **9 advisories resolved.** `cargo audit` and `cargo deny check` both pass again; they had gone red as advisories were published after the last release.
  - `rustls-webpki` 0.101 (RUSTSEC-2026-0098/0099/0104 — name-constraint bypasses and a CRL parse panic) reached the tree because the AWS SDKs' default `rustls` feature selects their *legacy* hyper-0.14/rustls-0.21 stack. Fixed by building the SDKs with `default-features = false` plus `default-https-client`, which uses rustls 0.23 / webpki 0.103.
  - `pyo3` 0.24 → 0.29 (RUSTSEC-2026-0176 out-of-bounds read in `PyList`/`PyTuple` iterators, RUSTSEC-2026-0177 missing `Sync` bound).
  - `quinn-proto` (RUSTSEC-2026-0185), `crossbeam-epoch` (RUSTSEC-2026-0204), `lru` (RUSTSEC-2026-0002) resolved by `cargo update`.
  - `quick-xml` 0.31 (RUSTSEC-2026-0194/0195) remains, pinned by `azure_core` 0.21 — the last release of the legacy Azure SDK line. Both are denial-of-service via a malicious XML *response*, so they require a hostile storage endpoint and are unreachable from vault data. Documented with the upgrade path in `deny.toml` and the new `.cargo/audit.toml`; clearing them needs the rewritten `azure_storage_blob` crate, still in beta.
- **Stale advisory ignores removed** from `deny.toml` — the three `rustls-webpki` entries were suppressing advisories that are now actually fixed.


### Added

- **Non-interactive passphrase resolution** — `prompt_passphrase()` now resolves in order: `$aimodelvault_PASSPHRASE` (literal value or KMS URI) → a line piped on stdin when stdin is not a terminal → interactive masked prompt. Every passphrase-gated command (`store`, `get`, `list`, `sign`, `cloud *`, …) is now usable from CI and from agents. The env var was documented in `AGENTS.md` but had never been read by any code path.
- **KMS URIs for signing keys** — `aim sign --key` and `aim verify --key` accept a KMS URI as well as a file path; `docs/KMS.md` advertised this and it had never been implemented. The stored secret may be a keypair JSON document or a bare hex seed (`ModelSigner::keypair_from_seed` / `parse_keypair`). A KMS-backed key is never generated or written to disk.
- **`aimodelvault_HOME`** — relocates all config/data/cache directories under one root, for test isolation, containers, and per-project vaults.
- **KMS URI scheme** (`src/kms.rs`) — `KmsUri` parser and `kms::fetch` / `kms::resolve` for `env://NAME`, `file:///path`, `aws-sm://secret`, `azure-kv://vault/secret`, `vault://mount/path/key`. `docs/KMS.md` documented this scheme; no parser existed.
- **KMS backends implemented** — `file://` (rejects group/world-readable files on Unix), Azure Key Vault and HashiCorp Vault over REST (KV v2 with v1 fallback), and AWS Secrets Manager via `aws-sdk-secretsmanager` behind the `s3` feature. Previously three of four backends were stubs that returned an error unconditionally — including with `s3` enabled, which the stub's own message told users to turn on.
- **CI feature matrix job** — clippy over `default`, `s3`, `azure`, `cloud`, `api`, `database`. CI only ever built `full,graphql`, so breakage in the cloud features went unnoticed.
- **CLI integration tests** — 7 tests covering the store → list → get round-trip, wrong-passphrase rejection, `env://` and `file://` URIs, unresolvable-URI failure, and stdin. No CLI test previously exercised a passphrase-gated command.


### Fixed

- **First-run directory creation raced with its own permission tightening.** `ensure_directories` created each directory and immediately rewrote its ACL before creating the next — but several are nested under `data_dir`, and on Windows `icacls /inheritance:r` briefly leaves a directory without a usable DACL, so a concurrent create of a child failed with "Access is denied". All directories are now created first and restricted afterwards, with a single retry for a genuinely concurrent creator. Only reachable when the config directory does not yet exist, which is why it survived: on any machine that had already run `aim`, the `if !dir.exists()` guard skipped the whole path.
- **`aim convert` never worked on a vaulted model.** Version records persist `format.name()` (`"PyTorch"`), but the handler parsed it with `ModelFormat::from_extension`, which only knows extensions (`pt`/`pth`/`bin`). Every stored format silently became `Custom("pytorch")`, so path lookup always failed with "No conversion path from PyTorch to ONNX" — while the header printed `Source format: PyTorch`, because `Custom` renders its own string. `aim diff` on `name@version` had the same bug and silently fell back to a generic byte diff instead of tensor-level comparison. Added `ModelFormat::from_name` / `from_stored` with a round-trip test over every variant, and pointed both call sites at it. This went unnoticed because no CLI test could unlock a vault until this release.
- **`POST /api/v1/convert` returned plan JSON labelled as the target format.** Four converters (PyTorch→ONNX, ONNX→TensorRT, ONNX→CoreML, SafeTensors→GGUF) need an external Python toolchain and emit a JSON *plan* instead of model bytes. The REST endpoint base64-encoded that plan into `data_base64` and returned HTTP 200 with `target_format: "onnx"` — a client decoding it into `model.onnx` got a corrupt file. The response now carries `converted: false` and a `plan` object, and omits `data_base64` entirely. `.well-known/openapi.yaml`'s `ConversionResult` schema, which described a completely different shape (`success`/`output_path`/`output_size_bytes`) than the endpoint actually returns, was corrected to match.
- **"Is this a plan?" is now typed, not sniffed.** `ConversionResult` gained `plan: Option<Value>` and `is_plan()`, and `Converter` gained `produces_plan()`. The CLI previously detected this by parsing its own output looking for a `"converter"` key; any other consumer of the library API had no way to tell at all. When a conversion is a plan, `data` is empty, so no caller can write it out as a model file.
- **Multi-step conversions no longer feed a plan into the next converter.** PyTorch→ONNX→TensorRT used to run step 2 on step 1's plan JSON, producing a meaningless plan-of-a-plan. The pipeline now stops at the first step needing external tooling and returns that plan.
- **`aim convert` writes `<output>.plan.json`** rather than leaving the user with no artifact, and states plainly that no target-format file was produced.
- **`aimodelvault_VAULT` and `aimodelvault_CONFIG` were never read.** Both are documented in `AGENTS.md`; nothing consumed them. The consequence was that the entire CLI test suite believed it was writing to a tempdir and was in fact operating on the developer's real vault — which is why tests could only run serially. Implemented as documented, pointed the tests at `aimodelvault_HOME`, and the CLI suite now passes in parallel (11s, down from 103s).
- **Python package version drift** — the package was at 1.3.0 while `test_version_is_set` asserted 1.2.1, so CI's python job failed. Both are now 1.7.0, matching the crate, and the test asserts the version's shape plus equality with `Cargo.toml` rather than a literal that can rot.
- **Dockerfile pinned `rust:1.85`** while the crate's MSRV is 1.89 — the image build could not have succeeded. Bumped to 1.89, and the stale `1.5.0` OCI version labels corrected.
- **Helm chart pinned to 1.2.1** (`Chart.yaml` version/appVersion, `values.yaml` image tag), three releases behind. Synced to 1.7.0.
- **CI never ran on the default branch.** `ci.yml` and `security.yml` triggered on `main` and `develop`, but this repository's default branch is `master` — so the test suite, clippy, MSRV check, docs build, coverage, fuzz targets and benchmarks were skipped on every push, and the security audit only ever fired on its daily schedule. This is the root cause of nearly everything else fixed in this release: three red CI gates, two cargo features that did not compile, nine outstanding advisories, and a flagship command that had never worked all went unnoticed because nothing was checking. Both workflows now include `master`.
- **`mkdocs build --strict` failed with 51 warnings**, so CI's docs job was red despite v1.6.0 recording it as added and passing. 52 links from `docs/*.md` pointed at repo files outside `docs/` (`../src/kms.rs`, `../SECURITY.md`, …) which mkdocs cannot resolve for a published site; they now use absolute GitHub URLs. Fixed three genuinely broken targets, and added a `validation:` block that keeps link checking strict while allowing the one nav entry for rustdoc output that a different CI job injects after the build.
- **CI's mkdocs job installed only `mkdocs-material`** while `mkdocs.yml` declares the `minify` plugin — the build would have failed at config load, before rendering a single page. Verified the corrected install list builds strict-clean in a fresh virtualenv.
- **15 orphaned docs** added to the mkdocs nav (access control, KMS, policies, GC, tags, profiles, webhooks, plugins, lineage graph, TUI, vault bundle, validation, federation, blockchain audit, telemetry).

- **`s3` feature did not compile** — `src/cli/handlers/cloud.rs` used `ModelFormat` / `ModelMetadata` without importing them in the `cloud pull --store` path. Affected both `s3` and `azure`.
- **`azure` feature did not compile** — `put_block_blob` needs an owned body, `list_blobs().prefix()` takes a string rather than an `Option`, and `Pageable::next()` needs `StreamExt` in scope (`futures-util` added under the `azure` feature).
- **Deprecated AWS API** — `aws_config::from_env()` → `aws_config::defaults(BehaviorVersion::latest())`.
- **Clippy failures on current stable** — `manual_checked_ops`, `unnecessary_sort_by`, `manual_string_new`, `single_char_pattern`, `io_error_other`, `field_reassign_with_default`, `no_effect_underscore_binding`, `needless_range_loop`, `let_and_return`, `unit_arg`, `missing_const_for_thread_local`, `needless_borrow`. `float_cmp` is allowed per-file in test modules that assert on literal constants.
- **Two unused-code warnings in examples** — `examples/license_scan_demo.rs`, `examples/download_demo.rs`.


### Changed

- **BREAKING — Azure shared-key authentication removed.** `src/storage/azure.rs` now targets the Azure SDK for Rust v1, which provides no shared-key credential; `AZURE_STORAGE_KEY` is rejected with an error naming both alternatives rather than failing opaquely. Use `AZURE_STORAGE_SAS_TOKEN` (mint a SAS from the account key with `az storage container generate-sas`) or Entra ID via `AZURE_TENANT_ID` / `AZURE_CLIENT_ID` / `AZURE_CLIENT_SECRET`. This was the trade-off for clearing the last two advisories; docs updated across README, AGENTS.md, CLI.md, CLOUD_CLI.md, CLOUD_STORAGE.md and FEATURE_FLAGS.md.
- **Version** — 1.6.0 → 1.7.0 (Python package 1.3.0 → 1.7.0). Breaking for downstream code: `KmsBackend` gained a `File` variant, so exhaustive matches need a new arm; `aim sign`/`aim verify` take `--key` as a `String` rather than a `PathBuf`; `ConversionResult` gained a `plan` field, so struct literals need it; and `ConvertResponse.data_base64` is now optional.
- **CI clippy runs `--all-targets`** — examples, benches, and tests are now linted, not just the lib and bin.
- **CI feature matrix** extended with `python`.
- **Test count** — 2,059 → 2,088 Rust tests, 84 Python tests.

## [1.6.0] - 2026-04-06

### Added

- **Module integration tests** (`tests/module_integration_tests.rs`, 51 tests) — Cross-module integration tests covering tags/search, access control, lineage DAG, plugins, profiles, policies, validation, webhooks, quantization, evaluation, scheduler, multi-vault, signing, scanning, diff, license scanning, benchmarks, GC, and cross-module workflows
- **Property-based tests** (`tests/proptest_tests.rs`, 11 tests) — Proptest strategies for crypto round-trips, format detection, version serialization, SHA-256 invariants
- **Fuzz target expansion** (`fuzz/fuzz_targets/`, 3 new targets) — Pickle scanner, diff engine, model card parser (8 total)
- **Feature benchmarks** (`benches/feature_bench.rs`) — Criterion benchmarks for tags/search, ACL, lineage graph, plugins, profiles, policies, validation, webhooks, signing, scanning, diff, license scanning
- **CI benchmark tracking** — `benchmark-action/github-action-benchmark` job with 150% regression alert threshold
- **mkdocs nav expansion** — Added 8 missing docs to navigation: Examples, Model Download, Model Signing, Model Diffing, Engine Interop, Safety Scanning, License Scanning, Benchmarks
- **mkdocs build validation** — CI job with `mkdocs build --strict`
- **API reference generation** — Rustdoc auto-generated in CI, copied to mkdocs site, uploaded as artifact

### Changed

- **Version bump** — 1.5.0 → 1.6.0
- **MSRV** — Updated from 1.75 to 1.89 (ecosystem deps require edition 2024: `time-macros`, `async-graphql-value`, `asynk-strim`)
- **Test count** — 1,917 → 2,059

### Fixed

- **Import fixes** — Restored incorrectly removed imports in `vault.rs` (`VersionRepo`) and `database.rs` (`ChunkInfo`, `Document`)

### Security

- **`aws-lc-sys`** upgraded to v0.39.1 — fixed RUSTSEC-2026-0044 and RUSTSEC-2026-0048
- **Dependency audit** — 6 unmaintained transitive dep warnings documented in `deny.toml` ignore list; `cargo deny check` and `cargo audit` pass clean

## [1.5.0] - 2026-04-05

### Added

- **Quantization Pipeline** (`src/quantization.rs`, ~250 lines) — Profile-based quantization management with method selection (Q4_0, Q4_K_M, Q5_K_M, Q8_0, F16, F32), size estimation, and batch reporting. `QuantProfileStore` with `set`/`remove`/`get`/`list`. CLI: `aim quantize set/remove/list/estimate`
- **Evaluation Harness** (`src/evaluation.rs`, ~250 lines) — Record, compare, and query model evaluation results across suites and metrics. `EvalStore` with `record`/`get_runs`/`compare`/`suites`/`count`. CLI: `aim eval record/list/compare/suites`
- **Backup Scheduling** (`src/scheduler.rs`, ~250 lines) — Configurable vault backup schedules (hourly/daily/weekly/monthly) with rotation limits and history tracking. `BackupManager` with `set_schedule`/`remove_schedule`/`list_schedules`/`record_backup`. CLI: `aim backup set/remove/list/history`
- **Multi-Vault Management** (`src/multi_vault.rs`, ~200 lines) — Registry for managing multiple vaults with activate/deactivate switching. `VaultRegistry` with `register`/`unregister`/`activate`/`deactivate`/`list`. CLI: `aim vaults register/unregister/activate/deactivate/list`
- **4 new CLI handler files** in `src/cli/handlers/` — `quantization.rs`, `evaluation.rs`, `scheduler.rs`, `multi_vault.rs`
- **4 new Python binding classes** — `PyQuantProfileStore`, `PyEvalStore`, `PyBackupManager`, `PyVaultRegistry` (15 classes total)
- **12 new API endpoints** — REST routes for quantization profiles, evaluation runs, backup schedules, and multi-vault management under `/api/v1/`
- **28 new tests** from 4 new modules (1,865 → 1,917 total with integration tests)

### Changed

- **Version bump** — 1.4.0 → 1.5.0
- **CLI command count** — 38+ → 42+ commands
- **Test count** — 1,865 → 1,917
- **`src/lib.rs`** — 4 new `pub mod` declarations and 18 new type re-exports
- **`src/cli/args.rs`** — 4 new Commands variants, 4 new subcommand enums (QuantizeCommands, EvalCommands, BackupCommands, VaultsCommands)
- **`src/main.rs`** — Imports and match arms for all 4 new command variants
- **`src/python.rs`** — 4 new pyclass types registered in module init (11 → 15 classes)
- **`src/api/routes.rs`** — 12 new route handlers, 6 new request/response structs
- **`src/api/server.rs`** — 9 new route registrations under v1.5.0 endpoints section
- **Updated AGENTS.md** — New CLI commands, project layout, feature list
- **Updated CI/CD** — `.github/workflows/ci.yml` updated for new features

## [1.4.0] - 2026-04-04

### Added

- **Model Tags & Search** (`src/tags.rs`, ~250 lines) — Tag models with arbitrary labels and key-value annotations. Full-text search by name pattern, tags, or annotations. `TagStore` with `add_tags`/`remove_tags`/`search`. CLI: `aim tag add/remove/list/annotate`, `aim search`
- **Vault Export/Import** (`src/vault_bundle.rs`, ~200 lines) — Export entire vaults (or filtered subsets) as portable tar.gz bundles. Import bundles into new vaults with overwrite control. CLI: `aim vault-export`, `aim vault-import`
- **Garbage Collection** (`src/gc.rs`, ~200 lines) — Detect orphaned blobs, stale temp files, and reclaimable storage. Dry-run mode for safe preview. CLI: `aim gc [--dry-run]`
- **TUI Dashboard** (`src/tui.rs`, ~150 lines) — Terminal UI browser showing all vault models with version counts, sizes, formats, and timestamps. CLI: `aim browse`
- **Webhooks** (`src/webhooks.rs`, ~250 lines) — HTTP notification targets for vault events. Implements `EventSubscriber` for automatic dispatch on VaultEvent. CLI: `aim webhook add/remove/list/test`
- **Access Control** (`src/access_control.rs`, ~200 lines) — Role-based ACL (Reader/Writer/Admin) per principal with JSON persistence. CLI: `aim acl grant/revoke/list/check`
- **KMS Integration** (`src/kms.rs`, ~150 lines) — Fetch vault passphrases from external secrets managers (env, AWS Secrets Manager, Azure Key Vault, HashiCorp Vault). Library API only.
- **Model Validation** (`src/validation.rs`, ~250 lines) — Integrity probes with SHA-256 checksums per model version. CLI: `aim validate <NAME> [--version V]`
- **Retention Policies** (`src/policies.rs`, ~250 lines) — Configurable retention rules per model: max versions, max age, keep minimum. Dry-run enforcement. CLI: `aim policy set/remove/list/apply/apply-all`
- **Cross-Model Lineage DAG** (`src/lineage_graph.rs`, ~200 lines) — Directed acyclic graph tracking model derivation chains (fine-tune, quantization, distillation, merge, prune, conversion). CLI: `aim lineage-graph add/show/ancestors/descendants`
- **Plugin System** (`src/plugins.rs`, ~200 lines) — Discover, install, and uninstall plugins via JSON manifests with capability listing. CLI: `aim plugin discover/install/uninstall/list/info`
- **Config Profiles** (`src/profiles.rs`, ~200 lines) — Named configuration profiles with activate/deactivate switching and vault setting overrides. CLI: `aim profile create/remove/list/activate/deactivate/show`
- **11 new CLI handler files** in `src/cli/handlers/` for all new subcommands
- **56 new tests** from 12 new modules (1,809 → 1,865 total)

### Changed

- **Version bump** — 1.3.0 → 1.4.0
- **CLI command count** — 25+ → 38+ commands
- **Test count** — 1,809 → 1,865
- **`src/lib.rs`** — 12 new `pub mod` declarations and re-exports for all new public types
- **`src/version.rs`** — Added `list_models_owned()` and `import_version()` helper methods
- **`src/cli/args.rs`** — 13 new Commands variants, 8 new subcommand enums
- **`src/main.rs`** — Imports and match arms for all new command variants
- **Updated AGENTS.md** — New CLI commands, project layout, feature list
- **Updated README.md** — Feature comparison table, command count, test count, architecture tree
- **Updated ROADMAP.md** — v1.4.0 section with all 12 features

## [1.3.0] - 2026-04-04

### Added

- **Model download** (`src/download.rs`, ~350 lines) — Pull models from HuggingFace Hub, Ollama registry, or arbitrary URLs with streaming SHA-256 verification. `ModelSource` enum with `parse()`, `ModelDownloader` builder with `.with_hf_token()`. CLI: `aim pull <SOURCE> [-o DIR] [--sha256 HASH] [--token TOKEN] [--store] [--name NAME]`
- **Model signing & verification** (`src/signing.rs`, ~280 lines) — HMAC-SHA256 model signing with detached `.sig` files for provenance. `ModelSigner` static methods: `generate_keypair`, `sign`, `verify`, `save_signature`, `load_signature`. `SignatureVerification` struct with validity, hash match, and signer identity. CLI: `aim sign <NAME>`, `aim verify <NAME> --signature <SIG>`
- **Pickle safety scanning** (`src/scanning.rs`, ~300 lines) — Detect 7 dangerous opcodes (`REDUCE`, `GLOBAL`, `INST`, `OBJ`, `NEWOBJ`, `STACK_GLOBAL`, `NEWOBJ_EX`) and 12 suspicious patterns (`os.system`, `subprocess`, `eval`, etc.) in PyTorch/pickle files. `ScanReport` with severity classification. CLI: `aim scan [<NAME>] [--file PATH]`
- **Model diffing** (`src/diff.rs`, ~350 lines) — Tensor-level comparison for SafeTensors and GGUF models with generic binary fallback. SafeTensors header parsing, GGUF header parsing, `TensorMap` comparison. `DiffSummary` with human-readable display. CLI: `aim diff <LEFT> <RIGHT>` (supports `name@version` syntax)
- **Engine interop** (`src/interop.rs`, ~250 lines) — Register models with Ollama (`ollama create` via Modelfile generation) and LM Studio (copy to models directory). Cross-platform default path detection. CLI: `aim register <NAME> --engine <ollama|lm-studio> [--alias NAME]`
- **Benchmark metadata** (`src/benchmark.rs`, ~250 lines) — Store and query benchmark results (MMLU, HellaSwag, etc.) per model version with JSON filesystem storage. `BenchmarkStore` with `add_result`/`add_detailed_result`. CLI: `aim benchmark add <NAME> --benchmark <BENCH> --score <N>`, `aim benchmark show <NAME>`
- **License scanning** (`src/license_scan.rs`, ~370 lines) — Detect licenses from YAML frontmatter, `config.json`, GGUF metadata, and LICENSE files. 24 known licenses, SPDX normalization, `LicenseClass` classification (Permissive/Copyleft/NonCommercial/Proprietary/Unknown), compatibility warnings. CLI: `aim license-scan <PATH>`
- **7 CLI handler files** — `src/cli/handlers/{pull,sign,scan,diff,register,benchmark,license_scan}.rs`
- **46 new CLI integration tests** — expanded `tests/cli_tests.rs` from 17 to 63 tests covering all new subcommands

### Changed

- **Version bump** — 1.2.1 → 1.3.0
- **Test count** — 1,831 → 1,809 (consolidated; 623 lib + 63 CLI + 873 coverage + 250 other)
- **CLI command count** — 15+ → 25+ commands
- **Main thread stack** — Spawned `run()` on 4 MiB thread to prevent stack overflow on Windows from enlarged `Commands` enum
- **`tempfile`** moved from dev-dependencies to regular dependencies (used by handlers at runtime)
- **Updated AGENTS.md** — Added features #11–#17, 8 new CLI commands, 7 new source files in project layout
- **Updated README.md** — New features in comparison table, CLI section, architecture tree, additional capabilities

### Fixed

- **License scan Windows dedup** — Added `break` after first match in README/LICENSE file loops to prevent case-insensitive filesystem duplicates
- **Handler API patterns** — Fixed all 7 new handlers to use correct Vault API (`prompt_passphrase` with string arg, `build_vault` with config+sqlite, separate `unlock`, `get_model` not `get`, etc.)

## [1.2.1] - 2026-03-13

### Added

- **Fuzz testing targets** — 5 `cargo-fuzz` targets in `fuzz/`: `fuzz_crypto_roundtrip` (AES-256-GCM encrypt/decrypt roundtrip), `fuzz_format_detection` (ModelFormat::from_extension with arbitrary input), `fuzz_model_metadata` (ModelMetadata builder with fuzzed strings), `fuzz_version_parsing` (ModelVersion JSON deserialization), `fuzz_conversion_pipeline` (format string parsing and conversion path lookup)
- **API route tests** — 17 unit tests for `src/api/routes.rs`: `parse_format` (all 34 format aliases), `validate_model_name` (valid/empty/too-long/special-chars), `is_security_event` (all event types + negatives), `uuid_v4_simple` uniqueness, response struct serialization, stateless route handlers (`health`, `list_conversions`, `openapi_json`, `dashboard_index`)
- **API error tests** — 15 unit tests for `src/api/error.rs`: all 6 constructor methods (`bad_request`, `not_found`, `unauthorized`, `internal`, `conflict`, `rate_limited`), `IntoResponse` impl, `From<VaultError>` for all 7 match arms, `ApiErrorBody` serialization
- **API server tests** — 7 unit tests for `src/api/server.rs`: `RateLimiter` under/over limit, per-IP isolation, window reset, prune expired/active entries
- **Domain error Display tests** — exercises all `CryptoError`, `StorageError`, and `ConversionError` Display variants
- **Code coverage baseline** — 92.82% line coverage (12,094/13,029 lines) measured with cargo-llvm-cov (full features); 87.35% function coverage; 8 modules at 100% coverage
- **Performance baselines** — updated `docs/PERFORMANCE.md` with measured crypto benchmark results (AES-256-GCM, Argon2id, gzip/LZMA compression), vault benchmark results (store/retrieve, format detection, SHA-256, model card serialization), and per-module coverage table
- **Coverage improvements** — 53 new tests for low-coverage modules: `federation.rs` (VectorClock, delta computation, FederationManager lifecycle), `telemetry.rs` (event serialization, client enable/disable, tracking), `compliance.rs` (serialization, severity variants, checker toggle); total lib tests 447 → 505, full-feature tests 1,667
- **Vault benchmark fix** — fixed TempDir lifetime bug in `vault_bench.rs` (replaced `_` with `_tmp` to prevent premature directory cleanup)
- **Python bindings: VaultBuilder export** — registered `PyVaultBuilder` in the PyO3 module init and added `VaultBuilder` to `__init__.py` exports
- **Python bindings documentation** — new `docs/PYTHON_BINDINGS.md` with complete API reference for all 8 PyO3 classes, installation guide, quick start, and feature matrix
- **Python bindings: parse_format tests** — 25 Rust-side unit tests in `src/python.rs` covering all 23+ format aliases and case-insensitive parsing
- **Python test suite expansion** — added compression roundtrip tests, package init tests, vault property/error tests, and compression level tests
- **CLI integration tests** — 50 `assert_cmd` tests covering all major subcommands, help text, error handling, and format listings

### Changed

- **Python package version** — bumped from 1.1.0 to 1.2.0 in both `pyproject.toml` and `__init__.py`
- **Documentation polish** — updated 17 stale references across 10 files: test count 1,609→1,667, lib tests 447→505, coverage ~90%→92.82%, tarpaulin→cargo-llvm-cov

## [1.2.0]

### Added

- **Domain-specific error types** — introduced `CryptoError`, `StorageError`, and `ConversionError` enums in `src/error.rs` with typed variants and `From` conversions into the top-level `VaultError`. All three types are re-exported from the crate root.
- **REST API endpoints for model cards** — `GET /api/v1/models/{name}/card` generates a model card from vault metadata; `POST /api/v1/models/{name}/card` creates/overwrites a custom model card from JSON
- **REST API endpoint for compliance checks** — `GET /api/v1/compliance` runs FIPS 140-3, CVE, MITRE ATT&CK, and CMMC 2.0 checks and returns results as JSON
- **REST API endpoints for RAG** — `POST /api/v1/rag/search` searches the RAG document store; `POST /api/v1/rag/documents` adds a document with metadata
- **GraphQL routing** — wired existing `async-graphql` schema into the Axum router at `/graphql` (GET for Playground, POST for queries/mutations), gated behind `#[cfg(feature = "graphql")]`

### Changed

- **Removed `async-graphql-axum` dependency** — replaced with a manual bridge handler to avoid axum 0.7 / 0.8 version conflict; the `graphql` feature now only requires `async-graphql`
- **Fixed deprecated `TimeoutLayer::new`** — migrated to `TimeoutLayer::with_status_code(REQUEST_TIMEOUT, ...)` per tower-http 0.6.7+
- **Added `timeout` feature to tower-http** in Cargo.toml (was missing, caused compilation failure with `api` feature)
- **Removed unused `ConnectInfo` import** from `src/api/server.rs`
- **Version bump** — 1.1.0 → 1.2.0

### Changed

- **Real SafeTensors ↔ PyTorch converters** — replaced shim/plan converters with real pure-Rust implementations
  - SafeTensors → PyTorch: generates valid ZIP archives with pickle v2 bytecode and tensor data files
  - PyTorch → SafeTensors: parses ZIP archives, extracts tensor metadata from pickle bytecode, produces SafeTensors binary output
  - Full roundtrip conversion support with dtype mapping (F32↔FloatStorage, F16↔HalfStorage, BF16↔BFloat16Storage, etc.)
- **Telemetry changed to opt-in** — disabled by default for privacy
  - `TelemetryConfig::default()` now sets `enabled: false`
  - Unified environment variable handling: both `AIM_TELEMETRY_ENABLED=false` and `AIM_TELEMETRY_DISABLED=1` are respected in all code paths
  - Updated module documentation to reflect opt-in model
  - CLI `telemetry status` now shows both env var options
- **CI/CD hardening**
  - Added `permissions` and `concurrency` blocks to all GitHub Actions workflows
  - Release workflow now generates SHA-256 checksums for all binary artifacts
  - Release binaries properly renamed (e.g., `aim-linux-amd64`, `aim-darwin-arm64`)
  - Removed automatic crates.io publishing from release workflow
  - Consolidated Docker workflow: removed redundant API image job, added per-variant features
  - Fixed duplicate Alpine target in Docker workflow matrix
  - Updated `dependency-review-action` from v3 to v4
  - Added `--locked` flag to cargo install commands in CI
  - Added cargo cache to coverage job
- **deny.toml**: Rewrote for cargo-deny 0.19 schema — removed deprecated fields (`vulnerability`, `unmaintained`, `yanked`, `notice`, `unlicensed`, `copyleft`, `allow-osi-fsf-free`, `default`, `deny`), added `version = 2` to `[licenses]`, added `CC0-1.0`, `CDLA-Permissive-2.0`, `OpenSSL`, `Zlib`, `MPL-2.0` to license allow list
- **Updated qdrant-client** from 1.7 to 1.13 — migrated to builder-pattern API (`CreateCollectionBuilder`, `UpsertPointsBuilder`, `SearchPointsBuilder`, `DeletePointsBuilder`)
- **Replaced deprecated `serde_yaml`** (0.9) with maintained `serde_yml` (0.0.12) — drop-in replacement across all source and test files
- **Updated `zip` crate** from 0.6 to 4 — migrated `FileOptions` → `SimpleFileOptions` API in conversion.rs and utils.rs
- **Updated `bytes`** 1.10.1 → 1.11.1 (fixes RUSTSEC-2026-0007)
- **Updated `time`** 0.3.44 → 0.3.47 (fixes RUSTSEC-2026-0009)
- **Removed unused lancedb dependency** — v0.4 depends on arrow-arith v51 which is incompatible with Rust 1.93+
- **README overhaul** — updated test counts (331 → 1,580), fixed architecture diagram, added Architecture v2 features (API, GraphQL, federation, blockchain, GPU, streaming, VaultBuilder), fixed broken demo script paths, removed stale "NEW" labels, fixed AIMV_PATH_UPDATE link
- **AGENTS.md** — updated project layout, added `vector-db` feature, added telemetry env vars
- **Removed unused `futures` dependency** — confirmed zero usage in src/, not in any feature gate
- **Consolidated 12 coverage test files** into single `coverage_tests.rs` — reduced test binaries from 27 to 16, preserving all 1,609 tests
- **Expanded Makefile `examples` target** — now runs all 10 examples (was 2)
- **Fixed OpenAPI spec** — aligned `.well-known/openapi.yaml` with actual API routes: corrected model store path (`POST /api/v1/models/{name}`), version download path, version delete endpoint, added undocumented routes (health, audit, metrics, events, openapi.json), removed unimplemented routes (model cards, compliance, RAG, GraphQL)
- **Fixed Helm chart health probes** — corrected probe paths from `/health` to `/api/v1/health`, updated image tag to 1.1.0, added `startupProbe` for slow cold starts
- **Rewrote docs/PROJECT_STRUCTURE.md** — updated entire file to reflect current codebase: added 15+ missing src/ modules (crypto/gpu.rs, streaming.rs, cli/, api/, rag/, model_card.rs, blockchain.rs, federation.rs, telemetry.rs, traits.rs, version_sqlite.rs), updated tests/ from 6 to 14 files, examples/ from 5 to 10, docs/ with all new files, fixed license from MIT to AGPL-3.0, added deploy/, website/, .well-known/ directories
- **Updated reports/TEST_COVERAGE.md** — corrected test count from 119 to 1,609, updated test binary count to 16, added all missing test file entries, expanded coverage matrix with 8 new categories (model cards, CLI, VaultBuilder, blockchain, federation, telemetry, format conversion, RAG)
- **Updated all dependencies** — ran `cargo update` (141 packages updated within semver-compatible ranges), all 1,609 tests pass
- **Fixed format count references** — corrected "22 formats" to "23+ formats" across reports/TEST_COVERAGE.md, reports/COMPREHENSIVE_TEST_REPORT.md, reports/UTILITIES_IMPLEMENTATION_COMPLETE.md
- **Fixed MSRV references** — corrected "Rust 1.70+" to "Rust 1.75+" in docs/PROJECT_SUMMARY.md, updated Dockerfile example in docs/SECURITY_HARDENING.md to `rust:1.85-slim-bookworm`
- **Updated website version** — changed version badge from v1.0.0 to v1.1.0 in Header.tsx and page.tsx
- **Fixed stale test count in MIGRATION.md** — corrected "330+ tests" to "1,609+ tests"
- **Fixed stale test count in ROADMAP.md** — corrected "227 tests" to "1,609 tests"
- **Fixed website test count** — corrected "331+" to "1,609" in homepage stats
- **Fixed DEVELOPMENT.md MSRV** — corrected "Rust 1.70" to "Rust 1.75"
- **Fixed remaining "22+" → "23+" format count references** — docs/EXECUTIVE_SUMMARY.md, docs/api/formats.rst, reports/COMPREHENSIVE_TEST_REPORT.md, reports/PRODUCTION_READY.md, reports/PROJECT_COMPLETE.md, website Python docs
- **Root directory cleanup** — moved 5 Python coverage scripts (`analyze_cov.py`, `analyze_coverage.py`, `parse_coverage.py`, `parse_extra.py`, `parse_uncovered.py`) to `scripts/`, deleted tarpaulin artifacts, added `tarpaulin-report.json`, `tarpaulin_stderr.log`, `.cache/` to `.gitignore`
- **README overhaul (round 2)** — removed duplicate Project Structure section, removed duplicate Documentation section with garbled emoji headings, consolidated documentation table with 7 new entries (Architecture, Providers & Formats, Version Control, Cloud Storage, Model Cards, XDG, Roadmap, Changelog), fixed last `(22+)` → `(23+)` format count, updated architecture tree with new `scripts/` directory
- **Fixed all remaining "22+" → "23+" format count references** — ROADMAP.md, examples/huggingface_demo.rs, docs/EXECUTIVE_SUMMARY.md, docs/TOP_10_FEATURES.md, docs/guide/formats.rst, docs/archived/LAUNCH_READINESS.md, docs/archived/LAUNCH_READY.md, reports/COMPREHENSIVE_TEST_REPORT.md, reports/FEATURES_DEMO.md, reports/PRODUCTION_READY.md, reports/PROJECT_COMPLETE.md, reports/TESTING_COMPLETE.md, reports/UTILITIES_IMPLEMENTATION_COMPLETE.md

### Fixed

- Fixed all clippy warnings (29 warnings → 0)
  - Replaced `field_reassign_with_default` patterns with struct init syntax across src/ and tests/
  - Replaced `vec_init_then_push` with pre-initialized `vec![]` literals
  - Fixed `unused_must_use` on `cache_results()` calls
  - Fixed `unnecessary_get_then_check` in traits.rs
  - Fixed `unwrap_on_ok` / `expect_on_ok` in error.rs test
  - Removed unused imports (`EventSubscriber`, `VersionRepo`, `super`)
  - Fixed constant assertions (`assert!(X > 0)` → `assert_ne!(X, 0)`)
  - Suppressed deprecated `assert_cmd::Command::cargo_bin` warning
- Fixed 26 broken internal links across 8 docs/ files
  - Added `../` prefix for root-level files referenced from docs/ (README.md, LICENSE, SECURITY.md, CONTRIBUTING.md, DEVELOPMENT.md, FORMATS.md)
  - Removed redundant `docs/` prefix for same-directory references (QUICKSTART.md, CLI.md, UTILITIES.md)
  - Fixed reports/ directory references (FEATURES_DEMO.md, PRODUCTION_READY.md)
  - Removed links to non-existent files (COMPLIANCE.md, CRYPTO.md, API.md)
  - Fixed incorrect license references (MIT → AGPL-3.0-or-later)
  - Replaced manual `div_ceil` with standard library method in crypto/streaming.rs
  - Used `keys()` iterator instead of destructuring in conversion.rs
  - Added `#[allow(clippy::too_many_arguments)]` where appropriate

### Added

- **Next.js documentation website** (`website/`)
- **docs/FEATURE_FLAGS.md** — comprehensive documentation of all Cargo feature flags with build recipes
- **docs/PERFORMANCE.md** — benchmark baseline for encryption, hashing, compression, model card serialization
- **docs/GPU_ACCELERATION.md** — user guide for OpenCL GPU-accelerated encryption
- **docs/archived/** — moved stale launch readiness docs out of main docs/
- **ROADMAP: Future Improvements** section — documented error type granularity, API expansion, GraphQL routing as v1.2.0+ items
  - 21 documentation pages covering all features
  - Responsive layout with sidebar navigation and mobile menu
  - Light/dark theme with CSS custom properties
  - Reusable components: CodeBlock, Callout, FeatureCard
  - Static generation — all 25 routes prerendered
- Updated README badges and stats (331 tests, v1.0.0, Rust 1.75+)
- Updated ROADMAP version header to v1.0.0

## [1.0.0] - 2026-02-10

### Changed

- **Version bump to 1.0.0** — first production-stable release
  - Cargo.toml: `0.1.0` → `1.0.0`
  - pyproject.toml: `0.1.0` → `1.0.0`, classifier `Alpha` → `Production/Stable`
  - CLI version: `0.1.0` → `1.0.0`
  - OpenAPI spec: `0.5.0` → `1.0.0`

### Added

- **Multi-stage Dockerfile** with Alpine (default, ~12 MB) and Debian variants
  - Static musl binary via `x86_64-unknown-linux-musl` target
  - Non-root user, tini init, XDG volume mounts
  - Configurable `FEATURES` build arg (e.g., `--build-arg FEATURES=api`)
  - `.dockerignore` for minimal build context
- **Kubernetes Helm chart** (`deploy/helm/ai-model-vault/`)
  - Deployment with hardened security context (non-root, read-only FS, drop all caps)
  - Service (ClusterIP), Secret (auto-generated JWT), ServiceAccount
  - PersistentVolumeClaims for data, config, and cache
  - Optional Ingress with TLS support
  - HorizontalPodAutoscaler
  - Values: image, replicas, API config, persistence, resources, probes, autoscaling
- **Docker CI/CD workflow** (`.github/workflows/docker.yml`)
  - Builds and pushes Alpine, Debian, and API images to GHCR on tag push
  - Docker Buildx with GitHub Actions cache
  - OCI metadata labels via `docker/metadata-action`
- **Comprehensive migration guide** (`docs/MIGRATION.md`)
  - Covers Rust crate, Python package, CLI, REST API, Docker, and Kubernetes
  - Breaking changes summary, data migration notes, environment variables
- **Publication readiness metadata**
  - Cargo.toml: added `readme`, `homepage`, `documentation`, `rust-version` fields
  - pyproject.toml: added `[project.urls]` section (Homepage, Docs, Repo, Issues, Changelog)
  - Keywords trimmed to 5 for crates.io compliance

## [0.5.0] - 2026-02-10

### Added

- **REST API server** (`src/api/`, ~1200 lines, behind `api` feature flag)
  - Axum 0.7 HTTP server with 14 RESTful endpoints
  - JWT authentication (`jsonwebtoken` 9.3) with Bearer token auth
  - Endpoints: health, auth/token, models (list/get/store), versions (list/get/delete), lineage, conversions (list/convert), stats, audit
  - Multipart file upload for model storage
  - Base64-encoded conversion API for format conversion over HTTP
  - CORS support via `tower-http` with `--cors-permissive` flag
  - Request body size limits (default 512 MiB)
  - HTTP request tracing via `tower-http::trace`
- **OpenAPI 3.1 specification** at `/api/v1/openapi.json`
  - Complete API documentation with schemas, parameters, and security definitions
- **Embedded web dashboard** served at `/`
  - Single-page HTML/JS/CSS application (no build step required)
  - Model inventory browser with version drill-down
  - Storage statistics (models, versions, size, files)
  - Audit log viewer, conversion registry browser
  - Passphrase-based login with JWT session management
- **CLI `serve` command** (`aim serve`)
  - Flags: `--host`, `--port`, `--jwt-secret`, `--token-expiry`, `--cors-permissive`, `--no-dashboard`
  - Environment variables: `AIM_HOST`, `AIM_PORT`, `AIM_JWT_SECRET`
- **15 API tests** (3 auth unit + 12 integration via tower `oneshot`)
- Dependencies: axum 0.7, tower 0.5, tower-http 0.6, jsonwebtoken 9.3, utoipa 5, base64 0.22, hyper 1.4

## [0.4.0] - 2026-02-10

### Added

- **Format conversion pipeline** (`src/conversion.rs`, ~1350 lines)
  - `Converter` trait with `convert()`, `validate()`, `name()`, `source_format()`, `target_format()`
  - `ConversionPipeline` with BFS multi-step path finding and `with_builtins()` factory
  - `ConversionOptions`: quantization, opset_version, tolerance, preserve_metadata, extra params
  - `ConversionResult`: output data, conversion path, input/output sizes, optional validation report
  - `ConversionProgress` with step tracking and Display impl
  - `ValidationReport` and `ValidationCheck` structures
- **10 built-in format converters**
  - Pure Rust: SafeTensors↔Raw roundtrip, GGUF header parser, ONNX metadata extractor
  - Shim converters (JSON conversion plans): SafeTensors↔PyTorch, PyTorch→ONNX, ONNX→TensorRT, ONNX→CoreML, SafeTensors→GGUF
- **Magic-bytes validation** for SafeTensors, GGUF, PyTorch (ZIP/pickle), ONNX (protobuf), TFLite
- **CLI commands**
  - `aim convert` with `--opset`, `--validate`, `--plan-only` flags
  - `aim list-conversions` to show all registered converters and multi-step paths
- **53 conversion tests** (22 unit + 31 integration)

## [0.3.0] - 2026-02-10

### Added

- **Native Python bindings via PyO3** (`src/python.rs`, ~640 lines)
  - `Vault`: create, unlock, lock, store_model, get_model, list_models, list_versions, get_lineage, delete_version, get_stats, change_passphrase
  - `VaultConfig`: XDG-compliant configuration with optional custom vault directory
  - `ModelFormat`: 23+ format detection with name/extension properties
  - `ModelMetadata`: builder-style constructor (name, format, description, framework, task, architecture, parameters)
  - `ModelVersion`: read-only version snapshot (version, checkpoint_id, timestamp, format, size, checksum)
  - `ModelCard`: create, set_training_data, add_metric, add_metadata, serialization (JSON/YAML/Markdown), deserialization
  - `sha256_hex()`: FIPS-compliant SHA-256 hex digest
  - `version()`: native library version string
- `python` feature flag in Cargo.toml gating PyO3 dependency
- maturin build backend in `pyproject.toml` (replaced setuptools)
- Native import with graceful fallback in `__init__.py` (`_NATIVE` flag)
- **Streaming API** for large models
  - `Vault.store_model_streamed()`: ingest from any iterable of `bytes` chunks
  - `Vault.get_model_streamed()`: retrieve as `ModelStream` iterator (default 8 MiB chunks)
  - `ModelStream`: Python iterator with `total_size`, `remaining` properties
  - Rust `ModelStream` + `Vault::store_model_streamed()` / `Vault::get_model_chunked()`
- **Sphinx documentation** (`docs/`)
  - API reference: Vault, VaultConfig, ModelFormat, ModelMetadata, ModelVersion, ModelCard, utilities
  - User guides: vault lifecycle, format detection, model cards, version control
  - Quick start and installation guides (uv-based)

### Changed

- Python package now uses native Rust FFI instead of CLI subprocess wrappers when built with maturin
- `pyproject.toml`: build system switched from setuptools to maturin ≥1.7

## [0.2.0] - 2026-02-10

### Changed

- **License**: Switched from MIT to AGPL-3.0-or-later with commercial dual-license option + CLA
- **Architecture**: Split `rag.rs` (2,168 lines) into 7 submodules with backward-compatible re-exports
- **Architecture**: Split `main.rs` (2,931 lines) into 87-line dispatcher + `cli/` module tree (11 files)
- **Performance**: `ModelFormat::name()` and `extension()` return `&'static str` (zero allocation)
- **Performance**: `model_card.rs` uses `write!()` instead of `format!()+push_str()`, `String::with_capacity(2048)`

### Added

- `COMMERCIAL_LICENSE.md` for proprietary/commercial licensing inquiries
- `Vault::key_manager()` getter (resolves dead_code suppression)
- `VersionControl::vault_path()` getter (resolves dead_code suppression)
- `ComplianceChecker` gated methods with `enabled_checks` map
- 19 new tests (246 total): `change_passphrase`, `audit` logging, `FormatConverter`, `cleanup_old_versions`, `verify_checksum`, compliance check toggling
- `vault_bench` benchmarks: store/retrieve, format detection, SHA-256, model card ser/de

### Fixed

- Resolved all 5 `#[allow(dead_code)]` annotations in production code
- Removed redundant `CachedResult.query_hash` field; used timestamp in LRU eviction as tiebreaker

### Removed

- 10 temporary artifacts from root (test outputs, status files)
- Moved 23 status/completion files to `reports/`
- Moved 12 guides/demo scripts to `docs/`

## [0.1.1] - 2026-02-07

### Fixed

- **Critical**: Replaced panicking `.expect()` in `Vault::new()` with `match` returning `Result`
- **Critical**: Guarded `validate_sql_identifier()` against empty-string panic
- Deprecated `actions-rs/toolchain@v1` → `dtolnay/rust-toolchain@stable` in CI
- Deprecated `actions/create-release@v1` → `softprops/action-gh-release@v2`
- Fixed binary name `aimv` → `aim` in release.yml
- Made heavyweight Python deps optional in `pyproject.toml` (`[project.optional-dependencies] ml`)

### Added

- 40+ Python tests for ModelFormat, VaultConfig, Vault, FIPSCrypto
- `#[must_use]` annotations on all 15 pure functions
- `///` doc comments on 17+ public types and builder methods
- Warning docstring to `fips.py` documenting PBKDF2 vs Argon2id incompatibility

### Changed

- Synced Python `ModelFormat` enum 1:1 with Rust's 23-variant enum
- Committed `Cargo.lock` for reproducible binary builds
- Updated test count references from 171/119 → 227

## [0.1.0] - 2025-11-03

### Added

- **Core Vault System**
  - FIPS 140-3 compliant encryption using AES-256-GCM
  - Argon2id key derivation function (64MB memory, 3 iterations)
  - XDG Base Directory compliance for cross-platform support
  - Version control system with complete checkpoint history
  - Secure key storage with memory zeroization
  - Comprehensive audit logging for compliance

- **Model Format Support (23+ formats)**
  - PyTorch (.pt, .pth, .bin)
  - TensorFlow (.pb, .keras, .h5)
  - ONNX (.onnx)
  - Safetensors (.safetensors)
  - GGUF (.gguf) - Quantized LLMs
  - TensorRT (.plan)
  - TFLite (.tflite)
  - MLX (.npz) - Apple Silicon
  - Core ML (.mlmodel, .mlpackage)
  - And 13+ more formats
  - Automatic format detection
  - Metadata management

- **Compression**
  - Gzip (fast, moderate compression)
  - LZMA (slow, high compression)
  - Zlib (balanced)
  - Configurable compression levels (Fast/Balanced/Maximum)
  - Compression analysis and recommendations

- **Model Utilities (8 Components)**
  - ModelArchive: TAR/ZIP archiving for model backup
  - CompressionAnalyzer: Compression ratio analysis
  - RetrievalOptimizer: LRU cache for fast model access
  - QuantizationInfo: Track 10 quantization schemes
  - PruningInfo: Pruning metadata and sparsity calculation
  - ModelAnalyzer: Size and parameter analysis
  - ModelExporter: Export with JSON metadata
  - ModelDeduplicator: SHA-256 duplicate detection

- **Cloud Storage Support** ⭐ NEW
  - **StorageBackend trait**: Pluggable storage architecture
  - **AWS S3 backend**: Full S3 support with multipart uploads
  - **Azure Blob Storage backend**: Azure cloud storage integration
  - **Google Cloud Storage backend**: GCS support
  - **Async operations**: Non-blocking cloud uploads/downloads
  - **Multiple authentication methods**: IAM roles, access keys, service accounts
  - **Optional features**: Build only what you need (s3, azure, gcs, cloud)
  - **Complete documentation**: 600+ line cloud storage guide
- CLI interface with full command set
  - Core commands: `init`, `store`, `get`, `list`, `versions`, `lineage`, `delete`, `stats`, `compliance`
  - **Utility commands**: `archive`, `extract`, `analyze`, `deduplicate`, `export`, `cache`
- **Model Utilities Module** with comprehensive AI model operations:
  - **ModelArchive**: TAR/ZIP archiving for multiple models
  - **CompressionAnalyzer**: Compression ratio analysis and format-specific estimates
  - **RetrievalOptimizer**: LRU cache for fast model retrieval
  - **QuantizationInfo**: Quantization metadata tracking (10 schemes: FP32, FP16, INT8, Q4_0, etc.)
  - **PruningInfo**: Pruning information and sparsity calculation
  - **ModelAnalyzer**: Model analysis with human-readable size/parameter formatting
  - **ModelExporter**: Export models with JSON metadata
  - **ModelDeduplicator**: SHA-256 based duplicate detection and similarity scoring
- **RAG & AI Agent Integration** ⭐ NEW
  - Document store with vector embeddings
  - Knowledge base with text chunking
  - Rule engine for business logic
  - Retrieval cache with LRU eviction
  - Model Context Protocol (MCP) tools
  - Database abstraction layer
  - 23 comprehensive RAG tests

- **CLI Interface (15 Commands)**
  - Core: init, unlock, store, get, list, versions, lineage, delete, stats
  - Utilities: archive, extract, analyze, deduplicate, export, cache
  - Compliance: compliance check
  - Interactive help system
  - User-friendly error messages

- **Comprehensive Test Suite (148 tests)**
  - 37 library unit tests
  - 22 configuration and error tests
  - 14 cryptography tests
  - 15 format detection tests
  - 8 integration tests
  - 38 utilities tests
  - 23 RAG tests
  - 100% passing rate

- **Example Programs (4 demos)**
  - `basic_usage.rs`: Core vault operations
  - `security_demo.rs`: Security features
  - `utilities_demo.rs`: Model utilities showcase
  - `rag_demo.rs`: RAG pipeline demonstration

- **Complete Documentation (5,000+ lines)**
  - Quick start guide (5-minute tutorial)
  - CLI reference (all 15 commands)
  - Utilities guide (600+ lines)
  - RAG guide (600+ lines)
  - MCP tools guide (500+ lines)
  - Cloud storage guide (600+ lines)
  - HDF5 support guide
  - Security policy
  - Development guide
  - Test coverage report

### Security

- **FIPS 140-3** approved cryptographic algorithms
- **Authenticated encryption** with AES-256-GCM (128-bit auth tags)
- **Secure key derivation** with Argon2id (64MB memory, 3 iterations)
- **SHA-256 integrity** verification for all stored models
- **Memory zeroization** for sensitive data (keys, passphrases)
- **Audit logging** for all security-relevant operations
- **CMMC 2.0 Level 2** compliance (17 controls implemented)
- **MITRE ATT&CK** framework alignment (T1552, T1486, T1078, T1005)
- **CVE scanning** with automated vulnerability checks

### Changed

- Made HDF5 support optional (requires system library installation)
- Separated HDF5 into `hdf5-support` feature flag
- Updated build to work without HDF5 by default
- Optimized compression for large model files

### Fixed

- HDF5 build dependency issue (now truly optional)
- Build failures on systems without HDF5 installed
- Generic array deprecation warnings
- Cross-platform path handling improvements

### Documentation

- Added comprehensive HDF5 support guide
- Created launch readiness checklist
- Updated README with HDF5 installation instructions
- Expanded cloud storage documentation
- Added troubleshooting guides

## Future Releases

### Planned for v0.3.0

- Native Python bindings (PyO3/maturin)
- Direct Python API without subprocess
- PyPI publication as `aimodelvault`

### Planned for v0.4.0

- Real model format conversion pipeline
- PyTorch ↔ ONNX, SafeTensors ↔ PyTorch, GGUF ↔ SafeTensors

---

[0.2.0]: https://github.com/nervosys/AIModelVault/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/nervosys/AIModelVault/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/nervosys/AIModelVault/releases/tag/v0.1.0
