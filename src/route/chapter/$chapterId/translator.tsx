import { createFileRoute } from "@tanstack/react-router";
import type { ReactElement } from "react";
import { TranslatorPage } from "./business/translator/TranslatorPage";

export const Route = createFileRoute("/chapter/$chapterId/translator")({
  component: Translator,
});

function Translator(): ReactElement {
  const { chapterId } = Route.useParams();
  return <TranslatorPage chapterId={chapterId} />;
}
