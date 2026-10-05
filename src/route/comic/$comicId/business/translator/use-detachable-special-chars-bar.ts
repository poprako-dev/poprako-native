import type { RefObject } from "react";
import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import type { PointerEvent as ReactPointerEvent } from "react";
import { isBeyondDragThreshold } from "./drag-threshold";

type Options = {
  enabled: boolean;
  interactionKey: string;
};

type Position = {
  x: number;
  y: number;
  width: number;
  maxHeight: number;
};

type DragSession = {
  pointerId: number;
  pointerType: string;
  startX: number;
  startY: number;
  origin: Position;
  previous: Position | null;
  height: number;
  isDragging: boolean;
  captureTarget: HTMLElement;
};

function constrainPosition(position: Position, height: number): Position {
  const viewport = window.visualViewport;
  const left = (viewport?.offsetLeft ?? 0) + 8;
  const top = (viewport?.offsetTop ?? 0) + 8;
  const availableWidth = Math.max(
    0,
    (viewport?.width ?? window.innerWidth) - 16,
  );
  const maxHeight = Math.max(0, (viewport?.height ?? window.innerHeight) - 16);
  const width = Math.min(position.width, availableWidth);
  return {
    x: Math.max(left, Math.min(position.x, left + availableWidth - width)),
    y: Math.max(
      top,
      Math.min(position.y, top + maxHeight - Math.min(height, maxHeight)),
    ),
    width,
    maxHeight,
  };
}

