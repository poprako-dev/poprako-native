import { useId, useState } from "react";
import type { ReactElement } from "react";
import { commands, unwrap } from "@/bridge";
import type { Chapter, Comic } from "@/bridge/generated/bindings";
import { useLibraryWrite } from "@/route/business/use-library-write";
import { Button } from "@/shared/component/Button";
import { Dialog } from "@/shared/component/Dialog";
import { ButtonGroup } from "@/shared/component/ButtonGroup";

export type MetadataTarget =
  | { kind: "new_comic" }
  | { kind: "comic"; comic: Comic }
  | {
      kind: "new_chapter";
      comicId: string;
      comicTitle: string;
      baseline: string[];
    }
  | { kind: "chapter"; chapter: Chapter };

type Props = {
  target: MetadataTarget;
  onClose: () => void;
  onComplete: () => void;
};
const inputClass =
  "h-9 w-full rounded-md border border-border bg-background px-3 text-sm focus-visible:outline-2 focus-visible:outline-ring";

export function LibraryMetadataDialog({
  target,
  onClose,
  onComplete,
}: Props): ReactElement {
  const id = useId();
  const isComic = target.kind === "comic" || target.kind === "new_comic";
  const [title, setTitle] = useState(
    target.kind === "comic"
      ? target.comic.title
      : target.kind === "chapter"
        ? target.chapter.title
        : "",
  );
  const [author, setAuthor] = useState(
    target.kind === "comic" ? target.comic.author : "",
  );
  const [chapterTitle, setChapterTitle] = useState("第1章");
  const write = useLibraryWrite(onComplete);
  const heading =
    target.kind === "new_comic"
      ? "新建漫画与首章"
      : target.kind === "comic"
        ? "编辑漫画资料"
        : target.kind === "new_chapter"
          ? "新增章节"
          : "编辑章节名称";

  function submit(): void {
    if (write.locked) return;
    const metadata = { title, author };
    switch (target.kind) {
      case "new_comic":
        write.execute(async () =>
          unwrap(
            await commands.createComic({
              comic: metadata,
              chapter_title: chapterTitle,
            }),
          ),
        );
        break;
      case "comic":
        write.execute(async () =>
          unwrap(await commands.updateComicMetadata(target.comic.id, metadata)),
        );
        break;
      case "new_chapter":
        write.execute(async () =>
          unwrap(
            await commands.createChapter(target.comicId, target.baseline, {
              title,
            }),
          ),
        );
        break;
      case "chapter":
        write.execute(async () =>
          unwrap(
            await commands.updateChapterMetadata(target.chapter.id, { title }),
          ),
        );
        break;
    }
  }

  return (
    <Dialog
      closeDisabled={write.locked}
      open={true}
      onOpenChange={(open) => {
        if (!open && !write.locked) onClose();
      }}
      title={heading}
      description={
        target.kind === "new_chapter" ? `所属漫画：${target.comicTitle}` : ""
      }
      className=""
    >
      <form
        className="space-y-4"
        onSubmit={(event) => {
          event.preventDefault();
          submit();
        }}
      >
        <div className="space-y-1.5">
          <label htmlFor={`${id}-title`} className="text-xs font-medium">
            {isComic ? "漫画标题" : "章节名称"}
          </label>
          <input
            id={`${id}-title`}
            className={inputClass}
            value={title}
            required
            disabled={write.locked}
            onChange={(event) => {
              setTitle(event.currentTarget.value);
            }}
          />
        </div>
        {isComic && (
          <div className="space-y-1.5">
            <label htmlFor={`${id}-author`} className="text-xs font-medium">
              漫画作者
            </label>
            <input
              id={`${id}-author`}
              className={inputClass}
              value={author}
              disabled={write.locked}
              onChange={(event) => {
                setAuthor(event.currentTarget.value);
              }}
            />
          </div>
        )}
        {target.kind === "new_comic" && (
          <div className="space-y-1.5">
            <label htmlFor={`${id}-chapter`} className="text-xs font-medium">
              首章名称
            </label>
            <input
              id={`${id}-chapter`}
              className={inputClass}
              value={chapterTitle}
              required
              disabled={write.locked}
              onChange={(event) => {
                setChapterTitle(event.currentTarget.value);
              }}
            />
          </div>
        )}
        {write.error !== "" && (
          <p role="alert" className="text-xs text-destructive">
            {write.error}
          </p>
        )}
        <ButtonGroup>
          {write.uncertain ? (
            <Button
              variant="primary"
              onClick={write.retry}
              disabled={write.busy}
            >
              核实原操作结果
            </Button>
          ) : (
            <Button
              variant="primary"
              type="submit"
              disabled={
                write.busy ||
                title.trim() === "" ||
                (target.kind === "new_comic" && chapterTitle.trim() === "")
              }
            >
              {write.busy ? "正在保存…" : "保存"}
            </Button>
          )}
        </ButtonGroup>
      </form>
    </Dialog>
  );
}
