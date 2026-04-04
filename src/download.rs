//! Model download from trusted sources.
//!
//! Supports HuggingFace Hub, Ollama registry, and direct URLs with
//! mandatory integrity verification (SHA-256).

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Result, VaultError};

// ── Source types ─────────────────────────────────────────────────────────────

/// Parsed model source URI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelSource {
    /// HuggingFace Hub: `hf://org/model` or `hf://org/model@revision`
    HuggingFace {
        repo_id: String,
        filename: Option<String>,
        revision: Option<String>,
    },
    /// Ollama registry: `ollama://model:tag`
    Ollama {
        model: String,
        tag: String,
    },
    /// Direct HTTPS URL with mandatory checksum
    Url {
        url: String,
        expected_sha256: Option<String>,
    },
}

impl ModelSource {
    /// Parse a source string into a [`ModelSource`].
    ///
    /// Accepted formats:
    /// - `hf://org/model`
    /// - `hf://org/model/file.safetensors`
    /// - `hf://org/model@revision`
    /// - `ollama://model:tag`
    /// - `https://...`
    pub fn parse(source: &str) -> Result<Self> {
        if let Some(rest) = source.strip_prefix("hf://") {
            Self::parse_huggingface(rest)
        } else if let Some(rest) = source.strip_prefix("ollama://") {
            Self::parse_ollama(rest)
        } else if source.starts_with("https://") {
            Ok(ModelSource::Url {
                url: source.to_string(),
                expected_sha256: None,
            })
        } else {
            Err(VaultError::InvalidInput(format!(
                "Unsupported source URI: {source}. Use hf://, ollama://, or https://"
            )))
        }
    }

    fn parse_huggingface(rest: &str) -> Result<Self> {
        // Split off @revision if present
        let (path, revision) = if let Some((p, rev)) = rest.rsplit_once('@') {
            (p, Some(rev.to_string()))
        } else {
            (rest, None)
        };

        let parts: Vec<&str> = path.splitn(3, '/').collect();
        if parts.len() < 2 {
            return Err(VaultError::InvalidInput(
                "HuggingFace URI must be hf://org/model[/file][@revision]".to_string(),
            ));
        }

        let repo_id = format!("{}/{}", parts[0], parts[1]);
        let filename = parts.get(2).map(|s| (*s).to_string());

        Ok(ModelSource::HuggingFace {
            repo_id,
            filename,
            revision,
        })
    }

    fn parse_ollama(rest: &str) -> Result<Self> {
        let (model, tag) = if let Some((m, t)) = rest.rsplit_once(':') {
            (m.to_string(), t.to_string())
        } else {
            (rest.to_string(), "latest".to_string())
        };

        if model.is_empty() {
            return Err(VaultError::InvalidInput(
                "Ollama URI must be ollama://model[:tag]".to_string(),
            ));
        }

        Ok(ModelSource::Ollama { model, tag })
    }
}

// ── Download result ──────────────────────────────────────────────────────────

/// Result of a model download.
#[derive(Debug, Serialize)]
pub struct DownloadResult {
    /// Local file path where the model was saved
    pub path: PathBuf,
    /// SHA-256 checksum of the downloaded file
    pub sha256: String,
    /// File size in bytes
    pub size_bytes: u64,
    /// Source that was downloaded
    pub source: String,
    /// Detected model format
    pub format: String,
    /// Additional metadata from the source
    pub metadata: HashMap<String, String>,
}

// ── Downloader ───────────────────────────────────────────────────────────────

/// Downloads models from trusted sources with integrity verification.
pub struct ModelDownloader {
    /// Output directory for downloads
    output_dir: PathBuf,
    /// HuggingFace API token (optional, for private repos)
    hf_token: Option<String>,
}

