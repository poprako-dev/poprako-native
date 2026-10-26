import { useRef } from "react";
import type { ReactElement } from "react";
import { useStore } from "zustand";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import type { PageEditor } from "@/bridge/generated/bindings";
import { createEditorSession } from "./editor-session";
import type { EditorApi, EditorSession } from "./editor-session";
import { editorFixture } from "./editor-test-fixture";
import { UnitTextarea } from "./UnitTextarea";

let session: EditorSession;
const save = vi.fn<EditorApi["save"]>();

function Harness(): ReactElement {
  const state = useStore(session.store);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  return (
    <UnitTextarea
      inputRef={inputRef}
      aria-label="翻译"
      session={session}
      value={state.draft[0]?.translated_text ?? ""}
      onTextChange={(value) => {
        session.edit("a", { translated_text: value });
      }}
    />
  );
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe = vi.fn();
      disconnect = vi.fn();
    },
  );
  save.mockReset();
  save.mockImplementation((snapshot) =>
    Promise.resolve({
      ...editorFixture(),
      page: {
        ...editorFixture().page,
        unit_revision: snapshot.expected_revision + 1,
      },
      units: snapshot.units.map((unit, index) => ({
        ...unit,
        index,
        page_id: "page",
        created_at: 1,
        updated_at: 2,
      })),
    }),
  );
  session = createEditorSession(editorFixture(), "translation", { save });
});
afterEach(() => {
  cleanup();
  session.dispose();
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

test("IME previews neither save nor enter history and final input records once", async () => {
  render(<Harness />);
  const input = screen.getByRole("textbox");
  fireEvent.compositionStart(input);
  fireEvent.change(input, { target: { value: "pin" } });
  fireEvent.change(input, { target: { value: "拼" } });
  expect(session.store.getState().draft[0]?.translated_text).toBe("翻译 a");
  expect(session.store.getState().canUndo).toBe(false);
  await act(() => vi.advanceTimersByTimeAsync(20_000));
  expect(save).not.toHaveBeenCalled();
  const saving = session.save();
  fireEvent.compositionEnd(input, { data: "拼" });
  fireEvent.change(input, { target: { value: "拼" } });
  expect(session.store.getState().draft[0]?.translated_text).toBe("拼");
  await act(async () => {
    expect(await saving).toBe(true);
  });
  await act(() => vi.advanceTimersByTimeAsync(15_000));
  expect(save).toHaveBeenCalledTimes(1);
  act(() => {
    session.undo();
  });
  expect(session.store.getState().draft[0]?.translated_text).toBe("翻译 a");
  expect(session.store.getState().canUndo).toBe(false);
});

test("an older save receipt leaves the composition preview and selection intact", async () => {
  let finish: (page: PageEditor) => void = () => {
    throw new Error("save not started");
  };
  save.mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        finish = resolve;
      }),
  );
  session.edit("a", { translated_text: "before" });
  const saving = session.save();
  await Promise.resolve();
  render(<Harness />);
  const input = screen.getByRole("textbox");
  if (!(input instanceof HTMLTextAreaElement))
    throw new Error("missing textarea");
  input.focus();
  fireEvent.compositionStart(input);
  fireEvent.change(input, { target: { value: "before预览" } });
  input.setSelectionRange(2, 4);
  await act(async () => {
    finish({
      ...editorFixture(),
      units: editorFixture().units.map((unit) => ({
        ...unit,
        translated_text: "before",
      })),
    });
    await saving;
  });
  expect(input.value).toBe("before预览");
  expect([input.selectionStart, input.selectionEnd]).toEqual([2, 4]);
  fireEvent.compositionEnd(input);
  act(() => {
    session.undo();
  });
  expect(input.value).toBe("before");
});

test("cancelled composition and unmount release the save suspension", () => {
  const view = render(<Harness />);
  const input = screen.getByRole("textbox");
  fireEvent.compositionStart(input);
  fireEvent.compositionEnd(input);
  expect(session.store.getState().canUndo).toBe(false);
  fireEvent.compositionStart(input);
  fireEvent.change(input, { target: { value: "preview" } });
  view.unmount();
  expect(session.isComposing()).toBe(false);
  expect(session.store.getState().draft[0]?.translated_text).toBe("翻译 a");
});
