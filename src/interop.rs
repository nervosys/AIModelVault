//! Inference engine interop — register vault models with Ollama and LM Studio.
//!
//! Provides helpers to:
//! - Create an Ollama `Modelfile` and register a model via `ollama create`
//! - Symlink or copy a GGUF model into the LM Studio model directory

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::error::{Result, VaultError};

// ── Types ────────────────────────────────────────────────────────────────────

/// Which inference engine to register with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InferenceEngine {
    Ollama,
    LmStudio,
}

impl std::fmt::Display for InferenceEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ollama => write!(f, "Ollama"),
            Self::LmStudio => write!(f, "LM Studio"),
        }
    }
}

/// Result of a registration attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResult {
    pub engine: String,
    pub model_name: String,
    pub success: bool,
    pub message: String,
    /// Path where the model was placed or the Modelfile was created
    pub path: Option<String>,
}

/// Options for Ollama registration.
#[derive(Debug, Clone)]
pub struct OllamaOptions {
    /// Model name for Ollama
    pub name: String,
    /// Path to the GGUF file on disk
    pub model_path: PathBuf,
    /// Optional system prompt
    pub system_prompt: Option<String>,
    /// Optional template string
    pub template: Option<String>,
    /// Optional parameter overrides (e.g. temperature, top_k)
    pub parameters: Vec<(String, String)>,
}

/// Options for LM Studio registration.
#[derive(Debug, Clone)]
pub struct LmStudioOptions {
    /// Model name (used as directory name)
    pub name: String,
    /// Path to the model file on disk
    pub model_path: PathBuf,
    /// Custom LM Studio models directory (overrides default)
    pub models_dir: Option<PathBuf>,
}

// ── Ollama registration ──────────────────────────────────────────────────────

/// Register a model with Ollama by creating a Modelfile and running `ollama create`.
pub fn register_ollama(opts: &OllamaOptions) -> Result<RegistrationResult> {
    // Verify the model file exists and is GGUF
    if !opts.model_path.exists() {
        return Err(VaultError::InvalidInput(format!(
            "Model file not found: {}",
            opts.model_path.display()
        )));
    }

    // Build the Modelfile content
    let modelfile_content = build_modelfile(opts)?;

    // Write Modelfile to a temp location next to the model
    let modelfile_path = opts.model_path.with_extension("Modelfile");
    fs::write(&modelfile_path, &modelfile_content)?;

    // Check if ollama is available
    let ollama_check = Command::new("ollama").arg("--version").output();
    if ollama_check.is_err() {
        // Clean up Modelfile but still report where it is
        return Ok(RegistrationResult {
            engine: "Ollama".to_string(),
            model_name: opts.name.clone(),
            success: false,
            message: format!(
                "Ollama CLI not found. Modelfile created at: {}\nRun manually: ollama create {} -f {}",
                modelfile_path.display(),
                opts.name,
                modelfile_path.display()
            ),
            path: Some(modelfile_path.display().to_string()),
        });
    }

    // Run `ollama create`
    let output = Command::new("ollama")
        .arg("create")
        .arg(&opts.name)
        .arg("-f")
        .arg(&modelfile_path)
        .output()
        .map_err(VaultError::IoError)?;

    let success = output.status.success();
    let message = if success {
        format!(
            "Model '{}' registered with Ollama. Run: ollama run {}",
            opts.name, opts.name
        )
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        format!(
            "Failed to register with Ollama: {}. Modelfile at: {}",
            stderr.trim(),
            modelfile_path.display()
        )
    };

    Ok(RegistrationResult {
        engine: "Ollama".to_string(),
        model_name: opts.name.clone(),
        success,
        message,
        path: Some(modelfile_path.display().to_string()),
    })
}

/// Build an Ollama Modelfile from options.
fn build_modelfile(opts: &OllamaOptions) -> Result<String> {
    let mut lines = vec![format!("FROM {}", opts.model_path.display())];

    if let Some(system) = &opts.system_prompt {
        lines.push(format!("SYSTEM \"{}\"", system.replace('"', "\\\"")));
    }

    if let Some(template) = &opts.template {
        lines.push(format!("TEMPLATE \"{}\"", template.replace('"', "\\\"")));
    }

    for (key, value) in &opts.parameters {
        lines.push(format!("PARAMETER {} {}", key, value));
    }

    Ok(lines.join("\n") + "\n")
}

