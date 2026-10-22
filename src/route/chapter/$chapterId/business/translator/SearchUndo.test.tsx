import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import type * as Bridge from "@/bridge";
import { createEditorSession } from "./editor-session";
import type { EditorSession } from "./editor-session";
import { editorFixture } from "./editor-test-fixture";
import { SearchDialog } from "./SearchDialog";

const native = vi.hoisted(() => ({
  searchUnits: vi.fn<typeof Bridge.commands.searchUnits>(),
  replaceUnits: vi.fn<typeof Bridge.commands.replaceUnits>(),
}));

vi.mock("@/bridge", async (importOriginal) => {
  const original = await importOriginal<typeof Bridge>();
  return {
    ...original,
    commands: {
      ...original.commands,
      ...native,
    },
  };
});
let session: EditorSession;
const refresh = vi.fn<(changedPageIds: string[]) => Promise<void>>();

beforeEach(() => {
  vi.mocked(native.searchUnits)
    .mockReset()
    .mockResolvedValue({
      status: "ok",
      data: [
        {
          unit_id: "a",
          page_id: "page",
          page_index: 0,
          unit_index: 0,
          unit_revision: 1,
          text: "changed",
        },
      ],
    });
  vi.mocked(native.replaceUnits).mockReset();
  refresh.mockReset().mockResolvedValue(undefined);
  session = createEditorSession(editorFixture(), "translation", {
    save: (snapshot) =>
      Promise.resolve({
        ...editorFixture(),
        page: {
          ...editorFixture().page,
          unit_revision: snapshot.expected_revision + 1,
        },
        units: snapshot.units.map((unit, index) => ({
          ...unit,
          index,
          page_id: "page",
          created_at: 1,
          updated_at: 2,
        })),
      }),
  });
  session.edit("a", { translated_text: "changed" });
});
afterEach(() => {
  cleanup();
  session.dispose();
});

async function search(): Promise<void> {
  render(
    <SearchDialog session={session} onRefresh={refresh} onClose={vi.fn()} />,
  );
  fireEvent.change(screen.getByRole("textbox", { name: "搜索内容" }), {
    target: { value: "changed" },
  });
  fireEvent.click(screen.getByRole("button", { name: "搜索" }));
  await screen.findByText("找到 1 个单元");
}

test.each([
  { changedPages: [] },
  { changedPages: ["other"] },
  { changedPages: ["page"] },
])(
  "only replacements actually affecting this page clear history: %j",
  async ({ changedPages }) => {
    vi.mocked(native.replaceUnits).mockResolvedValue({
      status: "ok",
      data: {
        changed_page_ids: changedPages,
        changed_unit_count: changedPages.length,
      },
    });
    await search();
    expect(session.store.getState().canUndo).toBe(true);
    fireEvent.click(screen.getByRole("button", { name: "替换所选 1 个单元" }));
    await waitFor(() => {
      expect(refresh).toHaveBeenCalledWith(changedPages);
    });
    await waitFor(() => {
      expect(
        screen
          .getByRole("button", { name: "关闭弹窗" })
          .hasAttribute("disabled"),
      ).toBe(false);
    });
    expect(session.store.getState().canUndo).toBe(
      !changedPages.includes("page"),
    );
    fireEvent.click(screen.getByRole("button", { name: "关闭弹窗" }));
    act(() => {
      session.undo();
    });
    expect(session.store.getState().draft[0]?.translated_text).toBe(
      changedPages.includes("page") ? "changed" : "翻译 a",
    );
  },
);

test("a committed replacement clears history before failed refresh and retries only reading", async () => {
  vi.mocked(native.replaceUnits).mockResolvedValue({
    status: "ok",
    data: { changed_page_ids: ["page"], changed_unit_count: 1 },
  });
  refresh.mockRejectedValueOnce(new Error("read failed"));
  await search();
  fireEvent.click(screen.getByRole("button", { name: "替换所选 1 个单元" }));
  await screen.findByText("修改已保存，但刷新失败。请重试刷新，不要再次替换。");
  expect(session.store.getState()).toMatchObject({
    canUndo: false,
    locked: true,
  });
  fireEvent.click(screen.getByRole("button", { name: "重试刷新" }));
  await waitFor(() => {
    expect(refresh).toHaveBeenCalledTimes(2);
  });
  expect(native.replaceUnits).toHaveBeenCalledTimes(1);
  expect(refresh.mock.calls).toEqual([[["page"]], [["page"]]]);
});

test("a failed replacement retains the saved editing history", async () => {
  vi.mocked(native.replaceUnits).mockResolvedValue({
    status: "error",
    error: { code: "conflict", message: "内容已变更", recovery: "retry" },
  });
  await search();
  fireEvent.click(screen.getByRole("button", { name: "替换所选 1 个单元" }));
  await screen.findByText("内容已变更");
  expect(session.store.getState().canUndo).toBe(true);
  expect(refresh).not.toHaveBeenCalled();
});
