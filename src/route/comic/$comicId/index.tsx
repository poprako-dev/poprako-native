import { createFileRoute } from "@tanstack/react-router";
import type { ReactElement } from "react";
import { ComicPage } from "@/route/comic/$comicId/business/component/ComicPage";

export const Route = createFileRoute("/comic/$comicId/")({ component: Comic });

function Comic(): ReactElement {
  const { comicId } = Route.useParams();
  return <ComicPage key={comicId} comicId={comicId} />;
}
