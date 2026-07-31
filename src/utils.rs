//! AI Model Utilities - Common operations for AI models
//!
//! Provides utilities for model archiving, compression, retrieval optimization,
//! quantization metadata, pruning info, and model analysis.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{Result, VaultError};
use crate::formats::{ModelFormat, ModelMetadata};

/// Reject an archive member name that would escape the extraction directory.
///
/// Archive member names are attacker-controlled for any archive the user did
/// not produce. `extract_zip` returned them verbatim and `handle_extract` fed
/// them to `Path::join`, which discards its base when given an absolute path
/// and walks upward on `..` — so a member named `../../evil` wrote outside the
/// `--output` directory ("zip slip", CWE-22). A longer prefix or a drive
/// letter reaches anywhere the invoking user can write.
///
/// Both `create_tar` and `create_zip` write bare model names, so requiring a
/// single ordinary component rejects nothing this crate produces.
fn safe_archive_name(raw: &str) -> Result<String> {
    use std::path::Component;

    let mut components = Path::new(raw).components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(name)), None) => Ok(name.to_string_lossy().into_owned()),
        _ => Err(VaultError::InvalidInput(format!(
            "Refusing to extract archive member {raw:?}: member names must be a \
             single file name, with no directory separators, parent references, \
             or drive/root prefix"
        ))),
    }
}

/// Largest single archive member `extract_tar` / `extract_zip` will decompress.
///
/// Both extractors buffer members in memory, so without a ceiling the process
/// resident set is whatever the archive says it should be. A compressed member
/// costs the attacker almost nothing to declare: a zip of a few hundred KiB of
/// zeroes expands to gigabytes ("zip bomb"), and a tar header can claim any
/// `u64` length at all. Refusing oversized members turns an out-of-memory kill
/// — which takes down whatever else shares the process — into an error the
/// caller can report.
///
/// 8 GiB clears the largest model files this crate handles by a wide margin.
pub const MAX_ARCHIVE_MEMBER_BYTES: u64 = 8 * 1024 * 1024 * 1024;

/// Largest total uncompressed payload a single archive may expand to.
///
/// The per-member cap alone does not bound the whole: a million members each
/// just under the limit still exhausts memory. This bounds the sum.
pub const MAX_ARCHIVE_TOTAL_BYTES: u64 = 16 * 1024 * 1024 * 1024;

/// Reject a member whose declared size already exceeds the per-member cap, or
/// whose size would push the archive past the total cap.
fn check_archive_budget(name: &str, declared: u64, running_total: u64) -> Result<()> {
    if declared > MAX_ARCHIVE_MEMBER_BYTES {
        return Err(VaultError::InvalidInput(format!(
            "Refusing to extract archive member {name:?}: declared size {declared} bytes \
             exceeds the {MAX_ARCHIVE_MEMBER_BYTES}-byte per-member limit"
        )));
    }
    if running_total.saturating_add(declared) > MAX_ARCHIVE_TOTAL_BYTES {
        return Err(VaultError::InvalidInput(format!(
            "Refusing to extract archive member {name:?}: archive expands past the \
             {MAX_ARCHIVE_TOTAL_BYTES}-byte total limit"
        )));
    }
    Ok(())
}

/// Read at most `MAX_ARCHIVE_MEMBER_BYTES` from `reader`, erroring if there is
/// more.
///
/// The declared size is only a claim. A tar header can understate the payload
/// and a zip local header can lie outright, so the read itself is bounded too:
/// take one byte more than the cap allows and treat a full buffer as proof the
/// member overran.
///
/// `limit` is a parameter rather than the constant so tests can exercise the
/// overrun path without materialising 8 GiB; callers always pass
/// [`MAX_ARCHIVE_MEMBER_BYTES`].
fn read_member_bounded<R: Read>(name: &str, reader: &mut R, limit: u64) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let read = reader.take(limit + 1).read_to_end(&mut data)?;

    if read as u64 > limit {
        return Err(VaultError::InvalidInput(format!(
            "Refusing to extract archive member {name:?}: actual contents exceed the \
             {limit}-byte per-member limit (the declared size understated it)"
        )));
    }

    Ok(data)
}

/// Model archival utilities
pub struct ModelArchive;

impl ModelArchive {
    /// Create a TAR archive of multiple models
    pub fn create_tar(models: Vec<(String, Vec<u8>)>, output_path: &Path) -> Result<usize> {
        let file = std::fs::File::create(output_path)?;
        let mut ar = tar::Builder::new(file);

        let mut total_size = 0;

        for (name, data) in models {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();

            ar.append_data(&mut header, &name, &data[..])?;
            total_size += data.len();
        }

        ar.finish()?;
        Ok(total_size)
    }

