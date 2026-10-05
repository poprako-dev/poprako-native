import { Grip, Undo2 } from "lucide-react";
import type { ReactElement } from "react";
import type { SpecialCharacter } from "@/bridge/generated/bindings";
import type { SpecialCharsBarController } from "./use-detachable-special-chars-bar";

export type SymbolRequest = { id: string; unitId: string; text: string };
type SpecialCharsBarProps = {
  characters: SpecialCharacter[];
  controller: SpecialCharsBarController | null;
  floating: boolean;
  disabled: boolean;
  onInsert: (text: string) => void;
};

export function SpecialCharsBar({
  characters,
  controller,
  floating,
  disabled,
  onInsert,
}: SpecialCharsBarProps): ReactElement | null {
  if (!floating && controller?.position)
    return controller.placeholderHeight === null ? null : (
      <div
        aria-hidden="true"
        style={{ height: controller.placeholderHeight }}
      />
    );
  const favorites = characters.filter((character) => character.is_favorite);
  return (
    <div
      data-special-chars-bar
      role="group"
      aria-label="优选符号"
      className={`flex min-w-0 items-start gap-1 p-1 ${floating ? "rounded bg-background shadow-sm outline outline-1 outline-border" : ""}`}
    >
      <div
        className="flex min-w-0 flex-1 flex-wrap gap-1 overflow-y-auto"
        style={
          floating && controller?.position
            ? { maxHeight: Math.max(0, controller.position.maxHeight - 10) }
            : {}
        }
      >
        {favorites.map((character) => (
          <button
            type="button"
            key={character.id}
            disabled={disabled || controller?.isEnabled === false}
            onPointerDown={(event) => {
              event.preventDefault();
              event.stopPropagation();
            }}
            onMouseDown={(event) => {
              event.preventDefault();
            }}
            onClick={(event) => {
              event.stopPropagation();
              onInsert(character.text);
            }}
            className="min-w-6 max-w-full rounded border border-border bg-muted px-1 py-0.5 text-center font-mono text-xs break-all text-muted-foreground transition-colors hover:border-primary/30 hover:bg-accent hover:text-primary disabled:pointer-events-none disabled:opacity-40"
          >
            {character.text}
          </button>
        ))}
        {favorites.length === 0 && (
          <span className="py-1 text-xs text-muted-foreground">
            尚无优选符号
          </span>
        )}
      </div>
      {floating && controller && (
        <button
          type="button"
          title="放回符号栏"
          aria-label="放回符号栏"
          onPointerDown={(event) => {
            event.preventDefault();
          }}
          onClick={controller.dock}
          className="flex size-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-primary"
        >
          <Undo2 size={16} aria-hidden="true" />
        </button>
      )}
      {controller && (
        <button
          type="button"
          title={floating ? "拖动符号栏" : "拖出符号栏"}
          aria-label={floating ? "拖动符号栏" : "拖出符号栏"}
          disabled={!controller.isEnabled}
          onPointerDown={controller.handleGripPointerDown}
          onContextMenu={(event) => {
            event.preventDefault();
          }}
          className={`flex size-6 shrink-0 touch-none items-center justify-center rounded text-muted-foreground select-none hover:bg-muted ${controller.isGripHeld ? "cursor-grabbing bg-accent text-primary" : "cursor-grab"}`}
        >
          <Grip size={16} aria-hidden="true" />
        </button>
      )}
    </div>
  );
}
