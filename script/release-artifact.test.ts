import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

const sha = "a".repeat(40);
let directory = "";

async function fixture(platform: string, signed: boolean): Promise<void> {
  const suffix = platform === "macos-arm64" ? ".dmg" : "-setup.exe";
  const file = `PopRaKo-Native_1.0.0_${platform}${suffix}`;
  const bytes = Buffer.from(`Installer fixture: ${platform}`);
  await writeFile(join(directory, file), bytes);
  await writeFile(
    join(directory, `${platform}.json`),
    JSON.stringify({
      version: "1.0.0",
      sha,
      platform,
      file,
      sha256: createHash("sha256").update(bytes).digest("hex"),
      signing: signed
        ? platform === "macos-arm64"
          ? "developer-id"
          : "authenticode"
        : platform === "macos-arm64"
          ? "ad-hoc"
          : "unsigned",
    }),
  );
}

function verify(overrides: Record<string, string> = {}): void {
  execFileSync(
    process.execPath,
    [
      "run",
      "--allow-read",
      "--allow-write",
      "--allow-env",
      resolve("script/release-artifact.ts"),
      "verify",
    ],
    {
      stdio: "pipe",
      env: {
        ...process.env,
        RELEASE_TAG: "v1.0.0",
        RELEASE_SHA: sha,
        RELEASE_DIRECTORY: directory,
        RELEASE_SIGNING: "false",
        RELEASE_PUBLISH: "false",
        ...overrides,
      },
    },
  );
}

describe("release artifact gate", () => {
  beforeEach(async () => {
    directory = await mkdtemp(join(tmpdir(), "poprako-release-"));
  });

  afterEach(async () => {
    await rm(directory, { recursive: true, force: true });
  });

  it("assembles checksums for an ad-hoc macOS and unsigned Windows draft", async () => {
    await fixture("macos-arm64", false);
    await fixture("windows-x64", false);
    verify();
    const sums = await readFile(join(directory, "SHA256SUMS"), "utf8");
    expect(sums.trim().split("\n")).toHaveLength(2);
    expect(sums).toContain("macos-arm64.dmg");
    expect(sums).toContain("windows-x64-setup.exe");
  });

  it("rejects a missing platform", async () => {
    await fixture("macos-arm64", false);
    expect(() => {
      verify();
    }).toThrow("artifact set is incomplete");
  });

  it.each(["macos-arm64", "windows-x64"])(
    "rejects a %s installer over 10 MiB",
    async (platform) => {
      await fixture("macos-arm64", false);
      await fixture("windows-x64", false);
      const suffix = platform === "macos-arm64" ? ".dmg" : "-setup.exe";
      await writeFile(
        join(directory, `PopRaKo-Native_1.0.0_${platform}${suffix}`),
        Buffer.alloc(10 * 1024 * 1024 + 1),
      );
      expect(() => {
        verify();
      }).toThrow("at most 10 MiB");
    },
  );

  it("rejects an installer changed after collection", async () => {
    await fixture("macos-arm64", false);
    await fixture("windows-x64", false);
    await writeFile(
      join(directory, "PopRaKo-Native_1.0.0_windows-x64-setup.exe"),
      "changed",
    );
    expect(() => {
      verify();
    }).toThrow("checksum mismatch");
  });

  it("rejects artifacts from another commit", async () => {
    await fixture("macos-arm64", false);
    await fixture("windows-x64", false);
    expect(() => {
      verify({ RELEASE_SHA: "b".repeat(40) });
    }).toThrow("checksum mismatch");
  });

  it("accepts ad-hoc macOS and unsigned Windows for publication", async () => {
    await fixture("macos-arm64", false);
    await fixture("windows-x64", false);
    verify({ RELEASE_PUBLISH: "true" });
    expect(await readFile(join(directory, "SHA256SUMS"), "utf8")).toContain(
      "PopRaKo-Native_1.0.0",
    );
  });

  it("accepts matching signature attestations for publication", async () => {
    await fixture("macos-arm64", true);
    await fixture("windows-x64", true);
    verify({ RELEASE_SIGNING: "true", RELEASE_PUBLISH: "true" });
    expect(await readFile(join(directory, "SHA256SUMS"), "utf8")).toContain(
      "PopRaKo-Native_1.0.0",
    );
  });

  it("rejects mixed signing modes when retrying", async () => {
    await fixture("macos-arm64", true);
    await fixture("windows-x64", false);
    expect(() => {
      verify({ RELEASE_SIGNING: "true" });
    }).toThrow("checksum mismatch");
  });

  it("rejects an unsigned macOS artifact in ad-hoc mode", async () => {
    await fixture("macos-arm64", false);
    await fixture("windows-x64", false);
    const path = join(directory, "macos-arm64.json");
    const metadata = await readFile(path, "utf8");
    await writeFile(path, metadata.replace('"ad-hoc"', '"unsigned"'));
    expect(() => {
      verify();
    }).toThrow("checksum mismatch");
  });
});
