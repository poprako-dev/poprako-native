import type { ReactElement } from "react";
import { LoaderCircle, Share } from "lucide-react";

type Props = {
  busy: boolean;
  onExport: () => void;
};

export function TermbaseShareButton({ busy, onExport }: Props): ReactElement {
  return (
    <button
      type="button"
      aria-label="导出术语库"
      title="导出术语库"
      aria-busy={busy}
      disabled={busy}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      onPointerUp={(event) => {
        event.stopPropagation();
      }}
      onClick={(event) => {
        event.stopPropagation();
        onExport();
      }}
      className="flex size-5 shrink-0 items-center justify-center rounded-sm border border-border bg-card text-muted-foreground transition-colors enabled:hover:bg-accent disabled:cursor-not-allowed disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-ring"
    >
      {busy ? (
        <span role="status" aria-label="正在导出术语库">
          <LoaderCircle size={12} className="animate-spin" aria-hidden="true" />
        </span>
      ) : (
        <Share size={12} aria-hidden="true" />
      )}
    </button>
  );
}