    /// Extract models from TAR archive
    pub fn extract_tar(archive_path: &Path) -> Result<Vec<(String, Vec<u8>)>> {
        let file = std::fs::File::open(archive_path)?;
        let mut ar = tar::Archive::new(file);

        let mut models = Vec::new();
        let mut total: u64 = 0;

        for entry in ar.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            // tar entries were previously reduced with `file_name()`, which is
            // safe but silent: `../../etc/passwd` became `passwd` and quietly
            // overwrote a legitimate member of the same name. Reject instead,
            // matching the zip path.
            let name = safe_archive_name(&path.to_string_lossy())?;

            check_archive_budget(&name, entry.size(), total)?;
            let data = read_member_bounded(&name, &mut entry, MAX_ARCHIVE_MEMBER_BYTES)?;
            total = total.saturating_add(data.len() as u64);

            models.push((name, data));
        }

        Ok(models)
    }

    /// Create a ZIP archive of multiple models
    pub fn create_zip(models: Vec<(String, Vec<u8>)>, output_path: &Path) -> Result<usize> {
        let file = std::fs::File::create(output_path)?;
        let mut zip = zip::ZipWriter::new(file);

        let mut total_size = 0;

        for (name, data) in models {
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o644);

            zip.start_file(&name, options)?;
            zip.write_all(&data)?;
            total_size += data.len();
        }

        zip.finish()?;
        Ok(total_size)
    }

    /// Extract models from ZIP archive
    pub fn extract_zip(archive_path: &Path) -> Result<Vec<(String, Vec<u8>)>> {
        let file = std::fs::File::open(archive_path)?;
        let mut zip = zip::ZipArchive::new(file)?;

        let mut models = Vec::new();
        let mut total: u64 = 0;

        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            // Validated before any data is read, and the whole archive is
            // collected before the caller writes anything — so a hostile
            // member aborts the extraction rather than leaving files behind.
            let name = safe_archive_name(file.name())?;

            check_archive_budget(&name, file.size(), total)?;
            let data = read_member_bounded(&name, &mut file, MAX_ARCHIVE_MEMBER_BYTES)?;
            total = total.saturating_add(data.len() as u64);

            models.push((name, data));
        }

        Ok(models)
    }
}

/// Model compression utilities (metadata and analysis)
pub struct CompressionAnalyzer;

impl CompressionAnalyzer {
    /// Calculate compression ratio
    #[must_use]
    pub fn compression_ratio(original_size: u64, compressed_size: u64) -> f64 {
        if compressed_size == 0 {
            return 0.0;
        }
        original_size as f64 / compressed_size as f64
    }

    /// Estimate compression ratio for model format
    #[must_use]
    pub fn estimate_ratio(format: &ModelFormat) -> f64 {
        match format {
            ModelFormat::Safetensors => 1.05, // Low compression (already optimized)
            ModelFormat::GGUF => 1.0,         // Pre-quantized/compressed
            ModelFormat::PyTorch => 1.3,      // Moderate compression
            ModelFormat::ONNX => 1.2,
            ModelFormat::TensorRT => 1.0, // Compiled format
            ModelFormat::TFLite => 1.1,
            ModelFormat::HDF5 => 1.4,   // Good compression potential
            ModelFormat::Pickle => 1.5, // Python pickle compresses well
            _ => 1.2,                   // Default estimate
        }
    }

    /// Analyze compression effectiveness
    pub fn analyze_compression(
        original_size: u64,
        compressed_size: u64,
        format: &ModelFormat,
    ) -> CompressionReport {
        let actual_ratio = Self::compression_ratio(original_size, compressed_size);
        let estimated_ratio = Self::estimate_ratio(format);
        let space_saved = original_size.saturating_sub(compressed_size);
        let space_saved_percent = (space_saved as f64 / original_size as f64) * 100.0;

        CompressionReport {
            original_size,
            compressed_size,
            space_saved,
            compression_ratio: actual_ratio,
            estimated_ratio,
            space_saved_percent,
            efficiency: actual_ratio / estimated_ratio,
        }
    }
}

/// Compression analysis report
#[derive(Debug, Clone)]
pub struct CompressionReport {
    pub original_size: u64,
    pub compressed_size: u64,
    pub space_saved: u64,
    pub compression_ratio: f64,
    pub estimated_ratio: f64,
    pub space_saved_percent: f64,
    pub efficiency: f64,
}

/// Model retrieval optimization (caching and prefetching)
pub struct RetrievalOptimizer {
    cache: HashMap<String, CachedModel>,
    max_cache_size: usize,
    current_cache_size: usize,
}

#[derive(Debug, Clone)]
struct CachedModel {
    data: Vec<u8>,
    access_count: usize,
    last_access: std::time::SystemTime,
}

