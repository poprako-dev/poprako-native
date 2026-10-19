import type { PointerEvent, ReactElement } from "react";
import type { UnitDraft } from "@/bridge/generated/bindings";
import type { EditorSession, EditorState } from "./editor-session";
import { Marker, MARKER_SIZE, PIN_OFFSET } from "./Marker";

type CanvasMarkerLayerProps = {
  state: EditorState;
  session: EditorSession;
  moving: UnitDraft | null;
  shown: UnitDraft | null;
  scale: number;
  opacity: number;
  editable: boolean;
  dimmed: boolean;
  onStart: (event: PointerEvent<HTMLElement>, unit: UnitDraft | null) => void;
  onMove: (event: PointerEvent<HTMLElement>) => void;
  onEnd: (event: PointerEvent<HTMLElement>) => void;
  onDelete: (id: string) => void;
  onHover: (id: string) => void;
};

export function CanvasMarkerLayer({
  state,
  session,
  moving,
  shown,
  scale,
  opacity,
  editable,
  dimmed,
  onStart,
  onMove,
  onEnd,
  onDelete,
  onHover,
}: CanvasMarkerLayerProps): ReactElement {
  return (
    <>
      {state.draft.map((unit, index) => {
        const position = moving?.id === unit.id ? moving : unit;
        const completed =
          state.mode === "translation"
            ? Boolean(unit.translated_text.trim())
            : unit.is_proofread;
        return (
          <button
            type="button"
            key={unit.id}
            aria-label={`标记 ${String(index + 1)}，${unit.is_bubble ? "框内" : "框外"}`}
            className={`absolute size-8 border-0 bg-transparent p-0 ${unit.id === state.selected ? "z-30" : "z-10"} ${moving?.id === unit.id ? "cursor-grabbing" : "cursor-pointer"}`}
            style={{
              left: `${String(position.x_coord * 100)}%`,
              top: `${String(position.y_coord * 100)}%`,
              opacity,
              transformOrigin: "0 0",
              transform: `scale(${String(1 / scale)}) translate(-${String(MARKER_SIZE / 2)}px,-${String(PIN_OFFSET)}px)`,
            }}
            onPointerDown={(event) => {
              onStart(event, unit);
            }}
            onPointerMove={onMove}
            onPointerUp={onEnd}
            onClick={(event) => {
              if (event.detail === 0) session.select(unit.id);
            }}
            onDoubleClick={(event) => {
              event.stopPropagation();
              if (editable)
                session.edit(unit.id, { is_bubble: !unit.is_bubble });
            }}
            onContextMenu={(event) => {
              event.stopPropagation();
              event.preventDefault();
              if (editable) onDelete(unit.id);
            }}
            onMouseEnter={() => {
              onHover(unit.id);
            }}
            onMouseLeave={() => {
              onHover("");
            }}
          >
            <Marker
              index={index}
              bubble={unit.is_bubble}
              completed={completed}
              selected={unit.id === state.selected}
              dragging={moving?.id === unit.id}
              dimmed={dimmed}
            />
          </button>
        );
      })}
      {shown && (shown.proofread_text || shown.translated_text) && !moving && (
        <div
          className="pointer-events-none absolute z-50 rounded-sm border border-white/10 bg-slate-800/90 px-2 py-1 text-xs whitespace-pre text-slate-50 shadow-xl backdrop-blur-md"
          style={{
            left: `${String(shown.x_coord * 100)}%`,
            top: `${String(shown.y_coord * 100)}%`,
            transformOrigin: "0 0",
            transform: `scale(${String(1 / scale)}) translate(28px,${String(MARKER_SIZE - PIN_OFFSET)}px) translateY(-100%)`,
          }}
        >
          {shown.proofread_text || shown.translated_text}
        </div>
      )}
    </>
  );
}
