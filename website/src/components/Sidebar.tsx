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
    ],
  },
  {
    title: "Security",
    items: [
      { label: "Encryption", href: "/docs/security" },
      { label: "Security Hardening", href: "/docs/security-hardening" },
      { label: "Compliance", href: "/docs/compliance" },
    ],
  },
  {
    title: "Integrations",
    items: [
      { label: "REST API", href: "/docs/api" },
      { label: "Python Bindings", href: "/docs/python" },
      { label: "RAG & MCP Tools", href: "/docs/rag" },
      { label: "Cloud Storage", href: "/docs/cloud" },
    ],
  },
  {
    title: "Deployment",
    items: [
      { label: "Docker", href: "/docs/docker" },
      { label: "Kubernetes", href: "/docs/kubernetes" },
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
    <aside className="fixed top-[var(--header-height)] left-0 bottom-0 w-[var(--sidebar-width)] overflow-y-auto border-r border-[var(--color-border)] bg-[var(--color-sidebar-bg)] hidden lg:block">
      <nav className="p-4 pb-20">
        {navigation.map((section) => (
          <div key={section.title} className="mb-6">
            <h3 className="px-3 mb-1 text-xs font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">
              {section.title}
            </h3>
            <ul className="space-y-0.5">
              {section.items.map((item) => {
                const isActive = pathname === item.href;
                return (
                  <li key={item.href}>
                    <Link
                      href={item.href}
                      className={`block px-3 py-1.5 rounded-md text-sm transition-colors ${
                        isActive
                          ? "bg-[var(--color-sidebar-active)] text-[var(--color-primary)] font-medium"
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
