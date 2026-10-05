import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";
import { commands } from "@/bridge";
import type { ComicDetail, Page } from "@/bridge/generated/bindings";
import { PageManagementDialog } from "@/route/comic/$comicId/business/component/PageManagementDialog";

vi.mock("@/route/business/component/ManagedImage", () => ({
  ManagedImage: () => <span>图片</span>,
}));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

function page(id: string, index: number): Page {
  return {
    id,
    comic_id: "comic",
    index,
    unit_revision: 7,
    image: {
      reference: `image-${id}`,
      original_name: `${id}.png`,
      format: "png",
      width: 20,
      height: 30,
    },
    created_at: 1,
    updated_at: 1,
  };
}

it("waits for explicit deletion confirmation and submits the complete baseline", async () => {
  const user = userEvent.setup();
  const pages = [page("first", 0), page("second", 1)];
  const detail: ComicDetail = {
    comic: {
      id: "comic",
      title: "项目",
      subtitle: "",
      author: "",
      created_at: 1,
      updated_at: 1,
    },
    pages: pages.map((value) => ({
      page: value,
      unit_count: 1,
      translated_count: 0,
      proofread_count: 0,
      flagged_count: 0,
      edited_count: 0,
      proofreader_append_count: 0,
    })),
    work_position: null,
  };
  const remove = vi
    .spyOn(commands, "removePages")
    .mockResolvedValue({ status: "ok", data: [] });
  const complete = vi.fn<() => void>();
  render(
    <PageManagementDialog
      detail={detail}
      onClose={() => undefined}
      onComplete={complete}
      onReplace={() => undefined}
    />,
  );
  await user.click(screen.getByRole("checkbox", { name: "选择第 2 页" }));
  await user.click(screen.getByRole("button", { name: "删除所选" }));
  expect(remove).not.toHaveBeenCalled();
  await user.click(screen.getByRole("button", { name: "确认删除页面" }));
  await waitFor(() => {
    expect(complete).toHaveBeenCalledOnce();
  });
  expect(remove).toHaveBeenCalledWith({
    comic_id: "comic",
    baseline: pages.map((value) => ({
      id: value.id,
      unit_revision: value.unit_revision,
      image_reference: value.image.reference,
    })),
    page_ids: ["second"],
  });
});
