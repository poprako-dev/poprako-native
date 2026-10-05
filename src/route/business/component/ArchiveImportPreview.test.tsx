import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";
import type {
  ArchiveImportMode,
  ArchivePreview,
  ComicMetadata,
} from "@/bridge/generated/bindings";
import { ArchiveImportPreview } from "@/route/business/component/ArchiveImportPreview";

afterEach(cleanup);

it("previews skipped nonblank pages and requires the explicit replacement choice", async () => {
  const user = userEvent.setup();
  const submit =
    vi.fn<(metadata: ComicMetadata, mode: ArchiveImportMode) => void>();
  const preview: ArchivePreview = {
    metadata: { title: "输入资料", subtitle: "", author: "" },
    page_count: 1,
    unit_count: 3,
    warnings: [],
    pages: [
      {
        index: 0,
        source_image_name: "001.png",
        target_image_name: "scan.png",
        source_unit_count: 3,
        target_unit_count: 2,
      },
    ],
  };
  render(
    <ArchiveImportPreview
      preview={preview}
      existing={true}
      busy={false}
      error=""
      onSubmit={submit}
      onCancel={() => undefined}
    />,
  );
  expect(screen.getByText("跳过已有 2 个单元")).toBeDefined();
  expect(screen.queryByRole("textbox", { name: "标题" })).toBeNull();
  await user.click(screen.getByRole("button", { name: "确认只填空白页" }));
  expect(submit).toHaveBeenLastCalledWith(preview.metadata, "fill_empty");
  await user.click(screen.getByRole("radio", { name: /替换整页单元/u }));
  expect(screen.getByText("导入 3 个单元")).toBeDefined();
  await user.click(
    screen.getByRole("button", { name: "确认替换全部页面单元" }),
  );
  expect(submit).toHaveBeenLastCalledWith(preview.metadata, "replace_all");
});
