import { useStore } from "zustand";
import type { ReactElement } from "react";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import type { PageEditor, SavePageUnits } from "@/bridge/generated/bindings";
import { Canvas } from "./Canvas";
import { EditorToolbar } from "./EditorToolbar";
import { LeaveGuard } from "./LeaveGuard";
import { UnitList } from "./UnitList";
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
      <EditorToolbar
        state={state}
        session={session}
        relocation={false}
        onRelocation={() => undefined}
        preview={true}
        onPreview={() => undefined}
        highQuality={false}
        onQuality={() => undefined}
        creationEnabled={true}
        onCreation={() => undefined}
      />
      <UnitList
        state={state}
        session={session}
        flagged={false}
        characters={[]}
        symbolRequest={null}
        onDelete={() => undefined}
        characterBar={{
          isEnabled: true,
          position: null,
          isGripHeld: false,
          placeholderHeight: null,
          handleGripPointerDown: () => undefined,
          dock: () => undefined,
        }}
      />
      <LeaveGuard
        session={session}
        guardRef={guardRef}
        savePosition={() => Promise.resolve(true)}
      />
    </>
  );
}

function point(x: number): void {
  const canvas = screen.getByRole("application");
  fireEvent.pointerDown(canvas, { button: 0, clientX: x, clientY: 50 });
  fireEvent.pointerUp(canvas, { button: 0, clientX: x, clientY: 50 });
}

beforeEach(() => {
  vi.useFakeTimers();
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
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

test("manual saving keeps marker creation and typing available and preserves newer drafts", async () => {
  let resolve: (value: PageEditor) => void = () => {
    throw new Error("Missing save resolver");
  };
  const first = new Promise<PageEditor>((done) => {
    resolve = done;
  });
  save.mockImplementationOnce(() => first);
  render(<Harness />);
  point(20);
  fireEvent.click(screen.getByRole("button", { name: "保存，待保存" }));
  await act(() => Promise.resolve());
  const sent = save.mock.calls[0]?.[0];
  if (!sent) throw new Error("Save button did not submit");
  expect(sent.units).toHaveLength(1);
  expect(screen.getByRole("button", { name: "保存，保存中" })).toBeDefined();

  point(70);
  fireEvent.change(screen.getByRole("textbox", { name: "翻译 2" }), {
    target: { value: "保存期间的新译文" },
  });
  expect(session.store.getState().locked).toBe(false);
  expect(session.store.getState().draft).toHaveLength(2);
  expect(screen.queryByRole("dialog")).toBeNull();
  await act(async () => {
    resolve(receipt(sent));
    await Promise.resolve();
  });
  expect(screen.getByRole("button", { name: "保存，待保存" })).toBeDefined();
  expect(session.store.getState().draft[1]?.translated_text).toBe(
    "保存期间的新译文",
  );
  await act(async () => {
    fireEvent.click(screen.getByRole("button", { name: "保存，待保存" }));
    await Promise.resolve();
  });
  expect(screen.getByRole("button", { name: "保存，已保存" })).toBeDefined();
  expect(save).toHaveBeenCalledTimes(2);
  expect(save.mock.calls[1]?.[0].units).toHaveLength(2);
});

test("leaving during a marker gesture waits for pointerup instead of opening the save error dialog", async () => {
  render(<Harness />);
  const canvas = screen.getByRole("application");
  fireEvent.pointerDown(canvas, { button: 0, clientX: 40, clientY: 50 });
  let allowed: boolean | undefined;
  let leaving: Promise<void> | undefined;
  await act(async () => {
    leaving = guardRef.current.run().then((value) => {
      allowed = value;
    });
    await Promise.resolve();
  });
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(allowed).toBeUndefined();
  await act(async () => {
    fireEvent.pointerUp(canvas, { button: 0, clientX: 40, clientY: 50 });
    await leaving;
  });
  expect(allowed).toBe(true);
  expect(save.mock.calls[0]?.[0].units).toHaveLength(1);
  expect(screen.queryByRole("dialog")).toBeNull();
});
