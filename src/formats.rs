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

    /// Get format name
    pub fn name(&self) -> String {
        match self {
            ModelFormat::Safetensors => "Safetensors".to_string(),
            ModelFormat::GGUF => "GGUF".to_string(),
            ModelFormat::PyTorch => "PyTorch".to_string(),
            ModelFormat::TensorRT => "TensorRT".to_string(),
            ModelFormat::ONNX => "ONNX".to_string(),
            ModelFormat::MLX => "MLX".to_string(),
            ModelFormat::CoreML => "Core ML".to_string(),
            ModelFormat::TorchScript => "TorchScript".to_string(),
            ModelFormat::TFLite => "TensorFlow Lite".to_string(),
            ModelFormat::TensorFlow => "TensorFlow".to_string(),
            ModelFormat::Keras => "Keras".to_string(),
            ModelFormat::OpenVINO => "OpenVINO".to_string(),
            ModelFormat::TVM => "TVM".to_string(),
            ModelFormat::NCNN => "NCNN".to_string(),
            ModelFormat::MNN => "MNN".to_string(),
            ModelFormat::RKNN => "RKNN".to_string(),
            ModelFormat::Caffe => "Caffe".to_string(),
            ModelFormat::MXNet => "MXNet".to_string(),
            ModelFormat::Darknet => "Darknet".to_string(),
            ModelFormat::HDF5 => "HDF5".to_string(),
            ModelFormat::Pickle => "Pickle".to_string(),
            ModelFormat::NumPy => "NumPy".to_string(),
            ModelFormat::Custom(name) => name.clone(),
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
}
