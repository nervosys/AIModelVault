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

## Code Coverage

| Metric    | Value                    |
| --------- | ------------------------ |
| Tool      | cargo-llvm-cov           |
| Features  | full, graphql            |
| Lines     | 12,094 / 13,029          |
| Coverage  | **92.82%**               |
| Functions | 1,160 / 1,328 (87.35%)   |
| Regions   | 20,916 / 22,384 (93.44%) |

### Per-Module Coverage

| Module               | Lines (Covered/Total) | %     | Functions |
| -------------------- | --------------------- | ----- | --------- |
| `vault.rs`           | 1,420 / 1,446         | 98.2% | 90 / 92   |
| `blockchain.rs`      | 917 / 935             | 98.1% | 57 / 60   |
| `traits.rs`          | 900 / 920             | 97.8% | 85 / 91   |
| `version.rs`         | 434 / 435             | 99.8% | 42 / 42   |
| `utils.rs`           | 548 / 550             | 99.6% | 65 / 66   |
| `model_card.rs`      | 505 / 513             | 98.4% | 26 / 30   |
| `conversion.rs`      | 1,469 / 1,683         | 87.3% | 149 / 173 |
| `formats.rs`         | 296 / 296             | 100%  | 33 / 33   |
| `crypto/mod.rs`      | 254 / 262             | 97.0% | 33 / 38   |
| `crypto/streaming`   | 238 / 242             | 98.4% | 18 / 18   |
| `crypto/compression` | 122 / 126             | 96.8% | 17 / 21   |
| `storage.rs`         | 370 / 375             | 98.7% | 32 / 33   |
| `storage/local.rs`   | 95 / 97               | 97.9% | 21 / 21   |
| `rag/database.rs`    | 661 / 704             | 93.9% | 66 / 95   |
| `rag/mcp.rs`         | 429 / 441             | 97.3% | 58 / 62   |
| `rag/rules.rs`       | 286 / 286             | 100%  | 31 / 31   |
| `rag/cache.rs`       | 153 / 153             | 100%  | 19 / 19   |
| `rag/knowledge.rs`   | 131 / 131             | 100%  | 12 / 12   |
| `rag/documents.rs`   | 114 / 114             | 100%  | 20 / 20   |
| `rag/vector.rs`      | 137 / 137             | 100%  | 23 / 23   |
| `rag/mod.rs`         | 180 / 183             | 98.4% | 15 / 16   |
| `config.rs`          | 223 / 230             | 97.0% | 23 / 24   |
| `version_sqlite.rs`  | 558 / 674             | 82.8% | 43 / 75   |
| `compliance.rs`      | 226 / 274             | 82.5% | 29 / 33   |
| `federation.rs`      | 720 / 908             | 79.3% | 68 / 86   |
| `telemetry.rs`       | 357 / 538             | 66.4% | 47 / 75   |
| `error.rs`           | 145 / 150             | 96.7% | 16 / 16   |
| `audit.rs`           | 185 / 196             | 94.4% | 18 / 18   |
| `permissions.rs`     | 21 / 30               | 70.0% | 4 / 5     |

> Remaining low-coverage areas: `telemetry.rs` (opt-in, requires runtime init), `federation.rs` (requires multi-peer setup), `version_sqlite.rs` (partial SQLite backend paths).

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
