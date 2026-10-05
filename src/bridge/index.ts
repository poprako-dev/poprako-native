import { commands as native } from "./generated/bindings";
import type { CommandError } from "./generated/bindings";

import { write } from "./write";

export const commands = {
  ...native,
  confirmArchiveImport(
    taskId: Parameters<typeof native.confirmArchiveImport>[0],
    metadata: Parameters<typeof native.confirmArchiveImport>[1],
    mode: Parameters<typeof native.confirmArchiveImport>[2],
  ): ReturnType<typeof native.confirmArchiveImport> {
    return write("confirmArchiveImport", [taskId, metadata, mode], (request) =>
      native.confirmArchiveImport(taskId, metadata, mode, request),
    );
  },
  confirmImageImport(
    taskId: Parameters<typeof native.confirmImageImport>[0],
    clearUnits: Parameters<typeof native.confirmImageImport>[1],
  ): ReturnType<typeof native.confirmImageImport> {
    return write("confirmImageImport", [taskId, clearUnits], (request) =>
      native.confirmImageImport(taskId, clearUnits, request),
    );
  },
  removePages(
    input: Parameters<typeof native.removePages>[0],
  ): ReturnType<typeof native.removePages> {
    return write("removePages", [input], (request) =>
      native.removePages(input, request),
    );
  },
  reorderPages(
    input: Parameters<typeof native.reorderPages>[0],
  ): ReturnType<typeof native.reorderPages> {
    return write("reorderPages", [input], (request) =>
      native.reorderPages(input, request),
    );
  },
  createComic(
    metadata: Parameters<typeof native.createComic>[0],
  ): ReturnType<typeof native.createComic> {
    return write("createComic", [metadata], (request) =>
      native.createComic(metadata, request),
    );
  },
  updateComicMetadata(
    comicId: Parameters<typeof native.updateComicMetadata>[0],
    metadata: Parameters<typeof native.updateComicMetadata>[1],
  ): ReturnType<typeof native.updateComicMetadata> {
    return write("updateComicMetadata", [comicId, metadata], (request) =>
      native.updateComicMetadata(comicId, metadata, request),
    );
  },
  deleteComic(
    comicId: Parameters<typeof native.deleteComic>[0],
  ): ReturnType<typeof native.deleteComic> {
    return write("deleteComic", [comicId], (request) =>
      native.deleteComic(comicId, request),
    );
  },
  savePageUnits(
    snapshot: Parameters<typeof native.savePageUnits>[0],
  ): ReturnType<typeof native.savePageUnits> {
    return write("savePageUnits", [snapshot], (request) =>
      native.savePageUnits(snapshot, request),
    );
  },
  updatePreference(
    baseline: Parameters<typeof native.updatePreference>[0],
    value: Parameters<typeof native.updatePreference>[1],
  ): ReturnType<typeof native.updatePreference> {
    return write("updatePreference", [baseline, value], (request) =>
      native.updatePreference(baseline, value, request),
    );
  },
  updateWorkPosition(
    position: Parameters<typeof native.updateWorkPosition>[0],
  ): ReturnType<typeof native.updateWorkPosition> {
    return write("updateWorkPosition", [position], (request) =>
      native.updateWorkPosition(position, request),
    );
  },
  replaceUnits(
    input: Parameters<typeof native.replaceUnits>[0],
  ): ReturnType<typeof native.replaceUnits> {
    return write("replaceUnits", [input], (request) =>
      native.replaceUnits(input, request),
    );
  },
};

type CommandFailure = Extract<
  Awaited<ReturnType<typeof commands.getPreference>>,
  { status: "error" }
>;

export class NativeCommandError extends Error {
  readonly detail: CommandError;

  constructor(detail: CommandError) {
    super(detail.message);
    this.name = "NativeCommandError";
    this.detail = detail;
  }
}

export function unwrap<T>(
  result: { status: "ok"; data: T } | CommandFailure,
): T {
  if (result.status === "ok") return result.data;
  throw new NativeCommandError(result.error);
}
