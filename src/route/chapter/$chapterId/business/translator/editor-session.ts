import { createStore } from "zustand/vanilla";
import type { StoreApi } from "zustand";
import type {
  EditorMode,
  PageEditor,
  SavePageUnits,
  UnitDraft,
} from "@/bridge/generated/bindings";

import { createUnitHistory } from "./unit-history";

export interface EditorApi {
  save(snapshot: SavePageUnits): Promise<PageEditor>;
}

export type EditorState = {
  baseline: PageEditor;
  draft: UnitDraft[];
  generation: number;
  canUndo: boolean;
  mode: EditorMode;
  selected: string;
  recentSymbol: string;
  locateRevision: number;
  busy: boolean;
  locked: boolean;
  error: string;
  uncertain: boolean;
};

export interface EditorActions {
  undo(): void;
  clearHistory(): void;
  edit(id: string, patch: Partial<UnitDraft>): void;
  replace(units: UnitDraft[]): void;
  select(id: string): void;
  locate(id: string): void;
  useSymbol(text: string): void;
  reload(page: PageEditor): void;
  setMode(mode: EditorMode): void;
  isComposing(): boolean;
  suspend(value: boolean, reason?: "composition" | "gesture"): void;
  save(): Promise<boolean>;
  flush(): Promise<boolean>;
  cancelLeave(): void;
  discard(): boolean;
  resume(): void;
  dispose(): void;
}

export type EditorSession = EditorActions & { store: StoreApi<EditorState> };

export function unitDraft(unit: UnitDraft): UnitDraft {
  return {
    id: unit.id,
    x_coord: unit.x_coord,
    y_coord: unit.y_coord,
    is_bubble: unit.is_bubble,
    is_flagged: unit.is_flagged,
    translated_text: unit.translated_text,
    proofread_text: unit.proofread_text,
    is_proofread: unit.is_proofread,
  };
}

export function normalizedDraft(unit: UnitDraft): UnitDraft {
  // Match Rust char::is_whitespace; JavaScript trim also removes U+FEFF
  // and misses U+0085. Keep nonblank content and the original draft intact.
  const blank = /^\p{White_Space}*$/u;
  return {
    ...unitDraft(unit),
    translated_text: blank.test(unit.translated_text)
      ? ""
      : unit.translated_text,
    proofread_text: blank.test(unit.proofread_text) ? "" : unit.proofread_text,
  };
}

export function isDirty(state: EditorState): boolean {
  return (
    JSON.stringify(state.draft.map(normalizedDraft)) !==
    JSON.stringify(state.baseline.units.map(normalizedDraft))
  );
}

