import type { ReactElement, ReactNode } from "react";

type HomeLayoutProps = { action: ReactNode; children: ReactNode };

export function HomeLayout({
  action,
  children,
}: HomeLayoutProps): ReactElement {
  return (
    <main className="flex h-full w-full justify-center overflow-hidden p-4">
      <div className="flex h-full min-h-0 w-full max-w-5xl flex-col gap-3">
        {action}
        <section
          className="min-h-0 flex-1 overflow-y-auto"
          aria-label="项目列表"
        >
          {children}
        </section>
      </div>
    </main>
  );
}
