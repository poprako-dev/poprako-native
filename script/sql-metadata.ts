import { cp, mkdir, mkdtemp, readFile, readdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { run } from "./process.ts";

const temporary = await mkdtemp(join(tmpdir(), "poprako-sql-"));
const generated = join(temporary, "metadata");
const database = join(temporary, "schema.sqlite3");
const checked = resolve("src-tauri/.sqlx");
const check = process.argv.includes("--check");
try {
  await mkdir(generated);
  await run("cargo", [
    "run",
    "--manifest-path",
    "src-tauri/Cargo.toml",
    "--locked",
    "-p",
    "poprako-sql-prepare",
    "--",
    database,
  ]);
  // Recompile only this application so every macro emits fresh metadata, including unchanged SQL.
  await run("cargo", [
    "clean",
    "--manifest-path",
    "src-tauri/Cargo.toml",
    "-p",
    "poprako-native",
  ]);
  await run(
    "cargo",
    [
      "check",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "--locked",
      "--all-targets",
    ],
    process.cwd(),
    {
      ...process.env,
      SQLX_OFFLINE: "false",
      DATABASE_URL: `sqlite://${database}`,
      SQLX_OFFLINE_DIR: generated,
    },
  );
  const names = (await readdir(generated))
    .filter((name) => name.endsWith(".json"))
    .sort();
  if (names.length === 0) throw new Error("SQLx produced no query metadata");
  if (check) {
    const existing = (await readdir(checked))
      .filter((name) => name.endsWith(".json"))
      .sort();
    if (JSON.stringify(names) !== JSON.stringify(existing))
      throw new Error("SQL metadata file set is stale");
    for (const name of names) {
      if (
        (await readFile(join(generated, name), "utf8")) !==
        (await readFile(join(checked, name), "utf8"))
      ) {
        throw new Error(`SQL metadata is stale: ${name}`);
      }
    }
  }
  if (!check) {
    await rm(checked, { recursive: true, force: true });
    await cp(generated, checked, { recursive: true });
  }
} finally {
  await rm(temporary, { recursive: true, force: true });
}
