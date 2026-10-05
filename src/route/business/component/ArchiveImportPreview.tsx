import { useState } from "react";
import type { ReactElement } from "react";
import type {
  ArchiveImportMode,
  ArchivePreview,
  ComicMetadata,
} from "@/bridge/generated/bindings";
import { MetadataForm } from "@/route/business/component/MetadataForm";
import { Button } from "@/shared/component/Button";

type ArchiveImportPreviewProps = {
  preview: ArchivePreview;
  existing: boolean;
  busy: boolean;
  error: string;
  onSubmit: (metadata: ComicMetadata, mode: ArchiveImportMode) => void;
  onCancel: () => void;
};

export function ArchiveImportPreview({
  preview,
  existing,
  busy,
  error,
  onSubmit,
  onCancel,
}: ArchiveImportPreviewProps): ReactElement {
  const [mode, setMode] = useState<ArchiveImportMode>("fill_empty");
  return (
    <div className="space-y-4">
      <p className="text-sm">
        共 {preview.page_count} 页，{preview.unit_count}{" "}
        个翻校单元。请核对页面顺序后再确认。
      </p>
      {preview.warnings.map((warning) => (
        <p key={warning} className="text-xs text-muted-foreground">
          {warning}
        </p>
      ))}
      <div className="max-h-60 overflow-auto rounded border border-border">
        <table className="w-full text-left text-xs">
          <thead className="sticky top-0 bg-muted">
            <tr>
              <th className="p-2">页码</th>
              <th className="p-2">译文图片引用</th>
              <th className="p-2">对应图片</th>
              <th className="p-2">处理</th>
            </tr>
          </thead>
          <tbody>
            {preview.pages.map((page) => (
              <tr key={page.index} className="border-t border-border">
                <td className="p-2">P{page.index + 1}</td>
                <td
                  className="max-w-48 truncate p-2"
                  title={page.source_image_name}
                >
                  {page.source_image_name || "未提供名称"}
                </td>
                <td
                  className="max-w-48 truncate p-2"
                  title={page.target_image_name}
                >
                  {page.target_image_name}
                </td>
                <td className="p-2">
                  {existing &&
                  mode === "fill_empty" &&
                  page.target_unit_count > 0
                    ? `跳过已有 ${String(page.target_unit_count)} 个单元`
                    : `导入 ${String(page.source_unit_count)} 个单元`}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {existing && (
        <fieldset disabled={busy} className="space-y-2 text-sm">
          <legend className="mb-2 font-medium">覆盖方式</legend>
          <label className="flex gap-2">
            <input
              type="radio"
              name="overwrite"
              checked={mode === "fill_empty"}
              onChange={() => {
                setMode("fill_empty");
              }}
            />
            只填空白页：已有任何单元的页面全部跳过
          </label>
          <label className="flex gap-2">
            <input
              type="radio"
              name="overwrite"
              checked={mode === "replace_all"}
              onChange={() => {
                setMode("replace_all");
              }}
            />
            替换整页单元：清除原有全部单元，无法撤销
          </label>
        </fieldset>
      )}
      {!existing && (
        <MetadataForm
          title={preview.metadata.title}
          subtitle={preview.metadata.subtitle}
          author={preview.metadata.author}
          busy={busy}
          error={error}
          submitLabel="确认导入"
          onCancel={onCancel}
          onSubmit={(title, subtitle, author) => {
            onSubmit({ title, subtitle, author }, mode);
          }}
        />
      )}
      {existing && (
        <>
          <p className="text-xs text-muted-foreground">
            当前项目资料保持不变，仅按所选方式导入各页单元。
          </p>
          {error !== "" && (
            <p role="alert" className="text-xs text-destructive">
              {error}
            </p>
          )}
          <div className="flex justify-end gap-2">
            <Button variant="ghost" disabled={busy} onClick={onCancel}>
              取消
            </Button>
            <Button
              variant="primary"
              disabled={busy}
              onClick={() => {
                onSubmit(preview.metadata, mode);
              }}
            >
              {mode === "replace_all"
                ? "确认替换全部页面单元"
                : "确认只填空白页"}
            </Button>
          </div>
        </>
      )}
    </div>
  );
}
