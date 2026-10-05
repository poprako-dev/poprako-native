import { describe, expect, it } from "vitest";
import { assertDenoVersion } from "./toolchain.ts";

describe("Deno toolchain pin", () => {
  it("accepts the pinned runtime", () => {
    expect(() => {
      assertDenoVersion("2.9.6", "2.9.6");
    }).not.toThrow();
  });

  it.each(["2.9.5", "2.10.0", undefined])(
    "rejects a mismatched runtime (%s)",
    (actual) => {
      expect(() => {
        assertDenoVersion(actual, "2.9.6");
      }).toThrow("requires Deno 2.9.6");
    },
  );
});
