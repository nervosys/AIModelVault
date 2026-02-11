import CodeBlock from "@/components/DocElements";

export default function CLIPage() {
  return (
    <>
      <h1 className="text-4xl font-bold mb-4">CLI Reference</h1>
      <p className="text-lg text-[var(--color-text-secondary)] mb-8">
        Complete reference for the <code className="px-1.5 py-0.5 bg-[var(--color-bg-secondary)] rounded text-sm">aim</code> command-line tool.
      </p>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="global">Global Options</h2>
      <CodeBlock language="bash">{`aim [OPTIONS] <COMMAND>

Options:
  -v, --vault <NAME>    Vault name (uses default if not specified)
  -c, --config <PATH>   Config file path
  -h, --help            Print help
  -V, --version         Print version`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="vault">Vault Management</h2>

      <h3 className="text-lg font-semibold mt-6 mb-2">init</h3>
      <p className="text-[var(--color-text-secondary)] mb-2">Initialize a new vault.</p>
      <CodeBlock language="bash">{`aim init <NAME>
aim init my-vault`}</CodeBlock>

      <h3 className="text-lg font-semibold mt-6 mb-2">store</h3>
      <p className="text-[var(--color-text-secondary)] mb-2">Store a model in the vault.</p>
      <CodeBlock language="bash">{`aim store <NAME> --format <FORMAT> --file <PATH> [--description <DESC>]
aim store gpt2 --format safetensors --file model.safetensors --description "Fine-tuned GPT-2"`}</CodeBlock>

      <h3 className="text-lg font-semibold mt-6 mb-2">get</h3>
      <p className="text-[var(--color-text-secondary)] mb-2">Retrieve a model from the vault.</p>
      <CodeBlock language="bash">{`aim get <NAME> [--version <N>] [--output <DIR>]
aim get gpt2 --version 2 --output ./models/`}</CodeBlock>

      <h3 className="text-lg font-semibold mt-6 mb-2">list</h3>
      <p className="text-[var(--color-text-secondary)] mb-2">List all models in the vault.</p>
      <CodeBlock language="bash">{`aim list`}</CodeBlock>

      <h3 className="text-lg font-semibold mt-6 mb-2">delete</h3>
      <p className="text-[var(--color-text-secondary)] mb-2">Delete a model or specific version.</p>
      <CodeBlock language="bash">{`aim delete <NAME> [--version <N>]`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="versioning">Version Control</h2>

      <h3 className="text-lg font-semibold mt-6 mb-2">versions</h3>
      <CodeBlock language="bash">{`aim versions <NAME>`}</CodeBlock>

      <h3 className="text-lg font-semibold mt-6 mb-2">lineage</h3>
      <CodeBlock language="bash">{`aim lineage <NAME>`}</CodeBlock>

      <h3 className="text-lg font-semibold mt-6 mb-2">verify</h3>
      <CodeBlock language="bash">{`aim verify <NAME> [--version <N>]`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="conversion">Format Conversion</h2>

      <h3 className="text-lg font-semibold mt-6 mb-2">convert</h3>
      <CodeBlock language="bash">{`aim convert <FILE> --from <FORMAT> --to <FORMAT> [--output <FILE>] [--opset <N>] [--validate] [--plan-only]
aim convert model.safetensors --from safetensors --to pytorch --output model.pt`}</CodeBlock>

      <h3 className="text-lg font-semibold mt-6 mb-2">list-conversions</h3>
      <CodeBlock language="bash">{`aim list-conversions`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="model-cards">Model Cards</h2>

      <h3 className="text-lg font-semibold mt-6 mb-2">card create / show / export / attach</h3>
      <CodeBlock language="bash">{`aim card create <NAME> --author "Team" --task "text-generation"
aim card show <NAME>
aim card export <NAME> --format markdown --output card.md
aim card attach <NAME> --file card.json`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="utilities">Utilities</h2>
      <CodeBlock language="bash">{`aim archive <NAME> --format tar --output backup.tar
aim extract backup.tar --output ./restored/
aim analyze <NAME>
aim info <NAME>`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="cloud">Cloud Storage</h2>
      <CodeBlock language="bash">{`aim cloud config --provider s3 --show
aim cloud push <NAME> --provider s3 --bucket my-bucket
aim cloud pull <NAME> --provider s3 --bucket my-bucket
aim cloud list --provider s3 --bucket my-bucket`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="server">API Server</h2>
      <CodeBlock language="bash">{`aim serve [OPTIONS]

Options:
  --host <HOST>              Listen address (default: 127.0.0.1, env: AIM_HOST)
  --port <PORT>              Listen port (default: 8080, env: AIM_PORT)
  --jwt-secret <SECRET>      JWT signing key (env: AIM_JWT_SECRET)
  --token-expiry <SECONDS>   Token lifetime (default: 3600)
  --cors-permissive          Allow all CORS origins
  --no-dashboard             Disable web dashboard`}</CodeBlock>
    </>
  );
}
