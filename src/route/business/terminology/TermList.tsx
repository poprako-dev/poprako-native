import { useCallback, useState } from "react";
import type { ReactElement } from "react";
import { Copy } from "lucide-react";
import { commands, unwrap } from "@/bridge";
import type { Term, TerminologyContext } from "@/bridge/generated/bindings";
import { IconButton } from "@/shared/component/IconButton";
import { TerminologyList } from "./TerminologyList";
import { useTermList } from "./use-term-list";
import { useTermPress } from "./use-term-press";

type RowProps = {
  access: TerminologyContext;
  term: Term;
  directory: boolean;
  editable: boolean;
  onEdit: () => void;
};

function TermRow({
  access,
  term,
  directory,
  editable,
  onEdit,
}: RowProps): ReactElement {
  const { handlers: press } = useTermPress(editable, onEdit);
  const [notice, setNotice] = useState("");
  const [copying, setCopying] = useState(false);
  async function copy(): Promise<void> {
    setCopying(true);
    try {
      unwrap(await commands.copyTermTarget(access, term.id));
      setNotice("已复制首选译名");
    } catch (error) {
      setNotice(error instanceof Error ? error.message : "复制失败，请重试。");
    } finally {
      setCopying(false);
    }
  }
  return (
    <div role="listitem" className="border-b border-border/60 last:border-b-0">
      <div
        {...press}
        tabIndex={editable ? 0 : -1}
        aria-label={`${term.source}，${editable ? "按 F2 编辑" : "术语"}`}
        aria-keyshortcuts={editable ? "F2" : undefined}
        title={editable ? "长按或按 F2 编辑术语" : term.comment}
        className={`flex touch-pan-y items-center gap-2 hover:bg-accent focus-visible:outline-2 focus-visible:outline-ring ${directory ? "min-h-11 py-2 pr-2.5 pl-8 text-sm" : "min-h-8 px-2.5 py-1 text-xs"}`}
      >
        {directory && (
          <span className="flex w-4 shrink-0 justify-center" aria-hidden="true">
            <span className="size-1.5 rounded-full bg-primary" />
          </span>
        )}
        <span className="min-w-0 flex-1 truncate font-medium">
          {term.source}
        </span>
        <span
          className={`flex flex-wrap justify-end gap-1 ${directory ? "max-w-1/2" : "max-w-[68%]"}`}
        >
          {term.targets.map((target) => (
            <span
              key={target}
              className={`max-w-full rounded-md border border-primary/20 bg-accent px-1.5 leading-4 break-words text-primary ${directory ? "text-xs" : "text-[10px]"}`}
            >
              {target}
            </span>
          ))}
        </span>
        <span
          onPointerDown={(event) => {
            event.stopPropagation();
          }}
          onPointerUp={(event) => {
            event.stopPropagation();
          }}
          onKeyDown={(event) => {
            if (event.key === "F2") event.stopPropagation();
          }}
        >
          <IconButton
            label={`复制 ${term.source} 的首选译名`}
            disabled={copying}
            onClick={() => {
              void copy();
            }}
            className="size-6 border border-border bg-card"
          >
            <Copy size={13} />
          </IconButton>
        </span>
      </div>
      {notice !== "" && (
        <p role="status" className="px-2.5 pb-1 text-xs text-muted-foreground">
          {notice}
        </p>
      )}
    </div>
  );
}

type Props = {
  access: TerminologyContext;
  termbaseId: string;
  query: string;
  directory: boolean;
  editable: boolean;
  onEdit: (term: Term) => void;
};

export function TermList({
  access,
  termbaseId,
  query,
  directory,
  editable,
  onEdit,
}: Props): ReactElement {
  const load = useCallback(
    async (offset: number) =>
      unwrap(
        await commands.listTermInfos(access, termbaseId, {
          text: query,
          offset,
          limit: 30,
        }),
      ),
    [access, termbaseId, query],
  );
  const list = useTermList(load);
  return (
    <TerminologyList
      count={list.items.length}
      loading={list.loading}
      more={list.more}
      error={list.error}
      onNext={list.next}
      label="术语列表"
      empty="没有匹配的术语"
    >
      {list.items.map((term) => (
        <TermRow
          key={term.id}
          access={access}
          term={term}
          directory={directory}
          editable={editable}
          onEdit={() => {
            onEdit(term);
          }}
        />
      ))}
    </TerminologyList>
  );
}
