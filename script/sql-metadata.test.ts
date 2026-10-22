// @vitest-environment node
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { afterEach, expect, it, vi } from "vitest";
import { run } from "./process.ts";

vi.mock("./process.ts", () => ({ run: vi.fn() }));

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllEnvs();
  vi.resetModules();
});

it.each([false, true])(
  "preserves the running desktop binary while refreshing SQL metadata (custom target: %s)",
  async (customTarget) => {
    const root = await mkdtemp(join(tmpdir(), "poprako-sql-isolation-"));
    const target = join(
      root,
      customTarget ? "custom-target" : "src-tauri/target",
    );
    const executable = join(target, "debug/poprako-native");
    const metadata = join(root, "src-tauri/.sqlx");
    const originalArgs = process.argv;
    try {
      await mkdir(join(target, "debug"), { recursive: true });
      await mkdir(metadata, { recursive: true });
      await writeFile(executable, "running desktop binary");
      await writeFile(join(metadata, "query-test.json"), "{}");
      vi.spyOn(process, "cwd").mockReturnValue(root);
      vi.stubEnv("CARGO_TARGET_DIR", customTarget ? target : undefined);
      process.argv = [...originalArgs, "--check"];
      vi.mocked(run).mockImplementation(
        async (command, args, _cwd, env = process.env) => {
          expect(command).toBe("cargo");
          if (args[0] === "clean") {
            // Cargo clean -p removes this package's running executable as well as its library.
            const output = resolve(
              env["CARGO_TARGET_DIR"] ?? join(root, "src-tauri/target"),
            );
            await rm(join(output, "debug/poprako-native"), { force: true });
          }
          if (args[0] === "check") {
            const destination = env["SQLX_OFFLINE_DIR"];
            if (destination === undefined)
              throw new Error("Missing SQLx output directory");
            await writeFile(join(destination, "query-test.json"), "{}");
          }
        },
      );
      await import("./sql-metadata.ts");
      expect(await readFile(executable, "utf8")).toBe("running desktop binary");
      expect(await readFile(join(metadata, "query-test.json"), "utf8")).toBe(
        "{}",
      );
    } finally {
      process.argv = originalArgs;
      vi.restoreAllMocks();
      await rm(root, { recursive: true, force: true });
    }
  },
);
