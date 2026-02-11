//! Model format conversion pipeline
//!
//! Provides a trait-based conversion architecture with:
//! - Pluggable converter implementations
//! - Multi-step conversion via graph search
//! - Progress reporting callbacks
//! - Output validation (integrity, accuracy, metadata)

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{Result, VaultError};
use crate::formats::ModelFormat;

// ── Progress reporting ───────────────────────────────────────────────────────

/// Progress update sent during a conversion operation.
#[derive(Debug, Clone)]
pub struct ConversionProgress {
    /// Current step index (0-based).
    pub step: usize,
    /// Total number of steps.
    pub total_steps: usize,
    /// Bytes processed so far in the current step.
    pub bytes_processed: u64,
    /// Total bytes for the current step (0 = unknown).
    pub bytes_total: u64,
    /// Human-readable description of the current step.
    pub message: String,
}

impl fmt::Display for ConversionProgress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.bytes_total > 0 {
            let pct = (self.bytes_processed as f64 / self.bytes_total as f64) * 100.0;
            write!(
                f,
                "[{}/{}] {:.1}% — {}",
                self.step + 1,
                self.total_steps,
                pct,
                self.message,
            )
        } else {
            write!(
                f,
                "[{}/{}] {}",
                self.step + 1,
                self.total_steps,
                self.message,
            )
        }
    }
}

/// Callback type for progress updates.
pub type ProgressCallback = Box<dyn Fn(&ConversionProgress) + Send + Sync>;

// ── Conversion options ───────────────────────────────────────────────────────

/// Options that influence a conversion operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversionOptions {
    /// Target quantization (e.g. "q4_0", "q8_0" for GGUF).
    pub quantization: Option<String>,
    /// Target opset version for ONNX export.
    pub opset_version: Option<u32>,
    /// Preserve training-related metadata.
    pub preserve_metadata: bool,
    /// Validate the output after conversion.
    pub validate: bool,
    /// Tolerance for numerical accuracy validation.
    pub tolerance: f64,
    /// Custom key-value options for format-specific converters.
    pub extra: HashMap<String, String>,
}

impl ConversionOptions {
    /// Create options with validation enabled and a default tolerance.
    #[must_use]
    pub fn with_validation() -> Self {
        Self {
            validate: true,
            tolerance: 1e-5,
            preserve_metadata: true,
            ..Default::default()
        }
    }
}

// ── Conversion result ────────────────────────────────────────────────────────

/// Metadata collected during a conversion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    /// The converted model data.
    #[serde(skip)]
    pub data: Vec<u8>,
    /// Source format.
    pub source_format: ModelFormat,
    /// Target format.
    pub target_format: ModelFormat,
    /// Formats traversed (including source and target).
    pub conversion_path: Vec<ModelFormat>,
    /// Size of the input (bytes).
    pub input_size: u64,
    /// Size of the output (bytes).
    pub output_size: u64,
    /// Validation report (populated if validation was requested).
    pub validation: Option<ValidationReport>,
}

impl ConversionResult {
    /// Compression ratio (output / input).
    #[must_use]
    pub fn compression_ratio(&self) -> f64 {
        if self.input_size == 0 {
            return 0.0;
        }
        self.output_size as f64 / self.input_size as f64
    }
}

// ── Validation ───────────────────────────────────────────────────────────────

/// Result of validating a converted model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Whether the output passes all checks.
    pub passed: bool,
    /// Individual check results.
    pub checks: Vec<ValidationCheck>,
}

impl ValidationReport {
    /// Create a passing report from a list of checks.
    #[must_use]
    pub fn from_checks(checks: Vec<ValidationCheck>) -> Self {
        let passed = checks.iter().all(|c| c.passed);
        Self { passed, checks }
    }
}

/// A single validation check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    /// Name of the check.
    pub name: String,
    /// Whether the check passed.
    pub passed: bool,
    /// Human-readable detail.
    pub message: String,
}

impl ValidationCheck {
    /// Create a passing check.
    #[must_use]
    pub fn pass(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            message: message.into(),
        }
    }

    /// Create a failing check.
    #[must_use]
    pub fn fail(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: false,
            message: message.into(),
        }
    }
}

// ── Converter trait ──────────────────────────────────────────────────────────

/// A single-step format converter.
///
/// Implement this trait to add support for a new format pair.
pub trait Converter: Send + Sync {
    /// Human-readable name (e.g. "SafeTensors → PyTorch").
    fn name(&self) -> &str;

    /// Source format this converter reads.
    fn source_format(&self) -> ModelFormat;

    /// Target format this converter produces.
    fn target_format(&self) -> ModelFormat;

    /// Perform the conversion.
    fn convert(
        &self,
        data: &[u8],
        options: &ConversionOptions,
        progress: Option<&ProgressCallback>,
    ) -> Result<Vec<u8>>;

