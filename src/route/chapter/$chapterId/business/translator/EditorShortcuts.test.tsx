import { useState } from "react";
import type { ReactElement } from "react";
import { useStore } from "zustand";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Mock } from "vitest";
import { createEditorSession } from "./editor-session";
import type { EditorApi, EditorSession } from "./editor-session";
import { preference, detail } from "./editor-shortcut-test-fixture";
import { editorFixture } from "./editor-test-fixture";
import { useEditorShortcuts } from "./use-editor-shortcuts";
import { UnitItem } from "./UnitItem";
import type { SymbolRequest } from "./SpecialCharsBar";

const fixture = editorFixture();
const onPage = vi.fn<(id: string) => void>();
const onRelocation = vi.fn<() => void>();
let session: EditorSession;
let save: Mock<EditorApi["save"]>;

function Harness({ disabled = false }: { disabled?: boolean }): ReactElement {
  const state = useStore(session.store);
  const [preview, setPreview] = useState(true);
  const [request, setRequest] = useState<SymbolRequest | null>(null);
  useEditorShortcuts({
    session,
    detail,
    preference,
    disabled,
    onPage,
    onRelocation,
    onPreview: () => {
      setPreview((value) => !value);
    },
    onSymbol: (text) => {
      setRequest({
        id: crypto.randomUUID(),
        unitId: session.store.getState().selected,
        text,
      });
    },
  });
  return (
    <>
      <div role="application" tabIndex={0} aria-label="画布" />
      <input aria-label="搜索" />
      <output aria-label="预览">{preview ? "显示" : "淡化"}</output>
      {state.draft.map((unit, index) => (
        <UnitItem
          key={unit.id}
          unit={unit}
          index={index}
          mode={state.mode}
          locked={state.locked}
          selected={state.selected === unit.id}
          session={session}
          characters={preference.special_character}
          characterBar={null}
          symbolRequest={request}
          pending={false}
          dragging={false}
          dimmed={false}
          onIndexPointerDown={() => undefined}
          onDelete={() => undefined}
          onReorder={() => undefined}
        />
      ))}
    </>
  );
}

