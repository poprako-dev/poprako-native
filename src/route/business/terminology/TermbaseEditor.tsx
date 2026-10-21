import { useRef, useState } from "react";
import type { ReactElement } from "react";
import { commands, NativeCommandError, unwrap } from "@/bridge";
import type { Termbase, TerminologyContext } from "@/bridge/generated/bindings";
import {
  TerminologyDialog,
  terminologyInputClass,
  useTerminologyWrite,
} from "./TerminologyDialog";

type Props = {
  access: TerminologyContext;
  baseline: Termbase | null;
  onSaved: (value: Termbase) => void;
  onDeleted: (id: string) => void;
  onClose: () => void;
};

export function TermbaseEditor({
  access,
  baseline,
  onSaved,
  onDeleted,
  onClose,
}: Props): ReactElement {
  const [name, setName] = useState(baseline?.name ?? "");
  const [description, setDescription] = useState(baseline?.description ?? "");
  const [confirming, setConfirming] = useState(false);
  const [validation, setValidation] = useState("");
  const [selecting, setSelecting] = useState(false);
  const selectionBusy = useRef(false);
  const write = useTerminologyWrite(onClose, selecting);
  async function importFile(): Promise<void> {
    if (selectionBusy.current || write.locked) return;
    selectionBusy.current = true;
    setSelecting(true);
    setValidation("");
    try {
      const document = unwrap(await commands.chooseTermbaseImport());
      if (document !== null) {
        write.execute(async () => {
          onSaved(unwrap(await commands.importTermbase(access, document)));
        });
      }
    } catch (error) {
      setValidation(
        error instanceof NativeCommandError
          ? error.message
          : "无法读取术语库文件，请重试。",
      );
    } finally {
      selectionBusy.current = false;
      setSelecting(false);
    }
  }
  function save(): void {
    if (name.trim() === "") {
      setValidation("术语库名称不能为空。");
      return;
    }
    setValidation("");
    write.execute(async () => {
      const metadata = { name, description };
      const result = baseline
        ? await commands.updateTermbase(access, baseline, metadata)
        : await commands.createTermbase(access, metadata);
      onSaved(unwrap(result));
    });
  }
  function remove(): void {
    if (!baseline) return;
    write.execute(async () => {
      unwrap(await commands.deleteTermbase(access, baseline));
      onDeleted(baseline.id);
    });
  }
  return (
    <TerminologyDialog
      title={confirming ? "删除术语库" : baseline ? "编辑术语库" : "新建术语库"}
      write={write}
      confirming={confirming}
      canDelete={baseline !== null}
      validation={validation}
      onConfirm={setConfirming}
      onSave={save}
      onDelete={remove}
      onClose={onClose}
      onImport={
        baseline === null
          ? () => {
              void importFile();
            }
          : undefined
      }
    >
      {confirming ? (
        <p className="text-sm">删除“{name}”及其全部术语？此操作无法撤销。</p>
      ) : (
        <>
          <label className="block space-y-1 text-xs">
            名称
            <input
              autoFocus
              value={name}
              disabled={write.locked}
              onChange={(event) => {
                setName(event.target.value);
              }}
              className={terminologyInputClass}
            />
          </label>
          <label className="block space-y-1 text-xs">
            描述
            <textarea
              rows={3}
              value={description}
              disabled={write.locked}
              onChange={(event) => {
                setDescription(event.target.value);
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
