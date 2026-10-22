import { useEffect, useMemo, useRef, useState } from "react";
import { commands, NativeCommandError, unwrap } from "@/bridge";
import type { TerminologyContext } from "@/bridge/generated/bindings";

type ExportScope = {
  key: string;
};

type ExportState = {
  scope: ExportScope;
  pending: ReadonlySet<string>;
  notice: string;
  failed: boolean;
};

export type TermbaseExportController = {
  pending: ReadonlySet<string>;
  notice: string;
  failed: boolean;
  exportTermbase: (id: string) => Promise<void>;
};

const empty: ReadonlySet<string> = new Set();

function pendingIds(
  tasks: ReadonlyMap<string, ExportScope>,
  scope: ExportScope,
): ReadonlySet<string> {
  return new Set(
    Array.from(tasks)
      .filter(([, owner]) => owner === scope)
      .map(([id]) => id),
  );
}

export function useTermbaseExport(
  access: TerminologyContext,
): TermbaseExportController {
  const key = access.kind === "global" ? "global" : access.chapter_id;
  const scope = useMemo<ExportScope>(() => ({ key }), [key]);
  const active = useRef<ExportScope | null>(null);
  const tasks = useRef(new Map<string, ExportScope>());
  const [state, setState] = useState<ExportState>({
    scope,
    pending: empty,
    notice: "",
    failed: false,
  });
  useEffect(() => {
    active.current = scope;
    return () => {
      active.current = null;
    };
  }, [scope]);

  async function exportTermbase(id: string): Promise<void> {
    if (active.current !== scope || tasks.current.get(id) === scope) return;
    tasks.current.set(id, scope);
    setState({
      scope,
      pending: pendingIds(tasks.current, scope),
      notice: "",
      failed: false,
    });
    let notice = "";
    let failed = false;
    try {
      const result = unwrap(await commands.exportTermbase(access, id));
      if (result === "saved") notice = "已导出术语库";
    } catch (error) {
      failed = true;
      notice =
        error instanceof NativeCommandError
          ? `导出失败：${error.message}；请重试或更换保存位置。`
          : "导出失败，请重试或更换保存位置。";
    } finally {
      if (tasks.current.get(id) === scope) tasks.current.delete(id);
      if (active.current === scope) {
        setState({
          scope,
          pending: pendingIds(tasks.current, scope),
          notice,
          failed,
        });
      }
    }
  }

  return {
    pending: state.scope === scope ? state.pending : empty,
    notice: state.scope === scope ? state.notice : "",
    failed: state.scope === scope && state.failed,
    exportTermbase,
  };
}
