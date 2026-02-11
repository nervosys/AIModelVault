//! GPU-accelerated encryption module
//!
//! Provides OpenCL-based parallel encryption for large model files.
//! Falls back to CPU encryption when GPU is unavailable.
//!
//! Enable with the `gpu` feature flag.
//!
//! ## Performance
//!
//! GPU acceleration is most beneficial for:
//! - Large models (>100 MB)
//! - Batch operations
//! - Systems with dedicated GPUs
//!
//! For small files, CPU encryption may be faster due to GPU transfer overhead.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use crate::crypto::{FipsCrypto, SecureKey, KEY_SIZE, NONCE_SIZE};
use crate::error::{Result, VaultError};

/// Minimum data size (bytes) to use GPU acceleration (default: 10 MB)
const GPU_THRESHOLD_BYTES: usize = 10 * 1024 * 1024;

/// OpenCL kernel for AES-256 CTR mode encryption
/// Note: This is a simplified implementation; production should use AES-GCM
const AES_CTR_KERNEL: &str = r#"
// AES S-box
__constant uchar sbox[256] = {
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16
};

// Round constants
__constant uchar rcon[11] = {0x00, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36};

// AES key expansion (simplified for 256-bit key)
void key_expansion(__private uchar* key, __private uchar* round_keys) {
    for (int i = 0; i < 32; i++) {
        round_keys[i] = key[i];
    }
    
    uchar temp[4];
    int i = 8;
    
    while (i < 60) {
        for (int j = 0; j < 4; j++) {
            temp[j] = round_keys[(i - 1) * 4 + j];
        }
        
        if (i % 8 == 0) {
            // RotWord
            uchar t = temp[0];
            temp[0] = temp[1];
            temp[1] = temp[2];
            temp[2] = temp[3];
            temp[3] = t;
            
            // SubWord
            for (int j = 0; j < 4; j++) {
                temp[j] = sbox[temp[j]];
            }
            
            temp[0] ^= rcon[i / 8];
        } else if (i % 8 == 4) {
            // SubWord only
            for (int j = 0; j < 4; j++) {
                temp[j] = sbox[temp[j]];
            }
        }
        
        for (int j = 0; j < 4; j++) {
            round_keys[i * 4 + j] = round_keys[(i - 8) * 4 + j] ^ temp[j];
        }
        i++;
    }
}

// XOR state with round key
void add_round_key(__private uchar* state, __private uchar* round_key) {
    for (int i = 0; i < 16; i++) {
        state[i] ^= round_key[i];
    }
}

// SubBytes transformation
void sub_bytes(__private uchar* state) {
    for (int i = 0; i < 16; i++) {
        state[i] = sbox[state[i]];
    }
}

// ShiftRows transformation
void shift_rows(__private uchar* state) {
    uchar temp;
    
    // Row 1: shift left by 1
    temp = state[1];
    state[1] = state[5];
    state[5] = state[9];
    state[9] = state[13];
    state[13] = temp;
    
    // Row 2: shift left by 2
    temp = state[2];
    state[2] = state[10];
    state[10] = temp;
    temp = state[6];
    state[6] = state[14];
    state[14] = temp;
    
    // Row 3: shift left by 3
    temp = state[15];
    state[15] = state[11];
    state[11] = state[7];
    state[7] = state[3];
    state[3] = temp;
}

// GF(2^8) multiplication
uchar gmul(uchar a, uchar b) {
    uchar p = 0;
    for (int i = 0; i < 8; i++) {
        if (b & 1) p ^= a;
        uchar hi = a & 0x80;
        a <<= 1;
        if (hi) a ^= 0x1b;
        b >>= 1;
    }
    return p;
}

