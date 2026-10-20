import { afterEach, expect, test, vi } from "vitest";
import type { Unit } from "@/bridge/generated/bindings";
import {
  createEditorSession,
  isDirty,
  normalizedDraft,
} from "./editor-session";
import type { EditorApi, EditorSession } from "./editor-session";
import { editorFixture } from "./editor-test-fixture";

function firstUnit(): Unit {
  const unit = editorFixture().units[0];
  if (!unit) throw new Error("Missing fixture unit");
  return unit;
}

const sessions: EditorSession[] = [];
const whitespace = [
  0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x20, 0x85, 0xa0, 0x1680, 0x2000, 0x2001,
  0x2002, 0x2003, 0x2004, 0x2005, 0x2006, 0x2007, 0x2008, 0x2009, 0x200a,
  0x2028, 0x2029, 0x202f, 0x205f, 0x3000,
];

afterEach(() => {
  for (const session of sessions.splice(0)) session.dispose();
  vi.useRealTimers();
});

test.each(whitespace)(
  "normalizes Unicode White_Space code point %i in both fields",
  (code) => {
    const text = String.fromCodePoint(code);
    const result = normalizedDraft({
      ...firstUnit(),
      translated_text: text,
      proofread_text: text,
    });
    expect(result.translated_text).toBe("");
    expect(result.proofread_text).toBe("");
  },
);

test.each(["\ufeff", "\u200b", "\u180e", "\u0085 text \n", ""])(
  "preserves content outside the all-whitespace rule: %j",
  (text) => {
    const result = normalizedDraft({
      ...firstUnit(),
      translated_text: text,
      proofread_text: text,
    });
    expect(result.translated_text).toBe(text);
    expect(result.proofread_text).toBe(text);
  },
);

test.each(["translated_text", "proofread_text"] as const)(
  "flush converges after autosave and retains raw %s undo history",
  async (field) => {
    vi.useFakeTimers();
    const initial = editorFixture();
    initial.units = initial.units.map((unit) => ({
      ...unit,
      [field]: "original",
    }));
    const save = vi.fn<EditorApi["save"]>((snapshot) => {
      // Bound the old infinite flush loop while simulating Rust's NEL handling.
      if (save.mock.calls.length > 1) throw new Error("Repeated save");
      return Promise.resolve({
        page: {
          ...initial.page,
          unit_revision: snapshot.expected_revision + 1,
        },
        units: snapshot.units.map((unit, index) => ({
          ...unit,
          [field]: unit[field] === "\u0085" ? "" : unit[field],
          page_id: snapshot.page_id,
          index,
          created_at: 2,
          updated_at: 2,
        })),
      });
    });
    const session = createEditorSession(initial, "translation", { save });
    sessions.push(session);
    session.edit("a", { [field]: "\u0085" });
    await vi.advanceTimersByTimeAsync(15_000);
    expect(await session.flush()).toBe(true);
    expect(save).toHaveBeenCalledTimes(1);
    expect(save.mock.calls[0]?.[0].units[0]?.[field]).toBe("");
    expect(isDirty(session.store.getState())).toBe(false);
    expect(session.store.getState().draft[0]?.[field]).toBe("\u0085");
    session.cancelLeave();
    await vi.advanceTimersByTimeAsync(60_000);
    expect(save).toHaveBeenCalledTimes(1);
    session.edit("a", { [field]: "later" });
    session.undo();
    expect(session.store.getState().draft[0]?.[field]).toBe("\u0085");
    session.undo();
    expect(session.store.getState().draft[0]?.[field]).toBe("original");
    expect(session.store.getState().canUndo).toBe(false);
  },
);
