import { useState } from "react";
import { X } from "lucide-react";
import type { KeyboardEvent, ReactElement } from "react";
import type { Shortcut, ShortcutAction } from "@/bridge/generated/bindings";
import {
  formatShortcut,
  fixedShortcuts,
  hasShortcutConflict,
  shortcutLabel,
} from "@/route/business/shortcut";
import { Button } from "@/shared/component/Button";
import { isKeyboardComposing } from "@/shared/utility/keyboard";

type ShortcutSettingsProps = {
  value: Shortcut[];
  disabled: boolean;
  onChange: (value: Shortcut[]) => void;
};

export function ShortcutSettings({
  value,
  disabled,
  onChange,
}: ShortcutSettingsProps): ReactElement {
  const [capturing, setCapturing] = useState<ShortcutAction | null>(null);
  const [conflict, setConflict] = useState(false);

  function capture(
    event: KeyboardEvent<HTMLButtonElement>,
    shortcut: Shortcut,
  ): void {
    if (
      capturing !== shortcut.action ||
      isKeyboardComposing(event.nativeEvent) ||
      event.getModifierState("AltGraph")
    ) {
      return;
    }
    if (["Control", "Meta", "Alt", "Shift"].includes(event.key)) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    const updated: Shortcut[] = value.map((item) =>
      item.action === shortcut.action
        ? {
            ...item,
            binding: {
              kind: "chord",
              key: event.code,
              control: event.ctrlKey,
              meta: event.metaKey,
              alt: event.altKey,
              shift: event.shiftKey,
              scope: "editor",
            },
          }
        : item,
    );
    setCapturing(null);
    if (hasShortcutConflict(updated)) {
      setConflict(true);
      return;
    }
    setConflict(false);
    onChange(updated);
  }

  return (
    <div className="space-y-2">
      <div className="grid gap-3 border-b border-dashed border-border pb-3 sm:grid-cols-2">
        {fixedShortcuts.map(([label, gesture]) => (
          <div key={label} className="grid grid-cols-2 gap-2 text-xs">
            <span className="text-muted-foreground">{label}</span>
            <span>{gesture}</span>
          </div>
        ))}
      </div>
      {value.map((shortcut) => (
        <div
          className="flex items-center justify-between gap-2"
          key={shortcut.action}
        >
          <span className="text-xs">{shortcutLabel[shortcut.action]}</span>
          <div className="flex gap-1">
            <Button
              variant="secondary"
              disabled={disabled}
              aria-label={`设置${shortcutLabel[shortcut.action]}快捷键`}
              onClick={() => {
                setConflict(false);
                setCapturing(shortcut.action);
              }}
              onBlur={() => {
                setCapturing(null);
              }}
              onKeyDown={(event) => {
                capture(event, shortcut);
              }}
            >
              {capturing === shortcut.action
                ? "请按组合键…"
                : formatShortcut(shortcut.binding)}
            </Button>
            <Button
              variant="ghost"
              disabled={disabled || shortcut.binding.kind === "unbound"}
              aria-label={`清除${shortcutLabel[shortcut.action]}快捷键`}
              onClick={() => {
                onChange(
                  value.map((item) =>
                    item.action === shortcut.action
                      ? { ...item, binding: { kind: "unbound" } }
                      : item,
                  ),
                );
              }}
            >
              <X size={14} aria-hidden="true" />
            </Button>
          </div>
        </div>
      ))}
      {(conflict || hasShortcutConflict(value)) && (
        <p role="alert" className="text-xs text-destructive">
          快捷键冲突，已保留原有设置。
        </p>
      )}
    </div>
  );
}
