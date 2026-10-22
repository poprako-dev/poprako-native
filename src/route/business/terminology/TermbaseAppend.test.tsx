import "@testing-library/jest-dom/vitest";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import type { ReactElement } from "react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { commands } from "@/bridge";
import type { TerminologyContext } from "@/bridge/generated/bindings";
import { registerExitGuard } from "@/bridge/exit";
import { TerminologyPage } from "@/route/terminology/business/TerminologyPage";
import { TerminologyLookup } from "./TerminologyLookup";
import { TermbaseAppendNotice } from "./TermbaseAppendNotice";
import { useTermbaseAppend } from "./use-termbase-append";
import { useTermbaseExport } from "./use-termbase-export";
import { useTerminology } from "./use-terminology";
import {
  globalBase,
  localBase,
  setupTerminology,
} from "./terminology-test-fixture";

const block = vi.hoisted(() =>
  vi.fn<(options: { shouldBlockFn: () => boolean }) => void>(),
);

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useBlocker: block,
}));
vi.mock("@/bridge/exit", () => ({ registerExitGuard: vi.fn(() => vi.fn()) }));

const document = {
  terms: [{ source: "Elf", targets: ["精灵"], comment: "备注" }],
};
const outcome = {
  status: "ok",
  data: { added: 1, updated: 2, unchanged: 3 },
} as const;
const lookupAccess: TerminologyContext = {
  kind: "chapter",
  chapter_id: "chapter",
};

function Lookup({ readonly = false }: { readonly?: boolean }): ReactElement {
  const state = useTerminology(lookupAccess);
  const exports = useTermbaseExport(state.access);
  const appends = useTermbaseAppend(state.access, state.appendedTerms);
  return (
    <>
      {!readonly && (
        <TerminologyLookup
          state={state}
          locked={appends.locked}
          exports={exports}
          appends={appends}
          onInteraction={vi.fn()}
        />
      )}
      <TermbaseAppendNotice appends={appends} />
    </>
  );
}

async function appendButton(name: string): Promise<HTMLElement> {
  const list = await screen.findByRole("list", { name: "术语库列表" });
  const row = within(list)
    .getAllByRole("listitem")
    .find((item) => item.textContent.includes(name));
  if (!row) throw new Error("Missing library");
  return within(row).getByRole("button", { name: "追加术语" });
}

