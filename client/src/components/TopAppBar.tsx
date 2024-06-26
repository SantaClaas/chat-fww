import { JSX, VoidProps } from "solid-js";

/** A top app bar component inspired by Material You */
export default function TopAppBar({
  header,
  leadingAction,
  trailingAction,
}: VoidProps<{
  header: string;
  leadingAction?: JSX.Element;
  trailingAction?: JSX.Element;
}>) {
  return (
    <header class="px-1 min-h-16 py-2 grid grid-cols-[3rem,1fr,3rem] gap-1 bg-slate-100">
      {leadingAction}
      <h1 class="text-xl col-start-2 font-normal content-center text-center">
        {header}
      </h1>
      {trailingAction}
    </header>
  );
}
