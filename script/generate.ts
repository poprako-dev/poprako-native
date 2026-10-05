import { cp, mkdtemp, mkdir, readFile, readdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { Generator } from "@tanstack/router-generator";
import { run } from "./process.ts";
import { routeConfig } from "./route-config.ts";

async function compareFile(left: string, right: string): Promise<void> {
  if ((await readFile(left, "utf8")) !== (await readFile(right, "utf8"))) {
    throw new Error(`Generated output is stale: ${left}`);
  }
}

async function compareDirectory(left: string, right: string): Promise<void> {
  const entries = await readdir(left, { withFileTypes: true });
  for (const entry of entries) {
    const a = join(left, entry.name);
    const b = join(right, entry.name);
    if (entry.isDirectory()) await compareDirectory(a, b);
    if (entry.isFile()) await compareFile(a, b);
  }
}

const check = process.argv.includes("--check");
const root = process.cwd();
const temporary = await mkdtemp(join(tmpdir(), "poprako-generate-"));
try {
  await run("deno", [
    "run",
    "-A",
    "script/sql-metadata.ts",
    ...(check ? ["--check"] : []),
  ]);
  const destination = check
    ? join(temporary, "bindings.ts")
    : resolve("src/bridge/generated/bindings.ts");
  await run("cargo", [
    "run",
    "--manifest-path",
    "src-tauri/Cargo.toml",
    "--locked",
    "--bin",
    "export_bindings",
    "--",
    destination,
  ]);
  if (check) await compareFile("src/bridge/generated/bindings.ts", destination);
  if (check) {
    await mkdir(join(temporary, "src"), { recursive: true });
    await cp("src/route", join(temporary, "src/route"), { recursive: true });
    await new Generator({
      config: routeConfig(temporary),
      root: temporary,
    }).run();
    await compareFile(
      "src/route-tree.gen.ts",
      join(temporary, "src/route-tree.gen.ts"),
    );
    await compareDirectory("src/route", join(temporary, "src/route"));
  }
  if (!check) await new Generator({ config: routeConfig(root), root }).run();
} finally {
  await rm(temporary, { recursive: true, force: true });
}
