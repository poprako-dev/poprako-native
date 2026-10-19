import { BookOpen, Clock, Languages, SquareCheckBig, Tag } from "lucide-react";
import type { ReactElement } from "react";
import { ProgressBars } from "@/route/home/business/component/ProgressBars";

type ChapterItemProps = {
  title: string;
  activityAt: number;
  pageCount: number;
  unitCount: number;
  translatedCount: number;
  confirmedCount: number;
  onOpen: () => void;
};

const formatter = new Intl.DateTimeFormat("zh-CN", {
  month: "2-digit",
  day: "2-digit",
});
export function ChapterItem(props: ChapterItemProps): ReactElement {
  return (
    <button
      type="button"
      onClick={props.onOpen}
      className="flex min-h-12 min-w-0 flex-1 items-center gap-3 rounded-md py-2 pr-3 text-left focus-visible:outline-2 focus-visible:outline-ring"
    >
      <div className="flex min-w-0 flex-1 flex-wrap items-center gap-x-4 gap-y-2 text-xs text-muted-foreground tabular-nums">
        <span className="max-w-full truncate text-base font-medium text-foreground">
          {props.title}
        </span>
        <span className="flex items-center gap-1.5" title="最近活动">
          <Clock className="size-3.5 stroke-[1.5]" aria-hidden="true" />
          <time dateTime={new Date(props.activityAt).toISOString()}>
            {formatter.format(props.activityAt)}
          </time>
        </span>
        <span className="flex items-center gap-1.5" title="总页数">
          <BookOpen className="size-3.5 stroke-[1.5]" aria-hidden="true" />
          {props.pageCount} 页
        </span>
        <span className="hidden items-center gap-1.5 sm:flex" title="总单元数">
          <Tag className="size-3.5 stroke-[1.5]" aria-hidden="true" />
          {props.unitCount}
        </span>
        <span className="flex items-center gap-1.5" title="已翻译">
          <Languages className="size-3.5 stroke-[1.5]" aria-hidden="true" />
          {props.translatedCount}
        </span>
        <span className="flex items-center gap-1.5" title="已确认校对">
          <SquareCheckBig
            className="size-3.5 stroke-[1.5]"
            aria-hidden="true"
          />
          {props.confirmedCount}
        </span>
      </div>
      <ProgressBars
        total={props.unitCount}
        translated={props.translatedCount}
        confirmed={props.confirmedCount}
      />
    </button>
  );
}
