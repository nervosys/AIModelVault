//! Model format detection and conversion registry

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{Result, VaultError};

/// Supported AI model formats
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelFormat {
    // LLM-centric formats
    /// Safetensors (.safetensors) - HuggingFace default for Transformers
    Safetensors,
    /// GGUF (.gguf) - Quantized LLM format (llama.cpp, LM Studio, Ollama)
    GGUF,
    /// PyTorch weights (.pt, .pth, .bin) - Classic state_dict files
    PyTorch,
    /// TensorRT Engine (.plan) - NVIDIA compiled engines
    TensorRT,
    /// ONNX (.onnx) - Interchange/serving format
    ONNX,
    /// MLX (.npz) - Apple Silicon optimized format
    MLX,
    /// Core ML (.mlmodel) - iOS/macOS on-device inference
    CoreML,
    /// TorchScript (.pt traced/scripted) - PyTorch serialization
    TorchScript,
    /// TensorFlow Lite (.tflite) - Mobile/edge deployment
    TFLite,

    // General DL formats
    /// TensorFlow SavedModel - TensorFlow serving format
    TensorFlow,
    /// Keras (.h5, .keras) - Keras model format
    Keras,
    /// OpenVINO IR (.xml + .bin) - Intel optimization format
    OpenVINO,
    /// TVM compiled artifacts
    TVM,
    /// NCNN (.param + .bin) - Mobile-optimized format
    NCNN,
    /// MNN (.mnn) - Mobile Neural Network format
    MNN,
    /// RKNN (.rknn) - Rockchip NPU format
    RKNN,

    // Legacy formats
    /// Caffe (.caffemodel) - Legacy Caffe format
    Caffe,
    /// MXNet (.params) - Apache MXNet format
    MXNet,
    /// Darknet (.weights) - YOLO/Darknet format
    Darknet,

    // Data formats
    /// HDF5 (.h5, .hdf5) - Hierarchical data format
    HDF5,
    /// Pickle (.pkl) - Python pickle format
    Pickle,
    /// NumPy (.npy, .npz) - NumPy array format
    NumPy,

    /// Custom/Unknown format
    Custom(String),
}

