import { useCallback, useEffect, useState } from "react";

type ResourceState<T> =
  | { status: "loading" }
  | { status: "ready"; value: T }
  | { status: "error" };

type Resource<T> = { state: ResourceState<T>; reload: () => void };

type ResourceSnapshot<T> = {
  load: () => Promise<T>;
  generation: number;
  state: ResourceState<T>;
};

export function useResource<T>(load: () => Promise<T>): Resource<T> {
  const [snapshot, setSnapshot] = useState<ResourceSnapshot<T>>({
    load,
    generation: 0,
    state: { status: "loading" },
  });
  const [generation, setGeneration] = useState(0);
  const reload = useCallback(() => {
    setGeneration((value) => value + 1);
  }, []);

  useEffect(() => {
    let active = true;
    load().then(
      (value) => {
        if (active) {
          setSnapshot({ load, generation, state: { status: "ready", value } });
        }
      },
      () => {
        if (active) {
          setSnapshot({ load, generation, state: { status: "error" } });
        }
      },
    );
    return () => {
      active = false;
    };
  }, [load, generation]);

  const state: ResourceState<T> =
    snapshot.load === load && snapshot.generation === generation
      ? snapshot.state
      : { status: "loading" };
  return { state, reload };
}
