import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { WindowFrame } from "./WindowFrame";

const window = vi.hoisted(() => ({
  closeWindow: vi.fn<() => Promise<void>>(),
  dragWindow: vi.fn<() => Promise<void>>(),
  listenForWindowResize: vi.fn(),
  minimizeWindow: vi.fn<() => Promise<void>>(),
  readWindowState: vi.fn(),
  resizeWindow: vi.fn<() => Promise<void>>(),
  toggleWindowSize: vi.fn<() => Promise<void>>(),
}));

vi.mock("@/bridge/window", () => window);

describe("根窗口外框", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    window.readWindowState.mockResolvedValue({
      maximized: false,
      fullscreen: false,
    });
    window.listenForWindowResize.mockResolvedValue(vi.fn());
    window.closeWindow.mockResolvedValue();
    window.minimizeWindow.mockResolvedValue();
    window.toggleWindowSize.mockResolvedValue();
  });

  afterEach(() => {
    cleanup();
    delete document.body.dataset["windowExpanded"];
  });

  it("保留路由内容并调用窗口关闭请求及其他控制", async () => {
    render(
      <WindowFrame>
        <main>项目内容</main>
      </WindowFrame>,
    );
    await waitFor(() => {
      expect(window.readWindowState).toHaveBeenCalled();
    });
    expect(screen.getByText("项目内容")).not.toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "最小化" }));
    fireEvent.click(screen.getByRole("button", { name: "最大化" }));
    fireEvent.click(screen.getByRole("button", { name: "关闭窗口" }));
    await waitFor(() => {
      expect(window.minimizeWindow).toHaveBeenCalledOnce();
      expect(window.toggleWindowSize).toHaveBeenCalledOnce();
      expect(window.closeWindow).toHaveBeenCalledOnce();
    });
  });

  it("全屏状态使用还原按钮和无圆角外框，卸载清理监听", async () => {
    const cleanup = vi.fn();
    window.readWindowState.mockResolvedValue({
      maximized: false,
      fullscreen: true,
    });
    window.listenForWindowResize.mockResolvedValue(cleanup);
    const view = render(<WindowFrame>内容</WindowFrame>);
    await screen.findByRole("button", { name: "还原窗口" });
    await waitFor(() => {
      expect(document.body.dataset["windowExpanded"]).toBe("true");
    });
    view.unmount();
    expect(cleanup).toHaveBeenCalledOnce();
    expect(document.body.dataset["windowExpanded"]).toBeUndefined();
  });

  it("窗口请求失败保留内容并可重试", async () => {
    window.closeWindow.mockRejectedValueOnce(new Error("blocked"));
    render(<WindowFrame>未保存内容</WindowFrame>);
    fireEvent.click(screen.getByRole("button", { name: "关闭窗口" }));
    expect((await screen.findByRole("alert")).textContent).toContain(
      "窗口操作未完成",
    );
    expect(screen.getByText("未保存内容")).not.toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "关闭窗口" }));
    await waitFor(() => {
      expect(screen.queryByRole("alert")).toBeNull();
    });
  });
});
