import { FolderSymlink, Settings, Trash2 } from "lucide-react";
import { useCallback, useState } from "react";
import type { ReactElement } from "react";
import { commands, unwrap } from "@/bridge";
import type { Chapter, Comic } from "@/bridge/generated/bindings";
import { ChapterDeleteDialog } from "@/route/business/component/ChapterDeleteDialog";
import { LibraryMetadataDialog } from "@/route/business/component/LibraryMetadataDialog";
import type { MetadataTarget } from "@/route/business/component/LibraryMetadataDialog";
import { ChapterMoveDialog } from "@/route/home/business/component/ChapterMoveDialog";
import { ChapterItem } from "@/route/home/business/component/ChapterItem";
import { IconButton } from "@/shared/component/IconButton";
import { StatePanel } from "@/shared/component/StatePanel";
import { useResource } from "@/shared/hook/use-resource";

type Props = {
  comic: Comic;
  onReload: () => void;
  onOpen: (chapterId: string, continueWork: boolean) => void;
};
export function ChapterList({ comic, onReload, onOpen }: Props): ReactElement {
  const load = useCallback(
    async () => unwrap(await commands.listChapterInfos(comic.id)),
    [comic.id],
  );
  const resource = useResource(load);
  const [editing, setEditing] = useState<MetadataTarget | null>(null);
  const [deleting, setDeleting] = useState<Chapter | null>(null);
  const [moving, setMoving] = useState<Chapter | null>(null);
  if (resource.state.status === "loading")
    return <StatePanel state="loading" message="正在读取章节…" />;
  if (resource.state.status === "error")
    return (
      <StatePanel
        state="error"
        message="章节无法读取，请重试。"
        onRetry={resource.reload}
      />
    );
  const infos = resource.state.value;
  const baseline = infos.map((info) => info.chapter.id);
  return (
    <div className="py-1" role="group" aria-label={`${comic.title} 的章节`}>
      <ol>
        {infos.map((info) => (
          <li
            key={info.chapter.id}
            className="relative mb-3 flex flex-wrap items-center gap-1 rounded-md pr-2 pl-12 transition-colors hover:bg-accent focus-within:bg-accent before:absolute before:top-8 before:-bottom-6 before:left-[25px] before:border-l before:border-border last:before:hidden"
          >
            <span
              aria-hidden="true"
              className="absolute top-4 left-5 size-3 rounded-full border-2 border-muted-foreground/40"
            />
            <ChapterItem
              title={info.chapter.title}
              activityAt={info.last_opened_at ?? info.chapter.updated_at}
              pageCount={info.page_count}
              unitCount={info.unit_count}
              translatedCount={info.translated_count}
              confirmedCount={info.proofread_count}
              onOpen={() => {
                onOpen(info.chapter.id, false);
              }}
            />
            <div className="flex shrink-0 items-center gap-0.5">
              <IconButton
                label={`编辑章节 ${info.chapter.title}`}
                onClick={() => {
                  setEditing({ kind: "chapter", chapter: info.chapter });
                }}
              >
                <Settings size={14} />
              </IconButton>
              <IconButton
                label={`移动章节 ${info.chapter.title}`}
                onClick={() => {
                  setMoving(info.chapter);
                }}
              >
                <FolderSymlink size={14} />
              </IconButton>
              <IconButton
                label={`删除章节 ${info.chapter.title}`}
                onClick={() => {
                  setDeleting(info.chapter);
                }}
              >
                <Trash2 size={14} />
              </IconButton>
            </div>
          </li>
        ))}
      </ol>
      {infos.length === 0 && (
        <p className="py-2 pl-7 text-xs text-muted-foreground">
          暂无章节，可以新增或导入。
        </p>
      )}
      {editing !== null && (
        <LibraryMetadataDialog
          target={editing}
          onClose={() => {
            setEditing(null);
          }}
          onComplete={onReload}
        />
      )}
      {deleting !== null && (
        <ChapterDeleteDialog
          chapterId={deleting.id}
          title={deleting.title}
          onClose={() => {
            setDeleting(null);
          }}
          onComplete={onReload}
        />
      )}
      {moving !== null && (
        <ChapterMoveDialog
          chapter={moving}
          baseline={baseline}
          onClose={() => {
            setMoving(null);
          }}
          onComplete={onReload}
        />
      )}
    </div>
  );
}