impl ModelFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "safetensors" => ModelFormat::Safetensors,
            "gguf" => ModelFormat::GGUF,
            "pt" | "pth" | "bin" => ModelFormat::PyTorch,
            "plan" => ModelFormat::TensorRT,
            "onnx" => ModelFormat::ONNX,
            "mlx" => ModelFormat::MLX,
            "mlmodel" | "mlmodelc" => ModelFormat::CoreML,
            "tflite" => ModelFormat::TFLite,
            "pb" => ModelFormat::TensorFlow,
            "h5" | "keras" => ModelFormat::Keras,
            "xml" => ModelFormat::OpenVINO,
            "param" => ModelFormat::NCNN,
            "mnn" => ModelFormat::MNN,
            "rknn" => ModelFormat::RKNN,
            "caffemodel" => ModelFormat::Caffe,
            "params" => ModelFormat::MXNet,
            "weights" => ModelFormat::Darknet,
            "hdf5" => ModelFormat::HDF5,
            "pkl" | "pickle" => ModelFormat::Pickle,
            "npy" | "npz" => ModelFormat::NumPy,
            _ => ModelFormat::Custom(ext.to_string()),
        }
    }

    /// Get file extension for format
    pub fn extension(&self) -> &str {
        match self {
            ModelFormat::Safetensors => "safetensors",
            ModelFormat::GGUF => "gguf",
            ModelFormat::PyTorch => "pt",
            ModelFormat::TensorRT => "plan",
            ModelFormat::ONNX => "onnx",
            ModelFormat::MLX => "npz",
            ModelFormat::CoreML => "mlmodel",
            ModelFormat::TorchScript => "pt",
            ModelFormat::TFLite => "tflite",
            ModelFormat::TensorFlow => "pb",
            ModelFormat::Keras => "h5",
            ModelFormat::OpenVINO => "xml",
            ModelFormat::TVM => "so",
            ModelFormat::NCNN => "param",
            ModelFormat::MNN => "mnn",
            ModelFormat::RKNN => "rknn",
            ModelFormat::Caffe => "caffemodel",
            ModelFormat::MXNet => "params",
            ModelFormat::Darknet => "weights",
            ModelFormat::HDF5 => "h5",
            ModelFormat::Pickle => "pkl",
            ModelFormat::NumPy => "npy",
            ModelFormat::Custom(ext) => ext,
        }
    }

    /// Every non-`Custom` variant, for reverse lookups.
    const ALL: [ModelFormat; 22] = [
        ModelFormat::Safetensors,
        ModelFormat::GGUF,
        ModelFormat::PyTorch,
        ModelFormat::TensorRT,
        ModelFormat::ONNX,
        ModelFormat::MLX,
        ModelFormat::CoreML,
        ModelFormat::TorchScript,
        ModelFormat::TFLite,
        ModelFormat::TensorFlow,
        ModelFormat::Keras,
        ModelFormat::OpenVINO,
        ModelFormat::TVM,
        ModelFormat::NCNN,
        ModelFormat::MNN,
        ModelFormat::RKNN,
        ModelFormat::Caffe,
        ModelFormat::MXNet,
        ModelFormat::Darknet,
        ModelFormat::HDF5,
        ModelFormat::Pickle,
        ModelFormat::NumPy,
    ];

    /// Parse a format from its display [`name`](Self::name), case- and
    /// space-insensitively (`"Core ML"`, `"coreml"`, `"CORE ML"` all match).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let norm = |s: &str| s.to_lowercase().replace([' ', '-', '_'], "");
        let target = norm(name);
        Self::ALL.into_iter().find(|f| norm(f.name()) == target)
    }

    /// Parse a format string that may be either a display name or a file
    /// extension.
    ///
    /// Version records persist `format.name()` (e.g. `"PyTorch"`), which
    /// [`from_extension`](Self::from_extension) does not recognise — it would
    /// silently yield `Custom("pytorch")` and break conversion-path lookup and
    /// tensor-level diffing. Use this whenever the input came from storage.
    #[must_use]
    pub fn from_stored(value: &str) -> Self {
        Self::from_name(value).unwrap_or_else(|| Self::from_extension(value))
    }

    /// Get format name
    pub fn name(&self) -> &str {
        match self {
            ModelFormat::Safetensors => "Safetensors",
            ModelFormat::GGUF => "GGUF",
            ModelFormat::PyTorch => "PyTorch",
            ModelFormat::TensorRT => "TensorRT",
            ModelFormat::ONNX => "ONNX",
            ModelFormat::MLX => "MLX",
            ModelFormat::CoreML => "Core ML",
            ModelFormat::TorchScript => "TorchScript",
            ModelFormat::TFLite => "TensorFlow Lite",
            ModelFormat::TensorFlow => "TensorFlow",
            ModelFormat::Keras => "Keras",
            ModelFormat::OpenVINO => "OpenVINO",
            ModelFormat::TVM => "TVM",
            ModelFormat::NCNN => "NCNN",
            ModelFormat::MNN => "MNN",
            ModelFormat::RKNN => "RKNN",
            ModelFormat::Caffe => "Caffe",
            ModelFormat::MXNet => "MXNet",
            ModelFormat::Darknet => "Darknet",
            ModelFormat::HDF5 => "HDF5",
            ModelFormat::Pickle => "Pickle",
            ModelFormat::NumPy => "NumPy",
            ModelFormat::Custom(name) => name,
        }
    }

    /// Format label safe to report in telemetry.
    ///
    /// [`name`](Self::name) returns the caller's own string for
    /// [`ModelFormat::Custom`], which is free-form user input — a model name,
    /// a path fragment, anything they typed. Reporting it would break the
    /// guarantee that telemetry carries no user-supplied text, so this
    /// collapses every `Custom` to the literal `"custom"` and is otherwise a
    /// fixed set of `&'static str`.
    #[must_use]
    pub fn telemetry_name(&self) -> &'static str {
        match self {
            ModelFormat::Safetensors => "safetensors",
            ModelFormat::GGUF => "gguf",
            ModelFormat::PyTorch => "pytorch",
            ModelFormat::TensorRT => "tensorrt",
            ModelFormat::ONNX => "onnx",
            ModelFormat::MLX => "mlx",
            ModelFormat::CoreML => "coreml",
            ModelFormat::TorchScript => "torchscript",
            ModelFormat::TFLite => "tflite",
            ModelFormat::TensorFlow => "tensorflow",
            ModelFormat::Keras => "keras",
            ModelFormat::OpenVINO => "openvino",
            ModelFormat::TVM => "tvm",
            ModelFormat::NCNN => "ncnn",
            ModelFormat::MNN => "mnn",
            ModelFormat::RKNN => "rknn",
            ModelFormat::Caffe => "caffe",
            ModelFormat::MXNet => "mxnet",
            ModelFormat::Darknet => "darknet",
            ModelFormat::HDF5 => "hdf5",
            ModelFormat::Pickle => "pickle",
            ModelFormat::NumPy => "numpy",
            // Deliberately NOT `name`: that is whatever the user typed.
            ModelFormat::Custom(_) => "custom",
        }
    }
}

