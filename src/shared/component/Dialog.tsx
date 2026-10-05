import * as Primitive from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import type { ReactElement, ReactNode } from "react";
import { className } from "@/shared/utility/class-name";
import { useSurface } from "./Appearance";

type DialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description: string;
  children: ReactNode;
  className: string;
};

export function Dialog({
  open,
  onOpenChange,
  title,
  description,
  children,
  className: customClass,
}: DialogProps): ReactElement {
  const surface = useSurface();
  return (
    <Primitive.Root open={open} onOpenChange={onOpenChange}>
      <Primitive.Portal>
        <Primitive.Overlay className="fixed inset-0 z-50 bg-black/40" />
        <Primitive.Content
          data-ui-surface={surface}
          {...(description === "" ? { "aria-describedby": undefined } : {})}
          className={className(
            "fixed top-1/2 left-1/2 z-50 flex max-h-[92dvh] w-[calc(100%-2rem)] max-w-lg -translate-x-1/2 -translate-y-1/2 flex-col gap-4 overflow-y-auto rounded-lg border border-border bg-background p-6 shadow-xl",
            customClass,
          )}
        >
          <div className="space-y-1.5 pr-6">
            <Primitive.Title className="text-base font-semibold">
              {title}
            </Primitive.Title>
            {description !== "" && (
              <Primitive.Description className="text-xs text-muted-foreground">
                {description}
              </Primitive.Description>
            )}
          </div>
          {children}
          <Primitive.Close
            className="absolute top-4 right-4 rounded p-1 text-muted-foreground hover:bg-accent focus-visible:outline-2 focus-visible:outline-ring"
            aria-label="关闭弹窗"
          >
            <X className="size-4" aria-hidden="true" />
          </Primitive.Close>
        </Primitive.Content>
      </Primitive.Portal>
    </Primitive.Root>
  );
}
