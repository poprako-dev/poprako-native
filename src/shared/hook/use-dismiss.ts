import { useEffect } from "react";
import type { RefObject } from "react";

export function useDismiss(
  open: boolean,
  container: RefObject<HTMLElement | null>,
  close: () => void,
): void {
  useEffect(() => {
    if (!open) return;
    function pointer(event: PointerEvent): void {
      if (
        event.target instanceof Node &&
        !container.current?.contains(event.target)
      )
        close();
    }
    function key(event: KeyboardEvent): void {
      if (event.key === "Escape" && !event.isComposing) {
        event.preventDefault();
        close();
        container.current
          ?.querySelector<HTMLButtonElement>("button[aria-expanded]")
          ?.focus();
      }
    }
    document.addEventListener("pointerdown", pointer);
    document.addEventListener("keydown", key);
    return () => {
      document.removeEventListener("pointerdown", pointer);
      document.removeEventListener("keydown", key);
    };
  }, [open, container, close]);
}
