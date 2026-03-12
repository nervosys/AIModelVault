Format Detection API
====================

.. module:: aimodelvault
   :noindex:

ModelFormat
-----------

.. class:: ModelFormat(name)

   AI model format identifier. Supports 23+ formats including PyTorch,
   SafeTensors, ONNX, GGUF, TensorRT, CoreML, TFLite, and more.

   :param name: Format name string (e.g. ``"safetensors"``, ``"pytorch"``).
   :type name: str

   .. staticmethod:: detect(filename)

      Auto-detect format from a filename or path.

      :param filename: Filename with extension (e.g. ``"model.safetensors"``).
      :type filename: str
      :returns: Detected format.
      :rtype: ModelFormat

      .. code-block:: python

         fmt = ModelFormat.detect("weights.gguf")
         print(fmt.name)  # "GGUF"

   .. attribute:: name
      :type: str

      Human-readable format name (e.g. ``"Safetensors"``, ``"PyTorch"``).

   .. attribute:: extension
      :type: str

      Canonical file extension (e.g. ``"safetensors"``, ``"pt"``).

   **Supported Formats:**

   .. list-table::
      :header-rows: 1
      :widths: 25 20 55

      * - Format
        - Extension
        - Description
      * - Safetensors
        - ``.safetensors``
        - HuggingFace default for Transformers
      * - GGUF
        - ``.gguf``
        - Quantized LLM format (llama.cpp, Ollama)
      * - PyTorch
        - ``.pt``, ``.pth``
        - Classic state_dict files
      * - ONNX
        - ``.onnx``
        - Interchange/serving format
      * - TensorRT
        - ``.plan``
        - NVIDIA compiled engines
      * - CoreML
        - ``.mlmodel``
        - iOS/macOS on-device inference
      * - TFLite
        - ``.tflite``
        - Mobile/edge deployment
      * - TensorFlow
        - ``.pb``
        - TensorFlow SavedModel
      * - Keras
        - ``.h5``, ``.keras``
        - Keras model format
      * - OpenVINO
        - ``.xml``
        - Intel optimization format
      * - MLX
        - ``.npz``
        - Apple Silicon optimized
      * - HDF5
        - ``.hdf5``
        - Hierarchical data format
      * - NumPy
        - ``.npy``, ``.npz``
        - NumPy arrays


ModelMetadata
-------------

.. class:: ModelMetadata(name, format, *, description=None, framework=None, task=None, architecture=None, parameters=None)

   Metadata attached to a stored model version.

   :param name: Model name.
   :type name: str
   :param format: Format string (e.g. ``"safetensors"``).
   :type format: str
   :param description: Free-text description.
   :type description: str or None
   :param framework: Training framework (e.g. ``"PyTorch"``, ``"TensorFlow"``).
   :type framework: str or None
   :param task: Target task (e.g. ``"text-generation"``, ``"classification"``).
   :type task: str or None
   :param architecture: Model architecture (e.g. ``"Transformer"``, ``"CNN"``).
   :type architecture: str or None
   :param parameters: Parameter count.
   :type parameters: int or None

   .. method:: add_custom_field(key, value)

      Add a custom key-value metadata field.

      :param key: Field name.
      :type key: str
      :param value: Field value.
      :type value: str


ModelVersion
------------

.. class:: ModelVersion

   Read-only snapshot of a model version. Returned by
   :meth:`Vault.store_model` and :meth:`Vault.list_versions`.

   .. attribute:: version
      :type: int

      Version number (1-based, auto-incrementing).

   .. attribute:: checkpoint_id
      :type: str

      Unique checkpoint identifier (UUID).

   .. attribute:: timestamp
      :type: str

      ISO 8601 timestamp of when the version was stored.

   .. attribute:: parent_version
      :type: int or None

      Parent version number (for lineage tracking).

   .. attribute:: format
      :type: str

      Model format string.

   .. attribute:: size_bytes
      :type: int

      Original (uncompressed) model size in bytes.

   .. attribute:: compressed_size_bytes
      :type: int

      Compressed size in the vault.

   .. attribute:: checksum_sha256
      :type: str

      SHA-256 hex digest of the original model data.

   .. attribute:: metadata
      :type: dict[str, str]

      Custom metadata key-value pairs.
