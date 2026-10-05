import { useEffect, useState } from "react";
import { commands, NativeCommandError, unwrap } from "@/bridge";
import type { ImageKind } from "@/bridge/generated/bindings";
import { imageUrl } from "@/bridge/image-url";

type ImageState = { key: string; url: string; error: string };
type ManagedImage = { url: string; error: string; retry: () => void };

export function useManagedImage(
  comicId: string,
  pageId: string,
  reference: string,
  kind: ImageKind,
  visible: boolean,
): ManagedImage {
  const [attempt, setAttempt] = useState(0);
  const key = JSON.stringify([
    comicId,
    pageId,
    reference,
    kind,
    visible,
    attempt,
  ]);
  const [state, setState] = useState<ImageState>({
    key: "",
    url: "",
    error: "",
  });
  useEffect(() => {
    if (!visible) {
      return;
    }
    let active = true;
    let handle = "";
    async function release(value: string): Promise<void> {
      try {
        unwrap(await commands.releaseImageResources([value]));
      } catch {
        console.warn("图片显示资源释放失败，将在退出时回收。");
      }
    }
    async function load(): Promise<void> {
      try {
        const image = unwrap(
          await commands.getImageResource(comicId, pageId, kind),
        );
        if (!active) {
          await release(image.handle);
          return;
        }
        handle = image.handle;
        setState({ key, url: imageUrl(handle), error: "" });
      } catch (cause) {
        if (active) {
          setState({
            key,
            url: "",
            error:
              cause instanceof NativeCommandError
                ? cause.detail.message
                : "图片无法加载，请重试。",
          });
        }
      }
    }
    load().catch(() => {
      if (active) {
        setState({ key, url: "", error: "图片无法加载，请重试。" });
      }
    });
    return () => {
      active = false;
      if (handle !== "") {
        release(handle).catch(() => {
          console.warn("图片显示资源将在退出时回收。");
        });
      }
    };
  }, [comicId, pageId, kind, visible, key]);
  const current = state.key === key ? state : { url: "", error: "" };
  return {
    ...current,
    retry: () => {
      setAttempt((value) => value + 1);
    },
  };
}
