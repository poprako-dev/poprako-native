import {
  ArrowLeft,
  BookOpen,
  Download,
  Languages,
  PencilLine,
  Play,
  SquareCheckBig,
  Tag,
} from "lucide-react";
import type { ReactElement, ReactNode } from "react";
import { Button } from "@/shared/component/Button";

type ComicSidebarProps = {
  title: string;
  subtitle: string;
  author: string;
  pageCount: number;
  unitCount: number;
  translatedCount: number;
  confirmedCount: number;
  cover: ReactNode;
  busy: boolean;
  onBack: () => void;
  onContinue: () => void;
  onExport: () => void;
  onEdit: () => void;
};

export function ComicSidebar(props: ComicSidebarProps): ReactElement {
  const stats = [
    { label: "总页数", value: props.pageCount, icon: BookOpen },
    { label: "总单元数", value: props.unitCount, icon: Tag },
    { label: "已翻译", value: props.translatedCount, icon: Languages },
    { label: "已确认校对", value: props.confirmedCount, icon: SquareCheckBig },
  ];
  return (
    <>
      <div className="flex h-7 shrink-0 items-center">
        <Button
          variant="ghost"
          className="size-7 p-0"
          onClick={props.onBack}
          aria-label="返回项目列表"
        >
          <ArrowLeft className="size-4" aria-hidden="true" />
        </Button>
      </div>
      <div className="min-h-0 max-h-48 flex-1 overflow-hidden rounded">
        {props.cover}
      </div>
      <div className="min-w-0 shrink-0">
        <h1
          className="line-clamp-2 text-sm leading-5 font-medium wrap-anywhere"
          title={props.title}
        >
          {props.title}
        </h1>
        {props.subtitle !== "" && (
          <p
            className="mt-1 truncate text-xs text-muted-foreground"
            title={props.subtitle}
          >
            {props.subtitle}
          </p>
        )}
        {props.author !== "" && (
          <p
            className="mt-1 truncate text-xs text-muted-foreground"
            title={props.author}
          >
            {props.author}
          </p>
        )}
      </div>
      <dl className="shrink-0 divide-y divide-border/70">
        {stats.map((stat) => (
          <div
            key={stat.label}
            className="flex h-8 items-center justify-between gap-2"
          >
            <dt className="flex items-center gap-2 text-xs text-muted-foreground">
              <stat.icon className="size-3.5 stroke-[1.5]" aria-hidden="true" />
              {stat.label}
            </dt>
            <dd className="text-sm tabular-nums">{stat.value}</dd>
          </div>
        ))}
      </dl>
      <div className="flex shrink-0 flex-col gap-1 border-t border-border pt-3">
        <Button
          variant="primary"
          onClick={props.onContinue}
          disabled={props.busy || props.pageCount === 0}
        >
          <Play className="size-3.5" aria-hidden="true" />
          继续翻校
        </Button>
        <Button variant="ghost" onClick={props.onExport} disabled={props.busy}>
          <Download className="size-3.5" aria-hidden="true" />
          导出数据
        </Button>
        <Button variant="ghost" onClick={props.onEdit} disabled={props.busy}>
          <PencilLine className="size-3.5" aria-hidden="true" />
          编辑信息
        </Button>
      </div>
    </>
  );
}
