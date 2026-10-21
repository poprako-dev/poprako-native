import { useCallback, useEffect, useRef, useState } from "react";

type Page<T> = { items: T[]; loading: boolean; more: boolean; error: string };
type Snapshot<T> = Page<T> & { load: (offset: number) => Promise<T[]> };

export function useDebouncedQuery(value: string, delay: number): string {
  const [query, setQuery] = useState(value);
  useEffect(() => {
    const timer = setTimeout(() => {
      setQuery(value);
    }, delay);
    return () => {
      clearTimeout(timer);
    };
  }, [value, delay]);
  return query.trim();
}

export function useTermList<T>(
  load: (offset: number) => Promise<T[]>,
): Page<T> & { next: () => void } {
  const [snapshot, setSnapshot] = useState<Snapshot<T>>({
    load,
    items: [],
    loading: true,
    more: true,
    error: "",
  });
  const current = useRef({ load, busy: false, offset: 0 });
  const request = useCallback(
    async (first: boolean): Promise<void> => {
      const active = current.current;
      if (active.load !== load || active.busy) return;
      active.busy = true;
      const offset = first ? 0 : active.offset;
      setSnapshot((state) => ({
        ...state,
        items: first ? [] : state.items,
        load,
        loading: true,
        error: "",
      }));
      try {
        const items = await load(offset);
        if (current.current !== active) return;
        active.offset = offset + items.length;
        setSnapshot((state) => ({
          load,
          items: first ? items : [...state.items, ...items],
          loading: false,
          more: items.length === 30,
          error: "",
        }));
      } catch (error) {
        if (current.current !== active) return;
        setSnapshot((state) => ({
          ...state,
          load,
          loading: false,
          error: error instanceof Error ? error.message : "读取失败，请重试。",
        }));
      } finally {
        active.busy = false;
      }
    },
    [load],
  );
  useEffect(() => {
    current.current = {
      load,
      busy: false,
      offset: 0,
    };
    void request(true);
    return () => {
      current.current = {
        ...current.current,
      };
    };
  }, [load, request]);
  const state =
    snapshot.load === load
      ? snapshot
      : { items: [], loading: true, more: true, error: "" };
  return {
    ...state,
    next: () => {
      if (!state.loading && state.more)
        void request(current.current.offset === 0);
    },
  };
}
