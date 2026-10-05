import { expect, it, vi } from "vitest";
import type { WorkPosition } from "@/bridge/generated/bindings";
import { createPositionWriter } from "./work-position";
import type { WorkPositionWriter } from "./work-position";

it("recovers the same failed position before writing a new timestamp and selection", async () => {
  const original: WorkPosition = {
    comic_id: "comic",
    page_id: "page",
    unit_id: "one",
    mode: "translation",
    last_opened_at: 1,
  };
  const newer = { ...original, unit_id: "two", last_opened_at: 2 };
  const save = vi
    .fn<WorkPositionWriter["save"]>()
    .mockRejectedValueOnce(new Error("unknown result"))
    .mockResolvedValue(undefined);
  const writer = createPositionWriter({ save });
  await expect(writer.save(original)).rejects.toThrow("unknown result");
  await writer.save(newer);
  expect(save.mock.calls).toEqual([[original], [original], [newer]]);
});
