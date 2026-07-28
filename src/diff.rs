//! Model diffing — compare two versions of a model.
//!
//! Compares tensor metadata (shapes, dtypes, parameter counts) between
//! two model files or vault versions.  Does NOT load full tensor data
//! into memory; it parses headers only (SafeTensors, GGUF metadata).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

// ── Diff types ───────────────────────────────────────────────────────────────

/// Summary of differences between two model files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDiff {
    /// Left-side label (e.g. "model@v1")
    pub left: String,
    /// Right-side label (e.g. "model@v2")
    pub right: String,
    /// Size difference in bytes (right − left)
    pub size_delta: i64,
    /// Left file size
    pub left_size: u64,
    /// Right file size
    pub right_size: u64,
    /// Format of left file
    pub left_format: String,
    /// Format of right file
    pub right_format: String,
    /// Tensors added in right that aren't in left
    pub added_tensors: Vec<TensorInfo>,
    /// Tensors removed from left that aren't in right
    pub removed_tensors: Vec<TensorInfo>,
    /// Tensors present in both but with different shapes/dtypes
    pub changed_tensors: Vec<TensorChange>,
    /// Tensors unchanged between versions
    pub unchanged_count: usize,
    /// Total estimated parameter delta
    pub param_delta: i64,
    /// Summary statistics
    pub summary: DiffSummary,
}

/// Information about a single tensor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorInfo {
    pub name: String,
    pub shape: Vec<usize>,
    pub dtype: String,
    pub param_count: u64,
}

/// A tensor that changed between versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorChange {
    pub name: String,
    pub left_shape: Vec<usize>,
    pub right_shape: Vec<usize>,
    pub left_dtype: String,
    pub right_dtype: String,
    pub shape_changed: bool,
    pub dtype_changed: bool,
    pub param_delta: i64,
}

/// Summary statistics for a diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub total_left_tensors: usize,
    pub total_right_tensors: usize,
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
    pub unchanged: usize,
    pub size_change_pct: f64,
}

// ── Parsed tensor map ────────────────────────────────────────────────────────

/// Parsed tensor metadata from a model file header.
type TensorMap = BTreeMap<String, TensorInfo>;

// ── Diffing engine ───────────────────────────────────────────────────────────

/// Compares model files at the tensor level.
pub struct ModelDiffer;

impl ModelDiffer {
    /// Diff two model files on disk.
    pub fn diff_files(
        left_path: &Path,
        right_path: &Path,
        left_label: &str,
        right_label: &str,
    ) -> Result<ModelDiff> {
        let left_data = fs::read(left_path)?;
        let right_data = fs::read(right_path)?;

        let left_format = crate::formats::ModelFormat::from_extension(
            left_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("bin"),
        );
        let right_format = crate::formats::ModelFormat::from_extension(
            right_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("bin"),
        );

        let left_tensors = Self::parse_tensors(&left_data, &left_format);
        let right_tensors = Self::parse_tensors(&right_data, &right_format);

        Self::compute_diff(
            &left_tensors,
            &right_tensors,
            left_label,
            right_label,
            left_data.len() as u64,
            right_data.len() as u64,
            &format!("{:?}", left_format),
            &format!("{:?}", right_format),
        )
    }

