import { ArrowDown, ArrowUp, Star, Trash2 } from "lucide-react";
import { useState } from "react";
import type { ReactElement } from "react";
import type { SpecialCharacter } from "@/bridge/generated/bindings";
import { Button } from "@/shared/component/Button";

type CharacterSettingsProps = {
  value: SpecialCharacter[];
  disabled: boolean;
  onChange: (value: SpecialCharacter[]) => void;
};

export function CharacterSettings({
  value,
  disabled,
  onChange,
}: CharacterSettingsProps): ReactElement {
  const [text, setText] = useState("");

  function move(index: number, direction: number): void {
    const next = [...value];
    const source = next[index];
    const destination = next[index + direction];
    if (source === undefined || destination === undefined) {
      return;
    }
    next[index] = destination;
    next[index + direction] = source;
    onChange(next);
  }

  return (
    <div className="space-y-2">
      {value.map((character, index) => (
        <div className="flex items-center gap-1" key={character.id}>
          <input
            aria-label={`字符 ${String(index + 1)}`}
            value={character.text}
            disabled={disabled}
            onChange={(event) => {
              const textValue = event.currentTarget.value;
              onChange(
                value.map((item) =>
                  item.id === character.id
                    ? { ...item, text: textValue }
                    : item,
                ),
              );
            }}
            className="h-8 min-w-0 flex-1 rounded border border-border bg-background px-2 text-sm"
          />
          <Button
            variant="ghost"
            disabled={disabled}
            aria-label={character.is_favorite ? "取消收藏" : "收藏字符"}
            aria-pressed={character.is_favorite}
            onClick={() => {
              onChange(
                value.map((item) =>
                  item.id === character.id
                    ? { ...item, is_favorite: !item.is_favorite }
                    : item,
                ),
              );
            }}
          >
            <Star
              className={`size-3.5 ${character.is_favorite ? "fill-current" : ""}`}
              aria-hidden="true"
            />
          </Button>
          <Button
            variant="ghost"
            disabled={disabled || index === 0}
            aria-label="上移字符"
            onClick={() => {
              move(index, -1);
            }}
          >
            <ArrowUp className="size-3.5" aria-hidden="true" />
          </Button>
          <Button
            variant="ghost"
            disabled={disabled || index === value.length - 1}
            aria-label="下移字符"
            onClick={() => {
              move(index, 1);
            }}
          >
            <ArrowDown className="size-3.5" aria-hidden="true" />
          </Button>
          <Button
            variant="ghost"
            disabled={disabled}
            aria-label="删除字符"
            onClick={() => {
              onChange(value.filter((item) => item.id !== character.id));
            }}
          >
            <Trash2 className="size-3.5" aria-hidden="true" />
          </Button>
        </div>
      ))}
      <div className="flex gap-2">
        <input
          aria-label="新字符"
          value={text}
          onChange={(event) => {
            setText(event.currentTarget.value);
          }}
          disabled={disabled}
          className="h-8 min-w-0 flex-1 rounded border border-border bg-background px-2 text-sm"
          placeholder="输入要添加的字符"
        />
        <Button
          variant="secondary"
          disabled={disabled || text.trim() === ""}
          onClick={() => {
            onChange([
              ...value,
              { id: crypto.randomUUID(), text, is_favorite: false },
            ]);
            setText("");
          }}
        >
          添加
        </Button>
      </div>
    </div>
  );
}
