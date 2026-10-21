import type { ReactElement } from "react";
import { ChevronDown, ChevronUp, Plus, X } from "lucide-react";
import { IconButton } from "@/shared/component/IconButton";
import { Button } from "@/shared/component/Button";
import { terminologyInputClass } from "./TerminologyDialog";

export type TargetDraft = { id: string; value: string };
type Props = {
  targets: TargetDraft[];
  locked: boolean;
  onChange: (targets: TargetDraft[]) => void;
};

export function TermTargets({
  targets,
  locked,
  onChange,
}: Props): ReactElement {
  function move(index: number, direction: number): void {
    const next = [...targets];
    const [target] = next.splice(index, 1);
    if (!target) return;
    next.splice(index + direction, 0, target);
    onChange(next);
  }
  return (
    <fieldset className="space-y-1.5">
      <div className="flex items-center justify-between">
        <legend className="text-xs">译名</legend>
        <Button
          variant="ghost"
          disabled={locked}
          onClick={() => {
            onChange([...targets, { id: crypto.randomUUID(), value: "" }]);
          }}
        >
          <Plus size={12} />
          添加译名
        </Button>
      </div>
      {targets.map((target, index) => (
        <div key={target.id} className="flex items-center gap-1">
          <input
            aria-label={`译名 ${String(index + 1)}`}
            value={target.value}
            disabled={locked}
            onChange={(event) => {
              onChange(
                targets.map((item) =>
                  item.id === target.id
                    ? { ...item, value: event.target.value }
                    : item,
                ),
              );
            }}
            className={`${terminologyInputClass} min-w-0 flex-1`}
          />
          <IconButton
            label={`上移译名 ${String(index + 1)}`}
            disabled={locked || index === 0}
            onClick={() => {
              move(index, -1);
            }}
          >
            <ChevronUp size={13} />
          </IconButton>
          <IconButton
            label={`下移译名 ${String(index + 1)}`}
            disabled={locked || index === targets.length - 1}
            onClick={() => {
              move(index, 1);
            }}
          >
            <ChevronDown size={13} />
          </IconButton>
          <IconButton
            label={`删除译名 ${String(index + 1)}`}
            disabled={locked || targets.length === 1}
            onClick={() => {
              onChange(targets.filter((item) => item.id !== target.id));
            }}
          >
            <X size={13} />
          </IconButton>
        </div>
      ))}
    </fieldset>
  );
}
