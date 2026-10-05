import { Copy, Minus, Square, X } from "lucide-react";
import { useEffect, useState } from "react";
import type { ReactElement, ReactNode } from "react";
import {
  closeWindow,
  dragWindow,
  listenForWindowResize,
  minimizeWindow,
  readWindowState,
  resizeWindow,
  toggleWindowSize,
} from "@/bridge/window";
import type { WindowState } from "@/bridge/window";
import type { ResizeDirection } from "@/bridge/window";
import { IconButton } from "@/shared/component/IconButton";

type WindowFrameProps = { children: ReactNode };

const edges: { direction: ResizeDirection; className: string }[] = [
  { direction: "North", className: "top-0 left-2 right-2 h-1 cursor-n-resize" },
  {
    direction: "South",
    className: "bottom-0 left-2 right-2 h-1 cursor-s-resize",
  },
  { direction: "West", className: "left-0 top-2 bottom-2 w-1 cursor-w-resize" },
  {
    direction: "East",
    className: "right-0 top-2 bottom-2 w-1 cursor-e-resize",
  },
  { direction: "NorthWest", className: "top-0 left-0 size-2 cursor-nw-resize" },
  {
    direction: "NorthEast",
    className: "top-0 right-0 size-2 cursor-ne-resize",
  },
  {
    direction: "SouthWest",
    className: "bottom-0 left-0 size-2 cursor-sw-resize",
  },
  {
    direction: "SouthEast",
    className: "bottom-0 right-0 size-2 cursor-se-resize",
  },
];

export function WindowFrame({ children }: WindowFrameProps): ReactElement {
  const [state, setState] = useState<WindowState>({
    maximized: false,
    fullscreen: false,
  });
  const [error, setError] = useState("");
  const expanded = state.maximized || state.fullscreen;

  async function perform(operation: () => Promise<void>): Promise<void> {
    try {
      await operation();
      setError("");
    } catch {
      setError("窗口操作未完成，请重试。");
    }
  }

  useEffect(() => {
    let active = true;
    let cleanup: (() => void) | null = null;
    let revision = 0;
    async function refresh(): Promise<void> {
      const current = ++revision;
      try {
        const next = await readWindowState();
        if (active && current === revision) setState(next);
      } catch {
        if (active) setError("窗口状态读取失败，请重新打开应用。");
      }
    }
    void listenForWindowResize(() => {
      void refresh();
    })
      .then((unlisten) => {
        if (!active) {
          unlisten();
          return;
        }
        cleanup = unlisten;
        void refresh();
      })
      .catch(() => {
        if (active) setError("窗口状态监听失败，请重新打开应用。");
      });
    return () => {
      active = false;
      cleanup?.();
    };
  }, []);

  useEffect(() => {
    document.body.dataset["windowExpanded"] = String(expanded);
    return () => {
      delete document.body.dataset["windowExpanded"];
    };
  }, [expanded]);

  return (
    <div
      className={`relative flex h-full min-h-0 flex-col overflow-hidden bg-background ${expanded ? "" : "rounded-xl border border-[#cddcc5]"}`}
    >
      <header
        className="flex h-9 shrink-0 select-none items-center border-b border-[#cddcc5] bg-[#e2f2d9] text-[#687763]"
        aria-label="窗口标题栏"
        data-window-control="true"
      >
        <div
          className="flex h-full min-w-0 flex-1 items-center px-3"
          onPointerDown={(event) => {
            if (event.button === 0 && event.detail < 2)
              void perform(dragWindow);
          }}
          onDoubleClick={() => {
            void perform(toggleWindowSize);
          }}
        >
          <span className="pointer-events-none text-xs font-medium">
            白杨子 N
          </span>
        </div>
        <IconButton
          label="最小化"
          className="h-full w-11 rounded-none hover:bg-[#cddfc5]"
          onClick={() => {
            void perform(minimizeWindow);
          }}
        >
          <Minus className="size-4" />
        </IconButton>
        <IconButton
          label={expanded ? "还原窗口" : "最大化"}
          className="h-full w-11 rounded-none hover:bg-[#cddfc5]"
          onClick={() => {
            void perform(toggleWindowSize);
          }}
        >
          {expanded ? (
            <Copy className="size-3.5" />
          ) : (
            <Square className="size-3.5" />
          )}
        </IconButton>
        <IconButton
          label="关闭窗口"
          className="h-full w-11 rounded-none hover:bg-[#edd7d2] hover:text-[#985f59]"
          onClick={() => {
            void perform(closeWindow);
          }}
        >
          <X className="size-4" />
        </IconButton>
      </header>
      {error !== "" && (
        <div
          role="alert"
          className="shrink-0 bg-red-50 px-3 py-1 text-xs text-red-800"
        >
          {error}
        </div>
      )}
      <div className="min-h-0 min-w-0 flex-1 overflow-hidden">{children}</div>
      {!expanded &&
        edges.map(({ direction, className }) => (
          <div
            key={direction}
            aria-hidden="true"
            className={`absolute z-50 ${className}`}
            onPointerDown={(event) => {
              if (event.button !== 0) return;
              event.preventDefault();
              void perform(() => resizeWindow(direction));
            }}
          />
        ))}
    </div>
  );
}
