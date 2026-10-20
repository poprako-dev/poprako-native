import type { ComponentProps, ReactElement } from "react";
import type { EditorSession } from "./editor-session";
import { AutoResizeTextarea } from "./AutoResizeTextarea";
import { useComposedText } from "./use-composed-text";

type Props = Omit<
  ComponentProps<typeof AutoResizeTextarea>,
  "onChange" | "onCompositionStart" | "onCompositionEnd"
> & {
  session: EditorSession;
  onTextChange: (value: string) => void;
};

export function UnitTextarea({
  value,
  session,
  onTextChange,
  ...props
}: Props): ReactElement {
  const text = useComposedText({ value, session, onChange: onTextChange });
  return (
    <AutoResizeTextarea
      {...props}
      value={text.value}
      onChange={(event) => {
        text.change(event.currentTarget.value);
      }}
      onCompositionStart={text.start}
      onCompositionEnd={(event) => {
        text.end(event.currentTarget.value);
      }}
    />
  );
}
