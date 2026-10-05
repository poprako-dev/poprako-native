import { readReleaseVersion } from "./release-version.ts";
import { checkDenoVersion } from "./toolchain.ts";

checkDenoVersion();
console.log(`Release version: ${await readReleaseVersion(process.cwd())}`);
