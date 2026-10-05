import { createFileRoute } from "@tanstack/react-router";
import type { ReactElement } from "react";
import { TranslatorPage } from "./business/translator/TranslatorPage";

export const Route = createFileRoute("/comic/$comicId/translator")({
  component: Translator,
});

function Translator(): ReactElement {
  const { comicId } = Route.useParams();
  return <TranslatorPage comicId={comicId} />;
}
