import { Settings, Trash2 } from "lucide-react";
import { IconButton } from "@/shared/component/IconButton";
import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import type { ReactElement } from "react";
import { commands, NativeCommandError, unwrap } from "@/bridge";
import type { ComicInfo, ComicMetadata } from "@/bridge/generated/bindings";
import { ArchiveImportDialog } from "@/route/business/component/ArchiveImportDialog";
import { ComicDeleteDialog } from "@/route/business/component/ComicDeleteDialog";
import { MetadataForm } from "@/route/business/component/MetadataForm";
import { PreferenceDialog } from "@/route/business/component/PreferenceDialog";
import { ComicItem } from "@/route/home/business/component/ComicItem";
import { HomeLayout } from "@/route/home/business/component/HomeLayout";
import { QuickActions } from "@/route/home/business/component/QuickActions";
import { Button } from "@/shared/component/Button";
import { Dialog } from "@/shared/component/Dialog";
import { StatePanel } from "@/shared/component/StatePanel";

type HomeContentProps = {
  comics: ComicInfo[];
  offset: number;
  latest: ComicInfo | null;
  onReload: () => void;
  onOffsetChange: (offset: number) => void;
};

export function HomeContent({
  comics,
  offset,
  latest,
  onReload,
  onOffsetChange,
}: HomeContentProps): ReactElement {
  const navigate = useNavigate();
  const [creating, setCreating] = useState(false);
  const [settings, setSettings] = useState(false);
  const [importing, setImporting] = useState(false);
  const [deleting, setDeleting] = useState<ComicInfo | null>(null);
  const [pending, setPending] = useState<ComicMetadata | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [now] = useState(Date.now);

  function open(comicId: string, continueWork: boolean): void {
    navigate({
      to: continueWork ? "/comic/$comicId/translator" : "/comic/$comicId",
      params: { comicId },
    }).catch(() => {
      setNotice("无法打开项目，请重试。");
    });
  }

  async function create(
    title: string,
    subtitle: string,
    author: string,
  ): Promise<void> {
    setBusy(true);
    setError("");
    const metadata = pending ?? { title, subtitle, author };
    try {
      const comic = unwrap(await commands.createComic(metadata));
      setCreating(false);
      setPending(null);
      open(comic.id, false);
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
          : "创建结果未确认，请保留窗口并核实。",
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <HomeLayout
        action={
          <QuickActions
            latestTitle={latest?.comic.title ?? ""}
            canContinue={latest !== null}
            busy={busy}
            onContinue={() => {
              if (latest !== null) {
                open(latest.comic.id, latest.page_count > 0);
              }
            }}
            onCreate={() => {
              setError("");
              setCreating(true);
            }}
            onImport={() => {
              setImporting(true);
            }}
          />
        }
      >
        <div className="flex justify-end">
          <IconButton
            label="应用设置"
            onClick={() => {
              setSettings(true);
            }}
          >
            <Settings size={16} strokeWidth={1.5} aria-hidden="true" />
          </IconButton>
        </div>
        {notice !== "" && (
          <p role="status" className="px-3 py-2 text-xs text-muted-foreground">
            {notice}
          </p>
        )}
        <div className="flex flex-col gap-2">
          {comics.map((info) => {
            const activityAt = info.last_opened_at ?? info.comic.updated_at;
            const days = Math.max(0, (now - activityAt) / 86400000);
            return (
              <div
                key={info.comic.id}
                className="group flex items-center gap-1 rounded-md border border-border bg-card pr-1 transition-colors hover:bg-accent focus-within:bg-accent"
              >
                <ComicItem
                  title={info.comic.title}
                  subtitle={info.comic.subtitle}
                  activityAt={activityAt}
                  activity={
                    days <= 7 ? "recent" : days <= 30 ? "past" : "inactive"
                  }
                  pageCount={info.page_count}
                  unitCount={info.unit_count}
                  translatedCount={info.translated_count}
                  confirmedCount={info.proofread_count}
                  onOpen={() => {
                    open(info.comic.id, false);
                  }}
                />
                <IconButton
                  className="opacity-0 transition-opacity group-hover:text-foreground group-hover:opacity-100 group-focus-within:text-foreground group-focus-within:opacity-100 hover:bg-transparent focus-visible:opacity-100"
                  label={`删除项目 ${info.comic.title}`}
                  onClick={() => {
                    setDeleting(info);
                  }}
                >
                  <Trash2 size={14} strokeWidth={1.5} aria-hidden="true" />
                </IconButton>
              </div>
            );
          })}
        </div>
        {comics.length === 0 && (
          <StatePanel
            state="empty"
            message="暂无项目，创建一个项目开始翻校。"
          />
        )}
        {(offset > 0 || comics.length === 50) && (
          <nav
            aria-label="项目列表分页"
            className="flex justify-center gap-3 py-4"
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
      <Dialog
        open={creating}
        onOpenChange={(openState) => {
          if (!busy && pending === null) {
            setCreating(openState);
          }
        }}
        title="创建项目"
        description=""
        className=""
      >
        <MetadataForm
          title=""
          subtitle=""
          author=""
          busy={busy || pending !== null}
          error={error}
          submitLabel="创建项目"
          onCancel={() => {
            setCreating(false);
          }}
          onSubmit={(title, subtitle, author) => {
            create(title, subtitle, author).catch(() => {
              setError("创建未完成，请重试。");
            });
          }}
        />
        {pending !== null && (
          <Button
            variant="primary"
            disabled={busy}
            onClick={() => {
              create(pending.title, pending.subtitle, pending.author).catch(
                () => {
                  setError("创建结果未确认，请核实。");
                },
              );
            }}
          >
            核实原创建结果
          </Button>
        )}
      </Dialog>
      {settings && (
        <PreferenceDialog
          onClose={() => {
            setSettings(false);
          }}
        />
      )}
      {importing && (
        <ArchiveImportDialog
          target={{ kind: "new" }}
          onClose={() => {
            setImporting(false);
          }}
          onComplete={(comicId) => {
            setImporting(false);
            open(comicId, false);
          }}
        />
      )}
      {deleting !== null && (
        <ComicDeleteDialog
          comicId={deleting.comic.id}
          title={deleting.comic.title}
          onClose={() => {
            setDeleting(null);
          }}
          onComplete={() => {
            setDeleting(null);
            onReload();
          }}
        />
      )}
    </>
  );
}
