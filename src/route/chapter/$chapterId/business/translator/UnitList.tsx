import { ListCheck } from "lucide-react";
import { useRef } from "react";
import type { ReactElement } from "react";
import type { SpecialCharacter } from "@/bridge/generated/bindings";
import { normalizedDraft } from "./editor-session";
import type { EditorSession, EditorState } from "./editor-session";
import type { SymbolRequest } from "./SpecialCharsBar";
import type { SpecialCharsBarController } from "./use-detachable-special-chars-bar";
import { useUnitReorder } from "./use-unit-reorder";
import { UnitItem } from "./UnitItem";

type UnitListProps = {
  state: EditorState;
  session: EditorSession;
  flagged: boolean;
  characters: SpecialCharacter[];
  characterBar: SpecialCharsBarController;
  symbolRequest: SymbolRequest | null;
  onSymbolInserted?: (id: string) => void;
  onDelete: (id: string) => void;
};

export function UnitList({
  state,
  session,
  flagged,
  characters,
  characterBar,
  symbolRequest,
  onSymbolInserted = () => undefined,
  onDelete,
}: UnitListProps): ReactElement {
  const ref = useRef<HTMLDivElement>(null);
  const locked = state.locked || state.uncertain;
  const readonly = state.mode === "readonly" || locked;
  const visible = state.draft.filter((unit) => !flagged || unit.is_flagged);
  const allConfirmed =
    state.draft.length > 0 && state.draft.every((unit) => unit.is_proofread);
  function reorderVisible(id: string, target: number): void {
    const current = session.store.getState().draft;
    const units = current.filter((unit) => !flagged || unit.is_flagged);
    const from = units.findIndex((unit) => unit.id === id);
    const [moved] = units.splice(from, 1);
    if (!moved || from < 0) return;
    units.splice(target, 0, moved);
    const included = new Set(units.map((unit) => unit.id));
    let index = 0;
    session.replace(
      current.map((unit) =>
        included.has(unit.id) ? (units[index++] ?? unit) : unit,
      ),
    );
  }
  function reorder(from: string, to: string): void {
    const target = visible.findIndex((unit) => unit.id === to);
    if (target >= 0) {
      reorderVisible(from, target);
      return;
    }
    const current = session.store.getState().draft;
    const sourceIndex = current.findIndex((unit) => unit.id === from);
    const targetIndex = current.findIndex((unit) => unit.id === to);
    if (sourceIndex < 0 || targetIndex < 0) return;
    const direction = targetIndex < sourceIndex ? -1 : 1;
    const next = visible.findIndex((unit) => unit.id === from) + direction;
    if (next >= 0 && next < visible.length) reorderVisible(from, next);
  }
  const drag = useUnitReorder({
    units: visible,
    listRef: ref,
    enabled: !readonly,
    onActivateUnit: (id) => {
      const unit = session.store
        .getState()
        .draft.find((entry) => entry.id === id);
      if (unit) session.edit(id, { is_bubble: !unit.is_bubble });
    },
    onReorderUnit: reorderVisible,
    onGesture: (active) => {
      session.suspend(active, "gesture");
    },
  });
  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden bg-panel">
      <div
        ref={ref}
        className={`min-h-0 flex-1 overflow-y-auto ${drag.draggingUnitId ? "select-none" : ""}`}
      >
        {visible.length === 0 && (
          <div role="status" className="p-5 text-center text-xs text-stone-400">
            {flagged ? "没有关注单元" : "暂无单元"}
          </div>
        )}
        {drag.orderedUnits.map((unit) => {
          const baseline = state.baseline.units.find(
            (entry) => entry.id === unit.id,
          );
          const pending =
            baseline?.index !==
              state.draft.findIndex((entry) => entry.id === unit.id) ||
            JSON.stringify(normalizedDraft(unit)) !==
              JSON.stringify(normalizedDraft(baseline));
          return (
            <UnitItem
              key={unit.id}
              unit={unit}
              index={state.draft.findIndex((entry) => entry.id === unit.id)}
              selected={state.selected === unit.id}
              mode={state.mode}
              locked={locked}
              session={session}
              characters={characters}
              characterBar={characterBar}
              symbolRequest={symbolRequest}
              onSymbolInserted={onSymbolInserted}
              pending={pending}
              dragging={drag.draggingUnitId === unit.id}
              dimmed={
                drag.draggingUnitId !== null && drag.draggingUnitId !== unit.id
              }
              onIndexPointerDown={drag.handleIndexPointerDown}
              onDelete={onDelete}
              onReorder={reorder}
            />
          );
        })}
      </div>
      {state.mode === "proofreading" && (
        <button
          type="button"
          title={allConfirmed ? "取消整页确认" : "确认整页校对"}
          aria-label={allConfirmed ? "取消整页确认" : "确认整页校对"}
          disabled={locked || state.draft.length === 0}
          onClick={() => {
            session.replace(
              state.draft.map((unit) => ({
                ...unit,
                is_proofread: !allConfirmed,
              })),
            );
          }}
          className={`flex w-full shrink-0 items-center justify-center border-t-2 border-gray-300 bg-panel py-2 disabled:opacity-40 ${allConfirmed ? "text-red-600 hover:bg-red-50" : "text-gray-700 hover:bg-stone-200"}`}
        >
          <ListCheck size={22} aria-hidden="true" />
        </button>
      )}
    </div>
  );
}