export function createEditorSession(
  initial: PageEditor,
  mode: EditorMode,
  api: EditorApi,
): EditorSession {
  const store = createStore<EditorState>(() => ({
    baseline: initial,
    draft: initial.units.map(unitDraft),
    generation: 0,
    canUndo: false,
    mode,
    selected: "",
    recentSymbol: "",
    locateRevision: 0,
    busy: false,
    locked: false,
    error: "",
    uncertain: false,
  }));
  let timer: ReturnType<typeof setTimeout> | undefined;
  let inflight: Promise<boolean> | undefined;
  let interaction: Promise<boolean> | undefined;
  let finishInteraction: ((ready: boolean) => void) | undefined;
  const suspension = new Set<string>();
  const history = createUnitHistory();
  let disposed = false;
  let uncertainSubmission: EditorState | null = null;

  function clearTimer(): void {
    clearTimeout(timer);
    timer = undefined;
  }

  function releaseInteraction(ready: boolean): void {
    const finish = finishInteraction;
    interaction = undefined;
    finishInteraction = undefined;
    finish?.(ready);
  }

  async function waitForInteraction(): Promise<boolean> {
    while (!disposed && suspension.size > 0) {
      interaction ??= new Promise<boolean>((resolve) => {
        finishInteraction = resolve;
      });
      if (!(await interaction)) return false;
    }
    return !disposed;
  }

  function schedule(): void {
    clearTimer();
    const state = store.getState();
    if (
      disposed ||
      suspension.size > 0 ||
      state.locked ||
      state.error ||
      !isDirty(state)
    )
      return;
    timer = setTimeout(() => {
      void save();
    }, 15_000);
  }

  function replace(draft: UnitDraft[]): void {
    const state = store.getState();
    if (
      disposed ||
      state.locked ||
      state.uncertain ||
      state.mode === "readonly"
    )
      return;
    if (!history.record(state.draft, draft)) return;
    store.setState({
      draft,
      generation: state.generation + 1,
      canUndo: history.canUndo(),
    });
    schedule();
  }

  async function commit(submitted: EditorState): Promise<boolean> {
    // Register inflight before store subscribers or the API can reenter save.
    await Promise.resolve();
    try {
      if (disposed) return false;
      store.setState({ busy: true, error: "" });
      const result = await api.save({
        chapter_id: submitted.baseline.page.chapter_id,
        page_id: submitted.baseline.page.id,
        expected_revision: submitted.baseline.page.unit_revision,
        units: submitted.draft.map(normalizedDraft),
      });
      // dispose() can run while the native save is pending.
      // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
      if (disposed) return false;
      // A receipt advances only the submitted baseline, never subsequent edits.
      uncertainSubmission = null;
      store.setState({
        baseline: result,
        error: "",
        uncertain: false,
      });
      return true;
    } catch (error) {
      if (disposed) return false;
      const uncertain =
        error instanceof Error &&
        "detail" in error &&
        typeof error.detail === "object" &&
        error.detail !== null &&
        "recovery" in error.detail &&
        error.detail.recovery === "wait_for_confirmation";
      uncertainSubmission = uncertain ? submitted : null;
      store.setState({
        uncertain,
        error: error instanceof Error ? error.message : "保存失败，请重试。",
      });
      clearTimer();
      return false;
    } finally {
      inflight = undefined;
      store.setState({ busy: false });
      schedule();
    }
  }

  async function save(): Promise<boolean> {
    clearTimer();
    if (disposed) return false;
    if (suspension.size > 0) {
      if (!(await waitForInteraction())) return false;
      return save();
    }
    if (inflight) {
      if (!(await inflight)) return false;
      return save();
    }
    const submitted = uncertainSubmission ?? store.getState();
    if (!isDirty(submitted)) return true;
    inflight = commit(submitted);
    return inflight;
  }

  async function flush(): Promise<boolean> {
    if (disposed) return false;
    if (suspension.size > 0 && !(await waitForInteraction())) return false;
    store.setState({ locked: true });
    do {
      if (!(await save())) return false;
    } while (isDirty(store.getState()));
    return true;
  }

  return {
    store,
    replace,
    save,
    flush,
    clearHistory(): void {
      history.clear();
      store.setState({ canUndo: false });
    },
    undo(): void {
      const state = store.getState();
      if (
        disposed ||
        state.locked ||
        state.uncertain ||
        state.mode === "readonly" ||
        suspension.size > 0
      )
        return;
      const draft = history.undo(state.draft);
      if (!draft) return;
      store.setState({
        draft,
        generation: state.generation + 1,
        canUndo: history.canUndo(),
        selected: draft.some((unit) => unit.id === state.selected)
          ? state.selected
          : "",
      });
      schedule();
    },
    edit(id, patch): void {
      replace(
        store
          .getState()
          .draft.map((unit) =>
            unit.id === id ? { ...unit, ...patch, id } : unit,
          ),
      );
    },
    select(id): void {
      store.setState({ selected: id });
    },
    locate(id): void {
      store.setState({
        selected: id,
        locateRevision: store.getState().locateRevision + 1,
      });
    },
    useSymbol(text): void {
      store.setState({ recentSymbol: text });
    },
    reload(page): void {
      if (store.getState().busy || uncertainSubmission) return;
      clearTimer();
      history.clear();
      store.setState({
        canUndo: false,
        selected: page.units.some(
          (unit) => unit.id === store.getState().selected,
        )
          ? store.getState().selected
          : "",
        baseline: page,
        draft: page.units.map(unitDraft),
        generation: store.getState().generation + 1,
        error: "",
        uncertain: false,
      });
    },
    setMode(next): void {
      store.setState({ mode: next });
    },
    isComposing(): boolean {
      return suspension.has("composition");
    },
    suspend(value, reason = "composition"): void {
      if (value) suspension.add(reason);
      else suspension.delete(reason);
      if (value) clearTimer();
      else {
        if (suspension.size === 0) releaseInteraction(true);
        schedule();
      }
    },
    cancelLeave(): void {
      store.setState({ locked: false });
      schedule();
    },
    discard(): boolean {
      const state = store.getState();
      if (state.busy || state.uncertain) return false;
      clearTimer();
      history.clear();
      store.setState({
        canUndo: false,
        selected: state.baseline.units.some(
          (unit) => unit.id === state.selected,
        )
          ? state.selected
          : "",
        draft: state.baseline.units.map(unitDraft),
        error: "",
        locked: false,
      });
      return true;
    },
    resume(): void {
      disposed = false;
      schedule();
    },
    dispose(): void {
      disposed = true;
      releaseInteraction(false);
      clearTimer();
      history.clear();
      store.setState({ canUndo: false });
    },
  };
}
