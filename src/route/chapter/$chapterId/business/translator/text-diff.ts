import { diffArrays } from "diff";

export type TextDiffPart = {
  kind:
    | "unchanged"
    | "deleted"
    | "inserted"
    | "replacement-removed"
    | "replacement-added";
  text: string;
};
type RawPart = { kind: "unchanged" | "removed" | "added"; text: string };
const segmenter = new Intl.Segmenter("zh", { granularity: "grapheme" });

function tokenize(text: string): string[] {
  // Keep LF alignment while respecting emoji and combining-character boundaries.
  return Array.from(segmenter.segment(text), ({ segment }) => segment).flatMap(
    (segment) => (segment === "\r\n" ? ["\r", "\n"] : [segment]),
  );
}

function merge(parts: TextDiffPart[]): TextDiffPart[] {
  const merged: TextDiffPart[] = [];
  for (const part of parts) {
    const previous = merged.at(-1);
    if (previous?.kind === part.kind) {
      previous.text += part.text;
      continue;
    }
    merged.push({ ...part });
  }
  return merged;
}

function order(parts: RawPart[]): TextDiffPart[] {
  const ordered: TextDiffPart[] = [];
  let index = 0;
  while (index < parts.length) {
    const part = parts[index];
    if (!part) break;
    if (part.kind === "unchanged") {
      ordered.push({ kind: "unchanged", text: part.text });
      index += 1;
      continue;
    }
    const changed: RawPart[] = [];
    while (index < parts.length && parts[index]?.kind !== "unchanged") {
      const entry = parts[index];
      if (entry) changed.push(entry);
      index += 1;
    }
    const removedText = changed.some(
      (entry) => entry.kind === "removed" && /\S/u.test(entry.text),
    );
    const addedText = changed.some(
      (entry) => entry.kind === "added" && /\S/u.test(entry.text),
    );
    for (const kind of ["removed", "added"] as const) {
      for (const change of changed.filter((entry) => entry.kind === kind)) {
        for (const text of change.text.match(/\s+|\S+/gu) ?? []) {
          const replacement = removedText && addedText && /\S/u.test(text);
          ordered.push({
            kind:
              kind === "removed"
                ? replacement
                  ? "replacement-removed"
                  : "deleted"
                : replacement
                  ? "replacement-added"
                  : "inserted",
            text,
          });
        }
      }
    }
  }
  return merge(ordered);
}

export function buildTextDiff(
  translated: string,
  proofread: string,
): TextDiffPart[] {
  if (!proofread.trim())
    return translated ? [{ kind: "unchanged", text: translated }] : [];
  if (translated === proofread)
    return [{ kind: "unchanged", text: translated }];
  // Long or highly divergent input retains exact text without quadratic work.
  const fallback: RawPart[] = [
    { kind: "removed", text: translated },
    { kind: "added", text: proofread },
  ];
  if (translated.length + proofread.length > 16_384) return order(fallback);
  const changes = diffArrays(tokenize(translated), tokenize(proofread), {
    timeout: 20,
    maxEditLength: 512,
  });
  if (!changes) return order(fallback);
  return order(
    changes.map((change) => ({
      kind: change.removed ? "removed" : change.added ? "added" : "unchanged",
      text: change.value.join(""),
    })),
  );
}
