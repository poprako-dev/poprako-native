import type { ReactElement, ReactNode } from "react";

type ComicLayoutProps = { sidebar: ReactNode; children: ReactNode };

export function ComicLayout({
  sidebar,
  children,
}: ComicLayoutProps): ReactElement {
  return (
    <main className="flex h-full w-full overflow-hidden bg-background">
      <aside className="flex h-full min-h-0 w-40 shrink-0 flex-col gap-3 overflow-y-auto border-r border-border bg-accent/25 p-3 sm:w-52 sm:p-4">
        {sidebar}
      </aside>
      <section
        className="min-h-0 min-w-0 flex-1 overflow-y-auto p-3 sm:p-4"
        aria-label="项目页面"
      >
        {children}
      </section>
    </main>
  );
}
