import type { PageInfo, UnitDraft } from "@/bridge/generated/bindings";
import type { EditorState } from "./editor-session";
import { normalizedDraft, isDirty } from "./editor-session";

export type PageStatistic = {
  id: string;
  index: number;
  total: number;
  translated: number;
  confirmed: number;
  flagged: number;
  edited: number;
  appended: number;
  dirty: boolean;
};

export function countUnits(
  units: UnitDraft[],
): Omit<PageStatistic, "id" | "index" | "dirty"> {
  const normalized = units.map(normalizedDraft);
  return {
    total: normalized.length,
    translated: normalized.filter((unit) => unit.translated_text !== "").length,
    confirmed: normalized.filter((unit) => unit.is_proofread).length,
    flagged: normalized.filter((unit) => unit.is_flagged).length,
    edited: normalized.filter(
      (unit) =>
        unit.translated_text !== "" &&
        unit.proofread_text !== "" &&
        unit.translated_text !== unit.proofread_text,
    ).length,
    appended: normalized.filter(
      (unit) => unit.translated_text === "" && unit.proofread_text !== "",
    ).length,
  };
}

export function pageStatistics(
  pages: PageInfo[],
  state: EditorState,
): PageStatistic[] {
  return pages.map((info) => ({
    id: info.page.id,
    index: info.page.index,
    ...(info.page.id === state.baseline.page.id
      ? countUnits(state.draft)
      : {
          total: info.unit_count,
          translated: info.translated_count,
          confirmed: info.proofread_count,
          flagged: info.flagged_count,
          edited: info.edited_count,
          appended: info.proofreader_append_count,
        }),
    dirty: info.page.id === state.baseline.page.id && isDirty(state),
  }));
}

export function nextEditedPage(
  pages: PageStatistic[],
  currentId: string,
): PageStatistic | undefined {
  const current = pages.findIndex((page) => page.id === currentId);
  if (current < 0) return undefined;
  return pages.find(
    (page, index) => index > current && (page.edited > 0 || page.appended > 0),
  );
}
