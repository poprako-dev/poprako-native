import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { join } from "node:path";

export function record(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("Expected a JSON object.");
  }
  return Object.fromEntries(Object.entries(value));
}

export function stableVersion(value: unknown): string {
  if (
    typeof value !== "string" ||
    !/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(value)
  ) {
    throw new Error("Expected a stable version such as 1.0.0.");
  }
  return value;
}

export async function readReleaseVersion(root: string): Promise<string> {
  const packageJson = record(
    JSON.parse(await readFile(join(root, "package.json"), "utf8")),
  );
  const version = stableVersion(packageJson["version"]);
  const tauri = record(
    JSON.parse(await readFile(join(root, "src-tauri/tauri.conf.json"), "utf8")),
  );
  const manifest = record(
    JSON.parse(
      await readFile(join(root, ".release-please-manifest.json"), "utf8"),
    ),
  );
  const metadata = record(
    JSON.parse(
      execFileSync(
        "cargo",
        [
          "metadata",
          "--manifest-path",
          "src-tauri/Cargo.toml",
          "--format-version",
          "1",
          "--no-deps",
          "--locked",
        ],
        { cwd: root, encoding: "utf8" },
      ),
    ),
  );
  const packages: unknown = metadata["packages"];
  if (!Array.isArray(packages))
    throw new Error("Cargo metadata has no packages.");
  const application = packages
    .map((value: unknown) => record(value))
    .find((value) => value["name"] === "poprako-native");
  const lock = await readFile(join(root, "src-tauri/Cargo.lock"), "utf8");
  const lockedApplication = lock
    .split("[[package]]")
    .find((entry) => /^name = "poprako-native"$/m.test(entry));
  const lockedVersion = lockedApplication?.match(/^version = "([^"]+)"$/m)?.[1];
  if (
    tauri["version"] !== version ||
    application?.["version"] !== version ||
    lockedVersion !== version ||
    (manifest["."] !== undefined && manifest["."] !== version)
  ) {
    throw new Error(
      "Package, Tauri, Cargo, lockfile and release versions differ.",
    );
  }
  return version;
}
