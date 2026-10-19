import type { ReactElement } from "react";

export const MARKER_SIZE = 32;
export const MARKER_DOT = 8;
export const PIN_OFFSET = MARKER_SIZE + MARKER_DOT - 2;
type MarkerProps = {
  index: number;
  bubble: boolean;
  completed: boolean;
  selected: boolean;
  dragging: boolean;
  dimmed: boolean;
};

export function Marker({
  index,
  bubble,
  completed,
  selected,
  dragging,
  dimmed,
}: MarkerProps): ReactElement {
  const background = bubble
    ? dimmed
      ? "var(--marker-bubble-dimmed)"
      : "var(--marker-bubble)"
    : dimmed
      ? "var(--marker-note-dimmed)"
      : "var(--marker-note)";
  const border = selected
    ? "var(--marker-selected)"
    : completed
      ? "var(--brand-leaf)"
      : bubble
        ? dimmed
          ? "var(--marker-bubble-border-dimmed)"
          : "var(--marker-bubble-border)"
        : dimmed
          ? "var(--marker-note-border-dimmed)"
          : "var(--marker-note-border)";
  return (
    <span
      className="flex flex-col items-center select-none"
      style={{
        width: MARKER_SIZE,
        opacity: dragging ? 0.8 : undefined,
        transform: dragging ? "scale(1.1)" : undefined,
        transition: dragging
          ? "none"
          : "transform 0.15s ease-out, opacity 0.15s ease-out",
      }}
    >
      <span
        className={`relative flex items-center justify-center rounded-full border-2 shadow-lg ${selected ? "ring-4 ring-blue-500/10" : ""}`}
        style={{
          width: MARKER_SIZE,
          height: MARKER_SIZE,
          background,
          borderColor: border,
          transition:
            "background-color 0.2s, border-color 0.2s, box-shadow 0.2s",
        }}
      >
        <span className="text-[13px] leading-none font-black text-white tabular-nums">
          {index + 1}
        </span>
      </span>
      <span
        className="-mt-px rounded-full border-2 border-black/20 shadow-sm"
        style={{ width: MARKER_DOT, height: MARKER_DOT, background }}
      />
    </span>
  );
}
