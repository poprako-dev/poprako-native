import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import type * as Bridge from "@/bridge";
import type { ImageTaskStatus } from "@/bridge/generated/bindings";
import { ImageImportDialog } from "@/route/business/component/ImageImportDialog";

const mock = vi.hoisted(() => ({
  select: vi.fn<typeof Bridge.commands.selectImages>(),
  status: vi.fn<typeof Bridge.commands.getImageImport>(),
  confirm: vi.fn<typeof Bridge.commands.confirmImageImport>(),
  cancel: vi.fn<typeof Bridge.commands.cancelImageImport>(),
}));

vi.mock("@/bridge", async (original) => {
  const actual = await original<typeof Bridge>();
  return {
    ...actual,
    commands: {
      ...actual.commands,
      selectImages: mock.select,
      getImageImport: mock.status,
      confirmImageImport: mock.confirm,
      cancelImageImport: mock.cancel,
    },
  };
});

const ready: ImageTaskStatus = {
  task_id: "prepared-task",
  phase: "ready",
  total_files: 1,
  processed_files: 1,
  processed_bytes: 100,
  message: "图片已准备",
  cleanup_pending: false,
  previews: [],
};

afterEach(cleanup);
beforeEach(() => {
  vi.resetAllMocks();
  mock.select.mockResolvedValue({ status: "ok", data: ready });
  mock.status.mockResolvedValue({ status: "ok", data: ready });
  mock.confirm.mockResolvedValue({ status: "ok", data: [] });
});

it("shows committed cleanup warnings and allows closing without replaying the write", async () => {
  const user = userEvent.setup();
  const complete = vi.fn<() => void>();
  render(
    <ImageImportDialog
      target={{
        kind: "replace",
        comic_id: "comic",
        baseline: { id: "page", unit_revision: 0, image_reference: "old" },
      }}
      onClose={() => undefined}
      onComplete={complete}
    />,
  );
  await user.click(screen.getByRole("button", { name: "选择图片" }));
  await screen.findByRole("button", { name: "确认导入" });
  mock.status.mockResolvedValue({
    status: "ok",
    data: {
      ...ready,
      phase: "completed",
      cleanup_pending: true,
      message: "图片已保存",
    },
  });
  await user.click(screen.getByRole("button", { name: "确认导入" }));
  await screen.findByText(/下次导入时将自动重试清理/u);
  expect(complete).not.toHaveBeenCalled();
  await user.click(screen.getByRole("button", { name: "完成" }));
  expect(complete).toHaveBeenCalledOnce();
  expect(mock.confirm).toHaveBeenCalledOnce();
  expect(mock.cancel).not.toHaveBeenCalled();
});

it("keeps a successful write completed when the following status read fails", async () => {
  const user = userEvent.setup();
  const complete = vi.fn<() => void>();
  render(
    <ImageImportDialog
      target={{ kind: "append", comic_id: "comic", baseline: [] }}
      onClose={() => undefined}
      onComplete={complete}
    />,
  );
  await user.click(screen.getByRole("button", { name: "选择图片" }));
  await screen.findByRole("button", { name: "确认导入" });
  await waitFor(() => {
    expect(mock.status).toHaveBeenCalled();
  });
  mock.status.mockRejectedValue(new Error("status unavailable"));
  await user.click(screen.getByRole("button", { name: "确认导入" }));
  await screen.findByRole("button", { name: "完成" });
  expect(screen.queryByRole("button", { name: "核实提交结果" })).toBeNull();
  await user.click(screen.getByRole("button", { name: "完成" }));
  expect(complete).toHaveBeenCalledOnce();
  expect(mock.confirm).toHaveBeenCalledOnce();
});

it("ignores an older ready response after the write completes", async () => {
  const user = userEvent.setup();
  let resolve: (value: Awaited<ReturnType<typeof mock.status>>) => void = () =>
    undefined;
  const older = new Promise<Awaited<ReturnType<typeof mock.status>>>((done) => {
    resolve = done;
  });
  mock.status.mockReturnValueOnce(older);
  render(
    <ImageImportDialog
      target={{ kind: "append", comic_id: "comic", baseline: [] }}
      onClose={() => undefined}
      onComplete={() => undefined}
    />,
  );
  await user.click(screen.getByRole("button", { name: "选择图片" }));
  await screen.findByRole("button", { name: "确认导入" });
  await waitFor(() => {
    expect(mock.status).toHaveBeenCalledOnce();
  });
  mock.status.mockResolvedValue({
    status: "ok",
    data: { ...ready, phase: "completed", cleanup_pending: true },
  });
  await user.click(screen.getByRole("button", { name: "确认导入" }));
  await screen.findByRole("button", { name: "完成" });
  await act(async () => {
    resolve({ status: "ok", data: ready });
    await older;
  });
  expect(screen.queryByRole("button", { name: "确认导入" })).toBeNull();
  expect(screen.getByRole("button", { name: "完成" })).toBeDefined();
  expect(mock.confirm).toHaveBeenCalledOnce();
});

it("resumes status polling after cancellation returns stopping", async () => {
  const user = userEvent.setup();
  const selecting: ImageTaskStatus = { ...ready, phase: "selecting" };
  mock.select.mockResolvedValue({ status: "ok", data: selecting });
  mock.status.mockResolvedValue({ status: "ok", data: selecting });
  mock.cancel.mockResolvedValue({
    status: "ok",
    data: { ...ready, phase: "stopping" },
  });
  render(
    <ImageImportDialog
      target={{ kind: "append", comic_id: "comic", baseline: [] }}
      onClose={() => undefined}
      onComplete={() => undefined}
    />,
  );
  await user.click(screen.getByRole("button", { name: "选择图片" }));
  await waitFor(() => {
    expect(mock.status).toHaveBeenCalled();
  });
  mock.status.mockResolvedValue({
    status: "ok",
    data: { ...ready, phase: "cancelled", message: "已取消选择" },
  });
  await user.click(screen.getByRole("button", { name: "取消" }));
  await screen.findByText(/已取消选择/u);
  expect(screen.getByRole("button", { name: "选择图片" })).toBeDefined();
  expect(mock.cancel).toHaveBeenCalledOnce();
});
