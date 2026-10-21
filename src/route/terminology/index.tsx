import { createFileRoute } from "@tanstack/react-router";
import { TerminologyPage } from "./business/TerminologyPage";

export const Route = createFileRoute("/terminology/")({
  component: TerminologyPage,
});
