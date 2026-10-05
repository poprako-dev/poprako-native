import { useState } from "react";
import type { ReactElement } from "react";
import { commands, NativeCommandError, unwrap } from "@/bridge";
import type {
  ArchiveImportMode,
  ComicMetadata,
} from "@/bridge/generated/bindings";
import { ArchiveImportPreview } from "@/route/business/component/ArchiveImportPreview";
import { useArchiveTask } from "@/route/business/use-archive-task";
import { Button } from "@/shared/component/Button";
import { Dialog } from "@/shared/component/Dialog";

type ArchiveTaskDialogProps = {
  taskId: string;
  intent: "import" | "export";
  existing: boolean;
  onClose: () => void;
  onComplete: (comicId: string) => void;
};
type PendingImport = { metadata: ComicMetadata; mode: ArchiveImportMode };

export function ArchiveTaskDialog({
  taskId,
  intent,
  existing,
  onClose,
  onComplete,
}: ArchiveTaskDialogProps): ReactElement {
  const resource = useArchiveTask(taskId);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [pending, setPending] = useState<PendingImport | null>(null);
  const task = resource.task;
  const uncertain =
    pending !== null || (task?.phase === "failed" && task.commit_uncertain);
  const locked =
    busy ||
    uncertain ||
    task?.phase === "committing" ||
    task?.phase === "cleaning";

  async function consume(comicId: string): Promise<void> {
    unwrap(await commands.acknowledgeArchiveTask(taskId));
    onComplete(comicId);
  }

  async function close(): Promise<void> {
    setBusy(true);
    setError("");
    try {
      if (task?.phase === "completed") {
        await consume(task.result.comic_id);
        return;
      }
      if (task?.phase === "failed" || task?.phase === "cancelled") {
        unwrap(await commands.acknowledgeArchiveTask(taskId));
        onClose();
        return;
      }
      unwrap(await commands.cancelArchiveTask(taskId));
      resource.refresh();
    } catch (cause) {
      setError(
        cause instanceof NativeCommandError
          ? cause.detail.message
          : "任务结束状态尚未确认，请重新查询。",
      );
    } finally {
      setBusy(false);
    }
  }

  async function confirm(
    metadata: ComicMetadata,
    mode: ArchiveImportMode,
  ): Promise<void> {
    setBusy(true);
    setError("");
    const payload = pending ?? { metadata, mode };
    try {
      unwrap(
        await commands.confirmArchiveImport(
          taskId,
          payload.metadata,
          payload.mode,
        ),
      );
      setPending(null);
      resource.refresh();
    } catch (cause) {
      if (
        !(cause instanceof NativeCommandError) ||
        cause.detail.recovery === "wait_for_confirmation"
      ) {
        setPending(payload);
      }
      setError(
        cause instanceof NativeCommandError
          ? cause.detail.message
          : "提交结果尚未确认，请保持窗口并核实。",
      );
      resource.refresh();
    } finally {
      setBusy(false);
    }
  }

  function closeSafely(): void {
    close().catch(() => {
      setError("任务状态未确认，请重新查询。");
    });
  }
  const message =
    task === null
      ? "正在读取任务…"
      : task.phase === "preparing"
        ? `正在准备 ${String(task.completed)} / ${String(task.total)}`
        : task.phase === "committing"
          ? "正在保存，请勿关闭窗口…"
          : task.phase === "cleaning"
            ? "正在清理暂存文件…"
            : task.phase === "cancelled"
              ? "已取消。"
              : task.phase === "completed"
                ? intent === "export"
                  ? "导出已完成。"
                  : "导入已完成。"
                : task.phase === "failed"
                  ? task.message
                  : "准备完成，请核对后确认。";
  return (
    <Dialog
      open={true}
      onOpenChange={(open) => {
        if (!open && !locked) {
          closeSafely();
        }
      }}
      title={intent === "import" ? "导入项目内容" : "导出项目"}
      description="任务结果由本机资料库确认；取消请求可能需要等待当前步骤结束。"
      className="max-w-2xl"
    >
      <p role="status" className="text-sm">
        {message}
      </p>
      {task?.phase === "awaiting_confirmation" && !uncertain && (
        <ArchiveImportPreview
          preview={task.preview}
          existing={existing}
          busy={busy}
          error={error}
          onCancel={closeSafely}
          onSubmit={(metadata, mode) => {
            confirm(metadata, mode).catch(() => {
              setError("提交尚未确认，请重新查询。");
            });
          }}
        />
      )}
      {task?.phase === "completed" &&
        task.result.warnings.map((warning) => (
          <p key={warning} className="text-xs text-muted-foreground">
            {warning}
          </p>
        ))}
      {(error !== "" || resource.error !== "") && (
        <p role="alert" className="text-xs text-destructive">
          {error || resource.error}
        </p>
      )}
      <div className="flex justify-end gap-2">
        {(error !== "" || resource.error !== "" || uncertain) && (
          <Button
            variant="secondary"
            disabled={busy}
            onClick={resource.refresh}
          >
            重新查询
          </Button>
        )}
        {pending !== null && (
          <Button
            variant="primary"
            disabled={busy}
            onClick={() => {
              confirm(pending.metadata, pending.mode).catch(() => {
                setError("提交尚未确认，请保留窗口。");
              });
            }}
          >
            核实原提交结果
          </Button>
        )}
        {task?.phase !== "awaiting_confirmation" && (
          <Button variant="ghost" disabled={locked} onClick={closeSafely}>
            {task?.phase === "completed"
              ? "完成"
              : task?.phase === "failed" || task?.phase === "cancelled"
                ? "关闭"
                : "取消任务"}
          </Button>
        )}
      </div>
    </Dialog>
  );
}