beforeEach(() => {
  setupTerminology();
  vi.spyOn(commands, "chooseTermbaseAppend").mockResolvedValue({
    status: "ok",
    data: document,
  });
  vi.spyOn(commands, "appendTermbase").mockResolvedValue(outcome);
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

it("appends from a global row without selecting it or saving editor content", async () => {
  vi.mocked(commands.listTermbaseInfos).mockResolvedValue({
    status: "ok",
    data: [globalBase],
  });
  const save = vi.spyOn(commands, "savePageUnits");
  const user = userEvent.setup();
  render(<TerminologyPage />);
  const button = await appendButton(globalBase.termbase.name);
  button.focus();
  await user.keyboard("{Enter}");
  expect(
    await screen.findByText("已追加：新增 1 条，更新 2 条，未变更 3 条"),
  ).toBeVisible();
  expect(vi.spyOn(commands, "appendTermbase")).toHaveBeenCalledExactlyOnceWith(
    { kind: "global" },
    "global",
    document,
  );
  expect(screen.getByRole("textbox", { name: "搜索术语库名称" })).toBeVisible();
  expect(commands.listTermInfos).not.toHaveBeenCalled();
  expect(save).not.toHaveBeenCalled();
});

it("preserves both search fields and the selected library while refreshing its count and terms", async () => {
  const user = userEvent.setup();
  render(<TerminologyPage />);
  await user.type(
    screen.getByRole("textbox", { name: "搜索术语库名称" }),
    "奇幻",
  );
  await user.click(await screen.findByRole("button", { name: /奇幻世界/ }));
  await user.type(
    await screen.findByRole("textbox", { name: "搜索术语原文" }),
    "団",
  );
  await user.click(screen.getByRole("button", { name: "返回术语库" }));
  await user.click(await appendButton(globalBase.termbase.name));
  await screen.findByText("已追加：新增 1 条，更新 2 条，未变更 3 条");
  expect(screen.getByRole("textbox", { name: "搜索术语库名称" })).toHaveValue(
    "奇幻",
  );
  await user.click(await screen.findByRole("button", { name: /奇幻世界/ }));
  expect(screen.getByRole("textbox", { name: "搜索术语原文" })).toHaveValue(
    "団",
  );
  await waitFor(() => {
    expect(commands.getTermbase).toHaveBeenCalledTimes(2);
  });
  await waitFor(() => {
    expect(commands.listTermInfos).toHaveBeenLastCalledWith(
      { kind: "global" },
      "global",
      expect.objectContaining({ text: "団" }),
    );
  });
});

it("cancels silently and reports invalid files without submitting a write", async () => {
  vi.mocked(commands.chooseTermbaseAppend)
    .mockResolvedValueOnce({ status: "ok", data: null })
    .mockResolvedValueOnce({
      status: "error",
      error: {
        code: "termbase_import",
        recovery: "correct_input",
        message: "文件无效",
      },
    });
  const user = userEvent.setup();
  render(<TerminologyPage />);
  await user.click(await appendButton(globalBase.termbase.name));
  expect(screen.queryByRole("alert")).toBeNull();
  await user.click(await appendButton(globalBase.termbase.name));
  expect(await screen.findByRole("alert")).toHaveTextContent("文件无效");
  expect(vi.spyOn(commands, "appendTermbase")).not.toHaveBeenCalled();
});

it("retries known write failures with the retained document and refresh failures only reread", async () => {
  vi.spyOn(commands, "appendTermbase").mockResolvedValueOnce({
    status: "error",
    error: { code: "storage", recovery: "retry", message: "无法写入" },
  });
  const user = userEvent.setup();
  render(<TerminologyPage />);
  await user.click(await screen.findByRole("button", { name: /奇幻世界/ }));
  await user.click(screen.getByRole("button", { name: "返回术语库" }));
  await user.click(await appendButton(globalBase.termbase.name));
  expect(await screen.findByRole("alert")).toHaveTextContent("无法写入");
  vi.mocked(commands.getTermbase).mockResolvedValueOnce({
    status: "error",
    error: { code: "storage", recovery: "retry", message: "读取失败" },
  });
  await user.click(screen.getByRole("button", { name: "重试追加" }));
  await screen.findByText("已追加：新增 1 条，更新 2 条，未变更 3 条");
  await user.click(await screen.findByRole("button", { name: /奇幻世界/ }));
  await screen.findByText("无法读取术语库，请重试或返回库列表。");
  await user.click(screen.getByRole("button", { name: "重试" }));
  await screen.findByRole("heading", { name: "奇幻世界" });
  expect(commands.chooseTermbaseAppend).toHaveBeenCalledTimes(1);
  expect(vi.spyOn(commands, "appendTermbase")).toHaveBeenCalledTimes(2);
});

it("only exposes local append in Translator and keeps uncertain recovery available across readonly mode", async () => {
  const complete =
    vi.fn<
      (value: Awaited<ReturnType<typeof commands.appendTermbase>>) => void
    >();
  const pending = new Promise<
    Awaited<ReturnType<typeof commands.appendTermbase>>
  >((resolve) => {
    complete.mockImplementation(resolve);
  });
  vi.spyOn(commands, "appendTermbase").mockReturnValueOnce(pending);
  const user = userEvent.setup();
  const view = render(<Lookup />);
  await user.click(screen.getByRole("button", { name: "选择术语库" }));
  expect(
    await screen.findAllByRole("button", { name: "导出术语库" }),
  ).toHaveLength(2);
  expect(screen.getAllByRole("button", { name: "追加术语" })).toHaveLength(1);
  const button = await appendButton(localBase.termbase.name);
  fireEvent.pointerDown(button);
  await new Promise((resolve) => setTimeout(resolve, 550));
  fireEvent.pointerUp(button);
  expect(screen.queryByRole("dialog", { name: "编辑术语库" })).toBeNull();
  await user.click(button);
  expect(button).toBeDisabled();
  expect(screen.getByRole("combobox")).toBeDisabled();
  view.rerender(<Lookup readonly />);
  await act(async () => {
    complete({
      status: "error",
      error: {
        code: "commit_unknown",
        recovery: "wait_for_confirmation",
        message: "等待核实",
      },
    });
    await pending;
  });
  expect(await screen.findByRole("alert")).toHaveTextContent("等待核实");
  expect(block.mock.calls.at(-1)?.[0].shouldBlockFn()).toBe(true);
  const guard = vi.mocked(registerExitGuard).mock.calls.at(-1)?.[0];
  expect(await guard?.()).toBe(false);
  await user.click(screen.getByRole("button", { name: "核实写入结果" }));
  await screen.findByText("已追加：新增 1 条，更新 2 条，未变更 3 条");
  expect(commands.chooseTermbaseAppend).toHaveBeenCalledTimes(1);
  expect(vi.spyOn(commands, "appendTermbase").mock.calls).toEqual([
    [{ kind: "chapter", chapter_id: "chapter" }, "local", document],
    [{ kind: "chapter", chapter_id: "chapter" }, "local", document],
  ]);
});
