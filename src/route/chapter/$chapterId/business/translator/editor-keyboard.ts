import type { ShortcutBinding } from "@/bridge/generated/bindings";
import { isKeyboardComposing } from "@/shared/utility/keyboard";

export function shouldIgnoreEditorKey(event: KeyboardEvent): boolean {
  if (
    event.defaultPrevented ||
    isKeyboardComposing(event) ||
    event.getModifierState("AltGraph")
  )
    return true;
  const target =
    event.composedPath().find((node) => node instanceof Element) ??
    event.target;
  if (!(target instanceof Element)) return false;
  if (
    target.closest(
      '[data-app-dialog],[data-window-control],[role="dialog"],[role="alertdialog"]',
    )
  )
    return true;
  const editable =
    target.closest("input,textarea,select") !== null ||
    (target instanceof HTMLElement && target.isContentEditable);
  return editable && !target.closest("[data-unit-id]");
}

export function matchesEditorShortcut(
  event: KeyboardEvent,
  binding: ShortcutBinding,
): boolean {
  if (binding.kind === "unbound") return false;
  if (
    binding.control !== event.ctrlKey ||
    binding.meta !== event.metaKey ||
    binding.alt !== event.altKey ||
    binding.shift !== event.shiftKey
  )
    return false;
  if (/^Digit\d$/u.test(binding.key)) return event.code === binding.key;
  if (/^Key[A-Z]$/u.test(binding.key))
    return event.key.toLowerCase() === binding.key.slice(3).toLowerCase();
  return (
    event.key.toLowerCase() === binding.key.toLowerCase() ||
    event.code === binding.key
  );
}
