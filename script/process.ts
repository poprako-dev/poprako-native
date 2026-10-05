import { spawn } from "node:child_process";
import { checkDenoVersion } from "./toolchain.ts";

checkDenoVersion();

export async function run(
  command: string,
  args: string[],
  cwd = process.cwd(),
  env: NodeJS.ProcessEnv = process.env,
): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    const child = spawn(command, args, { cwd, env, stdio: "inherit" });
    child.on("error", reject);
    child.on("exit", (code) => {
      if (code === 0) {
        resolve();
        return;
      }
      reject(new Error(`${command} failed (${String(code)})`));
    });
  });
}
