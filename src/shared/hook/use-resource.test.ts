import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useResource } from "@/shared/hook/use-resource";

describe("resource request ownership", () => {
  it("ignores an earlier request after its consumer switches", async () => {
    const earlier = (): Promise<string> =>
      new Promise((resolve) => {
        setTimeout(() => {
          resolve("earlier");
        }, 40);
      });
    const current = (): Promise<string> => Promise.resolve("current");
    const { result, rerender } = renderHook(({ load }) => useResource(load), {
      initialProps: { load: earlier },
    });

    rerender({ load: current });
    await waitFor(() => {
      expect(result.current.state).toEqual({
        status: "ready",
        value: "current",
      });
    });
    await act(
      () =>
        new Promise<void>((resolve) => {
          setTimeout(resolve, 60);
        }),
    );
    expect(result.current.state).toEqual({ status: "ready", value: "current" });
  });

  it("reports a failed read and allows an explicit retry", async () => {
    const load = vi
      .fn<() => Promise<string>>()
      .mockRejectedValueOnce(new Error("private diagnostic"))
      .mockResolvedValueOnce("recovered");
    const { result } = renderHook(() => useResource(load));

    await waitFor(() => {
      expect(result.current.state).toEqual({ status: "error" });
    });
    act(() => {
      result.current.reload();
    });
    await waitFor(() => {
      expect(result.current.state).toEqual({
        status: "ready",
        value: "recovered",
      });
    });
    expect(load).toHaveBeenCalledTimes(2);
  });
});
