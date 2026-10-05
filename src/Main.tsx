import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { RouterProvider } from "@tanstack/react-router";
import { router } from "@/application/router";
import "@/application/style.css";

const rootElement = document.getElementById("root");

if (rootElement === null) {
  throw new Error("Application root element is missing.");
}

createRoot(rootElement).render(
  <StrictMode>
    <RouterProvider router={router} />
  </StrictMode>,
);
