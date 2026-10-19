// @vitest-environment node
import { once } from "node:events";
import { mkdtemp, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { createServer, loadConfigFromFile } from "vite";
import { expect, it, vi } from "vitest";

it("sends a module update after saving a file in local desktop development", async () => {
  vi.stubEnv("TAURI_DEV_HOST", undefined);
  const loaded = await loadConfigFromFile(
    { command: "serve", mode: "development" },
    resolve("vite.config.ts"),
  );
  if (loaded === null) throw new Error("Vite configuration is missing");
  const root = await realpath(await mkdtemp(join(tmpdir(), "poprako-hmr-")));
  const entry = join(root, "entry.js");
  await writeFile(entry, "export const value = 1; import.meta.hot.accept();");
  const server = await createServer({
    configFile: false,
    root,
    logLevel: "silent",
    server: {
      ...loaded.config.server,
      host: "127.0.0.1",
      port: 0,
      strictPort: false,
    },
    optimizeDeps: { noDiscovery: true },
  });
  let socket: WebSocket | undefined;
  try {
    await server.listen();
    const address = server.httpServer?.address();
    if (
      address === null ||
      address === undefined ||
      typeof address === "string"
    ) {
      throw new Error("Vite did not open a local port");
    }
    const origin = `127.0.0.1:${String(address.port)}`;
    const response = await fetch(`http://${origin}/entry.js`);
    expect(response.status).toBe(200);
    expect(
      await server.environments.client.moduleGraph.getModuleByUrl("/entry.js"),
    ).toBeDefined();
    socket = new WebSocket(`ws://${origin}/`, "vite-hmr");
    const messages: string[] = [];
    socket.addEventListener("message", (event) => {
      if (typeof event.data === "string") messages.push(event.data);
    });
    await once(socket, "open", { signal: AbortSignal.timeout(3000) });
    await expect
      .poll(() => Object.values(server.watcher.getWatched()).flat())
      .toContain("entry.js");
    await writeFile(entry, "export const value = 2; import.meta.hot.accept();");
    await expect
      .poll(() => messages.join("\n"), { timeout: 3000 })
      .toContain('"type":"js-update"');
    expect(messages.join("\n")).toContain('"path":"/entry.js"');
  } finally {
    socket?.close();
    await server.close();
    await rm(root, { recursive: true, force: true });
    vi.unstubAllEnvs();
  }
}, 10000);
