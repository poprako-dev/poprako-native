import { useState } from "react";
import type { ReactElement } from "react";
import { useStore } from "zustand";
import { SquareArrowRight } from "lucide-react";
import type { ComicDetail } from "@/bridge/generated/bindings";
import { usePreference } from "@/route/business/preference-context";
import { PreferenceDialog } from "@/route/business/component/PreferenceDialog";
import { Button } from "@/shared/component/Button";
import { IconButton } from "@/shared/component/IconButton";
import { Appearance } from "@/shared/component/Appearance";
import { Dialog } from "@/shared/component/Dialog";
import type { EditorSession } from "./editor-session";
import { Canvas } from "./Canvas";
import { UnitList } from "./UnitList";
import { EditorToolbar } from "./EditorToolbar";
import { EditorToolbox } from "./EditorToolbox";
import { EditorPaginator } from "./EditorPaginator";
import { PageStatisticActions } from "./PageStatisticActions";
import { pageStatistics } from "./page-statistic";
import { usePageImage } from "./use-page-image";
import { SearchDialog } from "./SearchDialog";
import { useDetachableSpecialCharsBar } from "./use-detachable-special-chars-bar";
import { SpecialCharsBar } from "./SpecialCharsBar";
import type { SymbolRequest } from "./SpecialCharsBar";
import { useEditorShortcuts } from "./use-editor-shortcuts";

type EditorWorkspaceProps = {
  session: EditorSession;
  detail: ComicDetail;
  onPage: (id: string) => void;
  onExit: () => void;
  onRefresh: () => Promise<void>;
};

