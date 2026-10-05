import { readFileSync } from "node:fs";

export function assertDenoVersion(
  actual: string | undefined,
  required: string,
): void {
  if (actual !== required) {
    throw new Error(
      `This project requires Deno ${required}; current runtime is ${actual ?? "not Deno"}. Install the pinned version before running project tasks.`,
    );
  }
}

export function checkDenoVersion(): void {
  const required = readFileSync(
    new URL("../.deno-version", import.meta.url),
    "utf8",
  ).trim();
  assertDenoVersion(process.versions["deno"], required);
}
