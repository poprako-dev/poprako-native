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
        message="项目列表无法读取，请重试。"
        onRetry={resource.reload}
      />
    );
  }
  return (
    <HomeContent
      comics={resource.state.value.comics}
      latest={resource.state.value.latest}
      onReload={resource.reload}
      offset={offset}
      onOffsetChange={setOffset}
    />
  );
}
