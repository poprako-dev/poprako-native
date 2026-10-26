import { StrictMode } from "react";
import { createStore } from "zustand/vanilla";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import {
  createMemoryHistory,
  createRootRoute,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { clearMocks, mockConvertFileSrc } from "@tauri-apps/api/mocks";
import { commands } from "@/bridge";
import { commands as native } from "@/bridge/generated/bindings";
import type { PageEditor, SavePageUnits } from "@/bridge/generated/bindings";
import { PreferenceContext } from "@/route/business/preference-context";
import { TranslatorPage } from "./TranslatorPage";
import { detail, preference } from "./editor-shortcut-test-fixture";

class TestPointerEvent extends MouseEvent {
  readonly pointerId = 1;
  readonly isPrimary = true;
  readonly pointerType = "mouse";
}

const pages = new Map<string, PageEditor>();

function receipt(snapshot: SavePageUnits): PageEditor {
  const before = pages.get(snapshot.page_id);
  if (!before) throw new Error("Unknown page");
  const result = {
    page: { ...before.page, unit_revision: snapshot.expected_revision + 1 },
    units: snapshot.units.map((unit, index) => ({
      ...unit,
      index,
      page_id: snapshot.page_id,
      created_at: 1,
      updated_at: 2,
    })),
  };
  pages.set(snapshot.page_id, result);
  return result;
}

beforeEach(() => {
  pages.clear();
  for (const entry of detail.pages)
    pages.set(entry.page.id, { page: entry.page, units: [] });
  mockConvertFileSrc("windows");
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal("PointerEvent", TestPointerEvent);
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe = vi.fn();
      disconnect = vi.fn();
    },
  );
  HTMLElement.prototype.scrollIntoView = vi.fn();
  HTMLElement.prototype.setPointerCapture = vi.fn();
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue(
    new DOMRect(0, 0, 100, 100),
  );
  vi.spyOn(commands, "getChapterDetail").mockResolvedValue({
    status: "ok",
    data: detail,
  });
  vi.spyOn(commands, "getPageEditor").mockImplementation((_chapter, id) => {
    const page = pages.get(id);
    if (!page) throw new Error("Unknown page");
    return Promise.resolve({ status: "ok", data: page });
  });
  vi.spyOn(commands, "getImageResource").mockResolvedValue({
    status: "ok",
    data: { handle: "image", width: 100, height: 100 },
  });
  vi.spyOn(commands, "releaseImageResources").mockResolvedValue({
    status: "ok",
    data: null,
  });
  vi.spyOn(native, "updateWorkPosition").mockImplementation((position) =>
    Promise.resolve({ status: "ok", data: position }),
  );
  let sequence = 1;
  vi.spyOn(native, "getWriteSession").mockImplementation(() =>
    Promise.resolve({
      status: "ok",
      data: { session_id: "test", next_sequence: sequence },
    }),
  );
  vi.spyOn(native, "acknowledgeWriteResult").mockImplementation(() => {
    sequence += 1;
    return Promise.resolve({ status: "ok", data: null });
  });
  vi.spyOn(native, "savePageUnits").mockImplementation((snapshot) =>
    Promise.resolve({ status: "ok", data: receipt(snapshot) }),
  );
});

