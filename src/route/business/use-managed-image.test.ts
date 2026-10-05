import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ImageKind, ImageResource } from "@/bridge/generated/bindings";
import { useManagedImage } from "@/route/business/use-managed-image";

type ImageReply = { status: "ok"; data: ImageResource };
const mock = vi.hoisted(() => ({
  get: vi.fn<
    (comic: string, page: string, kind: ImageKind) => Promise<ImageReply>
  >(),
  release:
    vi.fn<(handles: string[]) => Promise<{ status: "ok"; data: null }>>(),
}));
vi.mock("@/bridge", () => ({
  commands: { getImageResource: mock.get, releaseImageResources: mock.release },
  unwrap: (reply: ImageReply | { status: "ok"; data: null }) => reply.data,
  NativeCommandError: class extends Error {},
}));
vi.mock("@/bridge/image-url", () => ({
  imageUrl: (handle: string) => `resource://${handle}`,
}));

afterEach(cleanup);
beforeEach(() => {
  mock.get.mockReset();
  mock.release.mockReset().mockResolvedValue({ status: "ok", data: null });
});

describe("managed image ownership", () => {
  it("only loads visible images and releases the handle when hidden", async () => {
    mock.get.mockResolvedValue({
      status: "ok",
      data: { handle: "visible", width: 20, height: 30 },
    });
    const { result, rerender } = renderHook(
      ({ visible }) =>
        useManagedImage("comic", "page", "image", "thumbnail", visible),
      { initialProps: { visible: false } },
    );
    expect(mock.get).not.toHaveBeenCalled();
    rerender({ visible: true });
    await waitFor(() => {
      expect(result.current.url).toBe("resource://visible");
    });
    rerender({ visible: false });
    await waitFor(() => {
      expect(mock.release).toHaveBeenCalledWith(["visible"]);
    });
    expect(result.current.url).toBe("");
  });

  it("releases a late response without replacing the current image", async () => {
    let resolveEarlier: ((value: ImageReply) => void) | undefined;
    const earlier = new Promise<ImageReply>((resolve) => {
      resolveEarlier = resolve;
    });
    mock.get.mockReturnValueOnce(earlier).mockResolvedValueOnce({
      status: "ok",
      data: { handle: "current", width: 20, height: 30 },
    });
    const { result, rerender, unmount } = renderHook(
      ({ page }) => useManagedImage("comic", page, "image", "preview", true),
      { initialProps: { page: "old" } },
    );
    rerender({ page: "new" });
    await waitFor(() => {
      expect(result.current.url).toBe("resource://current");
    });
    await act(async () => {
      if (resolveEarlier === undefined) {
        throw new Error("Missing fixture resolver");
      }
      resolveEarlier({
        status: "ok",
        data: { handle: "earlier", width: 20, height: 30 },
      });
      await earlier;
    });
    expect(mock.release).toHaveBeenCalledWith(["earlier"]);
    expect(result.current.url).toBe("resource://current");
    unmount();
    await waitFor(() => {
      expect(mock.release).toHaveBeenCalledWith(["current"]);
    });
  });
});
