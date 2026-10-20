import type { UnitDraft } from "@/bridge/generated/bindings";

type Entry = {
  previous: UnitDraft[];
  added: string[];
  order: string[] | null;
};

export interface UnitHistory {
  record(before: UnitDraft[], after: UnitDraft[]): boolean;
  undo(current: UnitDraft[]): UnitDraft[] | null;
  clear(): void;
  canUndo(): boolean;
}

function sameContent(left: UnitDraft, right: UnitDraft): boolean {
  return (
    left.x_coord === right.x_coord &&
    left.y_coord === right.y_coord &&
    left.is_bubble === right.is_bubble &&
    left.is_flagged === right.is_flagged &&
    left.is_proofread === right.is_proofread &&
    left.translated_text === right.translated_text &&
    left.proofread_text === right.proofread_text
  );
}

export function createUnitHistory(): UnitHistory {
  const entries: Entry[] = [];
  return {
    record(before, after): boolean {
      const old = new Map(before.map((unit) => [unit.id, unit]));
      const next = new Map(after.map((unit) => [unit.id, unit]));
      const previous = before
        .filter((unit) => {
          const replacement = next.get(unit.id);
          return !replacement || !sameContent(unit, replacement);
        })
        .map((unit) => ({ ...unit }));
      const added = after
        .filter((unit) => !old.has(unit.id))
        .map((unit) => unit.id);
      const order =
        before.length === after.length &&
        before.every((unit, index) => unit.id === after[index]?.id)
          ? null
          : before.map((unit) => unit.id);
      if (previous.length === 0 && added.length === 0 && order === null)
        return false;
      entries.push({ previous, added, order });
      return true;
    },
    undo(current): UnitDraft[] | null {
      const entry = entries.pop();
      if (!entry) return null;
      const units = new Map(current.map((unit) => [unit.id, unit]));
      for (const id of entry.added) units.delete(id);
      for (const unit of entry.previous) units.set(unit.id, unit);
      return (entry.order ?? current.map((unit) => unit.id)).map((id) => {
        const unit = units.get(id);
        if (!unit) throw new Error("回退历史缺少单元");
        return unit;
      });
    },
    clear(): void {
      entries.length = 0;
    },
    canUndo(): boolean {
      return entries.length > 0;
    },
  };
}
