import { convertFileSrc } from "@tauri-apps/api/core";

export function imageUrl(handle: string): string {
  return convertFileSrc(handle, "poprako-resource");
}