// ── LM Studio registration ──────────────────────────────────────────────────

/// Register a model with LM Studio by copying it to the models directory.
pub fn register_lm_studio(opts: &LmStudioOptions) -> Result<RegistrationResult> {
    if !opts.model_path.exists() {
        return Err(VaultError::InvalidInput(format!(
            "Model file not found: {}",
            opts.model_path.display()
        )));
    }

    let models_dir = match &opts.models_dir {
        Some(d) => d.clone(),
        None => default_lm_studio_dir()?,
    };

    // Create model subdirectory
    let model_dir = models_dir.join(&opts.name);
    fs::create_dir_all(&model_dir)?;

    // Copy model file
    let file_name = opts
        .model_path
        .file_name()
        .ok_or_else(|| VaultError::InvalidInput("Invalid model file path".to_string()))?;
    let dest = model_dir.join(file_name);

    fs::copy(&opts.model_path, &dest)?;

    Ok(RegistrationResult {
        engine: "LM Studio".to_string(),
        model_name: opts.name.clone(),
        success: true,
        message: format!(
            "Model '{}' copied to LM Studio models directory. Restart LM Studio to see it.",
            opts.name
        ),
        path: Some(dest.display().to_string()),
    })
}

/// Determine the default LM Studio models directory.
fn default_lm_studio_dir() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(home) = std::env::var("USERPROFILE") {
            return Ok(PathBuf::from(home)
                .join(".cache")
                .join("lm-studio")
                .join("models"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join(".cache")
                .join("lm-studio")
                .join("models"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join(".cache")
                .join("lm-studio")
                .join("models"));
        }
    }

    Err(VaultError::InvalidInput(
        "Cannot determine LM Studio models directory. Use --models-dir to specify.".to_string(),
    ))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_build_modelfile_basic() {
        let opts = OllamaOptions {
            name: "my-model".to_string(),
            model_path: PathBuf::from("/tmp/model.gguf"),
            system_prompt: None,
            template: None,
            parameters: vec![],
        };
        let mf = build_modelfile(&opts).unwrap();
        assert!(mf.starts_with("FROM /tmp/model.gguf"));
    }

    #[test]
    fn test_build_modelfile_full() {
        let opts = OllamaOptions {
            name: "test".to_string(),
            model_path: PathBuf::from("/models/test.gguf"),
            system_prompt: Some("You are a helpful assistant.".to_string()),
            template: Some("{{.Prompt}}".to_string()),
            parameters: vec![
                ("temperature".to_string(), "0.7".to_string()),
                ("top_k".to_string(), "40".to_string()),
            ],
        };
        let mf = build_modelfile(&opts).unwrap();
        assert!(mf.contains("FROM /models/test.gguf"));
        assert!(mf.contains("SYSTEM"));
        assert!(mf.contains("PARAMETER temperature 0.7"));
        assert!(mf.contains("PARAMETER top_k 40"));
    }

    #[test]
    fn test_register_ollama_missing_file() {
        let opts = OllamaOptions {
            name: "bad".to_string(),
            model_path: PathBuf::from("/nonexistent/model.gguf"),
            system_prompt: None,
            template: None,
            parameters: vec![],
        };
        let result = register_ollama(&opts);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_lm_studio_missing_file() {
        let opts = LmStudioOptions {
            name: "bad".to_string(),
            model_path: PathBuf::from("/nonexistent/model.gguf"),
            models_dir: None,
        };
        let result = register_lm_studio(&opts);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_lm_studio_copy() {
        let dir = tempfile::tempdir().unwrap();
        let model_path = dir.path().join("model.gguf");
        {
            let mut f = fs::File::create(&model_path).unwrap();
            f.write_all(b"fake gguf data").unwrap();
        }

        let models_dir = dir.path().join("lm-studio-models");
        let opts = LmStudioOptions {
            name: "test-model".to_string(),
            model_path,
            models_dir: Some(models_dir.clone()),
        };

        let result = register_lm_studio(&opts).unwrap();
        assert!(result.success);
        assert!(models_dir.join("test-model").join("model.gguf").exists());
    }

    #[test]
    fn test_inference_engine_display() {
        assert_eq!(InferenceEngine::Ollama.to_string(), "Ollama");
        assert_eq!(InferenceEngine::LmStudio.to_string(), "LM Studio");
    }
}
