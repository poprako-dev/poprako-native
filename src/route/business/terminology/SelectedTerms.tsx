import type { ReactElement } from "react";
import { FolderOpen, Plus } from "lucide-react";
import { IconButton } from "@/shared/component/IconButton";
import { StatePanel } from "@/shared/component/StatePanel";
import { TermList } from "./TermList";
import { TerminologyToolbar } from "./TerminologyToolbar";
import { useDebouncedQuery } from "./use-term-list";
import type { TerminologyController } from "./use-terminology";

type Props = {
  state: TerminologyController;
  directory: boolean;
  locked: boolean;
};

export function SelectedTerms({
  state,
  directory,
  locked,
}: Props): ReactElement {
  const query = useDebouncedQuery(state.termQuery, directory ? 250 : 300);
  const selection = state.selection.state;
  if (selection.status === "loading")
    return <StatePanel state="loading" message="正在读取术语库…" />;
  if (selection.status === "error")
    return (
      <StatePanel
        state="error"
        message="无法读取术语库，请重试或返回库列表。"
        onRetry={state.selection.reload}
      />
    );
  const info = selection.value;
  if (!info) return <StatePanel state="empty" message="请先选择术语库。" />;
  const base = info.termbase;
  const editable =
    !locked && (state.access.kind === "global" || base.scope.kind === "comic");
  function create(): void {
    state.setEditor({ kind: "term", value: null, termbaseId: base.id });
  }
  return (
    <>
      {directory && (
        <TerminologyToolbar
          query={state.termQuery}
          label="搜索术语原文"
          createLabel="新建术语"
          canCreate={editable}
          onQuery={state.setTermQuery}
          onCreate={create}
        />
      )}
      <div
        className={`flex shrink-0 items-center gap-2 border-b border-border px-2.5 py-2 ${directory ? "min-h-12" : "min-h-8"}`}
      >
        {directory && (
          <FolderOpen
            size={18}
            className="shrink-0 text-muted-foreground"
            aria-hidden="true"
          />
        )}
        <h3
          className={`min-w-0 flex-1 truncate font-semibold ${directory ? "text-base" : "text-sm"}`}
        >
          {base.name}
        </h3>
        <span className="text-xs text-muted-foreground tabular-nums">
          {info.term_count} 条
        </span>
        {!directory && (
          <span className="rounded border border-border px-1 text-[9px] text-muted-foreground">
            {base.scope.kind === "global" ? "全局" : "本作"}
          </span>
        )}
        {!directory && editable && (
          <IconButton label="新建术语" onClick={create}>
            <Plus size={13} />
          </IconButton>
        )}
      </div>
      <TermList
        key={`${base.id}:${String(state.termRevision)}`}
        access={state.access}
        termbaseId={base.id}
        query={query}
        directory={directory}
        editable={editable}
        onEdit={(term) => {
          state.setEditor({ kind: "term", value: term, termbaseId: base.id });
        }}
      />
    </>
  );
}
