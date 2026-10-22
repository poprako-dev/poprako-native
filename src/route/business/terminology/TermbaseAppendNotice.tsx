import type { ReactElement } from "react";
import { Button } from "@/shared/component/Button";
import { ButtonGroup } from "@/shared/component/ButtonGroup";
import type { TermbaseAppendController } from "./use-termbase-append";

type Props = { appends: TermbaseAppendController };

export function TermbaseAppendNotice({ appends }: Props): ReactElement | null {
  const message = appends.busy
    ? "正在追加术语…"
    : appends.error || appends.notice;
  if (message === "") return null;
  return (
    <div className="pointer-events-auto space-y-1 rounded border border-border bg-card px-2.5 py-1 text-xs">
      <p
        role={appends.error ? "alert" : "status"}
        className={appends.error ? "text-destructive" : "text-muted-foreground"}
      >
        {message}
      </p>
      {appends.canRetry && (
        <ButtonGroup>
          <Button
            variant="secondary"
            disabled={appends.busy}
            onClick={() => {
              void appends.retry();
            }}
          >
            {appends.uncertain ? "核实写入结果" : "重试追加"}
          </Button>
        </ButtonGroup>
      )}
    </div>
  );
}
