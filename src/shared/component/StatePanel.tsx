import { AlertCircle, LoaderCircle } from "lucide-react";
import type { ReactElement } from "react";
import { Button } from "@/shared/component/Button";

type StatePanelProps =
  | { state: "loading" | "empty"; message: string }
  | { state: "error"; message: string; onRetry: () => void };

export function StatePanel(props: StatePanelProps): ReactElement {
  return (
    <div
      className="flex min-h-32 flex-col items-center justify-center gap-3 p-6 text-center text-sm text-muted-foreground"
      role={props.state === "error" ? "alert" : "status"}
    >
      {props.state === "loading" && (
        <LoaderCircle className="size-5 animate-spin" aria-hidden="true" />
      )}
      {props.state === "error" && (
        <AlertCircle className="size-5" aria-hidden="true" />
      )}
      <p>{props.message}</p>
      {props.state === "error" && (
        <Button variant="secondary" onClick={props.onRetry}>
          重试
        </Button>
      )}
    </div>
  );
}
