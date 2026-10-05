import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useStore } from "zustand";
import type { ReactElement } from "react";
import type { PageEditor } from "@/bridge/generated/bindings";
import { createEditorSession } from "./editor-session";
import type { EditorSession } from "./editor-session";
import { UnitItem } from "./UnitItem";
import { EditorToolbar } from "./EditorToolbar";
import type { SymbolRequest } from "./SpecialCharsBar";

function fixture(): PageEditor {
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
        height: 100,
      },
      created_at: 1,
      updated_at: 1,
    },
    units: [
      {
        id: "unit",
        page_id: "page",
        index: 0,
        x_coord: 0.5,
        y_coord: 0.5,
        is_bubble: true,
        is_flagged: false,
        translated_text: "原来的翻译",
        proofread_text: "",
        is_proofread: false,
        created_at: 1,
        updated_at: 1,
      },
    ],
  };
}

type HarnessProps = {
  session: EditorSession;
  symbolRequest?: SymbolRequest | null;
  onSymbolInserted?: (id: string) => void;
};

function Harness({
  session,
  symbolRequest = null,
  onSymbolInserted = () => undefined,
}: HarnessProps): ReactElement {
  const state = useStore(session.store);
  const unit = state.draft[0];
  if (!unit) return <p>没有单元</p>;
  return (
    <>
      <EditorToolbar
        state={state}
        session={session}
        relocation={false}
        onRelocation={() => undefined}
        preview={true}
        onPreview={() => undefined}
        creationEnabled={true}
        onCreation={() => undefined}
        highQuality={false}
        onQuality={() => undefined}
      />
      <UnitItem
        unit={unit}
        index={0}
        mode={state.mode}
        locked={state.locked}
        selected={true}
        session={session}
        characterBar={null}
        symbolRequest={symbolRequest}
        onSymbolInserted={onSymbolInserted}
        pending={false}
        dragging={false}
        dimmed={false}
        onIndexPointerDown={() => undefined}
        characters={[{ id: "character", text: "♪", is_favorite: true }]}
        onDelete={() => undefined}
        onReorder={() => undefined}
      />
    </>
  );
}

describe("unit editing from user controls", () => {
  it("consumes an inserted symbol request so remounting never replays it", () => {
    const session = createEditorSession(fixture(), "translation", {
      save: () => Promise.reject(new Error("unused")),
    });
    session.select("unit");
    let pending: SymbolRequest | null = {
      id: "request",
      unitId: "unit",
      text: "♪",
    };
    function inserted(id: string): void {
      if (pending?.id === id) pending = null;
    }
    const view = render(
      <Harness
        key="first"
        session={session}
        symbolRequest={pending}
        onSymbolInserted={inserted}
      />,
    );
    expect(pending).toBeNull();
    view.rerender(
      <Harness
        key="second"
        session={session}
        symbolRequest={pending}
        onSymbolInserted={inserted}
      />,
    );
    expect(session.store.getState().draft[0]?.translated_text).toBe(
      "原来的翻译♪",
    );
    session.dispose();
  });
  beforeEach(() => {
    vi.stubGlobal(
      "ResizeObserver",
      class {
        observe = vi.fn();
        disconnect = vi.fn();
      },
    );
    HTMLElement.prototype.scrollIntoView = vi.fn();
    vi.useFakeTimers();
  });
  afterEach(() => {
    cleanup();
    Reflect.deleteProperty(document, "fonts");
    Reflect.deleteProperty(Range.prototype, "getBoundingClientRect");
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it("copies into empty proofreading without confirming and never overwrites existing proofreading", () => {
    const session = createEditorSession(fixture(), "proofreading", {
      save: () => Promise.reject(new Error("not used")),
    });
    render(<Harness session={session} />);
    fireEvent.click(screen.getByRole("button", { name: "复制翻译到空校对" }));
    expect(session.store.getState().draft[0]?.proofread_text).toBe(
      "原来的翻译",
    );
    expect(session.store.getState().draft[0]?.is_proofread).toBe(false);
    fireEvent.change(screen.getByRole("textbox", { name: "校对 1" }), {
      target: { value: "独立的校对" },
    });
    const copy = screen.getByRole("button", { name: "复制翻译到空校对" });
    expect(copy instanceof HTMLButtonElement && copy.disabled).toBe(true);
    fireEvent.click(copy);
    expect(session.store.getState().draft[0]?.proofread_text).toBe(
      "独立的校对",
    );
    session.dispose();
  });

  it("keeps confirmation when proofreading text changes and inserts symbols at the selection", () => {
    const session = createEditorSession(fixture(), "proofreading", {
      save: () => Promise.reject(new Error("not used")),
    });
    render(<Harness session={session} />);
    fireEvent.click(screen.getByRole("button", { name: "确认校对" }));
    const input = screen.getByRole("textbox", { name: "校对 1" });
    fireEvent.change(input, { target: { value: "你好世界" } });
    if (!(input instanceof HTMLTextAreaElement))
      throw new Error("missing textarea");
    input.setSelectionRange(2, 4);
    fireEvent.click(screen.getByRole("button", { name: "♪" }));
    expect(session.store.getState().draft[0]?.proofread_text).toBe("你好♪");
    expect(session.store.getState().draft[0]?.is_proofread).toBe(true);
    session.dispose();
  });

  it("allows leaving readonly while blocking content controls", () => {
    const session = createEditorSession(fixture(), "readonly", {
      save: () => Promise.reject(new Error("not used")),
    });
    render(<Harness session={session} />);
    expect(screen.queryByRole("button", { name: "关注单元" })).toBeNull();
    expect(session.store.getState().draft[0]?.is_flagged).toBe(false);
    fireEvent.click(
      screen.getByRole("button", { name: "当前只读模式，切换模式" }),
    );
    fireEvent.change(screen.getByRole("textbox", { name: "翻译 1" }), {
      target: { value: "可以继续翻译" },
    });
    expect(session.store.getState().draft[0]?.translated_text).toBe(
      "可以继续翻译",
    );
    session.dispose();
  });

  it("restores multiline height when returning from readonly", () => {
    // jsdom has no font or text-range layout; provide only the measurement surface.
    Object.defineProperty(document, "fonts", {
      configurable: true,
      value: {
        status: "loaded",
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
      },
    });
    Range.prototype.getBoundingClientRect = () => new DOMRect(0, 0, 1, 24);
    const initial = fixture();
    const unit = initial.units[0];
    if (!unit) throw new Error("missing fixture");
    unit.translated_text = "第一行\n第二行";
    vi.spyOn(
      HTMLTextAreaElement.prototype,
      "scrollHeight",
      "get",
    ).mockReturnValue(64);
    const session = createEditorSession(initial, "readonly", {
      save: () => Promise.reject(new Error("not used")),
    });
    render(<Harness session={session} />);
    fireEvent.click(
      screen.getByRole("button", { name: "当前只读模式，切换模式" }),
    );
    const input = screen.getByRole("textbox", { name: "翻译 1" });
    expect(input instanceof HTMLTextAreaElement && input.style.height).toBe(
      "64px",
    );
    session.dispose();
  });
});