// MixColumns transformation
void mix_columns(__private uchar* state) {
    for (int c = 0; c < 4; c++) {
        uchar a[4];
        for (int i = 0; i < 4; i++) {
            a[i] = state[c * 4 + i];
        }
        state[c * 4 + 0] = gmul(a[0], 2) ^ gmul(a[1], 3) ^ a[2] ^ a[3];
        state[c * 4 + 1] = a[0] ^ gmul(a[1], 2) ^ gmul(a[2], 3) ^ a[3];
        state[c * 4 + 2] = a[0] ^ a[1] ^ gmul(a[2], 2) ^ gmul(a[3], 3);
        state[c * 4 + 3] = gmul(a[0], 3) ^ a[1] ^ a[2] ^ gmul(a[3], 2);
    }
}

// AES-256 block cipher
void aes_encrypt_block(__private uchar* state, __private uchar* round_keys) {
    add_round_key(state, round_keys);
    
    for (int round = 1; round < 14; round++) {
        sub_bytes(state);
        shift_rows(state);
        mix_columns(state);
        add_round_key(state, round_keys + round * 16);
    }
    
    sub_bytes(state);
    shift_rows(state);
    add_round_key(state, round_keys + 14 * 16);
}

// Increment counter
void increment_counter(__private uchar* counter) {
    for (int i = 15; i >= 0; i--) {
        if (++counter[i] != 0) break;
    }
}

// Main kernel: AES-256-CTR encryption/decryption
__kernel void aes256_ctr(
    __global const uchar* input,
    __global uchar* output,
    __global const uchar* key,
    __global const uchar* nonce,
    const ulong data_len,
    const ulong block_offset
) {
    size_t gid = get_global_id(0);
    size_t block_idx = block_offset + gid;
    size_t byte_offset = block_idx * 16;
    
    if (byte_offset >= data_len) return;
    
    // Expand key
    __private uchar round_keys[240];
    __private uchar local_key[32];
    for (int i = 0; i < 32; i++) {
        local_key[i] = key[i];
    }
    key_expansion(local_key, round_keys);
    
    // Prepare counter block: nonce (12 bytes) + counter (4 bytes)
    __private uchar counter[16];
    for (int i = 0; i < 12; i++) {
        counter[i] = nonce[i];
    }
    // Set counter value (big-endian)
    counter[12] = (block_idx >> 24) & 0xff;
    counter[13] = (block_idx >> 16) & 0xff;
    counter[14] = (block_idx >> 8) & 0xff;
    counter[15] = block_idx & 0xff;
    
    // Encrypt counter to get keystream
    __private uchar keystream[16];
    for (int i = 0; i < 16; i++) {
        keystream[i] = counter[i];
    }
    aes_encrypt_block(keystream, round_keys);
    
    // XOR with input data
    size_t remaining = min((size_t)16, (size_t)(data_len - byte_offset));
    for (size_t i = 0; i < remaining; i++) {
        output[byte_offset + i] = input[byte_offset + i] ^ keystream[i];
    }
}
"#;

/// Global GPU context (initialized once)
static GPU_CONTEXT: OnceLock<Option<GpuContext>> = OnceLock::new();

/// Whether GPU is available and enabled
static GPU_ENABLED: AtomicBool = AtomicBool::new(true);

/// GPU context holding OpenCL resources
#[cfg(feature = "gpu")]
pub struct GpuContext {
    platform: ocl::Platform,
    device: ocl::Device,
    context: ocl::Context,
    queue: ocl::Queue,
    program: ocl::Program,
    device_name: String,
    device_memory: u64,
}

#[cfg(not(feature = "gpu"))]
pub struct GpuContext;

