import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import { routeConfig } from "./route-config.ts";

it("keeps route staging files inside the requested generation root", () => {
  const root = resolve(tmpdir(), "poprako-route-check");
  expect(routeConfig(root).tmpDir).toBe(resolve(root, ".tanstack/tmp"));
});
