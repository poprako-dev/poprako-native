import { clsx } from "clsx";
import type { ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function className(...values: ClassValue[]): string {
  return twMerge(clsx(values));
}
