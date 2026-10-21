import { useCallback, useState } from "react";
import { commands, unwrap } from "@/bridge";
import type {
  Term,
  Termbase,
  TermbaseInfo,
  TerminologyContext,
} from "@/bridge/generated/bindings";
import { useResource } from "@/shared/hook/use-resource";

export type TerminologyEditor =
  | { kind: "base"; value: Termbase | null }
  | { kind: "term"; value: Term | null; termbaseId: string };

export function useTerminology(access: TerminologyContext): {
  access: TerminologyContext;
  selectedId: string | null;
  selection: ReturnType<typeof useResource<TermbaseInfo | null>>;
  baseQuery: string;
  setBaseQuery: (value: string) => void;
  termQuery: string;
  setTermQuery: (value: string) => void;
  baseRevision: number;
  termRevision: number;
  editor: TerminologyEditor | null;
  setEditor: (value: TerminologyEditor | null) => void;
  select: (id: string) => void;
  savedBase: (base: Termbase) => void;
  deletedBase: (id: string) => void;
  savedTerm: (created: boolean) => void;
} {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [baseQuery, setBaseQuery] = useState("");
  const [termQuery, setTermQuery] = useState("");
  const [baseRevision, setBaseRevision] = useState(0);
  const [termRevision, setTermRevision] = useState(0);
  const [editor, setEditor] = useState<TerminologyEditor | null>(null);
  const load = useCallback(
    async () =>
      selectedId === null
        ? null
        : unwrap(await commands.getTermbase(access, selectedId)),
    [access, selectedId],
  );
  const selection = useResource(load);
  function select(id: string): void {
    if (id !== selectedId) setTermQuery("");
    setSelectedId(id);
  }
  function savedBase(base: Termbase): void {
    setBaseRevision((value) => value + 1);
    if (editor?.kind === "base" && editor.value === null) setBaseQuery("");
    if (selectedId === base.id) selection.reload();
    setEditor(null);
  }
  function deletedBase(id: string): void {
    setBaseRevision((value) => value + 1);
    if (selectedId === id) {
      setSelectedId(null);
      setTermQuery("");
    }
    setEditor(null);
  }
  function savedTerm(created: boolean): void {
    setBaseRevision((value) => value + 1);
    setTermRevision((value) => value + 1);
    selection.reload();
    if (created) setTermQuery("");
    setEditor(null);
  }
  return {
    access,
    selectedId,
    selection,
    baseQuery,
    setBaseQuery,
    termQuery,
    setTermQuery,
    baseRevision,
    termRevision,
    editor,
    setEditor,
    select,
    savedBase,
    deletedBase,
    savedTerm,
  };
}

export type TerminologyController = ReturnType<typeof useTerminology>;