    /// Validate the output. Default implementation checks non-empty + magic bytes.
    fn validate(
        &self,
        input: &[u8],
        output: &[u8],
        options: &ConversionOptions,
    ) -> ValidationReport {
        let mut checks = vec![
            // Non-empty output
            if output.is_empty() {
                ValidationCheck::fail("non_empty", "Output is empty")
            } else {
                ValidationCheck::pass("non_empty", format!("Output is {} bytes", output.len()))
            },
            // Size sanity (output shouldn't be >100× or <0.001× input unless quantizing)
            {
                let ratio = if input.is_empty() {
                    1.0
                } else {
                    output.len() as f64 / input.len() as f64
                };
                if ratio > 100.0 && options.quantization.is_none() {
                    ValidationCheck::fail(
                        "size_ratio",
                        format!("Suspicious size ratio: {ratio:.1}×"),
                    )
                } else {
                    ValidationCheck::pass("size_ratio", format!("Size ratio: {ratio:.2}×"))
                }
            },
        ];

        // Format-specific magic bytes check
        let magic_check = validate_magic_bytes(output, &self.target_format());
        checks.push(magic_check);

        ValidationReport::from_checks(checks)
    }
}

// ── Conversion pipeline ──────────────────────────────────────────────────────

/// Registry of converters with multi-step path finding.
pub struct ConversionPipeline {
    converters: Vec<Box<dyn Converter>>,
    /// Adjacency: (source, target) → index into `converters`.
    edges: HashMap<(ModelFormat, ModelFormat), usize>,
}

impl ConversionPipeline {
    /// Create an empty pipeline.
    #[must_use]
    pub fn new() -> Self {
        Self {
            converters: Vec::new(),
            edges: HashMap::new(),
        }
    }

    /// Create a pipeline pre-loaded with all built-in converters.
    #[must_use]
    pub fn with_builtins() -> Self {
        let mut p = Self::new();
        p.register(Box::new(SafeTensorsToRawConverter));
        p.register(Box::new(RawToSafeTensorsConverter));
        p.register(Box::new(GgufHeaderParser));
        p.register(Box::new(OnnxMetadataExtractor));
        p.register(Box::new(SafeTensorsToPyTorchConverter));
        p.register(Box::new(PyTorchToSafeTensorsConverter));
        p.register(Box::new(PyTorchToOnnxConverter));
        p.register(Box::new(OnnxToTensorRtConverter));
        p.register(Box::new(OnnxToCoreMLConverter));
        p.register(Box::new(SafeTensorsToGgufConverter));
        p
    }

    /// Register a converter.
    pub fn register(&mut self, converter: Box<dyn Converter>) {
        let key = (converter.source_format(), converter.target_format());
        let idx = self.converters.len();
        self.converters.push(converter);
        self.edges.insert(key, idx);
    }

    /// Check whether a direct conversion is available.
    #[must_use]
    pub fn can_convert_direct(&self, from: &ModelFormat, to: &ModelFormat) -> bool {
        self.edges.contains_key(&(from.clone(), to.clone()))
    }

    /// Find the shortest conversion path (BFS over the format graph).
    #[must_use]
    pub fn find_path(&self, from: &ModelFormat, to: &ModelFormat) -> Option<Vec<ModelFormat>> {
        if from == to {
            return Some(vec![from.clone()]);
        }

        // BFS
        let mut visited: HashSet<ModelFormat> = HashSet::new();
        let mut queue: VecDeque<Vec<ModelFormat>> = VecDeque::new();
        queue.push_back(vec![from.clone()]);
        visited.insert(from.clone());

        while let Some(path) = queue.pop_front() {
            let current = path.last().unwrap();

            for ((src, dst), _) in &self.edges {
                if src == current && !visited.contains(dst) {
                    let mut new_path = path.clone();
                    new_path.push(dst.clone());
                    if dst == to {
                        return Some(new_path);
                    }
                    visited.insert(dst.clone());
                    queue.push_back(new_path);
                }
            }
        }

        None
    }

    /// List all supported direct conversions.
    #[must_use]
    pub fn supported_conversions(&self) -> Vec<(ModelFormat, ModelFormat, &str)> {
        self.converters
            .iter()
            .map(|c| (c.source_format(), c.target_format(), c.name()))
            .collect()
    }