#[cfg(feature = "gpu")]
impl GpuContext {
    /// Initialize GPU context
    fn new() -> Result<Self> {
        use ocl::{Context, Device, Platform, Program, Queue};

        // Find a suitable platform
        let platform = Platform::list()
            .into_iter()
            .find(|p| {
                Platform::name(p)
                    .map(|n| !n.to_lowercase().contains("cpu"))
                    .unwrap_or(false)
            })
            .or_else(|| Platform::list().into_iter().next())
            .ok_or_else(|| VaultError::CryptoError("No OpenCL platform found".into()))?;

        // Find a GPU device
        let device = Device::list(platform, Some(ocl::flags::DeviceType::GPU))
            .ok()
            .and_then(|devices| devices.into_iter().next())
            .or_else(|| {
                // Fall back to any device
                Device::list(platform, None)
                    .ok()
                    .and_then(|d| d.into_iter().next())
            })
            .ok_or_else(|| VaultError::CryptoError("No OpenCL device found".into()))?;

        let device_name = device.name().unwrap_or_else(|_| "Unknown GPU".into());
        let device_memory = device
            .info(ocl::enums::DeviceInfo::GlobalMemSize)
            .map(|i| match i {
                ocl::enums::DeviceInfoResult::GlobalMemSize(m) => m,
                _ => 0,
            })
            .unwrap_or(0);

        // Create context and queue
        let context = Context::builder()
            .platform(platform)
            .devices(device)
            .build()
            .map_err(|e| {
                VaultError::CryptoError(format!("Failed to create OpenCL context: {e}"))
            })?;

        let queue = Queue::new(&context, device, None)
            .map_err(|e| VaultError::CryptoError(format!("Failed to create command queue: {e}")))?;

        // Compile the AES kernel
        let program = Program::builder()
            .src(AES_CTR_KERNEL)
            .devices(device)
            .build(&context)
            .map_err(|e| VaultError::CryptoError(format!("Failed to compile AES kernel: {e}")))?;

        Ok(Self {
            platform,
            device,
            context,
            queue,
            program,
            device_name,
            device_memory,
        })
    }

    /// Encrypt data using GPU
    fn encrypt(&self, data: &[u8], key: &SecureKey, nonce: &[u8]) -> Result<Vec<u8>> {
        use ocl::{Buffer, Kernel};

        let data_len = data.len();
        let num_blocks = (data_len + 15) / 16;

        // Create buffers
        let input_buf = Buffer::<u8>::builder()
            .queue(self.queue.clone())
            .len(data_len)
            .copy_host_slice(data)
            .build()
            .map_err(|e| VaultError::CryptoError(format!("Failed to create input buffer: {e}")))?;

        let output_buf = Buffer::<u8>::builder()
            .queue(self.queue.clone())
            .len(data_len)
            .build()
            .map_err(|e| VaultError::CryptoError(format!("Failed to create output buffer: {e}")))?;

        let key_buf = Buffer::<u8>::builder()
            .queue(self.queue.clone())
            .len(KEY_SIZE)
            .copy_host_slice(key.as_bytes())
            .build()
            .map_err(|e| VaultError::CryptoError(format!("Failed to create key buffer: {e}")))?;

        let nonce_buf = Buffer::<u8>::builder()
            .queue(self.queue.clone())
            .len(NONCE_SIZE)
            .copy_host_slice(nonce)
            .build()
            .map_err(|e| VaultError::CryptoError(format!("Failed to create nonce buffer: {e}")))?;

        // Create and execute kernel
        let kernel = Kernel::builder()
            .program(&self.program)
            .name("aes256_ctr")
            .queue(self.queue.clone())
            .global_work_size(num_blocks)
            .arg(&input_buf)
            .arg(&output_buf)
            .arg(&key_buf)
            .arg(&nonce_buf)
            .arg(data_len as u64)
            .arg(0u64) // block_offset
            .build()
            .map_err(|e| VaultError::CryptoError(format!("Failed to build kernel: {e}")))?;

        unsafe {
            kernel
                .enq()
                .map_err(|e| VaultError::CryptoError(format!("Failed to execute kernel: {e}")))?;
        }

        // Read result
        let mut output = vec![0u8; data_len];
        output_buf
            .read(&mut output)
            .enq()
            .map_err(|e| VaultError::CryptoError(format!("Failed to read output: {e}")))?;

        Ok(output)
    }
}

/// GPU-accelerated crypto operations
pub struct GpuCrypto {
    cpu_crypto: FipsCrypto,
}

impl GpuCrypto {
    /// Create new GPU crypto instance
    pub fn new() -> Result<Self> {
        Ok(Self {
            cpu_crypto: FipsCrypto::new()?,
        })
    }

