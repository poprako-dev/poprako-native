import {
  BookOpenText,
  ChevronDown,
  ChevronRight,
  FolderInput,
  Plus,
  Settings,
  Trash2,
} from "lucide-react";
import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import type { ReactElement } from "react";
import { commands, unwrap } from "@/bridge";
import type {
  ArchiveTarget,
  Comic,
  ComicInfo,
} from "@/bridge/generated/bindings";
import { ArchiveImportDialog } from "@/route/business/component/ArchiveImportDialog";
import { LibraryMetadataDialog } from "@/route/business/component/LibraryMetadataDialog";
import type { MetadataTarget } from "@/route/business/component/LibraryMetadataDialog";
import { PreferenceDialog } from "@/route/business/component/PreferenceDialog";
import { ChapterList } from "@/route/home/business/component/ChapterList";
import { ComicDeleteDialog } from "@/route/home/business/component/ComicDeleteDialog";
import { HomeLayout } from "@/route/home/business/component/HomeLayout";
import { QuickActions } from "@/route/home/business/component/QuickActions";
import { Button } from "@/shared/component/Button";
import { IconButton } from "@/shared/component/IconButton";
import { StatePanel } from "@/shared/component/StatePanel";

type Props = {
  comics: ComicInfo[];
  latest: ComicInfo | null;
  offset: number;
  expanded: ReadonlySet<string>;
  onToggle: (id: string) => void;
  onReload: () => void;
  onOffsetChange: (offset: number) => void;
};
export function HomeContent({
  comics,
  latest,
  offset,
  expanded,
  onToggle,
  onReload,
  onOffsetChange,
}: Props): ReactElement {
  const navigate = useNavigate();
  const [editing, setEditing] = useState<MetadataTarget | null>(null);
  const [deleting, setDeleting] = useState<Comic | null>(null);
  const [importing, setImporting] = useState<ArchiveTarget | null>(null);
  const [preparingChapter, setPreparingChapter] = useState(false);
  const [settings, setSettings] = useState(false);
  const [notice, setNotice] = useState("");
  function open(chapterId: string, continueWork: boolean): void {
    navigate({
      to: continueWork
        ? "/chapter/$chapterId/translator"
        : "/chapter/$chapterId",
      params: { chapterId },
    }).catch(() => {
      setNotice("无法打开章节，请重试。");
    });
  }
  function prepareChapter(comic: Comic, action: "create" | "import"): void {
    if (preparingChapter) return;
    setPreparingChapter(true);
    setNotice("");
    commands
      .listChapterInfos(comic.id)
      .then((result) => {
        const baseline = unwrap(result).map((info) => info.chapter.id);
        if (action === "create") {
          setEditing({
            kind: "new_chapter",
            comicId: comic.id,
            comicTitle: comic.title,
            baseline,
          });
        } else {
          setImporting({ kind: "new_chapter", comic_id: comic.id, baseline });
        }
      })
      .catch(() => {
        setNotice("无法读取漫画的章节信息，请重试新增或导入。");
      })
      .finally(() => {
        setPreparingChapter(false);
      });
  }
  function continueLatest(): void {
    const id = latest?.latest_chapter_id;
    if (id === null || id === undefined) return;
    commands
      .getChapterDetail(id)
      .then((result) => {
        const detail = unwrap(result);
        open(id, detail.pages.length > 0);
      })
      .catch(() => {
        setNotice("最近章节无法读取，请重试。");
      });
  }
  return (
    <>
      <HomeLayout
        action={
          <QuickActions
            latestTitle={latest?.comic.title ?? ""}
            canContinue={latest?.latest_chapter_id != null}
            onContinue={continueLatest}
            onCreate={() => {
              setEditing({ kind: "new_comic" });
            }}
            onImport={() => {
              setImporting({ kind: "new" });
            }}
          />
        }
      >
        <div className="flex justify-end">
          <IconButton
            label="全局术语库"
            onClick={() => {
              void navigate({ to: "/terminology" }).catch(() => {
                setNotice("无法打开术语库，请重试。");
              });
            }}
          >
            <BookOpenText size={16} strokeWidth={1.5} />
          </IconButton>
          <IconButton
            label="应用设置"
            onClick={() => {
              setSettings(true);
            }}
          >
            <Settings size={16} strokeWidth={1.5} />
          </IconButton>
        </div>
        {notice !== "" && (
          <p role="status" className="px-3 py-2 text-xs text-muted-foreground">
            {notice}
          </p>
        )}
        <div className="flex flex-col gap-2">
          {comics.map((info) => {
            const isExpanded = expanded.has(info.comic.id);
            return (
              <div key={info.comic.id} className="py-1">
                <div className="flex items-center gap-1 rounded-md border border-border bg-card pr-1 transition-colors hover:bg-accent focus-within:bg-accent">
                  <button
                    type="button"
                    aria-expanded={isExpanded}
                    onClick={() => {
                      onToggle(info.comic.id);
                    }}
                    className="flex min-h-12 min-w-0 flex-1 items-center gap-3 rounded-md px-2 py-3 text-left focus-visible:outline-2 focus-visible:outline-ring"
                  >
                    {isExpanded ? (
                      <ChevronDown
                        className="size-4 shrink-0"
                        aria-hidden="true"
                      />
                    ) : (
                      <ChevronRight
                        className="size-4 shrink-0"
                        aria-hidden="true"
                      />
                    )}
                    <span className="truncate text-base font-medium">
                      {info.comic.title}
                    </span>
                    <span className="text-xs text-muted-foreground">
                      {info.chapter_count} 章
                    </span>
                    {info.comic.author !== "" && (
                      <span className="truncate text-xs text-muted-foreground">
                        {info.comic.author}
                      </span>
                    )}
                  </button>
                  <IconButton
                    label={`在 ${info.comic.title} 中新增章节`}
                    disabled={preparingChapter}
                    onClick={() => {
                      prepareChapter(info.comic, "create");
                    }}
                  >
                    <Plus size={16} />
                  </IconButton>
                  <IconButton
                    label={`向 ${info.comic.title} 导入章节`}
                    disabled={preparingChapter}
                    onClick={() => {
                      prepareChapter(info.comic, "import");
                    }}
                  >
                    <FolderInput size={16} />
                  </IconButton>
                  <IconButton
                    label={`编辑漫画 ${info.comic.title}`}
                    onClick={() => {
                      setEditing({ kind: "comic", comic: info.comic });
                    }}
                  >
                    <Settings size={14} />
                  </IconButton>
                  <IconButton
                    label={`删除漫画 ${info.comic.title}`}
                    onClick={() => {
                      setDeleting(info.comic);
                    }}
                  >
                    <Trash2 size={14} />
                  </IconButton>
                </div>
                {isExpanded && (
                  <ChapterList
                    comic={info.comic}
                    onReload={onReload}
                    onOpen={open}
                  />
                )}
              </div>
            );
          })}
        </div>
        {comics.length === 0 && (
          <StatePanel
            state="empty"
            message="暂无漫画，新建漫画与首章开始翻校。"
          />
        )}
        {(offset > 0 || comics.length === 50) && (
          <nav
            aria-label="漫画列表分页"
            className="grid grid-cols-2 gap-3 py-4"
          >
            <Button
              variant="ghost"
              disabled={offset === 0}
              onClick={() => {
                onOffsetChange(Math.max(0, offset - 50));
              }}
            >
              上一页
            </Button>
            <Button
              variant="ghost"
              disabled={comics.length < 50}
              onClick={() => {
                onOffsetChange(offset + 50);
              }}
            >
              下一页
            </Button>
          </nav>
        )}
      </HomeLayout>
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
        <ComicDeleteDialog
          comic={deleting}
          onClose={() => {
            setDeleting(null);
          }}
          onComplete={onReload}
        />
      )}
      {importing !== null && (
        <ArchiveImportDialog
          target={importing}
          onClose={() => {
            setImporting(null);
          }}
          onComplete={(id) => {
            setImporting(null);
            open(id, false);
          }}
        />
      )}
      {settings && (
        <PreferenceDialog
          onClose={() => {
            setSettings(false);
          }}
        />
      )}
    </>
  );
}
