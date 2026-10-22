import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useMemo } from "react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { commands } from "@/bridge";
import type { TerminologyContext } from "@/bridge/generated/bindings";
import { TerminologyLookup } from "./TerminologyLookup";
import { setupTerminology } from "./terminology-test-fixture";
import { useTermbaseExport } from "./use-termbase-export";
import { useTermbaseAppend } from "./use-termbase-append";
import { useTerminology } from "./use-terminology";

function Lookup(props: {
  chapterId: string;
  locked: boolean;
  onInteraction: (active: boolean) => void;
}): React.ReactElement {
  const access = useMemo<TerminologyContext>(
    () => ({ kind: "chapter", chapter_id: props.chapterId }),
    [props.chapterId],
  );
  const exports = useTermbaseExport(access);
  const state = useTerminology(access);
  const appends = useTermbaseAppend(state.access, state.appendedTerms);
  return (
    <TerminologyLookup
      state={state}
      locked={props.locked}
      onInteraction={props.onInteraction}
      exports={exports}
      appends={appends}
    />
  );
}

vi.mock("@tanstack/react-router", () => ({ useBlocker: vi.fn() }));
beforeEach(setupTerminology);
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

it("looks up global terms and copies without saving or exposing global mutation controls", async () => {
  const user = userEvent.setup();
  const save = vi.spyOn(commands, "savePageUnits");
  const interaction = vi.fn();
  render(
    <Lookup chapterId="chapter" locked={false} onInteraction={interaction} />,
  );
  await user.click(screen.getByRole("button", { name: "选择术语库" }));
  await user.click(await screen.findByRole("button", { name: /奇幻世界/ }));
  await user.click(screen.getByRole("combobox", { name: "搜索术语原文" }));
  await user.click(
    await screen.findByRole("button", { name: "复制 団長 的首选译名" }),
  );
  expect(await screen.findByText("已复制首选译名")).toBeTruthy();
  expect(commands.copyTermTarget).toHaveBeenCalledWith(
    { kind: "chapter", chapter_id: "chapter" },
    "term",
  );
  expect(screen.queryByRole("button", { name: "新建术语" })).toBeNull();
  fireEvent.keyDown(screen.getByText("団長"), { key: "F2" });
  expect(screen.queryByRole("dialog", { name: "编辑术语" })).toBeNull();
  expect(save).not.toHaveBeenCalled();
  expect(interaction).toHaveBeenLastCalledWith(true);
});

it("allows local term editing, isolates copy from long press, and closes panels on Escape", async () => {
  const user = userEvent.setup();
  const interaction = vi.fn();
  render(
    <Lookup chapterId="chapter" locked={false} onInteraction={interaction} />,
  );
  await user.click(screen.getByRole("button", { name: "选择术语库" }));
  await user.click(await screen.findByRole("button", { name: /本作人名/ }));
  await user.click(screen.getByRole("combobox", { name: "搜索术语原文" }));
  expect(await screen.findByRole("button", { name: "新建术语" })).toBeTruthy();
  const row = screen.getByLabelText("団長，按 F2 编辑");
  fireEvent.keyDown(row, { key: "F2" });
  expect(screen.getByRole("dialog", { name: "编辑术语" })).toBeTruthy();
  await user.click(screen.getByRole("button", { name: "关闭弹窗" }));
  await user.click(
    screen.getByRole("button", { name: "复制 団長 的首选译名" }),
  );
  expect(screen.queryByRole("dialog", { name: "编辑术语" })).toBeNull();
  await user.keyboard("{Escape}");
  await waitFor(() => {
    expect(screen.queryByRole("dialog", { name: "查询术语" })).toBeNull();
  });
  expect(interaction).toHaveBeenLastCalledWith(false);
});
