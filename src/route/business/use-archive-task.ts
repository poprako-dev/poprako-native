import { useEffect, useState } from "react";
import { commands, NativeCommandError, unwrap } from "@/bridge";
import type { ArchiveTask } from "@/bridge/generated/bindings";

export function useArchiveTask(taskId: string): {
  task: ArchiveTask | null;
  error: string;
  refresh: () => void;
} {
  const [task, setTask] = useState<ArchiveTask | null>(null);
  const [error, setError] = useState("");
  const [generation, setGeneration] = useState(0);
  useEffect(() => {
    let active = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    async function poll(): Promise<void> {
      try {
        const value = unwrap(await commands.getArchiveTask(taskId));
        if (!active) {
          return;
        }
        setTask(value);
        setError("");
        if (["preparing", "committing", "cleaning"].includes(value.phase)) {
          timer = setTimeout(() => {
            poll().catch(() => {
              if (active) {
                setError("任务无法查询，请重试。");
              }
            });
          }, 500);
        }
      } catch (cause) {
        if (active) {
          setError(
            cause instanceof NativeCommandError
              ? cause.detail.message
              : "任务无法查询，请重试。操作可能仍在继续。",
          );
        }
      }
    }
    poll().catch(() => {
      if (active) {
        setError("任务无法查询，请重试。");
      }
    });
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [taskId, generation]);
  return {
    task,
    error,
    refresh: () => {
      setGeneration((value) => value + 1);
    },
  };
}
