import { useEffect, useRef } from "react";
import type { KeyboardEventHandler, PointerEventHandler } from "react";

type PressHandlers = {
  onPointerDown: PointerEventHandler<HTMLElement>;
  onPointerMove: PointerEventHandler<HTMLElement>;
  onPointerUp: PointerEventHandler<HTMLElement>;
  onPointerCancel: PointerEventHandler<HTMLElement>;
  onPointerLeave: PointerEventHandler<HTMLElement>;
  onKeyDown: KeyboardEventHandler<HTMLElement>;
};

export function useTermPress(
  enabled: boolean,
  onEdit: () => void,
): { handlers: PressHandlers; activate: () => boolean } {
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const origin = useRef({ x: 0, y: 0 });
  const consumed = useRef(false);
  function cancel(): void {
    if (timer.current !== null) clearTimeout(timer.current);
    timer.current = null;
  }
  useEffect(() => cancel, []);
  return {
    activate: () => {
      const allowed = !consumed.current;
      consumed.current = false;
      return allowed;
    },
    handlers: {
      onPointerDown: (event) => {
        cancel();
        consumed.current = false;
        if (!enabled || event.button !== 0) return;
        origin.current = { x: event.clientX, y: event.clientY };
        timer.current = setTimeout(() => {
          consumed.current = true;
          onEdit();
        }, 500);
      },
      onPointerMove: (event) => {
        if (timer.current === null) return;
        if (
          Math.hypot(
            event.clientX - origin.current.x,
            event.clientY - origin.current.y,
          ) > 6
        ) {
          consumed.current = true;
          cancel();
        }
      },
      onPointerUp: cancel,
      onPointerCancel: cancel,
      onPointerLeave: cancel,
      onKeyDown: (event) => {
        if (enabled && event.key === "F2") {
          event.preventDefault();
          event.stopPropagation();
          onEdit();
        }
      },
    },
  };
}
