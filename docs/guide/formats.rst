Format Detection Guide
======================

AI Model Vault recognizes 23+ AI model formats out of the box, enabling automatic
format detection, metadata tagging, and (in v0.4.0+) format conversion.


Auto-Detection
--------------

Format detection works by file extension:

.. code-block:: python

   from aimodelvault import ModelFormat

   fmt = ModelFormat.detect("llama-7b.gguf")
   print(fmt.name)       # "GGUF"
   print(fmt.extension)  # "gguf"

   # Works with full paths
   fmt2 = ModelFormat.detect("/models/bert-base.safetensors")
   print(fmt2.name)  # "Safetensors"


Custom Formats
--------------

Unrecognized extensions are handled as custom formats:

.. code-block:: python

   fmt = ModelFormat("my-custom-format")
   print(fmt.name)  # "my-custom-format"

   fmt2 = ModelFormat.detect("model.xyz")
   print(fmt2.name)  # "xyz"


Format Categories
-----------------

**LLM-centric:**
  Safetensors, GGUF, PyTorch, TorchScript, MLX

**Deployment:**
  ONNX, TensorRT, CoreML, TFLite, OpenVINO

**General deep learning:**
  TensorFlow, Keras, TVM, NCNN, MNN, RKNN

**Legacy:**
  Caffe, MXNet, Darknet

**Data formats:**
  HDF5, Pickle, NumPy


Using with Metadata
-------------------

.. code-block:: python

   from aimodelvault import ModelMetadata

   meta = ModelMetadata(
       "my-model", "safetensors",
       description="A transformer model",
       framework="PyTorch",
       task="text-generation",
       architecture="LlamaForCausalLM",
       parameters=7_000_000_000,
   )
   meta.add_custom_field("quantization", "Q4_K_M")
   meta.add_custom_field("context_length", "4096")
