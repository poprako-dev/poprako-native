import { useMemo, useRef } from "react";
import type { ReactElement } from "react";
import { buildTextDiff } from "./text-diff";
import { LineBreakOverlay } from "./LineBreakOverlay";

type TextDifferenceProps = {
  translated: string;
  proofread: string;
  selected: boolean;
  onSelect: () => void;
};

export function TextDifference({
  translated,
  proofread,
  selected,
  onSelect,
}: TextDifferenceProps): ReactElement {
  const ref = useRef<HTMLDivElement>(null);
  const parts = useMemo(
    () => buildTextDiff(translated, proofread),
    [translated, proofread],
  );
  return (
    <div className="relative min-w-0 flex-1">
      <div
        ref={ref}
        role="textbox"
        aria-label="翻译与校对差异"
        aria-readonly="true"
        tabIndex={0}
        onFocus={onSelect}
        className={`min-h-[1.2em] pr-4 whitespace-pre-wrap text-base leading-relaxed break-words outline-none ${selected ? "font-medium text-stone-900" : "text-stone-700"}`}
      >
        {parts.length === 0 && (
          <span className="text-muted-foreground">无翻译内容</span>
        )}
        {parts.map((part, index) => {
          const key = `${String(index)}-${part.kind}`;
          if (part.kind === "deleted" || part.kind === "replacement-removed")
            return (
              <del
                key={key}
                title={part.kind === "deleted" ? "初翻删除" : "初翻被替换"}
                className={`rounded-[2px] line-through decoration-1 ${part.kind === "deleted" ? "bg-(--diff-delete-bg) font-light text-(--diff-delete-text) decoration-(--diff-delete-text)" : "bg-(--diff-replaced-bg) font-normal text-(--diff-replaced-text) decoration-(--diff-replaced-text)"}`}
              >
                {part.text}
              </del>
            );
          if (part.kind === "inserted" || part.kind === "replacement-added")
            return (
              <ins
                key={key}
                title={part.kind === "inserted" ? "校对新增" : "校对替换"}
                className={`rounded-[2px] font-medium no-underline ${part.kind === "inserted" ? "bg-(--diff-insert-bg) text-(--diff-insert-text)" : "bg-(--diff-replacement-bg) text-(--diff-replacement-text)"}`}
              >
                {part.text}
              </ins>
            );
          return <span key={key}>{part.text}</span>;
        })}
      </div>
      {parts.some((part) => part.text.includes("\n")) && (
        <LineBreakOverlay targetRef={ref} layoutKey={[parts, selected]} />
      )}
    </div>
  );
}
