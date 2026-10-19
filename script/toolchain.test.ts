import { describe, expect, it } from "vitest";
import { assertNodeVersion } from "./toolchain.ts";

describe("Node.js toolchain pin", () => {
  it("accepts the pinned runtime", () => {
    expect(() => {
      assertNodeVersion("26.11.1", "26.11.1");
    }).not.toThrow();
  });

  it.each(["26.11.0", "27.0.0", undefined])(
    "rejects a mismatched runtime (%s)",
    (actual) => {
      expect(() => {
        assertNodeVersion(actual, "26.11.1");
      }).toThrow("requires Node.js 26.11.1");
    },
  );
});
