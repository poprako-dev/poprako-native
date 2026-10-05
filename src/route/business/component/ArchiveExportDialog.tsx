import { useState } from "react";
import type { ReactElement } from "react";
import { commands, NativeCommandError, unwrap } from "@/bridge";
import { ArchiveTaskDialog } from "@/route/business/component/ArchiveTaskDialog";
import { Button } from "@/shared/component/Button";
import { Dialog } from "@/shared/component/Dialog";

type ArchiveExportDialogProps = { comicId: string; onClose: () => void };

export function ArchiveExportDialog({
  comicId,
  onClose,
}: ArchiveExportDialogProps): ReactElement {
  const [includeImages, setIncludeImages] = useState(true);
  const [taskId, setTaskId] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  async function start(): Promise<void> {
    setBusy(true);
    setError("");
    try {
      const selection = unwrap(
        await commands.exportArchive(comicId, includeImages),
      );
      if (selection.status === "selected") {
        setTaskId(selection.task_id);
      }
    } catch (cause) {
      setError(
        cause instanceof NativeCommandError
          ? cause.detail.message
          : "导出未开始，请重试。",
      );
    } finally {
      setBusy(false);
    }
  }

  if (taskId !== null) {
    return (
      <ArchiveTaskDialog
        taskId={taskId}
        intent="export"
        existing={true}
        onClose={onClose}
        onComplete={onClose}
      />
    );
  }
  return (
    <Dialog
      open={true}
      onOpenChange={(open) => {
        if (!open && !busy) {
          onClose();
        }
      }}
      title="导出项目"
      description=""
      className=""
    >
      <fieldset disabled={busy} className="space-y-3 text-sm">
        <label className="flex items-start gap-2">
          <input
            type="radio"
            name="archive"
            checked={includeImages}
            onChange={() => {
              setIncludeImages(true);
            }}
          />
          <span>
            含图片 ZIP
            <span className="mt-1 block text-xs text-muted-foreground">
              含全部图片，可继续翻校。
            </span>
          </span>
        </label>
        <label className="flex items-start gap-2">
          <input
            type="radio"
            name="archive"
            checked={!includeImages}
            onChange={() => {
              setIncludeImages(false);
            }}
          />
          <span>
            译文 ZIP
            <span className="mt-1 block text-xs text-muted-foreground">
              需要另行配套图片。
            </span>
          </span>
        </label>
      </fieldset>
      {error !== "" && (
        <p role="alert" className="text-xs text-destructive">
          {error}
        </p>
      )}
      <div className="flex justify-end gap-2">
        <Button variant="ghost" disabled={busy} onClick={onClose}>
          取消
        </Button>
        <Button
          variant="primary"
          disabled={busy}
          onClick={() => {
            start().catch(() => {
              setError("导出未开始，请重试。");
            });
          }}
        >
          {busy ? "正在选择…" : "选择保存位置"}
        </Button>
      </div>
    </Dialog>
  );
}