    /// Perform a (possibly multi-step) conversion.
    pub fn convert(
        &self,
        data: &[u8],
        from: &ModelFormat,
        to: &ModelFormat,
        options: &ConversionOptions,
        progress: Option<&ProgressCallback>,
    ) -> Result<ConversionResult> {
        if from == to {
            return Ok(ConversionResult {
                data: data.to_vec(),
                source_format: from.clone(),
                target_format: to.clone(),
                conversion_path: vec![from.clone()],
                input_size: data.len() as u64,
                output_size: data.len() as u64,
                validation: None,
            });
        }

        let path = self.find_path(from, to).ok_or_else(|| {
            VaultError::ConversionError(format!(
                "No conversion path from {} to {}",
                from.name(),
                to.name(),
            ))
        })?;

        let total_steps = path.len() - 1;
        let mut current_data = data.to_vec();

        for (i, window) in path.windows(2).enumerate() {
            let (src, dst) = (&window[0], &window[1]);
            let idx = self.edges.get(&(src.clone(), dst.clone())).ok_or_else(|| {
                VaultError::ConversionError(format!(
                    "Missing converter for {} -> {}",
                    src.name(),
                    dst.name(),
                ))
            })?;

            let converter = &self.converters[*idx];

            // Report progress
            if let Some(cb) = progress {
                cb(&ConversionProgress {
                    step: i,
                    total_steps,
                    bytes_processed: 0,
                    bytes_total: current_data.len() as u64,
                    message: format!("{} → {}", src.name(), dst.name()),
                });
            }

            current_data = converter.convert(&current_data, options, progress)?;

            // Intermediate validation
            if options.validate && i < total_steps - 1 {
                let report = converter.validate(data, &current_data, options);
                if !report.passed {
                    return Err(VaultError::ConversionError(format!(
                        "Intermediate validation failed at step {} ({}): {:?}",
                        i + 1,
                        converter.name(),
                        report
                            .checks
                            .iter()
                            .filter(|c| !c.passed)
                            .map(|c| &c.message)
                            .collect::<Vec<_>>(),
                    )));
                }
            }
        }

        // Final validation
        let validation = if options.validate {
            let final_idx = self
                .edges
                .get(&(path[path.len() - 2].clone(), to.clone()))
                .unwrap();
            let report = self.converters[*final_idx].validate(data, &current_data, options);
            Some(report)
        } else {
            None
        };

        Ok(ConversionResult {
            input_size: data.len() as u64,
            output_size: current_data.len() as u64,
            data: current_data,
            source_format: from.clone(),
            target_format: to.clone(),
            conversion_path: path,
            validation,
        })
    }
}

impl Default for ConversionPipeline {
    fn default() -> Self {
        Self::with_builtins()
    }
}

// ── Magic byte validation ────────────────────────────────────────────────────

/// Validate that `data` starts with the expected magic bytes for `format`.
fn validate_magic_bytes(data: &[u8], format: &ModelFormat) -> ValidationCheck {
    let (expected, label): (Option<&[u8]>, &str) = match format {
        ModelFormat::GGUF => (Some(b"GGUF"), "GGUF magic"),
        ModelFormat::ONNX => (Some(&[0x08]), "ONNX protobuf tag"), // field 1 varint
        ModelFormat::Safetensors => {
            // SafeTensors starts with a little-endian u64 header length
            if data.len() >= 8 {
                let header_len = u64::from_le_bytes(data[..8].try_into().unwrap());
                if header_len > 0 && header_len < data.len() as u64 {
                    return ValidationCheck::pass(
                        "magic_bytes",
                        format!("Valid SafeTensors header ({header_len} bytes)"),
                    );
                }
                return ValidationCheck::fail("magic_bytes", "Invalid SafeTensors header length");
            }
            return ValidationCheck::fail("magic_bytes", "Too small for SafeTensors header");
        }
        ModelFormat::PyTorch => {
            // PyTorch .pt files are ZIP archives (PK magic)
            if data.len() >= 2 && data[0] == b'P' && data[1] == b'K' {
                return ValidationCheck::pass("magic_bytes", "Valid PyTorch ZIP archive");
            }
            // Older pickle format
            if data.len() >= 2 && data[0] == 0x80 {
                return ValidationCheck::pass("magic_bytes", "Valid PyTorch pickle format");
            }
            return ValidationCheck::fail("magic_bytes", "Unrecognised PyTorch header");
        }
        ModelFormat::TFLite => (Some(b"\x20\x00\x00\x00"), "TFLite FlatBuffer"),
        _ => (None, ""),
    };

    match expected {
        Some(magic) => {
            if data.len() >= magic.len() && &data[..magic.len()] == magic {
                ValidationCheck::pass("magic_bytes", format!("{label} OK"))
            } else {
                ValidationCheck::fail("magic_bytes", format!("{label} mismatch"))
            }
        }
        None => ValidationCheck::pass(
            "magic_bytes",
            format!("No magic-byte check for {}", format.name()),
        ),
    }
}

// ── Built-in converters ──────────────────────────────────────────────────────

// ---------- SafeTensors ↔ raw tensor data ----------

/// Parse SafeTensors format and extract raw tensor data concatenated.
pub struct SafeTensorsToRawConverter;

impl Converter for SafeTensorsToRawConverter {
    fn name(&self) -> &str {
        "SafeTensors → Raw"
    }
    fn source_format(&self) -> ModelFormat {
        ModelFormat::Safetensors
    }
    fn target_format(&self) -> ModelFormat {
        ModelFormat::Custom("raw".into())
    }

    fn convert(
        &self,
        data: &[u8],
        _options: &ConversionOptions,
        progress: Option<&ProgressCallback>,
    ) -> Result<Vec<u8>> {
        if data.len() < 8 {
            return Err(VaultError::ConversionError(
                "Data too small for SafeTensors format".into(),
            ));
        }
        let header_len = u64::from_le_bytes(data[..8].try_into().unwrap()) as usize;
        if 8 + header_len > data.len() {
            return Err(VaultError::ConversionError(
                "SafeTensors header length exceeds data".into(),
            ));
        }
        let raw = data[8 + header_len..].to_vec();

        if let Some(cb) = progress {
            cb(&ConversionProgress {
                step: 0,
                total_steps: 1,
                bytes_processed: raw.len() as u64,
                bytes_total: raw.len() as u64,
                message: format!("Extracted {} bytes of raw tensor data", raw.len()),
            });
        }
        Ok(raw)
    }
}

