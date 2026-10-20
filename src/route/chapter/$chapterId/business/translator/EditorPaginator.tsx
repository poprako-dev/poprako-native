import { ChevronLeft, ChevronRight, PencilLine, Star } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import type { ReactElement } from "react";
import { useDismiss } from "@/shared/hook/use-dismiss";
import type { PageStatistic } from "./page-statistic";

type EditorPaginatorProps = {
  pages: PageStatistic[];
  currentId: string;
  locked: boolean;
  onPage: (id: string) => void;
};

export function EditorPaginator({
  pages,
  currentId,
  locked,
  onPage,
}: EditorPaginatorProps): ReactElement {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const selected = useRef<HTMLButtonElement>(null);
  const close = useCallback(() => {
    setOpen(false);
  }, []);
  useDismiss(open, ref, close);
  const index = pages.findIndex((page) => page.id === currentId);
  useEffect(() => {
    if (open) selected.current?.scrollIntoView({ block: "nearest" });
  }, [open, currentId]);
  function move(offset: number): void {
    const page = pages[index + offset];
    if (page && !locked) onPage(page.id);
  }
  return (
    <>
      <div ref={ref} className="relative z-50 inline-block select-none">
        <nav
          aria-label="页面导航"
          className="inline-flex h-8 w-24 items-stretch overflow-hidden rounded-sm bg-white/95 opacity-85 shadow-2xl backdrop-blur-md"
        >
          <button
            type="button"
            aria-label="上一页"
            title="上一页"
            disabled={index <= 0 || locked}
            onClick={() => {
              move(-1);
            }}
            className="flex flex-1 items-center justify-center text-gray-600 transition-colors hover:bg-stone-400/40 disabled:opacity-20 disabled:hover:bg-transparent"
          >
            <ChevronLeft size={14} aria-hidden="true" />
          </button>
          <button
            type="button"
            aria-label="展开页面列表"
            aria-expanded={open}
            aria-controls="editor-page-picker"
            disabled={locked || pages.length === 0}
            onClick={() => {
              setOpen(!open);
            }}
            className="flex w-16 flex-none items-center justify-center gap-0.5 border-x border-gray-100 bg-gray-50/20 text-xs hover:bg-stone-400/30 disabled:opacity-40"
          >
            <span className="font-bold text-gray-900">{index + 1}</span>
            <span className="font-light text-gray-300">/</span>
            <span className="font-semibold text-gray-500">{pages.length}</span>
          </button>
          <button
            type="button"
            aria-label="下一页"
            title="下一页"
            disabled={index >= pages.length - 1 || locked}
            onClick={() => {
              move(1);
            }}
            className="flex flex-1 items-center justify-center text-gray-600 transition-colors hover:bg-stone-400/40 disabled:opacity-20 disabled:hover:bg-transparent"
          >
            <ChevronRight size={14} aria-hidden="true" />
          </button>
        </nav>
        {open && (
          <div
            id="editor-page-picker"
            aria-label="页面列表"
            className="absolute top-full right-0 z-50 mt-1 max-h-60 w-56 max-w-[calc(100vw-1rem)] overflow-y-auto rounded-sm border border-black/5 bg-white/95 shadow-2xl backdrop-blur-md"
          >
            {pages.map((page) => (
              <button
                type="button"
                key={page.id}
                ref={page.id === currentId ? selected : undefined}
                aria-current={page.id === currentId ? "page" : undefined}
                aria-label={`第 ${String(page.index + 1)} 页，单元 ${String(page.total)}，翻译 ${String(page.translated)}，校对 ${String(page.confirmed)}，关注 ${String(page.flagged)}`}
                onClick={() => {
                  onPage(page.id);
                  close();
                }}
                className={`flex w-full items-center justify-between px-3 py-1.5 text-xs transition-colors hover:bg-stone-100 active:bg-stone-200 ${page.id === currentId ? "bg-stone-100" : ""}`}
              >
                <span className="flex items-center gap-1.5">
                  <span
                    aria-hidden="true"
                    className={`size-2 shrink-0 rounded-full ${page.total > 0 && page.confirmed >= page.total ? "bg-brand-leaf" : page.total > 0 && page.translated >= page.total ? "bg-orange-400" : "bg-gray-400"}`}
                  />
                  <span className="font-medium text-stone-700">
                    P{page.index + 1}
                  </span>
                  {page.flagged > 0 && (
                    <Star
                      size={12}
                      fill="currentColor"
                      className="text-status-flag"
                      aria-label={`${String(page.flagged)} 个关注标记`}
                    />
                  )}
                  {page.dirty && (
                    <PencilLine
                      size={12}
                      className="text-stone-400"
                      aria-label="有未保存草稿"
                    />
                  )}
                </span>
                <span className="font-mono text-[11px]">
                  <span className="text-stone-400">{page.total}</span>
                  <span className="mx-px text-stone-300">/</span>
                  <span className="text-orange-400">{page.translated}</span>
                  <span className="mx-px text-stone-300">/</span>
                  <span className="text-pink-400">{page.confirmed}</span>
                </span>
              </button>
            ))}
          </div>
        )}
      </div>
      {open && (
        <button
          type="button"
          aria-label="关闭页面列表"
          onClick={close}
          className="fixed inset-0 z-40 bg-black/25"
        />
      )}
    </>
  );
}
