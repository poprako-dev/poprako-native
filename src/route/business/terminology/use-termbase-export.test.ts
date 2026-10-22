import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { commands } from "@/bridge";
import { useTermbaseExport } from "./use-termbase-export";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

it("deduplicates immediate calls and ignores completions from an old context", async () => {
  const finish =
    vi.fn<
      (result: Awaited<ReturnType<typeof commands.exportTermbase>>) => void
    >();
  const pending = new Promise<
    Awaited<ReturnType<typeof commands.exportTermbase>>
  >((resolve) => {
    finish.mockImplementation(resolve);
  });
  const exported = vi
    .spyOn(commands, "exportTermbase")
    .mockReturnValueOnce(pending)
    .mockResolvedValue({ status: "ok", data: "cancelled" });
  const view = renderHook(
    ({ chapterId }) =>
      useTermbaseExport({ kind: "chapter", chapter_id: chapterId }),
    {
      initialProps: { chapterId: "one" },
      reactStrictMode: true,
    },
  );
  let first: Promise<void> = Promise.resolve();
  act(() => {
    first = view.result.current.exportTermbase("base");
    void view.result.current.exportTermbase("base");
  });
  expect(exported).toHaveBeenCalledOnce();
  expect(view.result.current.pending.has("base")).toBe(true);
  view.rerender({ chapterId: "two" });
  expect(view.result.current.pending.size).toBe(0);
  await act(() => view.result.current.exportTermbase("base"));
  await act(async () => {
    finish({ status: "ok", data: "saved" });
    await first;
  });
  expect(view.result.current.notice).toBe("");
  expect(view.result.current.pending.size).toBe(0);
  expect(exported).toHaveBeenCalledTimes(2);
});
