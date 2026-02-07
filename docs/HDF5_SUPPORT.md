# HDF5 Support

AI Model Vault includes optional support for HDF5 format (.h5, .hdf5 files). This feature requires the HDF5 library to be installed on your system.

## Installation

### Ubuntu/Debian
```bash
sudo apt-get install libhdf5-dev pkg-config
```

### macOS
```bash
brew install hdf5 pkg-config
```

### Windows
1. Download HDF5 from https://www.hdfgroup.org/downloads/hdf5/
2. Install to a standard location (e.g., `C:\Program Files\HDF5`)
3. Set environment variable: `HDF5_DIR=C:\Program Files\HDF5`

## Building with HDF5 Support

Once the HDF5 library is installed:

```bash
# Build with HDF5 support
cargo build --features hdf5-support

# Build with all features including HDF5
cargo build --features full,hdf5-support

# Run tests with HDF5 support
cargo test --features hdf5-support
```

## Building Without HDF5 (Default)

By default, AI Model Vault builds without HDF5 support:

```bash
# Standard build (no HDF5)
cargo build --release

# Run tests (no HDF5)
cargo test

# Build with cloud storage but no HDF5
cargo build --features cloud
```

## Format Support Without HDF5

Even without HDF5 support, AI Model Vault supports 20+ other formats:
- PyTorch (.pt, .pth, .bin)
- TensorFlow (.pb, .keras)
- ONNX (.onnx)
- Safetensors (.safetensors)
- GGUF (.gguf)
- And 15+ more!

HDF5 is only needed if you specifically work with .h5/.hdf5 files.

## Troubleshooting

### Error: "Unable to locate HDF5 root directory"

This means the HDF5 library is not installed. Either:
1. Install HDF5 (see Installation above)
2. Build without HDF5: `cargo build` (don't use `--features hdf5-support`)

### Error: "hdf5-sys build failed"

Ensure pkg-config can find HDF5:

**Linux/macOS:**
```bash
pkg-config --cflags --libs hdf5
```

**Windows:**
```powershell
# Set environment variable
$env:HDF5_DIR = "C:\Program Files\HDF5"
```

## Production Deployment

For production deployments, we recommend:

1. **Docker**: Include HDF5 in your container image
   ```dockerfile
   FROM rust:latest
   RUN apt-get update && apt-get install -y libhdf5-dev
   COPY . /app
   WORKDIR /app
   RUN cargo build --release --features hdf5-support
   ```

2. **System Package**: Install HDF5 as a system dependency
   ```bash
   # Add to deployment script
   apt-get install -y libhdf5-dev  # Debian/Ubuntu
   yum install hdf5-devel          # RHEL/CentOS
   ```

3. **Skip HDF5**: If you don't need .h5 files, skip the feature entirely

## Feature Matrix

| Format      | Default Build | With hdf5-support |
| ----------- | ------------- | ----------------- |
| PyTorch     | ✅             | ✅                 |
| TensorFlow  | ✅             | ✅                 |
| ONNX        | ✅             | ✅                 |
| Safetensors | ✅             | ✅                 |
| GGUF        | ✅             | ✅                 |
| **HDF5**    | ❌             | ✅                 |
| NumPy       | ✅             | ✅                 |
| All others  | ✅             | ✅                 |

Only HDF5 format requires the optional feature.
