import CodeBlock from "@/components/DocElements";

export default function UtilitiesPage() {
  return (
    <>
      <h1 className="text-4xl font-bold mb-4">Utilities</h1>
      <p className="text-lg text-[var(--color-text-secondary)] mb-8">
        Model analysis, caching, archival, and other convenience tools.
      </p>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="analysis">Model Analysis</h2>
      <CodeBlock language="bash">{`# Get model information
aim info my-model

# Show detailed statistics
aim stats

# Verify model integrity
aim verify my-model`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="cache">Cache Management</h2>
      <CodeBlock language="bash">{`# Show cache usage
aim cache info

# Clear cache
aim cache clear

# Set cache size limit (in MB)
aim cache set-limit 1024`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="archive">Model Archival</h2>
      <CodeBlock language="bash">{`# Archive a model (compress and store separately)
aim archive my-model

# List archived models
aim archive list

# Restore from archive
aim archive restore my-model`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="cleanup">Cleanup</h2>
      <CodeBlock language="bash">{`# Remove old versions (keep last N)
aim cleanup --keep-versions 3

# Remove orphaned data
aim cleanup --orphans

# Dry run (show what would be deleted)
aim cleanup --keep-versions 3 --dry-run`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="import-export">Import & Export</h2>
      <CodeBlock language="bash">{`# Export a vault (tar.gz, encrypted)
aim export --output vault-backup.tar.gz

# Import a vault backup
aim import vault-backup.tar.gz --target-vault restored-vault`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="rust-api">Rust API</h2>
      <CodeBlock language="rust">{`use ai_model_vault::utils::{
    ModelAnalyzer, CacheManager, ArchiveManager
};

// Analyze model
let analyzer = ModelAnalyzer::new();
let info = analyzer.analyze(&model_data, "safetensors")?;
println!(
    "Format: {}, Size: {}, Tensors: {}",
    info.format, info.size_formatted, info.tensor_count
);

// Cache management
let cache = CacheManager::new()?;
let usage = cache.usage()?;
println!("Cache: {} / {}", usage.used_formatted, usage.limit_formatted);

// Archive a model
let archive = ArchiveManager::new()?;
archive.archive_model("my-model", &vault)?;`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="env">Environment Variables</h2>
      <div className="overflow-x-auto">
        <table className="w-full text-sm border-collapse">
          <thead>
            <tr className="border-b border-[var(--color-border)]">
              <th className="text-left p-3 font-semibold">Variable</th>
              <th className="text-left p-3 font-semibold">Description</th>
            </tr>
          </thead>
          <tbody className="text-[var(--color-text-secondary)]">
            {[
              ["AIM_VAULT_DIR", "Override default vault directory"],
              ["AIM_LOG_LEVEL", "Logging level (error, warn, info, debug, trace)"],
              ["AIM_CACHE_LIMIT", "Cache size limit in bytes"],
              ["AIM_JWT_SECRET", "JWT signing secret for API server"],
              ["AIM_HOST", "API server bind address"],
              ["AIM_PORT", "API server port"],
            ].map(([name, desc]) => (
              <tr key={name} className="border-b border-[var(--color-border)]">
                <td className="p-3"><code className="px-1.5 py-0.5 bg-[var(--color-bg-secondary)] rounded text-xs">{name}</code></td>
                <td className="p-3">{desc}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  );
}
