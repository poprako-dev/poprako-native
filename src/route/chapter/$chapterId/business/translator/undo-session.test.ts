import { afterEach, beforeEach, expect, test, vi } from "vitest";
import type { PageEditor, SavePageUnits } from "@/bridge/generated/bindings";
import { createEditorSession, isDirty, unitDraft } from "./editor-session";
import type { EditorApi, EditorSession } from "./editor-session";
import { editorFixture } from "./editor-test-fixture";

const sessions: EditorSession[] = [];

function receipt(snapshot: SavePageUnits): PageEditor {
  return {
    page: {
      ...editorFixture().page,
      unit_revision: snapshot.expected_revision + 1,
    },
    units: snapshot.units.map((unit, index) => ({
      ...unit,
      page_id: snapshot.page_id,
      index,
      created_at: 2,
      updated_at: 2,
    })),
  };
}

function setup(): {
  session: EditorSession;
  save: ReturnType<typeof vi.fn<EditorApi["save"]>>;
} {
  const save = vi.fn<EditorApi["save"]>((snapshot) =>
    Promise.resolve(receipt(snapshot)),
  );
  const session = createEditorSession(editorFixture(), "translation", { save });
  sessions.push(session);
  return { session, save };
}

beforeEach(() => {
  vi.useFakeTimers();
});
afterEach(() => {
  for (const session of sessions.splice(0)) session.dispose();
  vi.useRealTimers();
});

test.each(Array.from({ length: 16 }, (_, index) => index))(
  "restores mixed edits across save schedule %i",
  async (mask) => {
    const { session } = setup();
    const snapshots = [session.store.getState().draft];
    session.edit("a", {
      translated_text: "unsent text",
      x_coord: 0.9,
      is_bubble: false,
    });
    snapshots.push(session.store.getState().draft);
    if (mask & 1) await session.save();
    session.edit("b", {
      proofread_text: "校对",
      is_proofread: true,
      is_flagged: true,
    });
    snapshots.push(session.store.getState().draft);
    if (mask & 2) await session.save();
    session.replace([...session.store.getState().draft].reverse());
    snapshots.push(session.store.getState().draft);
    if (mask & 4) await session.save();
    session.replace(
      session.store.getState().draft.filter((unit) => unit.id !== "a"),
    );
    if (mask & 8) await session.save();
    for (const expected of snapshots.reverse()) {
      session.undo();
      expect(session.store.getState().draft).toEqual(expected);
      await session.save();
    }
    expect(session.store.getState().canUndo).toBe(false);
    expect(isDirty(session.store.getState())).toBe(false);
    expect(session.store.getState().baseline.units.map(unitDraft)).toEqual(
      editorFixture().units.map(unitDraft),
    );
  },
);

test("undo during a deletion save preserves original identity and subsequent edits", async () => {
  const { session, save } = setup();
  let complete: (page: PageEditor) => void = () => {
    throw new Error("save not started");
  };
  save.mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        complete = resolve;
      }),
  );
  session.edit("a", { translated_text: "never saved" });
  session.replace(
    session.store.getState().draft.filter((unit) => unit.id !== "a"),
  );
  const first = session.save();
  await Promise.resolve();
  const submitted = save.mock.calls[0]?.[0];
  if (!submitted) throw new Error("missing submission");
  const fixed = structuredClone(submitted);
  session.undo();
  session.edit("b", { translated_text: "new edit" });
  const second = session.save();
  complete(receipt(submitted));
  await Promise.all([first, second]);
  expect(submitted).toEqual(fixed);
  expect(save).toHaveBeenCalledTimes(2);
  expect(save.mock.calls[1]?.[0].expected_revision).toBe(1);
  expect(
    session.store.getState().draft.find((unit) => unit.id === "a")
      ?.translated_text,
  ).toBe("never saved");
  session.undo();
  session.undo();
  expect(session.store.getState().draft).toEqual(
    editorFixture().units.map(unitDraft),
  );
});

