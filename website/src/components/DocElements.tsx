import { ReactNode } from "react";

type CodeBlockProps = {
  language?: string;
  title?: string;
  children: ReactNode;
};

export default function CodeBlock({ language = "bash", title, children }: CodeBlockProps) {
  return (
    <div className="my-4 rounded-lg overflow-hidden border border-[var(--color-border)]">
      {title && (
        <div className="px-4 py-2 bg-[var(--color-bg-secondary)] border-b border-[var(--color-border)] text-xs font-medium text-[var(--color-text-secondary)]">
          {title}
        </div>
      )}
      <div className="bg-[var(--color-bg-code)] p-4 overflow-x-auto">
        <pre className="text-sm text-gray-100">
          <code className={`language-${language}`}>{children}</code>
        </pre>
      </div>
    </div>
  );
}

type CalloutProps = {
  type?: "info" | "warning" | "tip" | "danger";
  title?: string;
  children: ReactNode;
};

const calloutStyles = {
  info: "border-blue-500 bg-blue-50 dark:bg-blue-950/30",
  warning: "border-amber-500 bg-amber-50 dark:bg-amber-950/30",
  tip: "border-emerald-500 bg-emerald-50 dark:bg-emerald-950/30",
  danger: "border-red-500 bg-red-50 dark:bg-red-950/30",
};

const calloutIcons = {
  info: "ℹ️",
  warning: "⚠️",
  tip: "💡",
  danger: "🚨",
};

export function Callout({ type = "info", title, children }: CalloutProps) {
  return (
    <div className={`my-4 border-l-4 rounded-r-lg p-4 ${calloutStyles[type]}`}>
      {title && (
        <p className="font-semibold mb-1">
          {calloutIcons[type]} {title}
        </p>
      )}
      <div className="text-sm">{children}</div>
    </div>
  );
}

type FeatureCardProps = {
  icon: string;
  title: string;
  description: string;
  href?: string;
};

export function FeatureCard({ icon, title, description, href }: FeatureCardProps) {
  const content = (
    <div className="group p-6 rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] hover:border-[var(--color-primary)]/50 hover:shadow-lg transition-all">
      <div className="text-3xl mb-3">{icon}</div>
      <h3 className="text-lg font-semibold mb-2 group-hover:text-[var(--color-primary)] transition-colors">
        {title}
      </h3>
      <p className="text-sm text-[var(--color-text-secondary)]">{description}</p>
    </div>
  );

  if (href) {
    const Link = require("next/link").default;
    return <Link href={href}>{content}</Link>;
  }

  return content;
}
