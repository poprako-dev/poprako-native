import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import type { PageInfo } from "@/bridge/generated/bindings";
import { createEditorSession } from "./editor-session";
import { editorFixture } from "./editor-test-fixture";
import { countUnits, nextEditedPage, pageStatistics } from "./page-statistic";
import type { PageStatistic } from "./page-statistic";
import { EditorPaginator } from "./EditorPaginator";

function pages(): PageStatistic[] {
  return ["page", "second", "third"].map((id, index) => ({
    id,
    index,
    total: 3,
    translated: 2,
    confirmed: 1,
    flagged: 1,
    edited: index === 0 ? 1 : 0,
    appended: index === 2 ? 1 : 0,
    dirty: index === 0,
  }));
}

describe("page counts and navigation", () => {
  afterEach(cleanup);
  it("counts proofreading edits, additions and confirmation independently", () => {
    const units = editorFixture().units;
    const result = countUnits(
      units.map((unit, index) => ({
        ...unit,
        translated_text: index === 2 ? " \t\n" : "原文",
        proofread_text: index === 1 ? "原文" : "校对",
        is_proofread: true,
      })),
    );
    expect(result).toMatchObject({
      total: 3,
      translated: 2,
      confirmed: 3,
      edited: 1,
      appended: 1,
    });
    expect(countUnits([])).toMatchObject({ total: 0, edited: 0, appended: 0 });
  });

  it("uses live drafts only for the open page without mutating readonly content", () => {
    const fixture = editorFixture();
    const session = createEditorSession(fixture, "translation", {
      save: () => Promise.reject(new Error("unused")),
    });
    const details: PageInfo[] = ["page", "second"].map((id) => ({
      page: { ...fixture.page, id },
      unit_count: 9,
      translated_count: 8,
      proofread_count: 7,
      flagged_count: 6,
      edited_count: 5,
      proofreader_append_count: 4,
    }));
    session.edit("a", { proofread_text: "修改" });
    session.setMode("readonly");
    const before = session.store.getState().draft;
    const result = pageStatistics(details, session.store.getState());
    expect(result[0]).toMatchObject({ total: 3, edited: 1, dirty: true });
    expect(result[1]).toMatchObject({
      total: 9,
      edited: 5,
      appended: 4,
      dirty: false,
    });
    expect(session.store.getState().draft).toEqual(before);
    session.dispose();
  });

  it("finds only a later edited or appended page and never wraps", () => {
    expect(nextEditedPage(pages(), "page")?.id).toBe("third");
    expect(nextEditedPage(pages(), "third")).toBeUndefined();
    expect(nextEditedPage(pages(), "missing")).toBeUndefined();
  });

  it("navigates from the floating list, dismisses with Escape and restores focus", () => {
    HTMLElement.prototype.scrollIntoView = vi.fn();
    const onPage = vi.fn<(id: string) => void>();
    render(
      <EditorPaginator
        pages={pages()}
        currentId="page"
        locked={false}
        onPage={onPage}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "上一页" }));
    expect(onPage).not.toHaveBeenCalled();
    const trigger = screen.getByRole("button", { name: "展开页面列表" });
    fireEvent.click(trigger);
    expect(screen.getByLabelText("有未保存草稿")).toBeTruthy();
    fireEvent.keyDown(document, { key: "Escape" });
    expect(screen.queryByLabelText("页面列表")).toBeNull();
    expect(document.activeElement).toBe(trigger);
    fireEvent.click(trigger);
    fireEvent.click(screen.getByRole("button", { name: /^第 3 页/ }));
    expect(onPage).toHaveBeenLastCalledWith("third");
    expect(screen.queryByLabelText("页面列表")).toBeNull();
  });
});
