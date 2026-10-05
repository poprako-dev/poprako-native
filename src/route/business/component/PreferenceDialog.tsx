import { useState } from "react";
import type { ReactElement } from "react";
import { commands, NativeCommandError } from "@/bridge";
import type { ApplicationPreference } from "@/bridge/generated/bindings";
import { CharacterSettings } from "@/route/business/component/CharacterSettings";
import { ShortcutSettings } from "@/route/business/component/ShortcutSettings";
import { usePreference } from "@/route/business/preference-context";
import { hasShortcutConflict } from "@/route/business/shortcut";
import { Button } from "@/shared/component/Button";
import { Dialog } from "@/shared/component/Dialog";

type PreferenceDialogProps = {
  onClose: () => void;
  initialTab?: "general" | "character" | "shortcut";
};

export function PreferenceDialog({
  onClose,
  initialTab = "general",
}: PreferenceDialogProps): ReactElement {
  const source = usePreference();
  const [draft, setDraft] = useState(source.preference);
  const [tab, setTab] = useState<"general" | "character" | "shortcut">(
    initialTab,
  );
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [pending, setPending] = useState<ApplicationPreference | null>(null);
  const locked = busy || pending !== null;
  const invalid =
    hasShortcutConflict(draft.shortcut) ||
    draft.special_character.some((character) => character.text.trim() === "");

  async function save(): Promise<void> {
    setBusy(true);
    setError("");
    try {
      await source.save(pending ?? draft);
      setPending(null);
      onClose();
    } catch (cause) {
      if (
        !(cause instanceof NativeCommandError) ||
        cause.detail.recovery === "wait_for_confirmation"
      ) {
        setPending(pending ?? draft);
      }
      setError(
        cause instanceof NativeCommandError
          ? cause.detail.message
          : "设置保存未完成，请保留当前窗口并重试。",
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog
      open={true}
      onOpenChange={(open) => {
        if (!open && !locked) {
          onClose();
        }
      }}
      title="应用设置"
      description=""
      className="max-w-xl"
    >
      <nav
        aria-label="设置分类"
        className="flex gap-1 border-b border-border pb-3"
      >
        <Button
          variant={tab === "general" ? "secondary" : "ghost"}
          onClick={() => {
            setTab("general");
          }}
        >
          常规
        </Button>
        <Button
          variant={tab === "character" ? "secondary" : "ghost"}
          onClick={() => {
            setTab("character");
          }}
        >
          特殊字符
        </Button>
        <Button
          variant={tab === "shortcut" ? "secondary" : "ghost"}
          onClick={() => {
            setTab("shortcut");
          }}
        >
          快捷键
        </Button>
      </nav>
      {tab === "general" && (
        <div className="space-y-5">
          <label className="flex items-center justify-between gap-3 text-sm">
            选中单元时自动定位图片
            <input
              aria-label="选中单元时自动定位图片"
              type="checkbox"
              checked={draft.relocation_enabled}
              disabled={locked}
              onChange={(event) => {
                setDraft({
                  ...draft,
                  relocation_enabled: event.currentTarget.checked,
                });
              }}
            />
          </label>
          <label className="flex items-center justify-between gap-3 text-sm">
            标记透明度
            <input
              aria-label="标记透明度"
              type="range"
              min="0.05"
              max="1"
              step="0.05"
              value={draft.marker_opacity}
              disabled={locked}
              onChange={(event) => {
                setDraft({
                  ...draft,
                  marker_opacity: event.currentTarget.valueAsNumber,
                });
              }}
            />
          </label>
        </div>
      )}
      {tab === "character" && (
        <CharacterSettings
          value={draft.special_character}
          disabled={locked}
          onChange={(special_character) => {
            setDraft({ ...draft, special_character });
          }}
        />
      )}
      {tab === "shortcut" && (
        <ShortcutSettings
          value={draft.shortcut}
          disabled={locked}
          onChange={(shortcut) => {
            setDraft({ ...draft, shortcut });
          }}
        />
      )}
      {error !== "" && (
        <p role="alert" className="text-xs text-destructive">
          {error}
        </p>
      )}
      <div className="flex justify-end gap-2 border-t border-border pt-4">
        <Button
          variant="ghost"
          disabled={locked}
          onClick={() => {
            setBusy(true);
            commands
              .getDefaultPreference()
              .then(
                (value) => {
                  setDraft(value);
                  setError("");
                },
                () => {
                  setError("默认设置无法读取，请重试。");
                },
              )
              .finally(() => {
                setBusy(false);
              })
              .catch(() => {
                setError("默认设置无法读取，请重试。");
              });
          }}
        >
          恢复默认
        </Button>
        <Button variant="ghost" disabled={locked} onClick={onClose}>
          取消
        </Button>
        <Button
          variant="primary"
          disabled={busy || invalid}
          onClick={() => {
            save().catch(() => {
              setError("设置保存未完成，请重试。");
            });
          }}
        >
          {busy ? "正在保存…" : pending !== null ? "核实保存结果" : "保存设置"}
        </Button>
      </div>
    </Dialog>
  );
}
