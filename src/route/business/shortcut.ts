import type {
  Shortcut,
  ShortcutAction,
  ShortcutBinding,
} from "@/bridge/generated/bindings";

export const shortcutLabel: Record<ShortcutAction, string> = {
  save: "保存",
  cycle_mode: "切换模式",
  toggle_relocation: "启用重定位",
  previous_unit: "上一个标记",
  next_unit: "下一个标记",
  previous_page: "上一页",
  next_page: "下一页",
  toggle_proofread_preview: "切换标记透明度",
  insert_recent_symbol: "输入最近一次符号",
  insert_favorite_one: "输入优选符号#1",
  insert_favorite_two: "输入优选符号#2",
  insert_favorite_three: "输入优选符号#3",
};

export const fixedShortcuts = [
  ["创建框内单元", "左键图片空白处"],
  ["创建框外单元", "右键图片空白处"],
  ["聚焦单元", "左键已有单元标记"],
  ["删除单元", "右键已有单元标记"],
  ["切换框内外", "左键双击已有标记"],
];

export function formatShortcut(binding: ShortcutBinding): string {
  if (binding.kind === "unbound") {
    return "未绑定";
  }
  return [
    binding.control ? "Ctrl" : "",
    binding.meta ? "Cmd" : "",
    binding.alt ? "Alt" : "",
    binding.shift ? "Shift" : "",
    binding.key.replace(/^Key|^Digit/u, ""),
  ]
    .filter((part) => part !== "")
    .join(" + ");
}

export function hasShortcutConflict(shortcuts: Shortcut[]): boolean {
  const seen = new Set<string>();
  for (const shortcut of shortcuts) {
    if (shortcut.binding.kind === "unbound") {
      continue;
    }
    const binding = shortcut.binding;
    const key = [
      binding.key,
      binding.control,
      binding.meta,
      binding.alt,
      binding.shift,
    ].join("|");
    if (seen.has(key)) {
      return true;
    }
    seen.add(key);
  }
  return false;
}
