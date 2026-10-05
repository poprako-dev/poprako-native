import { useEffect, useState } from "react";
import type { ReactElement } from "react";
import { commands, NativeCommandError, unwrap } from "@/bridge";
import { registerExitGuard } from "@/bridge/exit";
import type {
  Replacement,
  ReplaceUnits,
  SearchHit,
  TextStage,
} from "@/bridge/generated/bindings";
import { Button } from "@/shared/component/Button";
import { Dialog } from "@/shared/component/Dialog";
import type { EditorSession } from "./editor-session";

type SearchDialogProps = {
  session: EditorSession;
  onRefresh: () => Promise<void>;
  onClose: () => void;
};
type Phase = "idle" | "searching" | "replacing" | "refresh" | "uncertain";

export function SearchDialog({
  session,
  onRefresh,
  onClose,
}: SearchDialogProps): ReactElement {
  const [query, setQuery] = useState("");
  const [stage, setStage] = useState<TextStage>("translation");
  const [hits, setHits] = useState<SearchHit[]>([]);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [rules, setRules] = useState<Replacement[]>([
    { origin: "", target: "" },
  ]);
  const [phase, setPhase] = useState<Phase>("idle");
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");
  const [pending, setPending] = useState<ReplaceUnits | null>(null);
  const busy = phase !== "idle";
  const comicId = session.store.getState().baseline.page.comic_id;
  useEffect(() => registerExitGuard(() => Promise.resolve(!busy)), [busy]);

  async function search(): Promise<void> {
    setPhase("searching");
    setError("");
    try {
      if (!(await session.flush()))
        throw new Error(
          session.store.getState().error || "请结束当前输入后重试保存。",
        );
      const found = unwrap(
        await commands.searchUnits({ comic_id: comicId, stage, query }),
      );
      setHits(found);
      setSelected(new Set(found.map((hit) => hit.unit_id)));
      setStatus(`找到 ${String(found.length)} 个单元`);
      setRules([{ origin: query.trim(), target: "" }]);
    } catch (failure) {
      setError(
        failure instanceof Error ? failure.message : "搜索失败，请重试。",
      );
    } finally {
      setPhase("idle");
    }
  }

  async function refresh(): Promise<void> {
    setPhase("refresh");
    try {
      await onRefresh();
      const found = unwrap(
        await commands.searchUnits({ comic_id: comicId, stage, query }),
      );
      setHits(found);
      setSelected(new Set());
      setError("");
      setPhase("idle");
    } catch {
      setError("修改已保存，但刷新失败。请重试刷新，不要再次替换。");
    }
  }

  async function replace(): Promise<void> {
    const request = pending ?? {
      comic_id: comicId,
      stage,
      units: hits
        .filter((hit) => selected.has(hit.unit_id))
        .map((hit) => ({ hit, rules })),
    };
    setPending(request);
    setPhase("replacing");
    setError("");
    try {
      if (!(await session.flush()))
        throw new Error(session.store.getState().error || "当前内容尚未保存。");
      const result = unwrap(await commands.replaceUnits(request));
      setPending(null);
      setStatus(
        `已修改 ${String(result.changed_unit_count)} 个单元，校对确认状态保持不变。`,
      );
      await refresh();
    } catch (failure) {
      const uncertain =
        failure instanceof NativeCommandError &&
        failure.detail.recovery === "wait_for_confirmation";
      setError(
        failure instanceof Error
          ? failure.message
          : "替换失败，请刷新搜索结果后重试。",
      );
      setPhase(uncertain ? "uncertain" : "idle");
      if (!uncertain) setPending(null);
    }
  }

  function close(): void {
    if (busy) return;
    session.cancelLeave();
    onClose();
  }

  return (
    <Dialog
      open={true}
      onOpenChange={(value) => {
        if (!value) close();
      }}
      title="搜索与替换"
      description=""
      className="max-w-2xl"
    >
      <div className="flex gap-2">
        <select
          aria-label="搜索文本阶段"
          className="rounded border border-border bg-background p-2 text-sm"
          value={stage}
          disabled={busy}
          onChange={(event) => {
            const value = event.target.value;
            if (value === "translation" || value === "proofreading") {
              setStage(value);
              setHits([]);
              setSelected(new Set());
            }
          }}
        >
          <option value="translation">翻译文本</option>
          <option value="proofreading">校对文本</option>
        </select>
        <input
          aria-label="搜索内容"
          className="min-w-0 flex-1 rounded border border-border p-2 text-sm"
          value={query}
          disabled={busy}
          onChange={(event) => {
            setQuery(event.target.value);
            setHits([]);
            setSelected(new Set());
          }}
          onKeyDown={(event) => {
            if (
              event.key === "Enter" &&
              !event.nativeEvent.isComposing &&
              !busy &&
              query.trim()
            )
              void search();
          }}
        />
        <Button
          variant="primary"
          disabled={busy || !query.trim()}
          onClick={() => {
            void search();
          }}
        >
          搜索
        </Button>
      </div>
      <p role="status" className="text-xs text-muted-foreground">
        {status}
      </p>
      {error && (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}
      <div className="max-h-64 overflow-auto rounded border border-border">
        {hits.map((hit) => (
          <label
            key={hit.unit_id}
            className="flex gap-2 border-b border-border p-2 text-sm last:border-0"
          >
            <input
              type="checkbox"
              checked={selected.has(hit.unit_id)}
              disabled={busy}
              onChange={(event) => {
                const next = new Set(selected);
                if (event.target.checked) next.add(hit.unit_id);
                else next.delete(hit.unit_id);
                setSelected(next);
              }}
            />
            <span className="shrink-0 text-xs text-muted-foreground">
              P{hit.page_index + 1} · {hit.unit_index + 1}
            </span>
            <span className="whitespace-pre-wrap">{hit.text}</span>
          </label>
        ))}
      </div>
      <fieldset disabled={busy} className="space-y-2">
        <legend className="mb-2 text-xs">依次应用替换规则（最多 20 条）</legend>
        {rules.map((rule, index) => (
          <div key={index} className="flex items-center gap-2">
            <input
              aria-label={`规则 ${String(index + 1)} 原文`}
              className="min-w-0 flex-1 rounded border border-border p-2 text-sm"
              value={rule.origin}
              onChange={(event) => {
                setRules(
                  rules.map((entry, at) =>
                    at === index
                      ? { ...entry, origin: event.target.value }
                      : entry,
                  ),
                );
              }}
            />
            <span>→</span>
            <input
              aria-label={`规则 ${String(index + 1)} 替换文本`}
              className="min-w-0 flex-1 rounded border border-border p-2 text-sm"
              value={rule.target}
              onChange={(event) => {
                setRules(
                  rules.map((entry, at) =>
                    at === index
                      ? { ...entry, target: event.target.value }
                      : entry,
                  ),
                );
              }}
            />
            <Button
              variant="ghost"
              disabled={rules.length === 1}
              onClick={() => {
                setRules(rules.filter((_, at) => at !== index));
              }}
            >
              移除
            </Button>
          </div>
        ))}
        <Button
          variant="secondary"
          disabled={rules.length >= 20}
          onClick={() => {
            setRules([...rules, { origin: "", target: "" }]);
          }}
        >
          增加规则
        </Button>
      </fieldset>
      <div className="flex justify-end gap-2">
        <Button variant="secondary" disabled={busy} onClick={close}>
          关闭
        </Button>
        {phase === "refresh" ? (
          <Button
            variant="primary"
            onClick={() => {
              void refresh();
            }}
          >
            重试刷新
          </Button>
        ) : (
          <Button
            variant="primary"
            disabled={
              session.store.getState().mode === "readonly" ||
              (busy && phase !== "uncertain") ||
              selected.size === 0 ||
              rules.some((rule) => !rule.origin)
            }
            onClick={() => {
              void replace();
            }}
          >
            {phase === "uncertain"
              ? "核实替换结果"
              : `替换所选 ${String(selected.size)} 个单元`}
          </Button>
        )}
      </div>
    </Dialog>
  );
}
