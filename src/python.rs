//! Native Python bindings via PyO3
//!
//! Provides zero-copy access to the Rust vault, crypto, format detection,
//! and model card APIs — replacing the legacy CLI-wrapper Python package.

use pyo3::exceptions::{PyIOError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use std::collections::HashMap;

use crate::config::VaultConfig;
use crate::error::VaultError;
use crate::formats::{ModelFormat, ModelMetadata};
use crate::model_card::{
    Evaluation, IntendedUse, Metric, ModelCard, ModelDetails, TrainingData,
};
use crate::vault::Vault;
use crate::version::ModelVersion;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Convert `VaultError` → Python exception.
fn to_py_err(e: VaultError) -> PyErr {
    match &e {
        VaultError::AuthenticationFailed => PyRuntimeError::new_err(e.to_string()),
        VaultError::SecurityViolation(_) => PyRuntimeError::new_err(e.to_string()),
        VaultError::IoError(_) => PyIOError::new_err(e.to_string()),
        VaultError::ModelNotFound(_) => PyValueError::new_err(e.to_string()),
        VaultError::VersionNotFound(_, _) => PyValueError::new_err(e.to_string()),
        VaultError::UnsupportedFormat(_) => PyValueError::new_err(e.to_string()),
        _ => PyRuntimeError::new_err(e.to_string()),
    }
}

fn parse_format(s: &str) -> PyResult<ModelFormat> {
    match s.to_lowercase().as_str() {
        "safetensors" => Ok(ModelFormat::Safetensors),
        "gguf" => Ok(ModelFormat::GGUF),
        "pytorch" | "pt" | "pth" => Ok(ModelFormat::PyTorch),
        "tensorrt" | "plan" => Ok(ModelFormat::TensorRT),
        "onnx" => Ok(ModelFormat::ONNX),
        "mlx" => Ok(ModelFormat::MLX),
        "coreml" | "mlmodel" => Ok(ModelFormat::CoreML),
        "torchscript" => Ok(ModelFormat::TorchScript),
        "tflite" => Ok(ModelFormat::TFLite),
        "tensorflow" | "tf" | "pb" => Ok(ModelFormat::TensorFlow),
        "keras" | "h5" => Ok(ModelFormat::Keras),
        "openvino" => Ok(ModelFormat::OpenVINO),
        "tvm" => Ok(ModelFormat::TVM),
        "ncnn" => Ok(ModelFormat::NCNN),
        "mnn" => Ok(ModelFormat::MNN),
        "rknn" => Ok(ModelFormat::RKNN),
        "caffe" => Ok(ModelFormat::Caffe),
        "mxnet" => Ok(ModelFormat::MXNet),
        "darknet" => Ok(ModelFormat::Darknet),
        "hdf5" => Ok(ModelFormat::HDF5),
        "pickle" | "pkl" => Ok(ModelFormat::Pickle),
        "numpy" | "npy" | "npz" => Ok(ModelFormat::NumPy),
        other => Ok(ModelFormat::Custom(other.to_string())),
    }
}

// ── PyModelFormat ────────────────────────────────────────────────────────────

/// AI model format identifier.
///
/// Use `detect("model.safetensors")` to auto-detect from filename.
#[pyclass(name = "ModelFormat")]
#[derive(Clone)]
struct PyModelFormat {
    inner: ModelFormat,
}

#[pymethods]
impl PyModelFormat {
    // -- constructors --------------------------------------------------------

    #[new]
    #[pyo3(signature = (name))]
    fn new(name: &str) -> PyResult<Self> {
        Ok(Self {
            inner: parse_format(name)?,
        })
    }

    /// Detect format from a filename/path.
    #[staticmethod]
    fn detect(filename: &str) -> PyResult<Self> {
        let ext = filename
            .rsplit('.')
            .next()
            .ok_or_else(|| PyValueError::new_err("No file extension"))?;
        Ok(Self {
            inner: ModelFormat::from_extension(ext),
        })
    }

    // -- properties ----------------------------------------------------------

    /// Human-readable format name.
    #[getter]
    fn name(&self) -> &str {
        self.inner.name()
    }

    /// Canonical file extension.
    #[getter]
    fn extension(&self) -> &str {
        self.inner.extension()
    }

    fn __repr__(&self) -> String {
        format!("ModelFormat('{}')", self.inner.name())
    }

    fn __str__(&self) -> String {
        self.inner.name().to_string()
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

// ── PyModelMetadata ──────────────────────────────────────────────────────────

/// Metadata attached to a stored model.
#[pyclass(name = "ModelMetadata")]
#[derive(Clone)]
struct PyModelMetadata {
    inner: ModelMetadata,
}

#[pymethods]
impl PyModelMetadata {
    #[new]
    #[pyo3(signature = (name, format, *, description=None, framework=None, task=None, architecture=None, parameters=None))]
    fn new(
        name: &str,
        format: &str,
        description: Option<String>,
        framework: Option<String>,
        task: Option<String>,
        architecture: Option<String>,
        parameters: Option<u64>,
    ) -> PyResult<Self> {
        let fmt = parse_format(format)?;
        let mut md = ModelMetadata::new(name.to_string(), fmt);
        if let Some(d) = description {
            md = md.with_description(d);
        }
        if let Some(f) = framework {
            md = md.with_framework(f);
        }
        if let Some(t) = task {
            md = md.with_task(t);
        }
        if let Some(a) = architecture {
            md = md.with_architecture(a);
        }
        if let Some(p) = parameters {
            md = md.with_parameters(p);
        }
        Ok(Self { inner: md })
    }

    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    #[getter]
    fn format(&self) -> PyModelFormat {
        PyModelFormat {
            inner: self.inner.format.clone(),
        }
    }

    #[getter]
    fn description(&self) -> Option<&str> {
        self.inner.description.as_deref()
    }

    #[getter]
    fn framework(&self) -> Option<&str> {
        self.inner.framework.as_deref()
    }

    #[getter]
    fn task(&self) -> Option<&str> {
        self.inner.task.as_deref()
    }

    #[getter]
    fn architecture(&self) -> Option<&str> {
        self.inner.architecture.as_deref()
    }

    #[getter]
    fn parameters(&self) -> Option<u64> {
        self.inner.parameters
    }

    /// Add a custom key/value field.
    fn add_custom_field(&mut self, key: String, value: String) {
        self.inner
            .custom_fields
            .insert(key, value);
    }

    fn __repr__(&self) -> String {
        format!(
            "ModelMetadata(name='{}', format='{}')",
            self.inner.name,
            self.inner.format.name()
        )
    }
}

// ── PyModelVersion ───────────────────────────────────────────────────────────

/// Read-only snapshot of a model version.
#[pyclass(name = "ModelVersion")]
#[derive(Clone)]
struct PyModelVersion {
    inner: ModelVersion,
}

#[pymethods]
impl PyModelVersion {
    #[getter]
    fn version(&self) -> u32 {
        self.inner.version
    }

    #[getter]
    fn checkpoint_id(&self) -> &str {
        &self.inner.checkpoint_id
    }

    #[getter]
    fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }

    #[getter]
    fn parent_version(&self) -> Option<u32> {
        self.inner.parent_version
    }

    #[getter]
    fn format(&self) -> &str {
        &self.inner.format
    }

    #[getter]
    fn size_bytes(&self) -> u64 {
        self.inner.size_bytes
    }

    #[getter]
    fn compressed_size_bytes(&self) -> u64 {
        self.inner.compressed_size_bytes
    }

    #[getter]
    fn checksum_sha256(&self) -> &str {
        &self.inner.checksum_sha256
    }

    #[getter]
    fn metadata(&self) -> HashMap<String, String> {
        self.inner.metadata.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "ModelVersion(version={}, format='{}', size={})",
            self.inner.version,
            self.inner.format,
            self.inner.size_bytes
        )
    }
}

// ── PyVaultConfig ────────────────────────────────────────────────────────────

/// Vault configuration — XDG-compliant paths and crypto settings.
#[pyclass(name = "VaultConfig")]
#[derive(Clone)]
struct PyVaultConfig {
    inner: VaultConfig,
}

#[pymethods]
impl PyVaultConfig {
    /// Create a new VaultConfig.
    ///
    /// If `vault_dir` is given it overrides the default XDG location.
    #[new]
    #[pyo3(signature = (vault_dir=None))]
    fn new(vault_dir: Option<String>) -> PyResult<Self> {
        let cfg = if let Some(dir) = vault_dir {
            let path = std::path::PathBuf::from(&dir);
            let dirs = crate::config::DirectoryPaths {
                config_dir: path.join("config"),
                data_dir: path.clone(),
                cache_dir: path.join("cache"),
                vault_dir: path.join("vaults"),
                log_dir: path.join("logs"),
                backends_dir: path.join("backends"),
                utilities_dir: path.join("utilities"),
                databases_dir: path.join("databases"),
            };
            VaultConfig::with_dirs(dirs).map_err(to_py_err)?
        } else {
            VaultConfig::new().map_err(to_py_err)?
        };
        Ok(Self { inner: cfg })
    }

    #[getter]
    fn vault_path(&self) -> String {
        self.inner.get_vault_path(None).to_string_lossy().to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "VaultConfig(vault_path='{}')",
            self.inner.get_vault_path(None).display()
        )
    }
}