afterEach(async () => {
  cleanup();
  await Promise.resolve();
  clearMocks();
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

async function open(): Promise<void> {
  const root = createRootRoute({
    component: () => <TranslatorPage chapterId="chapter" />,
  });
  const router = createRouter({
    routeTree: root,
    history: createMemoryHistory(),
  });
  await router.load();
  render(
    <StrictMode>
      <PreferenceContext.Provider
        value={{
          store: createStore(() => ({ preference })),
          save: () => Promise.resolve(),
        }}
      >
        <RouterProvider router={router} />
      </PreferenceContext.Provider>
    </StrictMode>,
  );
  await screen.findByAltText("第 1 页漫画");
}

function point(x: number): void {
  const image = screen.getByRole("img", { name: /第 \d+ 页漫画/ });
  fireEvent.pointerDown(image, { button: 0, clientX: x, clientY: 50 });
  fireEvent.pointerUp(image, { button: 0, clientX: x, clientY: 50 });
  fireEvent.click(image, { button: 0, clientX: x, clientY: 50 });
}

test("manual save through the real page and write queue finishes and subsequent marking stays on the page", async () => {
  await open();
  point(20);
  fireEvent.click(screen.getByRole("button", { name: "保存，待保存" }));
  await screen.findByRole("button", { name: "保存，已保存" });
  point(70);
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(screen.getByAltText("第 1 页漫画")).toBeDefined();
  expect(screen.getAllByRole("textbox", { name: /翻译 \d/ })).toHaveLength(2);
  fireEvent.click(screen.getByRole("button", { name: "下一页" }));
  await screen.findByAltText("第 2 页漫画");
  expect(pages.get("page")?.units).toHaveLength(2);
});

test("a marker at a fractional pixel ratio saves once before turning the page", async () => {
  await open();
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue(
    new DOMRect(0, 0, 997, 997),
  );
  point(1);
  fireEvent.click(screen.getByRole("button", { name: "下一页" }));
  await screen.findByAltText("第 2 页漫画");
  expect(native.savePageUnits).toHaveBeenCalledTimes(1);
  expect(pages.get("page")?.units[0]?.x_coord).toBe(1 / 997);
  expect(screen.queryByRole("dialog")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "上一页" }));
  await screen.findByAltText("第 1 页漫画");
  expect(screen.getByRole("button", { name: "保存，已保存" })).toBeDefined();
});

test("the real page buffers marks and text while its manual native save is pending", async () => {
  let finish = (): void => {
    throw new Error("Save not started");
  };
  vi.mocked(native.savePageUnits).mockImplementationOnce(
    (snapshot) =>
      new Promise((resolve) => {
        finish = () => {
          resolve({ status: "ok", data: receipt(snapshot) });
        };
      }),
  );
  await open();
  point(20);
  fireEvent.click(screen.getByRole("button", { name: "保存，待保存" }));
  await waitFor(() => {
    expect(native.savePageUnits).toHaveBeenCalledTimes(1);
  });
  point(70);
  fireEvent.change(screen.getByRole("textbox", { name: "翻译 2" }), {
    target: { value: "新的译文" },
  });
  expect(screen.queryByRole("dialog")).toBeNull();
  await act(async () => {
    finish();
    await Promise.resolve();
  });
  await screen.findByRole("button", { name: "保存，待保存" });
  fireEvent.click(screen.getByRole("button", { name: "保存，待保存" }));
  await screen.findByRole("button", { name: "保存，已保存" });
  expect(pages.get("page")?.units[1]?.translated_text).toBe("新的译文");
  expect(screen.getByAltText("第 1 页漫画")).toBeDefined();
});

test("turning the page during autosave cannot be turned into a save failure by another canvas pointerdown", async () => {
  let finish = (): void => {
    throw new Error("Save not started");
  };
  vi.mocked(native.savePageUnits).mockImplementationOnce(
    (snapshot) =>
      new Promise((resolve) => {
        finish = () => {
          resolve({ status: "ok", data: receipt(snapshot) });
        };
      }),
  );
  await open();
  vi.useFakeTimers();
  point(20);
  await act(() => vi.advanceTimersByTimeAsync(15_000));
  vi.useRealTimers();
  expect(native.savePageUnits).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole("button", { name: "下一页" }));
  const image = screen.getByAltText("第 1 页漫画");
  fireEvent.pointerDown(image, { button: 0, clientX: 70, clientY: 50 });
  await act(async () => {
    finish();
    await Promise.resolve();
  });
  await screen.findByAltText("第 2 页漫画");
  expect(screen.queryByRole("dialog")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "上一页" }));
  await screen.findByAltText("第 1 页漫画");
  expect(screen.getAllByRole("textbox", { name: /翻译 \d/ })).toHaveLength(1);
  expect(pages.get("page")?.units).toHaveLength(1);
});