impl std::fmt::Display for ModelFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub format: ModelFormat,
    pub description: Option<String>,
    pub framework: Option<String>,
    pub task: Option<String>,
    pub architecture: Option<String>,
    pub parameters: Option<u64>,
    pub custom_fields: HashMap<String, String>,
}

impl ModelMetadata {
    /// Create new model metadata
    pub fn new(name: String, format: ModelFormat) -> Self {
        Self {
            name,
            format,
            description: None,
            framework: None,
            task: None,
            architecture: None,
            parameters: None,
            custom_fields: HashMap::new(),
        }
    }

    /// Builder pattern for optional fields
    pub fn with_description(mut self, desc: String) -> Self {
        self.description = Some(desc);
        self
    }

    /// Set the framework used to train the model.
    pub fn with_framework(mut self, framework: String) -> Self {
        self.framework = Some(framework);
        self
    }

    /// Set the model's target task (e.g., classification, generation).
    pub fn with_task(mut self, task: String) -> Self {
        self.task = Some(task);
        self
    }

    /// Set the model architecture (e.g., Transformer, CNN).
    pub fn with_architecture(mut self, arch: String) -> Self {
        self.architecture = Some(arch);
        self
    }

    /// Set the model parameter count.
    pub fn with_parameters(mut self, params: u64) -> Self {
        self.parameters = Some(params);
        self
    }

    /// Add a custom metadata field as a key-value pair.
    pub fn add_custom_field(mut self, key: String, value: String) -> Self {
        self.custom_fields.insert(key, value);
        self
    }
}

/// Type alias for format conversion function
type ConversionFn = fn(&[u8]) -> Result<Vec<u8>>;

/// Format converter registry
pub struct FormatConverter {
    converters: HashMap<(ModelFormat, ModelFormat), ConversionFn>,
}

impl FormatConverter {
    /// Create new format converter registry
    pub fn new() -> Self {
        Self {
            converters: HashMap::new(),
        }
    }

    /// Register a format converter
    pub fn register(
        &mut self,
        from: ModelFormat,
        to: ModelFormat,
        converter: fn(&[u8]) -> Result<Vec<u8>>,
    ) {
        self.converters.insert((from, to), converter);
    }

    /// Check if conversion is supported
    pub fn can_convert(&self, from: ModelFormat, to: ModelFormat) -> bool {
        self.converters.contains_key(&(from, to))
    }

    /// Convert between formats
    pub fn convert(&self, data: &[u8], from: ModelFormat, to: ModelFormat) -> Result<Vec<u8>> {
        if from == to {
            return Ok(data.to_vec());
        }

        let converter = self
            .converters
            .get(&(from.clone(), to.clone()))
            .ok_or_else(|| {
                VaultError::ConversionError(format!(
                    "No converter available for {} -> {}",
                    from.name(),
                    to.name()
                ))
            })?;

        converter(data)
    }
}

