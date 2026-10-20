import { readFileSync } from "node:fs";

export function assertNodeVersion(
  actual: string | undefined,
  required: string,
): void {
  if (actual !== required) {
    throw new Error(
      `This project requires Node.js ${required}; current runtime is ${actual ?? "not Node.js"}. Install the pinned version before running project tasks.`,
    );
  }
}

export function checkNodeVersion(): void {
  const required = readFileSync(
    new URL("../.node-version", import.meta.url),
    "utf8",
  ).trim();
  assertNodeVersion(process.versions.node, required);
}
