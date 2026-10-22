import type { ReactElement } from "react";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
} from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { useTermPress } from "./use-term-press";

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

it("distinguishes hover, long press, scrolling, and ordinary activation", () => {
  vi.useFakeTimers();
  vi.stubGlobal("PointerEvent", MouseEvent);
  const edit = vi.fn();
  const select = vi.fn();
  function Row(): ReactElement {
    const press = useTermPress(true, edit);
    return (
      <button
        type="button"
        {...press.handlers}
        onClick={() => {
          if (press.activate()) select();
        }}
      >
        术语库
      </button>
    );
  }
  render(<Row />);
  const row = screen.getByRole("button");
  fireEvent.pointerMove(row, { clientX: 100, clientY: 100 });
  fireEvent.click(row);
  expect(select).toHaveBeenCalledTimes(1);
  fireEvent.pointerDown(row, { button: 0, clientX: 100, clientY: 100 });
  act(() => {
    vi.advanceTimersByTime(500);
  });
  expect(edit).toHaveBeenCalledTimes(1);
  fireEvent.pointerUp(row);
  fireEvent.click(row);
  expect(select).toHaveBeenCalledTimes(1);
  fireEvent.pointerDown(row, { button: 0, clientX: 100, clientY: 100 });
  fireEvent.pointerMove(row, { clientX: 100, clientY: 110 });
  act(() => {
    vi.advanceTimersByTime(500);
  });
  fireEvent.pointerUp(row);
  fireEvent.click(row);
  expect(edit).toHaveBeenCalledTimes(1);
  expect(select).toHaveBeenCalledTimes(1);
  fireEvent.pointerDown(row, { button: 0 });
  fireEvent.pointerUp(row);
  fireEvent.click(row);
  expect(select).toHaveBeenCalledTimes(2);
});
