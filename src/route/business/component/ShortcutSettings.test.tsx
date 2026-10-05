import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Shortcut } from "@/bridge/generated/bindings";
import { preference } from "@/route/comic/$comicId/business/translator/editor-shortcut-test-fixture";
import { ShortcutSettings } from "./ShortcutSettings";

afterEach(cleanup);

function key(
  target: Element,
  value: string,
  modifiers: KeyboardEventInit = {},
): boolean {
  return fireEvent.keyDown(target, { key: value, code: value, ...modifiers });
}

describe("shortcut recording", () => {
  it("records normal WKWebView Option digits even when keyCode is 229", () => {
    const change = vi.fn<(value: Shortcut[]) => void>();
    render(
      <ShortcutSettings
        value={preference.shortcut}
        disabled={false}
        onChange={change}
      />,
    );
    const button = screen.getByRole("button", {
      name: "设置输入优选符号#3快捷键",
    });
    fireEvent.click(button);
    key(button, "3", { code: "Digit3", keyCode: 229, altKey: true });
    expect(
      change.mock.calls[0]?.[0].find(
        (item) => item.action === "insert_favorite_three",
      )?.binding,
    ).toEqual({
      kind: "chord",
      key: "Digit3",
      control: false,
      meta: false,
      alt: true,
      shift: false,
      scope: "editor",
    });
  });
  it("records Tab and Shift+Tab as configurable bindings", () => {
    const change = vi.fn<(value: Shortcut[]) => void>();
    render(
      <ShortcutSettings
        value={preference.shortcut.map((item) =>
          item.action === "previous_unit"
            ? { ...item, binding: { kind: "unbound" } }
            : item,
        )}
        disabled={false}
        onChange={change}
      />,
    );
    const button = screen.getByRole("button", { name: "设置下一个标记快捷键" });
    fireEvent.click(button);
    key(button, "Tab", { shiftKey: true });
    expect(
      change.mock.calls[0]?.[0].find((item) => item.action === "next_unit")
        ?.binding,
    ).toEqual({
      kind: "chord",
      key: "Tab",
      control: false,
      meta: false,
      alt: false,
      shift: true,
      scope: "editor",
    });
  });

  it("keeps the original binding when a recorded shortcut conflicts", () => {
    const change = vi.fn<(value: Shortcut[]) => void>();
    render(
      <ShortcutSettings
        value={preference.shortcut}
        disabled={false}
        onChange={change}
      />,
    );
    const button = screen.getByRole("button", { name: "设置下一个标记快捷键" });
    fireEvent.click(button);
    key(button, "Tab", { shiftKey: true });
    expect(change).not.toHaveBeenCalled();
    expect(screen.getByRole("alert").textContent).toBe(
      "快捷键冲突，已保留原有设置。",
    );
  });
});