test("creation undone during auto save is removed by the next save", async () => {
  const { session, save } = setup();
  let complete: (page: PageEditor) => void = () => {
    throw new Error("save not started");
  };
  save.mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        complete = resolve;
      }),
  );
  const original = session.store.getState().draft;
  const unit = original[0];
  if (!unit) throw new Error("missing fixture");
  session.replace([...original, { ...unit, id: "new" }]);
  session.select("new");
  await vi.advanceTimersByTimeAsync(15_000);
  const sent = save.mock.calls[0]?.[0];
  if (!sent) throw new Error("missing snapshot");
  session.undo();
  expect(session.store.getState().selected).toBe("");
  complete(receipt(sent));
  await vi.advanceTimersByTimeAsync(15_000);
  expect(save).toHaveBeenCalledTimes(2);
  expect(session.store.getState().baseline.units.map(unitDraft)).toEqual(
    original,
  );
  expect(session.store.getState().canUndo).toBe(false);
});

test("failed saves retain history and uncertain submissions block undo until confirmed", async () => {
  const { session, save } = setup();
  save.mockRejectedValueOnce(new Error("磁盘空间不足"));
  session.edit("a", { translated_text: "A" });
  expect(await session.save()).toBe(false);
  session.undo();
  expect(session.store.getState().draft[0]?.translated_text).toBe("翻译 a");
  session.edit("a", { translated_text: "B" });
  save.mockRejectedValueOnce(
    Object.assign(new Error("核实中"), {
      detail: { recovery: "wait_for_confirmation" },
    }),
  );
  expect(await session.save()).toBe(false);
  const before = session.store.getState();
  session.undo();
  expect(session.store.getState()).toBe(before);
  expect(await session.save()).toBe(true);
  expect(save.mock.calls[1]).toEqual(save.mock.calls[2]);
  session.undo();
  expect(session.store.getState().draft[0]?.translated_text).toBe("翻译 a");
  expect(await session.save()).toBe(true);
});

test("raw whitespace steps survive normalized receipts without adding receipt history", async () => {
  const { session, save } = setup();
  session.edit("a", { translated_text: "" });
  await session.save();
  session.clearHistory();
  session.edit("a", { translated_text: " " });
  expect(isDirty(session.store.getState())).toBe(false);
  session.edit("a", { translated_text: "A" });
  await session.save();
  session.undo();
  await session.save();
  expect(session.store.getState().draft[0]?.translated_text).toBe(" ");
  expect(session.store.getState().baseline.units[0]?.translated_text).toBe("");
  session.undo();
  expect(session.store.getState().draft[0]?.translated_text).toBe("");
  expect(session.store.getState().canUndo).toBe(false);
  const count = save.mock.calls.length;
  await session.save();
  expect(save).toHaveBeenCalledTimes(count);
});

test("empty or identical edits do not touch history, selection or timer", () => {
  const { session } = setup();
  const before = session.store.getState();
  session.undo();
  session.edit("a", { translated_text: "翻译 a" });
  session.replace([...before.draft]);
  expect(session.store.getState()).toBe(before);
  expect(vi.getTimerCount()).toBe(0);
});

test("modes and cancelled navigation retain history; reload and discard clear it", async () => {
  const { session } = setup();
  session.select("b");
  session.edit("a", { translated_text: "changed" });
  for (const reason of ["composition", "gesture"] as const) {
    session.suspend(true, reason);
    session.undo();
    expect(session.store.getState().draft[0]?.translated_text).toBe("changed");
    session.suspend(false, reason);
  }
  session.setMode("readonly");
  session.undo();
  expect(session.store.getState().canUndo).toBe(true);
  session.setMode("proofreading");
  await session.flush();
  session.undo();
  expect(session.store.getState().canUndo).toBe(true);
  session.cancelLeave();
  session.undo();
  expect(session.store.getState().selected).toBe("b");
  session.edit("a", { translated_text: "again" });
  session.discard();
  expect(session.store.getState().canUndo).toBe(false);
  session.edit("a", { translated_text: "last" });
  session.reload(editorFixture());
  expect(session.store.getState().canUndo).toBe(false);
});
