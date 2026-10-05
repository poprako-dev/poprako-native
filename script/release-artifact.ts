import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  copyFile,
  mkdir,
  readFile,
  readdir,
  stat,
  writeFile,
} from "node:fs/promises";
import { join } from "node:path";
import {
  readReleaseVersion,
  record,
  stableVersion,
} from "./release-version.ts";
import { checkDenoVersion } from "./toolchain.ts";

type Platform = "macos-arm64" | "windows-x64";

async function checkInstallerSize(path: string): Promise<void> {
  const { size } = await stat(path);
  if (size === 0 || size > 10 * 1024 * 1024) {
    throw new Error(
      `Installer must be nonempty and at most 10 MiB: ${path} (${String(size)} bytes).`,
    );
  }
  console.log(
    `Installer size: ${String(size)} bytes (${(size / 1024 / 1024).toFixed(2)} MiB).`,
  );
}

function required(name: string): string {
  const value = process.env[name];
  if (!value) throw new Error(`Missing ${name}.`);
  return value;
}

function assetName(version: string, platform: Platform): string {
  const suffix = platform === "macos-arm64" ? ".dmg" : "-setup.exe";
  return `PopRaKo-Native_${version}_${platform}${suffix}`;
}

function signingMode(platform: Platform): string {
  if (required("RELEASE_SIGNING") === "true") {
    return platform === "macos-arm64" ? "developer-id" : "authenticode";
  }
  return platform === "macos-arm64" ? "ad-hoc" : "unsigned";
}

async function digest(path: string): Promise<string> {
  return createHash("sha256")
    .update(await readFile(path))
    .digest("hex");
}

async function collect(
  directory: string,
  version: string,
  sha: string,
): Promise<void> {
  const platform = required("RELEASE_PLATFORM");
  if (platform !== "macos-arm64" && platform !== "windows-x64") {
    throw new Error("Unsupported release platform.");
  }
  const expected = platform === "macos-arm64" ? "darwin" : "win32";
  const architecture = platform === "macos-arm64" ? "arm64" : "x64";
  if (process.platform !== expected || process.arch !== architecture) {
    throw new Error("Runner does not match the release platform.");
  }
  const head = execFileSync("git", ["rev-parse", "HEAD"], {
    encoding: "utf8",
  }).trim();
  if (head !== sha || (await readReleaseVersion(process.cwd())) !== version) {
    throw new Error(
      "Build source or application version differs from release.",
    );
  }
  const target =
    platform === "macos-arm64"
      ? "aarch64-apple-darwin/release/bundle/dmg"
      : "x86_64-pc-windows-msvc/release/bundle/nsis";
  const source = join("src-tauri/target", target);
  const suffix = platform === "macos-arm64" ? ".dmg" : ".exe";
  const candidates = (await readdir(source)).filter((file) =>
    file.endsWith(suffix),
  );
  const [candidate] = candidates;
  if (candidates.length !== 1 || candidate === undefined) {
    throw new Error("Expected exactly one platform installer.");
  }
  await mkdir(directory, { recursive: true });
  const file = assetName(version, platform);
  await checkInstallerSize(join(source, candidate));
  await copyFile(join(source, candidate), join(directory, file));
  await writeFile(
    join(directory, `${platform}.json`),
    `${JSON.stringify(
      {
        version,
        sha,
        platform,
        file,
        sha256: await digest(join(directory, file)),
        signing: signingMode(platform),
      },
      null,
      2,
    )}\n`,
  );
}

async function verify(
  directory: string,
  version: string,
  sha: string,
): Promise<void> {
  const platforms: Platform[] = ["macos-arm64", "windows-x64"];
  const expectedFiles = platforms.flatMap((platform) => [
    assetName(version, platform),
    `${platform}.json`,
  ]);
  const actualFiles = await readdir(directory);
  if (
    actualFiles.length !== expectedFiles.length ||
    actualFiles.some((file) => !expectedFiles.includes(file))
  ) {
    throw new Error(
      "Release artifact set is incomplete or contains extra files.",
    );
  }
  const sums: string[] = [];
  for (const platform of platforms) {
    const metadata = record(
      JSON.parse(await readFile(join(directory, `${platform}.json`), "utf8")),
    );
    const file = assetName(version, platform);
    await checkInstallerSize(join(directory, file));
    const hash = await digest(join(directory, file));
    if (
      metadata["version"] !== version ||
      metadata["sha"] !== sha ||
      metadata["platform"] !== platform ||
      metadata["file"] !== file ||
      metadata["sha256"] !== hash ||
      metadata["signing"] !== signingMode(platform)
    ) {
      throw new Error(`Release metadata or checksum mismatch: ${platform}.`);
    }
    sums.push(`${hash}  ${file}`);
  }
  await writeFile(join(directory, "SHA256SUMS"), `${sums.join("\n")}\n`);
}

checkDenoVersion();
const directory = required("RELEASE_DIRECTORY");
const tag = required("RELEASE_TAG");
if (!tag.startsWith("v")) throw new Error("Release tag must start with v.");
const version = stableVersion(tag.slice(1));
const sha = required("RELEASE_SHA");
if (!/^[a-f0-9]{40}$/.test(sha)) throw new Error("Expected a full commit SHA.");
const command = process.argv[2] ?? "";
switch (command) {
  case "collect":
    await collect(directory, version, sha);
    break;
  case "verify":
    await verify(directory, version, sha);
    break;
  default:
    throw new Error("Expected collect or verify.");
}
