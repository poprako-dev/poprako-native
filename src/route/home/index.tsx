import { createFileRoute } from "@tanstack/react-router";
import { HomePage } from "@/route/home/business/component/HomePage";

export const Route = createFileRoute("/home/")({ component: HomePage });
