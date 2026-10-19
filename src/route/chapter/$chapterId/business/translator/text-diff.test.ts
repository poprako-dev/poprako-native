import { describe, expect, it } from "vitest";
import { buildTextDiff } from "./text-diff";

describe("Web text difference migration", () => {
  it("keeps emoji graphemes and unchanged Chinese text together", () => {
    expect(buildTextDiff("你好👨‍👩‍👧世界", "你好👨‍👩‍👧漫画")).toEqual([
      { kind: "unchanged", text: "你好👨‍👩‍👧" },
      { kind: "replacement-removed", text: "世界" },
      { kind: "replacement-added", text: "漫画" },
    ]);
  });
  it("falls back to translation for a blank proofreading field", () => {
    expect(buildTextDiff("翻译", " \t\n")).toEqual([
      { kind: "unchanged", text: "翻译" },
    ]);
  });
  it("bounds fine-grained work while preserving both large texts exactly", () => {
    const translated = "旧".repeat(20_000);
    const proofread = "新".repeat(20_000);
    const parts = buildTextDiff(translated, proofread);
    expect(
      parts
        .filter((part) => part.kind === "replacement-removed")
        .map((part) => part.text)
        .join(""),
    ).toBe(translated);
    expect(
      parts
        .filter((part) => part.kind === "replacement-added")
        .map((part) => part.text)
        .join(""),
    ).toBe(proofread);
  });
});