function key(
  target: Element | Window,
  value: string,
  modifiers: KeyboardEventInit = {},
): boolean {
  return fireEvent.keyDown(target, {
    key: value,
    code: /^[a-z]$/iu.test(value) ? `Key${value.toUpperCase()}` : value,
    ...modifiers,
  });
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
  HTMLElement.prototype.scrollIntoView = vi.fn();
  onPage.mockReset();
  onRelocation.mockReset();
  save = vi.fn<EditorApi["save"]>().mockResolvedValue(fixture);
  session = createEditorSession(fixture, "translation", { save });
});
afterEach(() => {
  cleanup();
  session.dispose();
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe("Web keyboard behavior", () => {
  it("cycles every marker with Tab and Shift+Tab while focusing its textarea", () => {
    render(<Harness />);
    const canvas = screen.getByRole("application");
    expect(key(canvas, "Tab")).toBe(false);
    expect(session.store.getState().selected).toBe("a");
    expect(document.activeElement).toBe(
      screen.getByRole("textbox", { name: "翻译 1" }),
    );
    for (const id of ["b", "c", "a"]) {
      key(document.activeElement ?? canvas, "Tab");
      expect(session.store.getState().selected).toBe(id);
    }
    key(document.activeElement ?? canvas, "Tab", { shiftKey: true });
    expect(session.store.getState().selected).toBe("c");
    key(document.activeElement ?? canvas, "Escape");
    expect(session.store.getState().selected).toBe("");
    key(canvas, "Tab", { shiftKey: true });
    expect(session.store.getState().selected).toBe("c");
  });

  it("switches modes, relocation and the actual preview state inside the unit textarea", () => {
    session.select("a");
    render(<Harness />);
    const input = screen.getByRole("textbox", { name: "翻译 1" });
    key(input, "l", { ctrlKey: true });
    expect(onRelocation).toHaveBeenCalledOnce();
    key(input, "x", { ctrlKey: true });
    expect(screen.getByLabelText("预览").textContent).toBe("淡化");
    expect(preference.marker_opacity).toBe(1);
    key(input, "x", { ctrlKey: true });
    expect(screen.getByLabelText("预览").textContent).toBe("显示");
    for (const mode of ["proofreading", "readonly", "translation"]) {
      key(screen.getByRole("application"), "m", { ctrlKey: true });
      expect(session.store.getState().mode).toBe(mode);
    }
  });

  it("focuses the next input at its end before a rapid symbol key", () => {
    render(<Harness />);
    key(screen.getByRole("application"), "Tab");
    key(document.activeElement ?? window, "q", { ctrlKey: true });
    expect(session.store.getState().draft[0]?.translated_text).toBe("翻译 a♪");
  });

  it("uses Ctrl on macOS too and saves edits from the text input", async () => {
    session.select("a");
    render(<Harness />);
    const input = screen.getByRole("textbox", { name: "翻译 1" });
    fireEvent.change(input, { target: { value: "已修改" } });
    expect(key(input, "s", { metaKey: true })).toBe(true);
    expect(save).not.toHaveBeenCalled();
    await act(() => {
      key(input, "s", { ctrlKey: true });
      return Promise.resolve();
    });
    expect(save).toHaveBeenCalledOnce();
    expect(save.mock.calls[0]?.[0]).toMatchObject({
      units: [
        expect.objectContaining({ translated_text: "已修改" }),
        expect.anything(),
        expect.anything(),
      ],
    });
  });

  it("uses Ctrl+U/D from the text input and stops at page boundaries", () => {
    session.select("a");
    render(<Harness />);
    const input = screen.getByRole("textbox", { name: "翻译 1" });
    key(input, "u", { ctrlKey: true });
    expect(onPage).not.toHaveBeenCalled();
    key(input, "d", { ctrlKey: true });
    expect(onPage).toHaveBeenLastCalledWith("second");
    act(() => {
      session.store.setState({
        baseline: { ...fixture, page: { ...fixture.page, id: "third" } },
      });
    });
    onPage.mockClear();
    key(input, "d", { ctrlKey: true });
    expect(onPage).not.toHaveBeenCalled();
    key(input, "u", { ctrlKey: true });
    expect(onPage).toHaveBeenLastCalledWith("second");
  });

  it("inserts recent and all three favorites at the selection, including from canvas focus", () => {
    session.select("a");
    render(<Harness />);
    const input = screen.getByRole<HTMLTextAreaElement>("textbox", {
      name: "翻译 1",
    });
    input.setSelectionRange(0, input.value.length);
    key(input, "q", { ctrlKey: true });
    expect(input.value).toBe("♪");
    act(() => {
      vi.advanceTimersByTime(20);
    });
    for (const [digit, symbol] of [
      ["1", "♪"],
      ["2", "★"],
      ["3", "♡"],
    ]) {
      input.setSelectionRange(0, input.value.length);
      key(screen.getByRole("application"), "Dead", {
        code: `Digit${String(digit)}`,
        altKey: true,
      });
      expect(input.value).toBe(symbol);
      act(() => {
        vi.advanceTimersByTime(20);
      });
    }
    input.setSelectionRange(1, 1);
    key(input, "q", { ctrlKey: true });
    expect(input.value).toBe("♡♡");
    expect(session.store.getState().recentSymbol).toBe("♡");
  });

  it("inserts symbols into proofreading without changing confirmation or translation", () => {
    session.setMode("proofreading");
    session.select("a");
    session.edit("a", { proofread_text: "校对内容", is_proofread: true });
    render(<Harness />);
    const input = screen.getByRole<HTMLTextAreaElement>("textbox", {
      name: "校对 1",
    });
    input.setSelectionRange(2, 4);
    key(input, "2", { code: "Digit2", altKey: true });
    expect(session.store.getState().draft[0]).toMatchObject({
      proofread_text: "校对★",
      translated_text: "翻译 a",
      is_proofread: true,
    });
  });

  it("keeps the caret for consecutive symbol keys before the next animation frame", () => {
    session.select("a");
    session.edit("a", { translated_text: "甲乙丙" });
    render(<Harness />);
    const input = screen.getByRole<HTMLTextAreaElement>("textbox", {
      name: "翻译 1",
    });
    input.setSelectionRange(1, 1);
    key(input, "1", { code: "Digit1", altKey: true });
    key(input, "2", { code: "Digit2", altKey: true });
    key(input, "q", { ctrlKey: true });
    expect(input.value).toBe("甲♪★★乙丙");
    expect(input.selectionStart).toBe(4);
    expect(input.selectionEnd).toBe(4);
  });

  it("accepts WKWebView Option digits reported as 229 while protecting active composition", () => {
    session.select("a");
    render(<Harness />);
    const input = screen.getByRole<HTMLTextAreaElement>("textbox", {
      name: "翻译 1",
    });
    input.setSelectionRange(0, input.value.length);
    key(input, "3", { code: "Digit3", keyCode: 229, altKey: true });
    expect(input.value).toBe("♡");
    fireEvent.compositionStart(input);
    expect(
      key(input, "2", { code: "Digit2", keyCode: 229, altKey: true }),
    ).toBe(true);
    expect(input.value).toBe("♡");
    fireEvent.compositionEnd(input);
    key(input, "2", { code: "Digit2", keyCode: 229, altKey: true });
    expect(input.value).toBe("♡★");
  });

  it("leaves search input and composition keys alone, including IME confirmation and AltGraph", () => {
    render(<Harness />);
    expect(key(screen.getByRole("textbox", { name: "搜索" }), "Tab")).toBe(
      true,
    );
    const canvas = screen.getByRole("application");
    for (const options of [{ isComposing: true }, { keyCode: 229 }]) {
      expect(key(canvas, "Tab", options)).toBe(true);
    }
    const event = new KeyboardEvent("keydown", {
      key: "Tab",
      bubbles: true,
      cancelable: true,
    });
    Object.defineProperty(event, "getModifierState", {
      value: (modifier: string) => modifier === "AltGraph",
    });
    canvas.dispatchEvent(event);
    expect(session.store.getState().selected).toBe("");
    expect(onPage).not.toHaveBeenCalled();
  });

  it("does not consume keys handled by a child or while a dialog is open", () => {
    render(<Harness />);
    const canvas = screen.getByRole("application");
    const event = new KeyboardEvent("keydown", {
      key: "Tab",
      bubbles: true,
      cancelable: true,
    });
    event.preventDefault();
    canvas.dispatchEvent(event);
    expect(session.store.getState().selected).toBe("");
    const dialog = document.createElement("div");
    dialog.setAttribute("role", "dialog");
    document.body.append(dialog);
    expect(key(canvas, "Tab")).toBe(true);
    expect(session.store.getState().selected).toBe("");
    dialog.remove();
  });

  it("allows readonly navigation and dimming while blocking editing shortcuts and stats overlay", () => {
    session.setMode("readonly");
    render(<Harness />);
    const canvas = screen.getByRole("application");
    key(canvas, "Tab");
    expect(session.store.getState().selected).toBe("a");
    for (const value of ["l", "q", "s"])
      expect(key(canvas, value, { ctrlKey: true })).toBe(true);
    expect(onRelocation).not.toHaveBeenCalled();
    key(canvas, "x", { ctrlKey: true });
    expect(screen.getByLabelText("预览").textContent).toBe("淡化");
    const stats = document.createElement("div");
    stats.setAttribute("aria-label", "页面统计详情");
    document.body.append(stats);
    expect(key(canvas, "Tab")).toBe(true);
    expect(session.store.getState().selected).toBe("a");
    stats.remove();
  });

  it("suspends shortcuts while leaving, locked or awaiting commit verification", () => {
    const view = render(<Harness disabled={true} />);
    const canvas = screen.getByRole("application");
    key(canvas, "Tab");
    expect(session.store.getState().selected).toBe("");
    view.rerender(<Harness />);
    for (const state of [
      { locked: true, uncertain: false },
      { locked: false, uncertain: true },
    ]) {
      act(() => {
        session.store.setState(state);
      });
      key(canvas, "Tab");
      expect(session.store.getState().selected).toBe("");
    }
  });
});