    /// Diff two raw byte slices (for vault-stored models).
    pub fn diff_bytes(
        left: &[u8],
        right: &[u8],
        left_label: &str,
        right_label: &str,
        left_fmt: &str,
        right_fmt: &str,
    ) -> Result<ModelDiff> {
        // These come from version records, which store `format.name()`.
        let left_format = crate::formats::ModelFormat::from_stored(left_fmt);
        let right_format = crate::formats::ModelFormat::from_stored(right_fmt);

        let left_tensors = Self::parse_tensors(left, &left_format);
        let right_tensors = Self::parse_tensors(right, &right_format);

        Self::compute_diff(
            &left_tensors,
            &right_tensors,
            left_label,
            right_label,
            left.len() as u64,
            right.len() as u64,
            left_fmt,
            right_fmt,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_diff(
        left_tensors: &TensorMap,
        right_tensors: &TensorMap,
        left_label: &str,
        right_label: &str,
        left_size: u64,
        right_size: u64,
        left_format: &str,
        right_format: &str,
    ) -> Result<ModelDiff> {
        let left_names: HashSet<&str> = left_tensors.keys().map(|s| s.as_str()).collect();
        let right_names: HashSet<&str> = right_tensors.keys().map(|s| s.as_str()).collect();

        let mut added_tensors = Vec::new();
        let mut removed_tensors = Vec::new();
        let mut changed_tensors = Vec::new();
        let mut unchanged_count = 0;
        let mut param_delta: i64 = 0;

        // Added (in right but not left)
        for name in &right_names {
            if !left_names.contains(name) {
                if let Some(t) = right_tensors.get(*name) {
                    param_delta += t.param_count as i64;
                    added_tensors.push(t.clone());
                }
            }
        }

        // Removed (in left but not right)
        for name in &left_names {
            if !right_names.contains(name) {
                if let Some(t) = left_tensors.get(*name) {
                    param_delta -= t.param_count as i64;
                    removed_tensors.push(t.clone());
                }
            }
        }

        // Changed or unchanged (in both)
        for name in left_names.intersection(&right_names) {
            let lt = &left_tensors[*name];
            let rt = &right_tensors[*name];

            let shape_changed = lt.shape != rt.shape;
            let dtype_changed = lt.dtype != rt.dtype;

            if shape_changed || dtype_changed {
                let delta = rt.param_count as i64 - lt.param_count as i64;
                param_delta += delta;
                changed_tensors.push(TensorChange {
                    name: (*name).to_string(),
                    left_shape: lt.shape.clone(),
                    right_shape: rt.shape.clone(),
                    left_dtype: lt.dtype.clone(),
                    right_dtype: rt.dtype.clone(),
                    shape_changed,
                    dtype_changed,
                    param_delta: delta,
                });
            } else {
                unchanged_count += 1;
            }
        }

        let size_change_pct = if left_size > 0 {
            (right_size as f64 - left_size as f64) / left_size as f64 * 100.0
        } else {
            0.0
        };

        let summary = DiffSummary {
            total_left_tensors: left_tensors.len(),
            total_right_tensors: right_tensors.len(),
            added: added_tensors.len(),
            removed: removed_tensors.len(),
            changed: changed_tensors.len(),
            unchanged: unchanged_count,
            size_change_pct,
        };

        Ok(ModelDiff {
            left: left_label.to_string(),
            right: right_label.to_string(),
            size_delta: right_size as i64 - left_size as i64,
            left_size,
            right_size,
            left_format: left_format.to_string(),
            right_format: right_format.to_string(),
            added_tensors,
            removed_tensors,
            changed_tensors,
            unchanged_count,
            param_delta,
            summary,
        })
    }

    // ── Tensor parsing (header-only) ─────────────────────────────────────

    fn parse_tensors(data: &[u8], format: &crate::formats::ModelFormat) -> TensorMap {
        match format {
            crate::formats::ModelFormat::Safetensors => Self::parse_safetensors_header(data),
            crate::formats::ModelFormat::GGUF => Self::parse_gguf_header(data),
            _ => Self::parse_generic(data),
        }
    }

    /// Parse SafeTensors header to extract tensor metadata.
    ///
    /// SafeTensors file layout: [8-byte LE header_size][JSON header][tensor data]
    fn parse_safetensors_header(data: &[u8]) -> TensorMap {
        let mut map = BTreeMap::new();

        if data.len() < 8 {
            return map;
        }

        let header_size = u64::from_le_bytes(data[0..8].try_into().unwrap_or_default()) as usize;

        if data.len() < 8 + header_size || header_size > 100_000_000 {
            return map;
        }

        let header_json = &data[8..8 + header_size];
        if let Ok(header) =
            serde_json::from_slice::<HashMap<String, serde_json::Value>>(header_json)
        {
            for (name, info) in &header {
                if name == "__metadata__" {
                    continue;
                }
                if let Some(obj) = info.as_object() {
                    let dtype = obj
                        .get("dtype")
                        .and_then(|v| v.as_str())
                        .unwrap_or("F32")
                        .to_string();
                    let shape: Vec<usize> = obj
                        .get("shape")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_u64().map(|n| n as usize))
                                .collect()
                        })
                        .unwrap_or_default();

                    let param_count: u64 = if shape.is_empty() {
                        1
                    } else {
                        shape.iter().product::<usize>() as u64
                    };

                    map.insert(
                        name.clone(),
                        TensorInfo {
                            name: name.clone(),
                            shape,
                            dtype,
                            param_count,
                        },
                    );
                }
            }
        }

