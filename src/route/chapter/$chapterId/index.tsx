import { createFileRoute } from "@tanstack/react-router";
import type { ReactElement } from "react";
import { ChapterPage } from "@/route/chapter/$chapterId/business/component/ChapterPage";

export const Route = createFileRoute("/chapter/$chapterId/")({
  component: Chapter,
});

function Chapter(): ReactElement {
  const { chapterId } = Route.useParams();
  return <ChapterPage key={chapterId} chapterId={chapterId} />;
}
