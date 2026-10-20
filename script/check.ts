import { run } from "./process.ts";

await run("pnpm", ["run", "format:check"]);
await run("pnpm", ["run", "typecheck"]);
await run("pnpm", ["run", "lint"]);
await run(process.execPath, ["script/check-license.ts"]);
await run(process.execPath, ["script/check-release-version.ts"]);
await run(process.execPath, ["script/check-style.ts"]);
await run(process.execPath, ["script/generate.ts", "--check"]);
await run("cargo", [
  "fmt",
  "--manifest-path",
  "src-tauri/Cargo.toml",
  "--all",
  "--check",
]);
await run("cargo", [
  "check",
  "--manifest-path",
  "src-tauri/Cargo.toml",
  "--workspace",
  "--all-targets",
  "--locked",
]);
await run("cargo", [
  "clippy",
  "--manifest-path",
  "src-tauri/Cargo.toml",
  "--workspace",
  "--all-targets",
  "--locked",
  "--",
  "-D",
  "warnings",
]);
await run("pnpm", ["run", "test"]);
