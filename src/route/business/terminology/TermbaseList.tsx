import { useCallback } from "react";
import type { ReactElement } from "react";
import { Folder } from "lucide-react";
import { commands, unwrap } from "@/bridge";
import type {
  TermbaseInfo,
  TerminologyContext,
} from "@/bridge/generated/bindings";
import { TerminologyList } from "./TerminologyList";
import { useTermList } from "./use-term-list";
import { useTermPress } from "./use-term-press";
import { TermbaseShareButton } from "./TermbaseShareButton";
import type { TermbaseExportController } from "./use-termbase-export";
import { TermbaseAppendButton } from "./TermbaseAppendButton";
import type { TermbaseAppendController } from "./use-termbase-append";

type RowProps = {
  info: TermbaseInfo;
  selected: boolean;
  directory: boolean;
  editable: boolean;
  busy: boolean;
  onExport: () => void;
  canAppend: boolean;
  appendBusy: boolean;
  appendDisabled: boolean;
  onAppend: () => void;
  onSelect: () => void;
  onEdit: () => void;
};

function TermbaseRow(props: RowProps): ReactElement {
  const { activate, handlers: press } = useTermPress(
    props.editable,
    props.onEdit,
  );
  const base = props.info.termbase;
  return (
    <div
      role="listitem"
      className={`flex items-center gap-3 border-b border-border/60 pr-2.5 last:border-b-0 hover:bg-accent ${props.selected ? "bg-accent" : ""}`}
    >
      <button
        type="button"
        {...press}
        onClick={() => {
          if (activate()) props.onSelect();
        }}
        aria-pressed={props.selected}
        aria-keyshortcuts={props.editable ? "F2" : undefined}
        title={props.editable ? "长按或按 F2 编辑术语库" : base.description}
        className={`flex min-w-0 flex-1 touch-pan-y items-center gap-3 pl-2.5 text-left focus-visible:outline-2 focus-visible:outline-ring ${props.directory ? "min-h-12 py-2 text-sm" : "min-h-7 py-1 text-xs"}`}
      >
        {props.directory && (
          <Folder
            size={18}
            className="shrink-0 text-muted-foreground"
            aria-hidden="true"
          />
        )}
        <span className="flex min-w-0 max-w-[68%] items-center gap-1.5">
          <span className="truncate font-medium">{base.name}</span>
          <span className="shrink-0 border-l border-border pl-1.5 text-[10px] text-muted-foreground tabular-nums">
            {props.info.term_count} 条
          </span>
          <span className="shrink-0 rounded border border-border px-1 text-[9px] text-muted-foreground">
            {base.scope.kind === "global" ? "全局" : "本作"}
          </span>
        </span>
        <span className="min-w-0 flex-1 truncate text-right text-[10px] text-muted-foreground">
          {base.description || "暂无描述"}
        </span>
      </button>
      <div className="flex shrink-0 items-center gap-1">
        <TermbaseShareButton busy={props.busy} onExport={props.onExport} />
        {props.canAppend && (
          <TermbaseAppendButton
            busy={props.appendBusy}
            disabled={props.appendDisabled}
            onAppend={props.onAppend}
          />
        )}
      </div>
    </div>
  );
}

type Props = {
  access: TerminologyContext;
  query: string;
  selectedId: string | null;
  directory: boolean;
  locked: boolean;
  exports: TermbaseExportController;
  appends: TermbaseAppendController;
  onSelect: (info: TermbaseInfo) => void;
  onEdit: (info: TermbaseInfo) => void;
};

export function TermbaseList(props: Props): ReactElement {
  const { access, query } = props;
  const load = useCallback(
    async (offset: number) =>
      unwrap(
        await commands.listTermbaseInfos(access, {
          text: query,
          offset,
          limit: 30,
        }),
      ),
    [access, query],
  );
  const list = useTermList(load);
  return (
    <TerminologyList
      count={list.items.length}
      loading={list.loading}
      more={list.more}
      error={list.error}
      onNext={list.next}
      label="术语库列表"
      empty="没有找到术语库"
    >
      {list.items.map((info) => (
        <TermbaseRow
          key={info.termbase.id}
          info={info}
          selected={info.termbase.id === props.selectedId}
          directory={props.directory}
          busy={props.exports.pending.has(info.termbase.id)}
          canAppend={
            access.kind === "global" || info.termbase.scope.kind === "comic"
          }
          appendBusy={props.appends.pendingId === info.termbase.id}
          appendDisabled={props.locked || props.appends.locked}
          onAppend={() => {
            void props.appends.appendTermbase(info.termbase.id);
          }}
          onExport={() => {
            void props.exports.exportTermbase(info.termbase.id);
          }}
          editable={
            !props.locked &&
            (access.kind === "global" || info.termbase.scope.kind === "comic")
          }
          onSelect={() => {
            props.onSelect(info);
          }}
          onEdit={() => {
            props.onEdit(info);
          }}
        />
      ))}
    </TerminologyList>
  );
}
