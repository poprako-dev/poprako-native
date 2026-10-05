import { afterEach, describe, expect, it, vi } from "vitest";
import type { PageEditor, SavePageUnits } from "@/bridge/generated/bindings";
import { createEditorSession, isDirty } from "./editor-session";
import type { EditorApi } from "./editor-session";

function fixture(): PageEditor {
  return {
    page: {
      id: "page",
      comic_id: "comic",
      index: 0,
      unit_revision: 0,
      image: {
        reference: "managed",
        original_name: "001.png",
        format: "png",
        width: 100,
        height: 200,
      },
      created_at: 1,
      updated_at: 1,
    },
    units: [
      {
        id: "unit",
        page_id: "page",
        index: 0,
        x_coord: 0.5,
        y_coord: 0.5,
        is_bubble: true,
        is_flagged: false,
        translated_text: "",
        proofread_text: "",
        is_proofread: false,
        created_at: 1,
        updated_at: 1,
      },
    ],
  };
}

function receipt(snapshot: SavePageUnits): PageEditor {
  return {
    page: { ...fixture().page, unit_revision: snapshot.expected_revision + 1 },
    units: snapshot.units.map((unit, index) => ({
      ...unit,
      page_id: "page",
      index,
      created_at: 1,
      updated_at: 2,
    })),
  };
}

