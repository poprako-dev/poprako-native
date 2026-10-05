import { useCallback, useEffect, useRef, useState } from "react";
import type { ReactElement, RefObject } from "react";
import { useBlocker } from "@tanstack/react-router";
import { useStore } from "zustand";
import { registerExitGuard } from "@/bridge/exit";
import { Button } from "@/shared/component/Button";
import { Dialog } from "@/shared/component/Dialog";
import type { EditorSession } from "./editor-session";

export interface LeaveAction {
  run(): Promise<boolean>;
}
type LeaveGuardProps = {
  session: EditorSession;
  guardRef: RefObject<LeaveAction>;
  savePosition: LeaveAction["run"];
};

export function LeaveGuard({
  session,
  guardRef,
  savePosition,
}: LeaveGuardProps): ReactElement {
  const state = useStore(session.store);
  const [open, setOpen] = useState(false);
  const resolver = useRef<((allowed: boolean) => void) | null>(null);
  const pending = useRef<Promise<boolean> | null>(null);

  const leave = useCallback(async (): Promise<boolean> => {
    if (pending.current) return pending.current;
    pending.current = (async (): Promise<boolean> => {
      if (document.activeElement instanceof HTMLElement)
        document.activeElement.blur();
      if (await session.flush()) return savePosition();
      setOpen(true);
      return new Promise<boolean>((resolve) => {
        resolver.current = resolve;
      });
    })();
    const allowed = await pending.current;
    pending.current = null;
    if (!allowed) session.cancelLeave();
    return allowed;
  }, [session, savePosition]);

  useEffect(() => {
    guardRef.current = { run: leave };
    const unregister = registerExitGuard(leave);
    return () => {
      unregister();
      resolver.current?.(false);
    };
  }, [guardRef, leave]);

  useBlocker({
    shouldBlockFn: async () => !(await leave()),
    enableBeforeUnload: false,
  });

  function finish(allowed: boolean): void {
    setOpen(false);
    resolver.current?.(allowed);
    resolver.current = null;
  }

  async function retry(): Promise<void> {
    if ((await session.flush()) && (await savePosition())) finish(true);
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(value) => {
        if (!value) finish(false);
      }}
      title="修改尚未保存"
      description={
        state.error || "请结束输入或拖动后重试保存，也可以取消离开。"
      }
      className=""
    >
      {state.uncertain && (
        <p className="text-sm">请重试核实上次写入结果，确认前不能丢弃修改。</p>
      )}
      <div className="flex flex-wrap justify-end gap-2">
        <Button
          variant="secondary"
          onClick={() => {
            finish(false);
          }}
        >
          取消离开
        </Button>
        <Button
          variant="primary"
          disabled={state.busy}
          onClick={() => {
            void retry();
          }}
        >
          {state.uncertain ? "核实写入结果" : "重试保存"}
        </Button>
        <Button
          variant="danger"
          disabled={state.busy || state.uncertain}
          onClick={() => {
            if (session.discard()) finish(true);
          }}
        >
          放弃未保存修改并离开
        </Button>
      </div>
    </Dialog>
  );
}