        map
    }

    /// Parse GGUF header for tensor info.
    ///
    /// GGUF v3 layout: [4-byte magic "GGUF"][4-byte version][8-byte tensor_count]
    /// [8-byte metadata_count][metadata KV pairs][tensor infos].
    ///
    /// The metadata block is variable-length, so the tensor infos can only be
    /// reached by walking every KV pair. A truncated or malformed header yields
    /// whatever tensors were read before the break rather than an error — the
    /// caller only ever uses this for comparison.
    fn parse_gguf_header(data: &[u8]) -> TensorMap {
        let mut map = BTreeMap::new();

        // Check GGUF magic
        if data.len() < 24 || &data[0..4] != b"GGUF" {
            return map;
        }

        let mut r = GgufReader::after_magic(data);

        let Some(version) = r.u32() else { return map };
        if !(2..=3).contains(&version) {
            return map;
        }

        let (Some(tensor_count), Some(metadata_count)) = (r.u64(), r.u64()) else {
            return map;
        };

        // Walk past the metadata without decoding it; we only need the offset
        // it leaves the cursor at.
        for _ in 0..metadata_count {
            if r.string().is_none() {
                return map;
            }
            let Some(value_type) = r.u32() else {
                return map;
            };
            if r.skip_value(value_type, 0).is_none() {
                return map;
            }
        }

        for _ in 0..tensor_count {
            let Some(name) = r.string() else { return map };
            let Some(n_dims) = r.u32() else { return map };
            // ggml itself caps tensors at 4 dimensions; the extra headroom just
            // avoids rejecting a future revision outright.
            if n_dims > GGUF_MAX_DIMS {
                return map;
            }

            let mut shape = Vec::with_capacity(n_dims as usize);
            for _ in 0..n_dims {
                let Some(dim) = r.u64().and_then(|d| usize::try_from(d).ok()) else {
                    return map;
                };
                shape.push(dim);
            }

            let Some(ggml_type) = r.u32() else { return map };
            // Data offset — not part of the comparison.
            if r.u64().is_none() {
                return map;
            }

            let param_count = shape
                .iter()
                .try_fold(1u64, |acc, &d| acc.checked_mul(d as u64))
                .unwrap_or(u64::MAX);

            map.insert(
                name.clone(),
                TensorInfo {
                    name,
                    shape,
                    dtype: ggml_type_name(ggml_type).to_string(),
                    param_count,
                },
            );
        }

        map
    }

    /// Generic fallback: no tensor-level info available, just file-level.
    fn parse_generic(_data: &[u8]) -> TensorMap {
        BTreeMap::new()
    }
}

// ── GGUF header primitives ───────────────────────────────────────────────────

/// `gguf_metadata_value_type` tags.
const GGUF_UINT8: u32 = 0;
const GGUF_INT8: u32 = 1;
const GGUF_UINT16: u32 = 2;
const GGUF_INT16: u32 = 3;
const GGUF_UINT32: u32 = 4;
const GGUF_INT32: u32 = 5;
const GGUF_FLOAT32: u32 = 6;
const GGUF_BOOL: u32 = 7;
const GGUF_STRING: u32 = 8;
const GGUF_ARRAY: u32 = 9;
const GGUF_UINT64: u32 = 10;
const GGUF_INT64: u32 = 11;
const GGUF_FLOAT64: u32 = 12;

/// Upper bound on a tensor's declared dimension count.
const GGUF_MAX_DIMS: u32 = 8;

/// Human-readable name for a `ggml_type` discriminant.
fn ggml_type_name(ggml_type: u32) -> &'static str {
    match ggml_type {
        0 => "F32",
        1 => "F16",
        2 => "Q4_0",
        3 => "Q4_1",
        6 => "Q5_0",
        7 => "Q5_1",
        8 => "Q8_0",
        9 => "Q8_1",
        10 => "Q2_K",
        11 => "Q3_K",
        12 => "Q4_K",
        13 => "Q5_K",
        14 => "Q6_K",
        15 => "Q8_K",
        16 => "IQ2_XXS",
        17 => "IQ2_XS",
        18 => "IQ3_XXS",
        19 => "IQ1_S",
        20 => "IQ4_NL",
        21 => "IQ3_S",
        22 => "IQ2_S",
        23 => "IQ4_XS",
        24 => "I8",
        25 => "I16",
        26 => "I32",
        27 => "I64",
        28 => "F64",
        29 => "IQ1_M",
        30 => "BF16",
        // 4 and 5 were Q4_2/Q4_3, removed from ggml.
        _ => "UNKNOWN",
    }
}

