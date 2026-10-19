import type { ReactElement } from "react";
import { useCallback, useState } from "react";
import { commands, unwrap } from "@/bridge";
import type { ComicInfo } from "@/bridge/generated/bindings";
import { HomeContent } from "@/route/home/business/component/HomeContent";
import { StatePanel } from "@/shared/component/StatePanel";
import { useResource } from "@/shared/hook/use-resource";

type HomeData = { comics: ComicInfo[]; latest: ComicInfo | null };

export function HomePage(): ReactElement {
  const [offset, setOffset] = useState(0);
  const [expanded, setExpanded] = useState<ReadonlySet<string>>(
    () => new Set(),
  );
  function toggle(id: string): void {
    setExpanded((previous) => {
      const next = new Set(previous);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }
  const load = useCallback(async (): Promise<HomeData> => {
    const comics = unwrap(await commands.listComicInfos(offset, 50));
    const first =
      offset === 0 ? comics : unwrap(await commands.listComicInfos(0, 1));
    return { comics, latest: first[0] ?? null };
  }, [offset]);
  const resource = useResource(load);
  if (resource.state.status === "loading") {
    return <StatePanel state="loading" message="正在打开本地资料库…" />;
  }
  if (resource.state.status === "error") {
    return (
      <StatePanel
        state="error"
        message="漫画列表无法读取，请重试。"
        onRetry={resource.reload}
      />
    );
  }
  return (
    <HomeContent
      expanded={expanded}
      onToggle={toggle}
      comics={resource.state.value.comics}
      latest={resource.state.value.latest}
      onReload={resource.reload}
      offset={offset}
      onOffsetChange={setOffset}
    />
  );
}
