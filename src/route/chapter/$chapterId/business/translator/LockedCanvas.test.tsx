import { useStore } from "zustand";
import type { ReactElement } from "react";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import type { PageEditor, SavePageUnits } from "@/bridge/generated/bindings";
import { Canvas } from "./Canvas";
import { LeaveGuard } from "./LeaveGuard";
import { createEditorSession } from "./editor-session";
import type { EditorApi, EditorSession } from "./editor-session";
import { editorFixture } from "./editor-test-fixture";

vi.mock("@tanstack/react-router", () => ({ useBlocker: vi.fn() }));
vi.mock("@/bridge/exit", () => ({ registerExitGuard: () => () => undefined }));

class TestPointerEvent extends MouseEvent {
  readonly pointerId = 1;
}

let session: EditorSession;
const save = vi.fn<EditorApi["save"]>();
const guardRef = {
  current: { run: (): Promise<boolean> => Promise.resolve(false) },
};
function savePosition(): Promise<boolean> {
  return Promise.resolve(true);
}

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
      created_at: 1,
      updated_at: 2,
    })),
  };
}

function Harness(): ReactElement {
  const state = useStore(session.store);
  return (
    <>
      <Canvas
        state={state}
        session={session}
        imageUrl="test.png"
        imageError=""
        opacity={1}
        relocation={false}
        preview={true}
        highQuality={false}
        creationEnabled={true}
        onDelete={() => undefined}
      />
      <LeaveGuard
        session={session}
        guardRef={guardRef}
        savePosition={savePosition}
      />
    </>
  );
}

beforeEach(() => {
  vi.stubGlobal("PointerEvent", TestPointerEvent);
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe = vi.fn();
      disconnect = vi.fn();
    },
  );
  HTMLElement.prototype.scrollIntoView = vi.fn();
  HTMLElement.prototype.setPointerCapture = vi.fn();
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue(
    new DOMRect(0, 0, 100, 100),
  );
  save.mockReset();
  save.mockImplementation((snapshot) => Promise.resolve(receipt(snapshot)));
  session = createEditorSession(
    { ...editorFixture(), units: [] },
    "translation",
    { save },
  );
});

afterEach(() => {
  cleanup();
  session.dispose();
  vi.unstubAllGlobals();
});

/**
 * 用户场景：翻页的 flush 正在等一次保存（session 已 locked），此时用户在画布上
 * 点一下想新建 marker；保存失败后“修改尚未保存”对话框盖上来，pointerup 落在
 * 对话框而不是画布。若 locked 状态下仍 start() 了手势，suspension 永远留着，
 * “重试保存”就会一直失败/无反应。
 */
test("a canvas click while the page-turn flush is locked cannot wedge the retry", async () => {
  let failFirst: (error: Error) => void = () => undefined;
  save
    .mockImplementationOnce(
      () =>
        new Promise<PageEditor>((_resolve, reject) => {
          failFirst = reject;
        }),
    )
    .mockImplementation((snapshot) => Promise.resolve(receipt(snapshot)));

  render(<Harness />);
  const canvas = screen.getByRole("application");

  // 先建一个 marker，让翻页有内容要保存。
  fireEvent.pointerDown(canvas, { button: 0, clientX: 20, clientY: 50 });
  fireEvent.pointerUp(canvas, { button: 0, clientX: 20, clientY: 50 });

  // 点翻页：flush 锁住 session，保存挂起。
  let leaving: Promise<boolean> | undefined;
  await act(async () => {
    leaving = guardRef.current.run();
    await Promise.resolve();
  });
  expect(session.store.getState().locked).toBe(true);
  expect(screen.queryByRole("dialog")).toBeNull();

  // 等待保存时再次点击画布，locked 必须阻止新手势。
  fireEvent.pointerDown(canvas, { button: 0, clientX: 60, clientY: 50 });

  // 保存失败，弹“修改尚未保存”。
  await act(async () => {
    failFirst(new Error("磁盘错误"));
    await Promise.resolve();
  });
  const dialog = await screen.findByRole("dialog");
  expect(dialog.textContent).toContain("修改尚未保存");

  // pointerup 落在对话框上，画布的 end() 收不到。
  fireEvent.pointerUp(dialog, { button: 0, clientX: 60, clientY: 50 });

  // 再保存一次会成功；“重试保存”必须真的离开。
  fireEvent.click(screen.getByRole("button", { name: "重试保存" }));
  const outcome = await Promise.race([
    leaving?.then((value) => value),
    new Promise<"timeout">((resolve) => {
      setTimeout(() => {
        resolve("timeout");
      }, 500);
    }),
  ]);
  expect(outcome).toBe(true);
  await waitFor(() => {
    expect(screen.queryByRole("dialog")).toBeNull();
  });
  expect(save.mock.calls.length).toBeGreaterThanOrEqual(2);
});
