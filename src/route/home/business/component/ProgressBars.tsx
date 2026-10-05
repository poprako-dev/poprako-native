import type { ReactElement } from "react";

type ProgressBarsProps = {
  total: number;
  translated: number;
  confirmed: number;
};

export function ProgressBars({
  total,
  translated,
  confirmed,
}: ProgressBarsProps): ReactElement {
  const translatedPercent = total === 0 ? 0 : (translated / total) * 100;
  const confirmedPercent = total === 0 ? 0 : (confirmed / total) * 100;
  const label = `已翻译 ${String(Math.round(translatedPercent))}%，已确认校对 ${String(Math.round(confirmedPercent))}%`;

  return (
    <span
      className="grid size-3.5 shrink-0 grid-cols-4 gap-0.5"
      role="img"
      aria-label={label}
      title={label}
    >
      {[25, 50, 75, 100].map((threshold) => (
        <span
          key={threshold}
          aria-hidden="true"
          className={`rounded-sm ${
            confirmedPercent >= threshold
              ? "bg-(--stats-edit)"
              : translatedPercent >= threshold
                ? "bg-(--stats-translate)"
                : "bg-border"
          }`}
        />
      ))}
    </span>
  );
}
