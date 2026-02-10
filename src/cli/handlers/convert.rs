//! Convert command handler — model format conversion with guided instructions.

use ai_model_vault::formats::ModelFormat;
use ai_model_vault::{Result, Vault, VaultConfig, VaultError};
use std::path::PathBuf;

use crate::cli::helpers::prompt_passphrase;

pub fn handle_convert(
    name: String,
    to_format_str: String,
    output: Option<PathBuf>,
    version: Option<u32>,
    quantization: Option<String>,
    config: VaultConfig,
) -> Result<()> {
    println!("🔄 Converting model format");
    println!("   Model: {}", name);
    println!("   Target format: {}", to_format_str);

    // Parse target format
    let to_format = match to_format_str.to_lowercase().as_str() {
        "safetensors" => ModelFormat::Safetensors,
        "gguf" => ModelFormat::GGUF,
        "pytorch" | "pt" | "torch" => ModelFormat::PyTorch,
        "onnx" => ModelFormat::ONNX,
        "tensorrt" | "trt" => ModelFormat::TensorRT,
        "tflite" | "tensorflow-lite" => ModelFormat::TFLite,
        "coreml" | "mlmodel" => ModelFormat::CoreML,
        "mlx" => ModelFormat::MLX,
        "torchscript" => ModelFormat::TorchScript,
        "openvino" => ModelFormat::OpenVINO,
        "ncnn" => ModelFormat::NCNN,
        "mnn" => ModelFormat::MNN,
        _ => {
            return Err(VaultError::InvalidInput(format!(
                "Unsupported target format: {}. Supported formats: safetensors, gguf, pytorch, onnx, tensorrt, tflite, coreml, mlx, torchscript, openvino, ncnn, mnn",
                to_format_str
            )));
        }
    };

    // Open vault and get model
    let mut vault = Vault::new(Some(config.clone()))?;
    let passphrase = prompt_passphrase("Enter vault passphrase: ")?;
    vault.unlock(passphrase)?;

    // Get version to convert
    let version_num = if let Some(v) = version {
        v
    } else {
        vault
            .list_versions(&name)
            .last()
            .map(|mv| mv.version)
            .ok_or_else(|| {
                VaultError::ModelNotFound(format!("Model '{}' not found or has no versions", name))
            })?
    };

    // Get model data and metadata
    let data = vault.get_model(&name, Some(version_num))?;
    let versions = vault.list_versions(&name);
    let model_version = versions
        .iter()
        .find(|v| v.version == version_num)
        .ok_or_else(|| VaultError::VersionNotFound(version_num, name.clone()))?;

    // Parse source format from string
    let from_format = ModelFormat::from_extension(&model_version.format);
    println!("   Source format: {}", from_format.name());
    println!("   Source size: {} bytes", data.len());

    // Check if conversion is needed
    if from_format == to_format {
        println!("\n⚠️  Model is already in {} format", to_format.name());
        println!("   No conversion needed!");
        return Ok(());
    }

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let extension = to_format.extension();
        PathBuf::from(format!("{}_converted.{}", name, extension))
    });

    println!("   Output: {}", output_path.display());
    if let Some(quant) = &quantization {
        println!("   Quantization: {}", quant);
    }

    // Show conversion path
    println!(
        "\n🔄 Conversion: {} → {}",
        from_format.name(),
        to_format.name()
    );

    // Model format conversion requires external tools (PyTorch, ONNX Runtime,
    // llama.cpp, etc.) because binary format internals are framework-specific.
    // We export the model from the vault, provide the exact conversion commands,
    // and the user can re-import the converted file.
    //
    // This is the standard approach used by all major ML platforms.

    // Export the raw model for the user to convert
    let export_ext = from_format.extension();
    let export_file = format!("{}_v{}.{}", name, version_num, export_ext);

    // Write the model data out for conversion
    std::fs::write(&export_file, &data)?;
    println!("   Exported: {} ({} bytes)", export_file, data.len());

    println!("\n💡 Conversion steps:");

    // Suggest conversion paths
    match (from_format.name().as_str(), to_format.name().as_str()) {
        ("PyTorch", "Safetensors") => {
            println!("   1. Convert with Python:");
            println!("      from safetensors.torch import save_file");
            println!("      import torch");
            println!("      state_dict = torch.load('{}')", export_file);
            println!("      save_file(state_dict, '{}')", output_path.display());
            println!(
                "   2. Store back: aim store {} {} --format safetensors",
                name,
                output_path.display()
            );
        }
        ("PyTorch", "ONNX") => {
            println!("   1. Convert with torch.onnx.export():");
            println!("      import torch");
            println!("      model = torch.load('{}')", export_file);
            println!(
                "      torch.onnx.export(model, dummy_input, '{}')",
                output_path.display()
            );
            println!(
                "   2. Store back: aim store {} {} --format onnx",
                name,
                output_path.display()
            );
        }
        ("Safetensors", "GGUF") => {
            println!("   1. Use llama.cpp convert.py:");
            if let Some(quant) = &quantization {
                println!(
                    "      python convert.py {} --outtype {} --outfile {}",
                    export_file,
                    quant,
                    output_path.display()
                );
            } else {
                println!(
                    "      python convert.py {} --outfile {}",
                    export_file,
                    output_path.display()
                );
            }
            println!(
                "   2. Store back: aim store {}-gguf {} --format gguf",
                name,
                output_path.display()
            );
        }
        ("ONNX", "TensorRT") => {
            println!("   1. Use TensorRT:");
            println!(
                "      trtexec --onnx={} --saveEngine={} --fp16",
                export_file,
                output_path.display()
            );
            println!(
                "   2. Store back: aim store {} {} --format tensorrt",
                name,
                output_path.display()
            );
        }
        ("PyTorch", "TFLite") => {
            println!("   1. Use ai_edge_torch or ONNX→TFLite path");
            println!(
                "   2. Store back: aim store {} {} --format tflite",
                name,
                output_path.display()
            );
        }
        ("PyTorch", "Core ML") => {
            println!("   1. Use coremltools.convert()");
            println!(
                "   2. Store back: aim store {} {} --format coreml",
                name,
                output_path.display()
            );
        }
        (from, to) => {
            println!("   Conversion path for {} → {} not predefined", from, to);
            println!("   Consider using intermediate formats like ONNX");
            println!("   Common path: {} → ONNX → {}", from, to);
        }
    }

    Ok(())
}
