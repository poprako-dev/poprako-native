import { useCallback, useLayoutEffect, useRef } from "react";
import type { PointerEvent, ReactElement } from "react";
import { Check, Copy, Star, X } from "lucide-react";
import type {
  EditorMode,
  SpecialCharacter,
  UnitDraft,
} from "@/bridge/generated/bindings";
import type { EditorSession } from "./editor-session";
import { TextDifference } from "./TextDifference";
import { AutoResizeTextarea } from "./AutoResizeTextarea";
import { SpecialCharsBar } from "./SpecialCharsBar";
import type { SymbolRequest } from "./SpecialCharsBar";
import type { SpecialCharsBarController } from "./use-detachable-special-chars-bar";

type UnitItemProps = {
  unit: UnitDraft;
  index: number;
  selected: boolean;
  mode: EditorMode;
  locked: boolean;
  session: EditorSession;
  characters: SpecialCharacter[];
  characterBar: SpecialCharsBarController | null;
  symbolRequest: SymbolRequest | null;
  onSymbolInserted?: (id: string) => void;
  pending: boolean;
  dragging: boolean;
  dimmed: boolean;
  onIndexPointerDown: (
    event: PointerEvent<HTMLButtonElement>,
    id: string,
  ) => void;
  onDelete: (id: string) => void;
  onReorder: (from: string, to: string) => void;
};

