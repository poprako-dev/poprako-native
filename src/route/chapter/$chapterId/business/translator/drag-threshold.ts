const POINTER_DRAG_THRESHOLD = 4;
const TOUCH_DRAG_THRESHOLD = 8;

export function dragThreshold(pointerType: string): 8 | 4 {
  return pointerType === "touch"
    ? TOUCH_DRAG_THRESHOLD
    : POINTER_DRAG_THRESHOLD;
}

export function isBeyondDragThreshold(
  pointerType: string,
  deltaX: number,
  deltaY: number,
): boolean {
  return Math.hypot(deltaX, deltaY) > dragThreshold(pointerType);
}
