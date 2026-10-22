import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { commands } from "@/bridge";
import { useTermbaseAppend } from "./use-termbase-append";

vi.mock("@tanstack/react-router", () => ({ useBlocker: vi.fn() }));

const document = { terms: [{ source: "Elf", targets: ["精灵"], comment: "" }] };
const outcome = {
  status: "ok",
  data: { added: 1, updated: 0, unchanged: 0 },
} as const;

function deferred<T>(): { promise: Promise<T>; complete: (value: T) => void } {
  const complete = vi.fn<(value: T) => void>();
  const promise = new Promise<T>((resolve) => {
    complete.mockImplementation(resolve);
  });
  return { promise, complete };
}

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

it("deduplicates immediate clicks and does not write a selection returned after leaving its context", async () => {
  const pending =
    deferred<Awaited<ReturnType<typeof commands.chooseTermbaseAppend>>>();
  const choose = vi
    .spyOn(commands, "chooseTermbaseAppend")
    .mockReturnValue(pending.promise);
  const append = vi
    .spyOn(commands, "appendTermbase")
    .mockResolvedValue(outcome);
  const onAppended = vi.fn();
  const view = renderHook(
    ({ chapterId }) =>
      useTermbaseAppend({ kind: "chapter", chapter_id: chapterId }, onAppended),
    { initialProps: { chapterId: "one" }, reactStrictMode: true },
  );
  let first = Promise.resolve();
  act(() => {
    first = view.result.current.appendTermbase("base");
    void view.result.current.appendTermbase("base");
  });
  expect(choose).toHaveBeenCalledTimes(1);
  expect(view.result.current.locked).toBe(true);
  view.rerender({ chapterId: "two" });
  await act(async () => {
    pending.complete({ status: "ok", data: document });
    await first;
  });
  expect(append).not.toHaveBeenCalled();
  expect(view.result.current.locked).toBe(false);
  expect(view.result.current.notice).toBe("");
  expect(onAppended).not.toHaveBeenCalled();
});

it("only confirms an uncertain original job and deduplicates immediate confirmation clicks", async () => {
  const choose = vi
    .spyOn(commands, "chooseTermbaseAppend")
    .mockResolvedValue({ status: "ok", data: document });
  const pending =
    deferred<Awaited<ReturnType<typeof commands.appendTermbase>>>();
  const append = vi
    .spyOn(commands, "appendTermbase")
    .mockResolvedValueOnce({
      status: "error",
      error: {
        code: "commit_unknown",
        message: "等待核实",
        recovery: "wait_for_confirmation",
      },
    })
    .mockReturnValueOnce(pending.promise);
  const onAppended = vi.fn();
  const view = renderHook(() =>
    useTermbaseAppend({ kind: "global" }, onAppended),
  );
  await act(() => view.result.current.appendTermbase("base"));
  expect(view.result.current.uncertain).toBe(true);
  await act(() => view.result.current.appendTermbase("other"));
  expect(choose).toHaveBeenCalledTimes(1);
  let verification = Promise.resolve();
  act(() => {
    verification = view.result.current.retry();
    void view.result.current.retry();
  });
  expect(append).toHaveBeenCalledTimes(2);
  await act(async () => {
    pending.complete(outcome);
    await verification;
  });
  expect(onAppended).toHaveBeenCalledTimes(1);
  expect(view.result.current.locked).toBe(false);
  expect(append.mock.calls).toEqual([
    [{ kind: "global" }, "base", document],
    [{ kind: "global" }, "base", document],
  ]);
});

it("keeps unexpected IPC failures and recovery-required outcomes locked until confirmed", async () => {
  vi.spyOn(commands, "chooseTermbaseAppend").mockResolvedValue({
    status: "ok",
    data: document,
  });
  vi.spyOn(commands, "appendTermbase")
    .mockRejectedValueOnce(new Error("IPC lost"))
    .mockResolvedValueOnce({
      status: "error",
      error: {
        code: "recovery_required",
        message: "需要核实",
        recovery: "restart",
      },
    })
    .mockResolvedValue(outcome);
  const view = renderHook(() => useTermbaseAppend({ kind: "global" }, vi.fn()));
  await act(() => view.result.current.appendTermbase("base"));
  expect(view.result.current.uncertain).toBe(true);
  await act(() => view.result.current.retry());
  expect(view.result.current.locked).toBe(true);
  expect(view.result.current.uncertain).toBe(true);
  await act(() => view.result.current.retry());
  expect(view.result.current.locked).toBe(false);
});

it("does not notify a new context when an old write completes", async () => {
  vi.spyOn(commands, "chooseTermbaseAppend").mockResolvedValue({
    status: "ok",
    data: document,
  });
  const pending =
    deferred<Awaited<ReturnType<typeof commands.appendTermbase>>>();
  vi.spyOn(commands, "appendTermbase").mockReturnValue(pending.promise);
  const onAppended = vi.fn();
  const view = renderHook(
    ({ chapterId }) =>
      useTermbaseAppend({ kind: "chapter", chapter_id: chapterId }, onAppended),
    { initialProps: { chapterId: "one" } },
  );
  let first = Promise.resolve();
  await act(async () => {
    first = view.result.current.appendTermbase("base");
    await Promise.resolve();
  });
  view.rerender({ chapterId: "two" });
  await act(async () => {
    pending.complete(outcome);
    await first;
  });
  expect(onAppended).not.toHaveBeenCalled();
  expect(view.result.current.notice).toBe("");
});
