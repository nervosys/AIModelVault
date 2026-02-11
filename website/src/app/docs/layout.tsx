import Sidebar from "@/components/Sidebar";

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex">
      <Sidebar />
      <main className="flex-1 lg:ml-[var(--sidebar-width)] min-h-[calc(100vh-var(--header-height))]">
        <article className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8 pb-20">
          {children}
        </article>
      </main>
    </div>
  );
}
