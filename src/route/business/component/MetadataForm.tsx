import { useId, useState } from "react";
import type { ReactElement } from "react";
import { Button } from "@/shared/component/Button";

type MetadataFormProps = {
  title: string;
  subtitle: string;
  author: string;
  busy: boolean;
  error: string;
  submitLabel: string;
  onSubmit: (title: string, subtitle: string, author: string) => void;
  onCancel: () => void;
};

export function MetadataForm(props: MetadataFormProps): ReactElement {
  const id = useId();
  const [title, setTitle] = useState(props.title);
  const [subtitle, setSubtitle] = useState(props.subtitle);
  const [author, setAuthor] = useState(props.author);
  const inputClass =
    "h-9 w-full rounded-md border border-border bg-background px-3 text-sm focus-visible:outline-2 focus-visible:outline-ring";
  return (
    <form
      className="space-y-4"
      onSubmit={(event) => {
        event.preventDefault();
        if (!props.busy && title.trim() !== "") {
          props.onSubmit(title, subtitle, author);
        }
      }}
    >
      <div className="space-y-1.5">
        <label htmlFor={`${id}-title`} className="text-xs font-medium">
          标题
        </label>
        <input
          id={`${id}-title`}
          className={inputClass}
          value={title}
          onChange={(event) => {
            setTitle(event.currentTarget.value);
          }}
          required
          disabled={props.busy}
        />
      </div>
      <div className="space-y-1.5">
        <label htmlFor={`${id}-subtitle`} className="text-xs font-medium">
          副标题
        </label>
        <input
          id={`${id}-subtitle`}
          className={inputClass}
          value={subtitle}
          onChange={(event) => {
            setSubtitle(event.currentTarget.value);
          }}
          disabled={props.busy}
        />
      </div>
      <div className="space-y-1.5">
        <label htmlFor={`${id}-author`} className="text-xs font-medium">
          作者
        </label>
        <input
          id={`${id}-author`}
          className={inputClass}
          value={author}
          onChange={(event) => {
            setAuthor(event.currentTarget.value);
          }}
          disabled={props.busy}
        />
      </div>
      {props.error !== "" && (
        <p className="text-xs text-destructive" role="alert">
          {props.error}
        </p>
      )}
      <div className="flex justify-end gap-2">
        <Button variant="ghost" onClick={props.onCancel} disabled={props.busy}>
          取消
        </Button>
        <Button
          variant="primary"
          type="submit"
          disabled={props.busy || title.trim() === ""}
        >
          {props.busy ? "正在保存…" : props.submitLabel}
        </Button>
      </div>
    </form>
  );
}
