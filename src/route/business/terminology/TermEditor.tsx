import { useState } from "react";
import type { ReactElement } from "react";
import { commands, unwrap } from "@/bridge";
import type { Term, TerminologyContext } from "@/bridge/generated/bindings";
import {
  TerminologyDialog,
  terminologyInputClass,
  useTerminologyWrite,
} from "./TerminologyDialog";
import { TermTargets } from "./TermTargets";
import type { TargetDraft } from "./TermTargets";

type Props = {
  access: TerminologyContext;
  termbaseId: string;
  baseline: Term | null;
  onSaved: (created: boolean) => void;
  onClose: () => void;
};

export function TermEditor({
  access,
  termbaseId,
  baseline,
  onSaved,
  onClose,
}: Props): ReactElement {
  const [source, setSource] = useState(baseline?.source ?? "");
  const [targets, setTargets] = useState<TargetDraft[]>(() =>
    (baseline?.targets ?? [""]).map((value) => ({
      id: crypto.randomUUID(),
      value,
    })),
  );
  const [comment, setComment] = useState(baseline?.comment ?? "");
  const [confirming, setConfirming] = useState(false);
  const [validation, setValidation] = useState("");
  const write = useTerminologyWrite(onClose);
  function save(): void {
    const values = targets.map((target) => target.value);
    if (source.trim() === "" || values.some((value) => value.trim() === "")) {
      setValidation("原文和每个译名都不能为空。");
      return;
    }
    if (
      new Set(values.map((value) => value.trim().toLowerCase())).size !==
      values.length
    ) {
      setValidation("译名不能重复。");
      return;
    }
    setValidation("");
    write.execute(async () => {
      const metadata = { source, targets: values, comment };
      unwrap(
        baseline
          ? await commands.updateTerm(access, baseline, metadata)
          : await commands.createTerm(access, termbaseId, metadata),
      );
      onSaved(baseline === null);
    });
  }
  function remove(): void {
    if (!baseline) return;
    write.execute(async () => {
      unwrap(await commands.deleteTerm(access, baseline));
      onSaved(false);
    });
  }
  return (
    <TerminologyDialog
      title={confirming ? "删除术语" : baseline ? "编辑术语" : "新建术语"}
      write={write}
      confirming={confirming}
      canDelete={baseline !== null}
      validation={validation}
      onConfirm={setConfirming}
      onSave={save}
      onDelete={remove}
      onClose={onClose}
    >
      {confirming ? (
        <p className="text-sm">删除“{source}”及其全部译名？此操作无法撤销。</p>
      ) : (
        <>
          <label className="block space-y-1 text-xs">
            原文
            <input
              autoFocus
              value={source}
              disabled={write.locked}
              onChange={(event) => {
                setSource(event.target.value);
              }}
              className={terminologyInputClass}
            />
          </label>
          <TermTargets
            targets={targets}
            locked={write.locked}
            onChange={setTargets}
          />
          <label className="block space-y-1 text-xs">
            备注
            <textarea
              rows={2}
              value={comment}
              disabled={write.locked}
              onChange={(event) => {
                setComment(event.target.value);
              }}
              placeholder="选填"
              className={terminologyInputClass}
            />
          </label>
        </>
      )}
    </TerminologyDialog>
  );
}