impl RetrievalOptimizer {
    /// Create new retrieval optimizer with cache size limit (bytes)
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_cache_size,
            current_cache_size: 0,
        }
    }

    /// Cache a model for faster retrieval
    pub fn cache_model(&mut self, key: String, data: Vec<u8>) -> Result<()> {
        let data_size = data.len();

        // Evict old entries if needed
        while self.current_cache_size + data_size > self.max_cache_size && !self.cache.is_empty() {
            self.evict_lru();
        }

        // Don't cache if model is larger than max cache size
        if data_size > self.max_cache_size {
            return Ok(());
        }

        self.current_cache_size += data_size;
        self.cache.insert(
            key,
            CachedModel {
                data,
                access_count: 1,
                last_access: std::time::SystemTime::now(),
            },
        );

        Ok(())
    }

    /// Retrieve model from cache
    pub fn get_cached(&mut self, key: &str) -> Option<Vec<u8>> {
        if let Some(cached) = self.cache.get_mut(key) {
            cached.access_count += 1;
            cached.last_access = std::time::SystemTime::now();
            Some(cached.data.clone())
        } else {
            None
        }
    }

    /// Evict least recently used model
    fn evict_lru(&mut self) {
        if let Some(key) = self
            .cache
            .iter()
            .min_by_key(|(_, v)| v.last_access)
            .map(|(k, _)| k.clone())
        {
            if let Some(cached) = self.cache.remove(&key) {
                self.current_cache_size -= cached.data.len();
            }
        }
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.current_cache_size = 0;
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.cache.len(),
            total_size: self.current_cache_size,
            max_size: self.max_cache_size,
            utilization: (self.current_cache_size as f64 / self.max_cache_size as f64) * 100.0,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size: usize,
    pub max_size: usize,
    pub utilization: f64,
}

/// Quantization metadata utilities
pub struct QuantizationInfo;

impl QuantizationInfo {
    /// Common quantization schemes
    pub fn schemes() -> Vec<&'static str> {
        vec![
            "FP32",   // Full precision
            "FP16",   // Half precision
            "BF16",   // Brain float 16
            "INT8",   // 8-bit integer
            "INT4",   // 4-bit integer
            "Q8_0",   // GGUF 8-bit quantization
            "Q4_0",   // GGUF 4-bit quantization
            "Q4_K_M", // GGUF 4-bit K-quant medium
            "Q5_K_M", // GGUF 5-bit K-quant medium
            "Q6_K",   // GGUF 6-bit K-quant
        ]
    }

    /// Estimate model size after quantization
    #[must_use]
    pub fn estimate_size(original_size: u64, from_bits: u8, to_bits: u8) -> u64 {
        if from_bits == 0 || to_bits == 0 {
            return original_size;
        }
        (original_size as f64 * (to_bits as f64 / from_bits as f64)) as u64
    }

    /// Calculate memory savings from quantization
    #[must_use]
    pub fn memory_savings(original_size: u64, quantized_size: u64) -> QuantizationSavings {
        let saved_bytes = original_size.saturating_sub(quantized_size);
        let saved_percent = (saved_bytes as f64 / original_size as f64) * 100.0;

        QuantizationSavings {
            original_size,
            quantized_size,
            saved_bytes,
            saved_percent,
            size_ratio: original_size as f64 / quantized_size as f64,
        }
    }

    /// Validate quantization scheme
    pub fn is_valid_scheme(scheme: &str) -> bool {
        Self::schemes().contains(&scheme)
    }
}

/// Quantization savings information
#[derive(Debug, Clone)]
pub struct QuantizationSavings {
    pub original_size: u64,
    pub quantized_size: u64,
    pub saved_bytes: u64,
    pub saved_percent: f64,
    pub size_ratio: f64,
}

/// Model pruning metadata
pub struct PruningInfo {
    pub pruning_method: PruningMethod,
    pub sparsity_level: f64,
    pub original_params: u64,
    pub remaining_params: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PruningMethod {
    Magnitude,
    Structured,
    Unstructured,
    GradientBased,
    LayerWise,
    Custom(String),
}

impl PruningInfo {
    /// Create new pruning information
    pub fn new(method: PruningMethod, sparsity: f64, original: u64, remaining: u64) -> Self {
        Self {
            pruning_method: method,
            sparsity_level: sparsity.clamp(0.0, 1.0),
            original_params: original,
            remaining_params: remaining,
        }
    }

    /// Calculate actual sparsity from parameters
    #[must_use]
    pub fn calculate_sparsity(&self) -> f64 {
        if self.original_params == 0 {
            return 0.0;
        }
        1.0 - (self.remaining_params as f64 / self.original_params as f64)
    }

    /// Estimate size reduction from pruning
    #[must_use]
    pub fn size_reduction(&self) -> f64 {
        self.calculate_sparsity() * 100.0
    }
}

/// Model analysis utilities
pub struct ModelAnalyzer;

impl ModelAnalyzer {
    /// Analyze model file
    pub fn analyze(data: &[u8], metadata: &ModelMetadata) -> ModelAnalysis {
        let size_bytes = data.len() as u64;
        let size_mb = size_bytes as f64 / (1024.0 * 1024.0);
        let size_gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

        // Estimate parameters based on format and size
        let estimated_params = Self::estimate_parameters(size_bytes, &metadata.format);

        ModelAnalysis {
            format: metadata.format.clone(),
            size_bytes,
            size_mb,
            size_gb,
            estimated_parameters: estimated_params,
            framework: metadata.framework.clone(),
            task: metadata.task.clone(),
            architecture: metadata.architecture.clone(),
        }
    }