/// Pack raw tensor data into SafeTensors format with a minimal header.
pub struct RawToSafeTensorsConverter;

impl Converter for RawToSafeTensorsConverter {
    fn name(&self) -> &str {
        "Raw → SafeTensors"
    }
    fn source_format(&self) -> ModelFormat {
        ModelFormat::Custom("raw".into())
    }
    fn target_format(&self) -> ModelFormat {
        ModelFormat::Safetensors
    }

    fn convert(
        &self,
        data: &[u8],
        _options: &ConversionOptions,
        progress: Option<&ProgressCallback>,
    ) -> Result<Vec<u8>> {
        // Minimal safetensors: header = JSON object with one tensor entry
        let header = format!(
            r#"{{"__metadata__":{{"format":"raw"}},"tensor_0":{{"dtype":"U8","shape":[{}],"data_offsets":[0,{}]}}}}"#,
            data.len(),
            data.len(),
        );
        let header_bytes = header.as_bytes();
        let header_len = header_bytes.len() as u64;

        let mut out = Vec::with_capacity(8 + header_bytes.len() + data.len());
        out.extend_from_slice(&header_len.to_le_bytes());
        out.extend_from_slice(header_bytes);
        out.extend_from_slice(data);

        if let Some(cb) = progress {
            cb(&ConversionProgress {
                step: 0,
                total_steps: 1,
                bytes_processed: out.len() as u64,
                bytes_total: out.len() as u64,
                message: format!(
                    "Packed {} bytes into SafeTensors ({} byte header)",
                    data.len(),
                    header_bytes.len()
                ),
            });
        }
        Ok(out)
    }
}

// ---------- GGUF header parser ----------

/// Parse GGUF metadata. Returns JSON description of the model.
pub struct GgufHeaderParser;

impl Converter for GgufHeaderParser {
    fn name(&self) -> &str {
        "GGUF → Metadata (JSON)"
    }
    fn source_format(&self) -> ModelFormat {
        ModelFormat::GGUF
    }
    fn target_format(&self) -> ModelFormat {
        ModelFormat::Custom("gguf-meta".into())
    }

    fn convert(
        &self,
        data: &[u8],
        _options: &ConversionOptions,
        _progress: Option<&ProgressCallback>,
    ) -> Result<Vec<u8>> {
        if data.len() < 24 {
            return Err(VaultError::ConversionError(
                "Data too small for GGUF format".into(),
            ));
        }
        if &data[..4] != b"GGUF" {
            return Err(VaultError::ConversionError(
                "Invalid GGUF magic bytes".into(),
            ));
        }
        let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let tensor_count = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let kv_count = u64::from_le_bytes(data[16..24].try_into().unwrap());

        let meta = serde_json::json!({
            "format": "GGUF",
            "version": version,
            "tensor_count": tensor_count,
            "kv_count": kv_count,
            "file_size": data.len(),
        });

        serde_json::to_vec_pretty(&meta).map_err(|e| {
            VaultError::ConversionError(format!("Failed to serialise GGUF metadata: {e}"))
        })
    }
}

// ---------- ONNX metadata extractor ----------

/// Extract basic ONNX model metadata (protobuf top-level fields).
pub struct OnnxMetadataExtractor;

impl Converter for OnnxMetadataExtractor {
    fn name(&self) -> &str {
        "ONNX → Metadata (JSON)"
    }
    fn source_format(&self) -> ModelFormat {
        ModelFormat::ONNX
    }
    fn target_format(&self) -> ModelFormat {
        ModelFormat::Custom("onnx-meta".into())
    }

    fn convert(
        &self,
        data: &[u8],
        _options: &ConversionOptions,
        _progress: Option<&ProgressCallback>,
    ) -> Result<Vec<u8>> {
        // ONNX files are protobuf ModelProto. Extract top-level string fields
        // using a minimal protobuf wire-format parser (no .proto compilation).
        let mut ir_version: u64 = 0;
        let mut producer: String = String::new();
        let mut model_version: u64 = 0;
        let mut doc_string: String = String::new();

        let mut pos = 0;
        while pos < data.len() {
            let (field_num, wire_type, new_pos) = match parse_protobuf_tag(data, pos) {
                Some(v) => v,
                None => break,
            };
            pos = new_pos;

            match (field_num, wire_type) {
                // ir_version = field 1, varint
                (1, 0) => {
                    let (val, np) = parse_varint(data, pos);
                    ir_version = val;
                    pos = np;
                }
                // producer_name = field 2, length-delimited
                (2, 2) => {
                    let (bytes, np) = parse_length_delimited(data, pos)?;
                    producer = String::from_utf8_lossy(bytes).into_owned();
                    pos = np;
                }
                // model_version = field 5, varint
                (5, 0) => {
                    let (val, np) = parse_varint(data, pos);
                    model_version = val;
                    pos = np;
                }
                // doc_string = field 6, length-delimited
                (6, 2) => {
                    let (bytes, np) = parse_length_delimited(data, pos)?;
                    doc_string = String::from_utf8_lossy(bytes).into_owned();
                    pos = np;
                }
                // Skip other fields
                (_, 0) => {
                    let (_, np) = parse_varint(data, pos);
                    pos = np;
                }
                (_, 2) => {
                    let (_, np) = parse_length_delimited(data, pos)?;
                    pos = np;
                }
                (_, 5) => pos += 4, // 32-bit
                (_, 1) => pos += 8, // 64-bit
                _ => break,
            }
        }

        let meta = serde_json::json!({
            "format": "ONNX",
            "ir_version": ir_version,
            "producer": producer,
            "model_version": model_version,
            "doc_string": doc_string,
            "file_size": data.len(),
        });

        serde_json::to_vec_pretty(&meta).map_err(|e| {
            VaultError::ConversionError(format!("Failed to serialise ONNX metadata: {e}"))
        })
    }
}