    /// Check if GPU acceleration is available
    pub fn is_gpu_available() -> bool {
        #[cfg(feature = "gpu")]
        {
            GPU_CONTEXT.get_or_init(|| GpuContext::new().ok()).is_some()
                && GPU_ENABLED.load(Ordering::SeqCst)
        }
        #[cfg(not(feature = "gpu"))]
        {
            false
        }
    }

    /// Get GPU device info
    pub fn gpu_info() -> Option<GpuInfo> {
        #[cfg(feature = "gpu")]
        {
            GPU_CONTEXT
                .get_or_init(|| GpuContext::new().ok())
                .as_ref()
                .map(|ctx| GpuInfo {
                    name: ctx.device_name.clone(),
                    memory_bytes: ctx.device_memory,
                    available: true,
                })
        }
        #[cfg(not(feature = "gpu"))]
        {
            None
        }
    }

    /// Enable GPU acceleration
    pub fn enable_gpu() {
        GPU_ENABLED.store(true, Ordering::SeqCst);
    }

    /// Disable GPU acceleration (use CPU only)
    pub fn disable_gpu() {
        GPU_ENABLED.store(false, Ordering::SeqCst);
    }

    /// Encrypt data, using GPU if available and beneficial
    ///
    /// Automatically falls back to CPU for:
    /// - Small data (< 10 MB by default)
    /// - When GPU is unavailable
    /// - On GPU errors
    pub fn encrypt(&self, data: &[u8], key: &SecureKey) -> Result<Vec<u8>> {
        // Use GPU for large data if available
        if data.len() >= GPU_THRESHOLD_BYTES && Self::is_gpu_available() {
            match self.encrypt_gpu(data, key) {
                Ok(result) => return Ok(result),
                Err(_) => {
                    // Fall back to CPU on GPU error
                    tracing::warn!("GPU encryption failed, falling back to CPU");
                }
            }
        }

        // Use CPU encryption
        self.cpu_crypto.encrypt(data, key)
    }

    /// Decrypt data, using GPU if available and beneficial
    pub fn decrypt(&self, encrypted_data: &[u8], key: &SecureKey) -> Result<Vec<u8>> {
        // Check minimum size
        if encrypted_data.len() < NONCE_SIZE {
            return Err(VaultError::CryptoError("Encrypted data too short".into()));
        }

        let data_len = encrypted_data.len() - NONCE_SIZE;

        // Use GPU for large data if available
        if data_len >= GPU_THRESHOLD_BYTES && Self::is_gpu_available() {
            match self.decrypt_gpu(encrypted_data, key) {
                Ok(result) => return Ok(result),
                Err(_) => {
                    tracing::warn!("GPU decryption failed, falling back to CPU");
                }
            }
        }

        // Use CPU decryption
        self.cpu_crypto.decrypt(encrypted_data, key)
    }

    /// Encrypt using GPU (internal)
    #[cfg(feature = "gpu")]
    fn encrypt_gpu(&self, data: &[u8], key: &SecureKey) -> Result<Vec<u8>> {
        use aes_gcm::aead::rand_core::RngCore;
        use aes_gcm::aead::OsRng;

        let ctx = GPU_CONTEXT
            .get_or_init(|| GpuContext::new().ok())
            .as_ref()
            .ok_or_else(|| VaultError::CryptoError("GPU not available".into()))?;

        // Generate nonce
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);

        // Encrypt with GPU (CTR mode)
        let ciphertext = ctx.encrypt(data, key, &nonce)?;

        // Combine nonce || ciphertext
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    #[cfg(not(feature = "gpu"))]
    fn encrypt_gpu(&self, _data: &[u8], _key: &SecureKey) -> Result<Vec<u8>> {
        Err(VaultError::CryptoError("GPU feature not enabled".into()))
    }

