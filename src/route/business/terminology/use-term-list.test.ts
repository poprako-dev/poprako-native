import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { useTermList } from "./use-term-list";

afterEach(cleanup);

it("discards late old-query replies and retries only the failed page", async () => {
  let finishOld: (items: string[]) => void = () => undefined;
  const old = vi.fn<(offset: number) => Promise<string[]>>(
    () =>
      new Promise<string[]>((resolve) => {
        finishOld = resolve;
      }),
  );
  const first = Array.from({ length: 30 }, (_, index) => String(index));
  const next = vi
    .fn<(offset: number) => Promise<string[]>>()
    .mockResolvedValueOnce(first)
    .mockRejectedValueOnce(new Error("retry read"))
    .mockResolvedValueOnce(["last"]);
  const { result, rerender } = renderHook(({ load }) => useTermList(load), {
    initialProps: { load: old },
  });
  rerender({ load: next });
  await waitFor(() => {
    expect(result.current.items).toEqual(first);
  });
  await act(async () => {
    finishOld(["stale"]);
    await Promise.resolve();
  });
  expect(result.current.items).toEqual(first);
  act(() => {
    result.current.next();
    result.current.next();
  });
  await waitFor(() => {
    expect(result.current.error).toBe("retry read");
  });
  expect(result.current.items).toEqual(first);
  act(() => {
    result.current.next();
  });
  await waitFor(() => {
    expect(result.current.more).toBe(false);
  });
  expect(result.current.items).toEqual([...first, "last"]);
  expect(next.mock.calls).toEqual([[0], [30], [30]]);
});