impl Default for FormatConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(
            ModelFormat::from_extension("safetensors"),
            ModelFormat::Safetensors
        );
        assert_eq!(ModelFormat::from_extension("gguf"), ModelFormat::GGUF);
        assert_eq!(ModelFormat::from_extension("pt"), ModelFormat::PyTorch);
        assert_eq!(ModelFormat::from_extension("onnx"), ModelFormat::ONNX);
        assert_eq!(ModelFormat::from_extension("tflite"), ModelFormat::TFLite);
        assert_eq!(ModelFormat::from_extension("mlmodel"), ModelFormat::CoreML);
        assert_eq!(ModelFormat::from_extension("plan"), ModelFormat::TensorRT);
    }

    #[test]
    fn test_format_extension() {
        assert_eq!(ModelFormat::Safetensors.extension(), "safetensors");
        assert_eq!(ModelFormat::GGUF.extension(), "gguf");
        assert_eq!(ModelFormat::PyTorch.extension(), "pt");
        assert_eq!(ModelFormat::TensorRT.extension(), "plan");
        assert_eq!(ModelFormat::ONNX.extension(), "onnx");
        assert_eq!(ModelFormat::TFLite.extension(), "tflite");
        assert_eq!(ModelFormat::CoreML.extension(), "mlmodel");
    }

    #[test]
    fn test_format_names() {
        assert_eq!(ModelFormat::Safetensors.name(), "Safetensors");
        assert_eq!(ModelFormat::GGUF.name(), "GGUF");
        assert_eq!(ModelFormat::TensorRT.name(), "TensorRT");
        assert_eq!(ModelFormat::CoreML.name(), "Core ML");
        assert_eq!(ModelFormat::TFLite.name(), "TensorFlow Lite");
    }

    #[test]
    fn test_format_extension_all_variants() {
        // Cover lines 111, 114, 116, 119 — TorchScript, TensorFlow, Keras, TVM
        assert_eq!(ModelFormat::TorchScript.extension(), "pt");
        assert_eq!(ModelFormat::TensorFlow.extension(), "pb");
        assert_eq!(ModelFormat::Keras.extension(), "h5");
        assert_eq!(ModelFormat::TVM.extension(), "so");
        assert_eq!(ModelFormat::OpenVINO.extension(), "xml");
        assert_eq!(ModelFormat::NCNN.extension(), "param");
        assert_eq!(ModelFormat::MNN.extension(), "mnn");
        assert_eq!(ModelFormat::RKNN.extension(), "rknn");
        assert_eq!(ModelFormat::HDF5.extension(), "h5");
        assert_eq!(ModelFormat::NumPy.extension(), "npy");
        assert_eq!(ModelFormat::Custom("custom".into()).extension(), "custom");
    }

    #[test]
    fn test_format_name_all_variants() {
        // Cover line 146 — TorchScript name
        assert_eq!(ModelFormat::TorchScript.name(), "TorchScript");
        assert_eq!(ModelFormat::TensorFlow.name(), "TensorFlow");
        assert_eq!(ModelFormat::Keras.name(), "Keras");
        assert_eq!(ModelFormat::TVM.name(), "TVM");
        assert_eq!(ModelFormat::Custom("custom".into()).name(), "custom");
    }

    #[test]
    fn test_model_metadata_new() {
        // Cover line 175 — ModelMetadata::new
        let meta = ModelMetadata::new("model".to_string(), ModelFormat::PyTorch);
        assert_eq!(meta.name, "model");
        assert!(meta.description.is_none());
    }

    #[test]
    fn test_format_converter_register_and_convert() {
        // Cover line 258 — converter(data)
        let mut fc = FormatConverter::new();
        fc.register(ModelFormat::PyTorch, ModelFormat::ONNX, |data| {
            Ok(data.to_vec())
        });
        assert!(fc.can_convert(ModelFormat::PyTorch, ModelFormat::ONNX));
        let result = fc
            .convert(b"test", ModelFormat::PyTorch, ModelFormat::ONNX)
            .unwrap();
        assert_eq!(result, b"test");
    }

    #[test]
    fn test_format_converter_same_format() {
        let fc = FormatConverter::new();
        let result = fc
            .convert(b"data", ModelFormat::PyTorch, ModelFormat::PyTorch)
            .unwrap();
        assert_eq!(result, b"data");
    }

    #[test]
    fn test_format_display() {
        let s = format!("{}", ModelFormat::Safetensors);
        assert_eq!(s, "Safetensors");
    }

    #[test]
    fn test_from_extension_all_variants() {
        // Cover all from_extension branches
        assert_eq!(ModelFormat::from_extension("mlx"), ModelFormat::MLX);
        assert_eq!(ModelFormat::from_extension("pb"), ModelFormat::TensorFlow);
        assert_eq!(ModelFormat::from_extension("h5"), ModelFormat::Keras);
        assert_eq!(ModelFormat::from_extension("keras"), ModelFormat::Keras);
        assert_eq!(ModelFormat::from_extension("xml"), ModelFormat::OpenVINO);
        assert_eq!(ModelFormat::from_extension("param"), ModelFormat::NCNN);
        assert_eq!(ModelFormat::from_extension("mnn"), ModelFormat::MNN);
        assert_eq!(ModelFormat::from_extension("rknn"), ModelFormat::RKNN);
        assert_eq!(
            ModelFormat::from_extension("caffemodel"),
            ModelFormat::Caffe
        );
        assert_eq!(ModelFormat::from_extension("params"), ModelFormat::MXNet);
        assert_eq!(ModelFormat::from_extension("weights"), ModelFormat::Darknet);
        assert_eq!(ModelFormat::from_extension("hdf5"), ModelFormat::HDF5);
        assert_eq!(ModelFormat::from_extension("pkl"), ModelFormat::Pickle);
        assert_eq!(ModelFormat::from_extension("pickle"), ModelFormat::Pickle);
        assert_eq!(ModelFormat::from_extension("npy"), ModelFormat::NumPy);
        assert_eq!(ModelFormat::from_extension("npz"), ModelFormat::NumPy);
        assert_eq!(ModelFormat::from_extension("pth"), ModelFormat::PyTorch);
        assert_eq!(ModelFormat::from_extension("bin"), ModelFormat::PyTorch);
        assert_eq!(ModelFormat::from_extension("mlmodelc"), ModelFormat::CoreML);
        assert_eq!(
            ModelFormat::from_extension("xyz"),
            ModelFormat::Custom("xyz".to_string())
        );
    }

    #[test]
    fn test_name_all_remaining_variants() {
        assert_eq!(ModelFormat::NCNN.name(), "NCNN");
        assert_eq!(ModelFormat::MNN.name(), "MNN");
        assert_eq!(ModelFormat::RKNN.name(), "RKNN");
        assert_eq!(ModelFormat::Caffe.name(), "Caffe");
        assert_eq!(ModelFormat::MXNet.name(), "MXNet");
        assert_eq!(ModelFormat::Darknet.name(), "Darknet");
        assert_eq!(ModelFormat::HDF5.name(), "HDF5");
        assert_eq!(ModelFormat::Pickle.name(), "Pickle");
        assert_eq!(ModelFormat::NumPy.name(), "NumPy");
        assert_eq!(ModelFormat::MLX.name(), "MLX");
        assert_eq!(ModelFormat::OpenVINO.name(), "OpenVINO");
    }

    #[test]
    fn test_model_metadata_builder_chain() {
        let meta = ModelMetadata::new("m".to_string(), ModelFormat::ONNX)
            .with_description("desc".to_string())
            .with_framework("pytorch".to_string())
            .with_task("classification".to_string())
            .with_architecture("ResNet".to_string())
            .with_parameters(1_000_000)
            .add_custom_field("license".to_string(), "MIT".to_string());

        assert_eq!(meta.description, Some("desc".to_string()));
        assert_eq!(meta.framework, Some("pytorch".to_string()));
        assert_eq!(meta.task, Some("classification".to_string()));
        assert_eq!(meta.architecture, Some("ResNet".to_string()));
        assert_eq!(meta.parameters, Some(1_000_000));
        assert_eq!(meta.custom_fields.get("license"), Some(&"MIT".to_string()));
    }

    #[test]
    fn test_format_converter_missing_conversion() {
        let fc = FormatConverter::new();
        let result = fc.convert(b"data", ModelFormat::PyTorch, ModelFormat::ONNX);
        assert!(result.is_err());
        assert!(!fc.can_convert(ModelFormat::PyTorch, ModelFormat::ONNX));
    }

    #[test]
    fn test_format_converter_default() {
        let fc = FormatConverter::default();
        assert!(!fc.can_convert(ModelFormat::PyTorch, ModelFormat::ONNX));
    }

    #[test]
    fn test_extension_all_remaining() {
        assert_eq!(ModelFormat::Caffe.extension(), "caffemodel");
        assert_eq!(ModelFormat::MXNet.extension(), "params");
        assert_eq!(ModelFormat::Darknet.extension(), "weights");
        assert_eq!(ModelFormat::Pickle.extension(), "pkl");
        assert_eq!(ModelFormat::MLX.extension(), "npz");
    }

    #[test]
    fn test_from_name_roundtrip_for_every_variant() {
        // Version records persist `name()`, so every name must parse back to the
        // same variant — otherwise stored formats silently degrade to Custom.
        for fmt in ModelFormat::ALL {
            assert_eq!(
                ModelFormat::from_name(fmt.name()),
                Some(fmt.clone()),
                "name() -> from_name() must round-trip for {:?}",
                fmt
            );
        }
    }

    #[test]
    fn test_from_name_is_case_and_space_insensitive() {
        assert_eq!(ModelFormat::from_name("core ml"), Some(ModelFormat::CoreML));
        assert_eq!(ModelFormat::from_name("CoreML"), Some(ModelFormat::CoreML));
        assert_eq!(ModelFormat::from_name("CORE-ML"), Some(ModelFormat::CoreML));
        assert_eq!(
            ModelFormat::from_name("tensorflow lite"),
            Some(ModelFormat::TFLite)
        );
        assert_eq!(ModelFormat::from_name("not a format"), None);
    }

    #[test]
    fn test_from_stored_accepts_names_and_extensions() {
        // The shape actually written into version records.
        assert_eq!(ModelFormat::from_stored("PyTorch"), ModelFormat::PyTorch);
        assert_eq!(
            ModelFormat::from_stored("Safetensors"),
            ModelFormat::Safetensors
        );
        assert_eq!(ModelFormat::from_stored("ONNX"), ModelFormat::ONNX);
        // Still accepts plain extensions.
        assert_eq!(ModelFormat::from_stored("pt"), ModelFormat::PyTorch);
        assert_eq!(ModelFormat::from_stored("onnx"), ModelFormat::ONNX);
    }

    #[test]
    fn test_from_extension_alone_cannot_parse_stored_names() {
        // Documents the trap `from_stored` exists to avoid: this is what the
        // convert and diff paths used to do with a stored format string.
        assert_ne!(ModelFormat::from_extension("PyTorch"), ModelFormat::PyTorch);
    }

    /// `name()` returns the caller's own string for `Custom`, so it must never
    /// be what telemetry reports. This is the guard on that: whatever a user
    /// types as `--format`, the telemetry label stays a fixed literal.
    #[test]
    fn test_telemetry_name_never_echoes_a_custom_format_string() {
        for hostile in [
            "/home/alice/models/customer-data.bin",
            "s3://acme-private/secret-model",
            "Bearer abc123",
            "",
        ] {
            let fmt = ModelFormat::Custom(hostile.to_string());
            assert_eq!(
                fmt.telemetry_name(),
                "custom",
                "telemetry_name leaked a Custom payload"
            );
            assert_eq!(fmt.name(), hostile, "name() should still be verbatim");
        }
    }

    /// Every non-Custom variant must map to a distinct lowercase literal, or
    /// the collector groups unrelated formats together.
    #[test]
    fn test_telemetry_names_are_distinct_and_lowercase() {
        let all = [
            ModelFormat::Safetensors,
            ModelFormat::GGUF,
            ModelFormat::PyTorch,
            ModelFormat::TensorRT,
            ModelFormat::ONNX,
            ModelFormat::MLX,
            ModelFormat::CoreML,
            ModelFormat::TorchScript,
            ModelFormat::TFLite,
            ModelFormat::TensorFlow,
            ModelFormat::Keras,
            ModelFormat::OpenVINO,
            ModelFormat::TVM,
            ModelFormat::NCNN,
            ModelFormat::MNN,
            ModelFormat::RKNN,
            ModelFormat::Caffe,
            ModelFormat::MXNet,
            ModelFormat::Darknet,
            ModelFormat::HDF5,
            ModelFormat::Pickle,
            ModelFormat::NumPy,
        ];
        let mut seen = std::collections::HashSet::new();
        for f in &all {
            let label = f.telemetry_name();
            assert!(
                label
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
                "{label:?} is not a plain lowercase token"
            );
            assert!(seen.insert(label), "duplicate telemetry label {label:?}");
        }
        assert_eq!(seen.len(), all.len());
    }
}
