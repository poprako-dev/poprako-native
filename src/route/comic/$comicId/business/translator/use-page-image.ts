import { useEffect, useState } from "react";
import { commands, unwrap } from "@/bridge";
import { imageUrl } from "@/bridge/image-url";

export function usePageImage(
  comicId: string,
  pageId: string,
): { url: string; error: string } {
  const [state, setState] = useState({ comicId, pageId, url: "", error: "" });
  useEffect(() => {
    let active = true;
    let handle = "";
    async function release(value: string): Promise<void> {
      try {
        unwrap(await commands.releaseImageResources([value]));
      } catch {
        console.warn("图片显示资源释放未完成，将在退出应用时回收。");
      }
    }
    async function load(): Promise<void> {
      try {
        const resource = unwrap(
          await commands.getImageResource(comicId, pageId, "preview"),
        );
        if (!active) {
          await release(resource.handle);
          return;
        }
        handle = resource.handle;
        setState({ comicId, pageId, url: imageUrl(handle), error: "" });
      } catch (error) {
        if (active)
          setState({
            comicId,
            pageId,
            url: "",
            error:
              error instanceof Error
                ? error.message
                : "图片无法读取，请重新打开页面。",
          });
      }
    }
    void load();
    return () => {
      active = false;
      if (handle) void release(handle);
    };
  }, [comicId, pageId]);
  if (state.comicId !== comicId || state.pageId !== pageId)
    return { url: "", error: "" };
  return state;
}
