import { useCallback, useMemo, useRef, useState } from "react";
import type { ReactElement, ReactNode } from "react";
import { createStore } from "zustand";
import { commands, unwrap } from "@/bridge";
import type { ApplicationPreference } from "@/bridge/generated/bindings";
import { PreferenceContext } from "@/route/business/preference-context";
import type { PreferenceState } from "@/route/business/preference-context";

type PreferenceProviderProps = {
  initial: ApplicationPreference;
  children: ReactNode;
};

export function PreferenceProvider({
  initial,
  children,
}: PreferenceProviderProps): ReactElement {
  const [store] = useState(() =>
    createStore<PreferenceState>(() => ({ preference: initial })),
  );
  const baseline = useRef(initial);
  const inFlight = useRef(false);
  const save = useCallback(
    async (next: ApplicationPreference): Promise<void> => {
      if (inFlight.current) {
        throw new Error("请等待当前设置保存完成。");
      }
      inFlight.current = true;
      try {
        const committed = unwrap(
          await commands.updatePreference(baseline.current, next),
        );
        baseline.current = committed;
        store.setState({ preference: committed });
      } finally {
        inFlight.current = false;
      }
    },
    [store],
  );
  const value = useMemo(() => ({ store, save }), [store, save]);
  return <PreferenceContext value={value}>{children}</PreferenceContext>;
}
