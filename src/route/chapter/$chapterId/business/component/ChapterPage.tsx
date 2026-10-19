import type { ReactElement } from "react";
import { useCallback } from "react";
import { commands, unwrap } from "@/bridge";
import type { ChapterDetail } from "@/bridge/generated/bindings";
import { ChapterContent } from "@/route/chapter/$chapterId/business/component/ChapterContent";
import { StatePanel } from "@/shared/component/StatePanel";
import { useResource } from "@/shared/hook/use-resource";

type ChapterPageProps = { chapterId: string };

export function ChapterPage({ chapterId }: ChapterPageProps): ReactElement {
  const load = useCallback(
    async (): Promise<ChapterDetail> =>
      unwrap(await commands.getChapterDetail(chapterId)),
    [chapterId],
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
    <ChapterContent detail={resource.state.value} onReload={resource.reload} />
  );
}
