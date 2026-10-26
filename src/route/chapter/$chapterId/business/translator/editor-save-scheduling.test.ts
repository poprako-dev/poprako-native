import { afterEach, expect, test, vi } from "vitest";
import type { PageEditor, SavePageUnits } from "@/bridge/generated/bindings";
import { createEditorSession, isDirty } from "./editor-session";
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
      index,
      page_id: snapshot.page_id,
      created_at: 1,
      updated_at: 2,
    })),
  };
}

function setup(save: EditorApi["save"]): EditorSession {
  const session = createEditorSession(editorFixture(), "translation", {
    save,
  });
  sessions.push(session);
  return session;
}

afterEach(() => {
  for (const session of sessions.splice(0)) session.dispose();
  vi.useRealTimers();
});

test("registers the running save before notifying synchronous subscribers", async () => {
  const save = vi.fn<EditorApi["save"]>((snapshot) =>
    Promise.resolve(receipt(snapshot)),
  );
  const session = setup(save);
  session.edit("a", { translated_text: "手动保存" });
  const joined: Promise<boolean>[] = [];
  const unsubscribe = session.store.subscribe((state, previous) => {
    if (state.busy && !previous.busy) joined.push(session.save());
  });

  expect(await session.save()).toBe(true);
  expect(await Promise.all(joined)).toEqual([true]);
  unsubscribe();
  expect(save).toHaveBeenCalledTimes(1);
  expect(session.store.getState().busy).toBe(false);
});

test("clears saving after disposal even when the receipt is intentionally ignored", async () => {
  let resolve: (value: PageEditor) => void = () => {
    throw new Error("Missing save resolver");
  };
  const result = new Promise<PageEditor>((done) => {
    resolve = done;
  });
  const save = vi.fn<EditorApi["save"]>(() => result);
  const session = setup(save);
  session.edit("a", { translated_text: "已发送" });
  const saving = session.save();
  await Promise.resolve();
  const sent = save.mock.calls[0]?.[0];
  if (!sent) throw new Error("Missing save snapshot");
  session.dispose();
  resolve(receipt(sent));
  expect(await saving).toBe(false);
  expect(session.store.getState().busy).toBe(false);
  expect(session.store.getState().baseline.page.unit_revision).toBe(0);
});

test("buffers new markers during manual saving and waits for the gesture before leaving", async () => {
  let resolve: (value: PageEditor) => void = () => {
    throw new Error("Missing save resolver");
  };
  const result = new Promise<PageEditor>((done) => {
    resolve = done;
  });
  const save = vi
    .fn<EditorApi["save"]>()
    .mockImplementationOnce(() => result)
    .mockImplementation((snapshot) => Promise.resolve(receipt(snapshot)));
  const session = setup(save);
  session.edit("a", { translated_text: "第一版" });
  const saving = session.save();
  await Promise.resolve();
  const sent = save.mock.calls[0]?.[0];
  const original = session.store.getState().draft[0];
  if (!sent || !original) throw new Error("Missing save fixture");

  session.suspend(true, "gesture");
  let allowed: boolean | undefined;
  const leaving = session.flush().then((value) => {
    allowed = value;
  });
  await Promise.resolve();
  expect(allowed).toBeUndefined();
  expect(session.store.getState().locked).toBe(false);
  session.replace([
    ...session.store.getState().draft,
    { ...original, id: "new-marker", x_coord: 0.7 },
  ]);
  session.edit("a", { translated_text: "保存期间继续编辑" });
  session.suspend(false, "gesture");
  resolve(receipt(sent));
  expect(await saving).toBe(true);
  await leaving;

  expect(allowed).toBe(true);
  expect(save).toHaveBeenCalledTimes(2);
  expect(save.mock.calls[1]?.[0].expected_revision).toBe(1);
  expect(session.store.getState().draft).toHaveLength(4);
  expect(session.store.getState().draft[0]?.translated_text).toBe(
    "保存期间继续编辑",
  );
  expect(isDirty(session.store.getState())).toBe(false);
  expect(session.store.getState().busy).toBe(false);
});

test.each([false, true])(
  "autosave, flush and another save wait for every required receipt (new marker: %s)",
  async (newMarker) => {
    vi.useFakeTimers();
    const complete: (() => void)[] = [];
    const save = vi.fn<EditorApi["save"]>(
      (snapshot) =>
        new Promise((resolve) => {
          complete.push(() => {
            resolve(receipt(snapshot));
          });
        }),
    );
    const session = setup(save);
    const marker = session.store.getState().draft[0];
    if (!marker) throw new Error("Missing marker");
    session.replace([
      ...session.store.getState().draft,
      { ...marker, id: "first-marker" },
    ]);
    await vi.advanceTimersByTimeAsync(15_000);
    expect(save).toHaveBeenCalledTimes(1);
    if (newMarker)
      session.replace([
        ...session.store.getState().draft,
        { ...marker, id: "second-marker" },
      ]);
    let departed = false;
    const leaving = session.flush().then((success) => {
      if (success) {
        departed = true;
        session.dispose();
      }
      return success;
    });
    const repeated = session.save();
    complete[0]?.();
    await vi.advanceTimersByTimeAsync(0);
    expect(departed).toBe(!newMarker);
    expect(save).toHaveBeenCalledTimes(newMarker ? 2 : 1);
    if (newMarker) {
      expect(save.mock.calls[1]?.[0].expected_revision).toBe(1);
      expect(session.store.getState().busy).toBe(true);
      complete[1]?.();
    }
    expect(await leaving).toBe(true);
    await repeated;
    expect(isDirty(session.store.getState())).toBe(false);
    expect(session.store.getState().busy).toBe(false);
    expect(session.store.getState().error).toBe("");
  },
);