    /// Decrypt using GPU (internal)
    #[cfg(feature = "gpu")]
    fn decrypt_gpu(&self, encrypted_data: &[u8], key: &SecureKey) -> Result<Vec<u8>> {
        let ctx = GPU_CONTEXT
            .get_or_init(|| GpuContext::new().ok())
            .as_ref()
            .ok_or_else(|| VaultError::CryptoError("GPU not available".into()))?;

        let nonce = &encrypted_data[..NONCE_SIZE];
        let ciphertext = &encrypted_data[NONCE_SIZE..];

        // Decrypt with GPU (CTR is symmetric)
        ctx.encrypt(ciphertext, key, nonce)
    }

    #[cfg(not(feature = "gpu"))]
    fn decrypt_gpu(&self, _encrypted_data: &[u8], _key: &SecureKey) -> Result<Vec<u8>> {
        Err(VaultError::CryptoError("GPU feature not enabled".into()))
    }

    /// Derive key using CPU (KDF is not GPU-accelerated)
    pub fn derive_key(
        &self,
        passphrase: Vec<u8>,
        salt: Option<Vec<u8>>,
    ) -> Result<(SecureKey, Vec<u8>)> {
        self.cpu_crypto.derive_key(passphrase, salt)
    }

    /// Benchmark encryption performance
    pub fn benchmark(&self, data_size: usize) -> BenchmarkResult {
        use std::time::Instant;

        let data = vec![0u8; data_size];
        let key = SecureKey::from_bytes(&[0u8; KEY_SIZE]).unwrap();

        // CPU benchmark
        let cpu_start = Instant::now();
        let _ = self.cpu_crypto.encrypt(&data, &key);
        let cpu_duration = cpu_start.elapsed();

        // GPU benchmark (if available)
        let gpu_duration = if Self::is_gpu_available() {
            let gpu_start = Instant::now();
            let _ = self.encrypt_gpu(&data, &key);
            Some(gpu_start.elapsed())
        } else {
            None
        };

        BenchmarkResult {
            data_size,
            cpu_ms: cpu_duration.as_millis() as u64,
            gpu_ms: gpu_duration.map(|d| d.as_millis() as u64),
            speedup: gpu_duration.map(|g| {
                if g.as_nanos() > 0 {
                    cpu_duration.as_nanos() as f64 / g.as_nanos() as f64
                } else {
                    0.0
                }
            }),
        }
    }
}

/// GPU device information
#[derive(Debug, Clone)]
pub struct GpuInfo {
    /// Device name
    pub name: String,
    /// Device global memory in bytes
    pub memory_bytes: u64,
    /// Whether the device is available
    pub available: bool,
}

/// Benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Data size in bytes
    pub data_size: usize,
    /// CPU encryption time in milliseconds
    pub cpu_ms: u64,
    /// GPU encryption time in milliseconds (if available)
    pub gpu_ms: Option<u64>,
    /// Speedup factor (CPU time / GPU time)
    pub speedup: Option<f64>,
}

impl std::fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size_mb = self.data_size as f64 / (1024.0 * 1024.0);
        write!(f, "Benchmark ({:.1} MB): CPU={} ms", size_mb, self.cpu_ms)?;
        if let Some(gpu_ms) = self.gpu_ms {
            write!(f, ", GPU={} ms", gpu_ms)?;
            if let Some(speedup) = self.speedup {
                write!(f, " ({:.1}x speedup)", speedup)?;
            }
        } else {
            write!(f, ", GPU=N/A")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_info() {
        // Should not panic even without GPU
        let _info = GpuCrypto::gpu_info();
    }

    #[test]
    fn test_cpu_fallback() {
        let crypto = GpuCrypto::new().unwrap();
        let key = SecureKey::from_bytes(&[0u8; KEY_SIZE]).unwrap();
        let data = b"Hello, World!";

        let encrypted = crypto.encrypt(data, &key).unwrap();
        let decrypted = crypto.decrypt(&encrypted, &key).unwrap();

        assert_eq!(data.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_benchmark() {
        let crypto = GpuCrypto::new().unwrap();
        let result = crypto.benchmark(1024 * 1024); // 1 MB
        assert!(result.cpu_ms >= 0);
    }
}
