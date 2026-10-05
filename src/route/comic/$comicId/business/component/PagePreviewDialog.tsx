import { ChevronLeft, ChevronRight, ImagePlus } from "lucide-react";
import type { ReactElement } from "react";
import type { PageInfo } from "@/bridge/generated/bindings";
import { ManagedImage } from "@/route/business/component/ManagedImage";
import { Button } from "@/shared/component/Button";
import { IconButton } from "@/shared/component/IconButton";
import { Dialog } from "@/shared/component/Dialog";

type PagePreviewDialogProps = {
  title: string;
  pages: PageInfo[];
  selectedId: string;
  onSelect: (id: string) => void;
  onClose: () => void;
  onReplace: (page: PageInfo) => void;
};

export function PagePreviewDialog({
  title,
  pages,
  selectedId,
  onSelect,
  onClose,
  onReplace,
}: PagePreviewDialogProps): ReactElement | null {
  const index = pages.findIndex((info) => info.page.id === selectedId);
  const selected = pages[index];
  if (selected === undefined) {
    return null;
  }
  function move(direction: number): void {
    const next = pages[index + direction];
    if (next !== undefined) {
      onSelect(next.page.id);
    }
  }
  return (
    <Dialog
      open={true}
      onOpenChange={(open) => {
        if (!open) {
          onClose();
        }
      }}
      title={`第 ${String(selected.page.index + 1)} 页`}
      description={title}
      className="h-[92dvh] max-w-5xl"
    >
      <div className="min-h-0 flex-1 overflow-hidden rounded">
        <ManagedImage
          key={selected.page.id}
          comicId={selected.page.comic_id}
          pageId={selected.page.id}
          reference={selected.page.image.reference}
          kind="preview"
          alt={`第 ${String(selected.page.index + 1)} 页预览`}
          interactive={true}
        />
      </div>
      <div className="flex flex-wrap items-center justify-center gap-6">
        <Button
          variant="ghost"
          aria-label="上一页"
          disabled={index <= 0}
          onClick={() => {
            move(-1);
          }}
        >
          <ChevronLeft className="size-4" aria-hidden="true" />
        </Button>
        <span className="text-xs text-muted-foreground tabular-nums">
          {index + 1} / {pages.length}
        </span>
        <Button
          variant="ghost"
          aria-label="下一页"
          disabled={index >= pages.length - 1}
          onClick={() => {
            move(1);
          }}
        >
          <ChevronRight className="size-4" aria-hidden="true" />
        </Button>
        <IconButton
          label="更换图片／修复缺图"
          onClick={() => {
            onReplace(selected);
          }}
        >
          <ImagePlus size={16} aria-hidden="true" />
        </IconButton>
      </div>
    </Dialog>
  );
}