    /// Estimate parameter count from size
    fn estimate_parameters(size_bytes: u64, format: &ModelFormat) -> Option<u64> {
        // Rough estimation: 4 bytes per FP32 parameter
        let base_estimate = size_bytes / 4;

        match format {
            ModelFormat::GGUF => Some(base_estimate * 2), // Quantized, more params for same size
            ModelFormat::Safetensors | ModelFormat::PyTorch => Some(base_estimate),
            ModelFormat::ONNX => Some((base_estimate as f64 * 0.9) as u64), // Overhead
            ModelFormat::TFLite => Some(base_estimate * 4),                 // Compressed
            _ => Some(base_estimate),
        }
    }

    /// Get human-readable size
    #[must_use]
    pub fn format_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        if bytes >= TB {
            format!("{:.2} TB", bytes as f64 / TB as f64)
        } else if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    /// Get human-readable parameter count
    #[must_use]
    pub fn format_parameters(params: u64) -> String {
        const K: u64 = 1_000;
        const M: u64 = K * 1_000;
        const B: u64 = M * 1_000;

        if params >= B {
            format!("{:.2}B", params as f64 / B as f64)
        } else if params >= M {
            format!("{:.2}M", params as f64 / M as f64)
        } else if params >= K {
            format!("{:.2}K", params as f64 / K as f64)
        } else {
            format!("{}", params)
        }
    }
}

/// Model analysis result
#[derive(Debug, Clone)]
pub struct ModelAnalysis {
    pub format: ModelFormat,
    pub size_bytes: u64,
    pub size_mb: f64,
    pub size_gb: f64,
    pub estimated_parameters: Option<u64>,
    pub framework: Option<String>,
    pub task: Option<String>,
    pub architecture: Option<String>,
}

/// Model export utilities
pub struct ModelExporter;

impl ModelExporter {
    /// Export model with metadata
    pub fn export_with_metadata(
        model_data: Vec<u8>,
        metadata: &ModelMetadata,
        output_dir: &Path,
    ) -> Result<PathBuf> {
        std::fs::create_dir_all(output_dir)?;

        // Save model file
        let model_filename = format!("{}.{}", metadata.name, metadata.format.extension());
        let model_path = output_dir.join(&model_filename);
        std::fs::write(&model_path, model_data)?;

        // Save metadata as JSON
        let metadata_filename = format!("{}.meta.json", metadata.name);
        let metadata_path = output_dir.join(&metadata_filename);
        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        std::fs::write(metadata_path, metadata_json)?;

        Ok(model_path)
    }

    /// Export model to directory structure
    pub fn export_to_directory(
        models: Vec<(Vec<u8>, ModelMetadata)>,
        output_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        std::fs::create_dir_all(output_dir)?;

        let mut paths = Vec::new();

        for (data, metadata) in models {
            let path = Self::export_with_metadata(data, &metadata, output_dir)?;
            paths.push(path);
        }

        Ok(paths)
    }
}

/// Model deduplication utilities
pub struct ModelDeduplicator;

impl ModelDeduplicator {
    /// Calculate model hash for deduplication
    #[must_use]
    pub fn calculate_hash(data: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// Find duplicate models by hash
    pub fn find_duplicates(models: Vec<(String, Vec<u8>)>) -> HashMap<String, Vec<String>> {
        let mut hash_map: HashMap<String, Vec<String>> = HashMap::new();

        for (name, data) in models {
            let hash = Self::calculate_hash(&data);
            hash_map.entry(hash).or_default().push(name);
        }

        // Filter to only duplicates (more than one model with same hash)
        hash_map
            .into_iter()
            .filter(|(_, names)| names.len() > 1)
            .collect()
    }

    /// Calculate content similarity (simple byte comparison)
    #[must_use]
    pub fn similarity_score(data1: &[u8], data2: &[u8]) -> f64 {
        if data1.len() != data2.len() {
            return 0.0;
        }

        let matching_bytes = data1
            .iter()
            .zip(data2.iter())
            .filter(|(a, b)| a == b)
            .count();

        (matching_bytes as f64 / data1.len() as f64) * 100.0
    }
}

#[cfg(test)]
// Exact float comparison is intentional here: these assert on literal
// constants that round-trip bit-for-bit, not on computed results.
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_ratio() {
        let ratio = CompressionAnalyzer::compression_ratio(1000, 500);
        assert_eq!(ratio, 2.0);

        let ratio = CompressionAnalyzer::compression_ratio(1000, 250);
        assert_eq!(ratio, 4.0);
    }

    #[test]
    fn test_compression_analysis() {
        let report = CompressionAnalyzer::analyze_compression(1000, 750, &ModelFormat::PyTorch);

        assert_eq!(report.original_size, 1000);
        assert_eq!(report.compressed_size, 750);
        assert_eq!(report.space_saved, 250);
        assert_eq!(report.space_saved_percent, 25.0);
    }

    #[test]
    fn test_quantization_size_estimate() {
        let size = QuantizationInfo::estimate_size(1000, 32, 16);
        assert_eq!(size, 500);

        let size = QuantizationInfo::estimate_size(1000, 32, 8);
        assert_eq!(size, 250);
    }