export function useDetachableSpecialCharsBar({
  enabled,
  interactionKey,
}: Options): {
  isEnabled: boolean;
  position: Position | null;
  isGripHeld: boolean;
  placeholderHeight: number | null;
  floatingRef: RefObject<HTMLDivElement | null>;
  handleGripPointerDown: (event: ReactPointerEvent<HTMLButtonElement>) => void;
  dock: () => void;
} {
  const [position, setPosition] = useState<Position | null>(null);
  const [isGripHeld, setIsGripHeld] = useState(false);
  const [placeholderHeight, setPlaceholderHeight] = useState<number | null>(
    null,
  );
  const floatingRef = useRef<HTMLDivElement>(null);
  const positionRef = useRef<Position | null>(null);
  const sessionRef = useRef<DragSession | null>(null);

  const updatePosition = useCallback((next: Position | null) => {
    positionRef.current = next;
    setPosition(next);
  }, []);

  const finishDrag = useCallback(
    (shouldCommit: boolean) => {
      const session = sessionRef.current;
      if (!session) return;
      sessionRef.current = null;
      if (!shouldCommit || !session.isDragging) {
        const height =
          floatingRef.current?.getBoundingClientRect().height ?? session.height;
        updatePosition(
          session.previous ? constrainPosition(session.previous, height) : null,
        );
      }
      setIsGripHeld(false);
      setPlaceholderHeight(null);
      if (session.captureTarget.hasPointerCapture(session.pointerId)) {
        session.captureTarget.releasePointerCapture(session.pointerId);
      }
    },
    [updatePosition],
  );

  function handleGripPointerDown(
    event: ReactPointerEvent<HTMLButtonElement>,
  ): void {
    if (!enabled || !event.isPrimary || event.button !== 0) return;
    const bar = event.currentTarget.closest<HTMLElement>(
      "[data-special-chars-bar]",
    );
    if (!bar) return;
    event.preventDefault();
    event.stopPropagation();
    finishDrag(false);
    const rect = bar.getBoundingClientRect();
    const session: DragSession = {
      pointerId: event.pointerId,
      pointerType: event.pointerType,
      startX: event.clientX,
      startY: event.clientY,
      origin: {
        x: rect.left,
        y: rect.top,
        width: rect.width,
        maxHeight: rect.height,
      },
      previous: positionRef.current,
      height: rect.height,
      isDragging: false,
      // The embedded grip unmounts when detached; capture must survive that move.
      captureTarget: document.body,
    };
    sessionRef.current = session;
    session.captureTarget.setPointerCapture(event.pointerId);
    setIsGripHeld(true);
  }

  function dock(): void {
    finishDrag(false);
    updatePosition(null);
  }

  useEffect(() => {
    if (!enabled) return;

    const move = (event: PointerEvent): void => {
      const session = sessionRef.current;
      if (session?.pointerId !== event.pointerId) return;
      const deltaX = event.clientX - session.startX;
      const deltaY = event.clientY - session.startY;
      const isBeyondThreshold = isBeyondDragThreshold(
        session.pointerType,
        deltaX,
        deltaY,
      );
      if (!isBeyondThreshold && !session.isDragging) return;
      event.preventDefault();
      if (!session.isDragging) {
        session.isDragging = true;
        if (session.previous === null) setPlaceholderHeight(session.height);
      }
      const height =
        floatingRef.current?.getBoundingClientRect().height ?? session.height;
      updatePosition(
        constrainPosition(
          {
            ...session.origin,
            x: session.origin.x + deltaX,
            y: session.origin.y + deltaY,
          },
          height,
        ),
      );
    };
    const release = (event: PointerEvent): void => {
      if (sessionRef.current?.pointerId === event.pointerId) finishDrag(true);
    };
    const cancelPointer = (event: PointerEvent): void => {
      if (sessionRef.current?.pointerId === event.pointerId) finishDrag(false);
    };
    const cancel = (): void => {
      finishDrag(false);
    };
    const escape = (event: KeyboardEvent): void => {
      if (event.key !== "Escape" || !sessionRef.current || event.isComposing) {
        return;
      }
      event.preventDefault();
      event.stopImmediatePropagation();
      finishDrag(false);
    };

    const listeners = new AbortController();
    addEventListener("pointermove", move, {
      passive: false,
      signal: listeners.signal,
    });
    addEventListener("pointerup", release, { signal: listeners.signal });
    addEventListener("pointercancel", cancelPointer, {
      signal: listeners.signal,
    });
    addEventListener("lostpointercapture", cancelPointer, {
      capture: true,
      signal: listeners.signal,
    });
    window.addEventListener("blur", cancel, { signal: listeners.signal });
    addEventListener("keydown", escape, {
      capture: true,
      signal: listeners.signal,
    });
    return () => {
      finishDrag(false);
      listeners.abort();
    };
  }, [enabled, interactionKey, finishDrag, updatePosition]);

  const isDetached = position !== null;
  useLayoutEffect(() => {
    const element = floatingRef.current;
    if (!isDetached || !enabled || !element) return;
    let constrainFrame: number | null = null;
    const constrainNow = (): void => {
      const current = positionRef.current;
      if (!current) return;
      const next = constrainPosition(
        current,
        element.getBoundingClientRect().height,
      );
      if (
        next.x === current.x &&
        next.y === current.y &&
        next.width === current.width &&
        next.maxHeight === current.maxHeight
      ) {
        return;
      }
      updatePosition(next);
    };
    const constrain = (): void => {
      if (constrainFrame !== null) return;
      constrainFrame = requestAnimationFrame(() => {
        constrainFrame = null;
        constrainNow();
      });
    };
    // Keep a newly mounted or resized toolbar inside the viewport before paint.
    constrainNow();
    const observer = new ResizeObserver(constrain);
    observer.observe(element);
    window.addEventListener("resize", constrain);
    const viewport = window.visualViewport;
    viewport?.addEventListener("resize", constrain);
    viewport?.addEventListener("scroll", constrain);
    return () => {
      observer.disconnect();
      if (constrainFrame !== null) cancelAnimationFrame(constrainFrame);
      window.removeEventListener("resize", constrain);
      viewport?.removeEventListener("resize", constrain);
      viewport?.removeEventListener("scroll", constrain);
    };
  }, [enabled, isDetached, updatePosition]);

  return {
    isEnabled: enabled,
    position,
    isGripHeld,
    placeholderHeight,
    floatingRef,
    handleGripPointerDown,
    dock,
  };
}

export type SpecialCharsBarController = Omit<
  ReturnType<typeof useDetachableSpecialCharsBar>,
  "floatingRef"
>;
