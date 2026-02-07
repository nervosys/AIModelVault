//! AI Model Utilities - Common operations for AI models
//!
//! Provides utilities for model archiving, compression, retrieval optimization,
//! quantization metadata, pruning info, and model analysis.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{Result, VaultError};
use crate::formats::{ModelFormat, ModelMetadata};

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

        for entry in ar.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| VaultError::InvalidInput("Invalid filename in archive".to_string()))?
                .to_string();

            let mut data = Vec::new();
            entry.read_to_end(&mut data)?;

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
            let options = zip::write::FileOptions::default()
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

        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            let name = file.name().to_string();

            let mut data = Vec::new();
            file.read_to_end(&mut data)?;

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
}
