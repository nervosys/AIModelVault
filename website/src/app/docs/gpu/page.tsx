import CodeBlock from "@/components/DocElements";
import { Callout } from "@/components/DocElements";

export default function GpuAccelerationPage() {
  return (
    <>
      <h1 className="text-4xl font-bold mb-4">GPU Acceleration</h1>
      <p className="text-lg text-[var(--color-text-secondary)] mb-8">
        AI Model Vault can offload AES-256-GCM encryption and decryption to
        the GPU via OpenCL, delivering significant speedups for large models.
      </p>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="requirements">Requirements</h2>
      <ul className="list-disc pl-6 mb-6 space-y-2 text-[var(--color-text-secondary)]">
        <li>An OpenCL 1.2+ compatible GPU (NVIDIA, AMD, or Intel)</li>
        <li>OpenCL ICD loader installed (<code className="text-xs px-1 bg-[var(--color-bg-secondary)] rounded">ocl-icd-libopencl1</code> on Debian/Ubuntu)</li>
        <li>Vendor-specific OpenCL driver (e.g. <code className="text-xs px-1 bg-[var(--color-bg-secondary)] rounded">nvidia-opencl-icd</code>, <code className="text-xs px-1 bg-[var(--color-bg-secondary)] rounded">mesa-opencl-icd</code>)</li>
      </ul>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="building">Building with GPU Support</h2>
      <CodeBlock language="bash">{`# Build with GPU feature
cargo build --release --features gpu

# Verify GPU detection
aim gpu-info        # lists detected OpenCL platforms/devices`}</CodeBlock>
      <Callout type="info" title="Feature gate">
        All GPU code is behind <code className="text-xs px-1 bg-[var(--color-bg-secondary)] rounded">#[cfg(feature = &quot;gpu&quot;)]</code>.
        Without the flag the binary compiles normally with CPU-only crypto.
      </Callout>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="how-it-works">How It Works</h2>
      <p className="mb-4 text-[var(--color-text-secondary)]">
        The <code className="text-xs px-1 bg-[var(--color-bg-secondary)] rounded">GpuCrypto</code> module
        wraps an OpenCL context with pre-compiled AES-256-GCM kernels. When GPU
        support is enabled, the vault&apos;s encrypt/decrypt pipeline automatically
        routes data based on size:
      </p>
      <div className="overflow-x-auto mb-6">
        <table className="w-full text-sm border-collapse">
          <thead>
            <tr className="border-b border-[var(--color-border)]">
              <th className="text-left py-2 pr-4 font-semibold">Payload Size</th>
              <th className="text-left py-2 pr-4 font-semibold">Route</th>
              <th className="text-left py-2 font-semibold">Reason</th>
            </tr>
          </thead>
          <tbody className="text-[var(--color-text-secondary)]">
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">&le; 10 MB</td><td className="py-2 pr-4">CPU (AES-NI)</td><td className="py-2">PCIe transfer overhead exceeds GPU gain</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">&gt; 10 MB</td><td className="py-2 pr-4">GPU (OpenCL)</td><td className="py-2">Parallel cores dominate at larger sizes</td></tr>
          </tbody>
        </table>
      </div>
      <p className="mb-4 text-[var(--color-text-secondary)]">
        The 10 MB threshold is tuned experimentally and can be overridden in
        the vault configuration.
      </p>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="rust-api">Rust API</h2>
      <CodeBlock language="rust">{`use ai_model_vault::crypto::gpu::GpuCrypto;

// Initialise OpenCL context (picks best device automatically)
let gpu = GpuCrypto::new()?;

// Encrypt
let ciphertext = gpu.encrypt(&plaintext, &key, &nonce)?;

// Decrypt
let plaintext  = gpu.decrypt(&ciphertext, &key, &nonce)?;`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="when-to-use">When to Use GPU Acceleration</h2>
      <div className="overflow-x-auto mb-6">
        <table className="w-full text-sm border-collapse">
          <thead>
            <tr className="border-b border-[var(--color-border)]">
              <th className="text-left py-2 pr-4 font-semibold">Scenario</th>
              <th className="text-left py-2 font-semibold">Recommendation</th>
            </tr>
          </thead>
          <tbody className="text-[var(--color-text-secondary)]">
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">Models &gt; 100 MB (e.g. LLMs)</td><td className="py-2">Yes — substantial throughput improvement</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">Batch encrypt/decrypt workloads</td><td className="py-2">Yes — parallel processing shines</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">Small models (&lt; 10 MB)</td><td className="py-2">No — CPU AES-NI is faster</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">CI / headless servers (no GPU)</td><td className="py-2">No — build without the flag</td></tr>
          </tbody>
        </table>
      </div>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="benchmarking">Benchmarking</h2>
      <CodeBlock language="bash">{`# Run GPU-specific benchmarks
cargo bench --features gpu --bench crypto_bench

# Compare CPU vs GPU for a given file
aim benchmark encrypt ./large-model.safetensors --gpu
aim benchmark encrypt ./large-model.safetensors --cpu`}</CodeBlock>
      <Callout type="tip" title="Tip">
        Always benchmark on your target hardware — GPU gains vary widely
        between integrated graphics and dedicated cards.
      </Callout>
    </>
  );
}