// ── PyVault ──────────────────────────────────────────────────────────────────

/// The main vault — create, unlock, store, retrieve, and manage AI models.
///
/// Example::
///
///     from neuralvault import Vault, VaultConfig, ModelMetadata
///
///     vault = Vault()
///     vault.unlock(b"my-passphrase")
///     ver = vault.store_model("my-model", model_bytes,
///               ModelMetadata("my-model", "safetensors"))
///     data = vault.get_model("my-model")
///     vault.lock()
#[pyclass(name = "Vault")]
struct PyVault {
    inner: Vault,
}

#[pymethods]
impl PyVault {
    /// Create or open a vault.
    #[new]
    #[pyo3(signature = (config=None))]
    fn new(config: Option<&PyVaultConfig>) -> PyResult<Self> {
        let cfg = config.map(|c| c.inner.clone());
        let vault = Vault::new(cfg).map_err(to_py_err)?;
        Ok(Self { inner: vault })
    }

    /// Unlock the vault with a passphrase (`bytes`).
    fn unlock(&mut self, passphrase: &[u8]) -> PyResult<()> {
        self.inner
            .unlock(passphrase.to_vec())
            .map_err(to_py_err)
    }

    /// Lock the vault (zeroizes keys in memory).
    fn lock(&mut self) {
        self.inner.lock();
    }

