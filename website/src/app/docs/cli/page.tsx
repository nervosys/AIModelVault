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

      <h2 className="text-2xl font-bold mt-10 mb-4" id="download">Model Download</h2>

      <h3 className="text-lg font-semibold mt-6 mb-2">pull</h3>
      <p className="text-[var(--color-text-secondary)] mb-2">Download models from HuggingFace Hub, Ollama registry, or URLs.</p>
      <CodeBlock language="bash">{`aim pull <SOURCE> [-o DIR] [--sha256 HASH] [--token TOKEN] [--store] [--name NAME]

# HuggingFace
aim pull hf://TheBloke/Llama-2-7B-GGUF/llama-2-7b.Q4_K_M.gguf

# Ollama
aim pull ollama://llama2:7b

# URL with checksum verification
aim pull https://example.com/model.safetensors --sha256 abc123...

# Download and auto-store in vault
aim pull hf://user/repo/model.safetensors --store --name my-model`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="signing">Model Signing</h2>

      <h3 className="text-lg font-semibold mt-6 mb-2">sign</h3>
      <p className="text-[var(--color-text-secondary)] mb-2">Sign a model with HMAC-SHA256. Auto-generates key on first use.</p>
      <CodeBlock language="bash">{`aim sign <NAME> [-v VERSION] [-k KEY] [-i IDENTITY] [--file PATH]
aim sign my-model --identity "ML Team <ml@company.com>"`}</CodeBlock>

      <h3 className="text-lg font-semibold mt-6 mb-2">verify</h3>
      <p className="text-[var(--color-text-secondary)] mb-2">Verify a model signature.</p>
      <CodeBlock language="bash">{`aim verify <NAME> --signature <SIG> [-k KEY] [--file PATH]
aim verify my-model --signature my-model.sig`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="scanning">Safety Scanning</h2>

      <h3 className="text-lg font-semibold mt-6 mb-2">scan</h3>
      <p className="text-[var(--color-text-secondary)] mb-2">Scan PyTorch/pickle files for dangerous opcodes and malicious patterns.</p>
      <CodeBlock language="bash">{`aim scan [<NAME>] [--file PATH] [-v VERSION] [-f text|json]

# Scan a vault model
aim scan my-model

# Scan a file on disk
aim scan --file ./model.pt --format json`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="diffing">Model Diffing</h2>

      <h3 className="text-lg font-semibold mt-6 mb-2">diff</h3>
      <p className="text-[var(--color-text-secondary)] mb-2">Compare two models at the tensor level (SafeTensors, GGUF).</p>
      <CodeBlock language="bash">{`aim diff <LEFT> <RIGHT> [-f text|json]

# Compare files
aim diff model_v1.safetensors model_v2.safetensors

# Compare vault versions
aim diff mymodel@v1 mymodel@v2`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="interop">Engine Interop</h2>

      <h3 className="text-lg font-semibold mt-6 mb-2">register</h3>
      <p className="text-[var(--color-text-secondary)] mb-2">Register a model with Ollama or LM Studio.</p>
      <CodeBlock language="bash">{`aim register <NAME> --engine <ollama|lm-studio> [-v VERSION] [--alias NAME] [--system-prompt TEXT]

aim register my-model --engine ollama --alias my-assistant --system-prompt "You are helpful."
aim register my-model --engine lm-studio`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="benchmarks">Benchmarks</h2>

      <h3 className="text-lg font-semibold mt-6 mb-2">benchmark add</h3>
      <p className="text-[var(--color-text-secondary)] mb-2">Attach benchmark scores to model versions.</p>
      <CodeBlock language="bash">{`aim benchmark add <NAME> --version V --benchmark <BENCH> --score <N> --unit <UNIT> [--higher-is-better]
aim benchmark add my-model --version 1 --benchmark mmlu --score 72.5 --unit percent --higher-is-better`}</CodeBlock>

      <h3 className="text-lg font-semibold mt-6 mb-2">benchmark show</h3>
      <p className="text-[var(--color-text-secondary)] mb-2">Display benchmark results for a model.</p>
      <CodeBlock language="bash">{`aim benchmark show <NAME> [--version V] [-f text|json]
aim benchmark show my-model --version 1 --format json`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="license">License Scanning</h2>

      <h3 className="text-lg font-semibold mt-6 mb-2">license-scan</h3>
      <p className="text-[var(--color-text-secondary)] mb-2">Detect licenses from model cards, GGUF metadata, and config files.</p>
      <CodeBlock language="bash">{`aim license-scan <PATH> [-f text|json]

aim license-scan ./my-model/
aim license-scan model.gguf --format json`}</CodeBlock>
    </>
  );
}
