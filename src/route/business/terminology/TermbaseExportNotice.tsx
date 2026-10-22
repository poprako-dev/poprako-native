import type { ReactElement } from "react";
import type { TermbaseExportController } from "./use-termbase-export";

type Props = {
  exports: TermbaseExportController;
};

export function TermbaseExportNotice({ exports }: Props): ReactElement | null {
  if (exports.notice === "") return null;
  return (
    <p
      role={exports.failed ? "alert" : "status"}
      className={`rounded border border-border bg-card px-2.5 py-1 text-xs ${exports.failed ? "text-destructive" : "text-muted-foreground"}`}
    >
      {exports.notice}
    </p>
  );
}
