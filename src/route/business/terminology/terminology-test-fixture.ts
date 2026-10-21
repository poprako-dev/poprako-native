import { vi } from "vitest";
import { commands } from "@/bridge";
import type { Term, TermbaseInfo } from "@/bridge/generated/bindings";

export const globalBase: TermbaseInfo = {
  termbase: {
    id: "global",
    scope: { kind: "global" },
    name: "奇幻世界",
    description: "共用词",
    created_at: 1,
    updated_at: 1,
  },
  term_count: 1,
};
export const localBase: TermbaseInfo = {
  termbase: {
    ...globalBase.termbase,
    id: "local",
    name: "本作人名",
    scope: { kind: "comic", comic_id: "comic" },
  },
  term_count: 1,
};
export const term: Term = {
  id: "term",
  termbase_id: "global",
  source: "団長",
  targets: ["团长", "队长"],
  comment: "称呼",
  created_at: 1,
  updated_at: 1,
};

export function setupTerminology(): void {
  vi.stubGlobal(
    "IntersectionObserver",
    class {
      observe(): void {
        /* Explicit load-more buttons drive deterministic tests. */
      }
      disconnect(): void {
        /* No observer resources are allocated. */
      }
    },
  );
  vi.stubGlobal(
    "matchMedia",
    vi.fn(() => ({
      matches: true,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })),
  );
  vi.spyOn(commands, "listTermbaseInfos").mockResolvedValue({
    status: "ok",
    data: [globalBase, localBase],
  });
  vi.spyOn(commands, "getTermbase").mockImplementation((_access, id) =>
    Promise.resolve({
      status: "ok",
      data: id === "local" ? localBase : globalBase,
    }),
  );
  vi.spyOn(commands, "listTermInfos").mockImplementation((_access, id) =>
    Promise.resolve({ status: "ok", data: [{ ...term, termbase_id: id }] }),
  );
  vi.spyOn(commands, "copyTermTarget").mockResolvedValue({
    status: "ok",
    data: null,
  });
}
