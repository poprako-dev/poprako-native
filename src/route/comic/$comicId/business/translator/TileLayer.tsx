import { useEffect, useState } from "react";
import type { ReactElement } from "react";
import { commands, unwrap } from "@/bridge";
import { imageUrl } from "@/bridge/image-url";

type Tile = { x: number; y: number; width: number; height: number };
type TileImage = Tile & { handle: string };
type TileLayerProps = {
  comicId: string;
  pageId: string;
  width: number;
  height: number;
  viewportWidth: number;
  viewportHeight: number;
  displayWidth: number;
  scale: number;
  offsetX: number;
  offsetY: number;
};

export function visibleTiles(
  props: Omit<TileLayerProps, "comicId" | "pageId">,
): Tile[] {
  const {
    width,
    height,
    viewportWidth,
    viewportHeight,
    scale,
    offsetX,
    offsetY,
  } = props;
  const fit = props.displayWidth / width;
  const pixelScale = fit * scale;
  if (pixelScale <= 0) return [];
  const left = viewportWidth / 2 + offsetX - (width * pixelScale) / 2;
  const top = viewportHeight / 2 + offsetY - (height * pixelScale) / 2;
  const fromX = Math.max(0, Math.floor(-left / pixelScale / 1024) * 1024);
  const fromY = Math.max(0, Math.floor(-top / pixelScale / 1024) * 1024);
  const endX = Math.min(width, (viewportWidth - left) / pixelScale);
  const endY = Math.min(height, (viewportHeight - top) / pixelScale);
  const tiles: Tile[] = [];
  for (let y = fromY; y < endY; y += 1024) {
    for (let x = fromX; x < endX; x += 1024) {
      tiles.push({
        x,
        y,
        width: Math.min(1024, width - x),
        height: Math.min(1024, height - y),
      });
    }
  }
  const centerX = (viewportWidth / 2 - left) / pixelScale;
  const centerY = (viewportHeight / 2 - top) / pixelScale;
  // The view owns at most sixteen RGBA tiles (64 MiB); preview fills the rest.
  return tiles
    .sort(
      (a, b) =>
        Math.hypot(a.x + a.width / 2 - centerX, a.y + a.height / 2 - centerY) -
        Math.hypot(b.x + b.width / 2 - centerX, b.y + b.height / 2 - centerY),
    )
    .slice(0, 16);
}

export function TileLayer(props: TileLayerProps): ReactElement {
  const key = JSON.stringify(visibleTiles(props));
  const [result, setResult] = useState<{
    key: string;
    tiles: TileImage[];
    failed: boolean;
  }>({ key: "", tiles: [], failed: false });
  const { comicId, pageId, width, height } = props;
  useEffect(() => {
    let active = true;
    function isActive(): boolean {
      return active;
    }
    const handles: string[] = [];
    async function release(value: string[]): Promise<void> {
      if (value.length === 0) return;
      try {
        unwrap(await commands.releaseImageResources(value));
      } catch {
        console.warn("高清图块释放未完成，将在退出应用时回收。");
      }
    }
    async function load(): Promise<void> {
      const regions: unknown = JSON.parse(key);
      if (!Array.isArray(regions)) return;
      const images: TileImage[] = [];
      try {
        for (const region of regions) {
          if (!isActive()) return;
          if (!isTile(region)) continue;
          const resource = unwrap(
            await commands.getImageTile(
              comicId,
              pageId,
              region.x,
              region.y,
              region.width,
              region.height,
            ),
          );
          if (!isActive()) {
            await release([resource.handle]);
            return;
          }
          handles.push(resource.handle);
          images.push({ ...region, handle: resource.handle });
        }
        if (active) setResult({ key, tiles: images, failed: false });
      } catch {
        if (active) setResult({ key, tiles: [], failed: true });
      }
    }
    void load();
    return () => {
      active = false;
      void release(handles);
    };
  }, [comicId, pageId, key]);
  return (
    <>
      {result.key === key &&
        result.tiles.map((tile) => (
          <img
            key={tile.handle}
            src={imageUrl(tile.handle)}
            alt=""
            draggable={false}
            onError={() => {
              setResult({ key, tiles: [], failed: true });
            }}
            className="pointer-events-none absolute"
            style={{
              left: `${String((tile.x / width) * 100)}%`,
              top: `${String((tile.y / height) * 100)}%`,
              width: `${String((tile.width / width) * 100)}%`,
              height: `${String((tile.height / height) * 100)}%`,
            }}
          />
        ))}
      {result.key === key && result.failed && (
        <span
          role="status"
          className="absolute bottom-0 left-0 bg-background p-1 text-xs"
        >
          高清图块暂不可用，保留预览图片。
        </span>
      )}
    </>
  );
}

function isTile(value: unknown): value is Tile {
  return (
    typeof value === "object" &&
    value !== null &&
    "x" in value &&
    typeof value.x === "number" &&
    "y" in value &&
    typeof value.y === "number" &&
    "width" in value &&
    typeof value.width === "number" &&
    "height" in value &&
    typeof value.height === "number"
  );
}
