"""
Model format registry for AI Model Vault.

Enumerates supported model formats and provides detection/conversion guidance.
Mirrors the Rust `ModelFormat` enum in src/formats.rs.
"""

from enum import Enum
from pathlib import Path
from typing import Optional


class ModelFormat(Enum):
    """
    Supported AI model formats.

    Each variant corresponds to a file format that AI Model Vault
    can store, retrieve, and (where applicable) convert between.
    """

    # PyTorch formats
    PYTORCH = "pytorch"
    PYTORCH_SCRIPT = "pytorch_script"

    # TensorFlow / Keras
    TENSORFLOW = "tensorflow"
    TENSORFLOW_LITE = "tensorflow_lite"
    KERAS = "keras"

    # ONNX
    ONNX = "onnx"

    # SafeTensors (Hugging Face)
    SAFETENSORS = "safetensors"

    # Hugging Face Transformers
    HUGGINGFACE = "huggingface"

    # JAX / Flax
    JAX = "jax"
    FLAX = "flax"

    # Apple Core ML
    COREML = "coreml"

    # NVIDIA TensorRT
    TENSORRT = "tensorrt"

    # OpenVINO
    OPENVINO = "openvino"

    # GGML / GGUF (llama.cpp)
    GGML = "ggml"
    GGUF = "gguf"

    # MXNet
    MXNET = "mxnet"

    # PaddlePaddle
    PADDLE = "paddle"

    # Caffe
    CAFFE = "caffe"

    # Scikit-learn (pickle/joblib)
    SKLEARN = "sklearn"

    # XGBoost
    XGBOOST = "xgboost"

    # LightGBM
    LIGHTGBM = "lightgbm"

    # Raw binary / unknown
    RAW = "raw"
    UNKNOWN = "unknown"

    @classmethod
    def detect(cls, path: str) -> "ModelFormat":
        """
        Detect model format from file extension.

        Args:
            path: Path to the model file.

        Returns:
            Detected ModelFormat variant.
        """
        ext = Path(path).suffix.lower()
        extension_map = {
            ".pt": cls.PYTORCH,
            ".pth": cls.PYTORCH,
            ".bin": cls.PYTORCH,  # Common for HF models
            ".torchscript": cls.PYTORCH_SCRIPT,
            ".pb": cls.TENSORFLOW,
            ".tflite": cls.TENSORFLOW_LITE,
            ".h5": cls.KERAS,
            ".keras": cls.KERAS,
            ".onnx": cls.ONNX,
            ".safetensors": cls.SAFETENSORS,
            ".ggml": cls.GGML,
            ".gguf": cls.GGUF,
            ".mlmodel": cls.COREML,
            ".mlpackage": cls.COREML,
            ".trt": cls.TENSORRT,
            ".engine": cls.TENSORRT,
            ".xml": cls.OPENVINO,
            ".pdparams": cls.PADDLE,
            ".caffemodel": cls.CAFFE,
            ".pkl": cls.SKLEARN,
            ".joblib": cls.SKLEARN,
            ".xgb": cls.XGBOOST,
            ".lgb": cls.LIGHTGBM,
        }
        return extension_map.get(ext, cls.UNKNOWN)

    @property
    def file_extensions(self) -> list:
        """Return common file extensions for this format."""
        ext_map = {
            self.PYTORCH: [".pt", ".pth", ".bin"],
            self.PYTORCH_SCRIPT: [".torchscript"],
            self.TENSORFLOW: [".pb", ".savedmodel"],
            self.TENSORFLOW_LITE: [".tflite"],
            self.KERAS: [".h5", ".keras"],
            self.ONNX: [".onnx"],
            self.SAFETENSORS: [".safetensors"],
            self.GGML: [".ggml"],
            self.GGUF: [".gguf"],
            self.COREML: [".mlmodel", ".mlpackage"],
            self.TENSORRT: [".trt", ".engine"],
            self.OPENVINO: [".xml"],
            self.PADDLE: [".pdparams"],
            self.CAFFE: [".caffemodel"],
            self.SKLEARN: [".pkl", ".joblib"],
            self.XGBOOST: [".xgb"],
            self.LIGHTGBM: [".lgb"],
        }
        return ext_map.get(self, [])

    def __str__(self) -> str:
        return self.value
