import { useState } from "react";
import type { ReactElement } from "react";
import { commands, NativeCommandError, unwrap } from "@/bridge";
import type { Comic, ComicMetadata } from "@/bridge/generated/bindings";
import { MetadataForm } from "@/route/business/component/MetadataForm";
import { Button } from "@/shared/component/Button";
import { Dialog } from "@/shared/component/Dialog";

type ComicMetadataDialogProps = {
  comic: Comic;
  onClose: () => void;
  onComplete: () => void;
};

export function ComicMetadataDialog({
  comic,
  onClose,
  onComplete,
}: ComicMetadataDialogProps): ReactElement {
  const [busy, setBusy] = useState(false);
  const [pending, setPending] = useState<ComicMetadata | null>(null);
  const [error, setError] = useState("");
  const locked = busy || pending !== null;
  async function save(metadata: ComicMetadata): Promise<void> {
    setBusy(true);
    setError("");
    try {
      unwrap(await commands.updateComicMetadata(comic.id, metadata));
      setPending(null);
      onComplete();
    } catch (cause) {
      if (
        !(cause instanceof NativeCommandError) ||
        cause.detail.recovery === "wait_for_confirmation"
      ) {
        setPending(metadata);
      }
      setError(
        cause instanceof NativeCommandError
          ? cause.detail.message
          : "保存结果尚未确认，请保留当前窗口并核实。",
      );
    } finally {
      setBusy(false);
    }
  }
  function submit(metadata: ComicMetadata): void {
    save(metadata).catch(() => {
      setError("保存结果尚未确认，请核实。");
    });
  }
  return (
    <Dialog
      open={true}
      onOpenChange={(open) => {
        if (!open && !locked) {
          onClose();
        }
      }}
      title="编辑项目信息"
      description="修改资料不会改变页面顺序或翻校内容。"
      className=""
    >
      <MetadataForm
        title={comic.title}
        subtitle={comic.subtitle}
        author={comic.author}
        busy={locked}
        error={error}
        submitLabel="保存资料"
        onCancel={onClose}
        onSubmit={(title, subtitle, author) => {
          submit({ title, subtitle, author });
        }}
      />
      {pending !== null && (
        <Button
          variant="primary"
          disabled={busy}
          onClick={() => {
            submit(pending);
          }}
        >
          核实原保存结果
        </Button>
      )}
    </Dialog>
  );
}
