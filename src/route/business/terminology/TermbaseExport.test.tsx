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
import { TerminologyPage } from "@/route/terminology/business/TerminologyPage";
import { TerminologyLookup } from "./TerminologyLookup";
import { TermbaseExportNotice } from "./TermbaseExportNotice";
import {
  globalBase,
  localBase,
  setupTerminology,
} from "./terminology-test-fixture";
import { useTermbaseExport } from "./use-termbase-export";

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useBlocker: vi.fn(),
}));

type ExportResult = Awaited<ReturnType<typeof commands.exportTermbase>>;
type DeferredExport = {
  promise: Promise<ExportResult>;
  complete: (result: ExportResult) => void;
};

function deferred(): DeferredExport {
  const complete = vi.fn<(result: ExportResult) => void>();
  const promise = new Promise<ExportResult>((resolve) => {
    complete.mockImplementation(resolve);
  });
  return { promise, complete };
}

function Lookup({ readonly = false }: { readonly?: boolean }): ReactElement {
  const exports = useTermbaseExport({ kind: "chapter", chapter_id: "chapter" });
  return (
    <>
      {!readonly && (
        <TerminologyLookup
          chapterId="chapter"
          locked={false}
          exports={exports}
          onInteraction={vi.fn()}
        />
      )}
      <TermbaseExportNotice exports={exports} />
    </>
  );
}

beforeEach(() => {
  setupTerminology();
  vi.spyOn(commands, "exportTermbase").mockResolvedValue({
    status: "ok",
    data: "saved",
  });
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

it("exports from the global directory without selecting a library or saving editor content", async () => {
  vi.mocked(commands.listTermbaseInfos).mockResolvedValue({
    status: "ok",
    data: [globalBase],
  });
  const save = vi.spyOn(commands, "savePageUnits");
  const user = userEvent.setup();
  render(<TerminologyPage />);
  const button = await screen.findByRole("button", { name: "导出术语库" });
  button.focus();
  await user.keyboard("{Enter}");
  expect(commands.exportTermbase).toHaveBeenCalledExactlyOnceWith(
    { kind: "global" },
    "global",
  );
  expect(await screen.findByText("已导出术语库")).toBeVisible();
  expect(screen.getByRole("textbox", { name: "搜索术语库名称" })).toBeVisible();
  expect(commands.listTermInfos).not.toHaveBeenCalled();
  expect(save).not.toHaveBeenCalled();
});

it("exports read-only global libraries, isolates long press, and survives closing and mode changes", async () => {
  const pending = deferred();
  vi.mocked(commands.exportTermbase).mockReturnValue(pending.promise);
  const user = userEvent.setup();
  const view = render(<Lookup />);
  await user.click(screen.getByRole("button", { name: "选择术语库" }));
  const list = await screen.findByRole("list", { name: "术语库列表" });
  const rows = within(list).getAllByRole("listitem");
  const row = rows.find((item) =>
    item.textContent.includes(globalBase.termbase.name),
  );
  if (!row) throw new Error("Missing global row");
  const button = within(row).getByRole("button", { name: "导出术语库" });
  fireEvent.pointerDown(button);
  await new Promise((resolve) => setTimeout(resolve, 550));
  fireEvent.pointerUp(button);
  expect(screen.queryByRole("dialog", { name: "编辑术语库" })).toBeNull();
  await user.click(button);
  await user.click(button);
  expect(commands.exportTermbase).toHaveBeenCalledExactlyOnceWith(
    { kind: "chapter", chapter_id: "chapter" },
    "global",
  );
  expect(button).toBeDisabled();
  expect(button).toHaveAttribute("aria-busy", "true");
  expect(screen.getByRole("status", { name: "正在导出术语库" })).toBeVisible();
  expect(screen.getByRole("combobox")).toBeDisabled();
  await user.keyboard("{Escape}");
  await user.click(screen.getByRole("button", { name: "选择术语库" }));
  expect(
    await screen.findByRole("status", { name: "正在导出术语库" }),
  ).toBeVisible();
  view.rerender(<Lookup readonly />);
  await act(async () => {
    pending.complete({ status: "ok", data: "saved" });
    await pending.promise;
  });
  expect(screen.getByText("已导出术语库")).toBeVisible();
  expect(commands.listTermInfos).not.toHaveBeenCalled();
});

it("keeps pending state when filtering removes a row and supports independent libraries", async () => {
  const pending = deferred();
  vi.mocked(commands.exportTermbase).mockReturnValue(pending.promise);
  vi.mocked(commands.listTermbaseInfos).mockImplementation((_access, query) =>
    Promise.resolve({
      status: "ok",
      data: query.text === "本作" ? [localBase] : [globalBase, localBase],
    }),
  );
  const user = userEvent.setup();
  render(<Lookup />);
  await user.click(screen.getByRole("button", { name: "选择术语库" }));
  const buttons = await screen.findAllByRole("button", { name: "导出术语库" });
  const first = buttons[0];
  if (!first) throw new Error("Missing export button");
  await user.click(first);
  const search = screen.getByRole("textbox", { name: "搜索术语库名称" });
  await user.type(search, "本作");
  await waitFor(() => {
    expect(screen.queryByRole("button", { name: /奇幻世界/ })).toBeNull();
  });
  await user.click(await screen.findByRole("button", { name: "导出术语库" }));
  expect(commands.exportTermbase).toHaveBeenCalledTimes(2);
  await user.clear(search);
  await waitFor(() => {
    expect(
      screen.getAllByRole("status", { name: "正在导出术语库" }),
    ).toHaveLength(2);
  });
  await act(async () => {
    pending.complete({ status: "ok", data: "saved" });
    await pending.promise;
  });
  expect(screen.queryByRole("status", { name: "正在导出术语库" })).toBeNull();
});

it("silently handles cancellation and allows retry after failure", async () => {
  vi.mocked(commands.exportTermbase)
    .mockResolvedValueOnce({ status: "ok", data: "cancelled" })
    .mockResolvedValueOnce({
      status: "error",
      error: { code: "storage", message: "无法保存文件", recovery: "retry" },
    });
  const user = userEvent.setup();
  render(<TerminologyPage />);
  const buttons = await screen.findAllByRole("button", { name: "导出术语库" });
  const button = buttons[0];
  if (!button) throw new Error("Missing export button");
  await user.click(button);
  await waitFor(() => expect(button).toBeEnabled());
  expect(screen.queryByText("已导出术语库")).toBeNull();
  expect(screen.queryByRole("alert")).toBeNull();
  await user.click(button);
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "请重试或更换保存位置",
  );
  await user.click(button);
  expect(await screen.findByText("已导出术语库")).toBeVisible();
});
