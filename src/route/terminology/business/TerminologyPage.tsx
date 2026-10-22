import { useEffect, useRef, useState } from "react";
import type { ReactElement } from "react";
import { useNavigate } from "@tanstack/react-router";
import { ArrowLeft } from "lucide-react";
import type { TerminologyContext } from "@/bridge/generated/bindings";
import { IconButton } from "@/shared/component/IconButton";
import { useTerminology } from "@/route/business/terminology/use-terminology";
import { useDebouncedQuery } from "@/route/business/terminology/use-term-list";
import { TerminologyToolbar } from "@/route/business/terminology/TerminologyToolbar";
import { TermbaseList } from "@/route/business/terminology/TermbaseList";
import { SelectedTerms } from "@/route/business/terminology/SelectedTerms";
import { TerminologyEditors } from "@/route/business/terminology/TerminologyEditors";

const access: TerminologyContext = { kind: "global" };

export function TerminologyPage(): ReactElement {
  const state = useTerminology(access);
  const [level, setLevel] = useState<"bases" | "terms">("bases");
  const [error, setError] = useState("");
  const current = state.selectedId === null ? "bases" : level;
  const root = useRef<HTMLElement>(null);
  const content = useRef<HTMLDivElement>(null);
  const navigate = useNavigate();
  const query = useDebouncedQuery(state.baseQuery, 250);
  useEffect(() => {
    const selector =
      current === "bases"
        ? 'input[aria-label="搜索术语库名称"]'
        : 'button[aria-label="返回术语库"]';
    root.current
      ?.querySelector<HTMLElement>(selector)
      ?.focus({ preventScroll: true });
    const media = window.matchMedia("(prefers-reduced-motion: reduce)");
    if (media.matches) return;
    const animation = content.current?.animate(
      [{ opacity: 0 }, { opacity: 1 }],
      {
        duration: 150,
        easing: "ease-out",
      },
    );
    function motion(): void {
      if (media.matches) animation?.cancel();
    }
    media.addEventListener("change", motion);
    return () => {
      animation?.cancel();
      media.removeEventListener("change", motion);
    };
  }, [current]);
  function back(): void {
    if (current === "terms") {
      setLevel("bases");
      return;
    }
    void navigate({ to: "/home" }).catch(() => {
      setError("无法返回首页，请重试。");
    });
  }
  return (
    <main
      ref={root}
      className="mx-auto flex h-full min-h-0 w-full max-w-5xl flex-col overflow-hidden p-4"
    >
      <header className="mb-2 flex h-7 shrink-0 items-center gap-2">
        <IconButton
          label={current === "terms" ? "返回术语库" : "返回首页"}
          onClick={back}
        >
          <ArrowLeft size={16} />
        </IconButton>
        <h1 className="text-lg font-semibold">
          {current === "terms" ? "术语库" : "全局术语库"}
        </h1>
      </header>
      {error && (
        <p role="alert" className="text-xs text-destructive">
          {error}
        </p>
      )}
      <div ref={content} className="flex min-h-0 flex-1 flex-col">
        <div hidden={current !== "bases"} className="h-full min-h-0">
          <section
            aria-label="全局术语库"
            className="flex h-full min-h-0 flex-col"
          >
            <TerminologyToolbar
              query={state.baseQuery}
              label="搜索术语库名称"
              createLabel="新建术语库"
              canCreate={true}
              onQuery={state.setBaseQuery}
              onCreate={() => {
                state.setEditor({ kind: "base", value: null });
              }}
            />
            <TermbaseList
              key={state.baseRevision}
              access={access}
              query={query}
              selectedId={state.selectedId}
              directory={true}
              locked={false}
              onSelect={(info) => {
                state.select(info.termbase.id);
                setLevel("terms");
              }}
              onEdit={(info) => {
                state.setEditor({ kind: "base", value: info.termbase });
              }}
            />
          </section>
        </div>
        <div hidden={current !== "terms"} className="h-full min-h-0">
          <section
            aria-label="全局术语"
            className="flex h-full min-h-0 flex-col"
          >
            {state.selectedId !== null && (
              <SelectedTerms
                key={state.selectedId}
                state={state}
                directory={true}
                locked={false}
              />
            )}
          </section>
        </div>
      </div>
      <TerminologyEditors
        state={state}
        onBaseSaved={(base) => {
          const creating = state.editor?.value === null;
          state.savedBase(base);
          if (creating) {
            state.select(base.id);
            setLevel("terms");
          }
        }}
      />
    </main>
  );
}
