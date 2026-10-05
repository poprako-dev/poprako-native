import { createContext, useContext } from "react";
import { useStore } from "zustand";
import type { StoreApi } from "zustand";
import type { ApplicationPreference } from "@/bridge/generated/bindings";

export type PreferenceContextValue = {
  preference: ApplicationPreference;
  save: (preference: ApplicationPreference) => Promise<void>;
};

export type PreferenceState = { preference: ApplicationPreference };

export type PreferenceSource = {
  store: StoreApi<PreferenceState>;
  save: PreferenceContextValue["save"];
};

export const PreferenceContext = createContext<PreferenceSource | null>(null);

export function usePreference(): PreferenceContextValue {
  const value = useContext(PreferenceContext);
  if (value === null) {
    throw new Error("Preference provider is missing.");
  }
  const preference = useStore(value.store, (state) => state.preference);
  return { preference, save: value.save };
}
