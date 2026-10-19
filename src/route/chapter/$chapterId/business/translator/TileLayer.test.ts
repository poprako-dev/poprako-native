import { describe, expect, it } from "vitest";
import { visibleTiles } from "./TileLayer";

describe("visible image tile budget", () => {
  it("limits display tile residency and clips edges of irregular images", () => {
    const tiles = visibleTiles({
      width: 6000,
      height: 5001,
      viewportWidth: 1000,
      viewportHeight: 800,
      displayWidth: 835,
      scale: 1,
      offsetX: 0,
      offsetY: 0,
    });
    expect(tiles.length).toBeLessThanOrEqual(16);
    expect(
      tiles.reduce((bytes, tile) => bytes + tile.width * tile.height * 4, 0),
    ).toBeLessThanOrEqual(64 * 1024 * 1024);
    for (const tile of tiles) {
      expect(tile.x + tile.width).toBeLessThanOrEqual(6000);
      expect(tile.y + tile.height).toBeLessThanOrEqual(5001);
    }
  });

  it("does not request tiles when the image is outside the viewport", () => {
    expect(
      visibleTiles({
        width: 1000,
        height: 2000,
        viewportWidth: 1000,
        viewportHeight: 800,
        displayWidth: 348,
        scale: 1,
        offsetX: 5000,
        offsetY: 0,
      }),
    ).toEqual([]);
  });
});