    /// Whether the vault is currently unlocked.
    #[getter]
    fn is_unlocked(&self) -> bool {
        self.inner.is_unlocked()
    }

    /// Store a model and return the `ModelVersion`.
    ///
    /// Args:
    ///     name: Model name.
    ///     data: Raw model bytes.
    ///     metadata: `ModelMetadata` instance.
    ///     parent_version: Optional parent version number for lineage tracking.
    ///
    /// Returns:
    ///     `ModelVersion` — the newly created version.
    #[pyo3(signature = (name, data, metadata, parent_version=None))]
    fn store_model(
        &mut self,
        name: &str,
        data: &[u8],
        metadata: &PyModelMetadata,
        parent_version: Option<u32>,
    ) -> PyResult<PyModelVersion> {
        let ver = self
            .inner
            .store_model(
                name,
                data.to_vec(),
                metadata.inner.clone(),
                parent_version,
            )
            .map_err(to_py_err)?;
        Ok(PyModelVersion { inner: ver })
    }

    /// Retrieve model data as `bytes`.
    ///
    /// Args:
    ///     name: Model name.
    ///     version: Optional version number (latest if omitted).
    #[pyo3(signature = (name, version=None))]
    fn get_model<'py>(
        &self,
        py: Python<'py>,
        name: &str,
        version: Option<u32>,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let data = self.inner.get_model(name, version).map_err(to_py_err)?;
        Ok(PyBytes::new_bound(py, &data))
    }

    /// List all model names in the vault.
    fn list_models(&self) -> Vec<String> {
        self.inner.list_models()
    }

    /// List all versions for a given model.
    fn list_versions(&self, name: &str) -> Vec<PyModelVersion> {
        self.inner
            .list_versions(name)
            .into_iter()
            .map(|v| PyModelVersion { inner: v.clone() })
            .collect()
    }

    /// Get the full version lineage for a model version.
    fn get_lineage(&self, name: &str, version: u32) -> Vec<PyModelVersion> {
        self.inner
            .get_lineage(name, version)
            .into_iter()
            .map(|v| PyModelVersion { inner: v.clone() })
            .collect()
    }

    /// Delete a specific model version. Returns True if it existed.
    fn delete_version(&mut self, name: &str, version: u32) -> PyResult<bool> {
        self.inner.delete_version(name, version).map_err(to_py_err)
    }

    /// Get vault statistics.
    fn get_stats<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let stats = self.inner.get_stats().map_err(to_py_err)?;
        let dict = PyDict::new_bound(py);
        dict.set_item("model_count", stats.model_count)?;
        dict.set_item("total_versions", stats.total_versions)?;
        dict.set_item("total_size_bytes", stats.total_size_bytes)?;
        Ok(dict)
    }

    /// Change the vault passphrase (re-encrypts all models).
    ///
    /// Returns the number of models re-encrypted.
    fn change_passphrase(&mut self, new_passphrase: &[u8]) -> PyResult<usize> {
        self.inner
            .change_passphrase(new_passphrase.to_vec())
            .map_err(to_py_err)
    }

    /// Get the vault configuration.
    #[getter]
    fn config(&self) -> PyVaultConfig {
        PyVaultConfig {
            inner: self.inner.get_config().clone(),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Vault(unlocked={}, path='{}')",
            self.inner.is_unlocked(),
            self.inner.get_config().get_vault_path(None).display()
        )
    }
}

// ── PyModelCard ──────────────────────────────────────────────────────────────

