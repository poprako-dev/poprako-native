import type { Page, PageBaseline } from "@/bridge/generated/bindings";

export function pageBaseline(page: Page): PageBaseline {
  return {
    id: page.id,
    unit_revision: page.unit_revision,
    image_reference: page.image.reference,
  };
}
