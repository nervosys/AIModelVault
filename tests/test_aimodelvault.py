"""
Python unit tests for the aimodelvault package.

Tests cover:
- ModelFormat detection and enumeration
- VaultConfig initialization and defaults
- Vault class subprocess interface
- FIPSCrypto (standalone, NOT interop with Rust)
- Compression utilities
- Package initialization and version
"""

import os
import tempfile
from pathlib import Path
from unittest.mock import patch, MagicMock

import pytest


# ---------------------------------------------------------------------------
# ModelFormat tests
# ---------------------------------------------------------------------------

class TestModelFormat:
    """Tests for aimodelvault.formats.registry.ModelFormat."""

    def test_detect_safetensors(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("model.safetensors") == ModelFormat.SAFETENSORS

    def test_detect_gguf(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("llama-7b.gguf") == ModelFormat.GGUF

    def test_detect_pytorch_pt(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("weights.pt") == ModelFormat.PYTORCH

    def test_detect_pytorch_pth(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("weights.pth") == ModelFormat.PYTORCH

    def test_detect_pytorch_bin(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("pytorch_model.bin") == ModelFormat.PYTORCH

    def test_detect_onnx(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("model.onnx") == ModelFormat.ONNX

    def test_detect_tflite(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("model.tflite") == ModelFormat.TFLITE

    def test_detect_coreml(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("model.mlmodel") == ModelFormat.COREML

    def test_detect_tensorrt_plan(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("engine.plan") == ModelFormat.TENSORRT

    def test_detect_openvino(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("model.xml") == ModelFormat.OPENVINO

    def test_detect_keras_h5(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("model.h5") == ModelFormat.KERAS

    def test_detect_keras_ext(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("model.keras") == ModelFormat.KERAS

    def test_detect_tensorflow_pb(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("saved_model.pb") == ModelFormat.TENSORFLOW

    def test_detect_pickle(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("model.pkl") == ModelFormat.PICKLE

    def test_detect_numpy(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("weights.npy") == ModelFormat.NUMPY

    def test_detect_hdf5(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("data.hdf5") == ModelFormat.HDF5

    def test_detect_mnn(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("model.mnn") == ModelFormat.MNN

    def test_detect_rknn(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("model.rknn") == ModelFormat.RKNN

    def test_detect_darknet(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("yolov4.weights") == ModelFormat.DARKNET

    def test_detect_caffe(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("model.caffemodel") == ModelFormat.CAFFE

    def test_detect_unknown_returns_custom(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.detect("model.xyz") == ModelFormat.CUSTOM

    def test_detect_case_insensitive(self):
        from aimodelvault.formats.registry import ModelFormat
        # Path().suffix.lower() ensures case insensitivity
        assert ModelFormat.detect("MODEL.SAFETENSORS") == ModelFormat.SAFETENSORS

    def test_file_extensions_pytorch(self):
        from aimodelvault.formats.registry import ModelFormat
        exts = ModelFormat.PYTORCH.file_extensions
        assert ".pt" in exts
        assert ".pth" in exts

    def test_file_extensions_empty_for_custom(self):
        from aimodelvault.formats.registry import ModelFormat
        assert ModelFormat.CUSTOM.file_extensions == []

    def test_str_representation(self):
        from aimodelvault.formats.registry import ModelFormat
        assert str(ModelFormat.SAFETENSORS) == "safetensors"
        assert str(ModelFormat.PYTORCH) == "pytorch"

    def test_all_rust_variants_present(self):
        """Ensure every Rust ModelFormat variant has a Python counterpart."""
        from aimodelvault.formats.registry import ModelFormat
        expected = {
            "SAFETENSORS", "GGUF", "PYTORCH", "TENSORRT", "ONNX", "MLX",
            "COREML", "TORCHSCRIPT", "TFLITE", "TENSORFLOW", "KERAS",
            "OPENVINO", "TVM", "NCNN", "MNN", "RKNN", "CAFFE", "MXNET",
            "DARKNET", "HDF5", "PICKLE", "NUMPY", "CUSTOM",
        }
        actual = {m.name for m in ModelFormat}
        assert expected == actual, f"Missing: {expected - actual}, Extra: {actual - expected}"


# ---------------------------------------------------------------------------
# VaultConfig tests
# ---------------------------------------------------------------------------

class TestVaultConfig:
    """Tests for aimodelvault.core.config.VaultConfig."""

    def test_config_creates_directories(self):
        from aimodelvault.core.config import VaultConfig
        with tempfile.TemporaryDirectory() as tmpdir:
            with patch("aimodelvault.core.config.user_config_dir", return_value=os.path.join(tmpdir, "config")), \
                 patch("aimodelvault.core.config.user_data_dir", return_value=os.path.join(tmpdir, "data")), \
                 patch("aimodelvault.core.config.user_cache_dir", return_value=os.path.join(tmpdir, "cache")):
                cfg = VaultConfig()
                assert cfg.config_dir.exists()
                assert cfg.data_dir.exists()
                assert cfg.cache_dir.exists()

    def test_default_config_values(self):
        from aimodelvault.core.config import VaultConfig
        with tempfile.TemporaryDirectory() as tmpdir:
            with patch("aimodelvault.core.config.user_config_dir", return_value=os.path.join(tmpdir, "config")), \
                 patch("aimodelvault.core.config.user_data_dir", return_value=os.path.join(tmpdir, "data")), \
                 patch("aimodelvault.core.config.user_cache_dir", return_value=os.path.join(tmpdir, "cache")):
                cfg = VaultConfig()
                assert cfg.crypto_algorithm == "aes-256-gcm"
                assert cfg.kdf == "pbkdf2-hmac-sha256"
                assert cfg.kdf_iterations == 600000
                assert cfg.compression_algorithm == "gzip"
                assert cfg.max_versions == 10
                assert cfg.require_passphrase is True
                assert cfg.fips_mode is True

    def test_config_override(self):
        from aimodelvault.core.config import VaultConfig
        with tempfile.TemporaryDirectory() as tmpdir:
            with patch("aimodelvault.core.config.user_config_dir", return_value=os.path.join(tmpdir, "config")), \
                 patch("aimodelvault.core.config.user_data_dir", return_value=os.path.join(tmpdir, "data")), \
                 patch("aimodelvault.core.config.user_cache_dir", return_value=os.path.join(tmpdir, "cache")):
                cfg = VaultConfig(config_override={"custom_key": "custom_value"})
                assert cfg.config["custom_key"] == "custom_value"

    def test_save_and_reload_config(self):
        from aimodelvault.core.config import VaultConfig
        with tempfile.TemporaryDirectory() as tmpdir:
            with patch("aimodelvault.core.config.user_config_dir", return_value=os.path.join(tmpdir, "config")), \
                 patch("aimodelvault.core.config.user_data_dir", return_value=os.path.join(tmpdir, "data")), \
                 patch("aimodelvault.core.config.user_cache_dir", return_value=os.path.join(tmpdir, "cache")):
                cfg1 = VaultConfig()
                cfg1.config["test_marker"] = "present"
                cfg1.save_config()

                cfg2 = VaultConfig()
                assert cfg2.config.get("test_marker") == "present"

    def test_get_vault_path_default(self):
        from aimodelvault.core.config import VaultConfig
        with tempfile.TemporaryDirectory() as tmpdir:
            with patch("aimodelvault.core.config.user_config_dir", return_value=os.path.join(tmpdir, "config")), \
                 patch("aimodelvault.core.config.user_data_dir", return_value=os.path.join(tmpdir, "data")), \
                 patch("aimodelvault.core.config.user_cache_dir", return_value=os.path.join(tmpdir, "cache")):
                cfg = VaultConfig()
                vault_path = cfg.get_vault_path()
                assert "default" in str(vault_path)

    def test_get_vault_path_named(self):
        from aimodelvault.core.config import VaultConfig
        with tempfile.TemporaryDirectory() as tmpdir:
            with patch("aimodelvault.core.config.user_config_dir", return_value=os.path.join(tmpdir, "config")), \
                 patch("aimodelvault.core.config.user_data_dir", return_value=os.path.join(tmpdir, "data")), \
                 patch("aimodelvault.core.config.user_cache_dir", return_value=os.path.join(tmpdir, "cache")):
                cfg = VaultConfig()
                vault_path = cfg.get_vault_path("production")
                assert "production" in str(vault_path)


# ---------------------------------------------------------------------------
# Vault class tests (subprocess mock)
# ---------------------------------------------------------------------------

class TestVault:
    """Tests for aimodelvault.core.vault.Vault subprocess wrapper."""

    def test_vault_init(self):
        from aimodelvault.core.vault import Vault
        with tempfile.TemporaryDirectory() as tmpdir:
            with patch("aimodelvault.core.config.user_config_dir", return_value=os.path.join(tmpdir, "config")), \
                 patch("aimodelvault.core.config.user_data_dir", return_value=os.path.join(tmpdir, "data")), \
                 patch("aimodelvault.core.config.user_cache_dir", return_value=os.path.join(tmpdir, "cache")):
                vault = Vault(os.path.join(tmpdir, "vault"))
                assert vault.vault_path == Path(os.path.join(tmpdir, "vault"))

    def test_vault_list_models_empty(self):
        from aimodelvault.core.vault import Vault
        with tempfile.TemporaryDirectory() as tmpdir:
            with patch("aimodelvault.core.config.user_config_dir", return_value=os.path.join(tmpdir, "config")), \
                 patch("aimodelvault.core.config.user_data_dir", return_value=os.path.join(tmpdir, "data")), \
                 patch("aimodelvault.core.config.user_cache_dir", return_value=os.path.join(tmpdir, "cache")):
                vault = Vault(os.path.join(tmpdir, "vault"))
                # Without a real Rust binary, list should return empty or raise
                with patch("subprocess.run") as mock_run:
                    mock_run.return_value = MagicMock(
                        returncode=0, stdout='["model_a","model_b"]', stderr=""
                    )
                    models = vault.list_models()
                    assert models == ["model_a", "model_b"]


# ---------------------------------------------------------------------------
# FIPSCrypto tests (standalone Python crypto, NOT interop with Rust)
# ---------------------------------------------------------------------------

class TestFIPSCrypto:
    """Tests for aimodelvault.crypto.fips.FIPSCrypto."""

    def test_key_generation(self):
        from aimodelvault.crypto.fips import FIPSCrypto
        crypto = FIPSCrypto()
        key, salt = crypto.generate_key(b"test-passphrase")
        assert len(key) == FIPSCrypto.KEY_SIZE
        assert len(salt) == FIPSCrypto.SALT_SIZE

    def test_key_deterministic_with_salt(self):
        from aimodelvault.crypto.fips import FIPSCrypto
        crypto = FIPSCrypto()
        key1, salt = crypto.generate_key(b"passphrase")
        key2, _ = crypto.generate_key(b"passphrase", salt=salt)
        assert key1 == key2

    def test_different_passwords_different_keys(self):
        from aimodelvault.crypto.fips import FIPSCrypto
        crypto = FIPSCrypto()
        key1, salt = crypto.generate_key(b"password-one")
        key2, _ = crypto.generate_key(b"password-two", salt=salt)
        assert key1 != key2

    def test_encrypt_decrypt_roundtrip(self):
        from aimodelvault.crypto.fips import FIPSCrypto
        crypto = FIPSCrypto()
        key, _ = crypto.generate_key(b"roundtrip-test")
        plaintext = b"Hello, AI Model Vault!"
        ciphertext = crypto.encrypt(plaintext, key)
        assert ciphertext != plaintext
        decrypted = crypto.decrypt(ciphertext, key)
        assert decrypted == plaintext

    def test_encrypt_produces_different_ciphertexts(self):
        from aimodelvault.crypto.fips import FIPSCrypto
        crypto = FIPSCrypto()
        key, _ = crypto.generate_key(b"nonce-test")
        plaintext = b"Same data, different nonces"
        ct1 = crypto.encrypt(plaintext, key)
        ct2 = crypto.encrypt(plaintext, key)
        # Different nonces → different ciphertexts
        assert ct1 != ct2

    def test_decrypt_wrong_key_fails(self):
        from aimodelvault.crypto.fips import FIPSCrypto
        crypto = FIPSCrypto()
        key1, _ = crypto.generate_key(b"correct-password")
        key2, _ = crypto.generate_key(b"wrong-password")
        plaintext = b"Secret model weights"
        ciphertext = crypto.encrypt(plaintext, key1)
        with pytest.raises(Exception):
            crypto.decrypt(ciphertext, key2)

    def test_encrypt_empty_data(self):
        from aimodelvault.crypto.fips import FIPSCrypto
        crypto = FIPSCrypto()
        key, _ = crypto.generate_key(b"empty-test")
        ciphertext = crypto.encrypt(b"", key)
        decrypted = crypto.decrypt(ciphertext, key)
        assert decrypted == b""

    def test_encrypt_large_data(self):
        from aimodelvault.crypto.fips import FIPSCrypto
        crypto = FIPSCrypto()
        key, _ = crypto.generate_key(b"large-test")
        plaintext = os.urandom(1024 * 1024)  # 1 MB
        ciphertext = crypto.encrypt(plaintext, key)
        decrypted = crypto.decrypt(ciphertext, key)
        assert decrypted == plaintext


# ---------------------------------------------------------------------------
# Compression tests
# ---------------------------------------------------------------------------

class TestCompression:
    """Tests for aimodelvault.crypto.compression module."""

    def test_gzip_roundtrip(self):
        from aimodelvault.crypto.compression import GzipCompressor
        c = GzipCompressor()
        data = b"AI Model Vault compression test" * 100
        compressed = c.compress(data)
        assert c.decompress(compressed) == data

    def test_zlib_roundtrip(self):
        from aimodelvault.crypto.compression import ZlibCompressor
        c = ZlibCompressor()
        data = b"Zlib compression test data" * 100
        compressed = c.compress(data)
        assert c.decompress(compressed) == data

    def test_lzma_roundtrip(self):
        from aimodelvault.crypto.compression import LZMACompressor
        c = LZMACompressor()
        data = b"LZMA compression test data" * 100
        compressed = c.compress(data)
        assert c.decompress(compressed) == data

    def test_compression_reduces_size(self):
        from aimodelvault.crypto.compression import GzipCompressor
        c = GzipCompressor()
        data = b"A" * 10_000
        compressed = c.compress(data)
        assert len(compressed) < len(data)

    def test_empty_data_roundtrip(self):
        from aimodelvault.crypto.compression import GzipCompressor
        c = GzipCompressor()
        compressed = c.compress(b"")
        assert c.decompress(compressed) == b""

    def test_get_compressor_gzip(self):
        from aimodelvault.crypto.compression import get_compressor, GzipCompressor
        c = get_compressor("gzip")
        assert isinstance(c, GzipCompressor)

    def test_get_compressor_lzma(self):
        from aimodelvault.crypto.compression import get_compressor, LZMACompressor
        c = get_compressor("lzma")
        assert isinstance(c, LZMACompressor)

    def test_get_compressor_zlib(self):
        from aimodelvault.crypto.compression import get_compressor, ZlibCompressor
        c = get_compressor("zlib")
        assert isinstance(c, ZlibCompressor)

    def test_get_compressor_unknown_raises(self):
        from aimodelvault.crypto.compression import get_compressor
        with pytest.raises(ValueError):
            get_compressor("brotli")

    def test_compression_levels(self):
        from aimodelvault.crypto.compression import GzipCompressor
        c = GzipCompressor()
        data = b"Test data for compression levels" * 500
        fast = c.compress(data, level=1)
        maximum = c.compress(data, level=9)
        # Both should decompress correctly
        assert c.decompress(fast) == data
        assert c.decompress(maximum) == data
        # Maximum compression should be at least as good (usually better)
        assert len(maximum) <= len(fast)


# ---------------------------------------------------------------------------
# Package initialization tests
# ---------------------------------------------------------------------------

class TestPackageInit:
    """Tests for aimodelvault package initialization."""

    def test_version_is_set(self):
        import aimodelvault
        assert aimodelvault.__version__ == "1.2.0"

    def test_native_flag_exists(self):
        import aimodelvault
        assert isinstance(aimodelvault._NATIVE, bool)

    def test_vault_is_importable(self):
        from aimodelvault import Vault
        assert Vault is not None

    def test_vault_config_is_importable(self):
        from aimodelvault import VaultConfig
        assert VaultConfig is not None

    def test_model_format_is_importable(self):
        from aimodelvault import ModelFormat
        assert ModelFormat is not None


# ---------------------------------------------------------------------------
# Vault path and property tests
# ---------------------------------------------------------------------------

class TestVaultProperties:
    """Tests for Vault class properties and initialization."""

    def test_vault_path_property(self):
        from aimodelvault.core.vault import Vault
        with tempfile.TemporaryDirectory() as tmpdir:
            with patch("aimodelvault.core.config.user_config_dir", return_value=os.path.join(tmpdir, "config")), \
                 patch("aimodelvault.core.config.user_data_dir", return_value=os.path.join(tmpdir, "data")), \
                 patch("aimodelvault.core.config.user_cache_dir", return_value=os.path.join(tmpdir, "cache")):
                vault_dir = os.path.join(tmpdir, "test_vault")
                vault = Vault(vault_dir)
                assert vault.path == Path(vault_dir)
                assert vault.path.exists()

    def test_vault_creates_directory(self):
        from aimodelvault.core.vault import Vault
        with tempfile.TemporaryDirectory() as tmpdir:
            with patch("aimodelvault.core.config.user_config_dir", return_value=os.path.join(tmpdir, "config")), \
                 patch("aimodelvault.core.config.user_data_dir", return_value=os.path.join(tmpdir, "data")), \
                 patch("aimodelvault.core.config.user_cache_dir", return_value=os.path.join(tmpdir, "cache")):
                vault_dir = os.path.join(tmpdir, "nested", "vault", "dir")
                vault = Vault(vault_dir)
                assert Path(vault_dir).exists()

    def test_vault_store_calls_aim(self):
        from aimodelvault.core.vault import Vault
        with tempfile.TemporaryDirectory() as tmpdir:
            with patch("aimodelvault.core.config.user_config_dir", return_value=os.path.join(tmpdir, "config")), \
                 patch("aimodelvault.core.config.user_data_dir", return_value=os.path.join(tmpdir, "data")), \
                 patch("aimodelvault.core.config.user_cache_dir", return_value=os.path.join(tmpdir, "cache")):
                vault = Vault(os.path.join(tmpdir, "vault"))
                with patch("subprocess.run") as mock_run:
                    mock_run.return_value = MagicMock(returncode=0, stdout="", stderr="")
                    vault.store("test-model", "/path/to/model.pt",
                                passphrase="secret", description="A test model")
                    mock_run.assert_called_once()
                    args = mock_run.call_args[0][0]
                    assert "store" in args
                    assert "test-model" in args
                    assert "--description" in args

    def test_vault_aim_not_found_raises(self):
        from aimodelvault.core.vault import Vault
        with tempfile.TemporaryDirectory() as tmpdir:
            with patch("aimodelvault.core.config.user_config_dir", return_value=os.path.join(tmpdir, "config")), \
                 patch("aimodelvault.core.config.user_data_dir", return_value=os.path.join(tmpdir, "data")), \
                 patch("aimodelvault.core.config.user_cache_dir", return_value=os.path.join(tmpdir, "cache")):
                vault = Vault(os.path.join(tmpdir, "vault"))
                with patch("subprocess.run", side_effect=FileNotFoundError):
                    with pytest.raises(FileNotFoundError, match="aim"):
                        vault.list_models()

    def test_vault_aim_error_raises_runtime(self):
        from aimodelvault.core.vault import Vault
        with tempfile.TemporaryDirectory() as tmpdir:
            with patch("aimodelvault.core.config.user_config_dir", return_value=os.path.join(tmpdir, "config")), \
                 patch("aimodelvault.core.config.user_data_dir", return_value=os.path.join(tmpdir, "data")), \
                 patch("aimodelvault.core.config.user_cache_dir", return_value=os.path.join(tmpdir, "cache")):
                vault = Vault(os.path.join(tmpdir, "vault"))
                with patch("subprocess.run") as mock_run:
                    mock_run.return_value = MagicMock(
                        returncode=1, stdout="", stderr="error: vault not found"
                    )
                    with pytest.raises(RuntimeError, match="aim command failed"):
                        vault.list_models()
