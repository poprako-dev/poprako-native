import { useState } from "react";
import type { ReactElement } from "react";
import { commands, NativeCommandError, unwrap } from "@/bridge";
import type { ArchiveTarget, TextStage } from "@/bridge/generated/bindings";
import { ArchiveTaskDialog } from "@/route/business/component/ArchiveTaskDialog";
import { Button } from "@/shared/component/Button";
import { Dialog } from "@/shared/component/Dialog";

type ArchiveImportDialogProps = {
  target: ArchiveTarget;
  onClose: () => void;
  onComplete: (comicId: string) => void;
};

export function ArchiveImportDialog({
  target,
  onClose,
  onComplete,
}: ArchiveImportDialogProps): ReactElement {
  const [stage, setStage] = useState<TextStage>("translation");
  const [pairImages, setPairImages] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [taskId, setTaskId] = useState<string | null>(null);

  async function choose(): Promise<void> {
    setBusy(true);
    setError("");
    try {
      const result = unwrap(
        await commands.chooseArchiveImport(
          target,
          stage,
          target.kind === "new" && pairImages,
        ),
      );
      if (result.status === "selected") {
        setTaskId(result.task_id);
      }
    } catch (cause) {
      setError(
        cause instanceof NativeCommandError
          ? cause.detail.message
          : "文件选择未完成，请重试。",
      );
    } finally {
      setBusy(false);
    }
  }

  if (taskId !== null) {
    return (
      <ArchiveTaskDialog
        taskId={taskId}
        intent="import"
        existing={target.kind === "existing"}
        onClose={onClose}
        onComplete={onComplete}
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
      title={target.kind === "new" ? "导入本地项目" : "导入项目译文"}
      description=""
      className=""
    >
      <fieldset disabled={busy} className="space-y-2 text-sm">
        <legend className="mb-2 font-medium">
          如果选择 LP TXT，将文本导入为
        </legend>
        <label className="flex gap-2">
          <input
            type="radio"
            name="stage"
            checked={stage === "translation"}
            onChange={() => {
              setStage("translation");
            }}
          />
          翻译文本
        </label>
        <label className="flex gap-2">
          <input
            type="radio"
            name="stage"
            checked={stage === "proofreading"}
            onChange={() => {
              setStage("proofreading");
            }}
          />
          校对文本（不自动确认校对）
        </label>
      </fieldset>
      {target.kind === "new" && (
        <fieldset disabled={busy} className="space-y-2 text-sm">
          <legend className="mb-2 font-medium">配套图片</legend>
          <label className="flex gap-2">
            <input
              type="radio"
              name="images"
              checked={pairImages}
              onChange={() => {
                setPairImages(true);
              }}
            />
            另选完整图片目录（单独 PRK／LP 或译文 ZIP）
          </label>
          <label className="flex gap-2">
            <input
              type="radio"
              name="images"
              checked={!pairImages}
              onChange={() => {
                setPairImages(false);
              }}
            />
            使用含图片 ZIP 内的图片
          </label>
          <p className="text-xs text-muted-foreground">
            图片按自然数字顺序与译文页序逐页对应，确认前展示配对结果。
          </p>
        </fieldset>
      )}
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
            choose().catch(() => {
              setError("选择未完成，请重试。");
            });
          }}
        >
          {busy ? "正在选择…" : "选择译文或项目包"}
        </Button>
      </div>
    </Dialog>
  );
}
