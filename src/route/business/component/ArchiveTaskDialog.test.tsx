import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { commands } from "@/bridge";
import type { ArchiveTask } from "@/bridge/generated/bindings";
import { ArchiveTaskDialog } from "./ArchiveTaskDialog";

const resource: {
  task: ArchiveTask;
  error: string;
  refresh: ReturnType<typeof vi.fn>;
} = vi.hoisted(() => ({
  task: { phase: "preparing", completed: 0, total: 1 },
  error: "",
  refresh: vi.fn(),
}));
vi.mock("@/route/business/use-archive-task", () => ({
  useArchiveTask: () => resource,
}));
beforeEach(() => {
  resource.refresh.mockReset();
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

it("closes a completed task by acknowledging its result and notifying the parent", async () => {
  resource.task = {
    phase: "completed",
    result: { chapter_id: "chapter", warnings: [] },
  };
  const acknowledge = vi
    .spyOn(commands, "acknowledgeArchiveTask")
    .mockResolvedValue({ status: "ok", data: null });
  const complete = vi.fn();
  render(
    <ArchiveTaskDialog
      taskId="task"
      intent="import"
      existing={false}
      onClose={vi.fn()}
      onComplete={complete}
    />,
  );
  expect(screen.queryByRole("button", { name: "完成" })).toBeNull();
  await userEvent.click(screen.getByRole("button", { name: "关闭弹窗" }));
  expect(acknowledge).toHaveBeenCalledExactlyOnceWith("task");
  expect(complete).toHaveBeenCalledExactlyOnceWith("chapter");
});

it("requests cancellation through the close control and waits for the task result", async () => {
  resource.task = { phase: "preparing", completed: 0, total: 1 };
  const cancel = vi
    .spyOn(commands, "cancelArchiveTask")
    .mockResolvedValue({ status: "ok", data: null });
  const close = vi.fn();
  render(
    <ArchiveTaskDialog
      taskId="task"
      intent="export"
      existing={false}
      onClose={close}
      onComplete={vi.fn()}
    />,
  );
  expect(screen.queryByRole("button", { name: "取消任务" })).toBeNull();
  await userEvent.click(screen.getByRole("button", { name: "关闭弹窗" }));
  expect(cancel).toHaveBeenCalledExactlyOnceWith("task");
  expect(resource.refresh).toHaveBeenCalledOnce();
  expect(close).not.toHaveBeenCalled();
});

it("blocks Escape and disables the close control while a commit is uncertain", async () => {
  resource.task = {
    phase: "failed",
    message: "待核实",
    commit_uncertain: true,
  };
  const close = vi.fn();
  render(
    <ArchiveTaskDialog
      taskId="task"
      intent="import"
      existing={false}
      onClose={close}
      onComplete={vi.fn()}
    />,
  );
  expect(screen.getByRole("button", { name: "关闭弹窗" })).toHaveProperty(
    "disabled",
    true,
  );
  await userEvent.keyboard("{Escape}");
  expect(close).not.toHaveBeenCalled();
  expect(screen.getByRole("dialog")).toBeTruthy();
});