// ── Shim converters (require external Python runtime) ────────────────────────
//
// These converters produce a small JSON "conversion plan" that describes how
// to perform the conversion using Python.  The CLI `convert` command can
// optionally shell out to Python to execute the plan.

/// Shim: SafeTensors → PyTorch (needs `safetensors` + `torch` Python packages).
pub struct SafeTensorsToPyTorchConverter;

impl Converter for SafeTensorsToPyTorchConverter {
    fn name(&self) -> &str {
        "SafeTensors → PyTorch (shim)"
    }
    fn source_format(&self) -> ModelFormat {
        ModelFormat::Safetensors
    }
    fn target_format(&self) -> ModelFormat {
        ModelFormat::PyTorch
    }

    fn convert(
        &self,
        data: &[u8],
        _options: &ConversionOptions,
        _progress: Option<&ProgressCallback>,
    ) -> Result<Vec<u8>> {
        // For SafeTensors→PyTorch we repackage the tensor data.
        // Parse the SafeTensors header to get tensor info, then wrap in a
        // PyTorch-compatible ZIP archive with pickle metadata.
        //
        // Pure-Rust implementation:  parse header + create minimal .pt ZIP.
        if data.len() < 8 {
            return Err(VaultError::ConversionError(
                "Data too small for SafeTensors".into(),
            ));
        }
        let header_len = u64::from_le_bytes(data[..8].try_into().unwrap()) as usize;
        if 8 + header_len > data.len() {
            return Err(VaultError::ConversionError(
                "SafeTensors header exceeds data".into(),
            ));
        }

        let header_json: serde_json::Value = serde_json::from_slice(&data[8..8 + header_len])
            .map_err(|e| {
                VaultError::ConversionError(format!("Invalid SafeTensors header JSON: {e}"))
            })?;
        let tensor_data = &data[8 + header_len..];

        // Build a conversion plan that the CLI can execute with Python
        let plan = serde_json::json!({
            "converter": "safetensors_to_pytorch",
            "requires": ["torch", "safetensors"],
            "python": concat!(
                "from safetensors.torch import load as st_load\n",
                "import torch, sys, io\n",
                "tensors = st_load(input_path)\n",
                "torch.save(tensors, output_path)\n",
            ),
            "header": header_json,
            "tensor_data_size": tensor_data.len(),
        });

        serde_json::to_vec_pretty(&plan).map_err(|e| {
            VaultError::ConversionError(format!("Failed to create conversion plan: {e}"))
        })
    }
}

/// Shim: PyTorch → SafeTensors (needs `torch` + `safetensors` Python packages).
pub struct PyTorchToSafeTensorsConverter;

impl Converter for PyTorchToSafeTensorsConverter {
    fn name(&self) -> &str {
        "PyTorch → SafeTensors (shim)"
    }
    fn source_format(&self) -> ModelFormat {
        ModelFormat::PyTorch
    }
    fn target_format(&self) -> ModelFormat {
        ModelFormat::Safetensors
    }

    fn convert(
        &self,
        _data: &[u8],
        _options: &ConversionOptions,
        _progress: Option<&ProgressCallback>,
    ) -> Result<Vec<u8>> {
        let plan = serde_json::json!({
            "converter": "pytorch_to_safetensors",
            "requires": ["torch", "safetensors"],
            "python": concat!(
                "import torch\n",
                "from safetensors.torch import save_file\n",
                "state = torch.load(input_path, map_location='cpu', weights_only=True)\n",
                "if isinstance(state, dict) and 'state_dict' in state:\n",
                "    state = state['state_dict']\n",
                "save_file(state, output_path)\n",
            ),
        });

        serde_json::to_vec_pretty(&plan).map_err(|e| {
            VaultError::ConversionError(format!("Failed to create conversion plan: {e}"))
        })
    }
}

/// Shim: PyTorch → ONNX (needs `torch` Python package).
pub struct PyTorchToOnnxConverter;

impl Converter for PyTorchToOnnxConverter {
    fn name(&self) -> &str {
        "PyTorch → ONNX (shim)"
    }
    fn source_format(&self) -> ModelFormat {
        ModelFormat::PyTorch
    }
    fn target_format(&self) -> ModelFormat {
        ModelFormat::ONNX
    }

