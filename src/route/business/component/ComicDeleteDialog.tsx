import { useState } from "react";
import type { ReactElement } from "react";
import { commands, NativeCommandError, unwrap } from "@/bridge";
import { Button } from "@/shared/component/Button";
import { Dialog } from "@/shared/component/Dialog";

type ComicDeleteDialogProps = {
  comicId: string;
  title: string;
  onClose: () => void;
  onComplete: () => void;
};

export function ComicDeleteDialog({
  comicId,
  title,
  onClose,
  onComplete,
}: ComicDeleteDialogProps): ReactElement {
  const [busy, setBusy] = useState(false);
  const [uncertain, setUncertain] = useState(false);
  const [error, setError] = useState("");
  async function remove(): Promise<void> {
    setBusy(true);
    setError("");
    try {
      unwrap(await commands.deleteComic(comicId));
      setUncertain(false);
      onComplete();
    } catch (cause) {
      if (
        !(cause instanceof NativeCommandError) ||
        cause.detail.recovery === "wait_for_confirmation"
      ) {
        setUncertain(true);
      }
      setError(
        cause instanceof NativeCommandError
          ? cause.detail.message
          : "删除结果尚未确认，请保持窗口并核实。",
      );
    } finally {
      setBusy(false);
    }
  }
  return (
    <Dialog
      open={true}
      onOpenChange={(open) => {
        if (!open && !busy && !uncertain) {
          onClose();
        }
      }}
      title="删除项目"
      description={`确定删除“${title}”及其全部页面和翻校内容？此操作无法撤销，原始导入目录中的图片不会被删除。`}
      className=""
    >
      {error !== "" && (
        <p role="alert" className="text-xs text-destructive">
          {error}
        </p>
      )}
      <div className="flex justify-end gap-2">
        <Button variant="ghost" disabled={busy || uncertain} onClick={onClose}>
          取消
        </Button>
        <Button
          variant="danger"
          disabled={busy}
          onClick={() => {
            remove().catch(() => {
              setError("删除结果尚未确认，请核实。");
            });
          }}
        >
          {busy ? "正在处理…" : uncertain ? "核实删除结果" : "删除项目"}
        </Button>
      </div>
    </Dialog>
  );
}