/// Bounds-checked little-endian cursor over a GGUF header.
///
/// Every accessor returns `None` rather than panicking when the header is
/// shorter than it claims to be, so a corrupt file degrades to a partial
/// tensor map instead of aborting the diff.
struct GgufReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> GgufReader<'a> {
    /// Start reading immediately after the 4-byte `GGUF` magic.
    fn after_magic(data: &'a [u8]) -> Self {
        Self { data, pos: 4 }
    }

    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        let slice = self.data.get(self.pos..end)?;
        self.pos = end;
        Some(slice)
    }

    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }

    /// GGUF strings are a u64 byte length followed by non-NUL-terminated UTF-8.
    fn string(&mut self) -> Option<String> {
        let len = usize::try_from(self.u64()?).ok()?;
        let bytes = self.take(len)?;
        Some(String::from_utf8_lossy(bytes).into_owned())
    }

    /// Advance past one metadata value of the given type without decoding it.
    fn skip_value(&mut self, value_type: u32, depth: u32) -> Option<()> {
        // Nested arrays are not legal GGUF, but a malformed file must not be
        // able to drive this into unbounded recursion.
        if depth > 4 {
            return None;
        }
        match value_type {
            GGUF_UINT8 | GGUF_INT8 | GGUF_BOOL => self.take(1).map(|_| ()),
            GGUF_UINT16 | GGUF_INT16 => self.take(2).map(|_| ()),
            GGUF_UINT32 | GGUF_INT32 | GGUF_FLOAT32 => self.take(4).map(|_| ()),
            GGUF_UINT64 | GGUF_INT64 | GGUF_FLOAT64 => self.take(8).map(|_| ()),
            GGUF_STRING => self.string().map(|_| ()),
            GGUF_ARRAY => {
                let elem_type = self.u32()?;
                let count = self.u64()?;
                for _ in 0..count {
                    self.skip_value(elem_type, depth + 1)?;
                }
                Some(())
            }
            // An unknown tag has an unknown width, so the rest of the header is
            // no longer navigable.
            _ => None,
        }
    }
}

// ── Display helpers ──────────────────────────────────────────────────────────

impl ModelDiff {
    /// Format the diff as a human-readable string.
    pub fn display(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("Model Diff: {} ↔ {}\n", self.left, self.right));
        out.push_str("──────────────────────────────────\n");
        out.push_str(&format!(
            "Size: {} → {} ({:+} bytes, {:+.1}%)\n",
            format_size(self.left_size),
            format_size(self.right_size),
            self.size_delta,
            self.summary.size_change_pct
        ));
        out.push_str(&format!(
            "Format: {} → {}\n",
            self.left_format, self.right_format
        ));
        out.push_str(&format!(
            "Tensors: {} → {} (added: {}, removed: {}, changed: {}, unchanged: {})\n",
            self.summary.total_left_tensors,
            self.summary.total_right_tensors,
            self.summary.added,
            self.summary.removed,
            self.summary.changed,
            self.summary.unchanged,
        ));

        if self.param_delta != 0 {
            out.push_str(&format!("Parameter delta: {:+}\n", self.param_delta));
        }

        if !self.added_tensors.is_empty() {
            out.push_str("\n+ Added tensors:\n");
            for t in &self.added_tensors {
                out.push_str(&format!(
                    "  + {} {:?} {} ({} params)\n",
                    t.name, t.shape, t.dtype, t.param_count
                ));
            }
        }

        if !self.removed_tensors.is_empty() {
            out.push_str("\n- Removed tensors:\n");
            for t in &self.removed_tensors {
                out.push_str(&format!(
                    "  - {} {:?} {} ({} params)\n",
                    t.name, t.shape, t.dtype, t.param_count
                ));
            }
        }

        if !self.changed_tensors.is_empty() {
            out.push_str("\n~ Changed tensors:\n");
            for t in &self.changed_tensors {
                if t.shape_changed {
                    out.push_str(&format!(
                        "  ~ {} shape: {:?} → {:?}\n",
                        t.name, t.left_shape, t.right_shape
                    ));
                }
                if t.dtype_changed {
                    out.push_str(&format!(
                        "  ~ {} dtype: {} → {}\n",
                        t.name, t.left_dtype, t.right_dtype
                    ));
                }
            }
        }

