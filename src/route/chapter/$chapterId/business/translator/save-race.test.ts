import { afterEach, describe, expect, it, vi } from "vitest";
import type { PageEditor, SavePageUnits } from "@/bridge/generated/bindings";
import { createEditorSession, isDirty } from "./editor-session";
import type { EditorApi, EditorSession } from "./editor-session";
import { editorFixture } from "./editor-test-fixture";

const sessions: EditorSession[] = [];

function deferredReceipt(): {
  promise: Promise<PageEditor>;
  resolve: (value: PageEditor) => void;
} {
  let resolve: (value: PageEditor) => void = () => {
    throw new Error("Missing save resolver");
  };
  const promise = new Promise<PageEditor>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

function receipt(snapshot: SavePageUnits): PageEditor {
  const fixture = editorFixture();
  return {
    page: {
      ...fixture.page,
      unit_revision: snapshot.expected_revision + 1,
    },
    units: snapshot.units.map((unit, index) => ({
      ...unit,
      page_id: snapshot.page_id,
      index,
      created_at: 1,
      updated_at: 2,
    })),
  };
}

function setup(save: EditorApi["save"]): EditorSession {
  const session = createEditorSession(editorFixture(), "translation", { save });
  sessions.push(session);
  return session;
}

function addMarker(session: EditorSession, id: string): void {
  const original = session.store.getState().draft[0];
  if (!original) throw new Error("Missing marker fixture");
  session.replace([
    ...session.store.getState().draft,
    { ...original, id, x_coord: 0.9 },
  ]);
}

afterEach(() => {
  for (const session of sessions.splice(0)) session.dispose();
  vi.useRealTimers();
});

/**
 * 翻页必须等待在途保存及后续草稿，后续提交使用回执推进后的版本。
 * 会话销毁后不再接受旧回执，但仍须清理在途状态。
 */
describe("page turn racing an in-flight marker save", () => {
  it("waits for the in-flight auto save, keeps a marker added meanwhile, and leaves no busy", async () => {
    const deferred = deferredReceipt();
    const save = vi
      .fn<EditorApi["save"]>()
      .mockImplementationOnce(() => deferred.promise)
      .mockImplementation((snapshot) => Promise.resolve(receipt(snapshot)));
    const session = setup(save);

    addMarker(session, "marker-1");
    const autoSave = session.save();
    await Promise.resolve();
    const sent = save.mock.calls[0]?.[0];
    if (!sent) throw new Error("auto save did not start");
    expect(session.store.getState().busy).toBe(true);

    // 普通保存等待回执期间仍允许新增 marker。
    addMarker(session, "marker-2");

    // 这时用户点翻页。
    const leaving = session.flush();
    await Promise.resolve();

    deferred.resolve(receipt(sent));
    expect(await autoSave).toBe(true);
    expect(await leaving).toBe(true);

    // 只允许一次“旧 revision”的提交；第二次必须用被回执推进后的 revision。
    expect(save).toHaveBeenCalledTimes(2);
    expect(save.mock.calls[1]?.[0].expected_revision).toBe(1);
    expect(save.mock.calls[1]?.[0].units.map((unit) => unit.id)).toEqual([
      ...sent.units.map((unit) => unit.id),
      "marker-2",
    ]);
    expect(session.store.getState().busy).toBe(false);
    expect(session.store.getState().error).toBe("");
    expect(session.store.getState().baseline.units).toHaveLength(
      sent.units.length + 1,
    );
    expect(isDirty(session.store.getState())).toBe(false);
  });

  it("does not leave busy behind when the editor is torn down mid-save (receipt already committed)", async () => {
    const deferred = deferredReceipt();
    const save = vi.fn<EditorApi["save"]>(() => deferred.promise);
    const session = setup(save);

    addMarker(session, "marker-1");
    const saving = session.save();
    await Promise.resolve();
    const sent = save.mock.calls[0]?.[0];
    if (!sent) throw new Error("save did not start");
    expect(session.store.getState().busy).toBe(true);

    // 旧 session 已销毁，但原生写入仍可能完成。
    session.dispose();
    deferred.resolve(receipt(sent));

    expect(await saving).toBe(false);
    // 即使不再接收回执，也必须结束旧会话的 busy 状态。
    expect(session.store.getState().busy).toBe(false);
    // 回执被丢弃，基线保持旧 revision；之后任何保存都不能拿旧 revision 再发一次。
    expect(session.store.getState().baseline.page.unit_revision).toBe(0);
    expect(session.store.getState().error).toBe("");
  });

  it("serializes a second save that is triggered synchronously while the first one is starting", async () => {
    const deferred = deferredReceipt();
    const save = vi
      .fn<EditorApi["save"]>()
      .mockImplementationOnce(() => deferred.promise)
      .mockImplementation((snapshot) => Promise.resolve(receipt(snapshot)));
    const session = setup(save);
    addMarker(session, "marker-1");

    // 模拟 busy 状态同步通知里再触发一次保存（自动保存/翻页/订阅者重叠）。
    const joined: Promise<boolean>[] = [];
    const unsubscribe = session.store.subscribe((state, previous) => {
      if (state.busy && !previous.busy) joined.push(session.save());
    });

    const first = session.save();
    await Promise.resolve();
    const sent = save.mock.calls[0]?.[0];
    if (!sent) throw new Error("save did not start");
    deferred.resolve(receipt(sent));

    expect(await first).toBe(true);
    expect(await Promise.all(joined)).toEqual([true]);
    // busy 的同步通知触发保存时，必须能够加入已注册的在途请求。
    const revisions = save.mock.calls.map((call) => call[0].expected_revision);
    expect(new Set(revisions).size).toBe(revisions.length);
    expect(save).toHaveBeenCalledTimes(1);
    unsubscribe();
  });
});

describe("page turn with unchanged and newer drafts", () => {
  it("no new edit while A is in flight: flush waits for A but sends no second request", async () => {
    const deferred = deferredReceipt();
    const save = vi.fn<EditorApi["save"]>(() => deferred.promise);
    const session = setup(save);
    addMarker(session, "marker-1");

    const autoSave = session.save();
    await Promise.resolve();
    const sent = save.mock.calls[0]?.[0];
    if (!sent) throw new Error("auto save did not start");

    let left = false;
    const leaving = session.flush().then((value) => {
      left = value;
    });
    await Promise.resolve();
    expect(left).toBe(false);
    deferred.resolve(receipt(sent));
    expect(await autoSave).toBe(true);
    await leaving;
    expect(left).toBe(true);

    // 递归 save() 检查 isDirty：同一份已保存草稿不再发请求。
    expect(save).toHaveBeenCalledTimes(1);
    expect(session.store.getState().busy).toBe(false);
    expect(isDirty(session.store.getState())).toBe(false);
  });

  it("a marker added while A is in flight: the second request uses the advanced revision", async () => {
    const deferred = deferredReceipt();
    const save = vi
      .fn<EditorApi["save"]>()
      .mockImplementationOnce(() => deferred.promise)
      .mockImplementation((snapshot) => Promise.resolve(receipt(snapshot)));
    const session = setup(save);
    addMarker(session, "marker-1");

    const autoSave = session.save();
    await Promise.resolve();
    const sent = save.mock.calls[0]?.[0];
    if (!sent) throw new Error("auto save did not start");
    addMarker(session, "marker-2");
    expect(isDirty(session.store.getState())).toBe(true);

    const leaving = session.flush();
    await Promise.resolve();
    deferred.resolve(receipt(sent));
    expect(await autoSave).toBe(true);
    expect(await leaving).toBe(true);

    expect(save).toHaveBeenCalledTimes(2);
    expect(save.mock.calls[1]?.[0].expected_revision).toBe(1);
    expect(session.store.getState().baseline.units).toHaveLength(
      sent.units.length + 1,
    );
    expect(isDirty(session.store.getState())).toBe(false);
  });
});
