import { useEffect, useLayoutEffect, useRef, useState } from "react";
import type { EditorSession } from "./editor-session";

type Options = {
  value: string;
  onChange: (value: string) => void;
  session: EditorSession;
};
type ComposedText = {
  value: string;
  change: (value: string) => void;
  start: () => void;
  end: (value: string) => void;
};

export function useComposedText({
  value,
  onChange,
  session,
}: Options): ComposedText {
  const composing = useRef(false);
  const submitted = useRef(value);
  const [preview, setPreview] = useState<string | null>(null);
  useLayoutEffect(() => {
    submitted.current = value;
  }, [value]);
  useEffect(
    () => () => {
      if (composing.current) {
        composing.current = false;
        session.suspend(false);
      }
    },
    [session],
  );
  function submit(next: string): void {
    if (submitted.current === next) return;
    submitted.current = next;
    onChange(next);
  }
  return {
    value: preview ?? value,
    change(next): void {
      if (composing.current) setPreview(next);
      else submit(next);
    },
    start(): void {
      composing.current = true;
      setPreview(value);
      session.suspend(true);
    },
    end(next): void {
      composing.current = false;
      setPreview(null);
      submit(next);
      session.suspend(false);
    },
  };
}
