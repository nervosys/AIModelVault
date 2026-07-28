import CodeBlock from "@/components/DocElements";

export default function InstallationPage() {
  return (
    <>
      <h1 className="text-4xl font-bold mb-4">Installation</h1>
      <p className="text-lg text-[var(--color-text-secondary)] mb-8">
        Multiple ways to install AI Model Vault depending on your workflow.
      </p>

      <h2 className="text-2xl font-bold mt-8 mb-4" id="requirements">Requirements</h2>
      <ul className="list-disc list-inside space-y-1 text-[var(--color-text-secondary)] mb-6">
        <li>Rust 1.75+ (for building from source)</li>
        <li>Python 3.9+ (for Python bindings)</li>
        <li>Docker (for container deployment)</li>
      </ul>

      <h2 className="text-2xl font-bold mt-8 mb-4" id="cargo">Via Cargo (Recommended)</h2>
      <CodeBlock language="bash">{`cargo install ai-model-vault`}</CodeBlock>
      <p className="text-[var(--color-text-secondary)] mt-2 mb-4">
        This installs the <code className="px-1.5 py-0.5 bg-[var(--color-bg-secondary)] rounded text-sm">aim</code> binary
        with default features (SafeTensors, ndarray, SQLite).
      </p>

      <h3 className="text-lg font-semibold mt-6 mb-2">With API server</h3>
      <CodeBlock language="bash">{`cargo install ai-model-vault --features api`}</CodeBlock>

      <h3 className="text-lg font-semibold mt-6 mb-2">With all features</h3>
      <CodeBlock language="bash">{`cargo install ai-model-vault --features full`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-8 mb-4" id="source">From Source</h2>
      <CodeBlock language="bash">{`git clone https://github.com/nervosys/AIModelVault.git
cd AIModelVault
cargo build --release

# Run directly
./target/release/aim --help

# Or install to PATH
cargo install --path .`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-8 mb-4" id="python">Python Package</h2>
      <CodeBlock language="bash">{`# Base package
pip install aimodelvault

# With ML frameworks
pip install "aimodelvault[ml]"

# With development tools
pip install "aimodelvault[dev]"

# With security auditing
pip install "aimodelvault[security]"`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-8 mb-4" id="docker">Docker</h2>
      <CodeBlock language="bash">{`# Alpine (smallest, ~12 MB)
docker pull ghcr.io/nervosys/ai-model-vault:1.1.0

# Debian
docker pull ghcr.io/nervosys/ai-model-vault:1.1.0-debian

# API variant
docker pull ghcr.io/nervosys/ai-model-vault:1.1.0-api`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-8 mb-4" id="features">Feature Flags</h2>
      <div className="overflow-x-auto">
        <table className="w-full text-sm border-collapse">
          <thead>
            <tr className="border-b border-[var(--color-border)]">
              <th className="text-left p-3 font-semibold">Feature</th>
              <th className="text-left p-3 font-semibold">Default</th>
              <th className="text-left p-3 font-semibold">Description</th>
            </tr>
          </thead>
          <tbody className="text-[var(--color-text-secondary)]">
            {[
              ["default", "Yes", "SafeTensors, ndarray, SQLite"],
              ["api", "No", "REST API server with axum"],
              ["python", "No", "PyO3 Python bindings"],
              ["cloud", "No", "AWS S3 & Azure Blob Storage"],
              ["full", "No", "All features combined"],
              ["sqlite", "Yes", "SQLite database backend"],
            ].map(([feature, def, desc]) => (
              <tr key={feature} className="border-b border-[var(--color-border)]">
                <td className="p-3"><code className="px-1.5 py-0.5 bg-[var(--color-bg-secondary)] rounded text-xs">{feature}</code></td>
                <td className="p-3">{def}</td>
                <td className="p-3">{desc}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <h2 className="text-2xl font-bold mt-8 mb-4" id="verify">Verify Installation</h2>
      <CodeBlock language="bash">{`aim --version
# aim 1.0.0

aim --help`}</CodeBlock>
    </>
  );
}
