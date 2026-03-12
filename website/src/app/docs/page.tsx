import Link from "next/link";
import { FeatureCard } from "@/components/DocElements";

export default function DocsIndex() {
  return (
    <>
      <h1 className="text-4xl font-bold mb-4">Documentation</h1>
      <p className="text-lg text-[var(--color-text-secondary)] mb-8">
        Welcome to AI Model Vault — a universal, cross-platform, FIPS 140-3 compliant
        secure vault for AI model storage, versioning, and management.
      </p>

      <div className="grid sm:grid-cols-2 gap-4 mb-12">
        <FeatureCard icon="🚀" title="Quick Start" description="Get up and running in 5 minutes with installation and basic usage." href="/docs/quickstart" />
        <FeatureCard icon="💻" title="CLI Reference" description="Complete reference for all aim commands and options." href="/docs/cli" />
        <FeatureCard icon="🔐" title="Security" description="Encryption, key derivation, and FIPS 140-3 compliance details." href="/docs/security" />
        <FeatureCard icon="🌐" title="REST API" description="14 RESTful endpoints with JWT auth and OpenAPI specification." href="/docs/api" />
      </div>

      <h2 className="text-2xl font-bold mb-4">What is AI Model Vault?</h2>
      <p className="text-[var(--color-text-secondary)] mb-4">
        AI Model Vault (<code className="px-1.5 py-0.5 bg-[var(--color-bg-secondary)] rounded text-sm">aim</code>) is
        a production-ready tool for storing, versioning, and protecting AI models. It supports 23+ model
        formats, provides military-grade AES-256-GCM encryption, and works across Windows, Linux, and macOS.
      </p>

      <h3 className="text-xl font-semibold mt-8 mb-3">Key Capabilities</h3>
      <ul className="space-y-2 text-[var(--color-text-secondary)]">
        <li className="flex items-start gap-2"><span>•</span> <span><strong>Secure Storage</strong> — AES-256-GCM encryption with Argon2id key derivation (FIPS 140-3)</span></li>
        <li className="flex items-start gap-2"><span>•</span> <span><strong>Universal Formats</strong> — SafeTensors, GGUF, PyTorch, ONNX, TensorRT, Core ML, and 17+ more</span></li>
        <li className="flex items-start gap-2"><span>•</span> <span><strong>Version Control</strong> — Git-like versioning with lineage, branching, and time travel</span></li>
        <li className="flex items-start gap-2"><span>•</span> <span><strong>Format Conversion</strong> — Pipeline-based conversion with 10 built-in converters and BFS path finding</span></li>
        <li className="flex items-start gap-2"><span>•</span> <span><strong>REST API</strong> — 14 endpoints with JWT auth, embedded web dashboard, and OpenAPI spec</span></li>
        <li className="flex items-start gap-2"><span>•</span> <span><strong>Python Bindings</strong> — Native PyO3 bindings for seamless Python integration</span></li>
        <li className="flex items-start gap-2"><span>•</span> <span><strong>Cloud Storage</strong> — AWS S3 and Azure Blob Storage with end-to-end encryption</span></li>
        <li className="flex items-start gap-2"><span>•</span> <span><strong>Docker & K8s</strong> — Multi-stage Dockerfile and production Helm chart</span></li>
      </ul>

      <h3 className="text-xl font-semibold mt-8 mb-3">Get Help</h3>
      <div className="flex gap-4 flex-wrap">
        <a href="https://github.com/nervosys/AIModelVault/issues" target="_blank" rel="noopener noreferrer"
          className="inline-flex items-center px-4 py-2 rounded-lg border border-[var(--color-border)] text-sm hover:border-[var(--color-primary)] transition-colors">
          Report an Issue
        </a>
        <a href="https://github.com/nervosys/AIModelVault/discussions" target="_blank" rel="noopener noreferrer"
          className="inline-flex items-center px-4 py-2 rounded-lg border border-[var(--color-border)] text-sm hover:border-[var(--color-primary)] transition-colors">
          Discussions
        </a>
      </div>
    </>
  );
}
