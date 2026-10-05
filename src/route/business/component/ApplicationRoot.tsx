import { WindowFrame } from "./WindowFrame";
import { Outlet } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { listenForExit } from "@/bridge/exit";
import type { ReactElement } from "react";
import { commands, unwrap } from "@/bridge";
import type { ApplicationPreference } from "@/bridge/generated/bindings";
import { PreferenceProvider } from "@/route/business/component/PreferenceProvider";
import { StatePanel } from "@/shared/component/StatePanel";
import { useResource } from "@/shared/hook/use-resource";

function ApplicationContent(): ReactElement {
  const resource = useResource(loadPreference);
  const [exitError, setExitError] = useState("");
  useEffect(() => {
    let active = true;
    let cleanup: (() => void) | null = null;
    void listenForExit(setExitError)
      .then((unlisten) => {
        if (!active) {
          unlisten();
          return;
        }
        cleanup = unlisten;
      })
      .catch(() => {
        if (active)
          setExitError("退出保护未能初始化，请勿在未保存时关闭窗口。");
      });
    return () => {
      active = false;
      cleanup?.();
    };
  }, []);
  if (resource.state.status === "loading") {
    return <StatePanel state="loading" message="正在打开本地资料库…" />;
  }
  if (resource.state.status === "error") {
    return (
      <StatePanel
        state="error"
        message="本地资料库无法打开。请重试；原有数据会保留。"
        onRetry={resource.reload}
      />
    );
  }
  return (
    <PreferenceProvider initial={resource.state.value}>
      {exitError !== "" && (
        <div role="alert" className="bg-red-50 p-2 text-red-800">
          {exitError}
        </div>
      )}
      <Outlet />
    </PreferenceProvider>
  );
}

async function loadPreference(): Promise<ApplicationPreference> {
  return unwrap(await commands.getPreference());
}

export function ApplicationRoot(): ReactElement {
  return (
    <WindowFrame>
      <ApplicationContent />
    </WindowFrame>
  );
}
