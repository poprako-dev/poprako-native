import type { ComponentProps, ReactElement } from "react";
import { className } from "@/shared/utility/class-name";

type ButtonProps = ComponentProps<"button"> & {
  variant: "primary" | "secondary" | "ghost" | "danger";
};

const variantClass = {
  primary: "bg-primary text-primary-foreground hover:bg-primary/90",
  secondary: "border border-border bg-card hover:bg-accent",
  ghost: "text-muted-foreground hover:bg-accent hover:text-foreground",
  danger: "bg-destructive text-white hover:bg-destructive/90",
};

export function Button({
  variant,
  className: customClass,
  type = "button",
  ...props
}: ButtonProps): ReactElement {
  return (
    <button
      type={
        type === "submit" ? "submit" : type === "reset" ? "reset" : "button"
      }
      className={className(
        "inline-flex h-8 shrink-0 items-center justify-center gap-2 rounded-md px-3 text-xs font-medium transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring disabled:pointer-events-none disabled:opacity-40",
        variantClass[variant],
        customClass,
      )}
      {...props}
    />
  );
}