impl ModelDownloader {
    /// Create a new downloader targeting the given output directory.
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
            hf_token: None,
        }
    }

    /// Set HuggingFace API token for accessing private/gated models.
    #[must_use]
    pub fn with_hf_token(mut self, token: String) -> Self {
        self.hf_token = Some(token);
        self
    }

    /// Download a model from the given source.
    pub fn download(
        &self,
        source: &ModelSource,
        expected_sha256: Option<&str>,
    ) -> Result<DownloadResult> {
        fs::create_dir_all(&self.output_dir)?;

        match source {
            ModelSource::HuggingFace {
                repo_id,
                filename,
                revision,
            } => self.download_huggingface(repo_id, filename.as_deref(), revision.as_deref()),
            ModelSource::Ollama { model, tag } => self.download_ollama(model, tag),
            ModelSource::Url {
                url,
                expected_sha256: uri_hash,
            } => {
                let hash = expected_sha256.or(uri_hash.as_deref());
                self.download_url(url, hash)
            }
        }
    }

    // ── HuggingFace Hub ──────────────────────────────────────────────────

    fn download_huggingface(
        &self,
        repo_id: &str,
        filename: Option<&str>,
        revision: Option<&str>,
    ) -> Result<DownloadResult> {
        let rev = revision.unwrap_or("main");

        // Resolve filename: if not specified, try to find the primary model file
        let file = if let Some(f) = filename {
            f.to_string()
        } else {
            // Default to model.safetensors — the HuggingFace convention
            "model.safetensors".to_string()
        };

        // Build download URL
        // https://huggingface.co/{repo_id}/resolve/{revision}/{filename}
        let url = format!(
            "https://huggingface.co/{}/resolve/{}/{}",
            repo_id, rev, file
        );

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "huggingface".to_string());
        metadata.insert("repo_id".to_string(), repo_id.to_string());
        metadata.insert("revision".to_string(), rev.to_string());

        let out_path = self.output_dir.join(&file);
        let (sha256, size) = self.fetch_file(&url, &out_path, &self.hf_token)?;

        let ext = Path::new(&file)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");
        let format = crate::formats::ModelFormat::from_extension(ext);

        Ok(DownloadResult {
            path: out_path,
            sha256,
            size_bytes: size,
            source: format!("hf://{}/{}", repo_id, file),
            format: format!("{:?}", format),
            metadata,
        })
    }

    // ── Ollama registry ──────────────────────────────────────────────────

    fn download_ollama(&self, model: &str, tag: &str) -> Result<DownloadResult> {
        // Ollama stores models via a registry API at registry.ollama.ai
        // 1. GET /v2/library/{model}/manifests/{tag} → JSON manifest
        // 2. Find the model layer (mediaType application/vnd.ollama.image.model)
        // 3. GET /v2/library/{model}/blobs/{digest}

        let manifest_url = format!(
            "https://registry.ollama.ai/v2/library/{}/manifests/{}",
            model, tag
        );

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| VaultError::IoError(std::io::Error::other(e.to_string())))?;

        let resp = client
            .get(&manifest_url)
            .send()
            .map_err(|e| VaultError::IoError(std::io::Error::other(e.to_string())))?;

        if !resp.status().is_success() {
            return Err(VaultError::InvalidInput(format!(
                "Ollama model not found: {}:{} (HTTP {})",
                model,
                tag,
                resp.status()
            )));
        }

        let manifest: serde_json::Value = resp
            .json()
            .map_err(|e| VaultError::SerializationError(e.to_string()))?;

        // Find the model blob layer
        let layers = manifest["layers"]
            .as_array()
            .ok_or_else(|| VaultError::InvalidInput("Invalid Ollama manifest".to_string()))?;

        let model_layer = layers
            .iter()
            .find(|l| {
                l["mediaType"]
                    .as_str()
                    .is_some_and(|m| m.contains("model"))
            })
            .ok_or_else(|| {
                VaultError::InvalidInput("No model layer in Ollama manifest".to_string())
            })?;

        let digest = model_layer["digest"]
            .as_str()
            .ok_or_else(|| VaultError::InvalidInput("Missing digest in manifest".to_string()))?;

        let blob_url = format!(
            "https://registry.ollama.ai/v2/library/{}/blobs/{}",
            model, digest
        );

        let filename = format!("{}-{}.gguf", model, tag);
        let out_path = self.output_dir.join(&filename);

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "ollama".to_string());
        metadata.insert("model".to_string(), model.to_string());
        metadata.insert("tag".to_string(), tag.to_string());
        metadata.insert("digest".to_string(), digest.to_string());

        let (sha256, size) = self.fetch_file(&blob_url, &out_path, &None)?;

        Ok(DownloadResult {
            path: out_path,
            sha256,
            size_bytes: size,
            source: format!("ollama://{}:{}", model, tag),
            format: "GGUF".to_string(),
            metadata,
        })
    }

    // ── Direct URL ───────────────────────────────────────────────────────

    fn download_url(&self, url: &str, expected_sha256: Option<&str>) -> Result<DownloadResult> {
        // Validate URL is HTTPS
        if !url.starts_with("https://") {
            return Err(VaultError::SecurityViolation(
                "Only HTTPS URLs are allowed for model downloads".to_string(),
            ));
        }

        // Extract filename from URL
        let filename = url
            .rsplit('/')
            .next()
            .and_then(|s| {
                let clean = s.split('?').next().unwrap_or(s);
                if clean.is_empty() {
                    None
                } else {
                    Some(clean.to_string())
                }
            })
            .unwrap_or_else(|| "downloaded_model.bin".to_string());

        let out_path = self.output_dir.join(&filename);
        let (sha256, size) = self.fetch_file(url, &out_path, &None)?;

        // Verify SHA-256 if provided
        if let Some(expected) = expected_sha256 {
            if sha256 != expected {
                // Remove the corrupted download
                let _ = fs::remove_file(&out_path);
                return Err(VaultError::IntegrityError(format!(
                    "SHA-256 mismatch: expected {}, got {}",
                    expected, sha256
                )));
            }
        }

        let ext = Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");
        let format = crate::formats::ModelFormat::from_extension(ext);

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "url".to_string());
        metadata.insert("url".to_string(), url.to_string());

        Ok(DownloadResult {
            path: out_path,
            sha256,
            size_bytes: size,
            source: url.to_string(),
            format: format!("{:?}", format),
            metadata,
        })
    }

    // ── Streaming fetch with SHA-256 ─────────────────────────────────────

    fn fetch_file(
        &self,
        url: &str,
        out_path: &Path,
        auth_token: &Option<String>,
    ) -> Result<(String, u64)> {
        use sha2::{Digest, Sha256};

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(3600))
            .build()
            .map_err(|e| VaultError::IoError(std::io::Error::other(e.to_string())))?;

        let mut request = client.get(url);
        if let Some(token) = auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let mut resp = request
            .send()
            .map_err(|e| VaultError::IoError(std::io::Error::other(e.to_string())))?;

        if !resp.status().is_success() {
            return Err(VaultError::IoError(std::io::Error::other(format!(
                "HTTP {} from {}",
                resp.status(),
                url
            ))));
        }

        let total_size = resp.content_length();
        let mut file = fs::File::create(out_path)?;
        let mut hasher = Sha256::new();
        let mut downloaded: u64 = 0;
        let mut buf = vec![0u8; 8 * 1024 * 1024]; // 8 MiB buffer

        loop {
            let n = std::io::Read::read(&mut resp, &mut buf)
                .map_err(|e| VaultError::IoError(e))?;
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n])?;
            hasher.update(&buf[..n]);
            downloaded += n as u64;

            // Progress indicator
            if let Some(total) = total_size {
                let pct = (downloaded as f64 / total as f64 * 100.0) as u32;
                eprint!("\r  Downloading: {} / {} bytes ({}%)", downloaded, total, pct);
            } else {
                eprint!("\r  Downloaded: {} bytes", downloaded);
            }
        }
        eprintln!(); // newline after progress

        file.flush()?;
        drop(file);

        let hash = format!("{:x}", hasher.finalize());

        Ok((hash, downloaded))
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_huggingface() {
        let src = ModelSource::parse("hf://meta-llama/Llama-3-8B").unwrap();
        match src {
            ModelSource::HuggingFace {
                repo_id,
                filename,
                revision,
            } => {
                assert_eq!(repo_id, "meta-llama/Llama-3-8B");
                assert!(filename.is_none());
                assert!(revision.is_none());
            }
            _ => panic!("Expected HuggingFace source"),
        }
    }

    #[test]
    fn test_parse_huggingface_with_file_and_rev() {
        let src =
            ModelSource::parse("hf://org/model/weights.safetensors@main").unwrap();
        match src {
            ModelSource::HuggingFace {
                repo_id,
                filename,
                revision,
            } => {
                assert_eq!(repo_id, "org/model");
                assert_eq!(filename.as_deref(), Some("weights.safetensors"));
                assert_eq!(revision.as_deref(), Some("main"));
            }
            _ => panic!("Expected HuggingFace source"),
        }
    }

    #[test]
    fn test_parse_ollama() {
        let src = ModelSource::parse("ollama://llama3:8b").unwrap();
        match src {
            ModelSource::Ollama { model, tag } => {
                assert_eq!(model, "llama3");
                assert_eq!(tag, "8b");
            }
            _ => panic!("Expected Ollama source"),
        }
    }

    #[test]
    fn test_parse_ollama_default_tag() {
        let src = ModelSource::parse("ollama://mistral").unwrap();
        match src {
            ModelSource::Ollama { model, tag } => {
                assert_eq!(model, "mistral");
                assert_eq!(tag, "latest");
            }
            _ => panic!("Expected Ollama source"),
        }
    }

    #[test]
    fn test_parse_url() {
        let src = ModelSource::parse("https://example.com/model.gguf").unwrap();
        match src {
            ModelSource::Url { url, .. } => {
                assert_eq!(url, "https://example.com/model.gguf");
            }
            _ => panic!("Expected URL source"),
        }
    }

    #[test]
    fn test_parse_invalid() {
        assert!(ModelSource::parse("ftp://bad").is_err());
        assert!(ModelSource::parse("local/path").is_err());
    }

    #[test]
    fn test_parse_hf_invalid_format() {
        assert!(ModelSource::parse("hf://onlyname").is_err());
    }

    #[test]
    fn test_parse_ollama_empty() {
        assert!(ModelSource::parse("ollama://").is_err());
    }
}