describe("editor save lifecycle", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it("resets the only timer until input has been idle for fifteen seconds", async () => {
    vi.useFakeTimers();
    const save = vi.fn<EditorApi["save"]>((snapshot) =>
      Promise.resolve(receipt(snapshot)),
    );
    const session = createEditorSession(fixture(), "translation", { save });
    session.edit("unit", { translated_text: "first" });
    await vi.advanceTimersByTimeAsync(14_000);
    session.edit("unit", { translated_text: "second" });
    await vi.advanceTimersByTimeAsync(14_999);
    expect(save).not.toHaveBeenCalled();
    expect(vi.getTimerCount()).toBe(1);
    await vi.advanceTimersByTimeAsync(1);
    expect(save).toHaveBeenCalledTimes(1);
    expect(isDirty(session.store.getState())).toBe(false);
    session.dispose();
  });

  it("keeps later typing and structural changes after an earlier receipt", async () => {
    vi.useFakeTimers();
    let resolve: (value: PageEditor) => void = () => {
      throw new Error("save did not start");
    };
    const save = vi.fn<EditorApi["save"]>(
      () =>
        new Promise((done) => {
          resolve = done;
        }),
    );
    const session = createEditorSession(fixture(), "translation", { save });
    session.edit("unit", { translated_text: "A" });
    const saving = session.save();
    const submitted = save.mock.calls[0]?.[0];
    expect(submitted).toBeDefined();
    session.edit("unit", { translated_text: "B", is_flagged: true });
    const added = { ...session.store.getState().draft[0], id: "second" };
    if (!added.translated_text || added.x_coord === undefined || !submitted)
      throw new Error("missing fixture");
    const original = session.store.getState().draft[0];
    if (!original) throw new Error("missing unit");
    session.replace([{ ...original, id: "second" }, original]);
    resolve(receipt(submitted));
    expect(await saving).toBe(true);
    expect(session.store.getState().draft.map((unit) => unit.id)).toEqual([
      "second",
      "unit",
    ]);
    expect(session.store.getState().draft[1]?.translated_text).toBe("B");
    expect(session.store.getState().baseline.units[0]?.translated_text).toBe(
      "A",
    );
    expect(isDirty(session.store.getState())).toBe(true);
    session.dispose();
  });

  it("serializes overlapping saves and submits subsequent changes against the new revision", async () => {
    vi.useFakeTimers();
    let resolve: (value: PageEditor) => void = () => {
      throw new Error("missing save");
    };
    const save = vi
      .fn<EditorApi["save"]>()
      .mockImplementationOnce(
        () =>
          new Promise((done) => {
            resolve = done;
          }),
      )
      .mockImplementation((snapshot) => Promise.resolve(receipt(snapshot)));
    const session = createEditorSession(fixture(), "translation", { save });
    session.edit("unit", { translated_text: "A" });
    const first = session.save();
    session.edit("unit", { translated_text: "B" });
    const second = session.save();
    expect(save).toHaveBeenCalledTimes(1);
    const snapshot = save.mock.calls[0]?.[0];
    if (!snapshot) throw new Error("missing snapshot");
    resolve(receipt(snapshot));
    expect(await first).toBe(true);
    expect(await second).toBe(true);
    expect(save).toHaveBeenCalledTimes(2);
    expect(save.mock.calls[1]?.[0].expected_revision).toBe(1);
    expect(save.mock.calls[1]?.[0].units[0]?.translated_text).toBe("B");
    session.dispose();
  });

  it("retains a failed draft without automatic retries, then flushes the latest draft", async () => {
    vi.useFakeTimers();
    const save = vi
      .fn<EditorApi["save"]>()
      .mockRejectedValueOnce(new Error("磁盘空间不足"))
      .mockImplementation((snapshot) => Promise.resolve(receipt(snapshot)));
    const session = createEditorSession(fixture(), "translation", { save });
    session.edit("unit", { translated_text: "A" });
    expect(await session.save()).toBe(false);
    session.edit("unit", { translated_text: "B" });
    await vi.advanceTimersByTimeAsync(60_000);
    expect(save).toHaveBeenCalledTimes(1);
    expect(await session.flush()).toBe(true);
    expect(save.mock.calls[1]?.[0].units[0]?.translated_text).toBe("B");
    expect(session.store.getState().locked).toBe(true);
    session.cancelLeave();
    expect(session.store.getState().locked).toBe(false);
    session.dispose();
  });

  it("does not save composition or drag intermediate state", async () => {
    vi.useFakeTimers();
    const save = vi.fn<EditorApi["save"]>((snapshot) =>
      Promise.resolve(receipt(snapshot)),
    );
    const session = createEditorSession(fixture(), "translation", { save });
    session.suspend(true);
    session.edit("unit", { translated_text: "拼音" });
    await vi.advanceTimersByTimeAsync(60_000);
    expect(await session.save()).toBe(false);
    expect(save).not.toHaveBeenCalled();
    session.suspend(false);
    await vi.advanceTimersByTimeAsync(15_000);
    expect(save).toHaveBeenCalledTimes(1);
    session.dispose();
  });

  it("reconciles the exact uncertain snapshot while preserving later input", async () => {
    vi.useFakeTimers();
    const failure = Object.assign(new Error("正在核实"), {
      detail: { recovery: "wait_for_confirmation" },
    });
    const save = vi
      .fn<EditorApi["save"]>()
      .mockRejectedValueOnce(failure)
      .mockImplementation((snapshot) => Promise.resolve(receipt(snapshot)));
    const session = createEditorSession(fixture(), "translation", { save });
    session.edit("unit", { translated_text: "A" });
    const first = session.save();
    session.edit("unit", { translated_text: "B" });
    expect(await first).toBe(false);
    expect(session.discard()).toBe(false);
    session.edit("unit", { translated_text: "blocked" });
    expect(await session.save()).toBe(true);
    expect(save).toHaveBeenCalledTimes(2);
    expect(save.mock.calls[1]).toEqual(save.mock.calls[0]);
    expect(session.store.getState().baseline.units[0]?.translated_text).toBe(
      "A",
    );
    expect(session.store.getState().draft[0]?.translated_text).toBe("B");
    expect(isDirty(session.store.getState())).toBe(true);
    session.dispose();
  });

  it("does not persist whitespace-only changes or let readonly mutate content", async () => {
    const save = vi.fn<EditorApi["save"]>();
    const session = createEditorSession(fixture(), "translation", { save });
    session.edit("unit", { translated_text: " \t\n" });
    expect(await session.save()).toBe(true);
    expect(save).not.toHaveBeenCalled();
    session.setMode("readonly");
    session.edit("unit", { is_proofread: true });
    expect(session.store.getState().draft[0]?.is_proofread).toBe(false);
    session.dispose();
  });
});