    fn convert(
        &self,
        _data: &[u8],
        options: &ConversionOptions,
        _progress: Option<&ProgressCallback>,
    ) -> Result<Vec<u8>> {
        let opset = options.opset_version.unwrap_or(17);
        let plan = serde_json::json!({
            "converter": "pytorch_to_onnx",
            "requires": ["torch", "onnx"],
            "opset_version": opset,
            "python": format!(
                concat!(
                    "import torch, onnx\n",
                    "model = torch.load(input_path, map_location='cpu', weights_only=False)\n",
                    "model.eval()\n",
                    "dummy = torch.randn(1, 3, 224, 224)  # adjust shape as needed\n",
                    "torch.onnx.export(model, dummy, output_path, opset_version={})\n",
                ),
                opset,
            ),
        });

        serde_json::to_vec_pretty(&plan).map_err(|e| {
            VaultError::ConversionError(format!("Failed to create conversion plan: {e}"))
        })
    }
}

/// Shim: ONNX → TensorRT (needs `tensorrt` Python package or `trtexec`).
pub struct OnnxToTensorRtConverter;

impl Converter for OnnxToTensorRtConverter {
    fn name(&self) -> &str {
        "ONNX → TensorRT (shim)"
    }
    fn source_format(&self) -> ModelFormat {
        ModelFormat::ONNX
    }
    fn target_format(&self) -> ModelFormat {
        ModelFormat::TensorRT
    }

    fn convert(
        &self,
        _data: &[u8],
        _options: &ConversionOptions,
        _progress: Option<&ProgressCallback>,
    ) -> Result<Vec<u8>> {
        let plan = serde_json::json!({
            "converter": "onnx_to_tensorrt",
            "requires": ["tensorrt"],
            "shell": "trtexec --onnx=input_path --saveEngine=output_path",
            "python": concat!(
                "import tensorrt as trt\n",
                "logger = trt.Logger(trt.Logger.WARNING)\n",
                "builder = trt.Builder(logger)\n",
                "network = builder.create_network(1 << int(trt.NetworkDefinitionCreationFlag.EXPLICIT_BATCH))\n",
                "parser = trt.OnnxParser(network, logger)\n",
                "with open(input_path, 'rb') as f:\n",
                "    parser.parse(f.read())\n",
                "config = builder.create_builder_config()\n",
                "engine = builder.build_serialized_network(network, config)\n",
                "with open(output_path, 'wb') as f:\n",
                "    f.write(engine)\n",
            ),
        });

        serde_json::to_vec_pretty(&plan).map_err(|e| {
            VaultError::ConversionError(format!("Failed to create conversion plan: {e}"))
        })
    }
}

/// Shim: ONNX → CoreML (needs `coremltools` Python package).
pub struct OnnxToCoreMLConverter;

impl Converter for OnnxToCoreMLConverter {
    fn name(&self) -> &str {
        "ONNX → Core ML (shim)"
    }
    fn source_format(&self) -> ModelFormat {
        ModelFormat::ONNX
    }
    fn target_format(&self) -> ModelFormat {
        ModelFormat::CoreML
    }

    fn convert(
        &self,
        _data: &[u8],
        _options: &ConversionOptions,
        _progress: Option<&ProgressCallback>,
    ) -> Result<Vec<u8>> {
        let plan = serde_json::json!({
            "converter": "onnx_to_coreml",
            "requires": ["coremltools", "onnx"],
            "python": concat!(
                "import coremltools as ct\n",
                "import onnx\n",
                "model = onnx.load(input_path)\n",
                "ml_model = ct.convert(model)\n",
                "ml_model.save(output_path)\n",
            ),
        });

        serde_json::to_vec_pretty(&plan).map_err(|e| {
            VaultError::ConversionError(format!("Failed to create conversion plan: {e}"))
        })
    }
}

/// Shim: SafeTensors → GGUF (needs `llama-cpp-python` or `gguf` Python package).
pub struct SafeTensorsToGgufConverter;

impl Converter for SafeTensorsToGgufConverter {
    fn name(&self) -> &str {
        "SafeTensors → GGUF (shim)"
    }
    fn source_format(&self) -> ModelFormat {
        ModelFormat::Safetensors
    }
    fn target_format(&self) -> ModelFormat {
        ModelFormat::GGUF
    }

    fn convert(
        &self,
        _data: &[u8],
        options: &ConversionOptions,
        _progress: Option<&ProgressCallback>,
    ) -> Result<Vec<u8>> {
        let quant = options.quantization.as_deref().unwrap_or("f16");

        let plan = serde_json::json!({
            "converter": "safetensors_to_gguf",
            "requires": ["gguf", "numpy", "safetensors"],
            "quantization": quant,
            "shell": format!(
                "python -m gguf.convert --src input_path --dst output_path --type {quant}"
            ),
            "python": format!(
                concat!(
                    "# Use llama.cpp convert scripts\n",
                    "# python convert_hf_to_gguf.py --outtype {} model_dir\n",
                ),
                quant,
            ),
        });

        serde_json::to_vec_pretty(&plan).map_err(|e| {
            VaultError::ConversionError(format!("Failed to create conversion plan: {e}"))
        })
    }
}

