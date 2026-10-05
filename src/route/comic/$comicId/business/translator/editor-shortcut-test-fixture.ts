import type {
  ApplicationPreference,
  ComicDetail,
  ShortcutAction,
} from "@/bridge/generated/bindings";
import { editorFixture } from "./editor-test-fixture";

const bindings: [ShortcutAction, string, boolean, boolean, boolean][] = [
  ["cycle_mode", "KeyM", true, false, false],
  ["toggle_relocation", "KeyL", true, false, false],
  ["toggle_proofread_preview", "KeyX", true, false, false],
  ["next_unit", "Tab", false, false, false],
  ["previous_unit", "Tab", false, false, true],
  ["previous_page", "KeyU", true, false, false],
  ["next_page", "KeyD", true, false, false],
  ["insert_recent_symbol", "KeyQ", true, false, false],
  ["insert_favorite_one", "Digit1", false, true, false],
  ["insert_favorite_two", "Digit2", false, true, false],
  ["insert_favorite_three", "Digit3", false, true, false],
  ["save", "KeyS", true, false, false],
];
export const preference: ApplicationPreference = {
  shortcut: bindings.map(([action, key, control, alt, shift]) => ({
    action,
    binding: {
      kind: "chord",
      key,
      control,
      meta: false,
      alt,
      shift,
      scope: "editor",
    },
  })),
  special_character: ["♪", "★", "♡"].map((text, index) => ({
    id: String(index + 1),
    text,
    is_favorite: true,
  })),
  relocation_enabled: false,
  marker_opacity: 1,
};
const fixture = editorFixture();
export const detail: ComicDetail = {
  comic: {
    id: "comic",
    title: "测试",
    subtitle: "",
    author: "",
    created_at: 1,
    updated_at: 1,
  },
  pages: ["page", "second", "third"].map((id, index) => ({
    page: { ...fixture.page, id, index },
    unit_count: 3,
    translated_count: 3,
    proofread_count: 0,
    flagged_count: 0,
    edited_count: 0,
    proofreader_append_count: 0,
  })),
  work_position: null,
};