        out
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GiB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MiB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.2} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_safetensors_header(tensors: &[(&str, &str, &[usize])]) -> Vec<u8> {
        let mut header = serde_json::Map::new();
        for (name, dtype, shape) in tensors {
            let mut info = serde_json::Map::new();
            info.insert("dtype".to_string(), serde_json::json!(dtype));
            info.insert("shape".to_string(), serde_json::json!(shape));
            info.insert("data_offsets".to_string(), serde_json::json!([0, 0]));
            header.insert(name.to_string(), serde_json::Value::Object(info));
        }
        let json = serde_json::to_vec(&header).unwrap();
        let mut data = Vec::new();
        data.extend_from_slice(&(json.len() as u64).to_le_bytes());
        data.extend_from_slice(&json);
        data
    }

    /// Build a syntactically valid GGUF v3 header: magic, counts, one string
    /// and one array metadata pair (to prove the walker steps over them), then
    /// the tensor infos.
    fn make_gguf_header(tensors: &[(&str, &[u64], u32)]) -> Vec<u8> {
        fn push_string(out: &mut Vec<u8>, s: &str) {
            out.extend_from_slice(&(s.len() as u64).to_le_bytes());
            out.extend_from_slice(s.as_bytes());
        }

        let mut data = Vec::new();
        data.extend_from_slice(b"GGUF");
        data.extend_from_slice(&3u32.to_le_bytes()); // version
        data.extend_from_slice(&(tensors.len() as u64).to_le_bytes());
        data.extend_from_slice(&2u64.to_le_bytes()); // metadata_count

        // general.architecture = "llama"
        push_string(&mut data, "general.architecture");
        data.extend_from_slice(&GGUF_STRING.to_le_bytes());
        push_string(&mut data, "llama");

        // tokenizer.ggml.tokens = ["a", "bb"]
        push_string(&mut data, "tokenizer.ggml.tokens");
        data.extend_from_slice(&GGUF_ARRAY.to_le_bytes());
        data.extend_from_slice(&GGUF_STRING.to_le_bytes());
        data.extend_from_slice(&2u64.to_le_bytes());
        push_string(&mut data, "a");
        push_string(&mut data, "bb");

        for (name, dims, ggml_type) in tensors {
            push_string(&mut data, name);
            data.extend_from_slice(&(dims.len() as u32).to_le_bytes());
            for d in *dims {
                data.extend_from_slice(&d.to_le_bytes());
            }
            data.extend_from_slice(&ggml_type.to_le_bytes());
            data.extend_from_slice(&0u64.to_le_bytes()); // offset
        }

        data
    }

    #[test]
    fn test_gguf_header_yields_real_tensor_metadata() {
        let data = make_gguf_header(&[
            ("blk.0.attn_q.weight", &[4096, 4096], 12), // Q4_K
            ("output_norm.weight", &[4096], 0),         // F32
        ]);

        let map = ModelDiffer::parse_tensors(&data, &crate::formats::ModelFormat::GGUF);

        assert_eq!(map.len(), 2);
        let q = &map["blk.0.attn_q.weight"];
        assert_eq!(q.shape, vec![4096, 4096]);
        assert_eq!(q.dtype, "Q4_K");
        assert_eq!(q.param_count, 4096 * 4096);
        let norm = &map["output_norm.weight"];
        assert_eq!(norm.shape, vec![4096]);
        assert_eq!(norm.dtype, "F32");
    }

    /// The previous implementation invented `tensor_0..N` names with empty
    /// shapes, so any two GGUF files with the same tensor count diffed as
    /// identical no matter how different they actually were.
    #[test]
    fn test_gguf_diff_detects_changes_at_equal_tensor_count() {
        let left = make_gguf_header(&[("blk.0.attn_q.weight", &[4096, 4096], 0)]);
        let right = make_gguf_header(&[("blk.0.attn_q.weight", &[4096, 4096], 12)]);

        let diff = ModelDiffer::diff_bytes(&left, &right, "f32", "q4k", "gguf", "gguf").unwrap();

        assert_eq!(diff.summary.changed, 1);
        assert_eq!(diff.changed_tensors[0].left_dtype, "F32");
        assert_eq!(diff.changed_tensors[0].right_dtype, "Q4_K");
        assert!(diff.changed_tensors[0].dtype_changed);
        assert!(!diff.changed_tensors[0].shape_changed);
    }

    #[test]
    fn test_gguf_renamed_tensor_is_add_plus_remove() {
        let left = make_gguf_header(&[("old.name", &[8], 0)]);
        let right = make_gguf_header(&[("new.name", &[8], 0)]);

        let diff = ModelDiffer::diff_bytes(&left, &right, "a", "b", "gguf", "gguf").unwrap();

        assert_eq!(diff.summary.added, 1);
        assert_eq!(diff.summary.removed, 1);
        assert_eq!(diff.summary.unchanged, 0);
    }

    #[test]
    fn test_gguf_truncated_header_is_not_fatal() {
        let full = make_gguf_header(&[("a", &[2], 0), ("b", &[2], 0)]);
        for cut in [4, 12, 24, 40, full.len() - 4] {
            let map = ModelDiffer::parse_tensors(&full[..cut], &crate::formats::ModelFormat::GGUF);
            assert!(
                map.len() < 2,
                "truncation at {cut} should not yield a full map"
            );
        }
    }

    #[test]
    fn test_gguf_rejects_bad_magic_and_version() {
        let mut bad_magic = make_gguf_header(&[("a", &[2], 0)]);
        bad_magic[0] = b'X';
        assert!(
            ModelDiffer::parse_tensors(&bad_magic, &crate::formats::ModelFormat::GGUF).is_empty()
        );

        let mut bad_version = make_gguf_header(&[("a", &[2], 0)]);
        bad_version[4..8].copy_from_slice(&99u32.to_le_bytes());
        assert!(
            ModelDiffer::parse_tensors(&bad_version, &crate::formats::ModelFormat::GGUF).is_empty()
        );
    }

    #[test]
    fn test_diff_identical() {
        let data = make_safetensors_header(&[
            ("layer.weight", "F32", &[768, 768]),
            ("layer.bias", "F32", &[768]),
        ]);

        let diff = ModelDiffer::diff_bytes(&data, &data, "v1", "v2", "safetensors", "safetensors")
            .unwrap();

        assert_eq!(diff.summary.added, 0);
        assert_eq!(diff.summary.removed, 0);
        assert_eq!(diff.summary.changed, 0);
        assert_eq!(diff.summary.unchanged, 2);
        assert_eq!(diff.size_delta, 0);
    }

    #[test]
    fn test_diff_added_tensor() {
        let left = make_safetensors_header(&[("weight", "F32", &[768, 768])]);
        let right =
            make_safetensors_header(&[("weight", "F32", &[768, 768]), ("bias", "F32", &[768])]);

        let diff = ModelDiffer::diff_bytes(&left, &right, "v1", "v2", "safetensors", "safetensors")
            .unwrap();

        assert_eq!(diff.summary.added, 1);
        assert_eq!(diff.summary.removed, 0);
        assert_eq!(diff.added_tensors[0].name, "bias");
    }

    #[test]
    fn test_diff_removed_tensor() {
        let left =
            make_safetensors_header(&[("weight", "F32", &[768, 768]), ("bias", "F32", &[768])]);
        let right = make_safetensors_header(&[("weight", "F32", &[768, 768])]);

        let diff = ModelDiffer::diff_bytes(&left, &right, "v1", "v2", "safetensors", "safetensors")
            .unwrap();

        assert_eq!(diff.summary.removed, 1);
        assert_eq!(diff.removed_tensors[0].name, "bias");
    }

    #[test]
    fn test_diff_changed_shape() {
        let left = make_safetensors_header(&[("weight", "F32", &[768, 768])]);
        let right = make_safetensors_header(&[("weight", "F32", &[1024, 768])]);

        let diff = ModelDiffer::diff_bytes(&left, &right, "v1", "v2", "safetensors", "safetensors")
            .unwrap();

        assert_eq!(diff.summary.changed, 1);
        assert!(diff.changed_tensors[0].shape_changed);
    }

    #[test]
    fn test_diff_display() {
        let left = make_safetensors_header(&[("weight", "F32", &[768, 768])]);
        let right = make_safetensors_header(&[
            ("weight", "F16", &[1024, 768]),
            ("new_layer", "F32", &[256]),
        ]);

        let diff = ModelDiffer::diff_bytes(
            &left,
            &right,
            "model@v1",
            "model@v2",
            "safetensors",
            "safetensors",
        )
        .unwrap();

        let display = diff.display();
        assert!(display.contains("model@v1"));
        assert!(display.contains("model@v2"));
    }
}
