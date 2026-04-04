import CodeBlock from "@/components/DocElements";
import { Callout } from "@/components/DocElements";

export default function PerformancePage() {
  return (
    <>
      <h1 className="text-4xl font-bold mb-4">Performance</h1>
      <p className="text-lg text-[var(--color-text-secondary)] mb-8">
        Benchmark results captured with Criterion 0.5 on Windows x86_64.
        All benchmarks use the default (non-GPU) code path in release mode.
      </p>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="encryption">AES-256-GCM Encryption</h2>
      <div className="overflow-x-auto mb-6">
        <table className="w-full text-sm border-collapse">
          <thead>
            <tr className="border-b border-[var(--color-border)]">
              <th className="text-left py-2 pr-4 font-semibold">Data Size</th>
              <th className="text-left py-2 pr-4 font-semibold">Encrypt</th>
              <th className="text-left py-2 pr-4 font-semibold">Decrypt</th>
              <th className="text-left py-2 font-semibold">Throughput (enc)</th>
            </tr>
          </thead>
          <tbody className="text-[var(--color-text-secondary)]">
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4 font-mono">1 KB</td><td className="py-2 pr-4">~1.03 us</td><td className="py-2 pr-4">~1.8 us</td><td className="py-2">~1.0 GB/s</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4 font-mono">10 KB</td><td className="py-2 pr-4">~8.0 us</td><td className="py-2 pr-4">~11.6 us</td><td className="py-2">~1.3 GB/s</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4 font-mono">100 KB</td><td className="py-2 pr-4">~89.6 us</td><td className="py-2 pr-4">~109 us</td><td className="py-2">~1.1 GB/s</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4 font-mono">1 MB</td><td className="py-2 pr-4">~3.5 ms</td><td className="py-2 pr-4">~2.7 ms</td><td className="py-2">~300 MB/s</td></tr>
          </tbody>
        </table>
      </div>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="key-derivation">Key Derivation (Argon2id)</h2>
      <div className="overflow-x-auto mb-4">
        <table className="w-full text-sm border-collapse">
          <thead>
            <tr className="border-b border-[var(--color-border)]">
              <th className="text-left py-2 pr-4 font-semibold">Operation</th>
              <th className="text-left py-2 font-semibold">Time (median)</th>
            </tr>
          </thead>
          <tbody className="text-[var(--color-text-secondary)]">
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">Key derivation</td><td className="py-2">~353 ms</td></tr>
          </tbody>
        </table>
      </div>
      <Callout type="info" title="By design">
        Argon2id is intentionally slow (64 MB memory, 3 iterations) to resist brute-force attacks.
        This dominates store/retrieve latency for small files.
      </Callout>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="compression">Compression</h2>
      <div className="overflow-x-auto mb-6">
        <table className="w-full text-sm border-collapse">
          <thead>
            <tr className="border-b border-[var(--color-border)]">
              <th className="text-left py-2 pr-4 font-semibold">Algorithm</th>
              <th className="text-left py-2 pr-4 font-semibold">1 KB</th>
              <th className="text-left py-2 pr-4 font-semibold">10 KB</th>
              <th className="text-left py-2 font-semibold">100 KB</th>
            </tr>
          </thead>
          <tbody className="text-[var(--color-text-secondary)]">
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">gzip</td><td className="py-2 pr-4">~7.7 us</td><td className="py-2 pr-4">~9.7 us</td><td className="py-2">~28.4 us</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">LZMA</td><td className="py-2 pr-4">~26.2 us</td><td className="py-2 pr-4">~388 us</td><td className="py-2">~1.99 ms</td></tr>
          </tbody>
        </table>
      </div>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="vault-ops">Vault Operations</h2>
      <div className="overflow-x-auto mb-6">
        <table className="w-full text-sm border-collapse">
          <thead>
            <tr className="border-b border-[var(--color-border)]">
              <th className="text-left py-2 pr-4 font-semibold">Operation</th>
              <th className="text-left py-2 pr-4 font-semibold">1 KB</th>
              <th className="text-left py-2 pr-4 font-semibold">10 KB</th>
              <th className="text-left py-2 font-semibold">100 KB</th>
            </tr>
          </thead>
          <tbody className="text-[var(--color-text-secondary)]">
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">Store</td><td className="py-2 pr-4">~46 ms</td><td className="py-2 pr-4">~49 ms</td><td className="py-2">~45 ms</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">Retrieve</td><td className="py-2 pr-4">~25 ms</td><td className="py-2 pr-4">~18 ms</td><td className="py-2">~16 ms</td></tr>
          </tbody>
        </table>
      </div>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="format-detection">Format Detection</h2>
      <div className="overflow-x-auto mb-6">
        <table className="w-full text-sm border-collapse">
          <thead>
            <tr className="border-b border-[var(--color-border)]">
              <th className="text-left py-2 pr-4 font-semibold">Operation</th>
              <th className="text-left py-2 font-semibold">Time (median)</th>
            </tr>
          </thead>
          <tbody className="text-[var(--color-text-secondary)]">
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4"><code className="text-xs px-1 bg-[var(--color-bg-secondary)] rounded">from_extension()</code></td><td className="py-2">~462 ns</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4"><code className="text-xs px-1 bg-[var(--color-bg-secondary)] rounded">format_name()</code></td><td className="py-2">~4 ns</td></tr>
          </tbody>
        </table>
      </div>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="hashing">SHA-256 Hashing</h2>
      <div className="overflow-x-auto mb-6">
        <table className="w-full text-sm border-collapse">
          <thead>
            <tr className="border-b border-[var(--color-border)]">
              <th className="text-left py-2 pr-4 font-semibold">Data Size</th>
              <th className="text-left py-2 font-semibold">Time (median)</th>
            </tr>
          </thead>
          <tbody className="text-[var(--color-text-secondary)]">
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4 font-mono">1 KB</td><td className="py-2">~453 ns</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4 font-mono">10 KB</td><td className="py-2">~5.3 us</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4 font-mono">100 KB</td><td className="py-2">~53 us</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4 font-mono">1 MB</td><td className="py-2">~571 us</td></tr>
          </tbody>
        </table>
      </div>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="model-cards">Model Card Serialization</h2>
      <div className="overflow-x-auto mb-6">
        <table className="w-full text-sm border-collapse">
          <thead>
            <tr className="border-b border-[var(--color-border)]">
              <th className="text-left py-2 pr-4 font-semibold">Operation</th>
              <th className="text-left py-2 font-semibold">Time (median)</th>
            </tr>
          </thead>
          <tbody className="text-[var(--color-text-secondary)]">
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">to_json()</td><td className="py-2">~4.4 us</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">to_yaml()</td><td className="py-2">~22 us</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">to_markdown()</td><td className="py-2">~1.5 us</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">from_json()</td><td className="py-2">~4.5 us</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">from_yaml()</td><td className="py-2">~59 us</td></tr>
          </tbody>
        </table>
      </div>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="api-endpoints">REST API Endpoints</h2>
      <div className="overflow-x-auto mb-4">
        <table className="w-full text-sm border-collapse">
          <thead>
            <tr className="border-b border-[var(--color-border)]">
              <th className="text-left py-2 pr-4 font-semibold">Endpoint</th>
              <th className="text-left py-2 pr-4 font-semibold">Method</th>
              <th className="text-left py-2 font-semibold">Time (median)</th>
            </tr>
          </thead>
          <tbody className="text-[var(--color-text-secondary)]">
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4 font-mono">/api/v1/health</td><td className="py-2 pr-4">GET</td><td className="py-2">~90 ms</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4 font-mono">/api/v1/auth/token</td><td className="py-2 pr-4">POST</td><td className="py-2">~311 ms</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4 font-mono">/api/v1/models</td><td className="py-2 pr-4">GET</td><td className="py-2">~195 ms</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4 font-mono">/api/v1/compliance</td><td className="py-2 pr-4">GET</td><td className="py-2">~1.55 s</td></tr>
          </tbody>
        </table>
      </div>
      <Callout type="info" title="Benchmark note">
        API benchmarks include per-request vault setup (tempdir + init). The compliance
        endpoint runs cargo audit externally, which dominates its latency.
      </Callout>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="coverage">Code Coverage</h2>
      <div className="overflow-x-auto mb-6">
        <table className="w-full text-sm border-collapse">
          <thead>
            <tr className="border-b border-[var(--color-border)]">
              <th className="text-left py-2 pr-4 font-semibold">Metric</th>
              <th className="text-left py-2 font-semibold">Value</th>
            </tr>
          </thead>
          <tbody className="text-[var(--color-text-secondary)]">
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">Tool</td><td className="py-2">cargo-llvm-cov</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">Lines</td><td className="py-2">15,187 / 17,786</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">Coverage</td><td className="py-2 font-semibold">85.4%</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">Lib tests</td><td className="py-2">623</td></tr>
            <tr className="border-b border-[var(--color-border)]"><td className="py-2 pr-4">Total tests</td><td className="py-2">1,831 (lib + integration)</td></tr>
          </tbody>
        </table>
      </div>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="running-benchmarks">Running Benchmarks</h2>
      <CodeBlock language="bash">{`# All benchmarks
cargo bench

# Crypto only
cargo bench --bench crypto_bench

# Vault operations
cargo bench --bench vault_bench

# API benchmarks (requires api feature)
cargo bench --bench api_bench --features api`}</CodeBlock>

      <p className="text-sm text-[var(--color-text-secondary)] mt-4">
        Results are stored in <code className="text-xs px-1 bg-[var(--color-bg-secondary)] rounded">target/criterion/</code> with
        HTML reports at <code className="text-xs px-1 bg-[var(--color-bg-secondary)] rounded">target/criterion/report/index.html</code>.
      </p>
    </>
  );
}
