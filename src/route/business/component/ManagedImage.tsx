import { ImageOff, LoaderCircle } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import type { ReactElement } from "react";
import type { ImageKind } from "@/bridge/generated/bindings";
import { useManagedImage } from "@/route/business/use-managed-image";
import { Button } from "@/shared/component/Button";

type ManagedImageProps = {
  comicId: string;
  pageId: string;
  reference: string;
  kind: ImageKind;
  alt: string;
  interactive: boolean;
};

export function ManagedImage({
  comicId,
  pageId,
  reference,
  kind,
  alt,
  interactive,
}: ManagedImageProps): ReactElement {
  const element = useRef<HTMLDivElement>(null);
  const [visible, setVisible] = useState(false);
  const [failedUrl, setFailedUrl] = useState("");
  const image = useManagedImage(comicId, pageId, reference, kind, visible);
  useEffect(() => {
    const target = element.current;
    if (target === null) {
      return;
    }
    const observer = new IntersectionObserver((entries) => {
      setVisible(entries.some((entry) => entry.isIntersecting));
    });
    observer.observe(target);
    return () => {
      observer.disconnect();
    };
  }, []);
  const error =
    image.error !== ""
      ? image.error
      : image.url !== "" && image.url === failedUrl
        ? "图片显示失败，请重试。"
        : "";
  return (
    <div
      ref={element}
      className="flex h-full min-h-20 w-full items-center justify-center overflow-hidden bg-muted"
    >
      {error !== "" ? (
        <div
          className="flex max-w-full flex-col items-center gap-2 p-3 text-center text-xs text-muted-foreground"
          role="status"
        >
          <ImageOff className="size-5" aria-hidden="true" />
          <p>{error}</p>
          {interactive && (
            <Button
              variant="secondary"
              onClick={() => {
                image.retry();
              }}
            >
              重新加载
            </Button>
          )}
        </div>
      ) : image.url === "" ? (
        <LoaderCircle
          className="size-4 animate-spin text-muted-foreground"
          aria-label="正在加载图片"
        />
      ) : (
        <img
          src={image.url}
          alt={alt}
          draggable={false}
          className="h-full w-full object-contain"
          onError={() => {
            setFailedUrl(image.url);
          }}
        />
      )}
    </div>
  );
}
