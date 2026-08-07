"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

type NavSection = {
  title: string;
  items: { label: string; href: string }[];
};

const navigation: NavSection[] = [
  {
    title: "Getting Started",
    items: [
      { label: "Introduction", href: "/docs" },
      { label: "Quick Start", href: "/docs/quickstart" },
      { label: "Installation", href: "/docs/installation" },
      { label: "Architecture", href: "/docs/architecture" },
    ],
  },
  {
    title: "Core Features",
    items: [
      { label: "CLI Reference", href: "/docs/cli" },
      { label: "Format Support", href: "/docs/formats" },
      { label: "Version Control", href: "/docs/version-control" },
      { label: "Model Cards", href: "/docs/model-cards" },
      { label: "Format Conversion", href: "/docs/conversion" },
      { label: "Model Download", href: "/docs/download" },
      { label: "Model Diffing", href: "/docs/diffing" },
      { label: "Benchmarks", href: "/docs/benchmarks" },
      { label: "Quantization", href: "/docs/quantization" },
      { label: "Evaluation", href: "/docs/evaluation" },
      { label: "Backup Scheduling", href: "/docs/backup-scheduling" },
      { label: "Multi-Vault", href: "/docs/multi-vault" },
    ],
  },
  {
    title: "Security",
    items: [
      { label: "Encryption", href: "/docs/security" },
      { label: "Security Hardening", href: "/docs/security-hardening" },
      { label: "Compliance", href: "/docs/compliance" },
      { label: "Model Signing", href: "/docs/signing" },
      { label: "Safety Scanning", href: "/docs/scanning" },
      { label: "License Scanning", href: "/docs/license-scanning" },
    ],
  },
  {
    title: "Integrations",
    items: [
      { label: "REST API", href: "/docs/api" },
      { label: "Python Bindings", href: "/docs/python" },
      { label: "RAG & MCP Tools", href: "/docs/rag" },
      { label: "Cloud Storage", href: "/docs/cloud" },
      { label: "Engine Interop", href: "/docs/engine-interop" },
    ],
  },
  {
    title: "Deployment",
    items: [
      { label: "Migration Guide", href: "/docs/migration" },
    ],
  },
  {
    title: "Utilities",
    items: [
      { label: "Model Utilities", href: "/docs/utilities" },
      { label: "XDG Compliance", href: "/docs/xdg" },
    ],
  },
];

export default function Sidebar() {
  const pathname = usePathname();

  return (
    <aside className="fixed top-[var(--header-height)] left-0 bottom-0 w-[var(--sidebar-width)] overflow-y-auto border-r border-[var(--color-border)] bg-[var(--color-sidebar-bg)] hidden lg:block theme-transition">
      <nav className="p-4 pb-20">
        {navigation.map((section) => (
          <div key={section.title} className="mb-6">
            <h3 className="px-3 mb-1.5 text-xs font-mono font-bold uppercase tracking-[0.2em] text-[var(--color-primary)] opacity-60">
              {section.title}
            </h3>
            <ul className="space-y-0.5">
              {section.items.map((item) => {
                const isActive = pathname === item.href;
                return (
                  <li key={item.href}>
                    <Link
                      href={item.href}
                      className={`block px-3 py-1.5 rounded text-sm font-mono transition-all ${
                        isActive
                          ? "bg-[var(--color-sidebar-active)] text-[var(--color-primary)] font-medium border-l-2 border-[var(--color-primary)]"
                          : "text-[var(--color-text-secondary)] hover:text-[var(--color-text)] hover:bg-[var(--color-sidebar-hover)]"
                      }`}
                    >
                      {item.label}
                    </Link>
                  </li>
                );
              })}
            </ul>
          </div>
        ))}
      </nav>
    </aside>
  );
}

export { navigation };
export type { NavSection };
