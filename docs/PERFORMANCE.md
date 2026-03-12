# Performance Baseline — v1.2.0

Benchmark results captured with `cargo bench` using Criterion 0.5 on Windows (x86_64).
All benchmarks run on the default (non-GPU) code path.

**Environment:** Windows 11, x86_64, Rust stable, `--release` profile

## Cryptography (`benches/crypto_bench.rs`)

### AES-256-GCM Encryption

| Data Size | Time (median) | Throughput |
| --------- | ------------- | ---------- |
| 1 KB      | ~1.03 µs      | ~1.0 GB/s  |
| 10 KB     | ~8.0 µs       | ~1.3 GB/s  |
| 100 KB    | ~89.6 µs      | ~1.1 GB/s  |
| 1 MB      | ~3.5 ms       | ~300 MB/s  |

### AES-256-GCM Decryption

| Data Size | Time (median) | Throughput |
| --------- | ------------- | ---------- |
| 1 KB      | ~1.8 µs       | ~570 MB/s  |
| 10 KB     | ~11.6 µs      | ~890 MB/s  |
| 100 KB    | ~109 µs       | ~940 MB/s  |
| 1 MB      | ~2.7 ms       | ~390 MB/s  |

### Key Derivation (Argon2id)

| Operation      | Time (median) |
| -------------- | ------------- |
| Key derivation | ~353 ms       |

> Argon2id is intentionally slow (64 MB memory, 3 iterations) to resist brute-force attacks.

### Compression

| Algorithm | Data Size | Time (median) |
| --------- | --------- | ------------- |
| gzip      | 1 KB      | ~7.7 µs       |
| gzip      | 10 KB     | ~9.7 µs       |
| gzip      | 100 KB    | ~28.4 µs      |
| LZMA      | 1 KB      | ~26.2 µs      |
| LZMA      | 10 KB     | ~388 µs       |
| LZMA      | 100 KB    | ~1.99 ms      |

## Vault Operations (`benches/vault_bench.rs`)

### Store & Retrieve (AES-256-GCM + Argon2id)

| Operation   | Data Size | Time (median) |
| ----------- | --------- | ------------- |
| Store       | 1 KB      | ~46 ms        |
| Retrieve    | 1 KB      | ~25 ms        |
| Store       | 10 KB     | ~49 ms        |
| Retrieve    | 10 KB     | ~18 ms        |
| Store       | 100 KB    | ~45 ms        |
| Retrieve    | 100 KB    | ~16 ms        |

> Store/retrieve times are dominated by Argon2id key derivation (~40 ms per call).

### Format Detection

| Operation           | Time (median) |
| ------------------- | ------------- |
| `from_extension()`  | ~462 ns       |
| `format_name()`     | ~4 ns         |

### SHA-256 Hashing

| Data Size | Time (median) |
| --------- | ------------- |
| 1 KB      | ~453 ns       |
| 10 KB     | ~5.3 µs       |
| 100 KB    | ~53 µs        |
| 1 MB      | ~571 µs       |

### Model Card Serialization

| Operation              | Time (median) |
| ---------------------- | ------------- |
| `to_json()`            | ~4.4 µs       |
| `to_yaml()`            | ~22 µs        |
| `to_markdown()`        | ~1.5 µs       |
| `from_json()`          | ~4.5 µs       |
| `from_yaml()`          | ~59 µs        |

## Code Coverage

| Metric     | Value            |
| ---------- | ---------------- |
| Tool       | cargo-tarpaulin  |
| Features   | default (sqlite) |
| Lines      | 3,676 / 4,385    |
| Coverage   | **83.83%**       |

### Per-Module Coverage

| Module                | Covered / Total | %     |
| --------------------- | --------------- | ----- |
| `vault.rs`            | 455 / 458       | 99.3% |
| `formats.rs`          | 115 / 115       | 100%  |
| `utils.rs`            | 198 / 198       | 100%  |
| `crypto/mod.rs`       | 97 / 98         | 99.0% |
| `crypto/compression`  | 40 / 40         | 100%  |
| `crypto/streaming`    | 91 / 94         | 96.8% |
| `model_card.rs`       | 125 / 125       | 100%  |
| `rag/mcp.rs`          | 151 / 151       | 100%  |
| `rag/database.rs`     | 256 / 262       | 97.7% |
| `blockchain.rs`       | 291 / 300       | 97.0% |
| `conversion.rs`       | 607 / 749       | 81.0% |
| `traits.rs`           | 256 / 276       | 92.8% |
| `version_sqlite.rs`   | 211 / 240       | 87.9% |
| `version.rs`          | 123 / 124       | 99.2% |
| `storage.rs`          | 111 / 113       | 98.2% |
| `config.rs`           | 90 / 94         | 95.7% |
| `compliance.rs`       | 46 / 77         | 59.7% |
| `telemetry.rs`        | 37 / 221        | 16.7% |
| `federation.rs`       | 28 / 268        | 10.4% |
| `error.rs`            | 21 / 21         | 100%  |

> Low coverage in `federation.rs` and `telemetry.rs` is expected — federation requires multi-node setup and telemetry is opt-in.

## Running Benchmarks

```bash
# All benchmarks
cargo bench

# Crypto only
cargo bench --bench crypto_bench

# Vault operations only
cargo bench --bench vault_bench

# API benchmarks (requires api feature)
cargo bench --bench api_bench --features api
```

## Running Coverage

```bash
# Default features (recommended)
cargo tarpaulin --lib --timeout 300 --out json

# Full features (requires ~8 GB RAM, Linux recommended)
cargo tarpaulin --features "full,graphql" --timeout 300
```

Results are stored in `target/criterion/` with HTML reports viewable at `target/criterion/report/index.html`.
