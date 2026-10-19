import { Clock, Star } from "lucide-react";
import type { ReactElement, ReactNode } from "react";

type PageItemProps = {
  index: number;
  total: number;
  translated: number;
  confirmed: number;
  flagged: number;
  recent: boolean;
  image: ReactNode;
  onOpen: () => void;
};

export function PageItem(props: PageItemProps): ReactElement {
  const translated =
    props.total === 0 ? 0 : (props.translated / props.total) * 100;
  const confirmed =
    props.total === 0 ? 0 : (props.confirmed / props.total) * 100;
  return (
    <button
      type="button"
      onClick={props.onOpen}
      aria-label={`预览第 ${String(props.index + 1)} 页，已翻译 ${String(Math.round(translated))}%，已确认校对 ${String(Math.round(confirmed))}%，关注 ${String(props.flagged)} 个`}
      className="group min-w-0 rounded text-left outline-offset-4 focus-visible:outline-2 focus-visible:outline-ring"
    >
      <div
        className={`relative aspect-[3/4] overflow-hidden rounded border border-border bg-card p-1 ring-primary/40 transition-colors group-hover:border-primary/50 ${props.recent ? "ring-2" : ""}`}
      >
        <div className="h-full overflow-hidden rounded-sm">{props.image}</div>
        {props.recent && (
          <span
            title="上次阅读"
            aria-label="上次阅读"
            className="absolute top-3 left-3 rounded bg-card/95 p-1 text-primary"
          >
            <Clock size={12} aria-hidden="true" />
          </span>
        )}
        <div
          className="absolute inset-x-3 bottom-3 flex flex-col gap-0.5"
          role="img"
          aria-label={`翻译 ${String(Math.round(translated))}%，校对确认 ${String(Math.round(confirmed))}%`}
        >
          <div className="h-1 overflow-hidden rounded-full bg-border">
            <div
              className="h-full rounded-full bg-state-secondary"
              style={{ width: `${String(translated)}%` }}
            />
          </div>
          <div className="h-1 overflow-hidden rounded-full bg-border">
            <div
              className="h-full rounded-full bg-state-edited"
              style={{ width: `${String(confirmed)}%` }}
            />
          </div>
        </div>
      </div>
      <div className="mt-1.5 flex items-center justify-between gap-2 px-0.5 text-xs text-muted-foreground tabular-nums">
        <span>P{props.index + 1}</span>
        {props.flagged > 0 && (
          <span
            title={`关注 ${String(props.flagged)} 个`}
            className="flex items-center gap-1 text-[11px]"
          >
            <Star size={11} aria-hidden="true" />
            {props.flagged}
          </span>
        )}
      </div>
    </button>
  );
}
