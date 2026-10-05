import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { NativeCommandError } from "@/bridge";
import type * as Bridge from "@/bridge";
import type { ApplicationPreference } from "@/bridge/generated/bindings";
import { PreferenceDialog } from "@/route/business/component/PreferenceDialog";

const mock = vi.hoisted(() => ({
  save: vi.fn<(preference: ApplicationPreference) => Promise<void>>(),
  defaults: vi.fn<() => Promise<ApplicationPreference>>(),
}));
const initial: ApplicationPreference = {
  shortcut: [],
  special_character: [],
  relocation_enabled: false,
  marker_opacity: 1,
};
vi.mock("@/route/business/preference-context", () => ({
  usePreference: () => ({ preference: initial, save: mock.save }),
}));
vi.mock("@/bridge", async (original) => {
  const actual = await original<typeof Bridge>();
  return {
    ...actual,
    commands: { ...actual.commands, getDefaultPreference: mock.defaults },
  };
});
afterEach(cleanup);
beforeEach(() => {
  mock.save.mockReset();
  mock.defaults.mockReset();
});

describe("preference saving", () => {
  it("locks an uncertain payload and retries that exact payload", async () => {
    const user = userEvent.setup();
    const close = vi.fn<() => void>();
    mock.save
      .mockRejectedValueOnce(
        new NativeCommandError({
          code: "commit_unknown",
          message: "请核实保存结果",
          recovery: "wait_for_confirmation",
        }),
      )
      .mockResolvedValueOnce(undefined);
    render(<PreferenceDialog onClose={close} />);
    await user.click(
      screen.getByRole("checkbox", { name: "选中单元时自动定位图片" }),
    );
    await user.click(screen.getByRole("button", { name: "保存设置" }));
    await screen.findByRole("button", { name: "核实保存结果" });
    expect(
      screen
        .getByRole("checkbox", { name: "选中单元时自动定位图片" })
        .hasAttribute("disabled"),
    ).toBe(true);
    expect(
      screen.getByRole("button", { name: "取消" }).hasAttribute("disabled"),
    ).toBe(true);
    expect(close).not.toHaveBeenCalled();
    await user.click(screen.getByRole("button", { name: "核实保存结果" }));
    await waitFor(() => {
      expect(close).toHaveBeenCalledOnce();
    });
    expect(mock.save).toHaveBeenNthCalledWith(1, {
      ...initial,
      relocation_enabled: true,
    });
    expect(mock.save).toHaveBeenNthCalledWith(2, {
      ...initial,
      relocation_enabled: true,
    });
  });

  it("loads Rust defaults into the draft without silently saving", async () => {
    const user = userEvent.setup();
    mock.defaults.mockResolvedValue({ ...initial, relocation_enabled: true });
    render(<PreferenceDialog onClose={() => undefined} />);
    await user.click(screen.getByRole("button", { name: "恢复默认" }));
    await waitFor(() => {
      expect(
        screen
          .getByRole("checkbox", { name: "选中单元时自动定位图片" })
          .getAttribute("disabled"),
      ).toBeNull();
    });
    expect(
      screen.getByRole<HTMLInputElement>("checkbox", {
        name: "选中单元时自动定位图片",
      }).checked,
    ).toBe(true);
    expect(mock.save).not.toHaveBeenCalled();
  });
});
