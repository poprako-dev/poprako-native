import { useNavigate } from "@tanstack/react-router";
import { ArrowDownUp, FileInput, ImagePlus, Trash2 } from "lucide-react";
import { useState } from "react";
import type { ReactElement } from "react";
import type {
  ChapterDetail,
  ImageImportTarget,
  Page,
} from "@/bridge/generated/bindings";
import { ArchiveExportDialog } from "@/route/business/component/ArchiveExportDialog";
import { ArchiveImportDialog } from "@/route/business/component/ArchiveImportDialog";
import { ChapterDeleteDialog } from "@/route/business/component/ChapterDeleteDialog";
import { ImageImportDialog } from "@/route/business/component/ImageImportDialog";
import { ManagedImage } from "@/route/business/component/ManagedImage";
import { ChapterLayout } from "@/route/chapter/$chapterId/business/component/ChapterLayout";
import { ChapterMetadataDialog } from "@/route/chapter/$chapterId/business/component/ChapterMetadataDialog";
import { ChapterSidebar } from "@/route/chapter/$chapterId/business/component/ChapterSidebar";
import { PageItem } from "@/route/chapter/$chapterId/business/component/PageItem";
import { PageManagementDialog } from "@/route/chapter/$chapterId/business/component/PageManagementDialog";
import { PagePreviewDialog } from "@/route/chapter/$chapterId/business/component/PagePreviewDialog";
import { pageBaseline } from "@/route/chapter/$chapterId/business/page-baseline";
import { IconButton } from "@/shared/component/IconButton";
import { StatePanel } from "@/shared/component/StatePanel";

type ChapterContentProps = { detail: ChapterDetail; onReload: () => void };
type OpenDialog =
  | { kind: "none" | "metadata" | "delete" | "manage" | "import" | "export" }
  | { kind: "image"; target: ImageImportTarget }
  | { kind: "preview"; pageId: string };

