import {
  type PointerEvent as ReactPointerEvent,
  type RefObject,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import type { UnitDraft } from "@/bridge/generated/bindings";
import { isBeyondDragThreshold } from "./drag-threshold";

const AUTO_SCROLL_EDGE = 32;
const MAX_AUTO_SCROLL_SPEED = 12;

type Args = {
  units: UnitDraft[];
  listRef: RefObject<HTMLDivElement | null>;
  enabled: boolean;
  onActivateUnit: (unitId: string) => void;
  onGesture: (active: boolean) => void;
  onReorderUnit: (unitId: string, targetIndex: number) => void;
};

type DragSession = {
  pointerId: number;
  unitId: string;
  startX: number;
  startY: number;
  lastClientY: number;
  sourceIndex: number;
  didDrag: boolean;
  previewOrder: string[];
  captureTarget: HTMLDivElement;
};

function sameOrder(lhs: string[], rhs: string[]): boolean {
  return (
    lhs.length === rhs.length && lhs.every((id, index) => id === rhs[index])
  );
}

export function useUnitReorder({
  units,
  listRef,
  enabled,
  onActivateUnit,
  onReorderUnit,
  onGesture,
}: Args): {
  orderedUnits: UnitDraft[];
  draggingUnitId: string | null;
  handleIndexPointerDown: (
    event: ReactPointerEvent<HTMLButtonElement>,
    targetUnitId: string,
  ) => void;
} {
  const [previewOrder, setPreviewOrder] = useState<string[] | null>(null);
  const [draggingUnitId, setDraggingUnitId] = useState<string | null>(null);

  const unitsRef = useRef(units);
  const enabledRef = useRef(enabled);
  const onActivateUnitRef = useRef(onActivateUnit);
  const onReorderUnitRef = useRef(onReorderUnit);
  const dragRef = useRef<DragSession | null>(null);
  const onGestureRef = useRef(onGesture);
  onGestureRef.current = onGesture;
  const autoScrollFrameRef = useRef<number | null>(null);

  unitsRef.current = units;
  enabledRef.current = enabled;
  onActivateUnitRef.current = onActivateUnit;
  onReorderUnitRef.current = onReorderUnit;

  const stopAutoScroll = useCallback(() => {
    if (autoScrollFrameRef.current === null) return;
    cancelAnimationFrame(autoScrollFrameRef.current);
    autoScrollFrameRef.current = null;
  }, []);

  const finishDrag = useCallback(
    (commit: boolean, activate: boolean) => {
      const session = dragRef.current;
      if (!session) return;

      dragRef.current = null;
      onGestureRef.current(false);
      stopAutoScroll();
      setPreviewOrder(null);
      setDraggingUnitId(null);

      if (session.captureTarget.hasPointerCapture(session.pointerId)) {
        session.captureTarget.releasePointerCapture(session.pointerId);
      }

      if (activate) {
        onActivateUnitRef.current(session.unitId);
        return;
      }
      if (!commit || !session.didDrag) return;

      const targetIndex = session.previewOrder.indexOf(session.unitId);
      if (targetIndex !== session.sourceIndex) {
        onReorderUnitRef.current(session.unitId, targetIndex);
      }
    },
    [stopAutoScroll],
  );

  const cancelDrag = useCallback(() => {
    finishDrag(false, false);
  }, [finishDrag]);

  const updatePreviewOrder = useCallback(
    (clientY: number) => {
      const session = dragRef.current;
      const list = listRef.current;
      if (!session?.didDrag || !list) return;

      const rows = [...list.querySelectorAll<HTMLElement>("[data-unit-id]")];
      const rowsById = new Map(rows.map((row) => [row.dataset["unitId"], row]));
      const remainingIds = session.previewOrder.filter(
        (id) => id !== session.unitId,
      );
      let targetIndex = 0;

      for (const id of remainingIds) {
        const row = rowsById.get(id);
        if (!row) continue;
        const rect = row.getBoundingClientRect();
        if (clientY < rect.top + rect.height / 2) break;
        targetIndex += 1;
      }

      const nextOrder = [...remainingIds];
      nextOrder.splice(targetIndex, 0, session.unitId);
      if (sameOrder(nextOrder, session.previewOrder)) return;

      session.previewOrder = nextOrder;
      setPreviewOrder(nextOrder);
    },
    [listRef],
  );

  const runAutoScroll = useCallback(
    function scrollFrame() {
      autoScrollFrameRef.current = null;
      const session = dragRef.current;
      const list = listRef.current;
      if (!session?.didDrag || !list) return;

      const rect = list.getBoundingClientRect();
      const topStrength = Math.min(
        1,
        Math.max(
          0,
          (rect.top + AUTO_SCROLL_EDGE - session.lastClientY) /
            AUTO_SCROLL_EDGE,
        ),
      );
      const bottomStrength = Math.min(
        1,
        Math.max(
          0,
          (session.lastClientY - rect.bottom + AUTO_SCROLL_EDGE) /
            AUTO_SCROLL_EDGE,
        ),
      );
      const speed = (bottomStrength - topStrength) * MAX_AUTO_SCROLL_SPEED;
      if (speed === 0) return;

      const previousScrollTop = list.scrollTop;
      list.scrollTop += speed;
      if (list.scrollTop === previousScrollTop) return;

      updatePreviewOrder(session.lastClientY);
      autoScrollFrameRef.current = requestAnimationFrame(scrollFrame);
    },
    [listRef, updatePreviewOrder],
  );

  const scheduleAutoScroll = useCallback(() => {
    if (autoScrollFrameRef.current !== null) return;
    autoScrollFrameRef.current = requestAnimationFrame(runAutoScroll);
  }, [runAutoScroll]);

  const handlePointerMove = useCallback(
    (event: PointerEvent) => {
      const session = dragRef.current;
      if (event.pointerId !== session?.pointerId) return;

      session.lastClientY = event.clientY;
      if (!session.didDrag) {
        const deltaX = event.clientX - session.startX;
        const deltaY = event.clientY - session.startY;
        if (!isBeyondDragThreshold(event.pointerType, deltaX, deltaY)) return;

        session.didDrag = true;
        setDraggingUnitId(session.unitId);
        setPreviewOrder(session.previewOrder);
      }

      event.preventDefault();
      updatePreviewOrder(event.clientY);
      scheduleAutoScroll();
    },
    [scheduleAutoScroll, updatePreviewOrder],
  );

  const handlePointerUp = useCallback(
    (event: PointerEvent) => {
      const session = dragRef.current;
      if (event.pointerId !== session?.pointerId) return;
      finishDrag(session.didDrag, !session.didDrag);
    },
    [finishDrag],
  );

  const handlePointerCancel = useCallback(
    (event: PointerEvent) => {
      if (dragRef.current?.pointerId === event.pointerId) cancelDrag();
    },
    [cancelDrag],
  );

  const handleLostPointerCapture = useCallback(
    (event: PointerEvent) => {
      if (dragRef.current?.pointerId === event.pointerId) cancelDrag();
    },
    [cancelDrag],
  );

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (event.isComposing) return;
      if (event.key !== "Escape" || !dragRef.current) return;
      event.preventDefault();
      cancelDrag();
    },
    [cancelDrag],
  );

  useEffect(() => {
    globalThis.addEventListener("pointermove", handlePointerMove, {
      passive: false,
    });
    globalThis.addEventListener("pointerup", handlePointerUp);
    globalThis.addEventListener("pointercancel", handlePointerCancel);
    globalThis.addEventListener(
      "lostpointercapture",
      handleLostPointerCapture,
      { capture: true },
    );
    globalThis.addEventListener("keydown", handleKeyDown);

    return () => {
      globalThis.removeEventListener("pointermove", handlePointerMove);
      globalThis.removeEventListener("pointerup", handlePointerUp);
      globalThis.removeEventListener("pointercancel", handlePointerCancel);
      globalThis.removeEventListener(
        "lostpointercapture",
        handleLostPointerCapture,
        true,
      );
      globalThis.removeEventListener("keydown", handleKeyDown);
    };
  }, [
    handleKeyDown,
    handleLostPointerCapture,
    handlePointerCancel,
    handlePointerMove,
    handlePointerUp,
  ]);

  const externalOrder = units.map((unit) => unit.id).join("\u{0}");
  const previousExternalOrderRef = useRef(externalOrder);

  useEffect(() => {
    const isOrderChanged = previousExternalOrderRef.current !== externalOrder;
    previousExternalOrderRef.current = externalOrder;
    if (!isOrderChanged || !dragRef.current) return;

    cancelDrag();
  }, [cancelDrag, externalOrder]);

  useEffect(
    () => () => {
      stopAutoScroll();
      const session = dragRef.current;
      dragRef.current = null;
      onGestureRef.current(false);
      if (session?.captureTarget.hasPointerCapture(session.pointerId)) {
        session.captureTarget.releasePointerCapture(session.pointerId);
      }
    },
    [stopAutoScroll],
  );

  const handleIndexPointerDown = useCallback(
    (event: ReactPointerEvent<HTMLButtonElement>, targetUnitId: string) => {
      if (!enabledRef.current || !event.isPrimary) return;
      if (event.pointerType === "mouse" && event.button !== 0) return;

      const order = unitsRef.current.map((unit) => unit.id);
      const sourceIndex = order.indexOf(targetUnitId);
      const list = listRef.current;
      if (sourceIndex === -1 || !list) return;

      if (dragRef.current) cancelDrag();
      event.stopPropagation();
      event.preventDefault();
      onGestureRef.current(true);
      list.setPointerCapture(event.pointerId);
      dragRef.current = {
        pointerId: event.pointerId,
        unitId: targetUnitId,
        startX: event.clientX,
        startY: event.clientY,
        lastClientY: event.clientY,
        sourceIndex,
        didDrag: false,
        previewOrder: order,
        captureTarget: list,
      };
    },
    [cancelDrag, listRef],
  );

  const orderedUnits = useMemo(() => {
    if (!previewOrder) return units;

    const unitsById = new Map(units.map((unit) => [unit.id, unit]));
    const reorderedUnits = previewOrder.flatMap((id) => {
      const unit = unitsById.get(id);
      return unit ? [unit] : [];
    });
    return reorderedUnits;
  }, [previewOrder, units]);

  return {
    orderedUnits,
    draggingUnitId,
    handleIndexPointerDown,
  };
}
