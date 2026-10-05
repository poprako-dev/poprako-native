import type { ComponentProps, ReactElement } from "react";
import { Button } from "./Button";

type IconButtonProps = Omit<
  ComponentProps<typeof Button>,
  "aria-label" | "title" | "variant"
> & {
  label: string;
};

export function IconButton({
  label,
  className = "",
  ...props
}: IconButtonProps): ReactElement {
  return (
    <Button
      variant="ghost"
      aria-label={label}
      title={label}
      className={`size-8 p-0 ${className}`}
      {...props}
    />
  );
}