export function ChapterContent({
  detail,
  onReload,
}: ChapterContentProps): ReactElement {
  const navigate = useNavigate();
  const [dialog, setDialog] = useState<OpenDialog>({ kind: "none" });
  const [notice, setNotice] = useState("");
  const first = detail.pages[0];
  const counts = detail.pages.reduce(
    (sum, info) => ({
      unit: sum.unit + info.unit_count,
      translated: sum.translated + info.translated_count,
      confirmed: sum.confirmed + info.proofread_count,
    }),
    { unit: 0, translated: 0, confirmed: 0 },
  );

  function close(): void {
    setDialog({ kind: "none" });
  }
  function complete(): void {
    close();
    onReload();
  }
  function home(): void {
    navigate({ to: "/home" }).catch(() => {
      setNotice("无法返回项目列表，请重试。");
    });
  }
  function continueWork(): void {
    if (detail.pages.length === 0) {
      return;
    }
    navigate({
      to: "/chapter/$chapterId/translator",
      params: { chapterId: detail.chapter.id },
    }).catch(() => {
      setNotice("无法打开翻校工作台，请重试。");
    });
  }
  function replace(page: Page): void {
    setDialog({
      kind: "image",
      target: {
        kind: "replace",
        chapter_id: detail.chapter.id,
        baseline: pageBaseline(page),
      },
    });
  }

  return (
    <>
      <ChapterLayout
        sidebar={
          <ChapterSidebar
            title={detail.comic.title}
            subtitle={detail.chapter.title}
            author={detail.comic.author}
            pageCount={detail.pages.length}
            unitCount={counts.unit}
            translatedCount={counts.translated}
            confirmedCount={counts.confirmed}
            cover={
              first === undefined ? (
                <div className="flex h-full min-h-24 items-center justify-center bg-muted text-xs text-muted-foreground">
                  暂无封面
                </div>
              ) : (
                <ManagedImage
                  chapterId={detail.chapter.id}
                  pageId={first.page.id}
                  reference={first.page.image.reference}
                  kind="thumbnail"
                  alt={`${detail.chapter.title} 封面`}
                  interactive={true}
                />
              )
            }
            busy={false}
            onBack={home}
            onContinue={continueWork}
            onExport={() => {
              setDialog({ kind: "export" });
            }}
            onEdit={() => {
              setDialog({ kind: "metadata" });
            }}
          />
        }
      >
        <div className="mb-3 flex flex-wrap items-center gap-2">
          <IconButton
            label="添加图片"
            onClick={() => {
              setDialog({
                kind: "image",
                target: {
                  kind: "append",
                  chapter_id: detail.chapter.id,
                  baseline: detail.pages.map((info) => pageBaseline(info.page)),
                },
              });
            }}
          >
            <ImagePlus className="size-3.5" aria-hidden="true" />
          </IconButton>
          <IconButton
            label="管理页面"
            disabled={detail.pages.length === 0}
            onClick={() => {
              setDialog({ kind: "manage" });
            }}
          >
            <ArrowDownUp className="size-3.5" aria-hidden="true" />
          </IconButton>
          <IconButton
            label="导入译文"
            onClick={() => {
              setDialog({ kind: "import" });
            }}
          >
            <FileInput className="size-3.5" aria-hidden="true" />
          </IconButton>
          <IconButton
            label="删除项目"
            className="ml-auto"
            onClick={() => {
              setDialog({ kind: "delete" });
            }}
          >
            <Trash2 className="size-3.5" aria-hidden="true" />
          </IconButton>
        </div>
        {notice !== "" && (
          <p role="status" className="mb-3 text-xs text-muted-foreground">
            {notice}
          </p>
        )}
        {detail.pages.length === 0 ? (
          <StatePanel
            state="empty"
            message="暂无页面。添加本地图片后即可开始翻校。"
          />
        ) : (
          <div className="grid grid-cols-[repeat(auto-fill,minmax(min(100%,9rem),1fr))] gap-3">
            {detail.pages.map((info) => (
              <PageItem
                key={info.page.id}
                index={info.page.index}
                total={info.unit_count}
                translated={info.translated_count}
                confirmed={info.proofread_count}
                flagged={info.flagged_count}
                recent={detail.work_position?.page_id === info.page.id}
                image={
                  <ManagedImage
                    chapterId={detail.chapter.id}
                    pageId={info.page.id}
                    reference={info.page.image.reference}
                    kind="thumbnail"
                    alt={`第 ${String(info.page.index + 1)} 页`}
                    interactive={false}
                  />
                }
                onOpen={() => {
                  setDialog({ kind: "preview", pageId: info.page.id });
                }}
              />
            ))}
          </div>
        )}
      </ChapterLayout>
      {dialog.kind === "metadata" && (
        <ChapterMetadataDialog
          chapter={detail.chapter}
          onClose={close}
          onComplete={complete}
        />
      )}
      {dialog.kind === "delete" && (
        <ChapterDeleteDialog
          chapterId={detail.chapter.id}
          title={detail.chapter.title}
          onClose={close}
          onComplete={home}
        />
      )}
      {dialog.kind === "image" && (
        <ImageImportDialog
          target={dialog.target}
          onClose={close}
          onComplete={complete}
        />
      )}
      {dialog.kind === "manage" && (
        <PageManagementDialog
          detail={detail}
          onClose={close}
          onComplete={complete}
          onReplace={replace}
        />
      )}
      {dialog.kind === "preview" && (
        <PagePreviewDialog
          title={detail.chapter.title}
          pages={detail.pages}
          selectedId={dialog.pageId}
          onSelect={(pageId) => {
            setDialog({ kind: "preview", pageId });
          }}
          onClose={close}
          onReplace={(info) => {
            replace(info.page);
          }}
        />
      )}
      {dialog.kind === "import" && (
        <ArchiveImportDialog
          target={{ kind: "existing", chapter_id: detail.chapter.id }}
          onClose={close}
          onComplete={complete}
        />
      )}
      {dialog.kind === "export" && (
        <ArchiveExportDialog chapterId={detail.chapter.id} onClose={close} />
      )}
    </>
  );
}
