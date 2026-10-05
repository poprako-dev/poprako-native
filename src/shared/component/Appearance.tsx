import { createContext, useContext } from "react";
import type { ReactElement, ReactNode } from "react";

type Surface = "native" | "translator";
type AppearanceProps = {
  surface: Surface;
  className: string;
  children: ReactNode;
};
const SurfaceContext = createContext<Surface>("native");

export function useSurface(): Surface {
  return useContext(SurfaceContext);
}

export function Appearance({
  surface,
  className,
  children,
}: AppearanceProps): ReactElement {
  return (
    <SurfaceContext value={surface}>
      <div data-ui-surface={surface} className={className}>
        {children}
      </div>
    </SurfaceContext>
  );
}
