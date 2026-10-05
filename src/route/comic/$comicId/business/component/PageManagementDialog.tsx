import { ArrowDown, ArrowUp } from "lucide-react";
import { useState } from "react";
import type { ReactElement } from "react";
import { commands, NativeCommandError, unwrap } from "@/bridge";
import type {
  ComicDetail,
  Page,
  RemovePages,
  ReorderPages,
} from "@/bridge/generated/bindings";
import { ManagedImage } from "@/route/business/component/ManagedImage";
import { pageBaseline } from "@/route/comic/$comicId/business/page-baseline";
import { Button } from "@/shared/component/Button";
import { Dialog } from "@/shared/component/Dialog";

type PageManagementDialogProps = {
  detail: ComicDetail;
  onClose: () => void;
  onComplete: () => void;
  onReplace: (page: Page) => void;
};
type Mutation =
  | { kind: "reorder"; input: ReorderPages }
  | { kind: "remove"; input: RemovePages };

export function PageManagementDialog({
  detail,
  onClose,
  onComplete,
  onReplace,
}: PageManagementDialogProps): ReactElement {
  const [order, setOrder] = useState(detail.pages.map((info) => info.page.id));
  const [selected, setSelected] = useState<string[]>([]);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [busy, setBusy] = useState(false);
  const [pending, setPending] = useState<Mutation | null>(null);
  const [error, setError] = useState("");
  const changed = order.some(
    (id, index) => detail.pages[index]?.page.id !== id,
  );
  const locked = busy || pending !== null;
  const baseline = detail.pages.map((info) => pageBaseline(info.page));

  function move(index: number, direction: number): void {
    const next = [...order];
    const source = next[index];
    const destination = next[index + direction];
    if (source === undefined || destination === undefined) {
      return;
    }
    next[index] = destination;
    next[index + direction] = source;
    setOrder(next);
  }

  async function submit(mutation: Mutation): Promise<void> {
    setBusy(true);
    setError("");
    try {
      if (mutation.kind === "reorder") {
        unwrap(await commands.reorderPages(mutation.input));
      }
      if (mutation.kind === "remove") {
        unwrap(await commands.removePages(mutation.input));
      }
      setPending(null);
      onComplete();
    } catch (cause) {
      if (
        !(cause instanceof NativeCommandError) ||
        cause.detail.recovery === "wait_for_confirmation"
      ) {
        setPending(mutation);
      }
      setError(
        cause instanceof NativeCommandError
          ? cause.detail.message
          : "操作结果未确认，请保留当前窗口并核实。",
      );
    } finally {
      setBusy(false);
    }
  }

  function run(mutation: Mutation): void {
    submit(mutation).catch(() => {
      setError("操作结果未确认，请保留当前窗口。");
    });
  }

  return (
    <Dialog
      open={true}
      onOpenChange={(open) => {
        if (!open && !locked) {
          onClose();
        }
      }}
      title="管理页面"
      description=""
      className="max-w-3xl"
    >
      <div className="flex items-center justify-between gap-2">
        <label className="flex items-center gap-2 text-xs">
          <input
            type="checkbox"
            disabled={locked}
            checked={selected.length === order.length && order.length > 0}
            onChange={(event) => {
              setSelected(event.currentTarget.checked ? [...order] : []);
            }}
          />
          全选页面
        </label>
        <span className="text-xs text-muted-foreground">
          已选 {selected.length} / {order.length}
        </span>
      </div>
      <div className="max-h-[50dvh] divide-y divide-border overflow-y-auto">
        {order.map((id, index) => {
          const info = detail.pages.find((item) => item.page.id === id);
          if (info === undefined) {
            return null;
          }
          return (
            <div key={id} className="flex items-center gap-3 py-2">
              <input
                type="checkbox"
                aria-label={`选择第 ${String(index + 1)} 页`}
                disabled={locked}
                checked={selected.includes(id)}
                onChange={(event) => {
                  setSelected(
                    event.currentTarget.checked
                      ? [...selected, id]
                      : selected.filter((value) => value !== id),
                  );
                }}
              />
              <div className="h-16 w-12 shrink-0 overflow-hidden rounded">
                <ManagedImage
                  comicId={detail.comic.id}
                  pageId={id}
                  reference={info.page.image.reference}
                  kind="thumbnail"
                  alt={`第 ${String(index + 1)} 页`}
                  interactive={false}
                />
              </div>
              <div className="min-w-0 flex-1">
                <p className="text-sm">P{index + 1}</p>
                <p className="truncate text-xs text-muted-foreground">
                  {info.page.image.original_name} · {info.unit_count} 个单元
                </p>
              </div>
              <Button
                variant="ghost"
                disabled={locked || changed}
                onClick={() => {
                  onReplace(info.page);
                }}
              >
                换图／修复
              </Button>
              <Button
                variant="ghost"
                aria-label={`上移第 ${String(index + 1)} 页`}
                disabled={locked || index === 0}
                onClick={() => {
                  move(index, -1);
                }}
              >
                <ArrowUp className="size-4" aria-hidden="true" />
              </Button>
              <Button
                variant="ghost"
                aria-label={`下移第 ${String(index + 1)} 页`}
                disabled={locked || index === order.length - 1}
                onClick={() => {
                  move(index, 1);
                }}
              >
                <ArrowDown className="size-4" aria-hidden="true" />
              </Button>
            </div>
          );
        })}
      </div>
      {error !== "" && (
        <p role="alert" className="text-xs text-destructive">
          {error}
        </p>
      )}
      {confirmDelete && (
        <div className="rounded border border-destructive/40 p-3 text-sm">
          <p>
            确定删除所选 {selected.length} 页及其全部单元？此操作无法撤销。
            {selected.length === order.length ? "删除后会保留空项目资料。" : ""}
          </p>
          <div className="mt-3 flex justify-end gap-2">
            <Button
              variant="ghost"
              disabled={locked}
              onClick={() => {
                setConfirmDelete(false);
              }}
            >
              取消删除
            </Button>
            <Button
              variant="danger"
              disabled={locked || selected.length === 0}
              onClick={() => {
                run({
                  kind: "remove",
                  input: {
                    comic_id: detail.comic.id,
                    baseline,
                    page_ids: selected,
                  },
                });
              }}
            >
              确认删除页面
            </Button>
          </div>
        </div>
      )}
      <div className="flex flex-wrap justify-end gap-2">
        <Button variant="ghost" disabled={locked} onClick={onClose}>
          取消
        </Button>
        <Button
          variant="danger"
          disabled={locked || selected.length === 0 || changed}
          onClick={() => {
            setConfirmDelete(true);
          }}
        >
          删除所选
        </Button>
        <Button
          variant="primary"
          disabled={locked || !changed}
          onClick={() => {
            run({
              kind: "reorder",
              input: { comic_id: detail.comic.id, baseline, page_ids: order },
            });
          }}
        >
          保存页序
        </Button>
        {pending !== null && (
          <Button
            variant="primary"
            disabled={busy}
            onClick={() => {
              run(pending);
            }}
          >
            核实原操作结果
          </Button>
        )}
      </div>
    </Dialog>
  );
}
