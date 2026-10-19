import { readReleaseVersion } from "./release-version.ts";
import { checkNodeVersion } from "./toolchain.ts";

checkNodeVersion();
console.log(`Release version: ${await readReleaseVersion(process.cwd())}`);
