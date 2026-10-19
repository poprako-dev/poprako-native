import { useEffect, useState } from "react";
import { commands, unwrap } from "@/bridge";
import { imageUrl } from "@/bridge/image-url";

export function usePageImage(
  chapterId: string,
  pageId: string,
): { url: string; error: string } {
  const [state, setState] = useState({ chapterId, pageId, url: "", error: "" });
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
          await commands.getImageResource(chapterId, pageId, "preview"),
        );
        if (!active) {
          await release(resource.handle);
          return;
        }
        handle = resource.handle;
        setState({ chapterId, pageId, url: imageUrl(handle), error: "" });
      } catch (error) {
        if (active)
          setState({
            chapterId,
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
  }, [chapterId, pageId]);
  if (state.chapterId !== chapterId || state.pageId !== pageId)
    return { url: "", error: "" };
  return state;
}
