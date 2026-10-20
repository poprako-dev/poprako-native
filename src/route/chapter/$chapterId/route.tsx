import { createFileRoute, Outlet } from "@tanstack/react-router";

export const Route = createFileRoute("/chapter/$chapterId")({
  component: Outlet,
});