export function UnitItem({
  unit,
  index,
  selected,
  mode,
  locked,
  session,
  characters,
  characterBar,
  symbolRequest,
  onSymbolInserted,
  pending,
  dragging,
  dimmed,
  onIndexPointerDown,
  onDelete,
  onReorder,
}: UnitItemProps): ReactElement {
  const input = useRef<HTMLTextAreaElement>(null);
  const row = useRef<HTMLDivElement>(null);
  const lastSymbol = useRef("");
  const insertion = useRef<{ cursor: number; active: Element | null } | null>(
    null,
  );
  const readonly = mode === "readonly" || locked;
  const proofreading = mode === "proofreading";
  const field = proofreading ? "proofread_text" : "translated_text";
  const showInput = !proofreading || selected || unit.proofread_text.length > 0;
  const complete =
    mode === "translation"
      ? unit.translated_text.trim().length > 0
      : unit.is_proofread;

  useLayoutEffect(() => {
    if (!selected) {
      if (document.activeElement === input.current) input.current?.blur();
      return;
    }
    if (session.store.getState().selected !== unit.id) return;
    row.current?.scrollIntoView({ block: "nearest" });
    const textarea = input.current;
    if (!readonly && textarea && document.activeElement !== textarea) {
      textarea.focus({ preventScroll: true });
      textarea.setSelectionRange(textarea.value.length, textarea.value.length);
    }
  }, [selected, readonly, mode, session, unit.id]);

  useLayoutEffect(() => {
    const pendingInsertion = insertion.current;
    if (!pendingInsertion) return;
    insertion.current = null;
    const textarea = input.current;
    if (
      !textarea ||
      readonly ||
      !selected ||
      session.store.getState().selected !== unit.id
    )
      return;
    if (
      document.activeElement !== textarea &&
      document.activeElement !== pendingInsertion.active
    )
      return;
    textarea.focus({ preventScroll: true });
    textarea.setSelectionRange(
      pendingInsertion.cursor,
      pendingInsertion.cursor,
    );
  }, [unit, readonly, selected, session]);

  const insert = useCallback(
    (text: string): void => {
      const textarea = input.current;
      if (!textarea || readonly || !selected) return;
      session.useSymbol(text);
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      insertion.current = {
        cursor: start + text.length,
        active: document.activeElement,
      };
      session.edit(unit.id, {
        [field]: unit[field].slice(0, start) + text + unit[field].slice(end),
      });
    },
    [readonly, selected, session, unit, field],
  );
  useLayoutEffect(() => {
    if (
      symbolRequest?.unitId !== unit.id ||
      symbolRequest.id === lastSymbol.current ||
      !selected ||
      readonly
    )
      return;
    lastSymbol.current = symbolRequest.id;
    insert(symbolRequest.text);
    onSymbolInserted?.(symbolRequest.id);
  }, [symbolRequest, unit.id, selected, readonly, insert, onSymbolInserted]);

  return (
    <div
      ref={row}
      data-unit-id={unit.id}
      className={`relative flex cursor-text items-stretch border-y border-unit-border transition-all duration-75 first:border-t-0 last:border-b-0 ${selected ? "z-10 bg-unit-selected" : "bg-transparent hover:bg-stone-100/70"} ${dimmed ? "bg-stone-100/60 opacity-40 grayscale" : ""} ${dragging ? "z-20 bg-panel outline outline-1 outline-brand-leaf shadow-md" : ""}`}
    >
      {dragging && (
        <div
          aria-hidden="true"
          className="pointer-events-none absolute -top-0.5 right-0 left-0 z-30 h-1 rounded-full bg-brand-leaf shadow-md"
        />
      )}
      <div
        className={`shrink-0 border-l-4 ${unit.is_bubble ? "border-unit-bubble" : "border-unit-note"}`}
      />
      <button
        type="button"
        className={`flex w-8 shrink-0 touch-none items-center justify-center font-mono text-xs font-bold tracking-tighter select-none hover:bg-stone-200/70 ${readonly ? "cursor-pointer" : "cursor-grab active:cursor-grabbing"} ${selected || dragging ? "text-stone-500" : "text-stone-300 hover:text-stone-500"}`}
        title={readonly ? "选择单元" : "轻点切换框内外，拖动调整顺序"}
        aria-label={`单元 ${String(index + 1)}，${readonly ? "选择单元" : "轻点切换框内外，拖动调整顺序"}`}
        aria-pressed={readonly ? undefined : unit.is_bubble}
        onPointerDown={(event) => {
          if (!readonly) onIndexPointerDown(event, unit.id);
        }}
        onClick={(event) => {
          if (readonly) session.select(unit.id);
          else if (event.detail === 0)
            session.edit(unit.id, { is_bubble: !unit.is_bubble });
        }}
        onContextMenu={(event) => {
          event.preventDefault();
        }}
        onKeyDown={(event) => {
          if (readonly || event.nativeEvent.isComposing) return;
          if (event.key === "Delete" || event.key === "Backspace") {
            event.preventDefault();
            onDelete(unit.id);
          }
          if (
            (event.ctrlKey || event.metaKey) &&
            (event.key === "ArrowUp" || event.key === "ArrowDown")
          ) {
            const neighbor =
              session.store.getState().draft[
                index + (event.key === "ArrowUp" ? -1 : 1)
              ];
            if (neighbor) {
              event.preventDefault();
              onReorder(unit.id, neighbor.id);
            }
          }
        }}
      >
        {index + 1}
      </button>
      <div className="flex min-w-0 flex-1 flex-col justify-center px-2 py-2">
        <div className="flex items-start gap-1">
          <div className="min-w-0 flex-1">
            {mode === "readonly" && (
              <TextDifference
                translated={unit.translated_text}
                proofread={unit.proofread_text}
                selected={selected}
                onSelect={() => {
                  session.select(unit.id);
                }}
              />
            )}
            {proofreading && (
              <button
                type="button"
                className={`w-full whitespace-pre-wrap text-left text-base leading-relaxed ${unit.proofread_text ? "text-gray-400" : selected ? "text-gray-900" : "text-gray-700"}`}
                onClick={() => {
                  session.select(unit.id);
                }}
              >
                {unit.translated_text || (
                  <span className="text-gray-300">无翻译内容</span>
                )}
              </button>
            )}
            {mode === "translation" && (
              <AutoResizeTextarea
                inputRef={input}
                aria-label={`翻译 ${String(index + 1)}`}
                rows={1}
                value={unit.translated_text}
                readOnly={readonly}
                placeholder="点击输入翻译..."
                className={`w-full resize-none overflow-hidden bg-transparent text-base leading-relaxed outline-none placeholder:text-gray-300 ${selected ? "text-gray-900" : "text-gray-700"}`}
                onFocus={() => {
                  session.select(unit.id);
                }}
                onCompositionStart={() => {
                  session.suspend(true);
                }}
                onCompositionEnd={() => {
                  session.suspend(false);
                }}
                onChange={(event) => {
                  session.edit(unit.id, {
                    translated_text: event.target.value,
                  });
                }}
              />
            )}
          </div>
          {mode !== "readonly" && (
            <button
              type="button"
              className={`flex size-7 shrink-0 items-center justify-center rounded transition-colors hover:bg-stone-200/70 disabled:opacity-50 ${unit.is_flagged ? "text-status-flag" : "text-stone-400"}`}
              title={unit.is_flagged ? "取消关注" : "关注单元"}
              aria-label={unit.is_flagged ? "取消关注" : "关注单元"}
              aria-pressed={unit.is_flagged}
              disabled={readonly}
              onPointerDown={(event) => {
                if (event.pointerType === "mouse") event.preventDefault();
                event.stopPropagation();
              }}
              onClick={(event) => {
                event.stopPropagation();
                session.edit(unit.id, { is_flagged: !unit.is_flagged });
              }}
            >
              <Star
                size={16}
                fill={unit.is_flagged ? "currentColor" : "none"}
                aria-hidden="true"
              />
            </button>
          )}
          <div className="flex size-7 shrink-0 items-center justify-center rounded p-1">
            <span
              role="img"
              aria-label={`${complete ? "已完成" : "未完成"}${pending ? "，有未保存草稿" : ""}`}
              title={pending ? "有未保存草稿" : complete ? "已完成" : "未完成"}
              className={`size-2.5 rounded-full ${pending ? "border-[2.5px] border-current" : "bg-current"} ${complete ? "text-brand-leaf" : "text-gray-200"}`}
            />
          </div>
        </div>
        {proofreading && showInput && (
          <>
            <div className="my-1 mr-10 h-[1.5px] bg-gray-300" />
            <div className="flex items-start gap-1">
              <AutoResizeTextarea
                inputRef={input}
                aria-label={`校对 ${String(index + 1)}`}
                rows={1}
                value={unit.proofread_text}
                readOnly={readonly}
                placeholder="输入校对..."
                className={`min-w-0 flex-1 resize-none overflow-hidden bg-transparent text-base leading-relaxed outline-none placeholder:text-gray-300 ${selected ? "text-gray-900" : "text-gray-700"}`}
                onFocus={() => {
                  session.select(unit.id);
                }}
                onCompositionStart={() => {
                  session.suspend(true);
                }}
                onCompositionEnd={() => {
                  session.suspend(false);
                }}
                onChange={(event) => {
                  session.edit(unit.id, { proofread_text: event.target.value });
                }}
              />
              <div className="size-7 shrink-0">
                {!readonly && (
                  <button
                    type="button"
                    title="从初翻复制"
                    aria-label="复制翻译到空校对"
                    disabled={
                      unit.proofread_text.trim().length > 0 ||
                      !unit.translated_text.trim()
                    }
                    className={`rounded p-1 text-gray-400 transition-colors hover:text-green-600 disabled:invisible ${unit.proofread_text.trim().length > 0 ? "invisible" : ""}`}
                    onPointerDown={(event) => {
                      event.preventDefault();
                    }}
                    onClick={() => {
                      session.edit(unit.id, {
                        proofread_text: unit.translated_text,
                      });
                    }}
                  >
                    <Copy size={20} aria-hidden="true" />
                  </button>
                )}
              </div>
              <div className="size-7 shrink-0">
                {!readonly && selected && (
                  <button
                    type="button"
                    title={unit.is_proofread ? "取消校对确认" : "确认校对"}
                    aria-label={unit.is_proofread ? "取消校对确认" : "确认校对"}
                    className={`rounded p-1 transition-colors ${unit.is_proofread ? "text-red-400 hover:text-red-600" : "text-gray-400 hover:text-green-600"}`}
                    onPointerDown={(event) => {
                      event.preventDefault();
                    }}
                    onClick={() => {
                      session.edit(unit.id, {
                        is_proofread: !unit.is_proofread,
                      });
                    }}
                  >
                    {unit.is_proofread ? (
                      <X size={20} aria-hidden="true" />
                    ) : (
                      <Check size={20} aria-hidden="true" />
                    )}
                  </button>
                )}
              </div>
            </div>
          </>
        )}
        {selected && !readonly && (
          <>
            <div className="my-1 mr-10 h-px bg-gray-200" />
            <SpecialCharsBar
              characters={characters}
              controller={characterBar}
              floating={false}
              disabled={false}
              onInsert={insert}
            />
          </>
        )}
      </div>
    </div>
  );
}