/// Model card for documentation and transparency.
///
/// Example::
///
///     from neuralvault import ModelCard
///
///     card = ModelCard(
///         name="my-model", version="1.0",
///         model_type="transformer", description="A fine-tuned LLM"
///     )
///     print(card.to_markdown())
#[pyclass(name = "ModelCard")]
#[derive(Clone)]
struct PyModelCard {
    inner: ModelCard,
}

#[pymethods]
impl PyModelCard {
    /// Create a new model card.
    #[new]
    #[pyo3(signature = (
        name, version, model_type, *,
        description=None,
        developers=None,
        license=None,
        primary_use=None,
        out_of_scope=None
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        name: &str,
        version: &str,
        model_type: &str,
        description: Option<String>,
        developers: Option<Vec<String>>,
        license: Option<String>,
        primary_use: Option<String>,
        out_of_scope: Option<Vec<String>>,
    ) -> Self {
        let details = ModelDetails {
            name: name.to_string(),
            version: version.to_string(),
            model_type: model_type.to_string(),
            description: description.unwrap_or_default(),
            developers: developers.unwrap_or_default(),
            license,
            contact: None,
            architecture: String::new(),
            size: String::new(),
            framework: String::new(),
            format: String::new(),
            citation: None,
            repository: None,
            paper: None,
        };
        let intended = IntendedUse {
            primary_uses: primary_use.map(|s| vec![s]).unwrap_or_default(),
            primary_users: Vec::new(),
            out_of_scope_uses: out_of_scope.unwrap_or_default(),
            use_case_examples: None,
        };
        Self {
            inner: ModelCard::new(details, intended),
        }
    }

    /// Add training data information.
    #[pyo3(signature = (description, *, source=None, preprocessing=None))]
    fn set_training_data(
        &mut self,
        description: String,
        source: Option<String>,
        preprocessing: Option<String>,
    ) {
        let td = TrainingData {
            datasets: vec![description],
            sources: source.map(|s| vec![s]),
            collection_methods: None,
            preprocessing: preprocessing.map(|s| vec![s]),
            size: None,
            splits: None,
            languages: None,
            demographics: None,
        };
        {
            let card = self.inner.clone().with_training_data(td);
            self.inner = card;
        }
    }

    /// Add an evaluation metric.
    fn add_metric(&mut self, name: String, value: f64, description: String) {
        let metric = Metric {
            name,
            value,
            description: Some(description),
            threshold: None,
        };
        let eval = Evaluation {
            metrics: vec![metric],
            datasets: Vec::new(),
            benchmarks: None,
            methodology: None,
            performance_by_group: None,
        };
        {
            let card = self.inner.clone().with_evaluation(eval);
            self.inner = card;
        }
    }

    /// Add a custom metadata key-value pair.
    fn add_metadata(&mut self, key: String, value: String) {
        let card = self.inner.clone().add_metadata(key, value);
        self.inner = card;
    }

    /// Serialize to JSON string.
    fn to_json(&self) -> PyResult<String> {
        self.inner.to_json().map_err(to_py_err)
    }

    /// Serialize to YAML string.
    fn to_yaml(&self) -> PyResult<String> {
        self.inner.to_yaml().map_err(to_py_err)
    }

    /// Render as Markdown string.
    fn to_markdown(&self) -> String {
        self.inner.to_markdown()
    }

    /// Deserialize from JSON.
    #[staticmethod]
    fn from_json(json: &str) -> PyResult<Self> {
        let card = ModelCard::from_json(json).map_err(to_py_err)?;
        Ok(Self { inner: card })
    }

    /// Deserialize from YAML.
    #[staticmethod]
    fn from_yaml(yaml: &str) -> PyResult<Self> {
        let card = ModelCard::from_yaml(yaml).map_err(to_py_err)?;
        Ok(Self { inner: card })
    }

    fn __repr__(&self) -> String {
        format!(
            "ModelCard(name='{}', version='{}')",
            self.inner.model_details.name, self.inner.model_details.version
        )
    }
}

// ── PyVaultError wrapper ─────────────────────────────────────────────────────

/// Standalone utility: SHA-256 hex digest of data.
#[pyfunction]
fn sha256_hex(data: &[u8]) -> String {
    crate::crypto::FipsCrypto::hash_sha256_hex(data)
}

/// Library version string.
#[pyfunction]
fn version() -> &'static str {
    crate::VERSION
}

// ── module init ──────────────────────────────────────────────────────────────

/// The `neuralvault._native` extension module.
#[pymodule]
#[pyo3(name = "_native")]
fn neuralvault_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyModelFormat>()?;
    m.add_class::<PyModelMetadata>()?;
    m.add_class::<PyModelVersion>()?;
    m.add_class::<PyVaultConfig>()?;
    m.add_class::<PyVault>()?;
    m.add_class::<PyModelCard>()?;
    m.add_function(wrap_pyfunction!(sha256_hex, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}
