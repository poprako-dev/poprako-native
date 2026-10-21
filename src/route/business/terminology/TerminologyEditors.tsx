import type { ReactElement } from "react";
import type { Termbase } from "@/bridge/generated/bindings";
import { TermbaseEditor } from "./TermbaseEditor";
import { TermEditor } from "./TermEditor";
import type { TerminologyController } from "./use-terminology";

type Props = {
  state: TerminologyController;
  onBaseSaved: (base: Termbase) => void;
};

export function TerminologyEditors({
  state,
  onBaseSaved,
}: Props): ReactElement | null {
  const editor = state.editor;
  if (!editor) return null;
  const close = (): void => {
    state.setEditor(null);
  };
  if (editor.kind === "base")
    return (
      <TermbaseEditor
        access={state.access}
        baseline={editor.value}
        onSaved={onBaseSaved}
        onDeleted={state.deletedBase}
        onClose={close}
      />
    );
  return (
    <TermEditor
      access={state.access}
      termbaseId={editor.termbaseId}
      baseline={editor.value}
      onSaved={state.savedTerm}
      onClose={close}
    />
  );
}
