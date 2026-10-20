import type { ReactElement } from "react";
import type { Chapter } from "@/bridge/generated/bindings";
import { LibraryMetadataDialog } from "@/route/business/component/LibraryMetadataDialog";

type Props = { chapter: Chapter; onClose: () => void; onComplete: () => void };
export function ChapterMetadataDialog({
  chapter,
  onClose,
  onComplete,
}: Props): ReactElement {
  return (
    <LibraryMetadataDialog
      target={{ kind: "chapter", chapter }}
      onClose={onClose}
      onComplete={onComplete}
    />
  );
}
