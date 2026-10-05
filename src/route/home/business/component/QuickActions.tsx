import { FolderInput, FolderPlus, Play } from "lucide-react";
import type { ReactElement } from "react";

type QuickActionsProps = {
  latestTitle: string;
  canContinue: boolean;
  busy: boolean;
  onContinue: () => void;
  onCreate: () => void;
  onImport: () => void;
};

export function QuickActions({
  latestTitle,
  canContinue,
  busy,
  onContinue,
  onCreate,
  onImport,
}: QuickActionsProps): ReactElement {
  return (
    <nav
      aria-label="项目操作"
      className="grid h-28 shrink-0 grid-cols-[minmax(0,2fr)_minmax(0,1fr)] overflow-hidden rounded-md border border-border bg-card"
    >
      <button
        type="button"
        disabled={!canContinue || busy}
        onClick={onContinue}
        className="flex min-w-0 items-center gap-3 px-4 text-left hover:bg-accent focus-visible:outline-2 focus-visible:outline-ring disabled:cursor-default disabled:hover:bg-transparent sm:gap-4 sm:px-6"
      >
        <Play
          className="size-6 shrink-0 stroke-[1.5] text-primary"
          aria-hidden="true"
        />
        <span className="flex min-w-0 flex-col gap-1">
          <span className="truncate text-sm font-medium">继续上次的项目</span>
          <span className="truncate text-xs text-muted-foreground">
            {canContinue ? latestTitle : "暂无最近项目"}
          </span>
        </span>
      </button>
      <div className="grid grid-rows-2 divide-y divide-border border-l border-border">
        <button
          type="button"
          disabled={busy}
          onClick={onCreate}
          className="flex min-w-0 items-center gap-2 px-3 text-left hover:bg-accent focus-visible:outline-2 focus-visible:outline-ring disabled:opacity-40 sm:gap-3 sm:px-4"
        >
          <FolderPlus
            className="size-4 shrink-0 stroke-[1.5] text-primary"
            aria-hidden="true"
          />
          <span className="flex min-w-0 flex-col gap-0.5">
            <span className="truncate text-xs font-medium sm:text-sm">
              创建新项目
            </span>
          </span>
        </button>
        <button
          type="button"
          disabled={busy}
          onClick={onImport}
          className="flex min-w-0 items-center gap-2 px-3 text-left hover:bg-accent focus-visible:outline-2 focus-visible:outline-ring disabled:opacity-40 sm:gap-3 sm:px-4"
        >
          <FolderInput
            className="size-4 shrink-0 stroke-[1.5] text-primary"
            aria-hidden="true"
          />
          <span className="flex min-w-0 flex-col gap-0.5">
            <span className="truncate text-xs font-medium sm:text-sm">
              导入本地项目
            </span>
          </span>
        </button>
      </div>
    </nav>
  );
}
