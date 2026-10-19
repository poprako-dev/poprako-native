import { useCallback, useState } from "react";
import type { ReactElement } from "react";
import { commands, unwrap } from "@/bridge";
import type { Chapter, MoveChapter } from "@/bridge/generated/bindings";
import { useLibraryWrite } from "@/route/business/use-library-write";
import { Button } from "@/shared/component/Button";
import { Dialog } from "@/shared/component/Dialog";
import { StatePanel } from "@/shared/component/StatePanel";
import { useResource } from "@/shared/hook/use-resource";

type Props = {
  chapter: Chapter;
  baseline: string[];
  onClose: () => void;
  onComplete: () => void;
};
export function ChapterMoveDialog({
  chapter,
  baseline,
  onClose,
  onComplete,
}: Props): ReactElement {
  const [offset, setOffset] = useState(0);
  const [target, setTarget] = useState("");
  const load = useCallback(
    async () => unwrap(await commands.listComicInfos(offset, 50)),
    [offset],
  );
  const comics = useResource(load);
  const loadOrder = useCallback(
    async () =>
      target === "" ? [] : unwrap(await commands.listChapterInfos(target)),
    [target],
  );
  const order = useResource(loadOrder);
  const write = useLibraryWrite(onComplete);
  function submit(): void {
    if (target === "" || order.state.status !== "ready" || write.locked) return;
    const input: MoveChapter = {
      chapter_id: chapter.id,
      source_comic_id: chapter.comic_id,
      target_comic_id: target,
      source_baseline: baseline,
      target_baseline: order.state.value.map((info) => info.chapter.id),
    };
    write.execute(async () => unwrap(await commands.moveChapter(input)));
  }
  return (
    <Dialog
      open={true}
      onOpenChange={(open) => {
        if (!open && !write.locked) onClose();
      }}
      title={`移动章节“${chapter.title}”`}
      description="追加到目标漫画末尾，采用目标漫画的标题与作者。页面、译文、图片和工作位置保持不变。"
      className=""
    >
      {comics.state.status === "loading" && (
        <StatePanel state="loading" message="正在读取漫画…" />
      )}
      {comics.state.status === "error" && (
        <StatePanel
          state="error"
          message="漫画列表无法读取。"
          onRetry={comics.reload}
        />
      )}
      {comics.state.status === "ready" && (
        <>
          <select
            aria-label="目标漫画"
            className="h-9 w-full rounded border border-border bg-background px-2"
            value={target}
            disabled={write.locked}
            onChange={(event) => {
              setTarget(event.currentTarget.value);
            }}
          >
            <option value="">选择目标漫画</option>
            {comics.state.value
              .filter((info) => info.comic.id !== chapter.comic_id)
              .map((info) => (
                <option key={info.comic.id} value={info.comic.id}>
                  {info.comic.title} · {info.comic.author || "未填写作者"} ·{" "}
                  {info.chapter_count} 章 · {info.comic.id.slice(0, 8)}
                </option>
              ))}
          </select>
          <div className="flex justify-center gap-2">
            <Button
              variant="ghost"
              disabled={write.locked || offset === 0}
              onClick={() => {
                setTarget("");
                setOffset(offset - 50);
              }}
            >
              上一页
            </Button>
            <Button
              variant="ghost"
              disabled={write.locked || comics.state.value.length < 50}
              onClick={() => {
                setTarget("");
                setOffset(offset + 50);
              }}
            >
              下一页
            </Button>
          </div>
        </>
      )}
      {order.state.status === "error" && (
        <StatePanel
          state="error"
          message="目标章节列表无法读取。"
          onRetry={order.reload}
        />
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
        {write.uncertain ? (
          <Button variant="primary" disabled={write.busy} onClick={write.retry}>
            核实移动结果
          </Button>
        ) : (
          <Button
            variant="primary"
            disabled={
              write.busy || target === "" || order.state.status !== "ready"
            }
            onClick={submit}
          >
            移动章节
          </Button>
        )}
      </div>
    </Dialog>
  );
}
