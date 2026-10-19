import type { WorkPosition } from "@/bridge/generated/bindings";

export interface WorkPositionWriter {
  save(position: WorkPosition): Promise<void>;
}

export function createPositionWriter(
  api: WorkPositionWriter,
): WorkPositionWriter {
  let queue = Promise.resolve();
  let pending: WorkPosition | null = null;
  return {
    save(position): Promise<void> {
      const operation = queue.then(async () => {
        // Recover the exact prior payload before a new timestamp or selection.
        if (pending) {
          await api.save(pending);
          pending = null;
        }
        pending = position;
        await api.save(position);
        pending = null;
      });
      // Only the sequencing tail consumes rejection; callers retain the error.
      queue = operation.then(
        () => undefined,
        () => undefined,
      );
      return operation;
    },
  };
}
