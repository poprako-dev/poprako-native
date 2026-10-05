import type { ReactElement } from "react";
import { useCallback } from "react";
import { commands, unwrap } from "@/bridge";
import type { ComicDetail } from "@/bridge/generated/bindings";
import { ComicContent } from "@/route/comic/$comicId/business/component/ComicContent";
import { StatePanel } from "@/shared/component/StatePanel";
import { useResource } from "@/shared/hook/use-resource";

type ComicPageProps = { comicId: string };

export function ComicPage({ comicId }: ComicPageProps): ReactElement {
  const load = useCallback(
    async (): Promise<ComicDetail> =>
      unwrap(await commands.getComicDetail(comicId)),
    [comicId],
  );
  const resource = useResource(load);
  if (resource.state.status === "loading") {
    return <StatePanel state="loading" message="正在打开项目…" />;
  }
  if (resource.state.status === "error") {
    return (
      <StatePanel
        state="error"
        message="项目无法读取，请重试或返回项目列表。"
        onRetry={resource.reload}
      />
    );
  }
  return (
    <ComicContent detail={resource.state.value} onReload={resource.reload} />
  );
}
