import {
  CaseSensitive,
  Command,
  Menu,
  ReplaceAll,
  Settings,
  Star,
} from "lucide-react";
import { useCallback, useRef, useState } from "react";
import type { ReactElement } from "react";
import { IconButton } from "@/shared/component/IconButton";
import { useDismiss } from "@/shared/hook/use-dismiss";

type EditorToolboxProps = {
  readonly: boolean;
  locked: boolean;
  flagged: boolean;
  onFlagged: () => void;
  onSettings: (tab: "general" | "character" | "shortcut") => void;
  onSearch: () => void;
};

export function EditorToolbox(props: EditorToolboxProps): ReactElement {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const close = useCallback(() => {
    setOpen(false);
  }, []);
  useDismiss(open, ref, close);
  const options = [
    ...(!props.readonly
      ? [
          {
            label: "快捷键",
            icon: Command,
            run: () => {
              props.onSettings("shortcut");
            },
          },
          {
            label: "特殊字符",
            icon: CaseSensitive,
            run: () => {
              props.onSettings("character");
            },
          },
          { label: "搜索与替换", icon: ReplaceAll, run: props.onSearch },
        ]
      : []),
    {
      label: props.flagged ? "显示全部单元" : "只看关注单元",
      icon: Star,
      run: props.onFlagged,
    },
    {
      label: "应用设置",
      icon: Settings,
      run: () => {
        props.onSettings("general");
      },
    },
  ];
  return (
    <div ref={ref} className="relative opacity-85">
      <IconButton
        label="工具菜单"
        aria-expanded={open}
        aria-controls="editor-tool-menu"
        className={`translator-floating-button ${open ? "bg-green-50" : ""}`}
        onClick={() => {
          setOpen(!open);
        }}
      >
        <Menu size={16} strokeWidth={3} aria-hidden="true" />
      </IconButton>
      {open && (
        <div
          id="editor-tool-menu"
          className="absolute top-full left-0 z-50 mt-3 w-8 overflow-hidden rounded-lg border border-gray-100 bg-white shadow-xl"
        >
          <div className="flex flex-col divide-y divide-gray-50">
            {options.map((option) => (
              <IconButton
                key={option.label}
                label={option.label}
                disabled={props.locked}
                aria-pressed={option.icon === Star ? props.flagged : undefined}
                className={`rounded-none text-gray-700 hover:bg-gray-50 ${option.icon === Star && props.flagged ? "text-status-flag" : ""}`}
                onClick={() => {
                  option.run();
                  close();
                }}
              >
                <option.icon size={20} aria-hidden="true" />
              </IconButton>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
