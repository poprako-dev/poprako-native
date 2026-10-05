import { commands as native } from "./generated/bindings";
import type { CommandError, WriteRequest } from "./generated/bindings";

type Failure = { status: "error"; error: CommandError };
type Outcome<T> = { status: "ok"; data: T } | Failure;
interface WriteOperation<T> {
  (request: WriteRequest): Promise<Outcome<T>>;
}
type PendingWrite = { key: string; request: WriteRequest; completed: boolean };
let pending: PendingWrite | null = null;
let queue: Promise<void> = Promise.resolve();

function uncertain(): Failure {
  return {
    status: "error",
    error: {
      code: "commit_unknown",
      message: "写入结果尚未确认。请保留当前修改并重试核实，不要关闭应用。",
      recovery: "wait_for_confirmation",
    },
  };
}

async function perform<T>(
  key: string,
  operation: WriteOperation<T>,
): Promise<Outcome<T>> {
  if (pending?.completed === true) {
    const acknowledged = await native
      .acknowledgeWriteResult(pending.request)
      .catch(() => null);
    if (acknowledged === null || acknowledged.status === "error")
      return uncertain();
    pending = null;
  }
  if (pending !== null && pending.key !== key) return uncertain();
  if (pending === null) {
    const session = await native.getWriteSession();
    if (session.status === "error") return session;
    pending = {
      key,
      completed: false,
      request: {
        session_id: session.data.session_id,
        sequence: session.data.next_sequence,
      },
    };
  }
  const active = pending;
  const request = active.request;
  // Every retry uses the same atomically deduplicated request and identical payload.
  for (let attempt = 0; attempt < 2; attempt += 1) {
    try {
      const outcome = await operation(request);
      if (
        outcome.status === "error" &&
        (outcome.error.code === "commit_unknown" ||
          outcome.error.code === "recovery_required")
      )
        return outcome;
      active.completed = true;
      const acknowledged = await native
        .acknowledgeWriteResult(request)
        .catch(() => null);
      if (acknowledged?.status === "ok") pending = null;
      return outcome;
    } catch {
      const status = await native.getWriteResult(request).catch(() => null);
      if (status?.status === "ok" && status.data.status === "uncertain")
        return uncertain();
    }
  }
  return uncertain();
}

export async function write<T>(
  name: string,
  args: unknown[],
  operation: WriteOperation<T>,
): Promise<Outcome<T>> {
  const previous = queue;
  let release = (): void => undefined;
  queue = new Promise<void>((resolve) => {
    release = resolve;
  });
  await previous;
  try {
    return await perform(JSON.stringify([name, ...args]), operation);
  } finally {
    release();
  }
}
