import { useEffect, useRef, useState } from "react";
import type { ReactElement } from "react";
import { commands, NativeCommandError, unwrap } from "@/bridge";
import type {
  ImageImportTarget,
  ImageTaskStatus,
} from "@/bridge/generated/bindings";
import { imageUrl } from "@/bridge/image-url";
import { Button } from "@/shared/component/Button";
import { Dialog } from "@/shared/component/Dialog";

type ImageImportDialogProps = {
  target: ImageImportTarget;
  onClose: () => void;
  onComplete: () => void;
};

function running(status: ImageTaskStatus): boolean {
  return [
    "selecting",
    "preparing",
    "stopping",
    "publishing",
    "committing",
  ].includes(status.phase);
}

export function ImageImportDialog({
  target,
  onClose,
  onComplete,
}: ImageImportDialogProps): ReactElement {
  const [task, setTask] = useState<ImageTaskStatus | null>(null);
  const [clearUnits, setClearUnits] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [pending, setPending] = useState<boolean | null>(null);
  const [refresh, setRefresh] = useState(0);
  const taskGeneration = useRef(0);
  const taskId = task?.task_id ?? "";
  const poll = task !== null && running(task);
  const uncertain = pending !== null || task?.phase === "uncertain";
  const locked = busy || uncertain;

  useEffect(() => {
    if (taskId === "") {
      return;
    }
    let active = true;
    const generation = taskGeneration.current;
    let timer: ReturnType<typeof setTimeout> | undefined;
    async function update(): Promise<void> {
      try {
        const next = unwrap(await commands.getImageImport(taskId));
        if (!active || generation !== taskGeneration.current) {
          return;
        }
        setTask(next);
        if (running(next)) {
          timer = setTimeout(() => {
            update().catch(() => {
              if (active && generation === taskGeneration.current) {
                setError("任务状态无法读取，请重新查询。");
              }
            });
          }, 500);
        }
      } catch {
        if (active && generation === taskGeneration.current) {
          setError("任务状态无法读取，请重新查询。操作可能仍在继续。");
        }
      }
    }
    update().catch(() => {
      if (active && generation === taskGeneration.current) {
        setError("任务状态无法读取，请重新查询。");
      }
    });
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [taskId, poll, refresh]);

  async function select(folder: boolean): Promise<void> {
    taskGeneration.current += 1;
    setBusy(true);
    setError("");
    try {
      setTask(
        unwrap(
          await (folder
            ? commands.selectImageFolder(target)
            : commands.selectImages(target)),
        ),
      );
    } catch (cause) {
      setError(
        cause instanceof NativeCommandError
          ? cause.detail.message
          : "图片选择未完成，请重试。",
      );
    } finally {
      setBusy(false);
    }
  }

  async function cancel(): Promise<void> {
    if (task === null) {
      onClose();
      return;
    }
    if (task.phase === "completed" && pending === null) {
      onComplete();
      return;
    }
    taskGeneration.current += 1;
    setBusy(true);
    setError("");
    try {
      const status = unwrap(await commands.cancelImageImport(task.task_id));
      setTask(status);
      setRefresh((value) => value + 1);
      if (status.phase === "cancelled" && !status.cleanup_pending) {
        onClose();
      }
    } catch (cause) {
      setError(
        cause instanceof NativeCommandError
          ? cause.detail.message
          : "取消尚未确认，请重新查询任务。",
      );
      setRefresh((value) => value + 1);
    } finally {
      setBusy(false);
    }
  }

  async function confirm(): Promise<void> {
    if (task === null) {
      return;
    }
    taskGeneration.current += 1;
    setBusy(true);
    setError("");
    const submitted = pending ?? clearUnits;
    let committed = false;
    try {
      unwrap(await commands.confirmImageImport(task.task_id, submitted));
      committed = true;
      setPending(null);
      const status = unwrap(await commands.getImageImport(task.task_id));
      setTask(status);
      if (!status.cleanup_pending) {
        onComplete();
      }
    } catch (cause) {
      if (committed) {
        setTask({
          ...task,
          phase: "completed",
          message: "图片已保存",
          previews: [],
        });
        setError(
          "图片已保存，清理状态暂时无法读取。可以完成并关闭窗口，下次导入时将重试清理。",
        );
        return;
      }
      if (
        !(cause instanceof NativeCommandError) ||
        cause.detail.recovery === "wait_for_confirmation"
      ) {
        setPending(submitted);
      }
      setError(
        cause instanceof NativeCommandError
          ? cause.detail.message
          : "写入结果未确认，请保持窗口并核实。",
      );
      setRefresh((value) => value + 1);
    } finally {
      setBusy(false);
    }
  }

  const canSelect =
    task === null || ["cancelled", "failed"].includes(task.phase);
  return (
    <Dialog
      open={true}
      onOpenChange={(open) => {
        if (!open && !locked) {
          cancel().catch(() => {
            setError("取消尚未确认，请重试。");
          });
        }
      }}
      title={target.kind === "append" ? "添加页面图片" : "重新选择页面图片"}
      description=""
      className="max-w-3xl"
    >
      {task !== null && (
        <p role="status" className="text-xs text-muted-foreground">
          {task.message} · {task.processed_files} / {task.total_files} 张
        </p>
      )}
      {task !== null && task.previews.length > 0 && (
        <div className="grid max-h-[45dvh] grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-3 overflow-y-auto">
          {task.previews.map((preview, index) => (
            <figure key={preview.handle} className="min-w-0">
              <img
                src={imageUrl(preview.handle)}
                loading="lazy"
                alt={`待导入第 ${String(index + 1)} 页`}
                className="aspect-[3/4] w-full rounded bg-muted object-contain"
              />
              <figcaption
                className="mt-1 truncate text-xs"
                title={preview.original_name}
              >
                {index + 1}. {preview.original_name}
                <span className="block text-muted-foreground">
                  {preview.width} × {preview.height}
                </span>
              </figcaption>
            </figure>
          ))}
        </div>
      )}
      {target.kind === "replace" && task?.phase === "ready" && (
        <fieldset disabled={locked} className="space-y-2 text-sm">
          <legend className="mb-2 font-medium">原有翻校单元</legend>
          <label className="flex gap-2">
            <input
              type="radio"
              name="units"
              checked={!clearUnits}
              onChange={() => {
                setClearUnits(false);
              }}
            />
            保留全部单元、文本和归一化位置
          </label>
          <label className="flex gap-2">
            <input
              type="radio"
              name="units"
              checked={clearUnits}
              onChange={() => {
                setClearUnits(true);
              }}
            />
            清空全部单元（无法撤销）
          </label>
        </fieldset>
      )}
      {task?.cleanup_pending === true && (
        <p className="text-xs text-muted-foreground">
          {task.phase === "completed"
            ? "图片已保存，可以完成并关闭窗口；下次导入时将自动重试清理。"
            : "暂存资源清理未完成，可再次取消以重试清理。"}
        </p>
      )}
      {error !== "" && (
        <p role="alert" className="text-xs text-destructive">
          {error}
        </p>
      )}
      <div className="flex flex-wrap justify-end gap-2">
        <Button
          variant="ghost"
          disabled={
            locked ||
            task?.phase === "committing" ||
            task?.phase === "publishing"
          }
          onClick={() => {
            cancel().catch(() => {
              setError("取消尚未确认，请重试。");
            });
          }}
        >
          {task?.phase === "completed" ? "关闭" : "取消"}
        </Button>
        {error !== "" && task !== null && (
          <Button
            variant="secondary"
            disabled={busy}
            onClick={() => {
              setError("");
              setRefresh((value) => value + 1);
            }}
          >
            重新查询
          </Button>
        )}
        {canSelect && (
          <Button
            variant="primary"
            disabled={locked || task?.cleanup_pending === true}
            onClick={() => {
              select(false).catch(() => {
                setError("图片选择未完成，请重试。");
              });
            }}
          >
            选择图片
          </Button>
        )}
        {canSelect && target.kind === "append" && (
          <Button
            variant="secondary"
            disabled={locked || task?.cleanup_pending === true}
            onClick={() => {
              select(true).catch(() => {
                setError("图片选择未完成，请重试。");
              });
            }}
          >
            选择图片文件夹
          </Button>
        )}
        {(task?.phase === "ready" || uncertain) && (
          <Button
            variant="primary"
            disabled={busy}
            onClick={() => {
              confirm().catch(() => {
                setError("提交未确认，请保持窗口并重试核实。");
              });
            }}
          >
            {busy
              ? "正在处理…"
              : uncertain
                ? "核实提交结果"
                : clearUnits
                  ? "确认换图并清空单元"
                  : "确认导入"}
          </Button>
        )}
        {task?.phase === "completed" && !uncertain && (
          <Button variant="primary" onClick={onComplete}>
            完成
          </Button>
        )}
      </div>
    </Dialog>
  );
}
