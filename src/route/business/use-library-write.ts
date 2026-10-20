import { useState } from "react";
import { NativeCommandError } from "@/bridge";

interface LibraryOperation {
  (): Promise<unknown>;
}

type LibraryWrite = {
  busy: boolean;
  locked: boolean;
  uncertain: boolean;
  error: string;
  execute: (operation: LibraryOperation) => void;
  retry: () => void;
};

export function useLibraryWrite(onComplete: () => void): LibraryWrite {
  const [busy, setBusy] = useState(false);
  const [pending, setPending] = useState<LibraryOperation | null>(null);
  const [error, setError] = useState("");
  async function perform(operation: LibraryOperation): Promise<void> {
    setBusy(true);
    setError("");
    try {
      await operation();
      setPending(null);
      onComplete();
    } catch (cause) {
      const uncertain =
        !(cause instanceof NativeCommandError) ||
        cause.detail.recovery === "wait_for_confirmation";
      setPending(uncertain ? () => operation : null);
      setError(
        cause instanceof NativeCommandError
          ? cause.detail.message
          : "操作结果尚未确认，请保留窗口并核实。",
      );
    } finally {
      setBusy(false);
    }
  }
  function execute(operation: LibraryOperation): void {
    perform(operation).catch(() => {
      setPending(() => operation);
      setError("操作结果尚未确认，请核实。");
      setBusy(false);
    });
  }
  return {
    busy,
    locked: busy || pending !== null,
    uncertain: pending !== null,
    error,
    execute,
    retry: () => {
      if (pending !== null && !busy) execute(pending);
    },
  };
}
