import { beforeEach, describe, expect, it, vi } from "vitest";

const native = vi.hoisted(() => ({
  getWriteSession: vi.fn(),
  acknowledgeWriteResult: vi.fn(),
  getWriteResult: vi.fn(),
}));
vi.mock("./generated/bindings", () => ({ commands: native }));

beforeEach(() => {
  vi.resetModules();
  vi.resetAllMocks();
  native.getWriteSession.mockResolvedValue({
    status: "ok",
    data: { session_id: "session", next_sequence: 1 },
  });
  native.acknowledgeWriteResult.mockResolvedValue({ status: "ok", data: null });
});

describe("write receipt recovery", () => {
  it("recovers two lost replies using the original request and fixed payload", async () => {
    const { write } = await import("./write");
    native.getWriteResult.mockResolvedValue({
      status: "ok",
      data: { status: "committed" },
    });
    const operation = vi
      .fn()
      .mockRejectedValueOnce(new Error("lost reply"))
      .mockRejectedValueOnce(new Error("lost reply"))
      .mockResolvedValue({ status: "ok", data: { revision: 2 } });
    const first = await write("save", ["original"], operation);
    expect(first).toMatchObject({
      status: "error",
      error: { code: "commit_unknown" },
    });
    const changed = vi.fn();
    await write("save", ["changed"], changed);
    expect(changed).not.toHaveBeenCalled();
    const result = await write("save", ["original"], operation);
    expect(result).toEqual({ status: "ok", data: { revision: 2 } });
    expect(operation.mock.calls).toEqual(
      Array.from({ length: 3 }, () => [{ session_id: "session", sequence: 1 }]),
    );
    expect(native.getWriteSession).toHaveBeenCalledTimes(1);
  });

  it("does not lose a committed result when acknowledgement replies are lost", async () => {
    const { write } = await import("./write");
    native.acknowledgeWriteResult.mockRejectedValueOnce(
      new Error("lost acknowledgement"),
    );
    const operation = vi.fn().mockResolvedValue({ status: "ok", data: 42 });
    expect(await write("save", [1], operation)).toEqual({
      status: "ok",
      data: 42,
    });
    native.getWriteSession.mockResolvedValue({
      status: "ok",
      data: { session_id: "session", next_sequence: 2 },
    });
    expect(await write("save", [2], operation)).toEqual({
      status: "ok",
      data: 42,
    });
    expect(operation.mock.calls).toEqual([
      [{ session_id: "session", sequence: 1 }],
      [{ session_id: "session", sequence: 2 }],
    ]);
    expect(native.acknowledgeWriteResult.mock.calls.slice(0, 2)).toEqual([
      [{ session_id: "session", sequence: 1 }],
      [{ session_id: "session", sequence: 1 }],
    ]);
  });

  it("keeps uncertain database commits unconsumed", async () => {
    const { write } = await import("./write");
    const operation = vi.fn().mockResolvedValue({
      status: "error",
      error: {
        code: "commit_unknown",
        message: "pending verification",
        recovery: "wait_for_confirmation",
      },
    });
    await write("save", [1], operation);
    expect(native.acknowledgeWriteResult).not.toHaveBeenCalled();
  });
});
