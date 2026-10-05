import {
  CheckCheck,
  CircleSlash,
  Eye,
  FileType,
  Image,
  Loader2,
  Lock,
  MapPin,
  Save,
} from "lucide-react";
import type { ComponentProps, ReactElement } from "react";
import type { EditorSession, EditorState } from "./editor-session";
import { isDirty } from "./editor-session";

type ToolbarButtonProps = ComponentProps<"button"> & {
  label: string;
  active: boolean;
};

function ToolbarButton({
  label,
  active,
  className = "",
  ...props
}: ToolbarButtonProps): ReactElement {
  return (
    <button
      type="button"
      title={label}
      aria-label={label}
      aria-pressed={active}
      className={`flex flex-1 items-center justify-center py-2 text-stone-600 shadow-[inset_0_1px_0_rgba(255,255,255,0.75)] transition-colors disabled:cursor-not-allowed disabled:opacity-40 ${active ? "bg-green-50 hover:bg-green-100" : "bg-white hover:bg-stone-100"} ${className}`}
      {...props}
    />
  );
}

type EditorToolbarProps = {
  state: EditorState;
  session: EditorSession;
  relocation: boolean;
  onRelocation: () => void;
  preview: boolean;
  onPreview: () => void;
  highQuality: boolean;
  onQuality: () => void;
  creationEnabled: boolean;
  onCreation: () => void;
};

export function EditorToolbar({
  state,
  session,
  relocation,
  onRelocation,
  preview,
  onPreview,
  highQuality,
  onQuality,
  creationEnabled,
  onCreation,
}: EditorToolbarProps): ReactElement {
  const label =
    state.mode === "translation"
      ? "翻译"
      : state.mode === "proofreading"
        ? "校对"
        : "只读";
  const next =
    state.mode === "translation"
      ? "proofreading"
      : state.mode === "proofreading"
        ? "readonly"
        : "translation";
  const status = state.busy
    ? "保存中"
    : state.error
      ? "保存失败，修改已保留"
      : isDirty(state)
        ? "待保存"
        : "已保存";
  return (
    <div className="flex shrink-0 items-center border-b-2 border-unit-border bg-panel">
      <div className="flex w-full divide-x divide-stone-200">
        <ToolbarButton
          label={`当前${label}模式，切换模式`}
          active={true}
          disabled={state.locked || state.uncertain}
          onClick={() => {
            session.setMode(next);
          }}
        >
          {state.mode === "translation" ? (
            <FileType size={18} aria-hidden="true" />
          ) : state.mode === "proofreading" ? (
            <CheckCheck size={18} aria-hidden="true" />
          ) : (
            <Lock size={18} aria-hidden="true" />
          )}
        </ToolbarButton>
        <ToolbarButton
          label="切换重定位模式"
          active={relocation}
          onClick={onRelocation}
        >
          <MapPin size={18} aria-hidden="true" />
        </ToolbarButton>
        {state.mode !== "readonly" && (
          <>
            <ToolbarButton
              label={creationEnabled ? "禁用标记创建" : "启用标记创建"}
              active={!creationEnabled}
              onClick={onCreation}
              className="hidden [@media(any-pointer:coarse)]:flex"
            >
              <CircleSlash size={18} aria-hidden="true" />
            </ToolbarButton>
            <ToolbarButton
              label={`保存，${status}`}
              active={false}
              disabled={state.locked || state.busy}
              onClick={() => {
                void session.save();
              }}
            >
              {state.busy ? (
                <Loader2
                  size={18}
                  className="animate-spin"
                  aria-hidden="true"
                />
              ) : (
                <Save size={18} aria-hidden="true" />
              )}
            </ToolbarButton>
          </>
        )}
        <ToolbarButton
          label={highQuality ? "切换到预览图片" : "切换到高清图块"}
          active={highQuality}
          disabled={state.locked}
          onClick={onQuality}
        >
          <Image size={18} aria-hidden="true" />
        </ToolbarButton>
        <ToolbarButton
          label={preview ? "降低标记透明度" : "恢复标记透明度"}
          active={preview}
          onClick={onPreview}
        >
          <Eye size={18} aria-hidden="true" />
        </ToolbarButton>
      </div>
      <span role="status" className="sr-only">
        {status}
      </span>
    </div>
  );
}
