import { run } from "./process.ts";

await run("deno", ["task", "format:check"]);
await run("deno", ["task", "typecheck"]);
await run("deno", ["task", "lint"]);
await run("deno", ["run", "-A", "script/check-license.ts"]);
await run("deno", ["run", "-A", "script/check-release-version.ts"]);
await run("deno", ["run", "-A", "script/check-style.ts"]);
await run("deno", ["run", "-A", "script/generate.ts", "--check"]);
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
await run("deno", ["task", "test"]);
