import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { commands } from "@/bridge";
import {
  globalBase,
  setupTerminology,
  term,
} from "@/route/business/terminology/terminology-test-fixture";
import { TerminologyPage } from "./TerminologyPage";

const { navigate } = vi.hoisted(() => ({
  navigate: vi.fn<() => Promise<void>>(),
}));
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => navigate,
  useBlocker: vi.fn(),
}));
beforeEach(() => {
  setupTerminology();
  vi.mocked(commands.listTermbaseInfos).mockResolvedValue({
    status: "ok",
    data: [globalBase],
  });
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

it("shows the utilities directory structure and preserves searches when returning", async () => {
  const user = userEvent.setup();
  render(<TerminologyPage />);
  const input = screen.getByRole("textbox", { name: "搜索术语库名称" });
  await user.type(input, "奇幻");
  await waitFor(() => {
    expect(commands.listTermbaseInfos).toHaveBeenLastCalledWith(
      { kind: "global" },
      { text: "奇幻", offset: 0, limit: 30 },
    );
  });
  await user.click(await screen.findByRole("button", { name: /奇幻世界/ }));
  expect(
    await screen.findByRole("textbox", { name: "搜索术语原文" }),
  ).toBeTruthy();
  expect(await screen.findByText("団長")).toBeTruthy();
  await user.type(screen.getByRole("textbox", { name: "搜索术语原文" }), "団");
  await waitFor(() => {
    expect(commands.listTermInfos).toHaveBeenLastCalledWith(
      { kind: "global" },
      "global",
      { text: "団", offset: 0, limit: 30 },
    );
  });
  await user.click(screen.getByRole("button", { name: "返回术语库" }));
  await waitFor(() => {
    expect(
      screen.getByRole("textbox", { name: "搜索术语库名称" }),
    ).toHaveProperty("value", "奇幻");
  });
  await user.click(screen.getByRole("button", { name: /奇幻世界/ }));
  await waitFor(() => {
    expect(
      screen.getByRole("textbox", { name: "搜索术语原文" }),
    ).toHaveProperty("value", "団");
  });
});

it("changes directory level even when animation frames are suspended", async () => {
  vi.stubGlobal(
    "matchMedia",
    vi.fn(() => ({
      matches: false,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })),
  );
  vi.spyOn(window, "requestAnimationFrame").mockReturnValue(1);
  const original = Object.getOwnPropertyDescriptor(
    HTMLElement.prototype,
    "animate",
  );
  Object.defineProperty(HTMLElement.prototype, "animate", {
    configurable: true,
    value: vi.fn<HTMLElement["animate"]>(),
  });
  try {
    const user = userEvent.setup();
    render(<TerminologyPage />);
    await user.click(await screen.findByRole("button", { name: /奇幻世界/ }));
    expect(
      await screen.findByRole("textbox", { name: "搜索术语原文" }),
    ).toBeTruthy();
    expect(
      screen.queryByRole("textbox", { name: "搜索术语库名称" }),
    ).toBeNull();
    await user.click(screen.getByRole("button", { name: "返回术语库" }));
    expect(
      screen.getByRole("textbox", { name: "搜索术语库名称" }),
    ).toBeTruthy();
  } finally {
    if (original)
      Object.defineProperty(HTMLElement.prototype, "animate", original);
    else Reflect.deleteProperty(HTMLElement.prototype, "animate");
  }
});

it("opens a newly created library and keeps failed term input and translation order", async () => {
  const user = userEvent.setup();
  const created = { ...globalBase.termbase, id: "new", name: "地名" };
  vi.spyOn(commands, "createTermbase").mockResolvedValue({
    status: "ok",
    data: created,
  });
  vi.mocked(commands.getTermbase).mockResolvedValue({
    status: "ok",
    data: { termbase: created, term_count: 0 },
  });
  const create = vi
    .spyOn(commands, "createTerm")
    .mockResolvedValueOnce({
      status: "error",
      error: { code: "storage_busy", message: "暂时繁忙", recovery: "retry" },
    })
    .mockResolvedValue({ status: "ok", data: term });
  render(<TerminologyPage />);
  await user.click(screen.getByRole("button", { name: "新建术语库" }));
  await user.type(screen.getByRole("textbox", { name: "名称" }), "地名");
  await user.click(screen.getByRole("button", { name: "保存" }));
  await user.click(await screen.findByRole("button", { name: "新建术语" }));
  await user.type(screen.getByRole("textbox", { name: "原文" }), "王都");
  await user.type(screen.getByRole("textbox", { name: "译名 1" }), "王城");
  await user.click(screen.getByRole("button", { name: "添加译名" }));
  await user.type(screen.getByRole("textbox", { name: "译名 2" }), "都城");
  await user.click(screen.getByRole("button", { name: "上移译名 2" }));
  await user.click(screen.getByRole("button", { name: "保存" }));
  expect(await screen.findByText("暂时繁忙")).toBeTruthy();
  expect(screen.getByRole("textbox", { name: "原文" })).toHaveProperty(
    "value",
    "王都",
  );
  expect(screen.getByRole("textbox", { name: "译名 1" })).toHaveProperty(
    "value",
    "都城",
  );
  await user.click(screen.getByRole("button", { name: "保存" }));
  await waitFor(() => {
    expect(screen.queryByRole("dialog")).toBeNull();
  });
  expect(create).toHaveBeenLastCalledWith({ kind: "global" }, "new", {
    source: "王都",
    targets: ["都城", "王城"],
    comment: "",
  });
});

it("supports keyboard editing and confirms cascading library deletion", async () => {
  const user = userEvent.setup();
  const remove = vi
    .spyOn(commands, "deleteTermbase")
    .mockResolvedValue({ status: "ok", data: null });
  render(<TerminologyPage />);
  const row = await screen.findByRole("button", { name: /奇幻世界/ });
  row.focus();
  fireEvent.keyDown(row, { key: "F2" });
  expect(screen.getByRole("dialog", { name: "编辑术语库" })).toBeTruthy();
  await user.click(screen.getByRole("button", { name: "删除" }));
  expect(screen.getByText(/及其全部术语/)).toBeTruthy();
  expect(remove).not.toHaveBeenCalled();
  await user.click(screen.getByRole("button", { name: "确认删除" }));
  await waitFor(() => {
    expect(remove).toHaveBeenCalledWith(
      { kind: "global" },
      globalBase.termbase,
    );
  });
});

it("retries only the read when refresh fails after a successful create", async () => {
  const user = userEvent.setup();
  const create = vi
    .spyOn(commands, "createTermbase")
    .mockResolvedValue({ status: "ok", data: globalBase.termbase });
  vi.mocked(commands.getTermbase)
    .mockRejectedValueOnce(new Error("read failed"))
    .mockResolvedValue({ status: "ok", data: globalBase });
  render(<TerminologyPage />);
  await user.click(screen.getByRole("button", { name: "新建术语库" }));
  await user.type(screen.getByRole("textbox", { name: "名称" }), "奇幻世界");
  await user.click(screen.getByRole("button", { name: "保存" }));
  await user.click(await screen.findByRole("button", { name: "重试" }));
  expect(await screen.findByText("団長")).toBeTruthy();
  expect(create).toHaveBeenCalledTimes(1);
});

it("retains an uncertain write and prevents canceling or changing its payload", async () => {
  const user = userEvent.setup();
  const create = vi
    .spyOn(commands, "createTermbase")
    .mockResolvedValueOnce({
      status: "error",
      error: {
        code: "commit_uncertain",
        message: "提交结果待核实",
        recovery: "wait_for_confirmation",
      },
    })
    .mockResolvedValue({ status: "ok", data: globalBase.termbase });
  render(<TerminologyPage />);
  await user.click(screen.getByRole("button", { name: "新建术语库" }));
  await user.type(screen.getByRole("textbox", { name: "名称" }), "奇幻世界");
  await user.click(screen.getByRole("button", { name: "保存" }));
  expect(await screen.findByText("提交结果待核实")).toBeTruthy();
  expect(screen.getByRole("textbox", { name: "名称" })).toHaveProperty(
    "disabled",
    true,
  );
  expect(screen.getByRole("button", { name: "关闭弹窗" })).toHaveProperty(
    "disabled",
    true,
  );
  await user.keyboard("{Escape}");
  expect(screen.getByRole("dialog", { name: "新建术语库" })).toBeTruthy();
  await user.click(screen.getByRole("button", { name: "核实写入结果" }));
  await waitFor(() => {
    expect(screen.queryByRole("dialog")).toBeNull();
  });
  expect(create.mock.calls[1]).toEqual(create.mock.calls[0]);
});
