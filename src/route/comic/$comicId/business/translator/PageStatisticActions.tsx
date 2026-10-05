import {
  ChartNoAxesGantt,
  CheckCheck,
  CircleArrowRight,
  FileType,
  Plus,
} from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import type { ReactElement } from "react";
import { IconButton } from "@/shared/component/IconButton";
import { useDismiss } from "@/shared/hook/use-dismiss";
import { nextEditedPage } from "./page-statistic";
import type { PageStatistic } from "./page-statistic";

type PageStatisticActionsProps = {
  pages: PageStatistic[];
  currentId: string;
  locked: boolean;
  onPage: (id: string) => void;
};

export function PageStatisticActions({
  pages,
  currentId,
  locked,
  onPage,
}: PageStatisticActionsProps): ReactElement {
  const [open, setOpen] = useState(false);
  const [notice, setNotice] = useState("");
  const ref = useRef<HTMLDivElement>(null);
  const selected = useRef<HTMLButtonElement>(null);
  const close = useCallback(() => {
    setOpen(false);
  }, []);
  useEffect(() => {
    if (!notice) return;
    const timer = setTimeout(() => {
      setNotice("");
    }, 2500);
    return () => {
      clearTimeout(timer);
    };
  }, [notice]);
  useDismiss(open, ref, close);
  useEffect(() => {
    if (open) selected.current?.scrollIntoView({ block: "nearest" });
  }, [open, currentId]);
  const limit = Math.max(
    0,
    ...pages.map((page) => page.translated + page.appended),
  );
  const scale = Math.max(1, limit);
  return (
    <div ref={ref} className="relative flex items-center gap-2">
      <IconButton
        label="页面单元统计"
        className="translator-floating-button"
        disabled={locked}
        aria-expanded={open}
        onClick={() => {
          setOpen(!open);
        }}
      >
        <ChartNoAxesGantt size={16} strokeWidth={1.8} aria-hidden="true" />
      </IconButton>
      {open && (
        <div
          role="region"
          aria-label="页面统计详情"
          className="absolute right-0 bottom-full z-50 mb-1.5 max-h-96 w-76 max-w-[calc(100vw-16px)] overflow-y-auto rounded-sm border border-black/5 bg-white/95 pb-1 font-mono shadow-2xl backdrop-blur-md"
        >
          <div className="sticky top-0 z-10 grid h-7 grid-cols-[1.75rem_minmax(0,1fr)_4.5rem] items-center gap-x-2 bg-stone-50 px-2.5 text-[10px] text-stone-400">
            <span />
            <div className="flex justify-between">
              <span>0</span>
              <span>{limit}</span>
            </div>
            <div className="grid grid-cols-3 justify-items-center">
              <FileType
                size={12}
                className="text-orange-400"
                aria-label="翻译"
              />
              <CheckCheck
                size={12}
                className="text-pink-400"
                aria-label="编辑"
              />
              <Plus size={12} className="text-brand-leaf" aria-label="追加" />
            </div>
          </div>
          {pages.map((page) => (
            <button
              type="button"
              key={page.id}
              ref={page.id === currentId ? selected : undefined}
              aria-current={page.id === currentId ? "page" : undefined}
              aria-label={`第 ${String(page.index + 1)} 页，翻译 ${String(page.translated)}，编辑 ${String(page.edited)}，追加 ${String(page.appended)}`}
              onClick={() => {
                close();
                onPage(page.id);
              }}
              className={`grid h-7 w-full grid-cols-[1.75rem_minmax(0,1fr)_4.5rem] items-center gap-x-2 px-2.5 text-[11px] transition-colors hover:bg-stone-100 ${page.id === currentId ? "bg-stone-100 font-semibold text-stone-800" : "text-stone-500"}`}
            >
              <span className="text-left">P{page.index + 1}</span>
              <span
                aria-hidden="true"
                className="relative flex h-full items-center border-x border-stone-200/50"
              >
                <span className="absolute inset-y-0 left-1/2 border-l border-stone-200/35" />
                <span className="relative h-2 w-full">
                  <span
                    className="absolute top-0 left-0 h-1 rounded-[1px] bg-orange-400"
                    style={{
                      width: `${String((page.translated / scale) * 100)}%`,
                    }}
                  />
                  <span
                    className="absolute top-0 h-1 rounded-[1px] bg-brand-leaf"
                    style={{
                      left: `${String((page.translated / scale) * 100)}%`,
                      width: `${String((page.appended / scale) * 100)}%`,
                    }}
                  />
                  <span
                    className="absolute bottom-0 left-0 h-0.5 rounded-[1px] bg-pink-400"
                    style={{ width: `${String((page.edited / scale) * 100)}%` }}
                  />
                </span>
              </span>
              <span className="grid grid-cols-3 text-center">
                {[page.translated, page.edited, page.appended].map(
                  (count, index) => (
                    <span
                      key={index}
                      className={
                        count === 0 ? "text-stone-300" : "text-stone-600"
                      }
                    >
                      {count}
                    </span>
                  ),
                )}
              </span>
            </button>
          ))}
        </div>
      )}
      <IconButton
        label="前进到下一个修改"
        disabled={locked || open}
        className="translator-floating-button"
        onClick={() => {
          const next = nextEditedPage(pages, currentId);
          if (next) {
            setNotice("");
            onPage(next.id);
          } else setNotice("后面没有修改页面");
        }}
      >
        <CircleArrowRight size={20} strokeWidth={2.25} aria-hidden="true" />
      </IconButton>
      {notice !== "" && (
        <span
          role="status"
          className="absolute right-0 bottom-full mb-2 whitespace-nowrap rounded bg-white/95 px-2 py-1 text-xs text-stone-500"
        >
          {notice}
        </span>
      )}
    </div>
  );
}