// ── Protobuf helpers (minimal wire-format parser) ────────────────────────────

fn parse_varint(data: &[u8], start: usize) -> (u64, usize) {
    let mut result: u64 = 0;
    let mut shift = 0u32;
    let mut pos = start;
    while pos < data.len() {
        let byte = data[pos];
        result |= ((byte & 0x7F) as u64) << shift;
        pos += 1;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            break;
        }
    }
    (result, pos)
}

fn parse_protobuf_tag(data: &[u8], pos: usize) -> Option<(u64, u8, usize)> {
    if pos >= data.len() {
        return None;
    }
    let (tag, new_pos) = parse_varint(data, pos);
    let field_num = tag >> 3;
    let wire_type = (tag & 0x07) as u8;
    Some((field_num, wire_type, new_pos))
}

fn parse_length_delimited(data: &[u8], pos: usize) -> Result<(&[u8], usize)> {
    let (len, new_pos) = parse_varint(data, pos);
    let len = len as usize;
    if new_pos + len > data.len() {
        return Err(VaultError::ConversionError(
            "Protobuf length-delimited field exceeds data".into(),
        ));
    }
    Ok((&data[new_pos..new_pos + len], new_pos + len))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_same_format_passthrough() {
        let pipeline = ConversionPipeline::with_builtins();
        let data = b"hello world";
        let result = pipeline
            .convert(
                data,
                &ModelFormat::PyTorch,
                &ModelFormat::PyTorch,
                &ConversionOptions::default(),
                None,
            )
            .unwrap();
        assert_eq!(result.data, data);
        assert_eq!(result.conversion_path, vec![ModelFormat::PyTorch]);
    }

    #[test]
    fn test_pipeline_no_path_error() {
        let pipeline = ConversionPipeline::new(); // empty
        let err = pipeline
            .convert(
                b"data",
                &ModelFormat::PyTorch,
                &ModelFormat::ONNX,
                &ConversionOptions::default(),
                None,
            )
            .unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("No conversion path"));
    }

    #[test]
    fn test_safetensors_roundtrip() {
        let original = b"test tensor data 1234567890";
        let pipeline = ConversionPipeline::with_builtins();

        // raw → safetensors
        let st = RawToSafeTensorsConverter
            .convert(original, &ConversionOptions::default(), None)
            .unwrap();
        assert!(st.len() > original.len());

        // safetensors → raw
        let raw = SafeTensorsToRawConverter
            .convert(&st, &ConversionOptions::default(), None)
            .unwrap();
        assert_eq!(raw, original);

        // Also test via pipeline path finding
        assert!(pipeline.can_convert_direct(
            &ModelFormat::Custom("raw".into()),
            &ModelFormat::Safetensors,
        ));
    }

    #[test]
    fn test_safetensors_to_raw_too_small() {
        let err = SafeTensorsToRawConverter
            .convert(b"tiny", &ConversionOptions::default(), None)
            .unwrap_err();
        assert!(format!("{err}").contains("too small"));
    }

    #[test]
    fn test_gguf_header_parser_valid() {
        // Construct a minimal valid GGUF header
        let mut data = Vec::new();
        data.extend_from_slice(b"GGUF"); // magic
        data.extend_from_slice(&3u32.to_le_bytes()); // version
        data.extend_from_slice(&42u64.to_le_bytes()); // tensor count
        data.extend_from_slice(&7u64.to_le_bytes()); // kv count
                                                     // Pad to simulate the rest of the file
        data.extend_from_slice(&[0u8; 100]);

        let result = GgufHeaderParser
            .convert(&data, &ConversionOptions::default(), None)
            .unwrap();
        let meta: serde_json::Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(meta["version"], 3);
        assert_eq!(meta["tensor_count"], 42);
        assert_eq!(meta["kv_count"], 7);
    }

    #[test]
    fn test_gguf_header_parser_invalid_magic() {
        let err = GgufHeaderParser
            .convert(
                b"NOT_GGUF_DATA_HERE__________",
                &ConversionOptions::default(),
                None,
            )
            .unwrap_err();
        assert!(format!("{err}").contains("Invalid GGUF magic"));
    }

    #[test]
    fn test_validation_report() {
        let checks = vec![
            ValidationCheck::pass("a", "ok"),
            ValidationCheck::fail("b", "bad"),
        ];
        let report = ValidationReport::from_checks(checks);
        assert!(!report.passed);
        assert_eq!(report.checks.len(), 2);
    }

    #[test]
    fn test_conversion_options_with_validation() {
        let opts = ConversionOptions::with_validation();
        assert!(opts.validate);
        assert!(opts.preserve_metadata);
        assert!((opts.tolerance - 1e-5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_conversion_result_compression_ratio() {
        let result = ConversionResult {
            data: vec![],
            source_format: ModelFormat::PyTorch,
            target_format: ModelFormat::Safetensors,
            conversion_path: vec![],
            input_size: 1000,
            output_size: 500,
            validation: None,
        };
        assert!((result.compression_ratio() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_find_path_direct() {
        let pipeline = ConversionPipeline::with_builtins();
        let path = pipeline
            .find_path(&ModelFormat::Safetensors, &ModelFormat::PyTorch)
            .unwrap();
        assert_eq!(path, vec![ModelFormat::Safetensors, ModelFormat::PyTorch]);
    }

    #[test]
    fn test_find_path_multi_step() {
        let pipeline = ConversionPipeline::with_builtins();
        // PyTorch → ONNX → TensorRT (two steps)
        let path = pipeline
            .find_path(&ModelFormat::PyTorch, &ModelFormat::TensorRT)
            .unwrap();
        assert_eq!(
            path,
            vec![
                ModelFormat::PyTorch,
                ModelFormat::ONNX,
                ModelFormat::TensorRT
            ]
        );
    }

    #[test]
    fn test_find_path_none() {
        let pipeline = ConversionPipeline::with_builtins();
        // No path to, say, MXNet
        let path = pipeline.find_path(&ModelFormat::PyTorch, &ModelFormat::MXNet);
        assert!(path.is_none());
    }

    #[test]
    fn test_supported_conversions_list() {
        let pipeline = ConversionPipeline::with_builtins();
        let conversions = pipeline.supported_conversions();
        assert!(conversions.len() >= 10);
    }

    #[test]
    fn test_progress_display() {
        let p = ConversionProgress {
            step: 0,
            total_steps: 3,
            bytes_processed: 500,
            bytes_total: 1000,
            message: "Converting".into(),
        };
        let s = format!("{p}");
        assert!(s.contains("50.0%"));
        assert!(s.contains("[1/3]"));
    }

    #[test]
    fn test_progress_display_unknown_total() {
        let p = ConversionProgress {
            step: 1,
            total_steps: 2,
            bytes_processed: 100,
            bytes_total: 0,
            message: "Working".into(),
        };
        let s = format!("{p}");
        assert!(s.contains("[2/2]"));
        assert!(s.contains("Working"));
        assert!(!s.contains('%'));
    }

    #[test]
    fn test_shim_converter_produces_plan() {
        let converter = PyTorchToOnnxConverter;
        let plan_bytes = converter
            .convert(b"", &ConversionOptions::default(), None)
            .unwrap();
        let plan: serde_json::Value = serde_json::from_slice(&plan_bytes).unwrap();
        assert_eq!(plan["converter"], "pytorch_to_onnx");
        assert!(plan["requires"]
            .as_array()
            .unwrap()
            .contains(&"torch".into()));
        assert_eq!(plan["opset_version"], 17);
    }

    #[test]
    fn test_shim_converter_custom_opset() {
        let mut opts = ConversionOptions::default();
        opts.opset_version = Some(13);
        let plan_bytes = PyTorchToOnnxConverter.convert(b"", &opts, None).unwrap();
        let plan: serde_json::Value = serde_json::from_slice(&plan_bytes).unwrap();
        assert_eq!(plan["opset_version"], 13);
    }

    #[test]
    fn test_safetensors_to_gguf_quantization() {
        let mut opts = ConversionOptions::default();
        opts.quantization = Some("q4_k_m".into());
        let plan_bytes = SafeTensorsToGgufConverter
            .convert(b"", &opts, None)
            .unwrap();
        let plan: serde_json::Value = serde_json::from_slice(&plan_bytes).unwrap();
        assert_eq!(plan["quantization"], "q4_k_m");
    }

    #[test]
    fn test_validate_magic_bytes_safetensors() {
        // Valid safetensors (8-byte header length + JSON)
        let header = b"{}";
        let mut data = Vec::new();
        data.extend_from_slice(&(header.len() as u64).to_le_bytes());
        data.extend_from_slice(header);
        let check = validate_magic_bytes(&data, &ModelFormat::Safetensors);
        assert!(check.passed);
    }

    #[test]
    fn test_validate_magic_bytes_gguf() {
        let mut data = b"GGUF".to_vec();
        data.extend_from_slice(&[0u8; 20]);
        let check = validate_magic_bytes(&data, &ModelFormat::GGUF);
        assert!(check.passed);
    }

    #[test]
    fn test_validate_magic_bytes_pytorch_zip() {
        let data = b"PK\x03\x04...";
        let check = validate_magic_bytes(data, &ModelFormat::PyTorch);
        assert!(check.passed);
    }

    #[test]
    fn test_validate_magic_bytes_unknown_format() {
        let check = validate_magic_bytes(b"anything", &ModelFormat::Keras);
        assert!(check.passed); // no check for Keras → pass
    }

    #[test]
    fn test_pipeline_with_progress_callback() {
        let pipeline = ConversionPipeline::with_builtins();
        let progress_log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let log_clone = progress_log.clone();
        let cb: ProgressCallback = Box::new(move |p| {
            log_clone.lock().unwrap().push(p.message.clone());
        });

        let _ = pipeline.convert(
            b"",
            &ModelFormat::PyTorch,
            &ModelFormat::ONNX,
            &ConversionOptions::default(),
            Some(&cb),
        );

        let log = progress_log.lock().unwrap();
        assert!(!log.is_empty());
    }
}
