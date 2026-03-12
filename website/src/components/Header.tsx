"use client";

import Link from "next/link";
import { useState } from "react";
import { navigation } from "./Sidebar";
import { usePathname } from "next/navigation";

export default function Header() {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const pathname = usePathname();

  return (
    <>
      <header className="fixed top-0 left-0 right-0 h-[var(--header-height)] bg-[var(--color-bg)]/95 backdrop-blur border-b border-[var(--color-border)] z-50">
        <div className="h-full flex items-center justify-between px-4 lg:px-6">
          <div className="flex items-center gap-3">
            <button
              className="lg:hidden p-2 rounded-md hover:bg-[var(--color-bg-secondary)]"
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
              aria-label="Toggle menu"
            >
              <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
                {mobileMenuOpen ? (
                  <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
                ) : (
                  <path fillRule="evenodd" d="M3 5a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 10a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 15a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z" clipRule="evenodd" />
                )}
              </svg>
            </button>
            <Link href="/" className="flex items-center gap-2 font-semibold text-lg">
              <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M12 2L2 7l10 5 10-5-10-5z" />
                <path d="M2 17l10 5 10-5" />
                <path d="M2 12l10 5 10-5" />
              </svg>
              <span>AI Model Vault</span>
            </Link>
          </div>

          <nav className="hidden md:flex items-center gap-6 text-sm">
            <Link href="/docs" className="text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">
              Docs
            </Link>
            <a href="/mkdocs/" className="text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">
              Full Docs
            </a>
            <Link href="/demos" className="text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">
              Demos
            </Link>
            <Link href="/docs/api" className="text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">
              API
            </Link>
            <Link href="/docs/quickstart" className="text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">
              Quick Start
            </Link>
            <a
              href="https://github.com/nervosys/AIModelVault"
              target="_blank"
              rel="noopener noreferrer"
              className="text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors"
            >
              GitHub
            </a>
          </nav>

          <div className="flex items-center gap-2">
            <span className="hidden sm:inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">
              v1.2.0
            </span>
            <a
              href="https://crates.io/crates/ai-model-vault"
              target="_blank"
              rel="noopener noreferrer"
              className="hidden sm:inline-flex px-3 py-1.5 rounded-md text-sm font-medium bg-[var(--color-primary)] text-white hover:bg-[var(--color-primary-dark)] transition-colors"
            >
              Install
            </a>
          </div>
        </div>
      </header>

      {/* Mobile menu overlay */}
      {mobileMenuOpen && (
        <div className="fixed inset-0 z-40 lg:hidden">
          <div
            className="fixed inset-0 bg-black/30"
            onClick={() => setMobileMenuOpen(false)}
          />
          <div className="fixed top-[var(--header-height)] left-0 bottom-0 w-72 bg-[var(--color-sidebar-bg)] border-r border-[var(--color-border)] overflow-y-auto z-50">
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
                            onClick={() => setMobileMenuOpen(false)}
                            className={`block px-3 py-1.5 rounded-md text-sm transition-colors ${
                              isActive
                                ? "bg-[var(--color-sidebar-active)] text-[var(--color-primary)] font-medium"
                                : "text-[var(--color-text-secondary)] hover:text-[var(--color-text)]"
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
          </div>
        </div>
      )}
    </>
  );
}
