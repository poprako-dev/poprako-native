import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";
import { commands } from "@/bridge";
import type {
  Chapter,
  ChapterInfo,
  ComicInfo,
} from "@/bridge/generated/bindings";
import { HomePage } from "@/route/home/business/component/HomePage";

const { navigate } = vi.hoisted(() => ({
  navigate: vi.fn<() => Promise<void>>(),
}));
vi.mock("@tanstack/react-router", () => ({ useNavigate: () => navigate }));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  navigate.mockReset();
});

const comic: ComicInfo = {
  comic: {
    id: "comic",
    title: "漫画",
    author: "作者",
    created_at: 1,
    updated_at: 1,
  },
  chapter_count: 1,
  page_count: 1,
  unit_count: 2,
  translated_count: 1,
  proofread_count: 1,
  cover_page_id: "page",
  cover_chapter_id: "chapter",
  latest_chapter_id: "chapter",
  last_opened_at: 2,
};
const chapter: Chapter = {
  id: "chapter",
  comic_id: "comic",
  title: "序章",
  index: 0,
  created_at: 1,
  updated_at: 1,
};
const info: ChapterInfo = {
  chapter,
  page_count: 1,
  unit_count: 2,
  translated_count: 1,
  proofread_count: 1,
  cover_page_id: "page",
  last_opened_at: 2,
};

it("loads chapters only on expansion, retries locally and supports keyboard disclosure", async () => {
  const user = userEvent.setup();
  vi.spyOn(commands, "listComicInfos").mockResolvedValue({
    status: "ok",
    data: [comic],
  });
  const chapters = vi
    .spyOn(commands, "listChapterInfos")
    .mockRejectedValueOnce(new Error("unavailable"))
    .mockResolvedValue({ status: "ok", data: [info] });
  const position = vi.spyOn(commands, "updateWorkPosition");
  render(<HomePage />);
  const row = await screen.findByRole("button", { expanded: false });
  expect(chapters).not.toHaveBeenCalled();
  expect(screen.queryByTitle("总单元数")).toBeNull();
  expect(screen.queryByTitle("最近活动")).toBeNull();
  row.focus();
  await user.keyboard("{Enter}");
  await screen.findByText("章节无法读取，请重试。");
  await user.click(screen.getByRole("button", { name: "重试" }));
  await screen.findByText("序章");
  expect(chapters).toHaveBeenLastCalledWith("comic");
  expect(position).not.toHaveBeenCalled();
  expect(navigate).not.toHaveBeenCalled();
  await user.click(screen.getByRole("button", { expanded: true }));
  expect(screen.queryByText("序章")).toBeNull();
  await user.click(screen.getByRole("button", { expanded: false }));
  await screen.findByText("序章");
  expect(screen.queryByRole("button", { name: "继续翻校 序章" })).toBeNull();
  expect(screen.queryByRole("button", { name: /^(上移|下移)/ })).toBeNull();
  navigate.mockResolvedValue();
  await user.click(screen.getByRole("button", { name: /^序章/ }));
  expect(navigate).toHaveBeenCalledWith({
    to: "/chapter/$chapterId",
    params: { chapterId: "chapter" },
  });
});

it("creates a comic and first chapter in one request and freezes uncertain input for verification", async () => {
  const user = userEvent.setup();
  vi.spyOn(commands, "listComicInfos").mockResolvedValue({
    status: "ok",
    data: [],
  });
  const create = vi
    .spyOn(commands, "createComic")
    .mockResolvedValueOnce({
      status: "error",
      error: {
        code: "commit_unknown",
        message: "需要核实",
        recovery: "wait_for_confirmation",
      },
    })
    .mockResolvedValue({ status: "ok", data: chapter });
  render(<HomePage />);
  await user.click(await screen.findByRole("button", { name: "新建漫画" }));
  await user.type(screen.getByRole("textbox", { name: "漫画标题" }), "新漫画");
  await user.type(screen.getByRole("textbox", { name: "漫画作者" }), "新作者");
  await user.click(screen.getByRole("button", { name: "保存" }));
  await screen.findByText("需要核实");
  expect(
    screen.getByRole("textbox", { name: "漫画标题" }).hasAttribute("disabled"),
  ).toBe(true);
  await user.keyboard("{Escape}");
  expect(screen.getByRole("dialog")).toBeDefined();
  await user.click(screen.getByRole("button", { name: "核实原操作结果" }));
  await waitFor(() => {
    expect(screen.queryByRole("dialog")).toBeNull();
  });
  expect(create).toHaveBeenCalledTimes(2);
  expect(create.mock.calls[0]).toEqual(create.mock.calls[1]);
  expect(create).toHaveBeenCalledWith({
    comic: { title: "新漫画", author: "新作者" },
    chapter_title: "第1章",
  });
});

it("creates and imports chapters from a collapsed comic with the current chapter baseline", async () => {
  const user = userEvent.setup();
  vi.spyOn(commands, "listComicInfos").mockResolvedValue({
    status: "ok",
    data: [comic],
  });
  const chapters = vi
    .spyOn(commands, "listChapterInfos")
    .mockResolvedValue({ status: "ok", data: [info] });
  const create = vi
    .spyOn(commands, "createChapter")
    .mockResolvedValue({ status: "ok", data: chapter });
  const choose = vi
    .spyOn(commands, "chooseArchiveImport")
    .mockResolvedValue({ status: "ok", data: { status: "cancelled" } });
  render(<HomePage />);
  await user.click(
    await screen.findByRole("button", { name: "在 漫画 中新增章节" }),
  );
  await user.type(
    await screen.findByRole("textbox", { name: "章节名称" }),
    "第二章",
  );
  await user.click(screen.getByRole("button", { name: "保存" }));
  await waitFor(() => {
    expect(screen.queryByRole("dialog")).toBeNull();
  });
  expect(create).toHaveBeenCalledWith("comic", ["chapter"], {
    title: "第二章",
  });
  expect(screen.getByRole("button", { expanded: false })).toBeDefined();
  await user.click(screen.getByRole("button", { name: "向 漫画 导入章节" }));
  await user.click(
    await screen.findByRole("button", { name: "选择译文或项目包" }),
  );
  expect(chapters).toHaveBeenCalledTimes(2);
  expect(choose).toHaveBeenCalledWith(
    { kind: "new_chapter", comic_id: "comic", baseline: ["chapter"] },
    "translation",
    true,
  );
});
