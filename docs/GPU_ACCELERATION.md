# GPU-Accelerated Encryption

AI Model Vault can offload AES-256 encryption to the GPU via OpenCL when processing large model files.

## Requirements

- An OpenCL-compatible GPU (NVIDIA, AMD, Intel)
- OpenCL runtime/drivers installed

## Building

```bash
cargo build --release --features gpu
```

## How It Works

The `GpuCrypto` module provides AES-256-CTR encryption using an OpenCL kernel. It integrates with the standard `encrypt()`/`decrypt()` path via automatic routing:

- **Data > 10 MB** and GPU available → GPU path
- **Data ≤ 10 MB** or GPU unavailable → CPU path (AES-NI)

The GPU handles the bulk cipher work while authentication (GCM tag) remains on the CPU for correctness.

## API

```rust
use ai_model_vault::crypto::gpu::GpuCrypto;

let mut crypto = GpuCrypto::new()?;

// Check GPU availability
if crypto.is_gpu_available() {
    let info = crypto.gpu_info();
    println!("GPU: {} ({} MB)", info.device_name, info.memory_bytes / 1_048_576);
}

// Encrypt — automatically uses GPU for large data
let encrypted = crypto.encrypt(&data, &key)?;
let decrypted = crypto.decrypt(&encrypted, &key)?;

// Force CPU-only
crypto.disable_gpu();

// Run comparative benchmark
let result = crypto.benchmark(&data, &key)?;
println!("{}", result); // e.g., "GPU: 1.5x speedup"
```

## When to Use

| Scenario          | Recommendation      |
| ----------------- | ------------------- |
| Models < 10 MB    | CPU (AES-NI) faster |
| Models 10–100 MB  | Marginal GPU gain   |
| Models > 100 MB   | GPU recommended     |
| Batch encryption  | GPU recommended     |
| No OpenCL runtime | CPU auto-fallback   |

## Benchmarking

```bash
# Run crypto benchmarks (CPU baseline)
cargo bench --bench crypto_bench

# With GPU feature enabled, GpuCrypto::benchmark() compares GPU vs CPU
cargo run --features gpu -- analyze <model-name>
```

The auto-selection threshold (10 MB) can be adjusted by recompiling with a custom `GPU_THRESHOLD_BYTES` constant in `src/crypto/gpu.rs`.
