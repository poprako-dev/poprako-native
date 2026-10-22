import { useEffect, useMemo, useRef, useState } from "react";
import { commands, NativeCommandError, unwrap } from "@/bridge";
import type {
  TermbaseAppend,
  TermbaseAppendResult,
  TerminologyContext,
} from "@/bridge/generated/bindings";
import { useTerminologyGuard } from "./TerminologyDialog";

type Scope = { key: string };
type Job = { id: string; document: TermbaseAppend };
type AppendState = {
  scope: Scope;
  id: string | null;
  busy: boolean;
  uncertain: boolean;
  job: Job | null;
  notice: string;
  error: string;
};

export type TermbaseAppendController = {
  pendingId: string | null;
  locked: boolean;
  busy: boolean;
  uncertain: boolean;
  canRetry: boolean;
  notice: string;
  error: string;
  appendTermbase: (id: string) => Promise<void>;
  retry: () => Promise<void>;
};

function initial(scope: Scope): AppendState {
  return {
    scope,
    id: null,
    busy: false,
    uncertain: false,
    job: null,
    notice: "",
    error: "",
  };
}

export function useTermbaseAppend(
  access: TerminologyContext,
  onAppended: () => void,
): TermbaseAppendController {
  const key = access.kind === "global" ? "global" : access.chapter_id;
  const scope = useMemo<Scope>(() => ({ key }), [key]);
  const active = useRef<Scope | null>(null);
  const running = useRef<{ scope: Scope; uncertain: boolean } | null>(null);
  const [snapshot, setSnapshot] = useState<AppendState>(() => initial(scope));
  const state = snapshot.scope === scope ? snapshot : initial(scope);
  const locked = state.busy || state.uncertain;
  useTerminologyGuard(locked);
  useEffect(() => {
    active.current = scope;
    return () => {
      active.current = null;
    };
  }, [scope]);

  async function perform(job: Job): Promise<void> {
    running.current = { scope, uncertain: false };
    setSnapshot({
      ...initial(scope),
      id: job.id,
      busy: true,
      uncertain: state.uncertain,
      job,
    });
    let result: TermbaseAppendResult;
    try {
      result = unwrap(
        await commands.appendTermbase(access, job.id, job.document),
      );
    } catch (cause) {
      if (active.current !== scope) return;
      const uncertain =
        !(cause instanceof NativeCommandError) ||
        cause.detail.recovery === "wait_for_confirmation" ||
        cause.detail.code === "commit_unknown" ||
        cause.detail.code === "recovery_required";
      const retryable =
        cause instanceof NativeCommandError &&
        cause.detail.recovery === "retry";
      running.current = uncertain ? { scope, uncertain: true } : null;
      setSnapshot({
        ...initial(scope),
        id: job.id,
        job: uncertain || retryable ? job : null,
        uncertain,
        error:
          cause instanceof NativeCommandError
            ? cause.message
            : "追加结果尚未确认，请保留窗口并核实。",
      });
      return;
    }
    if (running.current.scope === scope) running.current = null;
    if (active.current !== scope) return;
    setSnapshot({
      ...initial(scope),
      notice: `已追加：新增 ${String(result.added)} 条，更新 ${String(result.updated)} 条，未变更 ${String(result.unchanged)} 条`,
    });
    onAppended();
  }

  async function appendTermbase(id: string): Promise<void> {
    if (active.current !== scope || running.current?.scope === scope) return;
    running.current = { scope, uncertain: false };
    setSnapshot({ ...initial(scope), id, busy: true });
    let document: TermbaseAppend | null;
    try {
      document = unwrap(await commands.chooseTermbaseAppend());
    } catch (cause) {
      if (running.current.scope === scope) running.current = null;
      if (active.current === scope) {
        setSnapshot({
          ...initial(scope),
          error:
            cause instanceof NativeCommandError
              ? cause.message
              : "无法读取术语文件，请重新选择。",
        });
      }
      return;
    }
    if (active.current !== scope) return;
    if (document === null) {
      running.current = null;
      setSnapshot(initial(scope));
      return;
    }
    await perform({ id, document });
  }

  async function retry(): Promise<void> {
    if (state.busy || state.job === null || active.current !== scope) return;
    if (running.current?.scope === scope && !running.current.uncertain) return;
    await perform(state.job);
  }

  return {
    pendingId: locked ? state.id : null,
    locked,
    busy: state.busy,
    uncertain: state.uncertain,
    canRetry: state.job !== null,
    notice: state.notice,
    error: state.error,
    appendTermbase,
    retry,
  };
}
