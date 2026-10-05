import { useEffect, useRef, useState } from "react";
import type { PointerEvent, ReactElement } from "react";
import type { UnitDraft } from "@/bridge/generated/bindings";
import type { EditorSession, EditorState } from "./editor-session";
import { useCanvasWheel } from "./use-canvas-wheel";
import type { CanvasTransform } from "./use-canvas-wheel";
import { TileLayer } from "./TileLayer";
import { CanvasMarkerLayer } from "./CanvasMarkerLayer";

type CanvasProps = {
  state: EditorState;
  session: EditorSession;
  imageUrl: string;
  imageError: string;
  opacity: number;
  relocation: boolean;
  preview: boolean;
  highQuality: boolean;
  creationEnabled: boolean;
  onDelete: (id: string) => void;
};

type Gesture = {
  id: string;
  pointer: number;
  x: number;
  y: number;
  origin: CanvasTransform;
  unit: UnitDraft | null;
  moved: boolean;
};

export function Canvas({
  state,
  session,
  imageUrl,
  imageError,
  opacity,
  relocation,
  preview,
  highQuality,
  creationEnabled,
  onDelete,
}: CanvasProps): ReactElement {
  const container = useRef<HTMLDivElement>(null);
  const image = useRef<HTMLImageElement>(null);
  const gesture = useRef<Gesture | null>(null);
  const location = useRef({ selected: "", revision: 0, enabled: false });
  const [size, setSize] = useState({ width: 0, height: 0 });
  const [transform, setTransform] = useState<CanvasTransform>({
    scale: 1,
    x: 0,
    y: 0,
  });
  const [hovered, setHovered] = useState("");
  const [failed, setFailed] = useState(false);
  const [naturalWidth, setNaturalWidth] = useState(0);
  const [panning, setPanning] = useState(false);
  const [relocating, setRelocating] = useState(false);
  const relocationTimer = useRef<ReturnType<typeof setTimeout> | undefined>(
    undefined,
  );
  const [moving, setMoving] = useState<UnitDraft | null>(null);
  const movingRef = useRef<UnitDraft | null>(null);
  const editable =
    state.mode !== "readonly" && !state.locked && !state.uncertain;
  const focused = state.draft.find((unit) => unit.id === state.selected);
  const shown = state.draft.find(
    (unit) =>
      unit.id ===
      (hovered ||
        (preview && state.mode === "proofreading" ? state.selected : "")),
  );

  useEffect(() => {
    const node = container.current;
    if (!node) return;
    const observer = new ResizeObserver(([entry]) => {
      if (entry)
        setSize({
          width: entry.contentRect.width,
          height: entry.contentRect.height,
        });
    });
    observer.observe(node);
    return () => {
      observer.disconnect();
    };
  }, []);

  useCanvasWheel(container, image, setTransform);

  useEffect(() => {
    if (!focused || !image.current) return;
    const previous = location.current;
    location.current = {
      selected: focused.id,
      revision: state.locateRevision,
      enabled: relocation,
    };
    const requested = previous.revision !== state.locateRevision;
    if (
      !requested &&
      (!relocation || (previous.selected === focused.id && previous.enabled))
    )
      return;
    const width = image.current.offsetWidth;
    const height = image.current.offsetHeight;
    clearTimeout(relocationTimer.current);
    setRelocating(true);
    setTransform((current) => ({
      ...current,
      x: -(focused.x_coord - 0.5) * width * current.scale,
      y: -(focused.y_coord - 0.5) * height * current.scale,
    }));
    relocationTimer.current = setTimeout(() => {
      setRelocating(false);
    }, 200);
  }, [focused, relocation, state.locateRevision]);

  function add(clientX: number, clientY: number, bubble: boolean): void {
    const rectangle = image.current?.getBoundingClientRect();
    if (!editable || !creationEnabled || !rectangle || failed) return;
    const x = (clientX - rectangle.left) / rectangle.width;
    const y = (clientY - rectangle.top) / rectangle.height;
    if (x < 0 || x > 1 || y < 0 || y > 1) return;
    const id = crypto.randomUUID();
    session.replace([
      ...state.draft,
      {
        id,
        x_coord: x,
        y_coord: y,
        is_bubble: bubble,
        is_flagged: false,
        translated_text: "",
        proofread_text: "",
        is_proofread: false,
      },
    ]);
    session.select(id);
  }

  function start(
    event: PointerEvent<HTMLElement>,
    unit: UnitDraft | null,
  ): void {
    if (event.button !== 0 || !image.current) return;
    event.stopPropagation();
    if (unit && !editable) {
      session.select(unit.id);
      return;
    }
    if (!unit) {
      const rect = image.current.getBoundingClientRect();
      if (
        event.clientX < rect.left ||
        event.clientX > rect.right ||
        event.clientY < rect.top ||
        event.clientY > rect.bottom
      )
        return;
    }
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    gesture.current = {
      id: unit?.id ?? "",
      pointer: event.pointerId,
      x: event.clientX,
      y: event.clientY,
      origin: transform,
      unit,
      moved: false,
    };
    session.suspend(true, "gesture");
  }

  function move(event: PointerEvent<HTMLElement>): void {
    const active = gesture.current;
    if (event.pointerId !== active?.pointer) return;
    const dx = event.clientX - active.x;
    const dy = event.clientY - active.y;
    if (!active.moved && Math.hypot(dx, dy) <= (active.unit ? 3 : 8)) return;
    active.moved = true;
    if (!active.unit) {
      setPanning(true);
      setTransform({
        ...active.origin,
        x: active.origin.x + dx,
        y: active.origin.y + dy,
      });
      return;
    }
    const rectangle = image.current?.getBoundingClientRect();
    if (!rectangle) return;
    const next = {
      ...active.unit,
      x_coord: Math.max(
        0,
        Math.min(1, active.unit.x_coord + dx / rectangle.width),
      ),
      y_coord: Math.max(
        0,
        Math.min(1, active.unit.y_coord + dy / rectangle.height),
      ),
    };
    movingRef.current = next;
    setMoving(next);
  }

  function end(event: PointerEvent<HTMLElement>): void {
    const active = gesture.current;
    if (active?.pointer !== event.pointerId) return;
    gesture.current = null;
    setPanning(false);
    const position = movingRef.current;
    if (active.moved && active.unit && position)
      session.edit(active.id, {
        x_coord: position.x_coord,
        y_coord: position.y_coord,
      });
    if (!active.moved && active.unit) session.select(active.id);
    if (!active.moved && !active.unit) add(event.clientX, event.clientY, true);
    setMoving(null);
    movingRef.current = null;
    session.suspend(false, "gesture");
  }

  function cancel(): void {
    const active = gesture.current;
    gesture.current = null;
    if (active) setTransform(active.origin);
    setPanning(false);
    setMoving(null);
    movingRef.current = null;
    session.suspend(false, "gesture");
  }

  useEffect(
    () => () => {
      session.suspend(false, "gesture");
      clearTimeout(relocationTimer.current);
    },
    [session],
  );

  return (
    <div
      ref={container}
      role="application"
      aria-label="漫画画布，滚轮缩放，拖动画面平移，单击图片添加框内标记，右键添加框外标记"
      tabIndex={0}
      className={`relative h-full w-full touch-none overflow-hidden bg-canvas select-none focus-visible:outline-2 focus-visible:outline-ring ${panning ? "cursor-grabbing" : "cursor-default"}`}
      onKeyDown={(event) => {
        if (event.key === "Escape" && !event.nativeEvent.isComposing) {
          cancel();
          return;
        }
        if (
          event.target !== event.currentTarget ||
          event.nativeEvent.isComposing ||
          event.altKey ||
          event.metaKey ||
          event.ctrlKey
        )
          return;
        const direction = {
          ArrowLeft: [24, 0],
          ArrowRight: [-24, 0],
          ArrowUp: [0, 24],
          ArrowDown: [0, -24],
        }[event.key];
        if (direction) {
          event.preventDefault();
          setTransform((current) => ({
            ...current,
            x: current.x + (direction[0] ?? 0),
            y: current.y + (direction[1] ?? 0),
          }));
        }
      }}
      onPointerDown={(event) => {
        start(event, null);
      }}
      onPointerMove={move}
      onPointerUp={end}
      onPointerCancel={(event) => {
        if (gesture.current?.pointer === event.pointerId) cancel();
      }}
      onLostPointerCapture={(event) => {
        if (gesture.current?.pointer === event.pointerId) cancel();
      }}
      onContextMenu={(event) => {
        event.preventDefault();
        add(event.clientX, event.clientY, false);
      }}
    >
      {!imageUrl || failed ? (
        <div
          role="status"
          className="flex h-full items-center justify-center p-8 text-center text-white"
        >
          {imageError ||
            (failed
              ? "图片无法显示，请重新打开页面或检查图片资源。"
              : "正在加载图片…")}
        </div>
      ) : (
        <div
          className={`pointer-events-none absolute inset-0 flex items-center justify-center ${relocating ? "transition-transform duration-200 ease-out motion-reduce:transition-none" : ""}`}
          style={{
            transform: `translate(${String(transform.x)}px, ${String(transform.y)}px)`,
          }}
        >
          <div
            className="pointer-events-auto relative"
            style={{ transform: `scale(${String(transform.scale)})` }}
          >
            <img
              ref={image}
              src={imageUrl}
              alt={`第 ${String(state.baseline.page.index + 1)} 页漫画`}
              draggable={false}
              onLoad={(event) => {
                setNaturalWidth(event.currentTarget.naturalWidth);
              }}
              onError={() => {
                setFailed(true);
              }}
              className="block shadow-md"
              style={{
                maxWidth: size.width * 0.9,
                maxHeight: size.height * 0.95,
                width: "auto",
                height: "auto",
              }}
            />
            {highQuality && naturalWidth > 0 && (
              <TileLayer
                comicId={state.baseline.page.comic_id}
                pageId={state.baseline.page.id}
                width={state.baseline.page.image.width}
                height={state.baseline.page.image.height}
                viewportWidth={size.width}
                viewportHeight={size.height}
                displayWidth={Math.min(
                  size.width * 0.9,
                  (size.height * 0.95 * state.baseline.page.image.width) /
                    state.baseline.page.image.height,
                  naturalWidth,
                )}
                scale={transform.scale}
                offsetX={transform.x}
                offsetY={transform.y}
              />
            )}
            <CanvasMarkerLayer
              state={state}
              session={session}
              moving={moving}
              shown={shown ?? null}
              scale={transform.scale}
              opacity={opacity}
              editable={editable}
              dimmed={!preview}
              onStart={start}
              onMove={move}
              onEnd={end}
              onDelete={onDelete}
              onHover={setHovered}
            />
          </div>
        </div>
      )}
    </div>
  );
}