    #[test]
    fn test_quantization_savings() {
        let savings = QuantizationInfo::memory_savings(1000, 250);
        assert_eq!(savings.saved_bytes, 750);
        assert_eq!(savings.saved_percent, 75.0);
        assert_eq!(savings.size_ratio, 4.0);
    }

    #[test]
    fn test_pruning_info() {
        let info = PruningInfo::new(PruningMethod::Magnitude, 0.5, 1000, 500);
        assert_eq!(info.calculate_sparsity(), 0.5);
        assert_eq!(info.size_reduction(), 50.0);
    }

    #[test]
    fn test_retrieval_optimizer() {
        let mut optimizer = RetrievalOptimizer::new(1000);

        // Cache a model
        optimizer
            .cache_model("model1".to_string(), vec![0; 100])
            .unwrap();
        assert_eq!(optimizer.current_cache_size, 100);

        // Retrieve cached model
        let cached = optimizer.get_cached("model1");
        assert!(cached.is_some());

        // Cache another model
        optimizer
            .cache_model("model2".to_string(), vec![0; 100])
            .unwrap();
        assert_eq!(optimizer.current_cache_size, 200);
    }

    #[test]
    fn test_model_analyzer_format_size() {
        assert_eq!(ModelAnalyzer::format_size(500), "500 B");
        assert_eq!(ModelAnalyzer::format_size(1024), "1.00 KB");
        assert_eq!(ModelAnalyzer::format_size(1024 * 1024), "1.00 MB");
        assert_eq!(ModelAnalyzer::format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_model_analyzer_format_parameters() {
        assert_eq!(ModelAnalyzer::format_parameters(500), "500");
        assert_eq!(ModelAnalyzer::format_parameters(1_500), "1.50K");
        assert_eq!(ModelAnalyzer::format_parameters(1_500_000), "1.50M");
        assert_eq!(ModelAnalyzer::format_parameters(7_000_000_000), "7.00B");
    }

    #[test]
    fn test_deduplicator_hash() {
        let data1 = b"test data";
        let hash1 = ModelDeduplicator::calculate_hash(data1);
        let hash2 = ModelDeduplicator::calculate_hash(data1);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 produces 64 hex chars
    }

    #[test]
    fn test_deduplicator_similarity() {
        let data1 = b"test data";
        let data2 = b"test data";
        let data3 = b"different";

        let score1 = ModelDeduplicator::similarity_score(data1, data2);
        assert_eq!(score1, 100.0);

        let score2 = ModelDeduplicator::similarity_score(data1, data3);
        assert_eq!(score2, 0.0); // Different lengths
    }

    #[test]
    fn test_create_and_extract_tar() {
        // Covers lines 18, plus extract_tar
        let temp_dir = tempfile::tempdir().unwrap();
        let tar_path = temp_dir.path().join("test.tar");

        let models = vec![
            ("model_a.bin".to_string(), vec![1, 2, 3, 4]),
            ("model_b.bin".to_string(), vec![5, 6, 7, 8, 9]),
        ];

        let total = ModelArchive::create_tar(models, &tar_path).unwrap();
        assert_eq!(total, 9); // 4 + 5

        let extracted = ModelArchive::extract_tar(&tar_path).unwrap();
        assert_eq!(extracted.len(), 2);
        assert_eq!(extracted[0].1, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_create_and_extract_zip() {
        // Covers lines 64, 85
        let temp_dir = tempfile::tempdir().unwrap();
        let zip_path = temp_dir.path().join("test.zip");

        let models = vec![
            ("model_x.bin".to_string(), vec![10, 20, 30]),
            ("model_y.bin".to_string(), vec![40, 50]),
        ];

        let total = ModelArchive::create_zip(models, &zip_path).unwrap();
        assert_eq!(total, 5);

        let extracted = ModelArchive::extract_zip(&zip_path).unwrap();
        assert_eq!(extracted.len(), 2);
    }

    #[test]
    fn test_model_analyzer_analyze() {
        // Covers line 381
        let data = vec![0u8; 1024 * 1024]; // 1 MB
        let meta = ModelMetadata::new("test_model".to_string(), ModelFormat::PyTorch);
        let analysis = ModelAnalyzer::analyze(&data, &meta);

        assert_eq!(analysis.size_bytes, 1024 * 1024);
        assert!((analysis.size_mb - 1.0).abs() < 0.01);
        assert!(analysis.estimated_parameters.unwrap() > 0);
    }

    #[test]
    fn test_compression_ratio_zero() {
        let ratio = CompressionAnalyzer::compression_ratio(1000, 0);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_estimate_ratio_all_formats() {
        assert!(CompressionAnalyzer::estimate_ratio(&ModelFormat::Safetensors) > 1.0);
        assert_eq!(CompressionAnalyzer::estimate_ratio(&ModelFormat::GGUF), 1.0);
        assert!(CompressionAnalyzer::estimate_ratio(&ModelFormat::PyTorch) > 1.0);
        assert!(CompressionAnalyzer::estimate_ratio(&ModelFormat::ONNX) > 1.0);
        assert_eq!(
            CompressionAnalyzer::estimate_ratio(&ModelFormat::TensorRT),
            1.0
        );
        assert!(CompressionAnalyzer::estimate_ratio(&ModelFormat::TFLite) >= 1.0);
        assert!(CompressionAnalyzer::estimate_ratio(&ModelFormat::HDF5) > 1.0);
        assert!(CompressionAnalyzer::estimate_ratio(&ModelFormat::Pickle) > 1.0);
        // Default case
        assert!(CompressionAnalyzer::estimate_ratio(&ModelFormat::CoreML) > 0.0);
    }

    #[test]
    fn test_retrieval_optimizer_eviction() {
        let mut opt = RetrievalOptimizer::new(100);
        opt.cache_model("m1".to_string(), vec![0; 60]).unwrap();
        opt.cache_model("m2".to_string(), vec![0; 60]).unwrap();
        // m1 should have been evicted
        assert!(opt.get_cached("m1").is_none());
        assert!(opt.get_cached("m2").is_some());
    }

    #[test]
    fn test_retrieval_optimizer_oversized() {
        let mut opt = RetrievalOptimizer::new(10);
        opt.cache_model("big".to_string(), vec![0; 100]).unwrap();
        assert!(opt.get_cached("big").is_none());
    }

    #[test]
    fn test_retrieval_optimizer_clear() {
        let mut opt = RetrievalOptimizer::new(1000);
        opt.cache_model("m1".to_string(), vec![1]).unwrap();
        opt.clear_cache();
        assert!(opt.get_cached("m1").is_none());
        let stats = opt.cache_stats();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.total_size, 0);
    }

    #[test]
    fn test_retrieval_optimizer_stats() {
        let mut opt = RetrievalOptimizer::new(1000);
        opt.cache_model("m1".to_string(), vec![0; 100]).unwrap();
        let stats = opt.cache_stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.total_size, 100);
        assert_eq!(stats.max_size, 1000);
        assert!((stats.utilization - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_quantization_schemes() {
        let schemes = QuantizationInfo::schemes();
        assert!(schemes.contains(&"FP32"));
        assert!(schemes.contains(&"Q4_K_M"));
        assert!(!schemes.is_empty());
    }

    #[test]
    fn test_quantization_estimate_size_zero_bits() {
        let size = QuantizationInfo::estimate_size(1000, 0, 8);
        assert_eq!(size, 1000);
        let size2 = QuantizationInfo::estimate_size(1000, 32, 0);
        assert_eq!(size2, 1000);
    }

    #[test]
    fn test_quantization_is_valid_scheme() {
        assert!(QuantizationInfo::is_valid_scheme("FP32"));
        assert!(QuantizationInfo::is_valid_scheme("Q4_K_M"));
        assert!(!QuantizationInfo::is_valid_scheme("INVALID"));
    }

    #[test]
    fn test_pruning_zero_params() {
        let info = PruningInfo::new(PruningMethod::Magnitude, 0.5, 0, 0);
        assert_eq!(info.calculate_sparsity(), 0.0);
        assert_eq!(info.size_reduction(), 0.0);
    }

    #[test]
    fn test_pruning_methods() {
        assert_eq!(PruningMethod::Magnitude, PruningMethod::Magnitude);
        assert_ne!(PruningMethod::Structured, PruningMethod::Unstructured);
        // Construct the remaining variants to keep them exercised.
        assert_ne!(PruningMethod::GradientBased, PruningMethod::LayerWise);
        assert_eq!(
            PruningMethod::Custom("test".into()),
            PruningMethod::Custom("test".into())
        );
    }

    #[test]
    fn test_model_analyzer_format_size_tb() {
        let tb = 1024u64 * 1024 * 1024 * 1024;
        assert!(ModelAnalyzer::format_size(tb).contains("TB"));
    }

    #[test]
    fn test_model_analyzer_estimate_params_formats() {
        // Test analysis with different formats to hit different estimate branches
        let data = vec![0u8; 4000]; // 4KB
        for fmt in &[
            ModelFormat::GGUF,
            ModelFormat::ONNX,
            ModelFormat::TFLite,
            ModelFormat::Safetensors,
        ] {
            let meta = ModelMetadata::new("m".to_string(), fmt.clone());
            let analysis = ModelAnalyzer::analyze(&data, &meta);
            assert!(analysis.estimated_parameters.is_some());
        }
    }

    #[test]
    fn test_model_exporter_export_with_metadata() {
        let temp_dir = tempfile::tempdir().unwrap();
        let meta = ModelMetadata::new("exported".to_string(), ModelFormat::PyTorch);
        let data = vec![1, 2, 3, 4];
        let path = ModelExporter::export_with_metadata(data, &meta, temp_dir.path()).unwrap();
        assert!(path.exists());
        assert!(temp_dir.path().join("exported.meta.json").exists());
    }

    #[test]
    fn test_model_exporter_export_to_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let models = vec![
            (
                vec![1, 2],
                ModelMetadata::new("a".to_string(), ModelFormat::ONNX),
            ),
            (
                vec![3, 4],
                ModelMetadata::new("b".to_string(), ModelFormat::PyTorch),
            ),
        ];
        let paths = ModelExporter::export_to_directory(models, temp_dir.path()).unwrap();
        assert_eq!(paths.len(), 2);
        for p in &paths {
            assert!(p.exists());
        }
    }

    #[test]
    fn test_deduplicator_find_duplicates() {
        let models = vec![
            ("model_a".to_string(), vec![1, 2, 3]),
            ("model_b".to_string(), vec![1, 2, 3]), // duplicate of a
            ("model_c".to_string(), vec![4, 5, 6]), // unique
        ];
        let dupes = ModelDeduplicator::find_duplicates(models);
        assert_eq!(dupes.len(), 1);
        let names: Vec<&Vec<String>> = dupes.values().collect();
        let names = &names[0];
        assert!(names.contains(&"model_a".to_string()));
        assert!(names.contains(&"model_b".to_string()));
    }

    #[test]
    fn test_deduplicator_similarity_partial() {
        let data1 = vec![1, 2, 3, 4];
        let data2 = vec![1, 2, 0, 0];
        let score = ModelDeduplicator::similarity_score(&data1, &data2);
        assert!((score - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_similarity_score_different_lengths() {
        let a = vec![1u8, 2, 3];
        let b = vec![1u8, 2, 3, 4];
        let score = ModelDeduplicator::similarity_score(&a, &b);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_estimate_parameters_custom_format() {
        // Covers L411 — wildcard match arm in estimate_parameters for Custom format
        let data = vec![0u8; 1024]; // 1KB
        let meta = ModelMetadata::new(
            "custom_fmt".to_string(),
            ModelFormat::Custom("myformat".to_string()),
        );
        let analysis = ModelAnalyzer::analyze(&data, &meta);
        // Custom format should hit the wildcard `_ => Some(base_estimate)` at L411
        assert!(analysis.estimated_parameters.is_some());
        // base_estimate = 1024 / 4 = 256
        assert_eq!(analysis.estimated_parameters.unwrap(), 256);
    }
    // ── Archive extraction path traversal (zip slip, CWE-22) ────────────────

    #[test]
    fn test_safe_archive_name_accepts_ordinary_names() {
        assert_eq!(
            safe_archive_name("model.safetensors").unwrap(),
            "model.safetensors"
        );
        assert_eq!(safe_archive_name("llama-7b.bin").unwrap(), "llama-7b.bin");
    }

    /// Rejected on every platform: `/` is a separator everywhere, and `..`,
    /// `.` and the empty string are never ordinary file names.
    #[test]
    fn test_safe_archive_name_rejects_escapes() {
        for hostile in [
            "../evil",
            "../../evil",
            "a/b",
            "a/../b",
            "/etc/passwd",
            "..",
            ".",
            "",
        ] {
            assert!(
                safe_archive_name(hostile).is_err(),
                "{hostile:?} must be rejected"
            );
        }
    }

    /// Backslash is a separator on Windows and an ordinary character on Unix,
    /// so the correct answer genuinely differs by platform. `std::path`
    /// already encodes that, and `safe_archive_name` inherits it — these two
    /// tests pin both halves rather than assuming Windows rules everywhere.
    #[cfg(windows)]
    #[test]
    fn test_safe_archive_name_rejects_windows_separators_and_prefixes() {
        for hostile in [
            r"a\b",
            r"..\..\evil",
            r"C:\Windows\evil",
            "C:/Windows/evil",
            r"\\server\share\x",
        ] {
            assert!(
                safe_archive_name(hostile).is_err(),
                "{hostile:?} must be rejected on Windows"
            );
        }
    }

    /// On Unix a member literally named `a\b` is one legal file, not a path.
    /// Extracting it creates a single oddly-named file inside the output
    /// directory, which is not an escape — so accepting it is correct.
    #[cfg(unix)]
    #[test]
    fn test_safe_archive_name_treats_backslash_as_an_ordinary_character() {
        assert_eq!(safe_archive_name(r"a\b").unwrap(), r"a\b");
        assert_eq!(
            safe_archive_name(r"C:\Windows\evil").unwrap(),
            r"C:\Windows\evil"
        );

        // ...but a real Unix traversal is still refused.
        assert!(safe_archive_name("../evil").is_err());
    }

    /// A ZIP whose member name climbs out of the extraction directory must be
    /// refused outright, not extracted and not silently renamed.
    #[test]
    fn test_extract_zip_refuses_a_member_that_escapes() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("evil.zip");

        {
            let file = std::fs::File::create(&archive).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let opts = zip::write::SimpleFileOptions::default();
            zip.start_file("legit.bin", opts).unwrap();
            zip.write_all(b"harmless").unwrap();
            zip.start_file("../../ESCAPED.txt", opts).unwrap();
            zip.write_all(b"pwned").unwrap();
            zip.finish().unwrap();
        }

        let err = ModelArchive::extract_zip(&archive).unwrap_err();
        assert!(
            matches!(err, VaultError::InvalidInput(_)),
            "expected InvalidInput, got {err:?}"
        );
        assert!(err.to_string().contains("ESCAPED.txt"), "got: {err}");
    }

    /// The same for TAR, which previously reduced the name with `file_name()`
    /// — safe from traversal, but it silently turned `../../x` into `x`.
    #[test]
    fn test_extract_tar_refuses_a_member_that_escapes() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("evil.tar");

        {
            let file = std::fs::File::create(&archive).unwrap();
            let mut ar = tar::Builder::new(file);
            let payload = b"pwned";

            // `append_data` refuses a `..` path, which is exactly the point:
            // tar-rs will not *produce* a hostile archive, so one has to be
            // forged by writing the header's name field directly — which is
            // what an attacker does. The reader must not trust it.
            let mut header = tar::Header::new_gnu();
            let hostile = b"../../ESCAPED.txt";
            {
                let gnu = header.as_gnu_mut().expect("new_gnu is a GNU header");
                gnu.name[..hostile.len()].copy_from_slice(hostile);
            }
            header.set_size(payload.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();

            ar.append(&header, &payload[..]).unwrap();
            ar.finish().unwrap();
        }

        // Confirm the forged name really is in the archive, so a future change
        // to tar-rs that sanitises on read cannot make this test vacuous.
        {
            let file = std::fs::File::open(&archive).unwrap();
            let mut probe = tar::Archive::new(file);
            let names: Vec<String> = probe
                .entries()
                .unwrap()
                .map(|e| e.unwrap().path().unwrap().to_string_lossy().into_owned())
                .collect();
            assert!(
                names.iter().any(|n| n.contains("..")),
                "forged archive lost its traversal: {names:?}"
            );
        }

        let err = ModelArchive::extract_tar(&archive).unwrap_err();
        assert!(
            matches!(err, VaultError::InvalidInput(_)),
            "expected InvalidInput, got {err:?}"
        );
    }

    /// Archives this crate produces must still round-trip.
    #[test]
    fn test_round_trip_still_works_for_both_formats() {
        let dir = tempfile::tempdir().unwrap();
        let models = vec![
            ("alpha.safetensors".to_string(), b"aaa".to_vec()),
            ("beta.gguf".to_string(), b"bbb".to_vec()),
        ];

        let tar_path = dir.path().join("a.tar");
        ModelArchive::create_tar(models.clone(), &tar_path).unwrap();
        let mut out = ModelArchive::extract_tar(&tar_path).unwrap();
        out.sort();
        assert_eq!(out, models);

        let zip_path = dir.path().join("a.zip");
        ModelArchive::create_zip(models.clone(), &zip_path).unwrap();
        let mut out = ModelArchive::extract_zip(&zip_path).unwrap();
        out.sort();
        assert_eq!(out, models);
    }

    #[test]
    fn test_archive_budget_rejects_an_oversized_member() {
        let err = check_archive_budget("bomb.bin", MAX_ARCHIVE_MEMBER_BYTES + 1, 0).unwrap_err();
        assert!(
            err.to_string().contains("per-member limit"),
            "expected a per-member limit error, got: {err}"
        );

        // Exactly at the limit is allowed — the cap is inclusive.
        check_archive_budget("big.bin", MAX_ARCHIVE_MEMBER_BYTES, 0).unwrap();
    }

    #[test]
    fn test_archive_budget_rejects_many_members_that_are_each_legal() {
        // Each member is under the per-member cap, so only the running total
        // catches this one.
        let member = MAX_ARCHIVE_MEMBER_BYTES;
        let mut total: u64 = 0;
        let mut rejected_at = None;

        for i in 0..8 {
            match check_archive_budget("chunk.bin", member, total) {
                Ok(()) => total += member,
                Err(err) => {
                    assert!(
                        err.to_string().contains("total limit"),
                        "expected a total-limit error, got: {err}"
                    );
                    rejected_at = Some(i);
                    break;
                }
            }
        }

        assert_eq!(
            rejected_at,
            Some(2),
            "16 GiB of budget should admit exactly two 8 GiB members"
        );
    }

    #[test]
    fn test_archive_budget_does_not_overflow_on_a_u64_max_claim() {
        // `running_total + declared` would wrap without the saturating add, and
        // a wrapped sum compares as *under* the limit.
        let err = check_archive_budget("liar.bin", u64::MAX, u64::MAX).unwrap_err();
        assert!(err.to_string().contains("limit"), "got: {err}");
    }

    #[test]
    fn test_read_member_bounded_rejects_contents_that_overrun_the_declared_size() {
        // The declared size is only a claim; this is the backstop for a member
        // whose header understates it.
        let payload = vec![0u8; 64];

        let mut reader = &payload[..];
        let err = read_member_bounded("liar.bin", &mut reader, 16).unwrap_err();
        assert!(
            err.to_string().contains("understated"),
            "expected an overrun error, got: {err}"
        );

        // A member that fits is returned whole, including one exactly at the
        // limit — the +1 read must not be mistaken for an overrun.
        let mut reader = &payload[..];
        assert_eq!(
            read_member_bounded("ok.bin", &mut reader, 64).unwrap(),
            payload
        );
    }
}
