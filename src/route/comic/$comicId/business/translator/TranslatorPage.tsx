import { useCallback, useEffect, useRef, useState } from "react";
import type { ReactElement } from "react";
import { Link, useNavigate } from "@tanstack/react-router";
import { commands, unwrap } from "@/bridge";
import type { ComicDetail, EditorMode } from "@/bridge/generated/bindings";
import { StatePanel } from "@/shared/component/StatePanel";
import { Button } from "@/shared/component/Button";
import { createEditorSession } from "./editor-session";
import type { EditorSession } from "./editor-session";
import { EditorWorkspace } from "./EditorWorkspace";
import { LeaveGuard } from "./LeaveGuard";
import { createPositionWriter } from "./work-position";

type Loaded = { detail: ComicDetail; session: EditorSession };
type Resource =
  | { status: "loading" }
  | { status: "error"; message: string }
  | { status: "empty" }
  | { status: "ready"; value: Loaded };
type TranslatorPageProps = { comicId: string };

export function TranslatorPage({ comicId }: TranslatorPageProps): ReactElement {
  const [resource, setResource] = useState<Resource>({ status: "loading" });
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    let active = true;
    let session: EditorSession | undefined;
    async function load(): Promise<void> {
      try {
        const detail = unwrap(await commands.getComicDetail(comicId));
        const id =
          detail.pages.find(
            (entry) => entry.page.id === detail.work_position?.page_id,
          )?.page.id ?? detail.pages[0]?.page.id;
        if (!id) {
          if (active) setResource({ status: "empty" });
          return;
        }
        const page = unwrap(await commands.getPageEditor(comicId, id));
        if (!active) return;
        session = createEditorSession(
          page,
          detail.work_position?.mode ?? "translation",
          {
            save: async (snapshot) =>
              unwrap(await commands.savePageUnits(snapshot)),
          },
        );
        session.select(
          page.units.find((unit) => unit.id === detail.work_position?.unit_id)
            ?.id ?? "",
        );
        setResource({ status: "ready", value: { detail, session } });
      } catch (error) {
        if (active)
          setResource({
            status: "error",
            message:
              error instanceof Error ? error.message : "无法打开项目，请重试。",
          });
      }
    }
    void load();
    return () => {
      active = false;
      session?.dispose();
    };
  }, [comicId, attempt]);
  if (resource.status === "loading")
    return <StatePanel state="loading" message="正在打开翻校工作台…" />;
  if (resource.status === "error")
    return (
      <StatePanel
        state="error"
        message={resource.message}
        onRetry={() => {
          setAttempt(attempt + 1);
        }}
      />
    );
  if (resource.status === "empty")
    return (
      <div className="p-6 text-center">
        <StatePanel
          state="empty"
          message="项目还没有图片，请返回详情导入图片。"
        />
        <Link
          to="/comic/$comicId"
          params={{ comicId }}
          className="text-sm underline"
        >
          返回项目详情
        </Link>
      </div>
    );
  return (
    <LoadedTranslator
      key={resource.value.session.store.getState().baseline.page.id}
      initial={resource.value}
    />
  );
}

type LoadedTranslatorProps = { initial: Loaded };

function LoadedTranslator({ initial }: LoadedTranslatorProps): ReactElement {
  const [loaded, setLoaded] = useState(initial);
  const [error, setError] = useState("");
  const [failedPage, setFailedPage] = useState("");
  const [positionWriter] = useState(() =>
    createPositionWriter({
      save: async (position) => {
        unwrap(await commands.updateWorkPosition(position));
      },
    }),
  );
  const changing = useRef(false);
  const guardRef = useRef({
    run: (): Promise<boolean> => Promise.resolve(false),
  });
  const navigate = useNavigate();
  const comicId = loaded.detail.comic.id;
  const session = loaded.session;

  useEffect(() => {
    session.resume();
    return () => {
      session.dispose();
    };
  }, [session]);

  const savePosition = useCallback(async (): Promise<boolean> => {
    const state = session.store.getState();
    try {
      await positionWriter.save({
        comic_id: comicId,
        page_id: state.baseline.page.id,
        unit_id: state.baseline.units.some((unit) => unit.id === state.selected)
          ? state.selected
          : null,
        mode: state.mode,
        last_opened_at: Date.now(),
      });
      return true;
    } catch (failure) {
      setError(
        failure instanceof Error
          ? failure.message
          : "工作位置保存失败，请重试。",
      );
      return false;
    }
  }, [comicId, session, positionWriter]);

  useEffect(() => {
    queueMicrotask(() => {
      void savePosition();
    });
    return session.store.subscribe((state, previous) => {
      if (
        state.selected !== previous.selected ||
        state.mode !== previous.mode ||
        state.baseline !== previous.baseline
      )
        void savePosition();
    });
  }, [session, savePosition]);

  async function changePage(id: string): Promise<void> {
    if (changing.current || id === session.store.getState().baseline.page.id)
      return;
    changing.current = true;
    try {
      if (!(await guardRef.current.run())) return;
      const page = unwrap(await commands.getPageEditor(comicId, id));
      const detail = unwrap(await commands.getComicDetail(comicId));
      const mode: EditorMode = session.store.getState().mode;
      const next = createEditorSession(page, mode, {
        save: async (snapshot) =>
          unwrap(await commands.savePageUnits(snapshot)),
      });
      next.useSymbol(session.store.getState().recentSymbol);
      setLoaded({ detail, session: next });
      setError("");
      setFailedPage("");
    } catch (failure) {
      session.cancelLeave();
      setFailedPage(id);
      setError(
        failure instanceof Error
          ? failure.message
          : "目标页加载失败，当前页已保存，可以重试切页。",
      );
    } finally {
      changing.current = false;
    }
  }

  function openPage(id: string): void {
    void changePage(id);
  }

  async function refresh(): Promise<void> {
    const page = unwrap(
      await commands.getPageEditor(
        comicId,
        session.store.getState().baseline.page.id,
      ),
    );
    const detail = unwrap(await commands.getComicDetail(comicId));
    session.reload(page);
    setLoaded({ session, detail });
  }

  return (
    <div className="relative h-full min-h-0">
      <EditorWorkspace
        session={session}
        detail={loaded.detail}
        onPage={openPage}
        onRefresh={refresh}
        onExit={() => {
          void navigate({ to: "/comic/$comicId", params: { comicId } }).catch(
            () => {
              setError("暂时无法离开，请重试。");
            },
          );
        }}
      />
      <LeaveGuard
        session={session}
        guardRef={guardRef}
        savePosition={savePosition}
      />
      {error && (
        <div
          role="alert"
          className="absolute bottom-3 left-3 max-w-md rounded border border-destructive bg-background p-3 text-xs"
        >
          {error}
          <Button
            variant="ghost"
            onClick={() => {
              if (failedPage) openPage(failedPage);
              else
                void savePosition().then((success) => {
                  if (success) setError("");
                });
            }}
          >
            重试
          </Button>
          <Button
            variant="ghost"
            onClick={() => {
              setError("");
            }}
          >
            关闭提示
          </Button>
        </div>
      )}
    </div>
  );
}
