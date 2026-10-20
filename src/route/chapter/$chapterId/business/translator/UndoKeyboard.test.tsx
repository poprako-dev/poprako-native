import { cleanup, fireEvent, render, renderHook } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import { createEditorSession } from "./editor-session";
import { editorFixture } from "./editor-test-fixture";
import { detail, preference } from "./editor-shortcut-test-fixture";
import { useEditorShortcuts } from "./use-editor-shortcuts";

afterEach(cleanup);

test.each([false, true])(
  "undo key works inside unit input with mac modifier %s",
  (macos) => {
    const session = createEditorSession(editorFixture(), "translation", {
      save: vi.fn(),
    });
    const shortcuts = preference.shortcut.map((item) =>
      item.action === "undo"
        ? {
            ...item,
            binding: {
              kind: "chord" as const,
              key: "Period",
              control: !macos,
              meta: macos,
              alt: false,
              shift: false,
              scope: "editor" as const,
            },
          }
        : item,
    );
    const options = {
      session,
      detail,
      preference: { ...preference, shortcut: shortcuts },
      disabled: false,
      onPage: vi.fn(),
      onRelocation: vi.fn(),
      onPreview: vi.fn(),
      onSymbol: vi.fn(),
    };
    const hook = renderHook(
      (props) => {
        useEditorShortcuts(props);
      },
      { initialProps: options },
    );
    const view = render(
      <>
        <div data-unit-id="a">
          <textarea aria-label="unit" />
        </div>
        <input aria-label="other" />
      </>,
    );
    const input = view.getByLabelText("unit");
    const event = { key: ".", code: "Period", ctrlKey: !macos, metaKey: macos };
    session.edit("a", { translated_text: "A" });
    session.edit("a", { translated_text: "AB" });
    for (const extra of [
      { repeat: true },
      { isComposing: true },
      { keyCode: 229 },
    ])
      fireEvent.keyDown(input, { ...event, ...extra });
    fireEvent.keyDown(view.getByLabelText("other"), event);
    const dialog = document.createElement("div");
    dialog.setAttribute("role", "dialog");
    document.body.append(dialog);
    fireEvent.keyDown(input, event);
    dialog.remove();
    hook.rerender({ ...options, disabled: true });
    fireEvent.keyDown(input, event);
    hook.rerender(options);
    expect(session.store.getState().draft[0]?.translated_text).toBe("AB");
    expect(fireEvent.keyDown(input, { ...event, key: "z", code: "KeyZ" })).toBe(
      true,
    );
    expect(fireEvent.keyDown(input, event)).toBe(false);
    expect(session.store.getState().draft[0]?.translated_text).toBe("A");
    session.setMode("readonly");
    fireEvent.keyDown(input, event);
    expect(session.store.getState().draft[0]?.translated_text).toBe("A");
    session.setMode("translation");
    hook.rerender({
      ...options,
      preference: {
        ...preference,
        shortcut: shortcuts.map((item) =>
          item.action === "undo"
            ? { ...item, binding: { kind: "unbound" } }
            : item,
        ),
      },
    });
    fireEvent.keyDown(input, event);
    expect(session.store.getState().draft[0]?.translated_text).toBe("A");
    session.dispose();
  },
);
