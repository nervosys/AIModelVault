import Link from "next/link";
import { FeatureCard } from "@/components/DocElements";
import VideoCard from "@/components/VideoCard";

export default function HomePage() {
  return (
    <div className="min-h-[calc(100vh-var(--header-height))]">
      {/* Hero */}
      <section className="relative overflow-hidden bg-gradient-to-br from-slate-900 via-blue-950 to-slate-900 text-white">
        <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjAiIGhlaWdodD0iNjAiIHZpZXdCb3g9IjAgMCA2MCA2MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZyBmaWxsPSJub25lIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxnIGZpbGw9IiNmZmYiIGZpbGwtb3BhY2l0eT0iMC4wMyI+PHBhdGggZD0iTTM2IDE4YzMuMzE0IDAgNi0yLjY4NiA2LTZzLTIuNjg2LTYtNi02LTYgMi42ODYtNiA2IDIuNjg2IDYgNiA2em0wIDM2YzMuMzE0IDAgNi0yLjY4NiA2LTZzLTIuNjg2LTYtNi02LTYgMi42ODYtNiA2IDIuNjg2IDYgNiA2eiIvPjwvZz48L2c+PC9zdmc+')] opacity-50" />
        <div className="relative max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-24 sm:py-32">
          <div className="text-center">
            <div className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-blue-500/20 text-blue-300 border border-blue-500/30 mb-6">
              v1.2.1 — Production Release
            </div>
            <h1 className="text-4xl sm:text-5xl lg:text-6xl font-bold tracking-tight mb-6">
              AI Model Vault
            </h1>
            <p className="text-lg sm:text-xl text-blue-200 max-w-3xl mx-auto mb-8">
              Universal cross-platform secure vault for AI model storage, versioning,
              and management with military-grade encryption and 23+ format support.
            </p>
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <Link
                href="/docs/quickstart"
                className="inline-flex items-center justify-center px-6 py-3 rounded-lg bg-white text-slate-900 font-semibold hover:bg-blue-50 transition-colors"
              >
                Get Started
                <svg className="ml-2 w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" /></svg>
              </Link>
              <Link
                href="/docs"
                className="inline-flex items-center justify-center px-6 py-3 rounded-lg border border-white/20 text-white font-semibold hover:bg-white/10 transition-colors"
              >
                Documentation
              </Link>
            </div>
          </div>

          {/* Install snippet */}
          <div className="mt-12 max-w-xl mx-auto">
            <div className="bg-black/40 backdrop-blur rounded-lg border border-white/10 p-4 font-mono text-sm text-center">
              <span className="text-gray-400">$</span>{" "}
              <span className="text-green-400">cargo install</span>{" "}
              <span className="text-blue-300">ai-model-vault</span>
            </div>
          </div>
        </div>
      </section>

      {/* Features grid */}
      <section className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-20">
        <h2 className="text-3xl font-bold text-center mb-4">Everything you need for AI models</h2>
        <p className="text-center text-[var(--color-text-secondary)] mb-12 max-w-2xl mx-auto">
          From encryption to deployment, AI Model Vault provides a complete toolkit
          for managing models across their entire lifecycle.
        </p>
        <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-6">
          <FeatureCard
            icon="🔐"
            title="FIPS 140-3 Encryption"
            description="AES-256-GCM with Argon2id key derivation. Military-grade protection for your valuable model IP."
            href="/docs/security"
          />
          <FeatureCard
            icon="🎯"
            title="23+ Format Support"
            description="SafeTensors, GGUF, PyTorch, ONNX, TensorRT, Core ML, TFLite, and many more with auto-detection."
            href="/docs/formats"
          />
          <FeatureCard
            icon="🕐"
            title="Version Control"
            description="Git-like versioning with lineage tracking, branching, time travel, and automatic checksums."
            href="/docs/version-control"
          />
          <FeatureCard
            icon="🔄"
            title="Format Conversion"
            description="10 built-in converters with BFS multi-step path finding. Convert between any supported format."
            href="/docs/conversion"
          />
          <FeatureCard
            icon="🌐"
            title="REST API"
            description="14 RESTful endpoints with JWT auth, OpenAPI spec, and an embedded web dashboard."
            href="/docs/api"
          />
          <FeatureCard
            icon="🐍"
            title="Python Bindings"
            description="Native PyO3 bindings — use from Python with full type hints and async support."
            href="/docs/python"
          />
          <FeatureCard
            icon="🐳"
            title="Docker & Kubernetes"
            description="Production-ready Dockerfile and Helm chart with hardened security contexts."
            href="/docs/docker"
          />
          <FeatureCard
            icon="☁️"
            title="Cloud Storage"
            description="Push and pull models to AWS S3 and Azure Blob Storage with end-to-end encryption."
            href="/docs/cloud"
          />
          <FeatureCard
            icon="🤖"
            title="RAG & MCP Tools"
            description="Built-in document store, knowledge base, vector search, and Model Context Protocol agents."
            href="/docs/rag"
          />
        </div>
      </section>

      {/* CLI Demo Videos */}
      <section className="bg-[var(--color-bg-secondary)] border-y border-[var(--color-border)]">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-20">
          <h2 className="text-3xl font-bold text-center mb-4">See It in Action</h2>
          <p className="text-center text-[var(--color-text-secondary)] mb-12 max-w-2xl mx-auto">
            Watch quick CLI demos showing real workflows — from vault initialization
            to security compliance audits.
          </p>
          <div className="grid sm:grid-cols-1 lg:grid-cols-2 gap-8">
            <VideoCard
              src="/videos/CLIInit.mp4"
              title="Initialize a Vault"
              description="Create an encrypted vault with AES-256-GCM, unlock it, and check status."
              duration="0:11"
            />
            <VideoCard
              src="/videos/CLIStore.mp4"
              title="Store & List Models"
              description="Store multiple models with auto-format detection and list vault contents."
              duration="0:14"
            />
            <VideoCard
              src="/videos/CLIVersions.mp4"
              title="Version Control"
              description="Track version history, rollback to previous versions, and view lineage."
              duration="0:16"
            />
            <VideoCard
              src="/videos/CLIConvert.mp4"
              title="Format Conversion"
              description="Convert models between formats with quantization — GGUF, ONNX, and more."
              duration="0:13"
            />
            <VideoCard
              src="/videos/CLICompliance.mp4"
              title="Security Compliance"
              description="Run a full security audit with 12 checks and review the audit log."
              duration="0:11"
            />
          </div>
          <div className="text-center mt-8">
            <Link
              href="/demos"
              className="inline-flex items-center gap-2 px-5 py-2.5 rounded-lg border border-[var(--color-border)] text-sm font-medium hover:border-[var(--color-primary)]/50 hover:text-[var(--color-primary)] transition-colors"
            >
              View All Demos
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" /></svg>
            </Link>
          </div>
        </div>
      </section>

      {/* Quick comparison */}
      <section className="bg-[var(--color-bg-secondary)] border-y border-[var(--color-border)]">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-20">
          <h2 className="text-3xl font-bold text-center mb-12">At a Glance</h2>
          <div className="grid md:grid-cols-3 gap-8 text-center">
            <div>
              <div className="text-4xl font-bold text-[var(--color-primary)] mb-2">1,667</div>
              <div className="text-[var(--color-text-secondary)]">Tests Passing</div>
            </div>
            <div>
              <div className="text-4xl font-bold text-[var(--color-primary)] mb-2">23+</div>
              <div className="text-[var(--color-text-secondary)]">Model Formats</div>
            </div>
            <div>
              <div className="text-4xl font-bold text-[var(--color-primary)] mb-2">14</div>
              <div className="text-[var(--color-text-secondary)]">REST API Endpoints</div>
            </div>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-[var(--color-border)]">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
          <div className="flex flex-col md:flex-row justify-between items-center gap-6">
            <div className="text-sm text-[var(--color-text-secondary)]">
              &copy; 2026 NervoSys. Licensed under AGPL-3.0-or-later.
            </div>
            <div className="flex gap-6 text-sm">
              <a href="https://github.com/nervosys/AIModelVault" className="text-[var(--color-text-secondary)] hover:text-[var(--color-text)]">GitHub</a>
              <a href="https://crates.io/crates/ai-model-vault" className="text-[var(--color-text-secondary)] hover:text-[var(--color-text)]">crates.io</a>
              <a href="https://pypi.org/project/aimodelvault" className="text-[var(--color-text-secondary)] hover:text-[var(--color-text)]">PyPI</a>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}
