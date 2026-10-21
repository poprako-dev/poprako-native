import { useEffect, useRef } from "react";
import type { ReactElement, ReactNode } from "react";
import { BookOpenText, LoaderCircle } from "lucide-react";
import { Button } from "@/shared/component/Button";

type Props = {
  count: number;
  loading: boolean;
  more: boolean;
  error: string;
  label: string;
  empty: string;
  onNext: () => void;
  children: ReactNode;
};

export function TerminologyList(props: Props): ReactElement {
  const scroll = useRef<HTMLDivElement>(null);
  const sentinel = useRef<HTMLDivElement>(null);
  const { onNext, loading, more, error, count } = props;
  useEffect(() => {
    const root = scroll.current;
    const target = sentinel.current;
    if (!root || !target || loading || !more || error !== "") return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting) onNext();
      },
      { root, rootMargin: "72px 0px" },
    );
    observer.observe(target);
    return () => {
      observer.disconnect();
    };
  }, [onNext, loading, more, error, count]);
  return (
    <div
      ref={scroll}
      role="region"
      aria-label={`${props.label}，可滚动`}
      tabIndex={0}
      className="min-h-0 flex-1 overflow-y-auto overscroll-contain"
    >
      <div role="list" aria-label={props.label}>
        {props.children}
      </div>
      {count === 0 && !loading && error === "" && (
        <div className="flex min-h-28 flex-col items-center justify-center gap-2 text-xs text-muted-foreground">
          <BookOpenText size={18} />
          {props.empty}
        </div>
      )}
      <div
        ref={sentinel}
        className="flex min-h-8 items-center justify-center p-1 text-xs text-muted-foreground"
      >
        {loading ? (
          <LoaderCircle
            className="animate-spin"
            size={18}
            aria-label="正在加载术语数据"
          />
        ) : error !== "" ? (
          <div className="w-full text-center">
            <p role="alert">{error}</p>
            <Button className="w-full" variant="ghost" onClick={onNext}>
              重试
            </Button>
          </div>
        ) : more ? (
          <Button variant="ghost" onClick={onNext}>
            加载更多
          </Button>
        ) : count > 0 ? (
          "已显示全部"
        ) : null}
      </div>
    </div>
  );
}
