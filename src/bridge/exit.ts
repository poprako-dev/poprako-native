import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { commands, unwrap } from "./index";

export interface ExitGuard {
  (): Promise<boolean>;
}

const guards = new Set<ExitGuard>();
let closing = false;

export function registerExitGuard(guard: ExitGuard): () => void {
  guards.add(guard);
  return () => {
    guards.delete(guard);
  };
}

async function requestClose(onError: (message: string) => void): Promise<void> {
  if (closing) return;
  closing = true;
  try {
    for (const guard of guards) if (!(await guard())) return;
    unwrap(await commands.requestExit());
  } catch {
    onError("尚未安全退出，请检查保存状态后重试。");
  } finally {
    closing = false;
  }
}

export async function listenForExit(
  onError: (message: string) => void,
): Promise<UnlistenFn> {
  return listen("application-close-requested", () => {
    void requestClose(onError);
  });
}
