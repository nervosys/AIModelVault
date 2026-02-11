import CodeBlock from "@/components/DocElements";
import { Callout } from "@/components/DocElements";

export default function DockerPage() {
  return (
    <>
      <h1 className="text-4xl font-bold mb-4">Docker</h1>
      <p className="text-lg text-[var(--color-text-secondary)] mb-8">
        Container deployment with multi-stage Docker builds, Alpine and Debian variants.
      </p>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="images">Available Images</h2>
      <div className="overflow-x-auto">
        <table className="w-full text-sm border-collapse">
          <thead>
            <tr className="border-b border-[var(--color-border)]">
              <th className="text-left p-3 font-semibold">Image</th>
              <th className="text-left p-3 font-semibold">Base</th>
              <th className="text-left p-3 font-semibold">Size</th>
              <th className="text-left p-3 font-semibold">Use Case</th>
            </tr>
          </thead>
          <tbody className="text-[var(--color-text-secondary)]">
            <tr className="border-b border-[var(--color-border)]">
              <td className="p-3"><code className="text-xs">ai-model-vault:latest</code></td>
              <td className="p-3">Alpine 3.19</td>
              <td className="p-3">~12 MB</td>
              <td className="p-3">Minimal — CLI only</td>
            </tr>
            <tr className="border-b border-[var(--color-border)]">
              <td className="p-3"><code className="text-xs">ai-model-vault:debian</code></td>
              <td className="p-3">Debian slim</td>
              <td className="p-3">~80 MB</td>
              <td className="p-3">Broader compatibility</td>
            </tr>
            <tr className="border-b border-[var(--color-border)]">
              <td className="p-3"><code className="text-xs">ai-model-vault:api</code></td>
              <td className="p-3">Alpine 3.19</td>
              <td className="p-3">~15 MB</td>
              <td className="p-3">REST API server</td>
            </tr>
          </tbody>
        </table>
      </div>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="quickstart">Quick Start</h2>
      <CodeBlock language="bash">{`# Pull from GitHub Container Registry
docker pull ghcr.io/nervosys/ai-model-vault:latest

# Run with a vault volume
docker run -v vault-data:/data/vault \\
  ghcr.io/nervosys/ai-model-vault:latest \\
  aim list`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="api-server">Run API Server</h2>
      <CodeBlock language="bash">{`docker run -d \\
  --name aim-server \\
  -p 8080:8080 \\
  -v vault-data:/data/vault \\
  -e AIM_JWT_SECRET=your-secret \\
  ghcr.io/nervosys/ai-model-vault:api \\
  aim serve --host 0.0.0.0 --port 8080`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="volumes">Volumes</h2>
      <p className="text-[var(--color-text-secondary)] mb-4">XDG-compliant paths inside the container:</p>
      <ul className="space-y-1 text-[var(--color-text-secondary)]">
        <li>• <code className="text-xs px-1 bg-[var(--color-bg-secondary)] rounded">/config</code> — Configuration (<code className="text-xs">$XDG_CONFIG_HOME</code>)</li>
        <li>• <code className="text-xs px-1 bg-[var(--color-bg-secondary)] rounded">/data</code> — Data and vaults (<code className="text-xs">$XDG_DATA_HOME</code>)</li>
        <li>• <code className="text-xs px-1 bg-[var(--color-bg-secondary)] rounded">/cache</code> — Cache files (<code className="text-xs">$XDG_CACHE_HOME</code>)</li>
      </ul>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="build">Build from Source</h2>
      <CodeBlock language="bash">{`# Default Alpine image (CLI only)
docker build -t ai-model-vault .

# With API feature
docker build --build-arg FEATURES=api -t ai-model-vault:api .

# Debian variant
docker build --target runtime-debian -t ai-model-vault:debian .`}</CodeBlock>

      <Callout type="info" title="Non-root execution">
        All images run as a non-root user (UID 1000) by default with <code className="text-xs">tini</code> as
        the init system for proper signal handling.
      </Callout>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="compose">Docker Compose</h2>
      <CodeBlock language="yaml" title="docker-compose.yml">{`services:
  vault:
    image: ghcr.io/nervosys/ai-model-vault:api
    ports:
      - "8080:8080"
    volumes:
      - vault-data:/data/vault
      - vault-config:/config
    environment:
      AIM_JWT_SECRET: \${AIM_JWT_SECRET}
    restart: unless-stopped

volumes:
  vault-data:
  vault-config:`}</CodeBlock>
    </>
  );
}
