import type { ReactElement } from "react";
import { CirclePlus, Search } from "lucide-react";
import { IconButton } from "@/shared/component/IconButton";

type Props = {
  query: string;
  label: string;
  createLabel: string;
  canCreate: boolean;
  onQuery: (query: string) => void;
  onCreate: () => void;
};

export function TerminologyToolbar(props: Props): ReactElement {
  return (
    <div className="mb-2 flex h-10 shrink-0 items-center gap-2">
      <label className="relative min-w-0 flex-1">
        <Search
          size={18}
          className="pointer-events-none absolute top-2 left-3 text-muted-foreground"
          aria-hidden="true"
        />
        <input
          value={props.query}
          onChange={(event) => {
            props.onQuery(event.target.value);
          }}
          aria-label={props.label}
          placeholder={props.label}
          className="h-8 w-full rounded-md border border-border bg-card py-1 pr-3 pl-9 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
        />
      </label>
      {props.canCreate && (
        <IconButton
          label={props.createLabel}
          onClick={props.onCreate}
          className="border border-border bg-card"
        >
          <CirclePlus size={20} strokeWidth={2} />
        </IconButton>
      )}
    </div>
  );
}
