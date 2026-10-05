import { useEffect, useLayoutEffect, useRef } from "react";
import type {
  ApplicationPreference,
  ComicDetail,
} from "@/bridge/generated/bindings";
import type { EditorSession } from "./editor-session";
import {
  matchesEditorShortcut,
  shouldIgnoreEditorKey,
} from "./editor-keyboard";

type Options = {
  session: EditorSession;
  detail: ComicDetail;
  preference: ApplicationPreference;
  disabled: boolean;
  onPage: (id: string) => void;
  onRelocation: () => void;
  onPreview: () => void;
  onSymbol: (text: string) => void;
};

export function useEditorShortcuts(options: Options): void {
  const latest = useRef(options);
  useLayoutEffect(() => {
    latest.current = options;
  });
  useEffect(() => {
    function keydown(event: KeyboardEvent): void {
      const {
        session,
        detail,
        preference,
        disabled,
        onPage,
        onRelocation,
        onPreview,
        onSymbol,
      } = latest.current;
      if (disabled || session.isComposing() || shouldIgnoreEditorKey(event))
        return;
      const state = session.store.getState();
      if (state.locked || state.uncertain) return;
      if (document.querySelector('[role="dialog"],[role="alertdialog"]'))
        return;
      if (
        state.mode === "readonly" &&
        document.querySelector('[aria-label="页面统计详情"]')
      )
        return;
      const shortcut = preference.shortcut.find((item) =>
        matchesEditorShortcut(event, item.binding),
      );
      if (!shortcut) {
        if (event.key === "Escape") session.select("");
        return;
      }
      if (
        state.mode === "readonly" &&
        ![
          "next_unit",
          "previous_unit",
          "next_page",
          "previous_page",
          "cycle_mode",
          "toggle_proofread_preview",
        ].includes(shortcut.action)
      )
        return;
      event.preventDefault();
      switch (shortcut.action) {
        case "save":
          void session.save();
          return;
        case "cycle_mode":
          session.setMode(
            state.mode === "translation"
              ? "proofreading"
              : state.mode === "proofreading"
                ? "readonly"
                : "translation",
          );
          return;
        case "previous_unit":
        case "next_unit": {
          if (state.draft.length === 0) return;
          const index = state.draft.findIndex(
            (unit) => unit.id === state.selected,
          );
          const next =
            shortcut.action === "next_unit"
              ? (index + 1) % state.draft.length
              : index <= 0
                ? state.draft.length - 1
                : index - 1;
          const unit = state.draft[next];
          if (unit) session.select(unit.id);
          return;
        }
        case "previous_page":
        case "next_page": {
          const index = detail.pages.findIndex(
            (entry) => entry.page.id === state.baseline.page.id,
          );
          const page =
            detail.pages[index + (shortcut.action === "next_page" ? 1 : -1)];
          if (page) onPage(page.page.id);
          return;
        }
        case "toggle_relocation":
          onRelocation();
          return;
        case "toggle_proofread_preview":
          onPreview();
          return;
        case "insert_recent_symbol": {
          const text =
            state.recentSymbol || preference.special_character[0]?.text;
          if (text && state.selected) onSymbol(text);
          return;
        }
        case "insert_favorite_one":
        case "insert_favorite_two":
        case "insert_favorite_three": {
          const index = [
            "insert_favorite_one",
            "insert_favorite_two",
            "insert_favorite_three",
          ].indexOf(shortcut.action);
          const character = preference.special_character.filter(
            (item) => item.is_favorite,
          )[index];
          if (character && state.selected) onSymbol(character.text);
          return;
        }
      }
    }
    window.addEventListener("keydown", keydown);
    return () => {
      window.removeEventListener("keydown", keydown);
    };
  }, []);
}
