import { useCallback, useEffect, useId, useRef, useState } from "react";
import type { ReactElement } from "react";
import { Search } from "lucide-react";
import { useDismiss } from "@/shared/hook/use-dismiss";
import type { TerminologyController } from "./use-terminology";
import { useDebouncedQuery } from "./use-term-list";
import { TermbaseList } from "./TermbaseList";
import { SelectedTerms } from "./SelectedTerms";
import { TerminologyToolbar } from "./TerminologyToolbar";
import { TerminologyEditors } from "./TerminologyEditors";
import type { TermbaseExportController } from "./use-termbase-export";
import type { TermbaseAppendController } from "./use-termbase-append";

type Props = {
  state: TerminologyController;
  locked: boolean;
  exports: TermbaseExportController;
  appends: TermbaseAppendController;
  onInteraction: (active: boolean) => void;
};

export function TerminologyLookup({
  state,
  locked,
  exports,
  appends,
  onInteraction,
}: Props): ReactElement {
  const access = state.access;
  const [panel, setPanel] = useState<"closed" | "bases" | "terms">("closed");
  const root = useRef<HTMLDivElement>(null);
  const panelId = useId();
  const close = useCallback(() => {
    setPanel("closed");
  }, []);
  const active = panel !== "closed" || state.editor !== null;
  useEffect(() => {
    onInteraction(active);
    return () => {
      onInteraction(false);
    };
  }, [active, onInteraction]);
  useDismiss(panel !== "closed" && state.editor === null, root, close);
  const query = useDebouncedQuery(state.baseQuery, 300);
  const selected =
    state.selection.state.status === "ready"
      ? state.selection.state.value
      : null;
  const name = selected?.termbase.name ?? "";
  return (
    <>
      <div
        ref={root}
        data-testid="terminology-lookup"
        className={`absolute bottom-2 left-2 z-40 max-w-[calc(100%-1rem)] transition-[width] duration-300 motion-reduce:transition-none ${panel === "closed" ? "w-[min(max(9rem,20%),calc(100%-1rem))]" : "w-[calc(100%-1rem)] @min-[40rem]:w-[max(9rem,min(40%,24rem))]"}`}
      >
        {panel !== "closed" && (
          <div
            id={panelId}
            role="dialog"
            aria-label={panel === "bases" ? "选择术语库" : "查询术语"}
            className="absolute bottom-[calc(100%+0.25rem)] left-0 flex h-[20dvh] max-h-[min(18rem,calc(100dvh-5rem))] min-h-28 w-full flex-col overflow-hidden rounded-lg border border-border bg-card/95 shadow-sm backdrop-blur-md"
          >
            {panel === "bases" ? (
              <>
                <div className="px-1.5 pt-1.5">
                  <TerminologyToolbar
                    query={state.baseQuery}
                    label="搜索术语库名称"
                    createLabel="新建术语库"
                    canCreate={!locked}
                    onQuery={state.setBaseQuery}
                    onCreate={() => {
                      state.setEditor({ kind: "base", value: null });
                    }}
                  />
                </div>
                <TermbaseList
                  key={state.baseRevision}
                  access={access}
                  query={query}
                  selectedId={state.selectedId}
                  directory={false}
                  locked={locked}
                  exports={exports}
                  appends={appends}
                  onSelect={(info) => {
                    state.select(info.termbase.id);
                    state.setBaseQuery("");
                    close();
                  }}
                  onEdit={(info) => {
                    state.setEditor({ kind: "base", value: info.termbase });
                  }}
                />
              </>
            ) : (
              <SelectedTerms state={state} directory={false} locked={locked} />
            )}
          </div>
        )}
        <div className="flex h-8 overflow-hidden rounded-lg border border-border bg-card/95 shadow-sm backdrop-blur-md">
          <button
            type="button"
            aria-label={name ? `切换术语库，当前为 ${name}` : "选择术语库"}
            title={name || "选择术语库"}
            aria-haspopup="dialog"
            aria-expanded={panel === "bases"}
            aria-controls={panel === "bases" ? panelId : undefined}
            onClick={() => {
              setPanel(panel === "bases" ? "closed" : "bases");
            }}
            className="flex size-8 shrink-0 items-center justify-center border-r border-border text-xs font-semibold hover:bg-accent focus-visible:outline-2 focus-visible:outline-ring"
          >
            {Array.from(name)[0] ?? "术"}
          </button>
          <label className="relative min-w-0 flex-1">
            <Search
              size={13}
              className="pointer-events-none absolute top-2.5 left-2.5 text-muted-foreground"
              aria-hidden="true"
            />
            <input
              role="combobox"
              aria-label="搜索术语原文"
              aria-expanded={panel === "terms"}
              aria-controls={panel === "terms" ? panelId : undefined}
              disabled={state.selectedId === null}
              value={state.termQuery}
              placeholder={state.selectedId ? "搜索原文…" : "先选择术语库"}
              onFocus={() => {
                setPanel("terms");
              }}
              onChange={(event) => {
                state.setTermQuery(event.target.value);
                setPanel("terms");
              }}
              className="h-full w-full bg-transparent pr-2.5 pl-7.5 text-xs outline-none focus:bg-card disabled:opacity-60"
            />
          </label>
        </div>
      </div>
      <TerminologyEditors state={state} onBaseSaved={state.savedBase} />
    </>
  );
}
