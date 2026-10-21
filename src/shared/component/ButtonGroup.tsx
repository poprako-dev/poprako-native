import type { ReactElement, ReactNode } from "react";
import { className } from "@/shared/utility/class-name";

type Props = { children: ReactNode; className?: string };

export function ButtonGroup({
  children,
  className: customClass,
}: Props): ReactElement {
  return (
    <div
      className={className(
        "grid w-full auto-cols-fr grid-flow-col gap-2 empty:hidden [&>button]:h-auto [&>button]:min-h-8 [&>button]:w-full [&>button]:min-w-0 [&>button]:py-1.5 [&>button]:whitespace-normal",
        customClass,
      )}
    >
      {children}
    </div>
  );
}
