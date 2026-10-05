import { createRootRoute } from "@tanstack/react-router";
import type { ReactElement } from "react";
import { ApplicationRoot } from "@/route/business/component/ApplicationRoot";
import { WindowFrame } from "@/route/business/component/WindowFrame";
import { StatePanel } from "@/shared/component/StatePanel";

export const Route = createRootRoute({
  component: ApplicationRoot,
  pendingComponent: Pending,
  errorComponent: Failure,
});

function Pending(): ReactElement {
  return (
    <WindowFrame>
      <StatePanel state="loading" message="正在打开项目…" />
    </WindowFrame>
  );
}

function Failure({ reset }: { reset: () => void }): ReactElement {
  return (
    <WindowFrame>
      <StatePanel
        state="error"
        message="页面无法打开，请重试。已保存的内容不会受影响。"
        onRetry={reset}
      />
    </WindowFrame>
  );
}
