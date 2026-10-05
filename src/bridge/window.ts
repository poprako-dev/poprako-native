import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";

export type ResizeDirection = Parameters<
  ReturnType<typeof getCurrentWindow>["startResizeDragging"]
>[0];

export type WindowState = { maximized: boolean; fullscreen: boolean };

export async function readWindowState(): Promise<WindowState> {
  const window = getCurrentWindow();
  const [maximized, fullscreen] = await Promise.all([
    window.isMaximized(),
    window.isFullscreen(),
  ]);
  return { maximized, fullscreen };
}

export async function listenForWindowResize(
  callback: () => void,
): Promise<UnlistenFn> {
  return getCurrentWindow().onResized(callback);
}

export async function minimizeWindow(): Promise<void> {
  await getCurrentWindow().minimize();
}

export async function toggleWindowSize(): Promise<void> {
  const window = getCurrentWindow();
  if (await window.isFullscreen()) {
    await window.setFullscreen(false);
    return;
  }
  await window.toggleMaximize();
}

export async function closeWindow(): Promise<void> {
  // CloseRequested enters the existing draft-save and task-exit guards.
  await getCurrentWindow().close();
}

export async function dragWindow(): Promise<void> {
  await getCurrentWindow().startDragging();
}

export async function resizeWindow(direction: ResizeDirection): Promise<void> {
  await getCurrentWindow().startResizeDragging(direction);
}
