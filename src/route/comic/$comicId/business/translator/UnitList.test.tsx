import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useStore } from "zustand";
import type { ReactElement } from "react";
import type { EditorMode } from "@/bridge/generated/bindings";
import { createEditorSession, unitDraft } from "./editor-session";
import type { EditorSession } from "./editor-session";
import { editorFixture } from "./editor-test-fixture";
import { UnitList } from "./UnitList";

class TestPointerEvent extends MouseEvent {
  readonly pointerId: number;
  readonly pointerType: string;
  readonly isPrimary: boolean;
  constructor(type: string, options: PointerEventInit = {}) {
    super(type, options);
    this.pointerId = options.pointerId ?? 1;
    this.pointerType = options.pointerType ?? "mouse";
    this.isPrimary = options.isPrimary ?? true;
  }
}

function Harness({
  session,
  flagged,
}: {
  session: EditorSession;
  flagged: boolean;
}): ReactElement {
  const state = useStore(session.store);
  return (
    <UnitList
      state={state}
      session={session}
      flagged={flagged}
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
  );
}

function setup(
  mode: EditorMode = "translation",
  flagged = false,
): EditorSession {
  const session = createEditorSession(editorFixture(), mode, {
    save: () => Promise.reject(new Error("unexpected save")),
  });
  render(<Harness session={session} flagged={flagged} />);
  return session;
}

function drag(): void {
  fireEvent.pointerDown(screen.getByRole("button", { name: /^单元 1，/ }), {
    pointerId: 1,
    clientX: 10,
    clientY: 10,
    button: 0,
  });
  fireEvent.pointerMove(window, {
    pointerId: 1,
    clientX: 10,
    clientY: 110,
  });
}

describe("unit list gestures and page confirmation", () => {
  beforeEach(() => {
    vi.stubGlobal(
      "ResizeObserver",
      class {
        observe = vi.fn();
        disconnect = vi.fn();
      },
    );
    vi.useFakeTimers();
    vi.stubGlobal("PointerEvent", TestPointerEvent);
    HTMLElement.prototype.scrollIntoView = vi.fn();
    HTMLElement.prototype.setPointerCapture = vi.fn();
    HTMLElement.prototype.hasPointerCapture = vi.fn(() => false);
    HTMLElement.prototype.releasePointerCapture = vi.fn();
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(
      function (this: HTMLElement) {
        const siblings = this.parentElement
          ? [...this.parentElement.children]
          : [];
        const top = this.dataset["unitId"] ? siblings.indexOf(this) * 40 : 0;
        return new DOMRect(0, top, 300, this.dataset["unitId"] ? 40 : 160);
      },
    );
  });
  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });

  it("commits a drag without toggling category or losing content", () => {
    const session = setup();
    drag();
    expect(session.store.getState().draft.map((unit) => unit.id)).toEqual([
      "a",
      "b",
      "c",
    ]);
    fireEvent.pointerUp(window, { pointerId: 1, clientY: 110 });
    expect(session.store.getState().draft.map((unit) => unit.id)).toEqual([
      "b",
      "c",
      "a",
    ]);
    expect(session.store.getState().draft[2]).toMatchObject({
      is_bubble: true,
      translated_text: "翻译 a",
    });
    session.dispose();
  });

  it.each(["Escape", "pointercancel", "lostpointercapture"])(
    "cancels %s without committing the preview",
    (cancel) => {
      const session = setup();
      drag();
      if (cancel === "Escape") fireEvent.keyDown(window, { key: "Escape" });
      if (cancel !== "Escape")
        fireEvent(window, new TestPointerEvent(cancel, { pointerId: 1 }));
      fireEvent.pointerUp(window, { pointerId: 1, clientY: 110 });
      expect(session.store.getState().draft).toMatchObject(
        editorFixture().units.map(unitDraft),
      );
      session.dispose();
    },
  );

  it("treats an index tap as one category toggle", () => {
    const session = setup();
    const index = screen.getByRole("button", { name: /^单元 1，/ });
    fireEvent.pointerDown(index, {
      pointerId: 1,
      clientX: 10,
      clientY: 10,
      button: 0,
    });
    fireEvent.pointerUp(window, { pointerId: 1, clientX: 11, clientY: 11 });
    fireEvent.click(index, { detail: 1 });
    expect(session.store.getState().draft[0]?.is_bubble).toBe(false);
    session.dispose();
  });

  it("reorders only visible slots, then confirms the full filtered page", () => {
    const session = setup("proofreading", true);
    drag();
    fireEvent.pointerUp(window, { pointerId: 1, clientY: 110 });
    expect(session.store.getState().draft.map((unit) => unit.id)).toEqual([
      "c",
      "b",
      "a",
    ]);
    fireEvent.click(screen.getByRole("button", { name: "确认整页校对" }));
    expect(
      session.store.getState().draft.every((unit) => unit.is_proofread),
    ).toBe(true);
    expect(
      session.store
        .getState()
        .draft.every((unit) => unit.proofread_text === ""),
    ).toBe(true);
    fireEvent.click(screen.getByRole("button", { name: "取消整页确认" }));
    expect(
      session.store.getState().draft.some((unit) => unit.is_proofread),
    ).toBe(false);
    session.dispose();
  });

  it("blocks dragging, category edits and confirmation in readonly", () => {
    const session = setup("readonly");
    drag();
    fireEvent.pointerUp(window, { pointerId: 1, clientY: 110 });
    expect(session.store.getState().draft).toEqual(
      editorFixture().units.map(unitDraft),
    );
    expect(screen.queryByRole("button", { name: "确认整页校对" })).toBeNull();
    session.dispose();
  });

  it("moves across visible slots with the keyboard while preserving hidden units", () => {
    const session = setup("translation", true);
    fireEvent.keyDown(screen.getByRole("button", { name: /^单元 3，/ }), {
      key: "ArrowUp",
      ctrlKey: true,
    });
    expect(session.store.getState().draft.map((unit) => unit.id)).toEqual([
      "c",
      "b",
      "a",
    ]);
    session.dispose();
  });
});
