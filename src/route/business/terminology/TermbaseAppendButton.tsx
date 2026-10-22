import type { ReactElement } from "react";
import { ListPlus, LoaderCircle } from "lucide-react";

type Props = { busy: boolean; disabled: boolean; onAppend: () => void };

export function TermbaseAppendButton({
  busy,
  disabled,
  onAppend,
}: Props): ReactElement {
  return (
    <button
      type="button"
      aria-label="追加术语"
      title="追加术语"
      aria-busy={busy}
      disabled={disabled}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      onPointerUp={(event) => {
        event.stopPropagation();
      }}
      onClick={(event) => {
        event.stopPropagation();
        onAppend();
      }}
      className="flex size-5 shrink-0 items-center justify-center rounded-sm border border-border bg-card text-muted-foreground transition-colors enabled:hover:bg-accent disabled:cursor-not-allowed disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-ring"
    >
      {busy ? (
        <LoaderCircle size={12} className="animate-spin" aria-hidden="true" />
      ) : (
        <ListPlus size={12} aria-hidden="true" />
      )}
    </button>
  );
}
