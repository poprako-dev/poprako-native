import { useCallback } from "react";
import type { ReactElement } from "react";
import { commands, unwrap } from "@/bridge";
import type { Comic } from "@/bridge/generated/bindings";
import { useLibraryWrite } from "@/route/business/use-library-write";
import { Button } from "@/shared/component/Button";
import { Dialog } from "@/shared/component/Dialog";
import { StatePanel } from "@/shared/component/StatePanel";
import { useResource } from "@/shared/hook/use-resource";

type Props = { comic: Comic; onClose: () => void; onComplete: () => void };
export function ComicDeleteDialog({
  comic,
  onClose,
  onComplete,
}: Props): ReactElement {
  const load = useCallback(
    async () => unwrap(await commands.listChapterInfos(comic.id)),
    [comic.id],
  );
  const resource = useResource(load);
  const write = useLibraryWrite(onComplete);
  return (
    <Dialog
      open={true}
      onOpenChange={(open) => {
        if (!open && !write.locked) onClose();
      }}
      title="删除漫画"
      description={`删除“${comic.title}”及其全部章节、页面和翻校内容？此操作无法撤销，原始导入目录中的图片不会被删除。`}
      className=""
    >
      {resource.state.status === "loading" && (
        <StatePanel state="loading" message="正在读取所属章节…" />
      )}
      {resource.state.status === "error" && (
        <StatePanel
          state="error"
          message="无法读取章节，请重试。"
          onRetry={resource.reload}
        />
      )}
      {resource.state.status === "ready" && (
        <div className="max-h-48 overflow-auto text-sm">
          <p>包含 {resource.state.value.length} 个章节：</p>
          <ul className="list-inside list-disc">
            {resource.state.value.map((info) => (
              <li key={info.chapter.id}>
                {info.chapter.title}（{info.page_count} 页）
              </li>
            ))}
          </ul>
        </div>
      )}
      {write.error !== "" && (
        <p role="alert" className="text-xs text-destructive">
          {write.error}
        </p>
      )}
      <div className="flex justify-end gap-2">
        <Button variant="ghost" disabled={write.locked} onClick={onClose}>
          取消
        </Button>
        <Button
          variant="danger"
          disabled={write.busy || resource.state.status !== "ready"}
          onClick={() => {
            if (write.uncertain) write.retry();
            else
              write.execute(async () =>
                unwrap(await commands.deleteComic(comic.id)),
              );
          }}
        >
          {write.uncertain ? "核实删除结果" : "删除漫画及全部章节"}
        </Button>
      </div>
    </Dialog>
  );
}
