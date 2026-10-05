import type { PageEditor, Unit } from "@/bridge/generated/bindings";

export function editorFixture(): PageEditor {
  const units: Unit[] = ["a", "b", "c"].map((id, index) => ({
    id,
    page_id: "page",
    index,
    x_coord: 0.2 * (index + 1),
    y_coord: 0.5,
    is_bubble: true,
    is_flagged: index !== 1,
    translated_text: `翻译 ${id}`,
    proofread_text: "",
    is_proofread: false,
    created_at: 1,
    updated_at: 1,
  }));
  return {
    page: {
      id: "page",
      comic_id: "comic",
      index: 0,
      unit_revision: 0,
      image: {
        reference: "image",
        original_name: "001.png",
        format: "png",
        width: 100,
        height: 200,
      },
      created_at: 1,
      updated_at: 1,
    },
    units,
  };
}