export function EditorWorkspace({
  session,
  detail,
  onPage,
  onExit,
  onRefresh,
}: EditorWorkspaceProps): ReactElement {
  const state = useStore(session.store);
  const { preference, save } = usePreference();
  const resource = usePageImage(detail.comic.id, state.baseline.page.id);
  const [flagged, setFlagged] = useState(false);
  const [preview, setPreview] = useState(true);
  const [highQuality, setHighQuality] = useState(false);
  const [creationEnabled, setCreationEnabled] = useState(true);
  const [deleting, setDeleting] = useState("");
  const [preferenceError, setPreferenceError] = useState("");
  const [settings, setSettings] = useState<
    "general" | "character" | "shortcut" | null
  >(null);
  const [search, setSearch] = useState(false);
  const [symbolRequest, setSymbolRequest] = useState<SymbolRequest | null>(
    null,
  );
  const locked = state.locked || state.uncertain;
  const { floatingRef, ...characterBar } = useDetachableSpecialCharsBar({
    enabled: state.mode !== "readonly" && !locked && state.selected !== "",
    interactionKey: `${state.mode}:${state.selected}`,
  });
  const stats = pageStatistics(detail.pages, state);

  async function toggleRelocation(): Promise<void> {
    try {
      await save({
        ...preference,
        relocation_enabled: !preference.relocation_enabled,
      });
      setPreferenceError("");
    } catch (error) {
      setPreferenceError(
        error instanceof Error ? error.message : "设置保存失败，请重试。",
      );
    }
  }

  function requestSymbol(text: string): void {
    setSymbolRequest({
      id: crypto.randomUUID(),
      unitId: session.store.getState().selected,
      text,
    });
  }

  function togglePreview(): void {
    setPreview((visible) => !visible);
  }

  useEditorShortcuts({
    session,
    detail,
    preference,
    disabled: settings !== null || search || deleting !== "",
    onPage,
    onRelocation: () => {
      void toggleRelocation();
    },
    onPreview: togglePreview,
    onSymbol: requestSymbol,
  });

  return (
    <Appearance
      surface="translator"
      className="flex h-full w-full overflow-hidden portrait:flex-col"
    >
      <div className="relative min-h-0 min-w-0 flex-1 bg-stone-100">
        <Canvas
          key={state.baseline.page.id}
          state={state}
          session={session}
          imageUrl={resource.url}
          imageError={resource.error}
          opacity={preference.marker_opacity}
          relocation={preference.relocation_enabled}
          preview={preview}
          highQuality={highQuality}
          creationEnabled={creationEnabled}
          onDelete={setDeleting}
        />
        <div className="absolute top-2 left-2 flex items-center gap-2">
          <EditorToolbox
            readonly={state.mode === "readonly"}
            locked={locked}
            flagged={flagged}
            onFlagged={() => {
              setFlagged(!flagged);
            }}
            onSettings={setSettings}
            onSearch={() => {
              setSearch(true);
            }}
          />
          <IconButton
            label="退出翻校工作台"
            onClick={onExit}
            className="translator-floating-button"
          >
            <SquareArrowRight size={20} aria-hidden="true" />
          </IconButton>
        </div>
        <div className="absolute top-2 right-2">
          <EditorPaginator
            pages={stats}
            currentId={state.baseline.page.id}
            locked={locked}
            onPage={onPage}
          />
        </div>
        {state.mode === "readonly" && (
          <div className="absolute right-2 bottom-2">
            <PageStatisticActions
              pages={stats}
              currentId={state.baseline.page.id}
              locked={locked || state.busy}
              onPage={onPage}
            />
          </div>
        )}
      </div>
      <aside
        aria-label="单元编辑栏"
        className="flex min-h-0 shrink-0 flex-col overflow-hidden border-unit-border bg-panel portrait:h-2/5 portrait:border-t landscape:w-1/3 landscape:min-w-95 landscape:border-l sm:portrait:h-50"
      >
        <EditorToolbar
          state={state}
          session={session}
          relocation={preference.relocation_enabled}
          onRelocation={() => {
            void toggleRelocation();
          }}
          preview={preview}
          onPreview={togglePreview}
          highQuality={highQuality}
          onQuality={() => {
            setHighQuality(!highQuality);
          }}
          creationEnabled={creationEnabled}
          onCreation={() => {
            setCreationEnabled(!creationEnabled);
          }}
        />
        {(state.error || preferenceError) && (
          <div
            role="alert"
            className="border-b border-unit-border p-2 text-xs text-destructive"
          >
            {state.error || preferenceError}
            {state.error && (
              <Button
                variant="ghost"
                onClick={() => {
                  void session.save();
                }}
              >
                {state.uncertain ? "核实写入结果" : "重试保存"}
              </Button>
            )}
          </div>
        )}
        <UnitList
          key={state.baseline.page.id}
          state={state}
          session={session}
          flagged={flagged}
          characters={preference.special_character}
          characterBar={characterBar}
          symbolRequest={symbolRequest}
          onSymbolInserted={(id) => {
            setSymbolRequest((request) =>
              request?.id === id ? null : request,
            );
          }}
          onDelete={setDeleting}
        />
      </aside>
      {characterBar.position && characterBar.isEnabled && (
        <div
          ref={floatingRef}
          className="fixed z-40"
          style={{
            left: characterBar.position.x,
            top: characterBar.position.y,
            width: characterBar.position.width,
          }}
        >
          <SpecialCharsBar
            characters={preference.special_character}
            controller={characterBar}
            floating={true}
            disabled={!characterBar.isEnabled}
            onInsert={requestSymbol}
          />
        </div>
      )}
      {settings && (
        <PreferenceDialog
          initialTab={settings}
          onClose={() => {
            setSettings(null);
          }}
        />
      )}
      {search && (
        <SearchDialog
          session={session}
          onRefresh={onRefresh}
          onClose={() => {
            setSearch(false);
          }}
        />
      )}
      <Dialog
        open={deleting !== ""}
        onOpenChange={(open) => {
          if (!open) setDeleting("");
        }}
        title="删除单元？"
        description="标记及翻译、校对文本将被删除，保存后无法恢复。"
        className=""
      >
        <div className="flex justify-end gap-2">
          <Button
            variant="ghost"
            onClick={() => {
              setDeleting("");
            }}
          >
            取消
          </Button>
          <Button
            variant="danger"
            disabled={locked || state.mode === "readonly"}
            onClick={() => {
              session.replace(
                state.draft.filter((unit) => unit.id !== deleting),
              );
              setDeleting("");
            }}
          >
            删除单元
          </Button>
        </div>
      </Dialog>
    </Appearance>
  );
}
