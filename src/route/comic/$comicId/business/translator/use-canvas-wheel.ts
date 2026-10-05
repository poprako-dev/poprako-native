import { useEffect } from "react";
import type { Dispatch, RefObject, SetStateAction } from "react";

export type CanvasTransform = { scale: number; x: number; y: number };

export function useCanvasWheel(
  container: RefObject<HTMLDivElement | null>,
  image: RefObject<HTMLImageElement | null>,
  setTransform: Dispatch<SetStateAction<CanvasTransform>>,
): void {
  useEffect(() => {
    const node = container.current;
    if (!node) return;
    function wheel(event: WheelEvent): void {
      const rectangle = image.current?.getBoundingClientRect();
      if (
        !rectangle ||
        event.clientX < rectangle.left ||
        event.clientX > rectangle.right ||
        event.clientY < rectangle.top ||
        event.clientY > rectangle.bottom
      )
        return;
      event.preventDefault();
      const bounds = node?.getBoundingClientRect();
      if (!bounds) return;
      setTransform((current) => {
        const scale = Math.max(
          0.5,
          Math.min(5, current.scale * (event.deltaY < 0 ? 1.08 : 0.92)),
        );
        const factor = scale / current.scale - 1;
        return {
          scale,
          x:
            current.x -
            (event.clientX - bounds.left - bounds.width / 2 - current.x) *
              factor,
          y:
            current.y -
            (event.clientY - bounds.top - bounds.height / 2 - current.y) *
              factor,
        };
      });
    }
    node.addEventListener("wheel", wheel, { passive: false });
    return () => {
      node.removeEventListener("wheel", wheel);
    };
  }, [container, image, setTransform]);
}
