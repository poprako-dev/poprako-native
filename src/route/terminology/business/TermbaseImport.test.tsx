import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { commands } from "@/bridge";
import {
  globalBase,
  setupTerminology,
} from "@/route/business/terminology/terminology-test-fixture";
import { TerminologyPage } from "./TerminologyPage";

vi.mock("@tanstack/react-router", () => ({
  useNavigate: vi.fn(),
  useBlocker: vi.fn(),
}));

const document = {
  metadata: { name: "文件中的名称", description: "文件中的描述" },
  terms: [{ source: "団長", targets: ["团长", "队长"], comment: "" }],
};

beforeEach(setupTerminology);
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

it("imports the selected file without requiring manual metadata and enters the library", async () => {
  const user = userEvent.setup();
  vi.spyOn(commands, "chooseTermbaseImport").mockResolvedValue({
    status: "ok",
    data: document,
  });
  const imported = vi
    .spyOn(commands, "importTermbase")
    .mockResolvedValue({ status: "ok", data: globalBase.termbase });
  render(<TerminologyPage />);
  await user.click(screen.getByRole("button", { name: "新建术语库" }));
  await user.click(screen.getByRole("button", { name: "导入" }));
  expect(
    await screen.findByRole("textbox", { name: "搜索术语原文" }),
  ).toBeTruthy();
  expect(imported).toHaveBeenCalledExactlyOnceWith(
    { kind: "global" },
    document,
  );
});

it("keeps manual input after cancellation or an invalid file without submitting a write", async () => {
  const user = userEvent.setup();
  vi.spyOn(commands, "chooseTermbaseImport")
    .mockResolvedValueOnce({ status: "ok", data: null })
    .mockResolvedValue({
      status: "error",
      error: {
        code: "termbase_import",
        message: "文件无效",
        recovery: "correct_input",
      },
    });
  const imported = vi.spyOn(commands, "importTermbase");
  render(<TerminologyPage />);
  await user.click(screen.getByRole("button", { name: "新建术语库" }));
  await user.type(screen.getByRole("textbox", { name: "名称" }), "手填名称");
  await user.click(screen.getByRole("button", { name: "导入" }));
  expect(screen.getByRole("textbox", { name: "名称" })).toHaveProperty(
    "value",
    "手填名称",
  );
  await user.click(screen.getByRole("button", { name: "导入" }));
  expect(await screen.findByText("文件无效")).toBeTruthy();
  expect(screen.getByRole("textbox", { name: "名称" })).toHaveProperty(
    "value",
    "手填名称",
  );
  expect(imported).not.toHaveBeenCalled();
});

it("confirms an uncertain import with the same document without reopening the picker", async () => {
  const user = userEvent.setup();
  const choose = vi
    .spyOn(commands, "chooseTermbaseImport")
    .mockResolvedValue({ status: "ok", data: document });
  const imported = vi
    .spyOn(commands, "importTermbase")
    .mockResolvedValueOnce({
      status: "error",
      error: {
        code: "commit_unknown",
        message: "等待核实",
        recovery: "wait_for_confirmation",
      },
    })
    .mockResolvedValue({ status: "ok", data: globalBase.termbase });
  render(<TerminologyPage />);
  await user.click(screen.getByRole("button", { name: "新建术语库" }));
  await user.click(screen.getByRole("button", { name: "导入" }));
  expect(await screen.findByText("等待核实")).toBeTruthy();
  expect(screen.getByRole("button", { name: "导入" })).toHaveProperty(
    "disabled",
    true,
  );
  await user.click(screen.getByRole("button", { name: "核实写入结果" }));
  await waitFor(() => {
    expect(screen.queryByRole("dialog")).toBeNull();
  });
  expect(choose).toHaveBeenCalledTimes(1);
  expect(imported.mock.calls).toEqual([
    [{ kind: "global" }, document],
    [{ kind: "global" }, document],
  ]);
});
