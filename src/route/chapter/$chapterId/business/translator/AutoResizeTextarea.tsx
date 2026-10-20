import { useLayoutEffect, useRef } from "react";
import type { ComponentProps, ReactElement, RefObject } from "react";
import { LineBreakOverlay } from "./LineBreakOverlay";

type Props = Omit<ComponentProps<"textarea">, "value" | "ref"> & {
  value: string;
  inputRef: RefObject<HTMLTextAreaElement | null>;
};

function resize(textarea: HTMLTextAreaElement): void {
  textarea.style.height = "auto";
  textarea.style.height = `${String(textarea.scrollHeight)}px`;
}

export function AutoResizeTextarea({
  value,
  inputRef,
  className = "",
  ...props
}: Props): ReactElement {
  const mirror = useRef<HTMLDivElement>(null);
  const display = value.replaceAll(/\r\n?/gu, "\n");
  useLayoutEffect(() => {
    if (inputRef.current) resize(inputRef.current);
  }, [inputRef, value, className]);
  useLayoutEffect(() => {
    const textarea = inputRef.current;
    if (!textarea) return;
    let previousWidth = textarea.getBoundingClientRect().width;
    let frame: number | null = null;
    const observer = new ResizeObserver(() => {
      if (frame !== null) return;
      frame = requestAnimationFrame(() => {
        frame = null;
        const width = textarea.getBoundingClientRect().width;
        if (width === previousWidth) return;
        previousWidth = width;
        if (width > 0) resize(textarea);
      });
    });
    observer.observe(textarea);
    return () => {
      observer.disconnect();
      if (frame !== null) cancelAnimationFrame(frame);
    };
  }, [inputRef]);
  const layout = `box-border w-full border-0 p-0 pr-4 whitespace-pre-wrap [overflow-wrap:break-word] ${className}`;
  return (
    <div className="relative min-w-0 flex-1">
      <textarea
        {...props}
        ref={inputRef}
        rows={1}
        value={value}
        className={`block resize-none overflow-hidden bg-transparent transition-colors focus:outline-none ${layout}`}
        style={{ minHeight: "1.2em" }}
      />
      {display.includes("\n") && (
        <>
          <div
            ref={mirror}
            aria-hidden="true"
            className={`pointer-events-none invisible absolute inset-0 select-none ${layout}`}
          >
            {display}
          </div>
          <LineBreakOverlay
            targetRef={mirror}
            layoutKey={[display, className]}
          />
        </>
      )}
    </div>
  );
}
