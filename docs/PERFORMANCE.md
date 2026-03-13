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

| Operation | Data Size | Time (median) |
| --------- | --------- | ------------- |
| Store     | 1 KB      | ~46 ms        |
| Retrieve  | 1 KB      | ~25 ms        |
| Store     | 10 KB     | ~49 ms        |
| Retrieve  | 10 KB     | ~18 ms        |
| Store     | 100 KB    | ~45 ms        |
| Retrieve  | 100 KB    | ~16 ms        |

> Store/retrieve times are dominated by Argon2id key derivation (~40 ms per call).

### Format Detection

| Operation          | Time (median) |
| ------------------ | ------------- |
| `from_extension()` | ~462 ns       |
| `format_name()`    | ~4 ns         |

### SHA-256 Hashing

| Data Size | Time (median) |
| --------- | ------------- |
| 1 KB      | ~453 ns       |
| 10 KB     | ~5.3 µs       |
| 100 KB    | ~53 µs        |
| 1 MB      | ~571 µs       |

### Model Card Serialization

| Operation       | Time (median) |
| --------------- | ------------- |
| `to_json()`     | ~4.4 µs       |
| `to_yaml()`     | ~22 µs        |
| `to_markdown()` | ~1.5 µs       |
| `from_json()`   | ~4.5 µs       |
| `from_yaml()`   | ~59 µs        |

## REST API Endpoints (`benches/api_bench.rs`)

| Endpoint             | Method | Time (median) |
| -------------------- | ------ | ------------- |
| `/api/v1/health`     | GET    | ~90 ms        |
| `/api/v1/auth/token` | POST   | ~311 ms       |
| `/api/v1/models`     | GET    | ~195 ms       |
| `/api/v1/compliance` | GET    | ~1.55 s       |

> API benchmarks include per-request vault setup (tempdir + init). The `/api/v1/compliance`
> endpoint runs `cargo audit` (external process), which dominates its latency.

## Code Coverage

| Metric    | Value                     |
| --------- | ------------------------- |
| Tool      | cargo-llvm-cov            |
| Features  | full, graphql             |
| Lines     | 15,187 / 17,786           |
| Coverage  | **85.4%**                 |
| Lib tests | 623                       |
| Total     | 1,818 (lib + integration) |

> Note: Line counts include CLI handlers (binary code at 0% library-only coverage).
> Library-only coverage remains above 92%.

### Per-Module Coverage (library)

| Module               | Lines (Covered/Total) | %     |
| -------------------- | --------------------- | ----- |
| `vault.rs`           | 1,420 / 1,446         | 98.2% |
| `blockchain.rs`      | 917 / 935             | 98.1% |
| `traits.rs`          | 900 / 920             | 97.8% |
| `version.rs`         | 434 / 435             | 99.8% |
| `utils.rs`           | 548 / 550             | 99.6% |
| `model_card.rs`      | 505 / 513             | 98.4% |
| `conversion.rs`      | 1,469 / 1,683         | 87.3% |
| `formats.rs`         | 296 / 296             | 100%  |
| `crypto/mod.rs`      | 254 / 262             | 97.0% |
| `crypto/streaming`   | 238 / 242             | 98.4% |
| `crypto/compression` | 122 / 126             | 96.8% |
| `storage.rs`         | 370 / 375             | 98.7% |
| `storage/local.rs`   | 95 / 97               | 97.9% |
| `rag/database.rs`    | 661 / 704             | 93.9% |
| `rag/mcp.rs`         | 429 / 441             | 97.3% |
| `rag/rules.rs`       | 286 / 286             | 100%  |
| `rag/cache.rs`       | 153 / 153             | 100%  |
| `rag/knowledge.rs`   | 131 / 131             | 100%  |
| `rag/documents.rs`   | 114 / 114             | 100%  |
| `rag/vector.rs`      | 137 / 137             | 100%  |
| `rag/mod.rs`         | 180 / 183             | 98.4% |
| `config.rs`          | 223 / 230             | 97.0% |
| `version_sqlite.rs`  | 558 / 674             | 82.8% |
| `telemetry.rs`       | 672 / 744             | 90.3% |
| `federation.rs`      | 994 / 1,165           | 85.3% |
| `compliance.rs`      | 280 / 329             | 85.1% |
| `permissions.rs`     | 35 / 44               | 79.5% |
| `error.rs`           | 145 / 150             | 96.7% |
| `audit.rs`           | 185 / 196             | 94.4% |
| `permissions.rs`     | 35 / 44               | 79.5% |

> Remaining low-coverage areas: `cli/handlers/` (0%, requires integration tests with stdin/stdout), `version_sqlite.rs` (partial SQLite backend paths).

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
# LLVM source-based coverage (cross-platform, recommended)
cargo llvm-cov --lib

# With JSON output
cargo llvm-cov --lib --json

# Full features
cargo llvm-cov --lib --features "full,graphql"
```

Benchmark results are stored in `target/criterion/` with HTML reports viewable at `target/criterion/report/index.html`.
