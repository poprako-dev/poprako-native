import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { checkDenoVersion } from "./toolchain.ts";

checkDenoVersion();

// GNU's unmodified AGPL v3 text, verified by Licensee's Exact matcher.
// Source: https://www.gnu.org/licenses/agpl-3.0.txt
const expected =
  "0d96a4ff68ad6d4b6f1f30f713b18d5184912ba8dd389f86aa7710db079abcb0";
const actual = createHash("sha256")
  .update(await readFile("LICENSE"))
  .digest("hex");
if (actual !== expected) {
  throw new Error(
    "LICENSE differs from the official AGPL v3 text. Keep project notices in README.md.",
  );
}
console.log("LICENSE matches the official AGPL v3 text.");
