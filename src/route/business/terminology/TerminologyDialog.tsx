import { useEffect, useRef } from "react";
import type { ReactElement, ReactNode } from "react";
import { useBlocker } from "@tanstack/react-router";
import { registerExitGuard } from "@/bridge/exit";
import { useLibraryWrite } from "@/route/business/use-library-write";
import { Dialog } from "@/shared/component/Dialog";
import { Button } from "@/shared/component/Button";
import { ButtonGroup } from "@/shared/component/ButtonGroup";

export function useTerminologyGuard(locked: boolean): void {
  useBlocker({ shouldBlockFn: () => locked, enableBeforeUnload: false });
  useEffect(() => registerExitGuard(() => Promise.resolve(!locked)), [locked]);
}

export function useTerminologyWrite(
  onClose: () => void,
  selecting = false,
): ReturnType<typeof useLibraryWrite> {
  const write = useLibraryWrite(onClose);
  const locked = write.locked || selecting;
  useTerminologyGuard(locked);
  return { ...write, locked, busy: write.busy || selecting };
}

type Props = {
  title: string;
  write: ReturnType<typeof useLibraryWrite>;
  confirming: boolean;
  canDelete: boolean;
  validation: string;
  onConfirm: (value: boolean) => void;
  onSave: () => void;
  onDelete: () => void;
  onClose: () => void;
  onImport?: (() => void) | undefined;
  children: ReactNode;
};

export function TerminologyDialog(props: Props): ReactElement {
  const opener = useRef(document.activeElement);
  useEffect(
    () => () => {
      const target = opener.current;
      queueMicrotask(() => {
        if (target instanceof HTMLElement && target.isConnected) target.focus();
      });
    },
    [],
  );
  const write = props.write;
  return (
    <Dialog
      closeDisabled={write.locked}
      open={true}
      onOpenChange={(open) => {
        if (!open && !write.locked) props.onClose();
      }}
      title={props.title}
      description=""
      className="max-w-sm gap-3 p-4"
    >
      <form
        className="space-y-3"
        onSubmit={(event) => {
          event.preventDefault();
          if (!write.locked) {
            if (props.confirming) props.onDelete();
            else props.onSave();
          }
        }}
      >
        {props.children}
        {(write.error || props.validation) && (
          <p role="alert" className="text-xs text-destructive">
            {props.validation || write.error}
          </p>
        )}
        <ButtonGroup>
          {props.onImport && !props.confirming && (
            <Button
              variant="secondary"
              disabled={write.locked}
              onClick={props.onImport}
            >
              导入
            </Button>
          )}
          {props.canDelete && !props.confirming && (
            <Button
              variant="danger"
              disabled={write.locked}
              onClick={() => {
                props.onConfirm(true);
              }}
            >
              删除
            </Button>
          )}
          {props.confirming && (
            <Button
              variant="ghost"
              disabled={write.locked}
              onClick={() => {
                props.onConfirm(false);
              }}
            >
              返回编辑
            </Button>
          )}
          {write.uncertain ? (
            <Button
              variant="primary"
              disabled={write.busy}
              onClick={write.retry}
            >
              核实写入结果
            </Button>
          ) : (
            <Button
              type="submit"
              variant={props.confirming ? "danger" : "primary"}
              disabled={write.busy}
            >
              {write.busy
                ? "正在保存…"
                : props.confirming
                  ? "确认删除"
                  : "保存"}
            </Button>
          )}
        </ButtonGroup>
      </form>
    </Dialog>
  );
}

export const terminologyInputClass =
  "w-full rounded-md border border-border bg-card px-2.5 py-1.5 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50";
